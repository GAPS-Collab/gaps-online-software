"""
Channel-9 RMS data-quality cut.

Implements the cut decided on in the ch9 RMS review: ch9 full-waveform RMS
varies board-to-board and drifts (roughly sinusoidally) over the course of a
flight, so a single global threshold is wrong. Instead, the mean ch9 RMS is
recomputed per RB every hour, and hits are cut if they fall outside that
RB's current hourly mean +/- a fixed ADC window.

See WFch9cutter.ipynb / WFchannel9MangleStudy.ipynb for the underlying study.
"""
from __future__ import annotations

from dataclasses import dataclass

import numpy as np
import polars as pl

TIMESTAMP48_TICKS_PER_SECOND = 1e8  # timestamp48 is stored in 10 ns ticks

DEFAULT_WINDOW_ADC = 65.0
DEFAULT_BIN_HOURS = 1.0


def elapsed_hours(time_stamp: np.ndarray, t0: float | None = None) -> np.ndarray:
    """Convert raw ``time_stamp`` (timestamp48, 10 ns ticks) to hours since ``t0``."""
    ts = np.asarray(time_stamp, dtype=float)
    if t0 is None:
        t0 = np.nanmin(ts)
    return (ts - t0) / TIMESTAMP48_TICKS_PER_SECOND / 3600.0


def hour_bin_index(hours: np.ndarray, bin_hours: float = DEFAULT_BIN_HOURS) -> np.ndarray:
    """Non-overlapping bin index, one new bin every ``bin_hours`` hours."""
    return np.floor(np.asarray(hours, dtype=float) / bin_hours).astype(np.int64)


@dataclass
class Ch9CutResult:
    df: pl.DataFrame          # hit-level df with elapsed_hours/hour_bin/ch9_cut_* columns added
    thresholds: pl.DataFrame  # one row per (rb_id, hour_bin): mean, lo, hi, n_hits


def add_ch9_cut(
    df: pl.DataFrame,
    *,
    window_adc: float = DEFAULT_WINDOW_ADC,
    bin_hours: float = DEFAULT_BIN_HOURS,
    rb_col: str = "rb_id",
    ch9_col: str = "ch9_rmsfull",
    ts_col: str = "time_stamp",
    t0: float | None = None,
) -> Ch9CutResult:
    """
    Add per-RB, per-hour ch9 RMS cut columns to ``df``.

    For every RB, the mean ch9 RMS is recomputed in ``bin_hours``-wide windows.
    Hits whose ch9 RMS falls outside mean +/- window_adc are flagged False in
    the returned ``ch9_cut_keep`` column.
    """
    hours = elapsed_hours(df[ts_col].to_numpy(), t0=t0)
    bins = hour_bin_index(hours, bin_hours)

    work = df.with_columns(
        [
            pl.Series("elapsed_hours", hours),
            pl.Series("hour_bin", bins),
        ]
    )

    thresholds = (
        work.group_by([rb_col, "hour_bin"])
        .agg(
            pl.col(ch9_col).mean().alias("rb_hour_mean_ch9"),
            pl.len().alias("n_hits"),
        )
        .with_columns(
            [
                (pl.col("rb_hour_mean_ch9") - window_adc).alias("ch9_cut_lo"),
                (pl.col("rb_hour_mean_ch9") + window_adc).alias("ch9_cut_hi"),
            ]
        )
        .sort([rb_col, "hour_bin"])
    )

    out = work.join(thresholds, on=[rb_col, "hour_bin"], how="left").with_columns(
        (
            (pl.col(ch9_col) >= pl.col("ch9_cut_lo"))
            & (pl.col(ch9_col) <= pl.col("ch9_cut_hi"))
        ).alias("ch9_cut_keep")
    )

    return Ch9CutResult(df=out, thresholds=thresholds)


def basic_quality_mask(
    df: pl.DataFrame,
    *,
    baseline_a_col: str = "baseline_a",
    baseline_b_col: str = "baseline_b",
    time_a_col: str = "time_a",
    time_b_col: str = "time_b",
    max_time: float = 350.0,
) -> pl.Series:
    """
    Reusable version of the trivial quality mask from ch9pulseCut.py:
    require nonzero baseline/time on both sides and time <= max_time.
    """
    expr = (
        (pl.col(baseline_a_col) != 0)
        & (pl.col(baseline_b_col) != 0)
        & (pl.col(time_a_col) != 0)
        & (pl.col(time_b_col) != 0)
        & (pl.col(time_a_col) <= max_time)
        & (pl.col(time_b_col) <= max_time)
    ).alias("quality_keep")
    return df.select(expr).to_series()
