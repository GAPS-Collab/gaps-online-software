import sys
sys.path.insert(0, '/tmp/claude-1000/-home-gaps-software-padrick-dev-test-gaps-os-pro-analysis-notebooks-pad-notebooks/da138849-a795-44a3-b8cf-c9fdd0641ad9/scratchpad')
from full_recovery_v1 import quiet_mask, full_shift_scan, detect_event_shifts, residual_cut, FLIGHT_CALIB

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


def rms(w):
    return np.std(w)

good_rms = []
mangled_naive_rms = []
mangled_corrected_rms = []

MAX_GOOD = 400
MAX_MANGLED = 60
n_good = 0
n_mangled = 0

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
                good_rms.append(rms(cal[RMS_LO:RMS_HI]))
                n_good += 1

            elif rb.status in MANGLING and n_mangled < MAX_MANGLED:
                if len(non9) < 2:
                    continue
                shifts, anchor_info = detect_event_shifts(rb, rbcal, non9, active_1idx)
                if shifts is None:
                    continue
                ch = non9[0]
                raw = rb.get_waveform(ch)
                raw_f = np.asarray(raw, dtype=np.float64)

                shift = shifts[ch]
                cut = residual_cut(raw, ch, rbcal, stop_cell, shift)
                if cut >= RMS_LO:
                    continue  # corrupted segment reaches into the fixed RMS window, skip

                naive_cal = np.asarray(rbcal.voltages(ch, stop_cell, raw))
                mangled_naive_rms.append(rms(naive_cal[RMS_LO:RMS_HI]))

                csc = (stop_cell - shift) % 1024
                corr_cal = np.asarray(rbcal.voltages(ch, csc, raw))
                mangled_corrected_rms.append(rms(corr_cal[RMS_LO:RMS_HI]))
                n_mangled += 1

print(f"n_good={len(good_rms)} n_mangled_naive={len(mangled_naive_rms)} n_mangled_corrected={len(mangled_corrected_rms)}")
print(f"good_rms: mean={np.mean(good_rms):.2f} median={np.median(good_rms):.2f}")
print(f"mangled_naive_rms: mean={np.mean(mangled_naive_rms):.2f} median={np.median(mangled_naive_rms):.2f}")
print(f"mangled_corrected_rms: mean={np.mean(mangled_corrected_rms):.2f} median={np.median(mangled_corrected_rms):.2f}")

fig, ax = plt.subplots(figsize=(9, 6))
bins = np.linspace(0, max(np.percentile(good_rms, 99), np.percentile(mangled_naive_rms, 99)), 60)
ax.hist(good_rms, bins=bins, alpha=0.5, label=f"Good events (n={len(good_rms)})", color='green', density=True)
ax.hist(mangled_naive_rms, bins=bins, alpha=0.5, label=f"Mangled, naive calib (n={len(mangled_naive_rms)})", color='red', density=True)
ax.hist(mangled_corrected_rms, bins=bins, alpha=0.5, label=f"Mangled, corrected (n={len(mangled_corrected_rms)})", color='blue', density=True)
ax.set_xlabel(f"baseline std, fixed window [{RMS_LO}:{RMS_HI}], calibrated units")
ax.set_ylabel("density")
ax.legend()
ax.set_title(f"RMS distributions [{RMS_LO}:{RMS_HI}]: good vs. mangled (naive vs. corrected calibration)")
plt.tight_layout()
plt.savefig("DMrecovery/rms_distributions_good_vs_mangled.png", dpi=120)
print("saved DMrecovery/rms_distributions_good_vs_mangled.png")
