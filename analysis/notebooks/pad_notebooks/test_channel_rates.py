"""First 10 telemetry files accumulated in one ``processor`` run; then 10 PDF pages (one paddle, one axes each)."""
from __future__ import annotations

import sys
from pathlib import Path

import gondola as gon
import numpy as np
from matplotlib.backends.backend_pdf import PdfPages
import matplotlib.pyplot as plt

_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

import channel_rates  # noqa: E402


def main() -> int:
    files = gon.io.grace_get_telemetry_binaries(
        1766959800,
        1767039800,
        "/home/gaps/tof-data/antarctica/nextcloud/flight_2025-26",
    )

    paths = list(files[:10])
    n_avail = len(files)
    print(f"Accumulating first {len(paths)} of {n_avail} file(s); output_rates once at the end.\n")

    if not paths:
        print("No telemetry files in range.", file=sys.stderr)
        return 1

    proc = channel_rates.processor()
    first10_paddle_ids = proc.paddle_ids[:10]

    for i, path in enumerate(paths):
        print(path)
        proc.process(str(path), reset=(i == 0))

    timestamps, paddle_rates_history_hit, paddle_rates_history_beta = proc.output_rates()
    print(f"\nDone. Closed {len(timestamps)} rate window(s) over all files.")

    t_edges = np.asarray(timestamps, dtype=float)
    x_lo = x_hi = None
    if t_edges.size:
        lo, hi = float(t_edges.min()), float(t_edges.max())
        if lo == hi:
            pad = 15.0
            x_lo, x_hi = lo - pad, hi + pad
        else:
            x_lo, x_hi = lo, hi

    pdf_path = _HERE / "paddle_rates_first10_paddles.pdf"
    xlab = "Time (s) = timestamp48 / 1e8 (xlim: min to max of bin edges in data)"

    with PdfPages(pdf_path) as pdf:
        for pid in first10_paddle_ids:
            fig, ax = plt.subplots(figsize=(9, 4))
            ys = paddle_rates_history_hit.get(pid, [])
            ok = len(timestamps) >= 1 and len(ys) == len(timestamps)
            y_arr = np.asarray(ys, dtype=float) if ok else np.array([], dtype=float)
            if ok:
                ax.plot(
                    t_edges,
                    y_arr,
                    lw=0.9,
                    marker="o",
                    markersize=2.5,
                    markeredgewidth=0.3,
                    label="rate",
                )
                mean_hz = float(np.mean(y_arr))
                ax.axhline(mean_hz, color="C1", ls="--", lw=1.1, alpha=0.85, label="mean")
                ax.text(
                    0.02,
                    0.98,
                    f"mean = {mean_hz:.2f} Hz",
                    transform=ax.transAxes,
                    va="top",
                    ha="left",
                    fontsize=9,
                    color="C1",
                    bbox=dict(boxstyle="round,pad=0.25", facecolor="white", edgecolor="0.7", alpha=0.9),
                )
                ax.legend(loc="upper right", fontsize=8, framealpha=0.9)
            ax.set_ylabel("Hz")
            ax.set_xlabel(xlab)
            ax.set_title(
                f"Paddle {pid} — first {len(paths)} telemetry files (one continuous run)",
                fontsize=11,
            )
            ax.grid(True, alpha=0.3)
            if x_lo is not None:
                ax.set_xlim(x_lo, x_hi)
                ax.ticklabel_format(axis="x", useOffset=False, style="plain")
            fig.tight_layout()
            pdf.savefig(fig)
            plt.close(fig)

    print(f"Wrote {pdf_path} (10 pages, one full-size plot per paddle)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
