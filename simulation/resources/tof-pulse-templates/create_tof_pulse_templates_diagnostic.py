#!/usr/bin/env python3

"""
Create TOF waveform pulse templates from GAPS TOF data.

The input is a Gondola TOF packet stream containing TofEvents. For each
paddle, side, charge interval, and position interval, calibrated waveforms
are aligned at their 25% CFD crossing and accumulated into an average
waveform and RMS.

The resulting templates are stored as TofPulseTemplate packets in a
standard Gondola TofPacket stream.

Historical CreatePulseMap behavior is deliberately preserved where
relevant, in particular:

  * CFD alignment fraction: 0.25
  * charge binning:
        6 x 12 pC
       12 x 20 pC
       12 x 35 pC
  * position template selection:
        round(position_from_A_mm / 100)
  * waveform histogram range:
        -8 <= time < 100 ns
       -20 <= voltage < 800 mV
  * 1 ns time bins

Position is measured from paddle side A.
"""

from __future__ import annotations

import argparse
import collections
import math
import os
import tempfile
from dataclasses import dataclass
from pathlib import Path

import numpy as np

from gondola.calibration import load_rb_calibrations
from gondola.events import TofEvent
from gondola.io import TofPacketReader, TofPacketWriter
from gondola.packets import TofPacketType

# Adjust this import if TofPulseTemplate was exposed somewhere else.
from gondola.tof import TofPulseTemplate


# =============================================================================
# Configuration
# =============================================================================

CFD_FRACTION = 0.25

TIME_MIN = -8.0
TIME_MAX = 100.0
TIME_STEP = 1.0

N_TIME_BINS = int((TIME_MAX - TIME_MIN) / TIME_STEP)

VOLTAGE_MIN = -20.0
VOLTAGE_MAX = 800.0

POSITION_STEP_MM = 100.0

SIDE_A = 0
SIDE_B = 1


# =============================================================================
# Charge bins
# =============================================================================

@dataclass(frozen=True)
class ChargeBin:
    low: float
    high: float


def make_charge_bins() -> list[ChargeBin]:
    """
    Historical CreatePulseMap charge bins.

      6 x 12 pC
     12 x 20 pC
     12 x 35 pC

    giving a total range of 0 <= Q < 732 pC.
    """

    bins: list[ChargeBin] = []

    low = 0.0

    for i in range(30):
        if i < 6:
            width = 12.0
        elif i < 18:
            width = 20.0
        else:
            width = 35.0

        bins.append(
            ChargeBin(
                low=low,
                high=low + width,
            )
        )

        low += width

    return bins


CHARGE_BINS = make_charge_bins()


def find_charge_bin(charge: float) -> int | None:
    """
    Return the historical charge-bin index.

    Bins use [low, high).
    """

    if not math.isfinite(charge):
        return None

    for index, charge_bin in enumerate(CHARGE_BINS):
        if charge_bin.low <= charge < charge_bin.high:
            return index

    return None


# =============================================================================
# Position binning
# =============================================================================

def position_chunk_index(
    position_mm: float,
    paddle_length_mm: float,
) -> int | None:
    """
    Return the historical CreatePulseMap position-template index.

    position_mm is measured from paddle side A.

    The old C++ code used:

        round(position_mm / 100)

    where std::round() rounds positive half-integers upward.

    Python's round() uses bankers' rounding, so we reproduce the C++
    behavior explicitly.

    The old code also rejected:

      * positions <= 0
      * the exact paddle center, which represented a missing position
        in the historical data
      * positions mapping beyond the number of booked templates
    """

    if not math.isfinite(position_mm):
        return None

    if not math.isfinite(paddle_length_mm):
        return None

    if paddle_length_mm <= 0.0:
        return None

    if position_mm <= 0.0:
        return None

    # Historical CreatePulseMap treated the exact paddle center as
    # "no reconstructed position".
    if position_mm == 0.5 * paddle_length_mm:
        return None

    chunk = math.floor(
        position_mm / POSITION_STEP_MM + 0.5
    )

    n_chunks = math.ceil(
        paddle_length_mm / POSITION_STEP_MM
    )

    if chunk < 0 or chunk >= n_chunks:
        return None

    return chunk


