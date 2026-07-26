"""
Quick replot from cached DMrecovery/rms_distributions_clipped_data.npz --
no rerun of the expensive event-scanning needed. Edit the params below
(XLO/XHI, LOG_Y, NBINS, POPS) and rerun this script directly.
"""
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

DATA_PATH = "DMrecovery/rms_distributions_clipped_data.npz"
OUT_PATH = "DMrecovery/rms_distributions_clipped_replot.png"

# ---- easy knobs ----
XLO, XHI = 0, 20
NBINS = 101
LOG_Y = False
NORMALIZE = False  # False = raw counts (n) per bin; True = density normalized to full population
RMS_LO, RMS_HI = 700, 900
POPS = [
    # (array_key_in_npz, label, color)
    ("good_rms", "Good", "tab:green"),
    ("mangled_naive_rms", "Mangled, uncorrected (naive)", "tab:red"),
    ("mangled_corrected_700to900_clipped_rms", "Mangled, corrected, [700:900] clipped +8mV", "tab:purple"),
    # other available keys: good_madstd_rms, mangled_corrected_700to900_rms
    #                       (unclipped), mangled_corrected_quietmask_rms,
    #                       mangled_corrected_700to900_madstd_rms
]
# ---------------------

data = np.load(DATA_PATH)


def histify(vals, bins, normalize):
    """When normalize=True: density normalized by the FULL population size, not
    just values inside the plotted range -- so a population with more mass
    outside [XLO,XHI] correctly shows less area within the visible window.
    When normalize=False: raw counts (n) per bin, no normalization at all."""
    counts, edges = np.histogram(vals, bins=bins)
    if not normalize:
        return counts.astype(float), edges
    width = edges[1] - edges[0]
    return counts / (len(vals) * width), edges


bins = np.linspace(XLO, XHI, NBINS)
fig, ax = plt.subplots(figsize=(11, 6.5))
for key, label, color in POPS:
    vals = data[key]
    y, edges = histify(vals, bins, NORMALIZE)
    frac_outside = np.mean((vals < XLO) | (vals > XHI))
    ax.stairs(y, edges, color=color, alpha=0.15, fill=True)
    ax.stairs(y, edges, color=color, linewidth=1.8,
              label=f"{label} (n={len(vals)}, {100 * frac_outside:.1f}% outside)")
    print(f"{label}: n={len(vals)} mean={np.mean(vals):.3f} median={np.median(vals):.3f}")

ax.set_xlim(XLO, XHI)
if LOG_Y:
    ax.set_yscale('log')
ax.set_xlabel(f"baseline std, window [{RMS_LO}:{RMS_HI}], calibrated units")
ax.set_ylabel("count (n) per bin" if not NORMALIZE else "density (normalized to full population)"
              + (" -- log scale" if LOG_Y else ""))
ax.legend(fontsize=9)
ax.set_title(f"RMS distributions, window [{RMS_LO}:{RMS_HI}]" + ("" if NORMALIZE else " (raw counts)"))
plt.tight_layout()
plt.savefig(OUT_PATH, dpi=120)
print("saved", OUT_PATH)
