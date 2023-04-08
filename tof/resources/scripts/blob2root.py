#! /usr/bin/env python

"""
Convert raw RB event data ("blob") into root files.
"""

import pathlib as pl
import sys
import uproot as up

import gaps_tof as gt

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
    parser.add_argument('--calibration-file',\
                        metavar='cal_file',\
                        default="",\
                        type=str, help='Directory with calibration files')
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
    
    rb  = get_rb_id_from_file(infile.name)
    console.print(f'==> RB ID: {rb}')
    console.print(f'==> Reading file...')
    
    data = gt.splice_readoutboard_datafile(str(infile))
    console.print(f'==> Extracted the following fields:')
    for k in data.keys():
        console.print(f'-- -- {k} : {len(data[k])} events')

    console.print(f'==> Creating ROOT file...')
    f = up.recreate('test.root')
    f['rec'] = data
    f.close()
