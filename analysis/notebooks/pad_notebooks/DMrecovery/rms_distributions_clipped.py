import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
from full_recovery_v1 import quiet_mask, detect_event_shifts, residual_cut, FLIGHT_CALIB

import gondola as gon
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

calib = gon.calibration.load_rb_calibrations(FLIGHT_CALIB)
MANGLING = {gon.events.EventStatus.ChnSyncErrors, gon.events.EventStatus.CellSyncErrors, gon.events.EventStatus.CellAndChnSyncErrors}
GOOD = {gon.events.EventStatus.Perfect, gon.events.EventStatus.GoodNoCRCCheck,
        gon.events.EventStatus.GoodNoCRCOrErrBitCheck, gon.events.EventStatus.GoodNoErrBitCheck}

files = [l.strip() for l in open('run_lists/flight_30min.txt') if l.strip() and not l.startswith('#')]

RMS_LO, RMS_HI = 700, 900
CLIP_HI = 8.0  # mV upper limit only -- clamp samples above +8 (no lower bound), so an
               # in-window real pulse (which only ever swings positive here) can't
               # blow up the metric, without touching the negative side at all


def mad_std(window):
    """Robust std via median absolute deviation -- no threshold to tune, and has a
    50% breakdown point, so it's automatically insensitive to an in-window pulse as
    long as the pulse doesn't dominate more than half the window's samples."""
    med = np.median(window)
    mad = np.median(np.abs(window - med))
    return 1.4826 * mad  # scale factor makes this consistent with std for Gaussian noise


good_rms = []
good_madstd_rms = []
mangled_naive_rms = []
mangled_corrected_quietmask_rms = []
mangled_corrected_700to900_rms = []          # unclipped, for reference (not plotted)
mangled_corrected_700to900_clipped_rms = []  # clipped +8mV, no lower bound
mangled_corrected_700to900_madstd_rms = []   # NEW: robust MAD-based std, no threshold

MAX_GOOD = 30000
MAX_MANGLED = 30000
n_good = 0
n_mangled = 0
n_seen_mangled = 0

for f in files:
    if n_good >= MAX_GOOD and n_mangled >= MAX_MANGLED:
        break
    reader = gon.io.TofPacketReader(str(f), filter=gon.packets.TofPacketType.TofEvent)
    for pack in reader:
        if n_good >= MAX_GOOD and n_mangled >= MAX_MANGLED:
            break
        ev = gon.events.TofEvent.from_bytestream(pack.payload, 0)
        for rb in ev.rb_events:
            rid = rb.header.rb_id
            if rid not in calib:
                continue
            rbcal = calib[rid]
            stop_cell = rb.header.stop_cell
            active_1idx = sorted(c + 1 for c in rb.header.get_channels())
            non9 = [c for c in active_1idx if c != 9]
            if len(non9) < 1:
                continue

            if rb.status in GOOD and n_good < MAX_GOOD:
                ch = non9[0]
                raw = rb.get_waveform(ch)
                cal = np.asarray(rbcal.voltages(ch, stop_cell, raw))
                window = cal[RMS_LO:RMS_HI]
                good_rms.append(np.std(np.clip(window, None, CLIP_HI)))
                good_madstd_rms.append(mad_std(window))
                n_good += 1

            elif rb.status in MANGLING and n_mangled < MAX_MANGLED:
                if len(non9) < 2:
                    continue
                n_seen_mangled += 1
                shifts, anchor_info = detect_event_shifts(rb, rbcal, non9, active_1idx)
                if shifts is None:
                    continue
                ch, ratio = anchor_info
                raw = rb.get_waveform(ch)
                raw_f = np.asarray(raw, dtype=np.float64)

                shift = shifts[ch]
                cut = residual_cut(raw, ch, rbcal, stop_cell, shift)
                if cut >= RMS_LO:
                    continue

                qmask_window = quiet_mask(raw_f)[RMS_LO:RMS_HI]
                if qmask_window.sum() < 30:
                    continue

                naive_cal = np.asarray(rbcal.voltages(ch, stop_cell, raw))
                csc = (stop_cell - shift) % 1024
                corr_cal = np.asarray(rbcal.voltages(ch, csc, raw))
                window = corr_cal[RMS_LO:RMS_HI]

                mangled_naive_rms.append(np.std(naive_cal[RMS_LO:RMS_HI]))
                mangled_corrected_quietmask_rms.append(np.std(window[qmask_window]))
                mangled_corrected_700to900_rms.append(np.std(window))
                mangled_corrected_700to900_clipped_rms.append(np.std(np.clip(window, None, CLIP_HI)))
                mangled_corrected_700to900_madstd_rms.append(mad_std(window))
                n_mangled += 1

        if n_seen_mangled % 200 == 0 and n_seen_mangled > 0:
            print(f"...progress: seen_mangled={n_seen_mangled} kept_mangled={n_mangled} n_good={n_good}", flush=True)

