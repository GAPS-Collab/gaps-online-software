#! /usr/bin/env python

"""
Read raw tof data stream with the rust/python API

* Read waveforms and do waveform calibration
* Sensor data from RBs
* Plot simple distributions
"""

import os
import os.path
import re
import shutil
from pathlib import Path
from datetime import datetime

import charmingbeauty as cb
import charmingbeauty.layout as lo

cb.visual.set_style_present()
import dashi as d
import numpy as np
import matplotlib.pyplot as plt
import tqdm

import gaps_online as go

d.visual()

def baseline(wf):
    return wf[10:50].mean()

def write_main_article(plotdir,
                       nrecohit_plot,
                       occu_plot,
                       runconfig,
                       bad_pids):
    contentdir, rundir = os.path.split(plotdir)
    contentdir = os.path.split(contentdir)[0]
    rundir = int(rundir)
    dt     = datetime.utcnow()
    dt     = dt.strftime("%Y/%m/%d %H:%M:%S")
    rc     = open(runconfig)
    rc     = rc.read()
    text = f"""Title: Umbrella Checkout
Date: {dt}UTC
Category: Umbrella Checkout
![av wf A]({{static}}/images/{rundir}/{nrecohit_plot})
![av wf B]({{static}}/images/{rundir}/{occu_plot})
"""
    if len(bad_pids) > 0:
        text += f"""\n\n
Bad paddles (mean reco hit pk height == 0):
{bad_pids}
"""
    else:
        text += f"""
No bad channels found (all channels have mean reco pk height != 0
"""
    text += """
Runconfig\n\n

{rc}
"""
    with open(Path(contentdir) / f'main.md', 'w') as f:
        f.write(text)


def write_paddle_article(plotdir,
                         paddle_id,
                         wf_picA,
                         wf_picB,
                         bl_picA,
                         bl_picB,
                         pk_picA,
                         pk_picB,
                         charge_pic):
    """
    Write an article for pelican
    """
    contentdir, rundir = os.path.split(plotdir)
    contentdir = os.path.split(contentdir)[0]
    rundir = int(rundir)
    dt     = datetime.utcnow()
    dt     = dt.strftime("%Y/%m/%d %H:%M:%S")

    text = f"""Title: Paddle {paddle_id}
Date: {dt} UTC
Category: Paddle analysis
![av wf A]({{static}}/images/{rundir}/{wf_picA})
![av wf B]({{static}}/images/{rundir}/{wf_picB})
![bl A]({{static}}/images/{rundir}/{bl_picA})
![bl B]({{static}}/images/{rundir}/{bl_picB})
![pk A]({{static}}/images/{rundir}/{pk_picA})
![pk B]({{static}}/images/{rundir}/{pk_picB})
![charge]({{static}}/images/{rundir}/{charge_pic})"""
    with open(Path(contentdir) / f'paddle_{paddle_id}.md', 'w') as f:
        f.write(text)

rbch_pid_dict  = dict()
rbch_pid_label = dict()


