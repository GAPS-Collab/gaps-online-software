import sys
sys.path.insert(0, '/tmp/claude-1000/-home-gaps-software-padrick-dev-test-gaps-os-pro-analysis-notebooks-pad-notebooks/da138849-a795-44a3-b8cf-c9fdd0641ad9/scratchpad')
from full_recovery_v1 import quiet_mask, recover_event, FLIGHT_CALIB

import gondola as gon
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

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
stop_cell = rb.header.stop_cell
shifts = {1:549, 2:547, 3:545, 4:543, 5:541, 6:539, 7:537, 8:535, 9:533}

# ---------- Plot 1: ch1 (real pulse) vs ch3 (incoherent noise) residual, leading region ----------
fig, axes = plt.subplots(2, 1, figsize=(9, 6), sharex=True)
for ax, ch, label in [(axes[0], 1, "ch1 -- SMOOTH, coherent (real pulse)"),
                       (axes[1], 3, "ch3 -- INCOHERENT (corrupted noise)")]:
    raw = np.asarray(rb.get_waveform(ch), dtype=np.float64)
    template = rbcal.v_offsets[ch - 1]
    shift = shifts[ch]
    idx = np.arange(len(raw))
    cells = (stop_cell - shift + idx) % 1024
    resid = raw - template[cells]
    ax.plot(resid[:100], color=("C3" if ch == 1 else "C0"), lw=1.2)
    ax.axhline(0, color='gray', lw=0.6, ls=':')
    ax.set_title(f"{label} -- residual vs pedestal template, first 100 samples")
    ax.set_ylabel("residual (raw ADC)")
axes[-1].set_xlabel("sample index")
plt.tight_layout()
plt.savefig("DMrecovery/ch1_coherent_vs_ch3_incoherent_residual.png", dpi=120)
print("saved DMrecovery/ch1_coherent_vs_ch3_incoherent_residual.png")

# ---------- Plot 2: before/after calibration with cut line + fixed RMS-region [700:900] ----------
result = recover_event(rb, rbcal)
plot_channels = [1, 3, 9]
RMS_LO, RMS_HI = 700, 900

fig, axes = plt.subplots(len(plot_channels), 2, figsize=(13, 3 * len(plot_channels)), sharex=True)
for i, ch in enumerate(plot_channels):
    d = result["channels"][ch]
    naive_raw = rb.get_waveform(ch)
    naive_cal = np.asarray(rbcal.voltages(ch, stop_cell, naive_raw))
    corr_cal = d["raw_calibrated"]
    cut = d["cut"]
    color = "red" if ch == 9 else "C0"

    for col, (wave, title) in enumerate([(naive_cal, "NAIVE calibration"), (corr_cal, "CORRECTED calibration")]):
        ax = axes[i, col]
        ax.plot(wave, lw=0.7, color=color)
        ax.axvline(cut, color='black', ls='--', lw=1.2, label=f"cut={cut}")
        ax.axvspan(RMS_LO, RMS_HI, color='green', alpha=0.15)
        rms_val = np.std(wave[RMS_LO:RMS_HI])
        ax.set_title(f"ch{ch} {title}\nRMS([{RMS_LO}:{RMS_HI}])={rms_val:.2f}", fontsize=10)
        if col == 0:
            ax.legend(loc='upper right', fontsize=8)
axes[-1, 0].set_xlabel("sample index")
axes[-1, 1].set_xlabel("sample index")
fig.suptitle(f"rb_id=18 event_id=581162 -- black dashed = corrupted-segment cut, green shading = fixed RMS region [{RMS_LO}:{RMS_HI}]", y=1.0)
plt.tight_layout()
plt.savefig("DMrecovery/waveforms_with_cut_and_rms_region.png", dpi=120)
print("saved DMrecovery/waveforms_with_cut_and_rms_region.png")
