#!/usr/bin/env python3
# Licensed under a 3-clause BSD style license - see PYFITS.rst

'''
Authors: Jamie Ryan (jryan@astro.ucla.edu)
cal.py (c) UCLA 2021
Desc: Parse memory dump blob and plot traces
Created:  2021-09-20
Modified: 2022-03-17

Do voltage and timing calibrations and save to txt file

Usage: python3 cal.py [-n #] [-d #] [-t] input_file(s)
	input_file: "blob" / DRS4 readout board RAM buffer .dat file
	d #: voltage difference in mV between voltage calibration runs
	v file: use existing voltage calibration file + skip voltage calibration
	e edge: edge argument for timingCalibration - rising, falling, or average
	n name: file name (if blank, defaults to rb#_cal)
	npy: also save in .npy format
'''
import argparse

import tofpy

# Parse arguments
parser = argparse.ArgumentParser(description='Voltage and timing calibrations')
parser.add_argument('-d', default=0, type=float, help='voltage difference (mV)')
parser.add_argument('-v', default='', type=str, help='vcal file')
parser.add_argument('-e', default='average', type=str, help='edge arg for tcal')
parser.add_argument('-n', default='', type=str, help='name file something')
parser.add_argument('-npy', action='store_true', help='save as npy')
parser.add_argument('files', nargs='+', help='file(s) to use for calibration')
args = parser.parse_args()

fns = args.files
nfiles = len(fns)
dv = float(args.d)
vcalfile = args.v
edge = args.e
name = args.n
savenpy = args.npy

# load files
gbf = tofpy.load(fns[0])
gbf2 = None
gbft = None
if nfiles > 1:
	gbf2 = tofpy.load(fns[1])
if nfiles > 2:
	gbft = tofpy.load(fns[2])
	
# calibrate
tofpy.calibrateBoard(gbf,gbf2,gbft,dv,vcalfile,edge,name,savenpy)