def position_interval(
    chunk: int,
    paddle_length_mm: float,
) -> tuple[float, float]:
    """
    Return the actual position interval selecting a historical template.

    This is intentionally NOT simply:

        [chunk * 100, (chunk + 1) * 100)

    because historical CreatePulseMap selected templates using

        round(position / 100).

    Therefore the selection regions are approximately

        chunk 0 : [   0,  50) mm
        chunk 1 : [  50, 150) mm
        chunk 2 : [ 150, 250) mm
        ...

    clipped where appropriate by the physical paddle length.

    Keeping these physical selection intervals in TofPulseTemplate makes
    the new data product self-describing and avoids encoding the historical
    bin-number convention in the consumer.
    """

    if chunk == 0:
        low = 0.0
    else:
        low = (
            chunk - 0.5
        ) * POSITION_STEP_MM

    high = min(
        (chunk + 0.5) * POSITION_STEP_MM,
        paddle_length_mm,
    )

    return low, high


# =============================================================================
# CFD alignment
# =============================================================================

def cfd_crossing_time(
    times: np.ndarray,
    voltages: np.ndarray,
    fraction: float = CFD_FRACTION,
) -> float | None:
    """
    Find the first rising-edge crossing of fraction * max(voltage).

    The crossing time is linearly interpolated between the samples
    immediately below and above threshold.

    This mirrors the historical CreatePulseMap implementation.
    """

    if times.size == 0 or voltages.size == 0:
        return None

    if times.size != voltages.size:
        return None

    finite = (
        np.isfinite(times)
        & np.isfinite(voltages)
    )

    if not np.all(finite):
        times = times[finite]
        voltages = voltages[finite]

    if times.size < 2:
        return None

    maximum = float(np.max(voltages))
    threshold = fraction * maximum

    above = np.flatnonzero(
        voltages[1:] > threshold
    )

    if above.size == 0:
        return None

    # +1 because the search started at voltages[1:].
    index = int(above[0]) + 1

    before = float(voltages[index - 1])
    after = float(voltages[index])

    t_before = float(times[index - 1])
    t_after = float(times[index])

    if after != before:
        interpolation = (
            threshold - before
        ) / (
            after - before
        )
    else:
        interpolation = 0.0

    # Noise can occasionally move the interpolation fraction slightly
    # outside the sample interval.
    interpolation = float(
        np.clip(interpolation, 0.0, 1.0)
    )

    return (
        t_before
        + interpolation * (t_after - t_before)
    )


# =============================================================================
# Template accumulation
# =============================================================================

