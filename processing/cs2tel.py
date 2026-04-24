#! /usr/bin/env python 

"""
Write (orginal form conformative) telemetry 
files from caraspace files. 

Use to create new merged events from 
original merged events + new tracker hits

"""

import os
import shutil
import tqdm
import gondola as go
import time
import re
import matplotlib.pyplot as plt 
import matplotlib 
matplotlib.use('agg')
import numpy as np
import dashi as d 
d.visual()

from pathlib import Path
from glob import glob
from copy import deepcopy
from dataclasses import dataclass
from fancy_dataclass import TOMLDataclass

# try to suppress RUST logging
import logging
logging.getLogger('go').addHandler(logging.NullHandler())

import charmingbeauty as cb 
cb.visual.set_style_default()
cb.visual.set_style_streamlit_dark()

# check gondola version
if not (go.get_version_minor() >= 12 and go.get_version_patch() >= 15):
    print(f'ERROR - got version {go.get_version_major()}.{go.get_version_minor()}.{go.get_version_patch()}')
    raise ImportError("gondola needs to be at least version 0.12.8!")

strip_mask = go.db.TrackerStripMask.parse_from_file('/srv/gaps/crane/v26.03/calibration/resources/trk-2025/tracker_channel_enables_100.txt')
strip_mask = {k.strip_id : k for k in strip_mask}

active_strips = 0
all_strips = go.db.TrackerStrip.all_as_dict()
for strip in all_strips:
    if not strip in strip_mask:
        active_strips += 1 
        continue
    if strip_mask[strip].active:
        active_strips += 1 
active_strip_fraction = active_strips/len(all_strips)

print(f'-> {100*active_strip_fraction:.2f} % of strips active (as indicated by map)')
if __name__ == '__main__':

    import argparse
    import sys

    description = """Pre-process the telemetry data for the usage with SimpleDet"""
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument('--run-dir', default=Path('/data0/gaps/csbf/csbf-data/binaries/ethernet'),\
                        help='A directory with caraspace files for a single run',\
                        type=Path,
                        )
    parser.add_argument('-o','--outdir',\
                        help='Outdir for caraspace output files',
                        type=Path,
                        default=None)
    args = parser.parse_args()
  
    run_files = Path(args.run_dir)
    run_files = sorted([k for k in run_files.glob('*.gaps')]) 
    n_hits_merged = [] 
    n_hits_merged_plus = []
    n_hits_merged_plus_masked = []
    for f in tqdm.tqdm(run_files):
        reader = go.io.CRReader(str(f)) 
        # first packet/frame 
        for frame in reader:
            #print (frame)
            ev = frame.get_telemetryevent('TelemetryEvent')
            extra_trk_hits = frame.get_tracker_hitseries()
            #print(f'-> Event has {len(ev.tracker)} tracker hits')
            #print(f'-> We got {len(extra_trk_hits)} hits from the tracker events')
            extra_tracker_hits = [h for h in extra_trk_hits if not h in ev.tracker] 
            #print(f'-> These are  {len(extra_trk_hits)} extra hits')
            ev.add_tracker_hits(extra_trk_hits)
            #print(f'-> Event has {len(ev.tracker)} tracker hits')
            pack = ev.pack()
            ev = go.events.TelemetryEvent.from_telemetrypacket(pack)
            writer = go.io.TelemetryPacketWriter('deleteme', pack)
            break
        for frame in reader:
            #print (frame)
            ev = frame.get_telemetryevent('TelemetryEvent')
            n_hits_merged.append(len(ev.tracker))
            extra_trk_hits = frame.get_tracker_hitseries()
            #print(f'-> Event has {len(ev.tracker)} tracker hits')
            #print(f'-> We got {len(extra_trk_hits)} hits from the tracker events')
            extra_tracker_hits = [h for h in extra_trk_hits if not h in ev.tracker] 
            ev.add_tracker_hits(extra_trk_hits)
            #print(f'-> Event has {len(ev.tracker)} tracker hits')
            n_hits_merged_plus.append(len(ev.tracker))
            #print ('--------------------')
            n_unmasked = 0
            for h in ev.tracker:
                if strip_mask[h.strip_id].active: 
                    n_unmasked += 1 
            n_hits_merged_plus_masked.append(n_unmasked)
            tp = ev.pack() 
            # debug 
            #ev = go.events.TelemetryEvent.from_telemetrypacket(tp)
            ev = go.events.TelemetryEvent.from_telemetrypacket(tp)
            writer.add_telemetry_packet(tp)
            #for k in frame.index:
            #    if k.startswith('Tracker'):
            #        trk = frame.get_telemetrypacket(k) 
            #        trk_pk = go.events.TrackerDAQEventPacket.from_telemetrypacket(trk) 

        #print (f) 
   

    fig = plt.figure(figsize=cb.layout.FIGSIZE_A4_LANDSCAPE) 
    ax  = fig.gca() 
    #tbins = np.linspace(-250,250,100)
    bins = np.arange(-0.5, 250.5, 1)
    h    = d.factory.hist1d(n_hits_merged, bins) 
    h.line(filled=True, color='w', alpha=0.7, label='merged, telemetry') 
    h2   = d.factory.hist1d(n_hits_merged_plus, bins) 
    h2.line(filled=True, color='r', alpha=0.7, label='remerged with pk80')
    h3   = d.factory.hist1d(n_hits_merged_plus_masked, bins) 
    h3.line(filled=False, color='c', alpha=0.7, label=f'remerged, masked (active fraction {100*active_strip_fraction:.1f} \%)')
    ax.set_xlabel('hits/event',loc='right') 
    ax.set_ylabel('events', loc='top') 
    ax.legend(loc='upper right')
    #ax.set_title('', loc='right')
    #ax.set_yscale('log')
    #ax.set_xlim(right=250)
    ax.set_ylim(bottom=0)
    fig.savefig('extra-hits.png') 

