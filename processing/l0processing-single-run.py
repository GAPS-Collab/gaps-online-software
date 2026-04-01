#! /usr/bin/env python 

# Strategy
#
# First - remerge tracker packet 80 with tof data 

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
if not (go.get_version_minor() >= 12 and go.get_version_patch() >= 8):
    print(f'ERROR - got version {go.get_version_major()}.{go.get_version_minor()}.{go.get_version_patch()}')
    raise ImportError("gondola needs to be at least version 0.12.8!")

@dataclass 
class RunMeta(TOMLDataclass): 
    """
    Write out some statistics for this run and 
    write it ultimately to a .toml file on disk
    """
    start_gps_time : float = 0 
    stop_gps_time  : float = 0
    start_gcu_time : float = 0
    stop_gcu_time  : float = 0
    run_id         : int   = 0
    start_event_id : int   = 0
    stop_event_id  : int   = 0
    missing_evids  : int   = 0
    runtime_h      : float = 0
    n_events       : int   = 0
    avg_rate       : float = 0

def load_run(fname, telemetry_dir, seconds_pre=240, seconds_post=480):
    with open(fname, "r") as f:
        run_meta = RunMeta.load(f)
    files = go.io.grace_get_telemetry_binaries(run_meta.start_gcu_time - seconds_pre, run_meta.stop_gcu_time + seconds_post, telemetry_dir)
    files  = [str(f) for f in files]
    reader = go.io.TelemetryPacketReader(files)
    packs  = [pack for pack in reader if pack.header.gcutime >= run_meta.start_gcu_time and pack.header.gcutime <= run_meta.stop_gcu_time]
    return packs