@dataclass
class TemplateAccumulator:
    """
    Streaming waveform statistics for one pulse template.

    We keep only:

        count
        sum
        sum of squares

    for every time bin. This avoids retaining individual waveforms.
    """

    count: np.ndarray
    sum: np.ndarray
    sum_sq: np.ndarray
    n_waveforms: int

    @classmethod
    def create(cls) -> "TemplateAccumulator":
        return cls(
            count=np.zeros(
                N_TIME_BINS,
                dtype=np.uint64,
            ),
            sum=np.zeros(
                N_TIME_BINS,
                dtype=np.float64,
            ),
            sum_sq=np.zeros(
                N_TIME_BINS,
                dtype=np.float64,
            ),
            n_waveforms=0,
        )

    def add(
        self,
        times: np.ndarray,
        voltages: np.ndarray,
    ) -> None:
        """
        Add waveform samples to the template.

        This reproduces filling the historical TH2 directly with samples,
        but uses np.bincount rather than np.add.at for substantially lower
        Python/NumPy overhead.
        """

        valid = (
            np.isfinite(times)
            & np.isfinite(voltages)
            & (times >= TIME_MIN)
            & (times < TIME_MAX)
            & (voltages >= VOLTAGE_MIN)
            & (voltages < VOLTAGE_MAX)
        )

        if not np.any(valid):
            return

        # Count accepted waveforms, not individual time-bin samples.
        self.n_waveforms += 1

        valid_times = times[valid]
        valid_voltages = voltages[valid]

        bins = np.floor(
            (valid_times - TIME_MIN)
            / TIME_STEP
        ).astype(np.intp)

        self.count += np.bincount(
            bins,
            minlength=N_TIME_BINS,
        ).astype(
            np.uint64,
            copy=False,
        )

        self.sum += np.bincount(
            bins,
            weights=valid_voltages,
            minlength=N_TIME_BINS,
        )

        self.sum_sq += np.bincount(
            bins,
            weights=valid_voltages * valid_voltages,
            minlength=N_TIME_BINS,
        )

    def finalize(
        self,
    ) -> tuple[np.ndarray, np.ndarray]:
        """
        Return mean and RMS for every time bin.

        Empty bins are left at zero, matching the useful behavior of the
        old ROOT profiles for the serialized template.
        """

        mean = np.zeros(
            N_TIME_BINS,
            dtype=np.float32,
        )

        rms = np.zeros(
            N_TIME_BINS,
            dtype=np.float32,
        )

        occupied = self.count > 0

        if not np.any(occupied):
            return mean, rms

        count = self.count[occupied].astype(
            np.float64
        )

        local_mean = (
            self.sum[occupied] / count
        )

        variance = (
            self.sum_sq[occupied] / count
            - local_mean * local_mean
        )

        # Protect against tiny negative values from floating-point
        # cancellation.
        variance = np.maximum(
            variance,
            0.0,
        )

        mean[occupied] = local_mean.astype(
            np.float32
        )

        rms[occupied] = np.sqrt(
            variance
        ).astype(
            np.float32
        )

        return mean, rms

    @property
    def n_samples(self) -> int:
        return int(self.count.sum())


# =============================================================================
# Template key
# =============================================================================

@dataclass(frozen=True)
class TemplateKey:
    paddle_id: int
    side: int
    charge_bin: int
    position_chunk: int


# =============================================================================
# Event input
# =============================================================================

def iter_events(
    input_path: Path,
):
    """
    Yield TofEvents from a Gondola packet stream.
    """

    reader = TofPacketReader(
        str(input_path),
        filter=TofPacketType.TofEvent,
    )

    for packet in reader:
        yield TofEvent.unpack(packet)


# =============================================================================
# Event processing
# =============================================================================

def process_waveform_side(
    *,
    paddle_id: int,
    side: int,
    times: np.ndarray,
    voltages: np.ndarray,
    charge: float,
    position_mm: float,
    paddle_length_mm: float,
    accumulators: dict[
        TemplateKey,
        TemplateAccumulator,
    ],
) -> None:
    """
    Process one side of one calibrated paddle waveform.
    """

    if times.size < 2 or voltages.size < 2:
        return

    if times.size != voltages.size:
        return

    charge_bin = find_charge_bin(charge)

    if charge_bin is None:
        return

    position_chunk = position_chunk_index(
        position_mm,
        paddle_length_mm,
    )

    if position_chunk is None:
        return

    crossing_time = cfd_crossing_time(
        times,
        voltages,
    )

    if crossing_time is None:
        return

    # Preserve the historical saturation rejection.
    #
    # CreatePulseMap rejected a waveform if the first sample above
    # the CFD threshold itself exceeded the histogram voltage range.
    maximum = float(np.max(voltages))
    threshold = CFD_FRACTION * maximum

    above = np.flatnonzero(
        voltages[1:] > threshold
    )

    if above.size == 0:
        return

    first_above = int(above[0]) + 1

    if voltages[first_above] > VOLTAGE_MAX:
        return

    aligned_times = (
        times - crossing_time
    )

    key = TemplateKey(
        paddle_id=paddle_id,
        side=side,
        charge_bin=charge_bin,
        position_chunk=position_chunk,
    )

    accumulator = accumulators.get(key)

    if accumulator is None:
        accumulator = TemplateAccumulator.create()
        accumulators[key] = accumulator

    accumulator.add(
        aligned_times,
        voltages,
    )


