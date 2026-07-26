import gondola as gon
import numpy as np
from pathlib import Path
import sys

FLIGHT_CALIB = Path("/mnt/ucla-gaps-nas1/tof-data/antarctica/flight_ssd_data/calib/251222_010511UTC")
calib = gon.calibration.load_rb_calibrations(FLIGHT_CALIB)

MANGLING = {gon.events.EventStatus.ChnSyncErrors, gon.events.EventStatus.CellSyncErrors, gon.events.EventStatus.CellAndChnSyncErrors}

files = [l.strip() for l in open('run_lists/flight_hour.txt') if l.strip() and not l.startswith('#')]

def quiet_mask(raw, win=15, k=4.0):
    med = np.median(raw)
    mad = np.median(np.abs(raw - med)) + 1e-9
    dev = np.abs(raw - med)
    kernel = np.ones(win)
    smoothed = np.convolve(dev, kernel/win, mode='same')
    return smoothed < k*mad

def full_shift_scan(raw, template, stop_cell, mask, min_quiet=100):
    if mask.sum() < min_quiet:
        return None, None, None
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
    return best_shift, best_res, second

MAX_EVENTS = 40
tested = 0
model_holds = 0
model_fails = 0
sep_ratios = []
fail_details = []

for f in files:
    if tested >= MAX_EVENTS:
        break
    reader = gon.io.TofPacketReader(str(f), filter=gon.packets.TofPacketType.TofEvent)
    for pack in reader:
        if tested >= MAX_EVENTS:
            break
        ev = gon.events.TofEvent.from_bytestream(pack.payload, 0)
        for rb in ev.rb_events:
            if tested >= MAX_EVENTS:
                break
            if rb.status not in MANGLING:
                continue
            rid = rb.header.rb_id
            if rid not in calib:
                continue
            active = sorted(rb.header.get_channels())
            active_1idx = [c+1 for c in active]
            non9 = [c for c in active_1idx if c != 9]
            if len(non9) < 2:
                continue
            tested += 1
            stop_cell = rb.header.stop_cell
            rbcal = calib[rid]
            shifts = {}
            for ch in non9:
                raw = np.asarray(rb.get_waveform(ch), dtype=np.float64)
                template = rbcal.v_offsets[ch-1]
                mask = quiet_mask(raw)
                best_shift, best_res, second = full_shift_scan(raw, template, stop_cell, mask)
                if best_shift is None or best_res == 0:
                    continue
                ratio = second/best_res
                if ratio < 1.3:
                    continue  # low confidence, skip
                sep_ratios.append(ratio)
                pos = active_1idx.index(ch)
                shifts[ch] = (best_shift, pos)

            if len(shifts) < 2:
                print(f"[{tested}] rb={rid} ev={rb.header.event_id} nch={len(active_1idx)}: not enough confident channels ({len(shifts)})")
                continue

            # check constant = shift + 2*pos
            consts = {ch: sh + 2*pos for ch, (sh,pos) in shifts.items()}
            vals = list(consts.values())
            spread = max(vals) - min(vals)
            status = "OK" if spread <= 1 else "FAIL"
            if status == "OK":
                model_holds += 1
            else:
                model_fails += 1
                fail_details.append((rid, rb.header.event_id, shifts, consts))
            print(f"[{tested}] rb={rid} ev={rb.header.event_id} nch={len(active_1idx)} confident_ch={len(shifts)} spread={spread} -> {status}")
            sys.stdout.flush()

print("\n=== SUMMARY ===")
print(f"tested={tested} model_holds={model_holds} model_fails={model_fails}")
if sep_ratios:
    print(f"sep_ratio: mean={np.mean(sep_ratios):.2f} median={np.median(sep_ratios):.2f} min={np.min(sep_ratios):.2f}")
for fd in fail_details[:5]:
    print("FAIL DETAIL:", fd)
