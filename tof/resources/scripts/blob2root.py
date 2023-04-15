#! /usr/bin/env python

"""
Convert raw RB event data ("blob") into root files.
"""

import pathlib as pl
import sys
import uproot as up
import tqdm

import gaps_tof as gt
from copy import deepcopy as copy
from rich import console

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
    parser.add_argument('--calibration-file',\
                        default="",\
                        type=str, help='Calibration file for this specific RB')
    args = parser.parse_args()
    console = console.Console()

    infile = pl.Path(args.input).resolve()
    
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
    test = data['adc_ch1']
    for ch in range(1,10):
        print (f'Ch {ch}')
        vcal = []
        tcal = []
        for k in range(len(data[f'adc_ch{ch}'])):
            vcal.append(gt.apply_vcal(data['stop_cell'][k], cal[ch-1], data[f'adc_ch{ch}'][k]))
            tcal.append(gt.apply_tcal(data['stop_cell'][k], cal[ch-1]))
        data.update({f'v_ch{ch}' : copy(vcal)})
        data.update({f't_ch{ch}' : copy(tcal)})
    #foo = gt.apply_vcal(1, cal[0], data['adc_ch1'])
    for k in data:
        print (k, len(data[k]))

    console.print(f'==> Extracted the following fields:')
    for k in data.keys():
        console.print(f'-- -- {k} : {len(data[k])} events')

    output_path = pl.Path(args.output_dir) / infile.name.replace(".blob", ".root")
    console.print(f'==> Creating ROOT file...')

    # write root file
    f = up.recreate(output_path)
    f['rec'] = data
    f.close()
