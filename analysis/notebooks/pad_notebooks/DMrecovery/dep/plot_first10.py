import gondola as gon
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from pathlib import Path

FLIGHT_CALIB = Path("/mnt/ucla-gaps-nas1/tof-data/antarctica/flight_ssd_data/calib/251222_010511UTC")
calib = gon.calibration.load_rb_calibrations(FLIGHT_CALIB)
MANGLING = {gon.events.EventStatus.ChnSyncErrors, gon.events.EventStatus.CellSyncErrors, gon.events.EventStatus.CellAndChnSyncErrors}
files = [l.strip() for l in open('run_lists/flight_small.txt') if l.strip() and not l.startswith('#')]

def quiet_mask(raw, win=15, k=4.0):
    med = np.median(raw)
    mad = np.median(np.abs(raw - med)) + 1e-9
    dev = np.abs(raw - med)
    kernel = np.ones(win)
    smoothed = np.convolve(dev, kernel/win, mode='same')
    return smoothed < k*mad

def full_shift_scan(raw, template, stop_cell, mask, min_quiet=100):
    if mask.sum() < min_quiet:
        return None, None
    idx = np.arange(len(raw))
    best_shift, best_res = None, None
    for shift in range(1024):
        cells = (stop_cell + idx - shift) % 1024
        pred = template[cells]
        s = np.std(raw[mask] - pred[mask])
        if best_res is None or s < best_res:
            best_res = s
            best_shift = shift
    second = None
    for shift in range(1024):
        if abs(shift - best_shift) <= 5 or abs(shift - best_shift) >= 1019:
            continue
        cells = (stop_cell + idx - shift) % 1024
        pred = template[cells]
        s = np.std(raw[mask] - pred[mask])
        if second is None or s < second:
            second = s
    ratio = second/best_res if best_res > 0 else 0
    return best_shift, ratio

candidates = []
seen_rb = {}
for f in files:
    reader = gon.io.TofPacketReader(str(f), filter=gon.packets.TofPacketType.TofEvent)
    for pack in reader:
        ev = gon.events.TofEvent.from_bytestream(pack.payload, 0)
        for rb in ev.rb_events:
            if rb.status in MANGLING:
                rid = rb.header.rb_id
                nch = bin(rb.header.channel_mask).count('1')
                key = (rid, nch)
                if key not in seen_rb:
                    seen_rb[key] = (f, rb.header.event_id)
                    candidates.append((f, rid, rb.header.event_id, nch))
    if len(candidates) >= 10:
        break

results = []
for f, rid, eid, nch in candidates[:10]:
    if rid not in calib:
        continue
    rbcal = calib[rid]
    reader = gon.io.TofPacketReader(str(f), filter=gon.packets.TofPacketType.TofEvent)
    rb = None
    for pack in reader:
        ev = gon.events.TofEvent.from_bytestream(pack.payload, 0)
        for r in ev.rb_events:
            if r.header.rb_id == rid and r.header.event_id == eid:
                rb = r
                break
        if rb:
            break
    if rb is None:
        continue
    stop_cell = rb.header.stop_cell
    active_1idx = sorted(c+1 for c in rb.header.get_channels())
    non9 = [c for c in active_1idx if c != 9]

    best_anchor = None
    for ch in non9:
        raw = np.asarray(rb.get_waveform(ch), dtype=np.float64)
        template = rbcal.v_offsets[ch-1]
        mask = quiet_mask(raw)
        shift, ratio = full_shift_scan(raw, template, stop_cell, mask)
        if shift is None:
            continue
        if best_anchor is None or ratio > best_anchor[2]:
            pos = active_1idx.index(ch)
            best_anchor = (ch, shift, ratio, pos)

    if best_anchor is None:
        print(f"rb={rid} ev={eid}: no confident anchor found, skipping")
        continue

    ch_a, shift_a, ratio_a, pos_a = best_anchor
    intercept = shift_a + 2*pos_a
    plot_ch = ch_a  # plot the confidently-detected anchor channel itself
    plot_shift = shift_a

    raw_plot = rb.get_waveform(plot_ch)
    naive_cal = np.asarray(rbcal.voltages(plot_ch, stop_cell, raw_plot))
    csc = (stop_cell - plot_shift) % 1024
    corr_cal = np.asarray(rbcal.voltages(plot_ch, csc, raw_plot))

    qmask = quiet_mask(np.asarray(raw_plot, dtype=np.float64))
    naive_qstd = np.std(naive_cal[qmask])
    corr_qstd = np.std(corr_cal[qmask])

    results.append(dict(rid=rid, eid=eid, nch=nch, anchor_ch=ch_a, anchor_ratio=ratio_a,
                         plot_ch=plot_ch, naive=naive_cal, corr=corr_cal,
                         naive_std=naive_qstd, corr_std=corr_qstd))
    print(f"rb={rid} ev={eid} nch={nch}: anchor=ch{ch_a} ratio={ratio_a:.2f} quiet_naive_std={naive_qstd:.2f} quiet_corr_std={corr_qstd:.2f}")

fig, axes = plt.subplots(len(results), 2, figsize=(12, 2.2*len(results)), sharex=True)
for i, r in enumerate(results):
    axes[i,0].plot(r['naive'], lw=0.7, color='C0')
    axes[i,0].set_ylabel(f"rb{r['rid']} ev{r['eid']}\nch{r['plot_ch']} naive\nquiet_std={r['naive_std']:.1f}", rotation=0, labelpad=40, fontsize=8)
    axes[i,1].plot(r['corr'], lw=0.7, color='C0')
    axes[i,1].set_ylabel(f"ch{r['plot_ch']} corrected\nquiet_std={r['corr_std']:.1f}", rotation=0, labelpad=35, fontsize=8)
axes[0,0].set_title("NAIVE calibration (baseline std over quiet samples)")
axes[0,1].set_title("Shift-corrected calibration (baseline std over quiet samples)")
plt.tight_layout()
outpath = "DMrecovery/first10_events_before_after.png"
plt.savefig(outpath, dpi=110)
print("saved", outpath)
