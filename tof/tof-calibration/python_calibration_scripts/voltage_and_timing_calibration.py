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
#import argparse

from pathlib import Path
from glob import glob

import re
import sys

import tofpy
print (tofpy)

## Parse arguments
#parser = argparse.ArgumentParser(description='Voltage and timing calibrations')
#parser.add_argument('-d', default=0, type=float, help='voltage difference (mV)')
#parser.add_argument('-v', default='', type=str, help='vcal file')
#parser.add_argument('-e', default='average', type=str, help='edge arg for tcal')
#parser.add_argument('-n', default='', type=str, help='name file something')
#parser.add_argument('-npy', action='store_true', help='save as npy')
#parser.add_argument('files', nargs='+', help='file(s) to use for calibration')
#args = parser.parse_args()

# the number comes from Sydney
VOLTAGE_DIFFERENCE=182.0
dv = VOLTAGE_DIFFERENCE
savenpy = False
edge = 'average'

# available rbs
rbs = ['01','02','03','04','07','08','11','12','15','17','19','20','22','24','26','27']
rbs = ['02','03','04','07','08','11','12','15','17','19','20','22','24','26','27']
#rbs = ['11']

def is_not_empty(f):
    return f.stat().st_size != 0


# calibration file path
calpath = Path('/data0/gaps/nts/calibrations/44/')
#cal_output_path = Path(f'/tpool/tofdata/nts/calibrations/{sys.argv[1]}/txt-files/')
cal_output_path = Path('calibrations/txt-files')
vcal_files = sorted(glob(str(calpath / "*.vcal")))
tcal_files = sorted(glob(str(calpath / "*.tcal")))
noi_files  = sorted(glob(str(calpath / "*.noi")))

vcal_files = [Path(k) for k in vcal_files] 
tcal_files = [Path(k) for k in tcal_files]
noi_files  = [Path(k) for k in noi_files]

vcal_files = filter(is_not_empty, vcal_files)
tcal_files = filter(is_not_empty, tcal_files)
noi_files  = filter(is_not_empty, noi_files)


pattern = re.compile('tof-rb(?P<id>[0-9]*)')
vcal_files = filter(lambda x : x[0] is not None, [(pattern.search(str(k)).groupdict(), k) for k in vcal_files])
tcal_files = filter(lambda x : x[0] is not None, [(pattern.search(str(k)).groupdict(), k) for k in tcal_files])
noi_files  = filter(lambda x : x[0] is not None, [(pattern.search(str(k)).groupdict(), k) for k in noi_files])

vcal_files = {k[0]['id'] : k[1] for k in vcal_files}
tcal_files = {k[0]['id'] : k[1] for k in tcal_files}
noi_files  = {k[0]['id'] : k[1] for k in noi_files}

class CalibFiles:
    
    def __init__(self, bid, vcal, tcal, noi):
        self.id   = bid
        self.vcal = vcal
        self.tcal = tcal
        self.noi  = noi

    def __repr__(self):
        return f"<CalibFiles for RB {self.id}: \n {self.vcal} \n {self.tcal} \n {self.noi}>\n"

    def is_complete(self):
        return self.id != "" and self.vcal != "" and self.tcal != "" and self.noi !=""

calib_files = []
for k in rbs:
    if k in vcal_files:
        vcal = vcal_files[k]
    else:
        vcal = ""
    if k in tcal_files:
        tcal = tcal_files[k]
    else:
        tcal = ""
    if k in noi_files:
        noi = noi_files[k]
    else:
        noi = ""
    f = CalibFiles(k, vcal, tcal, noi)
    calib_files.append(f)

print (calib_files)
for k in calib_files:
    if not k.is_complete():
        print (f"Can not run calibration for board {k.id}, files missing or they have 0 size!")
    else:
        if k.id != "27":
            continue
        else:
            print (f'not doing for {k.id}')
        print (f"Attempting calibration for board {k.id}")
        gbf  = tofpy.load(k.noi)
        gbf2 = tofpy.load(k.vcal)
        gbft = tofpy.load(k.tcal)
        name = str(cal_output_path / f'rb{k.id}_cal')
        try:
            tofpy.calibrateBoard(gbf,gbf2,gbft,dv,"",edge,name,savenpy)
        except Exception as e:
            print (f"Can not calibrate board {k.id}, error {e}")
#
##fns = args.files
##nfiles = len(fns)
##vcalfile = args.v
##edge = args.e
##name = args.n
##savenpy = args.npy
#
## load files
#gbf = tofpy.load(fns[0])
#gbf2 = None
#gbft = None
#if nfiles > 1:
#	gbf2 = tofpy.load(fns[1])
#if nfiles > 2:
#	gbft = tofpy.load(fns[2])
#	
## calibrate
#tofpy.calibrateBoard(gbf,gbf2,gbft,dv,vcalfile,edge,name,savenpy)
#
