#! /usr/bin/env python 

"""
Adds extra tracker hits from disk drives, which have 
been dismissed from the telemetry by "wastie".
"""

import re
import tqdm
import gondola as go
import matplotlib.pyplot as plt 
import matplotlib 
matplotlib.use('agg')
import numpy as np
import dashi as d 
d.visual()

from pathlib import Path
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
    
    description = __doc__ 
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument('--run-dir', default=Path('/data0/gaps/csbf/csbf-data/binaries/ethernet'),\
                        help='A directory with .gaps (caraspace) files with flight data. All monitoring will be ignored',\
                        type=Path,
                        )
    parser.add_argument('-o','--outdir', default=Path(''),\
                        help='Output directory, which will hold the L0files with added wastie hits',\
                        type=Path,
                        )
    parser.add_argument('--wastie-dir', default=Path(''),\
                        help='A directory with telemetry binaries, as they were recovered from the disks of the flight instrument. The hits removed by wastie can solely be found on the tracker disks, e.g auxgcu',\
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
 
    #-------------------------------
    # strategy - we are scared of memory leaks. 
    # thus we get per each L0 file a few new binary files
    # in that way we work us through the whole run 

    # meta data
    run_meta      = Path(args.run_dir)
    run_meta      = [k for k in run_meta.glob('*.meta.toml')]
    #print (run_meta)
    run_meta      = run_meta[0]
    run_meta_data = RunMeta.load(run_meta)
    
    # now get all the files, and walk over them one-by-one 
    crfiles = [k for k in args.run_dir.glob('*.gaps')] 
    
    # timewindow for telemetry files 
    seconds_pre    = 65
    seconds_post   = 65 
    for f in tqdm.tqdm(crfiles):
        # first we check for first and last gcutime 
        crreader = go.io.CRReader(f) 
        for frame in crreader:
            firsttime = frame.get_first_gcutime() 
            break
        for frame in crreader:
            lasttime  = frame.get_first_gcutime() 
        crreader.rewind() 
        print (f'-> Found first gcu time {firsttime} and last gcu time {lasttime}')
        
        # now get the data from the aux disks
        auxgcu_bins    = go.io.grace_get_telemetry_binaries(firsttime - seconds_pre, lasttime + seconds_post, args.wastie_dir)
        auxgcu_bins = [str(b) for b in auxgcu_bins]
        print (f'-> We found the following binary files {auxgcu_bins}')
        if not len(auxgcu_bins):
            continue
        auxgcu_reader  = go.io.TelemetryPacketReader(auxgcu_bins) 
        # could count the packets here and prevent further execution?

        #print (auxgcu_reader.count_packets())
        auxgcu_tracker = [pack for pack in auxgcu_reader if pack.packet_type == go.packets.TelemetryPacketType.Tracker]
        print (f'-> Found {len(auxgcu_tracker)} tracker events!') 
        # unpack the tracker packets
        auxevents = dict() 
        
        for pack in tqdm.tqdm(auxgcu_tracker):
            try:
                ev = go.events.TrackerDAQEventPacket.from_telemetrypacket(pack)
            except Exception as e:
                print (f'ERROR Unpacking caused {e}') 
        #        log.tracker_unpack_errors += 1
                continue
            evids = ev.event_ids 
            for eid in evids:
                sub_event = ev.get_event_for_evid(eid) 
                if not eid in auxevents:
                    empty_ev = ev.emit_empty() 
                    empty_ev.add_event(sub_event)
                    auxevents[eid] = [empty_ev]
                else:
                    empty_ev = ev.emit_empty() 
                    empty_ev.add_event(sub_event)
                    auxevents[eid].append(empty_ev)
        pattern = re.compile("Run(?P<runid>\d+)_(?P<subrunid>\d+)\.(\d{6})_(\d{6})(UTC)?(\.tofsum|\.tof)?\.gaps$")
        fmeta      = pattern.search(f.name).groupdict()
        ftimestamp = go.io.get_utc_timestamp_from_unix(firsttime)  
        writer     = go.io.CRWriter(str(args.outdir), int(fmeta['runid']), int(fmeta['subrunid']),timestamp=ftimestamp ) 
        #print (writer)
        for frame in crreader:
            evid = frame.get_telemetryevent('TelemetryEvent').event_id 
            if evid in auxevents:
                for idx, trk_ev in enumerate(auxevents[evid]):
                    frame.put_telemetrypacket(trk_ev.pack(), f'TrkAuxGcu_{idx}') 
            writer.add_frame(frame)


