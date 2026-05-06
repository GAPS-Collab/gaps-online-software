#! /usr/bin/env python 

"""
"""

#import os
#import shutil
import tqdm
import gondola as go
#import time
#import re
import matplotlib.pyplot as plt 
import matplotlib 
matplotlib.use('agg')
import numpy as np
import dashi as d 
d.visual()

from pathlib import Path
#from glob import glob
#from copy import deepcopy
from dataclasses import dataclass
from fancy_dataclass import TOMLDataclass

# try to suppress RUST logging
import logging
logging.getLogger('go').addHandler(logging.NullHandler())

import charmingbeauty as cb 
cb.visual.set_style_default()
cb.visual.set_style_streamlit_dark()
import charmingbeauty.layout as lo

# check gondola version
if not (go.get_version_minor() >= 12 and go.get_version_patch() >= 17):
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

@dataclass 
class MergedSummary(TOMLDataclass): 
    """
    Simple logging of some quantities during 
    the creation of the run data.
    """ 
    tracker_unpack_errors         : int   = 0
    total_tracker_daq_packets     : int   = 0
    n_multiple_evids              : int   = 0
    associated_tracker_daq_events : int   = 0
    total_tracker_daq_events      : int   = 0 
    abs_smallest_delta_time       : float = np.inf
    abs_largest_delta_time        : float = 0
    used_gcu_cutoff_delta         : int   = 0 

def load_run(fname, telemetry_dir, seconds_pre=240, seconds_post=480, packet_offset=0, chunksize=0):
    """
    Load a run from a metadata file
    """
    with open(fname, "r") as f:
        run_meta = RunMeta.load(f)
    files = go.io.grace_get_telemetry_binaries(run_meta.start_gcu_time - seconds_pre, run_meta.stop_gcu_time + seconds_post, telemetry_dir)
    files  = [str(f) for f in files]
    reader = go.io.TelemetryPacketReader(files, skip_ahead=packet_offset, stop_after=chunksize)
    packs  = [pack for pack in reader if pack.header.gcutime >= run_meta.start_gcu_time and pack.header.gcutime <= run_meta.stop_gcu_time]
    return packs