# Why waveforms were not used, by (reason, rb_id, paddle_id). A whole
# readout board can drop out silently otherwise, e.g. when its
# calibration file is missing.
SKIPPED: collections.Counter = collections.Counter()
USED: collections.Counter = collections.Counter()


def process_event(
    event: TofEvent,
    calibrations,
    accumulators: dict[
        TemplateKey,
        TemplateAccumulator,
    ],
    paddle_lengths: dict[int, float],
) -> None:
    """
    Add all usable waveforms from one TofEvent to the template bank.
    """

    # Move hits out of the individual RBEvents first. The RBEvents retain
    # their waveform data, while event.hits becomes the single hit list.
    event.move_hits()

    hits_by_paddle: dict[int, list] = {}

    for hit in event.hits:
        hits_by_paddle.setdefault(
            hit.paddle_id,
            [],
        ).append(hit)

    for waveform in event.waveforms:
        paddle_id = waveform.paddle_id

        if paddle_id == 0:
            SKIPPED[("paddle id 0", waveform.rb_id, 0)] += 1
            continue

        hits = hits_by_paddle.get(
            paddle_id,
            [],
        )

        # Preserve the conservative association rule used in the Python
        # rewrite: only use a paddle when there is exactly one matching hit.
        if len(hits) != 1:
            SKIPPED[(f"{len(hits)} matching hits", waveform.rb_id, paddle_id)] += 1
            continue

        hit = hits[0]

        calibration = calibrations.get(
            waveform.rb_id
        )

        if calibration is None:
            if not any(r == "no calibration" and rb == waveform.rb_id for r, rb, _ in SKIPPED):
                print(f"WARNING: no calibration for RB {waveform.rb_id}, its waveforms are skipped")
            SKIPPED[("no calibration", waveform.rb_id, paddle_id)] += 1
            continue

        try:
            waveform.calibrate(
                calibration
            )
        except Exception as error:
            # A bad/missing calibration for one waveform should not abort
            # creation of the entire pulse map.
            if not any(r == "calibration failed" and rb == waveform.rb_id for r, rb, _ in SKIPPED):
                print(f"WARNING: calibration of RB {waveform.rb_id} failed: {error}")
            SKIPPED[("calibration failed", waveform.rb_id, paddle_id)] += 1
            continue

        paddle_length_mm = float(
            hit.paddle_len
        )

        if (
            not math.isfinite(paddle_length_mm)
            or paddle_length_mm <= 0.0
        ):
            SKIPPED[("no paddle length", waveform.rb_id, paddle_id)] += 1
            continue

        USED[(waveform.rb_id, paddle_id)] += 1

        paddle_lengths[paddle_id] = (
            paddle_length_mm
        )

        # Gondola's TofHit.pos convention is distance from paddle side A.
        position_mm = float(
            hit.pos
        )

        # ------------------------------------------------------------------
        # Side A
        # ------------------------------------------------------------------

        process_waveform_side(
            paddle_id=paddle_id,
            side=SIDE_A,
            times=np.asarray(
                waveform.times_a,
                dtype=np.float32,
            ),
            voltages=np.asarray(
                waveform.voltages_a,
                dtype=np.float32,
            ),
            charge=float(
                hit.charge_a
            ),
            position_mm=position_mm,
            paddle_length_mm=paddle_length_mm,
            accumulators=accumulators,
        )

        # ------------------------------------------------------------------
        # Side B
        # ------------------------------------------------------------------

        process_waveform_side(
            paddle_id=paddle_id,
            side=SIDE_B,
            times=np.asarray(
                waveform.times_b,
                dtype=np.float32,
            ),
            voltages=np.asarray(
                waveform.voltages_b,
                dtype=np.float32,
            ),
            charge=float(
                hit.charge_b
            ),
            position_mm=position_mm,
            paddle_length_mm=paddle_length_mm,
            accumulators=accumulators,
        )


# =============================================================================
# TofPulseTemplate creation
# =============================================================================

