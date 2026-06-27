#! /usr/bin/env python 

"""
Allows to add extra packets to pre-existing L0 data. This is meant to add additional 
sources (either Tracker hits of some kind, e.g. from the auxgcu disks, or Tof data of 
any kind). The source is supposed to be either TelemetryPackets or TofPackets. 

The script will write a copy of the runfiles into a dedicated subfolder of the run, 
simply called tmp. If executed successfully, the files inside this directory completely 
replace the initial inputfiles

FIXME - currently adding TOF packets is not supported 
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
GON_VERSION_REQUIRED = '0.12.8' 
if not go.version_at_least(GON_VERSION_REQUIRED):
    print(f'ERROR - got version {go.get_version()} but need version {GON_VERSION_REQUIRED}')
    raise ImportError("gondola needs to be at least version {GON_VERSION_REQUIRED}!")

if go.version_at_least('0.12.26'): 
    RunMeta  = go.run.RunMeta 

else:
    # if we have a gondola version smaller than 0.12.26, we have to create our RunMeta data here 
    from fancy_dataclass import TOMLDataclass

    @dataclass 
    class RunMeta(TOMLDataclass): 
        """
        Write out some statistics for this run and 
        write it ultimately to a .toml file on disk
        """
        start_gps_time : float = np.inf 
        stop_gps_time  : float = 0
        start_gcu_time : float = np.inf
        stop_gcu_time  : float = 0
        run_id         : int   = 0
        start_event_id : int   = np.inf
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


if __name__ == '__main__':

    import argparse
    
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run-dir', default=Path('/data0/gaps/csbf/csbf-data/binaries/ethernet'),\
                        help='A directory with .gaps (L0/caraspace) files containing Telemetry events. One single run only',\
                        type=Path,
                        )
    parser.add_argument('--packet-type', default=80,\
                        help='Add packets of this TelemetryPacketType from the additonal tlemetry files in "telemetry dir"',\
                        type=int,
                        )
    
    parser.add_argument('--telemetry-dir', default=Path(''),\
                        help='A directory with telemetry binaries, as they were recovered from the disks of the flight instrument. The hits removed by wastie can solely be found on the tracker disks, e.g auxgcu',\
                        type=Path,
                        )
    parser.add_argument('--packet-tag', default='TrkAuxGcu',\
                        help='A dedicated tag to mark the new hits in the frame',\
                        type=Path,
                        )
    parser.add_argument('--quiet', action='store_true',\
                        help='Suppress unnecessary output (e.g. progressbar) for use on cluster!')
    parser.add_argument('--gcu-seconds-time-cutoff', type=int,\
                        default=100, help='How many seconds of the gcu clock are allowed to pass between tracker and merged event packets to still allow the merge? Typically the time gap is ~60s. If this number is too large, we risk merging event ids from different runs')
    
    #parser.add_argument('-v','--verbose', action='store_true',\
    #                    help='More verbose output')
    args = parser.parse_args()
 
    #-------------------------------
    # strategy - we are scared of memory leaks. 
    # thus we get per each L0 file a few new binary files
    # in that way we work us through the whole run 

    # meta data
    run_meta      = Path(args.run_dir)
    print(f'-> Checking run dir {run_meta}')
    run_meta      = [k for k in run_meta.glob('*.meta.toml')]
    #print (run_meta)
    run_meta      = run_meta[0]
    run_meta_data = RunMeta.load(run_meta)
    
    # now get all the files, and walk over them one-by-one 
    crfiles = [k for k in args.run_dir.glob('*.gaps')] 
    
    # timewindow for telemetry files 
    seconds_pre    = 69
    seconds_post   = 69 
    
    # the packet type of the source    
    ptype = go.packets.TelemetryPacketType.from_u8(args.packet_type) 

    for f in tqdm.tqdm(crfiles, desc=f"Adding packets of type {args.packet_type}", disable=args.quiet ):
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
        telemetry_files    = go.io.grace_get_telemetry_binaries(firsttime - seconds_pre, lasttime + seconds_post, args.telemetry_dir)
        telemetry_files = [str(b) for b in telemetry_files]
        print (f'-> We found {len(telemetry_files)} telemetry files for {f}')
        if not len(telemetry_files):
            continue
        telemetry_reader  = go.io.TelemetryPacketReader(telemetry_files) 
        # could count the packets here and prevent further execution?

        #print (telemetry_reader.count_packets())
        telemetry_packets = [pack for pack in telemetry_reader if pack.packet_type == ptype]
        print (f'-> Found {len(telemetry_packets)} packets of type {args.packet_type}!') 
        auxevents = dict() 
        
        #for pack in tqdm.tqdm(telemetry_packets,disable=args.quiet, desc='Adding packets from telemetry files..'):
        for pack in tqdm.tqdm(telemetry_packets):
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
        tmp_outdir = args.run_dir / 'tmp' 
        tmp_outdir.mkdir(parents=True, exist_ok=True) 
        writer     = go.io.CRWriter(str(tmp_outdir), int(fmeta['runid']), int(fmeta['subrunid']),timestamp=ftimestamp ) 
        #print (writer)
        current_fname = Path(crreader.current_filename) 
        writer_current_fname = Path(writer.current_filename)
        for frame in crreader:
            # FIXME - we would like to want a method that can get the 
            #         event id from a frame without deserializing 
            #         everything
            evid = frame.get_telemetryevent('TelemetryEvent').event_id 
            if evid in auxevents:
                for idx, trk_ev in enumerate(auxevents[evid]):
                    frame.put_telemetrypacket(trk_ev.pack(), f'{args.packet_tag}_{idx}') 
            writer.add_frame(frame)
        # the work on this file has been concluded. CHeck if the size is larger than the input file 
        # and if so, move it out of the tmp directory (?) 
        if current_fname.stat().st_size < writer_current_fname.stat().st_size:
            print (f'-> {current_fname} -> {writer_current_fname} with added packets!') 
        else:
            print (f'-> [ERROR] {writer_current_fname} failed filesize check') 
print(f'-> Finished!')