# gather all the plots for the pelican article
article_images = dict()
umbrella_pids  = []
# bad paddles (recon pk heihgt == 0)
bad_pids       = []
paddles = go.db.get_umbrella_paddles()
for pdl in tqdm.tqdm(paddles, desc="Creating paddle dicts...", total=len(paddles)):
    uid = pdl.rb_id*100 + pdl.rb_chA 
    rbch_pid_dict[uid]  = pdl.paddle_id
    article_images[pdl.paddle_id] = []
    rbch_pid_label[uid] = f'{pdl.paddle_id}A' 
    uid = pdl.rb_id*100 + pdl.rb_chB 
    rbch_pid_dict[uid] = pdl.paddle_id
    rbch_pid_label[uid] = f'{pdl.paddle_id}B' 
    umbrella_pids.append(pdl.paddle_id)
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


    # read in calibration data
    cali = go.tof.calibrations.load_calibrations_rapi(args.calibration)
    if os.path.isfile(args.rundir):
        runfiles = [args.rundir]
    else:
        runfiles = Path(args.rundir)
        runfiles = [str(r) for r in runfiles.glob('*.tof.gaps')]
    
    # get the run id
    run_id = 'unk'
    run_pattern = re.compile('Run(?P<runid>[0-9]*)_')
    run_id = run_pattern.search(runfiles[0]).groupdict()['runid']
    print (f'=> Will analyze data for run {run_id}')
    args.plotdir = args.plotdir / run_id
    args.plotdir.mkdir(exist_ok=True, parents=True)
    # copy the .toml file
    shutil.copy(Path(args.rundir) / f'run{run_id}.toml', args.plotdir)

    print(f"=> Will run over {len(runfiles)} files!")
    nevents_tot = 0
    nwfs_cali   = 0

    # define plots

    event_status = []

    # charge A/B
    charge_ab = {k : {'a':[], 'b': []} for k in umbrella_pids}
    #charge_ab = dict()

    # average wf
    av_wf    = dict()

    # peak heights
    pk_height = {k : {'a':[], 'b': []} for k in umbrella_pids}

    # baseline histogram
    baselines = dict()

    # paddle occupancy
    ocu_paddles = []

    # nhit distribution
    nhit_distr  = []

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
            for rbev in ev.rbevents:
                event_status.append(rbev.header.status_byte)
            for wf in ev.waveforms:
                wf.calibrate(cali[wf.rb_id])
                nwfs_cali += 1
                if wf.rb_channel == 8:
                    continue
                nwfs_cali += 1
                uid = wf.rb_id*100 + wf.rb_channel + 1
                pid = rbch_pid_dict[uid]
                ocu_paddles.append(pid)
                if not uid in av_wf:
                    av_wf[uid] = [1,wf.voltages]
                else:
                    av_wf[uid][0] += 1
                    av_wf[uid][1] += wf.voltages
                if not uid in baselines:
                    baselines [uid] = (pid,[baseline(wf.voltages)])
                else:
                    baselines[uid][1].append(baseline(wf.voltages)) 
            nhit_distr.append(len(ev.hits))
            for h in ev.hits:
                charge_ab[h.paddle_id]['a'].append(h.charge_a)
                charge_ab[h.paddle_id]['b'].append(h.charge_b)
                pk_height[h.paddle_id]['a'].append(h.peak_a)
                pk_height[h.paddle_id]['b'].append(h.peak_b)


            #print (ev)
    print(f'=> Walked over {nevents_tot} events!')
    print(f'=> Calibrated {nwfs_cali} waveforms!')
    
    # plots
    #fig  = plt.figure(figsize = lo.FIGSIZE_A4_LANDSCAPE)
    #ax         = fig.gca()
    #bins       = 20
    ##oc_bins = np.linspace(xlim_left -0.5,xlim_right-0.5, ocu_nbins)
    #h    = d.factory.hist1d(event_status, oc_b)
    #h.line(filled=True, alpha=0.5, color='b')
    #ax.set_xlabel('paddleID')
    #ax.set_ylabel('occupancy (counts)')
    #fig.savefig(f'{args.plotdir}/paddle_occupancy.png')
    fig  = plt.figure(figsize = lo.FIGSIZE_A4_LANDSCAPE)
    ax   = fig.gca()
    xlim_left  = min(ocu_paddles)
    xlim_right = max(ocu_paddles)
    ocu_nbins  = len(set(ocu_paddles))
    oc_bins = np.arange(xlim_left,xlim_right, 1) - 0.5
    #oc_bins = np.linspace(xlim_left -0.5,xlim_right-0.5, ocu_nbins)
    h    = d.factory.hist1d(ocu_paddles, oc_bins)
    h.line(filled=True, alpha=0.5, color='b')
    ax.set_xlabel('paddleID')
    ax.set_ylabel('occupancy (counts)')
    figname_occ = f'paddle_occupancy.webp'
    fig.savefig(f'{args.plotdir}/{figname_occ}')

    fig  = plt.figure(figsize = lo.FIGSIZE_A4_LANDSCAPE)
    ax   = fig.gca()
    xlim_left  = min(nhit_distr)
    xlim_right = max(nhit_distr)
    bins = np.arange(xlim_left,xlim_right, 1) -0.5
    h    = d.factory.hist1d(nhit_distr, bins)
    h.line(filled=True, alpha=0.5, color='b')
    ax.set_xlabel('nhits/event')
    ax.set_ylabel('counts')
    ax.set_title('Reconstructed hits', loc='right')
    figname_nhits = 'nreco_hits.png'
    fig.savefig(f'{args.plotdir}/{figname_nhits}')

    runconfig = args.plotdir / f'run{run_id}.toml'

    # individual paddles
    for k in tqdm.tqdm(charge_ab.keys(), total=len(charge_ab.keys()), desc="Plotting charge correlations..."):
        fig_cab = plt.figure(figsize=lo.FIGSIZE_A4_SQUARE)
        ax = fig_cab.gca()
        ax.scatter(charge_ab[k]['a'],charge_ab[k]['b'])
        ax.set_xlabel('Charge A Side [mC]')
        ax.set_ylabel('Charge B Side [mC]')
        ax.set_title(f'Charge correlation Paddle {k}', loc='right')
        figname_cab = f'charge_ab_pid{k:02}.png'
        fig_cab.savefig(f'{args.plotdir}/{figname_cab}')
        article_images[k].append(figname_cab)
        plt.close()
        del ax
        del fig_cab
        
        #basebins = np.linspace(-5,5,70)
        bins = 70
        fig  = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE)
        ax   = fig.gca()
        h    = d.factory.hist1d(pk_height[k]['a'], bins)
        h.line(filled=True, alpha=0.7, color='b')
        label = f'{k}A' 
        if np.array(pk_height[k]['a']).mean() == 0.0:
            bad_pids.append(label)
        ax.set_title(f'Paddle {label}', loc='right')
        ax.set_ylabel('counts')
        ax.set_xlabel('reco pk height (mV)', loc='right')
        h.statbox()
        figname = f'pk_height{label}.png'
        fig.savefig(f'{args.plotdir}/{figname}')
        article_images[int(label[:-1])].append(figname)

        del fig
        del ax
        
        fig  = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE)
        ax   = fig.gca()
        h    = d.factory.hist1d(pk_height[k]['b'], bins)
        h.line(filled=True, alpha=0.7, color='b')
        label = f'{k}A' 
        if np.array(pk_height[k]['b']).mean() == 0.0:
            bad_pids.append(label)
        ax.set_title(f'Paddle {label}', loc='right')
        ax.set_ylabel('counts')
        ax.set_xlabel('reco pk height (mV)', loc='right')
        h.statbox()
        figname = f'pk_height{label}.png'
        fig.savefig(f'{args.plotdir}/{figname}')
        article_images[int(label[:-1])].append(figname)

        del fig
        del ax
        

    for k in tqdm.tqdm(av_wf.keys(), total=len(av_wf.keys()), desc='Plotting avg waveforms...'):
        fig  = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE)
        ax   = fig.gca()
        ax.plot(av_wf[k][1][5:]/av_wf[k][0], lw=1.2, color='b')
        ax.set_xlabel('bin')
        ax.set_ylabel('mV')
        label = rbch_pid_label[k] 
        ax.set_title(f'Paddle {label}, av response, first 5 bins removed', loc='left')
        figname_avwf = f'av_wf{label}.png'
        fig.savefig(f'{args.plotdir}/{figname_avwf}')
        article_images[int(label[:-1])].append(figname_avwf)
        del ax
        del fig

    for k in tqdm.tqdm(baselines.keys(), total=len(baselines.keys()), desc='Plotting baselines...'):
        basebins = np.linspace(-3.5,3.5,70)
        fig  = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE)
        ax   = fig.gca()
        h    = d.factory.hist1d(baselines[k][1], basebins)
        h.line(filled=True, alpha=0.7, color='b')
        label = rbch_pid_label[k] 
        ax.set_title(f'Paddle {label}')
        ax.set_ylabel('counts')
        ax.set_xlabel('baseline (mV)')
        h.statbox()
        figname_bl = f'basline{label}.png'
        fig.savefig(f'{args.plotdir}/{figname_bl}')
        article_images[int(label[:-1])].append(figname_bl)
        del fig
        

    # generation of content for pelican
    write_main_article(args.plotdir, 
                       figname_nhits,
                       figname_occ,
                       runconfig,
                       bad_pids)
    for k in article_images:
        art_args = [args.plotdir,k]
        for j in article_images[k]:
            art_args.append(j)
        #print (art_args)
        write_paddle_article(*art_args)
