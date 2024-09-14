#! /usr/bin/env python

import matplotlib
import matplotlib.pyplot as plt
import charmingbeauty.layout as lo
import numpy as np
from collections import defaultdict

matplotlib.use('agg') # for non-interactive use,
                      # if gtk and such are not installed

def mtb_rate_plot(data : list[tuple]):
    """
    Create a plot of MTB rates + MTB lost rate for these quantities
    as extrected from the telemetry stream

    # Arguments:
        data :  A list of tuples (met, MtbMoniData) where met is the
                "mission elapsed time" in seconds, which we can get
                from the TelemetryPacketHeader
    """
    fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT)
    ax = fig.gca()
    ax.set_ylabel('Hz', loc='top')
    ax.set_xlabel('MET [s] (gcu)')
    times   = np.array([j[0] for j in data])
    times  -= times[0]
    times   /= 1e9
    rates   = np.array([j[1].rate for j in data])
    l_rates = np.array([j[1].lost_rate for j in data])
    #print(times[l_rates < 500][-1])
    print(f'-> Avg MTB rate {rates.mean()}')
    print(f'-> Avg Lost rate {l_rates.mean()}')
    ax.plot(times, rates, lw=0.8, alpha=0.7, label='rate')
    ax.plot(times, l_rates, lw=0.8, alpha=0.7, label='lost rate')
    ax.legend(loc='upper right', frameon=False)
    ax.set_title(f'MTB rates', loc='right')
    return fig

def rb_rate_plots(rb_moni_series):
    """
    Create individual rate plots for all RBs
    """
    rbrates = defaultdict(list)
    for k in rb_moni_series:
        rbrates[k[1].board_id].append((k[0], k[1].rate))

    figures = []
    for k in sorted(rbrates):
        fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT)
        ax = fig.gca()
        ax.set_ylabel('Hz', loc='top')
        ax.set_xlabel('MET [s] (gcu)', loc='right')

        times = np.array([j[0] for j in rbrates[k]])
        times -= times[0]
        rates = np.array([j[1] for j in rbrates[k]])
        ax.plot(times, rates, lw=0.8, alpha=0.7)
        dbrb = go.db.ReadoutBoard.objects.filter(rb_id=k)[0]
        panels = []
        for j in dbrb.paddles:
            panels.append(j.panel_id)
        panels = set(panels)
        ax.set_title(f'RB {k} (panels {panels})', loc='right')
        figures.append((k,fig))
    return figures

def good_hits_trk(tracker_event):
    """
    Count tracker hits with asic event codes 0 or 2
    """
    n_good = 0
    rows   = defaultdict(lambda : 0)
    # filter hits takes a function with hit as an argument and needs to return bool
    for k in tracker_event.filter_hits(lambda h : h.asic_event_code == 2 or h.asic_event_code == 0):
        n_good += 1
        rows[k.row] += 1
    return n_good, rows

