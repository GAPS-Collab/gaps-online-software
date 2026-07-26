import gondola as gon
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from pathlib import Path

FLIGHT_CALIB = Path("/mnt/ucla-gaps-nas1/tof-data/antarctica/flight_ssd_data/calib/251222_010511UTC")
calib = gon.calibration.load_rb_calibrations(FLIGHT_CALIB)

target_file = "/mnt/ucla-gaps-nas1/tof-data/antarctica/flight_ssd_data/compressed/10178/Run10178_85.251225_222045UTC.tof.gaps"
target_event_id = 581162
target_rb_id = 18

rb = None
reader = gon.io.TofPacketReader(target_file, filter=gon.packets.TofPacketType.TofEvent)
for pack in reader:
    ev = gon.events.TofEvent.from_bytestream(pack.payload, 0)
    for r in ev.rb_events:
        if r.header.rb_id == target_rb_id and r.header.event_id == target_event_id:
            rb = r
            break
    if rb:
        break

rbcal = calib[target_rb_id]
stop_cell = rb.header.stop_cell
channels = [c + 1 for c in rb.header.get_channels()]

def spike_score(w):
    return np.abs(2 * w[1:-1] - w[:-2] - w[2:])

def is_transient(w, idx, halfwin=4):
    lo = max(0, idx - halfwin - 1)
    hi = min(len(w), idx + halfwin + 2)
    before = w[lo:idx-1]
    after = w[idx+2:hi]
    if len(before) == 0 or len(after) == 0:
        return False
    return abs(np.median(after) - np.median(before)) < 3 * np.std(w[lo:hi])

def detect_leading_corruption_boundary(rb, channels, min_score=8):
    """Per-channel transient-glitch index, extrapolated via rank-in-active-list for
    channels with no confident detection of their own."""
    detections = {}
    for ch in channels:
        w = np.asarray(rb.get_waveform(ch), dtype=float)
        s = spike_score(w)
        mad = np.median(np.abs(s - np.median(s))) + 1e-9
        norm = s / mad
        order = np.argsort(norm)[::-1]
        for cand in order[:15]:
            idx = cand + 1
            if norm[cand] < min_score:
                break
            if is_transient(w, idx):
                detections[ch] = idx
                break
    if len(detections) < 2:
        return {ch: 0 for ch in channels}  # not enough signal to trust any cut
    chs = np.array(list(detections.keys()), dtype=float)
    idxs = np.array(list(detections.values()), dtype=float)
    b, a = np.polyfit(chs, idxs, 1)
    return {ch: max(0, int(round(a + b*ch))) for ch in channels}

boundaries = detect_leading_corruption_boundary(rb, channels)
print("detected leading-corruption boundaries (per channel):", boundaries)

# per-channel exact stop_cell shift, established previously for this event
pred_shifts = {1:549, 2:547, 3:545, 4:543, 5:541, 6:539, 7:537, 8:535, 9:533}

fig, axes = plt.subplots(len(channels), 2, figsize=(12, 2*len(channels)), sharex=True)
for i, ch in enumerate(channels):
    raw = rb.get_waveform(ch)
    shift = pred_shifts[ch]
    csc = (stop_cell - shift) % 1024
    corr_cal = np.asarray(rbcal.voltages(ch, csc, raw))

    boundary = boundaries[ch]
    masked = corr_cal.copy()
    masked[:boundary + 1] = 0.0  # +1: the glitch sample itself sits at index `boundary`

    axes[i,0].plot(corr_cal, lw=0.7, color=("red" if ch==9 else "C0"))
    axes[i,0].axvline(boundary, color='gray', ls='--', lw=0.8)
    axes[i,0].set_ylabel(f"ch{ch}\nrecalibrated", rotation=0, labelpad=30)

    axes[i,1].plot(masked, lw=0.7, color=("red" if ch==9 else "C0"))
    axes[i,1].set_ylabel(f"ch{ch}\n+lead masked\n(cut={boundary})", rotation=0, labelpad=30)

axes[0,0].set_title("Recalibrated (stop_cell corrected)")
axes[0,1].set_title("+ leading corrupted segment zeroed")
fig.suptitle(f"rb_id={target_rb_id} event_id={target_event_id}")
plt.tight_layout()
outpath = "DMrecovery/rb18_ev581162_leading_masked.png"
plt.savefig(outpath, dpi=110)
print("saved", outpath)
