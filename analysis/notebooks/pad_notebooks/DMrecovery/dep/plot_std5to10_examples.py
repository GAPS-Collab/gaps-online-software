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

files = [l.strip() for l in open('run_lists/flight_small.txt') if l.strip() and not l.startswith('#')]

RMS_LO, RMS_HI = 700, 900
TARGET_LO, TARGET_HI = 5, 10
MAX_EXAMPLES = 15

examples = []
n_seen = 0

for f in files:
    if len(examples) >= MAX_EXAMPLES:
        break
    reader = gon.io.TofPacketReader(str(f), filter=gon.packets.TofPacketType.TofEvent)
    for pack in reader:
        if len(examples) >= MAX_EXAMPLES:
            break
        ev = gon.events.TofEvent.from_bytestream(pack.payload, 0)
        for rb in ev.rb_events:
            if len(examples) >= MAX_EXAMPLES:
                break
            if rb.status not in MANGLING:
                continue
            rid = rb.header.rb_id
            if rid not in calib:
                continue
            rbcal = calib[rid]
            stop_cell = rb.header.stop_cell
            active_1idx = sorted(c + 1 for c in rb.header.get_channels())
            non9 = [c for c in active_1idx if c != 9]
            if len(non9) < 2:
                continue
            n_seen += 1

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

            csc = (stop_cell - shift) % 1024
            corr_cal = np.asarray(rbcal.voltages(ch, csc, raw))

            qmask_window = quiet_mask(raw_f)[RMS_LO:RMS_HI]
            if qmask_window.sum() < 30:
                continue
            std_val = np.std(corr_cal[RMS_LO:RMS_HI][qmask_window])

            if TARGET_LO <= std_val <= TARGET_HI:
                examples.append(dict(f=f, rb_id=rid, event_id=rb.header.event_id, ch=ch,
                                      corr_cal=corr_cal, cut=cut, std_val=std_val, shift=shift))
                print(f"[{len(examples)}/{MAX_EXAMPLES}] rb={rid} ev={rb.header.event_id} ch={ch} std={std_val:.2f} (seen {n_seen} mangled so far)", flush=True)

print(f"\ncollected {len(examples)} examples after scanning {n_seen} mangled events")

fig, axes = plt.subplots(len(examples), 1, figsize=(10, 2 * len(examples)), sharex=True)
for i, d in enumerate(examples):
    ax = axes[i]
    ax.plot(d["corr_cal"], lw=0.7, color='C0')
    ax.axvline(d["cut"], color='black', ls='--', lw=1.0)
    ax.axvspan(RMS_LO, RMS_HI, color='green', alpha=0.15)
    ax.set_ylabel(f"rb{d['rb_id']} ev{d['event_id']}\nch{d['ch']}\nstd={d['std_val']:.2f}", rotation=0, labelpad=45, fontsize=8)
axes[-1].set_xlabel("sample index")
fig.suptitle(f"15 corrected waveforms with quiet_mask std in [{TARGET_LO}:{TARGET_HI}] over [{RMS_LO}:{RMS_HI}]\n(black dashed=cut, green shading=RMS window)", y=1.0)
plt.tight_layout()
plt.savefig("DMrecovery/std5to10_examples.png", dpi=110)
print("saved DMrecovery/std5to10_examples.png")
