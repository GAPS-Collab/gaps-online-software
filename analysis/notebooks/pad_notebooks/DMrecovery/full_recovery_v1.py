"""
Full mangled-event recovery recipe (v1), validated interactively on rb_id=18/event_id=581162
and spot-checked on ~9 other mangled events across rb_id in {1,2,4,18}.

Two independent, additive corrections:
  1. Exact per-channel stop_cell correction, derived from one confidently-detected
     "anchor" channel (full 1024-shift residual scan against calibration's v_offsets
     pedestal template) + a fixed per-active-channel-rank rule (shift = intercept - 2*rank).
  2. Leading corrupted-content segment: detected from the SAME per-sample residual
     (against v_offsets, at the corrected shift) -- corrupted samples show large
     residual; good samples settle to the noise floor. Cut is applied to ALL
     channels uniformly. Note: on some channels this residual looks like a smooth,
     coherent ramp rather than incoherent noise (e.g. ch1 in rb18/581162) -- this is
     ch9 reference-sine leakage bleeding into that channel's slot, not a real pulse,
     so cutting it uniformly with the other channels is correct, not a tradeoff.
"""
import gondola as gon
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from pathlib import Path

FLIGHT_CALIB = Path("/mnt/ucla-gaps-nas1/tof-data/antarctica/flight_ssd_data/calib/251222_010511UTC")


def quiet_mask(raw, win=15, k=4.0):
    med = np.median(raw)
    mad = np.median(np.abs(raw - med)) + 1e-9
    dev = np.abs(raw - med)
    kernel = np.ones(win)
    smoothed = np.convolve(dev, kernel / win, mode='same')
    return smoothed < k * mad


def full_shift_scan(raw, template, stop_cell, mask, min_quiet=100):
    """Full 1024-position residual scan against the pedestal template.
    Returns (best_shift, confidence_ratio) or (None, None) if too few quiet samples."""
    if mask.sum() < min_quiet:
        return None, None
    idx = np.arange(len(raw))
    best_shift, best_res = None, None
    for shift in range(1024):
        cells = (stop_cell + idx - shift) % 1024
        pred = template[cells]
        s = np.std(raw[mask] - pred[mask])
        if best_res is None or s < best_res:
            best_res, best_shift = s, shift
    second = None
    for shift in range(1024):
        if abs(shift - best_shift) <= 5 or abs(shift - best_shift) >= 1019:
            continue
        cells = (stop_cell + idx - shift) % 1024
        s = np.std(raw[mask] - template[cells][mask])
        if second is None or s < second:
            second = s
    ratio = second / best_res if best_res > 0 else 0
    return best_shift, ratio


def detect_event_shifts(rb, rbcal, non9_channels, active_1idx_all):
    """Anchor on the most confident channel, derive the rest via the rank rule."""
    stop_cell = rb.header.stop_cell
    best_anchor = None
    for ch in non9_channels:
        raw = np.asarray(rb.get_waveform(ch), dtype=np.float64)
        template = rbcal.v_offsets[ch - 1]
        mask = quiet_mask(raw)
        shift, ratio = full_shift_scan(raw, template, stop_cell, mask)
        if shift is None:
            continue
        if best_anchor is None or ratio > best_anchor[1]:
            pos = active_1idx_all.index(ch)
            best_anchor = (ch, ratio, shift, pos)
    if best_anchor is None:
        return None, None
    ch_a, ratio_a, shift_a, pos_a = best_anchor
    intercept = shift_a + 2 * pos_a
    shifts = {ch: intercept - 2 * active_1idx_all.index(ch) for ch in active_1idx_all}
    return shifts, (ch_a, ratio_a)


GOOD_BASELINE_STD = 0.6  # established from ~4000 good events' [700:900] baseline std


