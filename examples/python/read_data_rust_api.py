#! /usr/bin/env python

"""
Read raw tof data stream with the rust/python API

* Read waveforms and do waveform calibration
* Sensor data from RBs
* Plot simple distributions
"""

from pathlib import Path
import os
import os.path

import charmingbeauty as cb
import charmingbeauty.layout as lo

cb.visual.set_style_present()
import dashi as d
import numpy as np
import matplotlib.pyplot as plt

import gaps_online as go

import tqdm

d.visual()

def baseline(wf):
    return wf[10:50].mean()

rbch_pid_dict = dict()
paddles = go.db.get_tof_paddles()
for pdl in paddles:
    uid = pdl.rb_id*100 + pdl.rb_chA 
    rbch_pid_dict[uid] = pdl.paddle_id
    uid = pdl.rb_id*100 + pdl.rb_chB 
    rbch_pid_dict[uid] = pdl.paddle_id


if __name__ == '__main__':

    import argparse
    
    parser = argparse.ArgumentParser(description='Example to illustrate how to deal with TOF rawdata from .tof.gaps files. Read data, use RBcalibrations to get waveforms, read sensor data, etc...')
    parser.add_argument('rundir', metavar='rundir', type=str,
                        help='A directory with .tof.gaps files or a single .tof.gaps.file')
    parser.add_argument('--plotdir', metavar='plotdir', type=Path, default='output-plot-dir',
                        help='A directory where output plots will be saved.')
    parser.add_argument('--calibration', '-c', dest='calibration',
                        default='', type=Path,
                        help='Path to calibration txt files for all boards.')

    args = parser.parse_args()
    args.plotdir.mkdir(exist_ok=True, parents=True)

    # read in calibration data
    cali = go.tof.calibrations.load_calibrations_rapi(args.calibration)
    if os.path.isfile(args.rundir):
        runfiles = [args.rundir]
    else:
        runfiles = Path(args.rundir)
        runfiles = [str(r) for r in runfiles.glob('*.tof.gaps')]
    
    print(f"=> Will run over {len(runfiles)} files!")
    nevents_tot = 0
    nwfs_cali   = 0

    # define plots

    # charge A/B
    charge_ab = {k : {'a':[], 'b': []} for k in range(1, 161)}

    # baseline histogram
    baselines = dict()

    # paddle occupancy
    ocu_paddles = []

    for f in runfiles:
        event_reader = go.rust_api.io.TofPacketReader(f, filter=go.rust_api.io.PacketType.TofEvent)
        print('-> Creating packet index...')
        pi = event_reader.get_packet_index()
        print ('--- ---')
        for k in pi:
            print (f'  {k}\t : {pi[k]}')

        rbmoni       = go.rust_api.moni.RBMoniSeries()
        # this converts the custom series into a polars data frame!
        rbmoni       = rbmoni.from_file(f)
        print(rbmoni)
        for pack in tqdm.tqdm(event_reader, desc='Reading packets', total = pi[21]):
            ev = go.rust_api.events.TofEvent()
            ev.from_tofpacket(pack)
            nevents_tot += 1
            for wf in ev.waveforms:
                wf.calibrate(cali[wf.rb_id])
                if wf.rb_channel == 8:
                    continue
                nwfs_cali += 1
                uid = wf.rb_id*100 + wf.rb_channel + 1
                pid = rbch_pid_dict[uid]
                ocu_paddles.append(pid)
                if not uid in baselines:
                    baselines [uid] = (pid,[baseline(wf.voltages)])
                else:
                    baselines[uid][1].append(baseline(wf.voltages)) 
            for h in ev.hits:
                charge_ab[h.paddle_id]['a'].append(h.charge_a)
                charge_ab[h.paddle_id]['b'].append(h.charge_b)
            #print (ev)
            if len(ev.waveforms) > 0:
                wf = ev.waveforms[0] # assuming ev is the event from above
                wf.calibrate(cali[wf.rb_id])
                nwfs_cali += 1
    print(f'=> Walked over {nevents_tot} events!')
    print(f'=> Calibrated {nwfs_cali} waveforms!')
    
    # plots
    fig  = plt.figure(figsize = lo.FIGSIZE_A4_LANDSCAPE)
    ax   = fig.gca()
    oc_bins = np.linspace(0.5, 160.5, 160)
    h    = d.factory.hist1d(ocu_paddles, oc_bins)
    h.line(filled=True, alpha=0.5, color='b')
    ax.set_xlabel('paddleID')
    ax.set_ylabel('occupancy (counts)')
    fig.savefig(f'{args.plotdir}/paddle_occupancy.png')

    for k in tqdm.tqdm(range(1,161), total=160, desc="Plotting charge correlations..."):
        fig_cab = plt.figure(figsize=lo.FIGSIZE_A4_SQUARE)
        ax = fig_cab.gca()
        ax.scatter(charge_ab[k]['a'],charge_ab[k]['b'])
        ax.set_xlabel('Charge A Side [mC]')
        ax.set_ylabel('Charge B Side [mC]')
        ax.set_title(f'Charge correlation Paddle {k}', loc='right')
        fig_cab.savefig(f'{args.plotdir}/charge_ab_pid{k:02}.png')
        del ax
        del fig_cab

    for k in tqdm.tqdm(baselines.keys(), total=len(baselines.keys()), desc='Plotting baselines...'):
        basebins = np.linspace(-5,5,70)
        fig  = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE)
        ax   = fig.gca()
        h    = d.factory.hist1d(baselines[k][1], basebins)
        h.line(filled=True, alpha=0.7, color='b')
        ax.set_title(f'Channel {k}, Paddle {baselines[k][0]}')
        ax.set_ylabel('counts')
        ax.set_xlabel('baseline (mV)')
        h.statbox()
        fig.savefig(f'{args.plotdir}/baseline{k}.png')
