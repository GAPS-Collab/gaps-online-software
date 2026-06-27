#! /usr/env python

"""
Check if the sine fitter is good enough to live with 
an adjusted dynamic range of -50 to 950.

Just kept here as an example for the interested to 
be modified and extended.
"""

import gondola as gon
import tqdm as tqdm 
from pathlib import Path

# FIXME - adjust paths
# load some RB calibrations
calib  = gon.calibration.load_rb_calibrations(Path('/data1/gaps/mcmurdo/tof/calib/241212_125129UTC/'))
# read caraspace files 
reader = gon.io.CRReader('/prestaging/waveforms/9125')

ch9s         = [] 
fits_nominal = []
# we want to check what happens if we use only 
# the positiv side of the ch9 sine wave 
fits_only_pos = []
#diffs         = []

nevents = 7657222
nevents = 1e6
n = 0
for frame in tqdm.tqdm(reader, total=nevents):
    if not frame.has('PacketType.TofEvent'):
        continue
    n += 1
    if n >= nevents:
        break 
    ev = frame.get_tofevent('PacketType.TofEvent')
    for rbev in ev.rb_events:
        rbid = rbev.header.rb_id
        ch9  = rbev.get_waveform(9)
        if len(ch9) == 0:
            continue
        ch9  = calib[rbid].voltages(9,rbev.header.stop_cell, ch9)
        tim  = calib[rbid].nanoseconds(9,rbev.header.stop_cell)
        fit_nom = gon.algo.fit_sine_simple(tim, ch9) 
        ch9[ch9 < 0] = 0
        fit_new = gon.algo.fit_sine_simple(tim, ch9)
        fits_nominal.append(fit_nom)
        fits_only_pos.append(fit_new) 
        #ch9s.append((rbid,ch9))
