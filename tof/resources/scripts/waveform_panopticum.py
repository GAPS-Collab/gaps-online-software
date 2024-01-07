#! /usr/bin/env python

import tqdm
import re
import numpy as np
import pylab as p
import scipy.interpolate as inter


from rich.console import Console
console = Console()

import dashi as d
d.visual()

import hepbasestack as hep
hep.visual.set_style_present()

from pathlib import Path
from glob import glob
from collections import defaultdict

import gaps_tof as gt

########################################################

def downsample(data, axis, binstep, binsize, func=np.nanmean):
    """
    Stolen from https://stackoverflow.com/questions/21921178/binning-a-numpy-array
    """
    data = np.array(data)
    dims = np.array(data.shape)
    argdims = np.arange(data.ndim)
    argdims[0], argdims[axis]= argdims[axis], argdims[0]
    data = data.transpose(argdims)
    data = [func(np.take(data,np.arange(int(i*binstep),int(i*binstep+binsize)),0),0) for i in np.arange(dims[axis]//binstep)]
    data = np.array(data).transpose(argdims)
    return data

########################################################

def square_err(y0 , y1):
    return np.sqrt(((y1 - y0)**2).sum())

########################################################

def assemble_calibration_filename(cal_dir, rb_id):
    cal_dir = Path(cal_dir)
    return cal_dir / f'rb{rb_id}_cal.txt'

########################################################

def extract_rb_id(blobfilename):
    rbid = int(blobfilename.split('.dat')[0][-1])
    return rbid

def spline_fitter(time, waveform, resampled_bins=1024):
    if not resampled_bins == len (time):
        resampled_time = downsample(time,0, resampled_bins, resampled_bins)
        resampled_wf   = downsample(waveform,0, resampled_bins, resampled_bins)

    bsplines = inter.splrep(resampled_time,resampled_wf)
    spline_rep = []
    for i in range(len(resampled_time)):
        vec = np.zeros(len(resampled_time))
        vec[i] = 1
        x_list = list(bsplines)
        vec[i] = x_list[1][i]
        x_list[1] = vec.tolist()
        x_i = inter.splev(resampled_time, x_list)
        spline_rep.append(x_i)
    return bsplines,spline_rep, resampled_time

########################################################

def get_blob_files(indir, sortby='rbid'):
    if isinstance(indir, str):
        indir = Path(indir)
    files = glob( str(indir / '*.dat'))
    pattern = re.compile('d(?P<date>[0-9]*)_(?P<time>[0-9]*)_(?P<rb_id>[0-9]*).dat')
    readout_group = defaultdict(lambda : [])
    for f in files:
        print (f)
        try:
            identifier = (pattern.search(f).groupdict())
        except AttributeError:
            print (f'==> Can not parse {f}! Omitting!')
            continue
        if sortby == 'rbid':
            readout_group[int(identifier['rb_id'])].append(f)
        elif sortby == 'datetime':
            readout_group[identifier['date'] + '_' + identifier['time']].append(f)
        else:
            raise ValueError('Can not understand sortby keyword argument. Only "rbid" or "datetiem" are supported values!')
    print (f'==> We found {len(files)} files!')
    return readout_group

########################################################

if __name__ == '__main__':

    import argparse

    parser = argparse.ArgumentParser(description='Get waveforms from blob files')
    parser.add_argument('blobfile',
                        default="",
                        type=str,
                        help="Blobfile to extract waveforms from")
    parser.add_argument('--verbose', 
                        default=False,
                        action='store_true',
                        help="Increase output verbosity")
    parser.add_argument('--calibrate', 
                        default=False,
                        action='store_true',
                        help="Calibrate waveforms")
    parser.add_argument('--threshold', dest='threshold',
                        default=5.0, # default is 5mV
                        type=float,
                        help="Threshold for waveform in mV. Will only consider waveforms which cross this threshold")
    parser.add_argument('--event-id', dest='eventid',
                        default=-1, # default is 5mV
                        type=int,
                        help="Show all waveforms for a specific eventid")
    parser.add_argument('--average',
                        default=False, # default is 5mV
                        action='store_true',
                        help="average the waveforms per channel")
    parser.add_argument('--calibration-file-directory', dest='calibration_file_dir',
                        default='datafiles',
                        type=str,
                        help='Calibration file for a specific readout board')


    args = parser.parse_args()
    print (args)

    # if the calibration is desired, we need
    # to read in all the calibration files
    if args.calibrate:
    
        rb_id = extract_rb_id(args.blobfile)
        calibration_file = assemble_calibration_filename(args.calibration_file_dir, rb_id)
        calibrations = gt.read_calibration_file(str(calibration_file))

    # find out how many events are in a certain file
    nevents = gt.get_nevents_from_file(args.blobfile)
    print (f'==> Found {nevents} events in {args.blobfile}')

    # get all the events
    data = open(args.blobfile, 'rb').read()
    data = [k for k in data]
    print (f'==> Read {len(data)} bytes from {args.blobfile}')
    events = gt.get_events_from_stream(data,0)
    if events:
        print (f'==> Extracted {len(events)}. First evid {events[0].event_ctr}, last evid {events[-1].event_ctr}')
    else:
        print (f'==> No events could be extracted from {args.blobfile}!')

    console.rule()

    if args.calibrate:
        if args.average:
            waveforms = dict()
            for k in range(len(calibrations)):
                waveforms[k] = [0,np.zeros(1024)]

        wf_found = False
        errs = defaultdict(lambda : [])

        paddle_packets = []

        # event loop
        for ev in tqdm.tqdm(events, total=len(events)):
            if wf_found:
                break

            waveform_data = ev.get_ch_adc()
            if args.calibrate:

                waveform_data = gt.voltage_calibration(ev, calibrations)
                waveform_data = gt.remove_spikes(waveform_data, ev)
                times = gt.timing_calibration(ev, calibrations)

            waveform_data = np.array(waveform_data)

            for ch, wave in enumerate(waveform_data):#enumerate(waves):
                if ch==8: continue
                
                pedestal = gt.calculate_pedestal(wave, times[ch], ch)
                wave -= pedestal
                
                wave = np.asarray(wave)
                if wave[wave > args.threshold].any():

                    if args.verbose:
                        print (f'==> Calculated pedestal of {pedestal}')
                    if args.average:
                        waveforms[ch][0] += 1
                        waveforms[ch][1] += wave
                    if args.verbose:
                        wf_max = max(waveform_data[ch])
                        print (f'=> channel {ch} for event {ev.event_ctr} went over threshold with value {wf_max}!')
                    if (ev.event_ctr == args.eventid):
                        p.plot(times[ch],waveform_data[ch])
                        wf_found = True
                        break

                # spline fitting and histogram
                bsplines,splines, rs_time = spline_fitter(times[ch],\
                                                          waveform_data[ch],\
                                                          resampled_bins=8)
                fit = inter.splev(times[ch],bsplines)
                err = square_err(fit, waveform_data[ch])    
                errs[ch].append(err)

        # err histogram
        fig_histo = p.figure()
        ax_histo  = fig_histo.gca()
        bins = np.linspace(0,20,100)
        for k in errs:                         
            h = d.factory.hist1d(errs[k], bins)
            h = h.normalized()
            h.line(filled=True, alpha= 0.5)
        #p.show()
        ax_histo.set_xlabel('sqrt error')
        ax_histo.set_ylabel('normalized events')
        ax_histo.set_title('Data for a single readoutboard, all channels', loc='right')
        fig_histo.savefig('sq_err.png')
        # plot the average
        if args.average:
            for ch in waveforms:
                waveforms[ch][1] = waveforms[ch][1]/waveforms[ch][0] # average
            figsize = (6,3)
            fig, axes        = p.subplots(2, 1,
                                          figsize=figsize,
                                          sharex=True,
                                          sharey=False)#, gridspec_kw={'height_ratios': [2, 1]})
            bsplines,splines, rs_time = spline_fitter(times[0],\
                                                      waveforms[0][1],\
                                                      resampled_bins=16)
            for spline in splines:
                axes[0].plot(rs_time, spline)
            fit = inter.splev(times[0],bsplines)
            axes[0].plot(times[0], fit, lw=2, color='r', alpha=0.8)
            axes[0].plot(times[0], waveforms[0][1], lw=1, linestyle='dashed', color='b')
            axes[0].set_ylim(bottom=-5, top=50)            
            left_edge, right_edge = 280, 380
            roi = slice(left_edge, right_edge)
            axes[0].set_xlim(left=left_edge, right=right_edge)
            axes[1].set_xlim(left=left_edge, right=right_edge)
            axes[1].plot(times[0], fit/waveforms[0][1],color='k')
            axes[1].set_ylim(bottom=-10, top=10)
            axes[1].hlines([1],left_edge, right_edge, linestyles='dashed', colors=['k'])
            axes[1].set_xlabel('time [ns]')
            axes[1].set_ylabel('ratio')
            axes[0].set_ylabel('mV')
            axes[0].set_title('av. waveform, spline fit',loc='right')
            err = square_err(waveforms[0][1], fit)
            print (f'==> This has a quad error of {err:4.2f}')
            err = square_err(waveforms[0][1][roi], fit[roi])
            print (f'==> This has a quad error of {err:4.2f} in the selected roi')
            
        #   figsize = (4,2)
        #   fig, axes        = p.subplots(2, 4,
        #                              figsize=figsize,
        #                              sharex=True,
        #                              sharey=True)#, gridspec_kw={'height_ratios': [2, 1]})
        #    #print (waveforms.keys())
        #    for k, ch in enumerate(range(4)):
        #        axes[0][k].plot(times[ch],waveforms[ch][1])
        #        axes[0][k].set_ylabel('mV')
        #        axes[0][k].text(0.8, 0.8, f'Channel {ch}', transform=axes[0][k].transAxes)
        #    for k, ch in enumerate(range(4)):
        #        axes[1][k].plot(times[ch],waveforms[4+ch][1])
        #        axes[1][k].text(0.8, 0.8, f'Channel {4 + ch}', transform=axes[1][k].transAxes)
        #        axes[1][k].set_xlabel('ns')
        fig.savefig('spline_comparison.png')
        p.show()
        #p.savefig('foo.png')