def residual_cut(raw, ch, rbcal, stop_cell, shift, k=6.0, run_len=20,
                  good_baseline_std=GOOD_BASELINE_STD):
    """Leading-corruption cut point: EXACT, per-sample, no smoothing/windowing.

    v3: operates directly on the FULLY calibrated waveform (rbcal.voltages()),
    not a manual raw-vs-v_offsets residual. Two bugs in the earlier version:
      1. Unit mismatch -- a manual `raw - v_offsets[cells]` residual is in raw
         ADC counts, ~15.5x larger than the calibrated mV scale `voltages()`
         actually produces (it applies gain + other calibration stages beyond
         pedestal subtraction alone). A threshold tuned in one unit silently
         fails in the other.
      2. Breakdown point -- a PER-EVENT relative threshold (median/MAD of this
         event's own array) fails once the corrupted fraction of THIS event's
         1024 samples exceeds ~50%, since a robust statistic can't identify a
         majority as "the outlier". Verified: an event with corruption spanning
         ~54% of the array gave a false cut=0 even with a robust MAD estimator.
      Fix: use an ABSOLUTE threshold anchored to the known good-event baseline
      noise level (measured independently from thousands of good events, not
      from this event's own possibly-majority-corrupted array).

    v2 fixes (still applied): run_len raised from 8 to 20, since a short
    run_len can find a random quiet-looking lull inside oscillating corrupted
    noise that isn't actually sustained."""
    csc = (stop_cell - shift) % 1024
    corr_cal = np.asarray(rbcal.voltages(ch, csc, raw))
    med = np.median(corr_cal)
    anomalous = np.abs(corr_cal - med) > k * good_baseline_std
    for i in range(len(corr_cal) - run_len):
        if not anomalous[i:i + run_len].any():
            return i
    return 0


def recover_event(rb, rbcal):
    """Returns dict: channel -> (corrected_calibrated_waveform, shift, cut) plus diagnostics."""
    stop_cell = rb.header.stop_cell
    active_1idx = sorted(c + 1 for c in rb.header.get_channels())
    non9 = [c for c in active_1idx if c != 9]

    shifts, anchor_info = detect_event_shifts(rb, rbcal, non9, active_1idx)
    if shifts is None:
        return None

    # Raw per-channel cut detection first (unreliable on channels without a clean
    # baseline region, e.g. ch9's own big sine signal breaks the noise-floor estimate).
    raw_cuts = {}
    for ch in non9:
        raw = rb.get_waveform(ch)
        raw_cuts[ch] = residual_cut(raw, ch, rbcal, stop_cell, shifts[ch])

    # The cut follows the same "-2 per active-channel rank" rule as the calibration
    # shift (same underlying desync). Fit across the non-9 channels and use the fit
    # for every channel, including ones (like ch9) where direct detection is invalid.
    chs = np.array(list(raw_cuts.keys()), dtype=float)
    cuts = np.array(list(raw_cuts.values()), dtype=float)
    ranks = np.array([active_1idx.index(ch) for ch in raw_cuts], dtype=float)
    b, a = np.polyfit(ranks, cuts, 1)
    fitted_cuts = {ch: max(0, int(round(a + b * active_1idx.index(ch)))) for ch in active_1idx}

    out = {}
    for ch in active_1idx:
        raw = rb.get_waveform(ch)
        shift = shifts[ch]
        csc = (stop_cell - shift) % 1024
        cal = np.asarray(rbcal.voltages(ch, csc, raw))
        cut = fitted_cuts[ch]
        masked = cal.copy()
        masked[:cut + 1] = 0.0
        out[ch] = dict(raw_calibrated=cal, masked=masked, shift=shift, cut=cut,
                        raw_cut=raw_cuts.get(ch))
    return dict(channels=out, anchor=anchor_info, stop_cell=stop_cell, cut_fit=(a, b))


if __name__ == "__main__":
    calib = gon.calibration.load_rb_calibrations(FLIGHT_CALIB)
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

    rbcal = calib[18]
    result = recover_event(rb, rbcal)
    print("anchor:", result["anchor"])
    fig, axes = plt.subplots(len(result["channels"]), 1, figsize=(10, 2 * len(result["channels"])), sharex=True)
    for i, (ch, d) in enumerate(sorted(result["channels"].items())):
        color = "red" if ch == 9 else "C0"
        axes[i].plot(d["masked"], lw=0.7, color=color)
        axes[i].set_ylabel(f"ch{ch}\nshift={d['shift']}\ncut={d['cut']}", rotation=0, labelpad=30, fontsize=8)
        print(f"ch{ch}: shift={d['shift']} cut={d['cut']}")
    fig.suptitle(f"rb_id=18 event_id=581162 -- fully recovered (recalibrated + leading segment cut)")
    plt.tight_layout()
    plt.savefig("DMrecovery/rb18_ev581162_fully_recovered.png", dpi=110)
    print("saved DMrecovery/rb18_ev581162_fully_recovered.png")