def make_template(
    key: TemplateKey,
    accumulator: TemplateAccumulator,
    paddle_length_mm: float,
) -> TofPulseTemplate:
    """
    Convert one accumulated waveform template into a TofPulseTemplate.
    """

    mean, rms = accumulator.finalize()

    charge_bin = CHARGE_BINS[
        key.charge_bin
    ]

    position_low, position_high = (
        position_interval(
            key.position_chunk,
            paddle_length_mm,
        )
    )

    template = TofPulseTemplate()

    template.paddle_id = key.paddle_id
    template.side = key.side
    template.n_waveforms = accumulator.n_waveforms

    template.position_low = position_low
    template.position_high = position_high

    template.charge_low = charge_bin.low
    template.charge_high = charge_bin.high

    # Histogram bin centers:
    #
    #   [-8, -7), [-7, -6), ...
    #
    # correspond to
    #
    #   -7.5, -6.5, ...
    template.time_start = (
        TIME_MIN + 0.5 * TIME_STEP
    )

    template.time_step = TIME_STEP

    template.mean = mean.tolist()
    template.rms = rms.tolist()

    return template


# =============================================================================
# Output
# =============================================================================

def write_templates(
    output_path: Path,
    accumulators: dict[
        TemplateKey,
        TemplateAccumulator,
    ],
    paddle_lengths: dict[int, float],
) -> int:
    """
    Write templates using Gondola's standard TofPacketWriter.

    TofPacketWriter deliberately uses the normal Gondola run-file naming
    convention. We write into a temporary directory and rename the completed
    packet stream afterward so that the final filename clearly identifies
    this as a pulse-template calibration product.
    """

    output_path = output_path.resolve()

    output_path.parent.mkdir(
        parents=True,
        exist_ok=True,
    )

    if output_path.exists():
        raise FileExistsError(
            f"Output file already exists: "
            f"{output_path}"
        )

    n_templates = 0

    # Put the temporary directory beside the final output. This keeps the
    # final move on the same filesystem.
    with tempfile.TemporaryDirectory(
        prefix=".pulse_templates_",
        dir=output_path.parent,
    ) as tmp:
        tmp_path = Path(tmp)

        # The run ID is irrelevant here because the generated filename is
        # temporary. We deliberately use TofPacketWriter so that this data
        # product goes through Gondola's standard packet-writing machinery.
        writer = TofPacketWriter(
            str(tmp_path),
            0,
        )

        for key in sorted(
            accumulators,
            key=lambda item: (
                item.paddle_id,
                item.side,
                item.charge_bin,
                item.position_chunk,
            ),
        ):
            accumulator = accumulators[key]

            if accumulator.n_samples == 0:
                continue

            paddle_length_mm = (
                paddle_lengths.get(
                    key.paddle_id
                )
            )

            if paddle_length_mm is None:
                continue

            template = make_template(
                key,
                accumulator,
                paddle_length_mm,
            )

            packet = template.pack()

            writer.add_tof_packet(
                packet
            )

            n_templates += 1

        # TofPacketWriter currently has no explicit Python close() method.
        # Dropping the PyO3 object closes the underlying Rust File before
        # we rename it.
        del writer

        generated_files = list(
            tmp_path.glob("*.tof.gaps")
        )

        if len(generated_files) != 1:
            raise RuntimeError(
                "Expected TofPacketWriter to create exactly "
                f"one file, found {len(generated_files)}: "
                f"{generated_files}"
            )

        generated_file = generated_files[0]

        os.replace(
            generated_file,
            output_path,
        )

    return n_templates



# =============================================================================
# Optional accumulator diagnostics
# =============================================================================

