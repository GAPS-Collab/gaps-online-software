#!/usr/bin/env python3
from __future__ import annotations

import argparse
import time
from pathlib import Path

import numpy as np
import polars as pl


def corrected_rms_sq(mu, sigB, Num=200):
    val = ((Num - mu**2) / Num) * sigB**2 - mu**2
    if not np.isfinite(val):
        return np.nan

    if val < 0:
        return np.nan
    val = np.sqrt(val)
    return val
    

FLIGHT_CALIB = Path(
    "/mnt/ucla-gaps-nas1/tof-data/antarctica/flight_ssd_data/calib/251222_010511UTC"
)

GROUND_CALIB = Path(
    "/mnt/ucla-gaps-nas1/tof-data/antarctica/skua_hdd/data/calib/251123_215129UTC"
)

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Process TOF event files for the channel-9 mangling study."
    )
    parser.add_argument(
        "--file-list",
        default="flight_14hr",
        help=(
            "Run list name from run_lists/<name>.txt, or a direct path to a txt file. "
            "Defaults to flight_14hours."
        ),
    )
    parser.add_argument(
        "--outdir",

        default="saved_dfs",
        help="Directory for the parquet output. Defaults to saved_dfs.",
    )
    parser.add_argument(
        "--output",
        default="wf_ch9_cut_data_from_processor.parquet",
        help="Output parquet filename. Defaults to wf_ch9_cut_data_from_processor.parquet.",
    )
    parser.add_argument(
        "--num",
        type=int,
        default=200,
        help="Num value used in corrected_rms. Defaults to 200.",
    )
    parser.add_argument(
        "--ch9-rms-nbins",
        type=int,
        default=100,
        help="Number of leading ch9 waveform bins for the RMS cut. Defaults to 100.",
    )
    parser.add_argument(
        "--sleep-time",
        type=float,
        default=0.0,
        help="Seconds to sleep after each input file. Defaults to 0.",
    )
    return parser.parse_args()


def get_mangling_stati(gon) -> set:
    return {
        gon.events.EventStatus.ChnSyncErrors,
        gon.events.EventStatus.CellSyncErrors,
        gon.events.EventStatus.CellAndChnSyncErrors,
    }


def rb_has_mangling(rb, mangling_stati: set) -> bool:
    return rb.status in mangling_stati


def read_files_txt(path: Path) -> list[str]:
    return [
        line.strip()
        for line in path.read_text().splitlines()
        if line.strip() and not line.lstrip().startswith("#")
    ]


def resolve_file_list(file_list: str, script_dir: Path) -> Path:
    path = Path(file_list)
    if path.exists():
        return path

    run_list_path = script_dir / "run_lists" / f"{file_list}.txt"
    if run_list_path.exists():
        return run_list_path

    raise FileNotFoundError(
        f"Could not find '{file_list}' or '{run_list_path}'. "
        "Expected a path or run_lists/<name>.txt."
    )


def load_calibration(gon, file_list_name: str):
    if file_list_name.startswith("ground"):
        data_label = "Ground"
        calib_path = GROUND_CALIB
    else:
        data_label = "Flight"
        calib_path = FLIGHT_CALIB

    print(f"loading {data_label.lower()} calibration: {calib_path}")
    calib = gon.calibration.load_rb_calibrations(calib_path)
    return data_label, calib


def corrected_rms(mu, sig_b, num: int = 200):
    val = ((num - mu**2) / num) * sig_b**2 - mu**2

    if not np.isfinite(val):
        return np.nan

    if val < 0:
        return np.nan

    return np.sqrt(val)

