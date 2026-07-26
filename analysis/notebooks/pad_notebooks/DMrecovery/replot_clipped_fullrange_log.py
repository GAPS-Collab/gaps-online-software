"""
Full-range, log-y-scale version of replot_clipped_quick.py -- same bin width
(0.2) as the [0,20] view, but spanning the whole data range. Loads the same
cached DMrecovery/rms_distributions_clipped_data.npz, no rerun needed.
"""
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

DATA_PATH = "DMrecovery/rms_distributions_clipped_data.npz"
OUT_PATH = "DMrecovery/rms_distributions_clipped_fullrange_log.png"

BIN_WIDTH = 0.2  # same as the [0,20] view
XHI_CAP = 40  # cap the x-axis here even if the data's true max is larger
RMS_LO, RMS_HI = 700, 900
POPS = [
    ("good_rms", "Good", "tab:green"),
    ("mangled_naive_rms", "Mangled, uncorrected (naive)", "tab:red"),
    ("mangled_corrected_700to900_clipped_rms", "Mangled, corrected, [700:900] clipped +8mV", "tab:purple"),
]

data = np.load(DATA_PATH)
xmax = min(max(data[k].max() for k, _, _ in POPS), XHI_CAP)
xhi = np.ceil(xmax / BIN_WIDTH) * BIN_WIDTH
nbins = int(round(xhi / BIN_WIDTH)) + 1
bins = np.linspace(0, xhi, nbins)
print(f"xhi={xhi:.1f} nbins={nbins-1} bin_width={BIN_WIDTH}")

fig, ax = plt.subplots(figsize=(11, 6.5))
for key, label, color in POPS:
    vals = data[key]
    counts, edges = np.histogram(vals, bins=bins)
    ax.stairs(counts.astype(float), edges, color=color, alpha=0.15, fill=True)
    ax.stairs(counts.astype(float), edges, color=color, linewidth=1.5,
              label=f"{label} (n={len(vals)}, max={vals.max():.1f})")
    print(f"{label}: n={len(vals)} mean={np.mean(vals):.3f} median={np.median(vals):.3f} max={vals.max():.2f}")

ax.set_xlim(0, xhi)
ax.set_yscale('log')
ax.set_xlabel(f"baseline std, window [{RMS_LO}:{RMS_HI}], calibrated units")
ax.set_ylabel("count (n) per bin -- log scale")
ax.legend(fontsize=9)
ax.set_title(f"RMS distributions, window [{RMS_LO}:{RMS_HI}] -- full range, log y, bin width={BIN_WIDTH}")
plt.tight_layout()
plt.savefig(OUT_PATH, dpi=120)
print("saved", OUT_PATH)
