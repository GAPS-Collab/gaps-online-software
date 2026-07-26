import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
from full_recovery_v1 import detect_event_shifts, residual_cut, FLIGHT_CALIB

import gondola as gon
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

calib = gon.calibration.load_rb_calibrations(FLIGHT_CALIB)
MANGLING = {gon.events.EventStatus.ChnSyncErrors, gon.events.EventStatus.CellSyncErrors, gon.events.EventStatus.CellAndChnSyncErrors}

files = [l.strip() for l in open('run_lists/flight_30min.txt') if l.strip() and not l.startswith('#')]

RMS_LO, RMS_HI = 700, 900
CLIP_HI = 8.0
TARGET_LO, TARGET_HI = 2.2, 4.5
MAX_EXAMPLES = 100

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

            shift = shifts[ch]
            cut = residual_cut(raw, ch, rbcal, stop_cell, shift)
            if cut >= RMS_LO:
                continue

            csc = (stop_cell - shift) % 1024
            corr_cal = np.asarray(rbcal.voltages(ch, csc, raw))
            window = corr_cal[RMS_LO:RMS_HI]
            std_val = np.std(np.clip(window, None, CLIP_HI))

            if TARGET_LO <= std_val < TARGET_HI:
                examples.append(dict(rb_id=rid, event_id=rb.header.event_id, ch=ch,
                                      corr_cal=corr_cal, cut=cut, std=std_val))
                if len(examples) % 10 == 0:
                    print(f"[{len(examples)}/{MAX_EXAMPLES}] rb={rid} ev={rb.header.event_id} ch={ch} std={std_val:.2f} (seen {n_seen})", flush=True)

print(f"\ncollected {len(examples)} examples after scanning {n_seen} mangled events")

ncols = 10
nrows = (len(examples) + ncols - 1) // ncols
fig, axes = plt.subplots(nrows, ncols, figsize=(3*ncols, 1.6*nrows), sharex=True)
axes = axes.flatten()
for i, d in enumerate(examples):
    ax = axes[i]
    ax.plot(d["corr_cal"], lw=0.5, color='C0')
    ax.axvline(d["cut"], color='black', ls='--', lw=0.6)
    ax.axvspan(RMS_LO, RMS_HI, color='purple', alpha=0.15)
    ax.set_title(f"rb{d['rb_id']} ev{d['event_id']} ch{d['ch']}\nstd={d['std']:.2f}", fontsize=6)
    ax.tick_params(labelsize=5)
for j in range(len(examples), len(axes)):
    axes[j].axis('off')
fig.suptitle(f"100 corrected waveforms, clipped std in [{TARGET_LO}:{TARGET_HI}] over [{RMS_LO}:{RMS_HI}]", y=1.0)
plt.tight_layout()
outpath = "DMrecovery/100wf_first_hump_2.2to4.5.png"
plt.savefig(outpath, dpi=100)
print("saved", outpath)
