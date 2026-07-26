import sys
sys.path.insert(0, '/tmp/claude-1000/-home-gaps-software-padrick-dev-test-gaps-os-pro-analysis-notebooks-pad-notebooks/da138849-a795-44a3-b8cf-c9fdd0641ad9/scratchpad')
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

files = [l.strip() for l in open('run_lists/flight_small.txt') if l.strip() and not l.startswith('#')]

# window is relative to stop_cell: since sample i of the array already IS cell
# (stop_cell + i) % 1024, "stop_cell to stop_cell+80" is simply array indices [0:80].
WIN_LEN = 80
CLIP = 10.0

good_rms = []
mangled_corrected_quietmask_rms = []
mangled_corrected_clipped_rms = []

MAX_GOOD = 4000
MAX_MANGLED = 4000
n_good = 0
n_mangled = 0
n_seen_mangled = 0
n_skipped_cut_too_late = 0
cuts_seen = []

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
                good_rms.append(np.std(cal[0:WIN_LEN]))
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
                cuts_seen.append(cut)
                lo = cut + 1
                if lo >= WIN_LEN - 10:  # need at least 10 usable samples in-window
                    n_skipped_cut_too_late += 1
                    continue

                csc = (stop_cell - shift) % 1024
                corr_cal = np.asarray(rbcal.voltages(ch, csc, raw))
                window = corr_cal[lo:WIN_LEN]

                qmask_window = quiet_mask(raw_f)[lo:WIN_LEN]
                if qmask_window.sum() < 10:
                    continue

                mangled_corrected_quietmask_rms.append(np.std(window[qmask_window]))
                mangled_corrected_clipped_rms.append(np.std(np.clip(window, -CLIP, CLIP)))
                n_mangled += 1

        if n_seen_mangled % 200 == 0 and n_seen_mangled > 0:
            print(f"...progress: seen_mangled={n_seen_mangled} kept_mangled={n_mangled} n_good={n_good} skipped_cut_too_late={n_skipped_cut_too_late}", flush=True)

print(f"\nFINAL: n_good={len(good_rms)} n_mangled={len(mangled_corrected_clipped_rms)} (seen {n_seen_mangled} mangled events, skipped {n_skipped_cut_too_late} for cut>={WIN_LEN-10})")
print(f"cut stats: mean={np.mean(cuts_seen):.1f} median={np.median(cuts_seen):.1f} max={np.max(cuts_seen):.0f} frac_cut>=70: {np.mean(np.array(cuts_seen)>=70):.3f}")
for vals, label in [(good_rms, "good"),
                     (mangled_corrected_quietmask_rms, "mangled corrected quiet_mask"),
                     (mangled_corrected_clipped_rms, "mangled corrected clipped +/-10")]:
    print(f"{label}: mean={np.mean(vals):.3f} median={np.median(vals):.3f}")

np.savez("DMrecovery/rms_distributions_stopcell80_data.npz",
         good_rms=good_rms, mangled_corrected_quietmask_rms=mangled_corrected_quietmask_rms,
         mangled_corrected_clipped_rms=mangled_corrected_clipped_rms, cuts_seen=cuts_seen)

def true_density(vals, bins):
    counts, edges = np.histogram(vals, bins=bins)
    width = edges[1] - edges[0]
    return counts / (len(vals) * width), edges

XLO, XHI = 0, 20
bins = np.linspace(XLO, XHI, 101)
fig, ax = plt.subplots(figsize=(11, 6.5))
pops = [
    (good_rms, "Good", "tab:green"),
    (mangled_corrected_clipped_rms, "Mangled, corrected, clipped +/-10mV", "tab:purple"),
    (mangled_corrected_quietmask_rms, "Mangled, corrected, quiet_mask", "tab:blue"),
]
for vals, label, color in pops:
    dens, edges = true_density(vals, bins)
    frac_outside = np.mean((np.array(vals) < XLO) | (np.array(vals) > XHI))
    ax.stairs(dens, edges, color=color, alpha=0.15, fill=True)
    ax.stairs(dens, edges, color=color, linewidth=1.8,
              label=f"{label} (n={len(vals)}, {100*frac_outside:.1f}% outside)")
ax.set_xlim(XLO, XHI)
ax.set_xlabel(f"baseline std, window [stop_cell : stop_cell+{WIN_LEN}] (post-cut), calibrated units")
ax.set_ylabel("density (normalized to full population)")
ax.legend(fontsize=9)
ax.set_title(f"RMS using [stop_cell:stop_cell+{WIN_LEN}] instead of [700:900] (n={MAX_GOOD}/{MAX_MANGLED})")
plt.tight_layout()
plt.savefig("DMrecovery/rms_distributions_stopcell80.png", dpi=120)
print("saved DMrecovery/rms_distributions_stopcell80.png")