np.savez("DMrecovery/rms_distributions_clipped_data.npz",
         good_rms=good_rms, good_madstd_rms=good_madstd_rms,
         mangled_naive_rms=mangled_naive_rms,
         mangled_corrected_quietmask_rms=mangled_corrected_quietmask_rms,
         mangled_corrected_700to900_rms=mangled_corrected_700to900_rms,
         mangled_corrected_700to900_clipped_rms=mangled_corrected_700to900_clipped_rms,
         mangled_corrected_700to900_madstd_rms=mangled_corrected_700to900_madstd_rms)

print(f"\nFINAL: n_good={len(good_rms)} n_mangled={len(mangled_corrected_700to900_rms)} (seen {n_seen_mangled} mangled events total)")
for vals, label in [(good_rms, "good (clipped)"), (good_madstd_rms, "good (mad_std)"),
                     (mangled_naive_rms, "mangled naive"),
                     (mangled_corrected_quietmask_rms, "mangled corrected quiet_mask"),
                     (mangled_corrected_700to900_rms, "mangled corrected 700:900 (unclipped)"),
                     (mangled_corrected_700to900_clipped_rms, "mangled corrected 700:900 (clipped, +8 only)"),
                     (mangled_corrected_700to900_madstd_rms, "mangled corrected 700:900 (mad_std)")]:
    print(f"{label}: mean={np.mean(vals):.3f} median={np.median(vals):.3f}")

def true_density(vals, bins):
    counts, edges = np.histogram(vals, bins=bins)
    width = edges[1] - edges[0]
    return counts / (len(vals) * width), edges

XLO, XHI = 0, 20
bins = np.linspace(XLO, XHI, 101)
fig, ax = plt.subplots(figsize=(11, 6.5))
pops = [
    (good_rms, "Good", "tab:green"),
    (mangled_corrected_700to900_clipped_rms, "Mangled, corrected, [700:900] clipped +8mV (no lower bound)", "tab:purple"),
    (mangled_corrected_700to900_madstd_rms, "Mangled, corrected, [700:900] mad_std (no threshold)", "tab:brown"),
    (mangled_corrected_quietmask_rms, "Mangled, corrected, quiet_mask", "tab:blue"),
]
for vals, label, color in pops:
    dens, edges = true_density(vals, bins)
    frac_outside = np.mean((np.array(vals) < XLO) | (np.array(vals) > XHI))
    ax.stairs(dens, edges, color=color, alpha=0.15, fill=True)
    ax.stairs(dens, edges, color=color, linewidth=1.8,
              label=f"{label} (n={len(vals)}, {100*frac_outside:.1f}% outside)")
ax.set_xlim(XLO, XHI)
ax.set_xlabel(f"baseline std, window [{RMS_LO}:{RMS_HI}], calibrated units")
ax.set_ylabel("density (normalized to full population)")
ax.legend(fontsize=9)
ax.set_title(f"quiet_mask vs clipped +{CLIP_HI}mV vs mad_std -- pulse-contamination fixes for [700:900] (n={MAX_GOOD}/{MAX_MANGLED})")
plt.tight_layout()
plt.savefig("DMrecovery/rms_distributions_clipped.png", dpi=120)
print("saved DMrecovery/rms_distributions_clipped.png")