if __name__ == '__main__':

    import argparse
    import sys

    description = """Pre-process the telemetry data for the usage with SimpleDet"""
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument('--run-dir', default=Path('/data0/gaps/csbf/csbf-data/binaries/ethernet'),\
                        help='A directory with telemetry binaries, as received from the telemetry stream',\
                        type=Path,
                        )
    parser.add_argument('--telemetry-dir', default=Path('/data0/gaps/csbf/csbf-data/binaries/ethernet'),\
                        help='A directory with telemetry binaries, as received from the telemetry stream',\
                        type=Path,
                        )
    #parser.add_argument('-n','--npackets', type=int,\
    #                    default=-1, help='Limit readout to npackets, -1 for all packets (default)')
    #parser.add_argument('--n-tof-files', type=int,\
    #                    default=-1, help='Limit the readout to number of tif files, -1 for all files (default)')
    #parser.add_argument('--tof-dir', default='', type=Path,\
    #                    help='A directory with tof data files (.tof.gaps)',)
    #parser.add_argument('-s','--start-time',\
    #                    type=int, default=-1,\
    #                    help='The run start time, e.g. as taken from the elog')
    #parser.add_argument('-e','--end-time',
    #                    type=int, default=-1,\
    #                    help='The run end time, e.g. as taken from the elog')
    #parser.add_argument('-r','--run-id', default=-1, type=int,\
    #                    help='TOF Run id (only relevant when working with TOF files')
    parser.add_argument('-o','--outdir',\
                        help='Outdir for caraspace output files',
                        type=Path,
                        default=None)
    parser.add_argument('--load-remainder',\
                        help='Load a folder with files which could not be associated the last time this script was run',
                        type=Path,
                        default=None)
    
    #parser.add_argument('-v','--verbose', action='store_true',\
    #                    help='More verbose output')
    #parser.add_argument('--no-gps', action='store_true', \
    #                    help='Ignore the GPS to find matching telemetry and tof timestamps. Only use the tof file timestamps')
    #parser.add_argument('--reprocess', action='store_true', \
    #                    help='Recalculate tof packets with latest version of the code')
    args = parser.parse_args()
  
    run_meta = Path(args.run_dir)
    run_meta = [k for k in run_meta.glob('*.meta.toml')]
    print (run_meta)
    run_meta = run_meta[0]
    run_meta_data = RunMeta.load(run_meta)
    #print (run_meata) 
    
    run_packets = load_run(run_meta, args.telemetry_dir )
    print (f'-> We found {len(run_packets)} telemetry packets for this run!') 
    m_events = [pack for pack in run_packets if pack.is_event_packet]
    tracker  = [pack for pack in run_packets if int(pack.header.packet_type) == 80]
    m_events = [(go.events.TelemetryEvent.from_telemetrypacket(pack), pack) for pack in m_events]
    print (f'-> Found {len(m_events)} merged events for run {run_meta_data.run_id}')
    m_events = sorted(m_events, key=lambda k : f'{k[0].tof.run_id}{k[0].tof.event_id}')
    all_m_events = len(m_events)
    # remove run ids 0 
    m_events = [ev for ev in m_events if ev[0].tof.run_id != 0]
    print (f'-> Filtered out {all_m_events - len(m_events)} events with run id 0')
    #m_events_no_tr = []
    re_merged = {ev[0].tof.event_id : (ev,[]) for ev in m_events}

    # deal with the tracker packets
    tracker_unpack_errors         = 0
    total_tracker_daq_packets     = 0 
    n_multiple_evids              = 0
    associated_tracker_daq_events = 0
    total_tracker_daq_events      = 0
    smallest_delta_time           = np.inf
    gcu_time_differences          = []
    for pack in tqdm.tqdm(tracker):
        try:
            ev = go.events.TrackerDAQEventPacket.from_telemetrypacket(pack)
        except Exception as e:
            print (f'ERROR Unpacking caused {e}') 
            tracker_unpack_errors += 1
            continue
        total_tracker_daq_packets += 1
        remaining = len(ev.event_ids)
        for evid in ev.event_ids:
            candidates = []
            empty_ev   = ev.emit_empty()
            if evid in re_merged.keys():
                # the first packet is always the merged event
                delta_gcutime = pack.header.gcutime - re_merged[evid][0][0].header.gcutime
                gcu_time_differences.append(delta_gcutime) 
                delta_gcutime = abs(delta_gcutime)
                # seatbelt - kick out packets which are far apart 
                # in time
                if delta_gcutime < 100:
                    candidates.append((delta_gcutime,ev)) 
                if delta_gcutime < smallest_delta_time:
                    smallest_delta_time = delta_gcutime
            
            # this creates a list of candidates to which run this 
            # trackerevent might belong
            if candidates:
                # we just sort by gcutime difference and then use the 
                # one closest in time - throw the rest away  
                candidate = sorted(candidates, key=lambda x : x[0])[0]
                if len(candidates) > 1:
                    n_multiple_evids += 1
                __,ev   = candidate
                
                # check if the tof event id valid flag is set
                sub_event = ev.get_event_for_evid(evid) 
                if sub_event.flags1 & 0b1:
                    # we put the event in the now empty TrackerPacket 
                    empty_ev.add_event( sub_event)
                    re_merged[evid][1].append(empty_ev)
                    associated_tracker_daq_events += 1
                #ev.remove_event(evid) 
            # all events which have undergone any processing
            total_tracker_daq_events += 1
    
    #for k in m_events:
    print(f'--> We saw {tracker_unpack_errors} unpacking errors for tracker event packets!')
    print(f'--> We got {100*n_multiple_evids/len(tracker):.3f}% multiple evids!')
    print(f'--> We got {n_multiple_evids} (total) multiple evids!')
    
    print(f'--> The smallest delta gcutime for multiple events was {smallest_delta_time}')
    print(f'--> There were {tracker_unpack_errors} errors when unpacking merged events!')
    print(f'--> We saw {total_tracker_daq_packets} tracker daq events')
    print(f'--> We associated {associated_tracker_daq_events} tracker events with the merged events!')
    print(f'--> That is ~{float(associated_tracker_daq_events)/len(tracker):.5f} events/packet')
   
    fig = plt.figure(figsize=cb.layout.FIGSIZE_A4_LANDSCAPE) 
    ax  = fig.gca() 
    tbins = np.linspace(-250,250,100)
    h   = d.factory.hist1d(gcu_time_differences, tbins) 
    h.line(filled=True, color='w', alpha=0.7)
    ax.set_xlabel('ns',loc='right') 
    ax.set_ylabel('events', loc='top') 
    ax.set_title('Packet GCU time difference', loc='right')
    ax.set_yscale('log')
    #ax.set_xlim(right=250)
    fig.savefig('delta_gcutimes_merging-neg.png') 
    
    fig = plt.figure(figsize=cb.layout.FIGSIZE_A4_LANDSCAPE) 
    ax  = fig.gca() 
    tbins = np.linspace(250,1000,100)
    h   = d.factory.hist1d(gcu_time_differences, tbins) 
    h.line(filled=True, color='w', alpha=0.7)
    ax.set_xlabel('ns',loc='right') 
    ax.set_ylabel('events', loc='top') 
    ax.set_title('Packet GCU time difference', loc='right')
    ax.set_yscale('log')
    #ax.set_xlim(right=250)
    fig.savefig('delta_gcutimes_merging-pos.png') 

    # create frames, and write them out later 
    frames = [] 
    for k in re_merged:
        frame = go.io.CRFrame()
        #print (re_merged[k][0])
        frame.put_telemetrypacket(re_merged[k][0][1], 'TelemetryEvent') 
        for idx, trk_ev in enumerate(re_merged[k][1]):  
            frame.put_telemetrypacket(trk_ev.pack(), f'Tracker_{idx}') 
        frames.append(frame) 

    writer         = go.io.CRWriter(str(args.run_dir), run_meta_data.run_id)
    for frame in frames:
        writer.add_frame(frame)