def process_files(
    gon,
    files: list[str],
    num: int,
    ch9_rms_nbins: int,
    sleep_time: float,
):
    packcnt_ev = 0
    mangling_count = 0
    rb_event_count = 0

    mangling_stati = get_mangling_stati(gon)
    paddles = gon.db.TofPaddle.all()
    paddle_to_rb = {paddle.paddle_id: paddle.rb_id for paddle in paddles}

    baseline_a_rms_vals_corrected_sqrt = []
    peak_a_arr = []
    ch9_rmsfull_arr = []
    all_tot_data_arr_a = []
    all_tot_data_arr_b = []
    baseline_b_rms_vals_corrected_sqrt = []
    peak_b_arr = []
    baseline_a_arr = []
    baseline_b_arr = []
    time_a_arr = []
    time_b_arr = []
    rb_num_arr = []
    paddle_num_arr = []
    time_stamp_arr = []
    #for the complement just set to False
    everyother = False
    print(f"everyother: {everyother}")
    for idx, f in enumerate(files, start=1):
        print(f"\n[{idx}/{len(files)}] opening: {f}", flush=True)

        reader = gon.io.TofPacketReader(
            str(f),
            filter=gon.packets.TofPacketType.TofEvent,
        )
        
        for pack in reader:
            ev = gon.events.TofEvent.from_bytestream(pack.payload, 0)
            packcnt_ev += 1
            # skip every other event
            if everyother == True:
                everyother = False
                continue
            else:
                everyother = True
            

            for rb in ev.rb_events:
                rb_event_count += 1
                if rb_has_mangling(rb, mangling_stati):
                    mangling_count += 1
                    continue

                ch9_adc = np.asarray(rb.get_waveform(9), dtype=float)
                if ch9_adc.size < ch9_rms_nbins:
                    continue

                ch9_rms = np.std(ch9_adc[:ch9_rms_nbins])
                ch9_rms_full = np.std(ch9_adc)

                for hit in rb.hits:
                    sigma_a = corrected_rms(hit.baseline_a, hit.baseline_a_rms, num=num)
                    sigma_b = corrected_rms(hit.baseline_b, hit.baseline_b_rms, num=num)
                    if not np.isfinite(sigma_a):
                        continue

                    paddle_num_arr.append(hit.paddle_id)
                    rb_num_arr.append(paddle_to_rb[hit.paddle_id])
                    baseline_b_rms_vals_corrected_sqrt.append(sigma_b)
                    peak_b_arr.append(hit.peak_b)
                    baseline_a_rms_vals_corrected_sqrt.append(sigma_a)
                    peak_a_arr.append(hit.peak_a)
                    ch9_rmsfull_arr.append(ch9_rms_full)
                    all_tot_data_arr_a.append(hit.TOT_high_a)
                    all_tot_data_arr_b.append(hit.TOT_high_b)
                    baseline_a_arr.append(hit.baseline_a)
                    baseline_b_arr.append(hit.baseline_b)
                    time_a_arr.append(hit.time_a)
                    time_b_arr.append(hit.time_b)
                    time_stamp_arr.append(ev.timestamp48)
        if sleep_time > 0:
            time.sleep(sleep_time)

    print(f"\ncollected {len(peak_a_arr)} hits")
    if rb_event_count:
        print(f"{100 * mangling_count / rb_event_count}% of RBs have mangling")
    else:
        print("0 RB events found")
    print(f"mangling count: {mangling_count}")
    print(f"rb event count: {rb_event_count}")
    print(f"packet event count: {packcnt_ev}")

    

    return pl.DataFrame(
        {
            "paddle_id": paddle_num_arr,
            "rb_id": rb_num_arr,
            "baseline_a_rms_corr": baseline_a_rms_vals_corrected_sqrt,
            "peak_a": peak_a_arr,
            "baseline_b_rms_corr": baseline_b_rms_vals_corrected_sqrt,
            "peak_b": peak_b_arr,
            "ch9_rmsfull": ch9_rmsfull_arr,
            "TOT_high_a": all_tot_data_arr_a,
            "TOT_high_b": all_tot_data_arr_b,
            "baseline_a": baseline_a_arr,
            "baseline_b": baseline_b_arr,
            "time_a": time_a_arr,
            "time_b": time_b_arr,
            "time_stamp": time_stamp_arr,
        }
    )


def main() -> None:
    args = parse_args()
    script_dir = Path(__file__).resolve().parent

    import gondola as gon

    file_list_path = resolve_file_list(args.file_list, script_dir)
    files = read_files_txt(file_list_path)
    print(f"using file list: {file_list_path}")
    print(f"found {len(files)} input files")

    data_label, _calib = load_calibration(gon, file_list_path.stem)
    print(f"data label: {data_label}")

    df_wf_cut = process_files(
        gon=gon,
        files=files,
        num=args.num,
        ch9_rms_nbins=args.ch9_rms_nbins,
        sleep_time=args.sleep_time,
    )

    outdir = Path(args.outdir)
    outdir.mkdir(parents=True, exist_ok=True)
    output_path = outdir / args.output
    df_wf_cut.write_parquet(output_path)
    print(f"wrote {output_path}")


if __name__ == "__main__":
    main()
