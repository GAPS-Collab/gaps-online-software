import gondola as gon
import numpy as np
from pathlib import Path

FLIGHT_CALIB = Path("/mnt/ucla-gaps-nas1/tof-data/antarctica/flight_ssd_data/calib/251222_010511UTC")
calib = gon.calibration.load_rb_calibrations(FLIGHT_CALIB)
rbcal = calib[18]

target_file = "/mnt/ucla-gaps-nas1/tof-data/antarctica/flight_ssd_data/compressed/10178/Run10178_85.251225_222045UTC.tof.gaps"
reader = gon.io.TofPacketReader(target_file, filter=gon.packets.TofPacketType.TofEvent)
rb = None
for pack in reader:
    ev = gon.events.TofEvent.from_bytestream(pack.payload, 0)
    for r in ev.rb_events:
        if r.header.rb_id == 18 and r.header.event_id == 581162:
            rb = r
            break
    if rb:
        break

stop_cell = rb.header.stop_cell
shifts = {1:549, 2:547, 3:545, 4:543, 5:541, 6:539, 7:537, 8:535, 9:533}
channels = [c+1 for c in rb.header.get_channels()]

def find_cut_from_residual(raw, template, stop_cell, shift, k=6.0):
    """Per-sample residual against the pedestal template at the CORRECTED alignment.
    The corrupted lead shows large |residual| (raw content doesn't match any real cell's
    pedestal); once calibration is right, good samples settle to noise-level residual.
    Cut = last index (from the front) where a contiguous anomalous run ends."""
    idx = np.arange(len(raw))
    cells = (stop_cell - shift + idx) % 1024
    resid = raw.astype(np.float64) - template[cells]
    # establish the noise floor from the back half of the array (assumed uncorrupted)
    noise_floor = np.std(resid[512:])
    thresh = k * noise_floor
    anomalous = np.abs(resid - np.median(resid[512:])) > thresh
    # find the first index from the start where we get a run of `run_len` consecutive non-anomalous samples
    run_len = 8
    for i in range(len(raw) - run_len):
        if not anomalous[i:i+run_len].any():
            return i, resid, noise_floor
    return 0, resid, noise_floor

for ch in channels:
    if ch == 9:
        continue
    raw = rb.get_waveform(ch)
    template = rbcal.v_offsets[ch-1]
    shift = shifts[ch]
    cut, resid, noise_floor = find_cut_from_residual(raw, template, stop_cell, shift)
    print(f"ch{ch}: residual-based cut={cut}  noise_floor={noise_floor:.2f}  resid[0:cut+3]={resid[:cut+3].round(1).tolist()}")
