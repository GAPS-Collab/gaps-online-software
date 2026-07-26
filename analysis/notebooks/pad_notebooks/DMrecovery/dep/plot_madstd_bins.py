import sys
sys.path.insert(0, '/tmp/claude-1000/-home-gaps-software-padrick-dev-test-gaps-os-pro-analysis-notebooks-pad-notebooks/da138849-a795-44a3-b8cf-c9fdd0641ad9/scratchpad')
from full_recovery_v1 import detect_event_shifts, residual_cut, FLIGHT_CALIB

import gondola as gon
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

calib = gon.calibration.load_rb_calibrations(FLIGHT_CALIB)
MANGLING = {gon.events.EventStatus.ChnSyncErrors, gon.events.EventStatus.CellSyncErrors, gon.events.EventStatus.CellAndChnSyncErrors}

files = [l.strip() for l in open('run_lists/flight_small.txt') if l.strip() and not l.startswith('#')]

RMS_LO, RMS_HI = 700, 900
BIN_EDGES = [0, 2, 4, 6, 8, 10, 12]
RANGES = [(f"bin_{lo}to{hi}", lo, hi) for lo, hi in zip(BIN_EDGES[:-1], BIN_EDGES[1:])]
MAX_PER_RANGE = 10

def mad_std(window):
    med = np.median(window)
    mad = np.median(np.abs(window - med))
    return 1.4826 * mad

examples = {name: [] for name, _, _ in RANGES}
n_seen = 0

for f in files:
    if all(len(examples[name]) >= MAX_PER_RANGE for name, _, _ in RANGES):
        break
    reader = gon.io.TofPacketReader(str(f), filter=gon.packets.TofPacketType.TofEvent)
    for pack in reader:
        if all(len(examples[name]) >= MAX_PER_RANGE for name, _, _ in RANGES):
            break
        ev = gon.events.TofEvent.from_bytestream(pack.payload, 0)
        for rb in ev.rb_events:
            if all(len(examples[name]) >= MAX_PER_RANGE for name, _, _ in RANGES):
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
            window = corr_cal[RMS_LO:RMS_HI]
            std_val = mad_std(window)

            for name, rlo, rhi in RANGES:
                if len(examples[name]) < MAX_PER_RANGE and rlo <= std_val < rhi:
                    examples[name].append(dict(rb_id=rid, event_id=rb.header.event_id, ch=ch,
                                                corr_cal=corr_cal, cut=cut, std=std_val))
                    print(f"[{name} {len(examples[name])}/{MAX_PER_RANGE}] rb={rid} ev={rb.header.event_id} ch={ch} mad_std={std_val:.2f} (seen {n_seen})", flush=True)

for name, _, _ in RANGES:
    ex = examples[name]
    print(f"\n{name}: collected {len(ex)}")
    if not ex:
        continue
    fig, axes = plt.subplots(len(ex), 1, figsize=(10, 2 * max(len(ex), 1)), sharex=True)
    if len(ex) == 1:
        axes = [axes]
    for i, d in enumerate(ex):
        ax = axes[i]
        ax.plot(d["corr_cal"], lw=0.7, color='C0')
        ax.axvline(d["cut"], color='black', ls='--', lw=1.0)
        ax.axvspan(RMS_LO, RMS_HI, color='brown', alpha=0.15)
        ax.set_ylabel(f"rb{d['rb_id']} ev{d['event_id']}\nch{d['ch']}\nmad_std={d['std']:.2f}", rotation=0, labelpad=45, fontsize=8)
    axes[-1].set_xlabel("sample index")
    fig.suptitle(f"{name}: corrected waveforms, mad_std in [{name.split('_')[1]}] over [{RMS_LO}:{RMS_HI}]\n(black dashed=cut, brown shading=RMS window)", y=1.0)
    plt.tight_layout()
    outpath = f"DMrecovery/madstd_{name}_examples.png"
    plt.savefig(outpath, dpi=110)
    print("saved", outpath)
