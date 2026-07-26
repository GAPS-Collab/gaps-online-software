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

RMS_LO, RMS_HI = 700, 900

good_rms = []
mangled_naive_rms = []
mangled_corrected_quietmask_rms = []
mangled_corrected_700to900_rms = []

MAX_GOOD = 10000
MAX_MANGLED = 10000
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
                good_rms.append(np.std(cal[RMS_LO:RMS_HI]))
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
                    continue  # corrupted segment reaches into the fixed RMS window, skip

                naive_cal = np.asarray(rbcal.voltages(ch, stop_cell, raw))
                csc = (stop_cell - shift) % 1024
                corr_cal = np.asarray(rbcal.voltages(ch, csc, raw))

                qmask_window = quiet_mask(raw_f)[RMS_LO:RMS_HI]
                if qmask_window.sum() < 30:
                    continue  # not enough quiet samples in-window to trust the quiet-mask metric

                mangled_naive_rms.append(np.std(naive_cal[RMS_LO:RMS_HI]))
                mangled_corrected_quietmask_rms.append(np.std(corr_cal[RMS_LO:RMS_HI][qmask_window]))
                mangled_corrected_700to900_rms.append(np.std(corr_cal[RMS_LO:RMS_HI]))
                n_mangled += 1

        if n_seen_mangled % 100 == 0 and n_seen_mangled > 0:
            print(f"...progress: seen_mangled={n_seen_mangled} kept_mangled={n_mangled} n_good={n_good}", flush=True)

print(f"\nFINAL: n_good={len(good_rms)} n_mangled={len(mangled_corrected_700to900_rms)} (seen {n_seen_mangled} mangled events total)")
print(f"good_rms: mean={np.mean(good_rms):.3f} median={np.median(good_rms):.3f}")
print(f"mangled_naive_rms: mean={np.mean(mangled_naive_rms):.3f} median={np.median(mangled_naive_rms):.3f}")
print(f"mangled_corrected_quietmask_rms: mean={np.mean(mangled_corrected_quietmask_rms):.3f} median={np.median(mangled_corrected_quietmask_rms):.3f}")
print(f"mangled_corrected_700to900_rms: mean={np.mean(mangled_corrected_700to900_rms):.3f} median={np.median(mangled_corrected_700to900_rms):.3f}")

np.savez("DMrecovery/rms_distributions_4pop_data.npz",
         good_rms=good_rms, mangled_naive_rms=mangled_naive_rms,
         mangled_corrected_quietmask_rms=mangled_corrected_quietmask_rms,
         mangled_corrected_700to900_rms=mangled_corrected_700to900_rms)
print("saved DMrecovery/rms_distributions_4pop_data.npz (raw arrays, for replotting without rerunning)")

XLO, XHI = 0, 25
bins = np.linspace(XLO, XHI, 126)  # bin width 0.2

fig, ax = plt.subplots(figsize=(11, 6.5))
pops = [
    (good_rms, "Good", "tab:green", "-"),
    (mangled_naive_rms, "Mangled, naive", "tab:red", "-"),
    (mangled_corrected_quietmask_rms, "Mangled, corrected, quiet_mask", "tab:blue", "-"),
    (mangled_corrected_700to900_rms, "Mangled, corrected, [700:900]", "tab:orange", "-"),
]
for vals, label, color, ls in pops:
    # soft filled backdrop so each population still reads as a "mass", plus a crisp
    # step outline on top so overlapping distributions stay distinguishable
    ax.hist(vals, bins=bins, histtype='stepfilled', color=color, alpha=0.15, density=True)
    ax.hist(vals, bins=bins, histtype='step', color=color, linestyle=ls, linewidth=1.8,
            label=f"{label} (n={len(vals)})", density=True)
ax.set_xlim(XLO, XHI)
ax.set_xlabel(f"baseline std, window [{RMS_LO}:{RMS_HI}], calibrated units")
ax.set_ylabel("density")
ax.legend()
ax.set_title("RMS distributions (n~1000 each): good vs mangled -- quiet_mask vs fixed-window correction")
plt.tight_layout()
plt.savefig("DMrecovery/rms_distributions_4pop.png", dpi=120)
print("saved DMrecovery/rms_distributions_4pop.png")