if __name__ == '__main__':

    import argparse
    #import sys

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
    parser.add_argument('--gcu-seconds-time-cutoff', type=int,\
                        default=100, help='How many seconds of the gcu clock are allowed to pass between tracker and merged event packets to still allow the merge? Typically the time gap is ~60s. If this number is too large, we risk merging event ids from different runs')
    parser.add_argument('--control-plots', action='store_true',\
                        default=False,
                        help='More verbose output')
    
    #parser.add_argument('-v','--verbose', action='store_true',\
    #                    help='More verbose output')
    args = parser.parse_args()
  
    run_meta = Path(args.run_dir)
    run_meta = [k for k in run_meta.glob('*.meta.toml')]
    #print (run_meta)
    run_meta = run_meta[0]

    run_meta_data      = RunMeta.load(run_meta)
    max_packs_per_iter = int(4e6) # a typical 1h run has like 3.5M packets
    packet_offset      = 0
    iteration          = 0
    
    start_gcu_time : float = 0
    stop_gcu_time  : float = 0
    
    # auxgcu packets 
    seconds_pre    = 60
    seconds_post   = 60 
    telemetry_dir  = '/data-ssd0/flight-auxgcu-test-hour'
    auxgcu_bins    = go.io.grace_get_telemetry_binaries
    auxgcu_bins    = go.io.grace_get_telemetry_binaries(run_meta_data.start_gcu_time - seconds_pre, run_meta_data.stop_gcu_time + seconds_post, telemetry_dir)
    #print (auxgcu_bins)
    auxgcu_bins    = telemetry_dir
    auxgcu_reader  = go.io.TelemetryPacketReader(auxgcu_bins) 
    #print (auxgcu_reader.count_packets())
    auxgcu_tracker = [pack for pack in auxgcu_reader if pack.packet_type == go.packets.TelemetryPacketType.Tracker]
    print (f'-> Found {len(auxgcu_tracker)} tracker events!') 
    # unpack the tracker packets
    auxhits = dict() 
    for pack in tqdm.tqdm(auxgcu_tracker[:1000000]):
        try:
            ev = go.events.TrackerDAQEventPacket.from_telemetrypacket(pack)
        except Exception as e:
            print (f'ERROR Unpacking caused {e}') 
    #        log.tracker_unpack_errors += 1
            continue
        evids = ev.event_ids 
        for eid in evids:
            sub_event = ev.get_event_for_evid(eid) 
            if not eid in auxhits:
                auxhits[eid] = sub_event.hits 
            else:
                auxhits[eid].extend(sub_event.hits)
    #    log.total_tracker_daq_packets += 1
    l0files    = [str(f) for f in Path('/data-ssd0/L0/new-try/10420').glob('*.gaps')] 
    #print (l0files)
    all_n_wastie = []
    print ('-> Will read caraspace data!')
    end = False 
    for f in tqdm.tqdm(l0files):
        run_reader = go.io.CRReader(f)
        if end:
            break
        for frame in run_reader:
            #print (frame)
            #raise
            ev = frame.get_telemetryevent('TelemetryEvent')
            extra_trk_hits = frame.get_tracker_hitseries()
            #print(f'-> Event has {len(ev.tracker)} tracker hits')
            #print(f'-> We got {len(extra_trk_hits)} hits from the tracker events')
            extra_tracker_hits = [h for h in extra_trk_hits if not h in ev.tracker]
            #print(f'-> These are  {len(extra_trk_hits)} extra hits')
            ev.add_tracker_hits(extra_trk_hits)
            # now add wastie 
            n_wastie = 0
            if not ev.event_id in auxhits:
                continue
            for h in auxhits[ev.event_id]:
                if not h in ev.tracker:
                    n_wastie += 1
            #print (f'-> Added {n_wastie} wastie hits') 
            all_n_wastie.append(n_wastie)
            if len(all_n_wastie) >= 10000:
                end = True 
                break 
    print ('-> Creating histogram...')
    fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE)
    ax  = fig.gca() 
    h   = d.factory.hist1d(all_n_wastie, 70)
    h.line(filled=True, color='w', alpha=0.7)
    ax.set_ylim(bottom=0)
    ax.set_xlabel('N Hits added (wastie)', loc='right')
    fig.savefig('wastie-added-hits.png')
    #while True:
    #    print(f'-> Iteration {iteration}')
    #    iteration      += 1
    #    # now we have to go in chunks, since maybe all is too long
    #    chunksize       = max_packs_per_iter 
    #    run_packets     = load_run(run_meta, args.telemetry_dir, packet_offset = packet_offset, chunksize = chunksize )
    #    if not run_packets:
    #        break
    #    first_timestamp = go.io.get_utc_timestamp_from_unix(run_packets[0].header.gcutime)
    #    print (f'-> We found {len(run_packets)} telemetry packets for this run!') 
    #    print (f'-> First timestamp is {first_timestamp}')
    #    m_events = [pack for pack in run_packets if pack.is_event_packet]
    #    tracker  = [pack for pack in run_packets if int(pack.header.packet_type) == 80]
    #    m_events = [(go.events.TelemetryEvent.from_telemetrypacket(pack), pack) for pack in m_events]
    #    print (f'-> Found {len(m_events)} merged events for run {run_meta_data.run_id}')
    #    m_events = sorted(m_events, key=lambda k : f'{k[0].tof.run_id}{k[0].tof.event_id}')
    #    all_m_events = len(m_events)
    #    # remove run ids 0 
    #    m_events = [ev for ev in m_events if ev[0].tof.run_id != 0]
    #    print (f'-> Filtered out {all_m_events - len(m_events)} events with run id 0')
    #    #m_events_no_tr = []
    #    re_merged = {ev[0].tof.event_id : (ev,[]) for ev in m_events}
    #    if len(m_events) == 0 and len(tracker) == 0:
    #        break
    #    log                           = MergedSummary() 
    #    log.used_gcu_cutoff_delta     = args.gcu_seconds_time_cutoff 
    #    gcu_time_differences          = []
    #    for pack in tqdm.tqdm(tracker):
    #        try:
    #            ev = go.events.TrackerDAQEventPacket.from_telemetrypacket(pack)
    #        except Exception as e:
    #            print (f'ERROR Unpacking caused {e}') 
    #            log.tracker_unpack_errors += 1
    #            continue
    #        log.total_tracker_daq_packets += 1
    #        remaining = len(ev.event_ids)
    #        for evid in ev.event_ids:
    #            candidates = []
    #            empty_ev   = ev.emit_empty()
    #            if evid in re_merged.keys():
    #                # the first packet is always the merged event
    #                delta_gcutime = pack.header.gcutime - re_merged[evid][0][0].header.gcutime
    #                gcu_time_differences.append(delta_gcutime) 
    #                delta_gcutime = abs(delta_gcutime)
    #                # seatbelt - kick out packets which are far apart 
    #                # in time
    #                if delta_gcutime < 100:
    #                    candidates.append((delta_gcutime,ev)) 
    #                if delta_gcutime < log.abs_smallest_delta_time:
    #                    log.abs_smallest_delta_time = delta_gcutime
    #                if delta_gcutime > log.abs_largest_delta_time:
    #                    log.abs_largest_delta_time = delta_gcutime
    #            
    #            # this creates a list of candidates to which run this 
    #            # trackerevent might belong
    #            if candidates:
    #                # we just sort by gcutime difference and then use the 
    #                # one closest in time - throw the rest away  
    #                candidate = sorted(candidates, key=lambda x : x[0])[0]
    #                if len(candidates) > 1:
    #                    log.n_multiple_evids += 1
    #                __,ev   = candidate
    #                
    #                # check if the tof event id valid flag is set
    #                sub_event = ev.get_event_for_evid(evid) 
    #                if sub_event.flags1 & 0b1:
    #                    # we put the event in the now empty TrackerPacket 
    #                    empty_ev.add_event( sub_event)
    #                    re_merged[evid][1].append(empty_ev)
    #                    log.associated_tracker_daq_events += 1
    #            # all events which have undergone any processing
    #            log.total_tracker_daq_events += 1
    #    logfilename = args.run_dir / 'l0-caraspace.log'
    #    with open(logfilename,'w') as logfile:
    #        log.to_toml(logfile)
    #    
    #    #for k in m_events:
    #    print(f'--> We saw {log.tracker_unpack_errors} unpacking errors for tracker event packets!')
    #    print(f'--> We got {100*log.n_multiple_evids/len(tracker):.3f}% multiple evids!')
    #    print(f'--> We got {log.n_multiple_evids} (total) multiple evids!')
    #    
    #    print(f'--> The smallest abs delta gcutime for multiple events was {log.abs_smallest_delta_time}')
    #    print(f'--> The largest abs delta gcutime for multiple events was {log.abs_largest_delta_time}')
    #    print(f'--> There were {log.tracker_unpack_errors} errors when unpacking merged events!')
    #    print(f'--> We saw {log.total_tracker_daq_packets} tracker daq events')
    #    print(f'--> We associated {log.associated_tracker_daq_events} tracker events with the merged events!')
    #    print(f'--> That is ~{float(log.associated_tracker_daq_events)/len(tracker):.5f} events/packet')
 
    #    if args.control_plots:
    #        # control plots - gcu time as used in the merging 
    #        fig = plt.figure(figsize=cb.layout.FIGSIZE_A4_LANDSCAPE) 
    #        ax  = fig.gca() 
    #        tbins = np.linspace(-250,250,100)
    #        h   = d.factory.hist1d(gcu_time_differences, tbins) 
    #        h.line(filled=True, color='w', alpha=0.7)
    #        ax.set_xlabel('ns',loc='right') 
    #        ax.set_ylabel('events', loc='top') 
    #        ax.set_title('Packet GCU time delta, tracker packets earlier', loc='right')
    #        ax.set_yscale('log')
    #        fname = args.run_dir / 'delta_gcutimes_merging_neg.png'
    #        fig.savefig(fname) 
    #        
    #        fig = plt.figure(figsize=cb.layout.FIGSIZE_A4_LANDSCAPE) 
    #        ax  = fig.gca() 
    #        tbins = np.linspace(250,1000,100)
    #        h   = d.factory.hist1d(gcu_time_differences, tbins) 
    #        h.line(filled=True, color='w', alpha=0.7)
    #        ax.set_xlabel('ns',loc='right') 
    #        ax.set_ylabel('events', loc='top') 
    #        ax.set_title('Packet GCU time delta, tracker packets later', loc='right')
    #        ax.set_yscale('log')
    #        fname = args.run_dir / 'delta_gcutimes_merging_pos.png'
    #        fig.savefig(fname) 

    #    print ('-> Write carasapcae files')

    #    # create frames, and write them out later 
    #    frames = []
    #    sorted_evids = sorted(re_merged.keys())
    #    for k in sorted_evids:
    #        frame = go.io.CRFrame()
    #        frame.put_telemetrypacket(re_merged[k][0][1], 'TelemetryEvent', record_timestamp=True) 
    #        for idx, trk_ev in enumerate(re_merged[k][1]):  
    #            frame.put_telemetrypacket(trk_ev.pack(), f'Tracker_{idx}') 
    #        frames.append(frame) 

    #    writer         = go.io.CRWriter(str(args.run_dir), run_meta_data.run_id, timestamp=first_timestamp, file_len_gcu_sec=60)
    #    writer.set_mbytes_per_file(0)
    #    for frame in frames:
    #        writer.add_frame(frame)
    #    packet_offset += chunksize 
