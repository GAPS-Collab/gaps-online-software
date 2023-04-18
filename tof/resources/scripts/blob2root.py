#! /usr/bin/env python

"""
Convert raw RB event data ("blob") into root files.
"""

import pathlib as pl
import sys
import uproot as up
import tqdm
from collections import defaultdict

from copy import deepcopy as copy
from rich import console

import gaps_tof as gt


def get_rb_id_from_file(infile):
    rb = infile.split('_')
    rb = int(rb[0][2:])
    return rb

if __name__ == '__main__':


    import argparse
    parser = argparse.ArgumentParser(description='Convert raw readoutboard binary file ("blob") files to root files')
    parser.add_argument('input', metavar='input',\
                        type=str,\
                        help='Input .blob files')
    parser.add_argument('--output-dir',\
                        metavar='output_dir',\
                        default="",\
                        type=str, help='Directory to store the resulting root files')
    parser.add_argument('--write-root-file',\
                        default=False,\
                        action='store_true',\
                        help='Write a root file with adc [+ calibrated waveforms if available] data')
    parser.add_argument('--plot-pedestals',\
                        default=False,\
                        action='store_true',\
                        help='Plot pedestal distributions for the 9 channels')
    
    parser.add_argument('--plot-average-waveforms',\
                        default=False,\
                        action='store_true',\
                        help='Plot average waveforms')

    parser.add_argument('--calibration-file',\
                        default="",\
                        type=str, help='Calibration file for this specific RB')
    args = parser.parse_args()
    console = console.Console()

    infile = pl.Path(args.input).resolve()
   
    if args.plot_pedestals and not args.calibration_file:
        console.print('Can not calculate pedestals without calibration file!', style='red')
        sys.exit(1)

    console.print('=> Input file:')
    if infile.exists():
        output = f'-- {infile} found!'
        console.print(output, style="green")
    else:
        output = f'-- {infile} DOES NOT EXIST!'
        console.print(output, style="red bold")
        sys.exit(1)
    
    cal = False
    if args.calibration_file : 
        cal = pl.Path(args.calibration_file)
        if not cal.exists():
            raise ValueError(f'{cal} does not exist!')

        console.print(f'-- Will use {cal} for calibration')
        cal = gt.read_calibration_file(str(cal))
        console.print(f'We got calibration constants for {len(cal)} channels!')
    rb  = get_rb_id_from_file(infile.name)
    console.print(f'==> RB ID: {rb}')
    console.print(f'==> Reading file...')
     
    data = gt.splice_readoutboard_datafile(str(infile))
    console.print(f'==> Applying callibrations...')
    nevents = len(data['adc_ch1'])

    #for ch in range(1,10):
    #    print (f'Ch {ch}')
    #vcal = []
    #tcal = []
    #vcal = gt.apply_vcal(data['stop_cell'], cal[ch-1],\
    #                     data[f'adc_ch{ch}'])
    #tcal = gt.apply_tcal(data['stop_cell'], cal[ch-1])
    for ch in range(1,10):
        data[f'v_ch{ch}'] = []
        data[f't_ch{ch}'] = []
    for k in range(nevents):
        ch_data = [data['adc_ch1'][k],\
                   data['adc_ch2'][k],\
                   data['adc_ch3'][k],\
                   data['adc_ch4'][k],\
                   data['adc_ch5'][k],\
                   data['adc_ch6'][k],\
                   data['adc_ch7'][k],\
                   data['adc_ch8'][k],\
                   data['adc_ch9'][k]]
        
        vcal = gt.apply_vcal_allchan(data['stop_cell'][k], cal, ch_data)
        tcal = gt.apply_tcal_allchan(data['stop_cell'][k], cal)
        unspiked = gt.remove_spikes(data['stop_cell'][k], 
                                    vcal)
        
        for ch in range(1,10):
            data[f'v_ch{ch}'].append(unspiked[ch -1])
            data[f't_ch{ch}'].append(tcal[ch -1])

        #    tcal.append(gt.apply_tcal(data['stop_cell'][k], cal[ch-1]))
        #data.update({f'v_ch{ch}' : copy(vcal)})
        #data.update({f't_ch{ch}' : copy(tcal)})
    #foo = gt.apply_vcal(1, cal[0], data['adc_ch1'])
    #del vcal
    #del tcal
    for k in data:
        print (k, len(data[k]))

    console.print(f'==> Extracted the following fields:')
    for k in data.keys():
        console.print(f'-- -- {k} : {len(data[k])} events')

    if args.plot_average_waveforms:
        import pylab as p
        import hepbasestack.layout as lo
        import numpy as np

        averages = defaultdict(lambda : np.zeros(1024))
        for ch in range(1,10):
            for ev in range(nevents):
                averages[ch] += np.array(data[f'v_ch{ch}'][ev])
        
        fig,axs = p.subplots(3,3, figsize=(lo.FIGSIZE_A4_SQUARE[0]*3, lo.FIGSIZE_A4_SQUARE[1]*3))
        for i, ax in enumerate(axs.flat):
            p.sca(ax) 
            ax.plot(averages[i])
        p.savefig('average-waveforms.png')


    # get pedestals
    if args.plot_pedestals:

        import pylab as p
        import hepbasestack.layout as lo
        import dashi as d
        import numpy as np

        d.visual()
        pedestal_bins = np.linspace(-10,10,int(nevents/100))

        console.print(f'=> Calculating waveform pedestals!')
        pedestals = defaultdict(list)
        for ch in range(1,10):
            for ev in range(nevents):
                ped = gt.calculate_pedestal(data[f'v_ch{ch}'][ev],\
                                            data[f't_ch{ch}'][ev],\
                                            ch)
                pedestals[f'ch{ch}'].append(ped)

        fig,axs = p.subplots(3,3, figsize=(lo.FIGSIZE_A4_SQUARE[0]*3, lo.FIGSIZE_A4_SQUARE[1]*3))
        for i, ax in enumerate(axs.flat):
            print (ax)
            print (f'pedestal ch {i}')       
            print (pedestals[f'ch{i+1}'][:100])
            p.sca(ax)
            h = d.factory.hist1d(pedestals[f'ch{i+1}'],\
                                 pedestal_bins)
            h.line()
        fig.savefig('pedestals.png')

    if args.write_root_file:
        output_path = pl.Path(args.output_dir) / infile.name.replace(".blob", ".root")
        console.print(f'==> Creating ROOT file...')

        # write root file
        f = up.recreate(output_path)
        f['rec'] = data
        f.close()