def diagnose_accumulator(
    accumulators: dict[TemplateKey, TemplateAccumulator],
    paddle_lengths: dict[int, float],
    *,
    paddle_id: int,
    side: int,
    position_mm: float,
    charge_pc: float,
    output_path: Path,
) -> None:
    """Plot mean, RMS, and sample occupancy for one template accumulator."""
    import matplotlib.pyplot as plt

    charge_bin = find_charge_bin(charge_pc)
    length = paddle_lengths.get(paddle_id)
    if charge_bin is None or length is None:
        print("Diagnostic: charge outside range or paddle not found.")
        return
    chunk = position_chunk_index(position_mm, length)
    if chunk is None:
        print("Diagnostic: position does not select a valid template.")
        return

    key = TemplateKey(paddle_id, side, charge_bin, chunk)
    acc = accumulators.get(key)
    if acc is None or acc.n_samples == 0:
        print(f"Diagnostic: no accumulated samples for {key}.")
        return

    mean, rms = acc.finalize()
    counts = acc.count
    times = TIME_MIN + (np.arange(N_TIME_BINS) + 0.5) * TIME_STEP
    low, high = position_interval(chunk, length)
    qbin = CHARGE_BINS[charge_bin]
    side_name = "A" if side == SIDE_A else "B"

    print("\nAccumulator diagnostic")
    print("----------------------")
    print(f"Paddle / side    : {paddle_id} / {side_name}")
    print(f"Position         : [{low:g}, {high:g}) mm")
    print(f"Charge           : [{qbin.low:g}, {qbin.high:g}) pC")
    print(f"Total samples    : {acc.n_samples:,}")
    print(f"Total waveforms  : {acc.n_waveforms:,}")
    print(f"Occupied bins    : {np.count_nonzero(counts)} / {N_TIME_BINS}")
    print(f"Min/max occupancy: {counts.min():,} / {counts.max():,}")
    print("\n Time [ns]   Samples   Mean [mV]   RMS [mV]")
    for t in (0.5, 20.5, 50.5, 90.5, 95.5, 96.5, 97.5, 98.5, 99.5):
        i = int(round((t - (TIME_MIN + 0.5 * TIME_STEP)) / TIME_STEP))
        print(f"{times[i]:10.1f} {int(counts[i]):9,d} "
              f"{mean[i]:11.4f} {rms[i]:10.4f}")

    fig, (ax, occupancy) = plt.subplots(
        2, 1, sharex=True, figsize=(11, 7),
        gridspec_kw={"height_ratios": [2, 1]},
        constrained_layout=True,
    )
    ax.plot(times, mean, label="Mean")
    ax.fill_between(times, mean-rms, mean+rms, alpha=0.2, label="Mean ± RMS")
    ax.axvline(0, linestyle="--", color="gray", label="Alignment")
    ax.set_ylabel("Voltage [mV]")
    ax.legend()
    ax.grid(alpha=0.2)
    occupancy.step(times, counts, where="mid")
    occupancy.set_xlabel("Aligned time [ns]")
    occupancy.set_ylabel("Samples / bin")
    occupancy.grid(alpha=0.2)
    fig.suptitle(
        f"Paddle {paddle_id}, side {side_name}, "
        f"position [{low:g}, {high:g}) mm, "
        f"charge [{qbin.low:g}, {qbin.high:g}) pC"
    )
    output_path.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(output_path, dpi=160)
    plt.close(fig)
    print(f"Diagnostic plot: {output_path}")


# =============================================================================
# Command line
# =============================================================================

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Create Gondola TOF pulse-template packets "
            "from TOF event data."
        )
    )

    parser.add_argument(
        "input",
        type=Path,
        help=(
            "Input .tof.gaps file or directory containing "
            "TofEvent packets."
        ),
    )

    parser.add_argument(
        "calibration",
        type=Path,
        help=(
            "Directory containing readout-board calibration files."
        ),
    )

    parser.add_argument(
        "output",
        type=Path,
        help=(
            "Output pulse-template packet stream, e.g. "
            "tof_pulse_templates.tof.gaps."
        ),
    )

    parser.add_argument(
        "-n",
        "--nevents",
        type=int,
        default=None,
        help=(
            "Maximum number of TofEvents to process. "
            "By default all events are processed."
        ),
    )

    diagnostic = parser.add_argument_group("optional accumulator diagnostic")
    diagnostic.add_argument("--diagnostic-paddle", type=int)
    diagnostic.add_argument("--diagnostic-side", type=str.upper, choices=("A", "B"))
    diagnostic.add_argument("--diagnostic-position", type=float,
                            help="Distance in mm from side A.")
    diagnostic.add_argument("--diagnostic-charge", type=float,
                            help="Charge in pC.")
    diagnostic.add_argument("--diagnostic-output", type=Path,
                            default=Path("pulse_diagnostic.png"),
                            help="PNG output (default: pulse_diagnostic.png).")

    args = parser.parse_args()

    if (
        args.nevents is not None
        and args.nevents < 0
    ):
        parser.error(
            "--nevents must be >= 0"
        )

    selection = (
        args.diagnostic_paddle, args.diagnostic_side,
        args.diagnostic_position, args.diagnostic_charge,
    )
    if any(v is not None for v in selection) and not all(
        v is not None for v in selection
    ):
        parser.error("Specify all four diagnostic selection options together.")
    if args.diagnostic_paddle is not None and not 0 <= args.diagnostic_paddle <= 255:
        parser.error("--diagnostic-paddle must be in [0, 255].")

    return args


