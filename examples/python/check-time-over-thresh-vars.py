#! /usr/bin/env python

"""
Example script to test the TOF time over threshold 
variables to combat saturated waveforms.

This is neither complete nor useful, but might help 
to develop something actually useful (?)
"""

from pathlib import Path
from re import I


import time
import gaps_online as go

import tqdm
import numpy as np 
import matplotlib.pyplot as plt
import matplotlib 
import charmingbeauty as cb 
import charmingbeauty.layout as lo
import dashi as d
d.visual()

cb.visual.set_style_present()
matplotlib.use('agg')

calib = go.tof.calibrations.load_calibrations(Path('/data1/gaps/mcmurdo/tof/calib/241212_125129UTC'))

MINVAL = 400
MAXVAL = 500

THRSH  = 450

max_frames = 100000
max_frames = 1e9
def integrate(ns, volts) -> float:
    integral = np.trapezoid(volts, x=ns)
    return integral


tot_a_all = []
ch_a_all  = []

tot_b_all = []
ch_b_all  = []

nframes = 0
reader = go.io.CRReader(Path('/prestaging/waveforms/9125/'))
start = time.time()
for frame in tqdm.tqdm(reader):
    if frame.has('PacketType.TofEvent'):
        nframes += 1
        if nframes >= max_frames:
            break

        ev = frame.get_tofevent('PacketType.TofEvent')
        for wf in ev.waveforms:
            wf.calibrate(calib[wf.rb_id])
            if MINVAL < max(wf.voltages_a) < MAXVAL:
                tot_a = wf.get_tot_a(THRSH)
                pedestal = wf.voltages_a[10:50].mean()
            #    charge_a = integrate(wf.times_a,wf.voltages_a - pedestal)/50
            #    if tot_a > 0:
            #        tot_a_all.append(tot_a)
            #        ch_a_all.append(charge_a)
            #    #print(pedestal, charge_a, tot_a)
            #    #raise
            #if MINVAL < max(wf.voltages_b) < MAXVAL:
            #    tot_b = wf.get_tot_b(THRSH)
            #    pedestal = wf.voltages_b[10:50].mean()
            #    charge_b = integrate(wf.times_b,wf.voltages_b - pedestal)/50
            #    if tot_b > 0:
            #        tot_b_all.append(tot_b)
            #        ch_b_all.append(charge_b)

nbins = 70
max_tot = max(max(tot_a_all), max(tot_b_all))
max_ch  = max(max(ch_a_all), max(ch_b_all))
max_ch = 400
bins = (np.linspace(0,80,nbins), np.linspace(0,max_tot,nbins))
bins = (np.linspace(0,max_ch,nbins), np.linspace(0,max_tot,nbins))
h = d.factory.hist2d((ch_a_all, tot_a_all), bins)

fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE)
ax  = fig.gca()
h.imshow()
ax.set_xlabel("Charge A", loc='right')
ax.set_ylabel("TOT A", loc='top')
ax.spines['top'].set_visible(True)
ax.spines['right'].set_visible(True)
ax.set_title(f'THR = {THRSH} mV', loc='right')
fig.savefig('chA_ch_vs_tot.png')

h = d.factory.hist2d((ch_b_all, tot_b_all), bins)
fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE)
ax  = fig.gca()
h.imshow()
ax.set_xlabel("Charge A", loc='right')
ax.set_ylabel("TOT A", loc='top')
ax.spines['top'].set_visible(True)
ax.spines['right'].set_visible(True)
ax.set_title(f'THR = {THRSH} mV', loc='right')
fig.savefig('chB_ch_vs_tot.png')

print(max(tot_a_all))
print(max(tot_b_all))

print (f'The script took {time.time() - start:.2f}s')
