import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

data = np.load("DMrecovery/rms_distributions_4pop_data.npz")
good_rms = data["good_rms"]
mangled_naive_rms = data["mangled_naive_rms"]
mangled_corrected_quietmask_rms = data["mangled_corrected_quietmask_rms"]
mangled_corrected_700to900_rms = data["mangled_corrected_700to900_rms"]

RMS_LO, RMS_HI = 700, 900
XLO, XHI = 0, 25
bins = np.linspace(XLO, XHI, 126)
bin_width = bins[1] - bins[0]

def true_density(vals, bins):
    """Density normalized by the FULL population size (not just values inside
    the plotted range) -- so a population with a heavier out-of-range tail
    correctly shows LESS area within the visible window, instead of numpy's
    default density=True which renormalizes using only the in-range subset
    and silently overstates populations with more truncated tail."""
    counts, edges = np.histogram(vals, bins=bins)
    width = edges[1] - edges[0]
    return counts / (len(vals) * width), edges

fig, ax = plt.subplots(figsize=(11, 6.5))
pops = [
    (good_rms, "Good", "tab:green"),
    (mangled_naive_rms, "Mangled, naive", "tab:red"),
    (mangled_corrected_quietmask_rms, "Mangled, corrected, quiet_mask", "tab:blue"),
    (mangled_corrected_700to900_rms, "Mangled, corrected, [700:900]", "tab:orange"),
]
for vals, label, color in pops:
    dens, edges = true_density(vals, bins)
    frac_outside = np.mean((vals < XLO) | (vals > XHI))
    centers = (edges[:-1] + edges[1:]) / 2
    ax.stairs(dens, edges, color=color, alpha=0.15, fill=True)
    ax.stairs(dens, edges, color=color, linewidth=1.8,
              label=f"{label} (n={len(vals)}, {100*frac_outside:.1f}% outside [0,{XHI}])")
ax.set_xlim(XLO, XHI)
ax.set_xlabel(f"baseline std, window [{RMS_LO}:{RMS_HI}], calibrated units")
ax.set_ylabel("density (normalized to full population, not just visible range)")
ax.legend(fontsize=9)
ax.set_title(f"RMS distributions (n={len(good_rms)} good / {len(mangled_naive_rms)} mangled): quiet_mask vs fixed-window correction")
plt.tight_layout()
plt.savefig("DMrecovery/rms_distributions_4pop.png", dpi=120)
print("saved DMrecovery/rms_distributions_4pop.png")

for vals, label, _ in pops:
    print(f"{label}: mean={np.mean(vals):.3f} median={np.median(vals):.3f} frac_outside_0_25={np.mean((vals<XLO)|(vals>XHI)):.4f}")