# =============================================================================
# Main
# =============================================================================

def main() -> None:
    args = parse_args()

    print(
        f"Loading RB calibrations from "
        f"{args.calibration} ..."
    )

    calibrations = load_rb_calibrations(
        args.calibration
    )

    print(
        f"Loaded {len(calibrations)} "
        f"RB calibrations."
    )

    accumulators: dict[
        TemplateKey,
        TemplateAccumulator,
    ] = {}

    # Paddle lengths are stable geometry information. Remember the value
    # encountered in the data so that the final template can carry truthful
    # physical position-selection bounds.
    paddle_lengths: dict[int, float] = {}

    n_events = 0

    print(
        f"Reading events from {args.input} ..."
    )

    for event in iter_events(args.input):
        if (
            args.nevents is not None
            and n_events >= args.nevents
        ):
            break

        process_event(
            event,
            calibrations,
            accumulators,
            paddle_lengths,
        )

        n_events += 1

        if n_events % 10_000 == 0:
            print(
                f"Processed {n_events:,} events; "
                f"{len(accumulators):,} populated templates"
            )

    print(
        f"Finished processing {n_events:,} events."
    )

    # which waveforms were used, and whether a paddle lost all of them
    print("\nWaveform summary")
    print(f"  used: {sum(USED.values()):,} waveforms on {len(USED)} paddles")
    unused = sorted({(rb, pid) for _, rb, pid in SKIPPED if pid and USED[(rb, pid)] == 0})
    if unused:
        print("  Paddles with waveforms of which NONE was used (they get no templates):")
        for rb, pid in unused:
            reasons = {r: n for (r, b, p), n in SKIPPED.items() if b == rb and p == pid}
            print(f"    RB {rb:3d}, paddle {pid:3d}: {reasons}")
    else:
        print("  Every paddle with waveforms has used waveforms.")
    if SKIPPED:
        totals = collections.Counter()
        for (reason, _, _), n in SKIPPED.items():
            totals[reason] += n
        print("  Skipped waveforms by reason:")
        for reason, n in totals.most_common():
            note = " (expected: a readout board delivers the waveforms of all its channels, also of paddles without a hit)" if reason == "0 matching hits" else ""
            print(f"    {reason:20s} {n:,}{note}")
        print("  Paddles with the most skipped waveforms (skipped / used):")
        for (reason, rb, pid), n in SKIPPED.most_common(10):
            used = USED[(rb, pid)]
            print(f"    {reason:20s} RB {rb:3d}, paddle {pid:3d}: {n:,} / {used:,} ({n/max(n + used, 1):.1%} skipped)")

    if args.diagnostic_paddle is not None:
        diagnose_accumulator(
            accumulators,
            paddle_lengths,
            paddle_id=args.diagnostic_paddle,
            side=SIDE_A if args.diagnostic_side == "A" else SIDE_B,
            position_mm=args.diagnostic_position,
            charge_pc=args.diagnostic_charge,
            output_path=args.diagnostic_output,
        )

    print(
        f"Writing {len(accumulators):,} "
        f"populated template accumulators ..."
    )

    n_templates = write_templates(
        args.output,
        accumulators,
        paddle_lengths,
    )

    print(
        f"Wrote {n_templates:,} pulse templates to "
        f"{args.output}"
    )


if __name__ == "__main__":
    main()