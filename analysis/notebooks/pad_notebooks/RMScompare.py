#!/usr/bin/env python3

import os
os.environ["DJANGO_ALLOW_ASYNC_UNSAFE"] = "1"

from pathlib import Path

import gondola as gon
import numpy as np
import matplotlib.pyplot as plt

import dashi as d
import charmingbeauty as cb

from matplotlib import rcParams


def setup_plot_style():
    d.visual()
    cb.visual.set_style_present()

    plt.rcParams.update({"text.usetex": False})
    rcParams["font.family"] = "sans-serif"
    rcParams["font.sans-serif"] = ["Open Sans"]


def get_files():
    files = gon.io.grace_get_telemetry_binaries(
        1766035400,
        1766235400,
        "/home/gaps/tof-data/antarctica/nextcloud/flight_2025-26",
    )
    return files


def compute_sigreal(mu, sig_b, num):
    sigreal_sq = ((num - mu**2) / num) * sig_b**2 - mu**2

    if np.isfinite(sigreal_sq) and sigreal_sq >= 0:
        return np.sqrt(sigreal_sq)

    return np.nan


def collect_rms_values(files, num=200):
    sigB1_list = []
    sigB2_list = []
    sigReal1_list = []
    sigReal2_list = []

    for f in files:
        print(f"Reading {f}")

        reader = gon.io.TelemetryPacketReader(f)

        for pack in reader:
            is_event = (
                pack.packet_type == gon.packets.TelemetryPacketType.BoringEvent
                or pack.packet_type == gon.packets.TelemetryPacketType.InterestingEvent
            )

            if not is_event:
                continue

            ev = gon.events.TelemetryEvent.from_telemetrypacket(pack)

            for hit in ev.tof.hits:
                mu1 = hit.baseline_a
                sigB1 = hit.baseline_a_rms

                mu2 = hit.baseline_b
                sigB2 = hit.baseline_b_rms

                sigReal1 = compute_sigreal(mu1, sigB1, num)
                sigReal2 = compute_sigreal(mu2, sigB2, num)

                sigB1_list.append(sigB1)
                sigB2_list.append(sigB2)
                sigReal1_list.append(sigReal1)
                sigReal2_list.append(sigReal2)

    return (
        np.asarray(sigB1_list),
        np.asarray(sigB2_list),
        np.asarray(sigReal1_list),
        np.asarray(sigReal2_list),
    )


def save_hist_comparison(
    baseline_vals,
    sigreal_vals,
    side_label,
    outdir,
    filename,
):
    good_baseline = baseline_vals[np.isfinite(baseline_vals)]
    good_sigreal = sigreal_vals[np.isfinite(sigreal_vals)]

    plt.figure(figsize=(8, 5))
    plt.hist(
        good_baseline,
        bins=80,
        histtype="step",
        label=f"hit.baseline_{side_label.lower()}_rms",
    )
    plt.hist(
        good_sigreal,
        bins=80,
        histtype="step",
        label=rf"$\sigma_\mathrm{{real}}$ side {side_label}",
    )
    plt.xlabel("Sigma / RMS")
    plt.ylabel("Counts")
    plt.title(f"Side {side_label} baseline RMS comparison")
    plt.legend()
    plt.xlim(0, 7)
    plt.yscale("log")
    plt.tight_layout()

    save_path = outdir / filename
    plt.savefig(save_path, dpi=200)
    plt.close()

    print(f"Saved {save_path}")


def save_difference_hist(
    baseline_a,
    baseline_b,
    sigreal_a,
    sigreal_b,
    outdir,
):
    diff_a = sigreal_a - baseline_a
    diff_b = sigreal_b - baseline_b

    diff_a = diff_a[np.isfinite(diff_a)]
    diff_b = diff_b[np.isfinite(diff_b)]

    plt.figure(figsize=(8, 5))
    plt.hist(diff_a, bins=80, histtype="step", label="A")
    plt.hist(diff_b, bins=80, histtype="step", label="B")
    plt.xlabel(r"$\sigma_\mathrm{real} - \sigma_\mathrm{baseline\ RMS}$")
    plt.ylabel("Counts")
    plt.title("Derived sigma minus stored baseline RMS")
    plt.legend()
    plt.yscale("log")
    plt.xlim(-5, 1)
    plt.tight_layout()

    save_path = outdir / "sigreal_minus_baseline_rms.png"
    plt.savefig(save_path, dpi=200)
    plt.close()

    print(f"Saved {save_path}")


def main():
    setup_plot_style()

    outdir = Path("RMScompare")
    outdir.mkdir(parents=True, exist_ok=True)

    Num = 200

    files = get_files()

    baseline_a_rms_vals, baseline_b_rms_vals, sigreal_a_vals, sigreal_b_vals = (
        collect_rms_values(files, num=Num)
    )

    print("N side A:", len(sigreal_a_vals))
    print("N side B:", len(sigreal_b_vals))
    print("N finite side A:", np.sum(np.isfinite(sigreal_a_vals)))
    print("N finite side B:", np.sum(np.isfinite(sigreal_b_vals)))

    save_hist_comparison(
        baseline_a_rms_vals,
        sigreal_a_vals,
        side_label="A",
        outdir=outdir,
        filename="side_A_baseline_rms_comparison.png",
    )

    save_hist_comparison(
        baseline_b_rms_vals,
        sigreal_b_vals,
        side_label="B",
        outdir=outdir,
        filename="side_B_baseline_rms_comparison.png",
    )

    save_difference_hist(
        baseline_a_rms_vals,
        baseline_b_rms_vals,
        sigreal_a_vals,
        sigreal_b_vals,
        outdir=outdir,
    )


if __name__ == "__main__":
    main()
