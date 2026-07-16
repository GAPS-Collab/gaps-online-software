"""
Exercise ch9_cut.add_ch9_cut() against the cached ch9 hit-level parquet and
produce per-RB sanity-check plots: raw ch9 RMS vs time, the per-hour mean
used for that RB, and the +/- 100 ADC window around it, with kept/cut hits
colored differently.
"""
from __future__ import annotations

import sys
from pathlib import Path

import numpy as np
import polars as pl
from matplotlib.backends.backend_pdf import PdfPages
import matplotlib.pyplot as plt

_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

import ch9_cut  # noqa: E402

DATA_PATH = _HERE / "saved_dfs" / "wf_ch9_cut_data_from_processor40hr.parquet"
NEEDED_COLUMNS = ["rb_id", "ch9_rmsfull", "time_stamp"]

MAX_POINTS_PER_RB_PLOT = 8_000


def format_pct(n_kept: int, n_total: int, max_decimals: int = 6) -> str:
    """Percent-kept string with just enough decimals to not round to 100.0% when it isn't."""
    pct = 100.0 * n_kept / n_total
    if n_kept >= n_total:
        return "100.0%"
    decimals = 1
    while decimals < max_decimals and round(pct, decimals) >= 100.0:
        decimals += 1
    return f"{pct:.{decimals}f}%"


def main() -> int:
    print(f"Loading columns {NEEDED_COLUMNS} from {DATA_PATH} ...")
    df = pl.read_parquet(DATA_PATH, columns=NEEDED_COLUMNS)
    print(f"Loaded {len(df):,} hits across {df['rb_id'].n_unique()} RBs.")

    result = ch9_cut.add_ch9_cut(df)
    out = result.df

    n_total = len(out)
    n_kept = int(out["ch9_cut_keep"].sum())
    print(f"\nOverall: kept {n_kept:,} / {n_total:,} ({format_pct(n_kept, n_total)})")

    per_rb = (
        out.group_by("rb_id")
        .agg(
            pl.len().alias("n_hits"),
            pl.col("ch9_cut_keep").sum().alias("n_kept"),
        )
        .sort("rb_id")
    )
    per_rb = per_rb.with_columns(
        pl.Series(
            "pct_kept",
            [format_pct(k, n) for k, n in zip(per_rb["n_kept"], per_rb["n_hits"])],
        )
    )
    print("\nPer-RB summary:")
    with pl.Config(tbl_rows=-1):
        print(per_rb)

    rbs = per_rb["rb_id"].to_list()
    print(f"\nPlotting all {len(rbs)} RBs into the PDF ...")

    pdf_path = _HERE / "ch9_cut_sanity_check.pdf"
    with PdfPages(pdf_path) as pdf:
        for rb in rbs:
            rb_df = out.filter(pl.col("rb_id") == rb).sort("elapsed_hours")
            rb_thresh = result.thresholds.filter(pl.col("rb_id") == rb).sort("hour_bin")

            hours = rb_df["elapsed_hours"].to_numpy()
            ch9 = rb_df["ch9_rmsfull"].to_numpy()
            keep = rb_df["ch9_cut_keep"].to_numpy()

            stride = max(1, len(hours) // MAX_POINTS_PER_RB_PLOT)

            fig, ax = plt.subplots(figsize=(10, 5))
            ax.scatter(
                hours[keep][::stride],
                ch9[keep][::stride],
                s=4,
                alpha=0.35,
                color="tab:blue",
                linewidths=0,
                label="kept",
                rasterized=True,
            )
            ax.scatter(
                hours[~keep][::stride],
                ch9[~keep][::stride],
                s=4,
                alpha=0.6,
                color="tab:red",
                linewidths=0,
                label="cut",
                rasterized=True,
            )

            bin_hours = rb_thresh["hour_bin"].to_numpy() * ch9_cut.DEFAULT_BIN_HOURS
            mean_ch9 = rb_thresh["rb_hour_mean_ch9"].to_numpy()
            lo = rb_thresh["ch9_cut_lo"].to_numpy()
            hi = rb_thresh["ch9_cut_hi"].to_numpy()

            ax.step(bin_hours, mean_ch9, where="post", color="black", lw=1.5, label="hourly mean")
            ax.fill_between(
                bin_hours,
                lo,
                hi,
                step="post",
                color="tab:orange",
                alpha=0.25,
                label=f"mean +/- {ch9_cut.DEFAULT_WINDOW_ADC:g} ADC",
            )

            n_hits = len(rb_df)
            n_kept_rb = int(keep.sum())
            ax.text(
                0.02,
                0.98,
                f"kept {n_kept_rb:,} / {n_hits:,} ({format_pct(n_kept_rb, n_hits)})",
                transform=ax.transAxes,
                va="top",
                ha="left",
                fontsize=9,
                bbox=dict(boxstyle="round,pad=0.25", facecolor="white", edgecolor="0.7", alpha=0.9),
            )

            # zoom to the mean +/- window band (with padding) so a handful of far-off
            # cut outliers (e.g. near 0) don't blow out the y-scale and hide the band
            y_lo, y_hi = np.nanmin(lo), np.nanmax(hi)
            y_pad = 0.15 * (y_hi - y_lo)
            ax.set_ylim(y_lo - y_pad, y_hi + y_pad)

            ax.set_xlabel("Elapsed time since first event (hours)")
            ax.set_ylabel("Ch9 full-waveform RMS")
            ax.set_title(f"RB {rb} — ch9 RMS cut sanity check")
            ax.grid(alpha=0.25)
            ax.legend(loc="upper right", fontsize=8, framealpha=0.9)
            fig.tight_layout()
            pdf.savefig(fig)
            plt.close(fig)

    print(f"\nWrote {pdf_path} ({len(rbs)} page(s), one per RB)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