if __name__ == '__main__':

    import argparse
    import sys
    import tqdm
    import gaps_online as go

    # pretty plots
    import charmingbeauty as cb
    cb.visual.set_style_default()

    from pathlib import Path

    parser = argparse.ArgumentParser(description='Scrutinize run data for issues. Input can be either telemetry binary files, TOF run files (.tof.gaps) or both')
    parser.add_argument('--telemetry-dir', default='',\
                        help='A directory with telemetry binaries, as received from the telemetry stream',
                        )
    parser.add_argument('-n','--npackets', type=int,\
                        default=-1, help='Limit readout to npackets, -1 for all packets (default)')
    parser.add_argument('--tof-dir', default='',
                        help='A directory with tof data files (.tof.gaps)',)
    parser.add_argument('-s','--start-time',\
                        type=int, default=-1,\
                        help='The run start time, e.g. as taken from the elog')
    parser.add_argument('-e','--end-time',
                        type=int, default=-1,\
                        help='The run end time, e.g. as taken from the elog')
    parser.add_argument('-r','--run-id', default=-1,\
                        help='TOF Run id (only relevant when working with TOF files')
    parser.add_argument('-o','--outdir',\
                        help='Outdir to save plots',
                        default='')
    args = parser.parse_args()
    use_telemetry_stream = True
    use_tof_stream       = False
    if (args.start_time == -1 or args.end_time == -1):
        if args.run_id == -1:
            print(f'Please provide start and end times, or a TOF run id!')
            print(f'-- See ./run_vetting.py --help for more information.')
            sys.exit(1)
        else:
            print(f'--> Will use TOF-only data stream for run {args.run_id}')
            use_tof_stream = True
            use_binary_stream = False

    if use_telemetry_stream:
        if not args.telemetry_dir:
            print('Please provide a directory with telemetry files, e.g. /gaps_binaries/live/raw/ethernet on the gse systems!')
            print(f'-- See ./run_vetting.py --help for more information.')
            sys.exit(1)

        files = go.io.get_telemetry_binaries(args.start_time, args.end_time,\
                                             data_dir=args.telemetry_dir)

        npackets = 0
        nmerged  = 0
        # Example readout of merged events, MtbMoniData and RBMoniData
        # from the telemetry stream
        # merged_event = go.events.MergedEvent()


        mtb_moni_series = []
        rb_moni_series  = []
        merged_events   = []
        done = False
        for f in tqdm.tqdm(files, desc='Reading files..'):
            treader = go.io.TelemetryPacketReader(str(f))
            if done: #You're done!
                break
            for pack in treader:
                npackets += 1
                if args.npackets != -1:
                    if args.npackets == npackets:
                        done = True
                        break
                if pack.header.packet_type == go.io.TelemetryPacketType.MergedEvent: # merged events
                    nmerged += 1
                    merged_event = go.io.safe_unpack_merged_event(pack)
                    # admittedly, keeping all events in memory at the same time might be
                    # consuming too much memory for your machine. However, this can be
                    # mitigated by selecting the interesting variables here already and
                    # only keep these in memory or creating filling histograms here already
                    merged_events.append(merged_event)
                if pack.header.packet_type == go.io.TelemetryPacketType.AnyTofHK: # AnyTofHK
                    tp = go.io.TofPacket()
                    tp.from_bytestream(pack.payload, 0)
                    if tp.packet_type == go.io.TofPacketType.MonitorMtb:
                        mtb_moni = go.tof.monitoring.MtbMoniData()
                        mtb_moni.from_tofpacket(tp)
                        mtb_moni_series.append((pack.header.gcutime,mtb_moni))
                    if tp.packet_type == go.io.TofPacketType.RBMoniData:
                        rb_moni = go.tof.monitoring.RBMoniData()
                        rb_moni.from_tofpacket(tp)
                        rb_moni_series.append((pack.header.gcutime, rb_moni))

        print(f'-> Read {npackets} telemetry packets for this run!')
        print(f'-> Found {nmerged} merged event packets for this run!')

        errors = [k for k in tqdm.tqdm(merged_events, desc='Error checking..') if k is None]
        errors = len(errors)
        if errors:
            print (f'-> When unpacking the packets, we encountered {errors} errors. This is {100*errors/len(merged_events):.3f}')
        clean_events = [k for k in tqdm.tqdm(merged_events, desc='Filtering....') if k is not None]

        # plot creating section
        outdir = args.outdir
        if not outdir:
            # create generic output directory
            outdir = 'plots'
        outdir = Path(outdir)
        if not outdir.exists():
            outdir.mkdir(parents=True)

        # create mtb rate plot
        fig = mtb_rate_plot(mtb_moni_series)
        fig.savefig(outdir / 'mtb_rates.webp')

        # individual rb rate plots
        figs = rb_rate_plots(rb_moni_series)
        for rbid, fig in figs:
            fig.savefig(outdir / f'rb{rbid}_rate.webp')

        # check tracker occupancy/row
        layers = defaultdict(lambda: 0)
        layer_rows = {k: defaultdict(lambda: 0) for k in range(10)}
        total_hits = 0
        for ev in tqdm.tqdm(merged_events, desc='Counting tracker hits'):
            for tev in ev.tracker:
                n_good, rows = good_hits_trk(tev)
                total_hits += n_good
                layers[tev.layer - 128] += n_good
                for r in rows:
                    try:
                        layer_rows[tev.layer - 128][r] += rows[r]
                    except Exception as e:
                        continue
        print (f'Good tracker hits (total) {total_hits}')
        print (layers)
        print (layer_rows)

        # TBC




