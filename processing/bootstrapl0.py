#! /usr/bin/env python 

"""
Bootstrap the telemetry data. This script will go through all 
Telemetry data and create a directory structure with one 
directory per run, including some metadata for this run. 

Another script can then parse binary data based on this run 
meta informtion
"""

## Some overview information about the different disk drives 
# 2 disks on the gcu
#(py3) gaps@gse7:~$ ls /gaps_binaries/gcu/disk_a/ | wc -l
#881
#(py3) gaps@gse7:~$ ls /gaps_binaries/gcu/disk_b/ | wc -l
#10692
# telemetry (starlink)
#(py3) gaps@gse7:~$ ls /gaps_binaries/live/raw/starlink | wc -l
#54997
# 2 drives on the aux gcu
#(py3) gaps@gse7:~$ ls /sqlRAID/flight_drives/aux_gcu_1_gaps_data/ | wc -l
#42511
#(py3) gaps@gse7:~$ ls /sqlRAID/flight_drives/aux_gcu_2_backup_data/ | wc -l
#42515
# --- UH
# on uhcra we have 
#❯ ls /data1/nextcloud/cra_data/data/binaries_berkeley/starlink | wc -l
#60257 [not consistent with B]
#❯ ls /data1/nextcloud/cra_data/data/binaries_berkeley/auxgcu/disk_a | wc -l
#30188 [not consistent with B]
# FIXME !! Here is another one, which should be the actual gcu disks! 
# ls /data1/nextcloud/cra_data/data/binaries_berkeley/gcu_2_gcupool | wc -l
#53425

import os
import shutil
import tqdm
import gondola as go
import time
import re
import matplotlib.pyplot as plt 
import numpy as np

from pathlib import Path
from glob import glob
from copy import deepcopy
from dataclasses import dataclass

# try to suppress RUST logging
import logging
logging.getLogger('go').addHandler(logging.NullHandler())

import matplotlib 
import matplotlib.pyplot as plt 

matplotlib.use('agg')

import charmingbeauty as cb 
cb.visual.set_style_default()
# join the dark side 
cb.visual.set_style_streamlit_dark() 

# check gondola version
GON_VERSION_REQUIRED = '0.12.32' 
if not go.version_at_least(GON_VERSION_REQUIRED):
    print(f'ERROR - got version {go.get_version()} but need version {GON_VERSION_REQUIRED}')
    raise ImportError("gondola needs to be at least version {GON_VERSION_REQUIRED}!")



# special run times from the ELOG for specific runs:
ELOG_RUNTIMES = {\
    251: (1765018167,1765054302),\
    243: (1764929769,1764966600),\
    240: (1764841147,1764868791)}



if __name__ == '__main__':

    import argparse
    import sys

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('-t','--telemetry-dir', default=Path('/data1/nextcloud/cra_data/data/binaries_berkeley/starlink'),\
                        help='A directory with telemetry binaries, as received from the telemetry stream',\
                        type=Path,
                        )
    parser.add_argument('-o','--outdir',\
                        help='Outdir for caraspace output files',
                        type=Path,
                        default=None)
    parser.add_argument('--only-run',\
                        help='Just process a single run',
                        type=int,
                        default=-1)
    parser.add_argument('--exclude-tracker-calib',\
                        help='Do not take any data into account which is happening during the tracker calibration',
                        action='store_true')
    parser.add_argument('--reuse-existing', action='store_true',\
                        help='Use a directory with an already existing directory structure and expand on that')
    parser.add_argument('--bootstrap-only', action='store_true',\
                        help='Only create the directory structure and quit after')
    parser.add_argument('--ground', action='store_true',\
                        help='Advises the script that it is intended to deal with ground data, and skip sanity checks for flight data')
    parser.add_argument('-v','--verbose', action='store_true',\
                        help='More verbose output')
    parser.add_argument('--quiet', action='store_true',\
                        help='Suppress unnecessary output (e.g. progressbar) for use on cluster!')
    #parser.add_argument('--packet-tag',\
    #                    help='When packing the TelemetryPackets in CRFrames, tag them with this prefix',
    #                    type=str,
    #                    default='SL')
    
    #parser.add_argument('--no-gps', action='store_true', \
    #                    help='Ignore the GPS to find matching telemetry and tof timestamps. Only use the tof file timestamps')
    #parser.add_argument('--reprocess', action='store_true', \
    #                    help='Recalculate tof packets with latest version of the code')
    args = parser.parse_args()

    if args.exclude_tracker_calib:
        tcw = go.db.TrackerCaliTimeWindow.all()
        if not tcw:
            raise ValueError("Not able to get tracker cali time windows from db!")
        if args.reuse_existing:
            raise ValueError("Unable to combine arguments --reuse-esisting with --exclude-tracker-calib, because, we don't know if that argument had been used when creating the run meta files!")

    if not args.reuse_existing: 
        # the binary files here need to live in a flat directory (no subdirectories) 
        data = Path(args.telemetry_dir) 
        if args.ground and args.only_run in ELOG_RUNTIMES.keys():
            run_id = int(args.only_run)
            print (f'-> Retrieving data specificially for ground run {run_id}!')
            start, stop = ELOG_RUNTIMES[run_id]
            data = go.io.grace_get_telemetry_binaries(start, stop, str(data))
            data = [str(k) for k in sorted(data)]
        else:
            # remove the tracker calibration times if desired 
            if args.exclude_tracker_calib:
                clean_data = []
                n_removed = 0
                for fname in sorted(data.glob('*.bin')): 
                    f_ts = go.io.get_unix_timestamp_from_telemetry(str(fname))
                    if go.db.is_in_tracker_cali_window(f_ts,tcw):
                        n_removed += 1
                    else:
                        clean_data.append(str(fname)) 
                print(f'-> Removed {n_removed} files because they fell into a tracker calibration window') 
                data = clean_data
            else:
                data = [str(k) for k in sorted(data.glob("*.bin"))]
                    
        print (f'-> We found {len(data)} .bin files in {args.telemetry_dir}! for the requested run/time period') 
                

        # store the run informaton in a dict runid -> metadata 
        # we seem to have 545 runs, add some for good measure 
        # the first run is 10000
        runs        = {10000 + k : go.run.RunMeta() for k in range(600)}
        # for ground data, this is different, We have all kinds of 
        # run numbers which are typically smaller than 10000
        if args.ground:
            runs = {k : go.run.RunMeta() for k in range(10000)}
        for k in runs:
            runs[k].run_id = k
        # tof configuration packets - keep that for the future for now, 
        # since the configuratoin itself does not know which run id it is 
        # it is easy to confuse them. Rather get the config files from the 
        # tof disk drives 
        #tof_configs = dict()
        
        # go with the one-file, one reader even though it is 
        # a bit slower, but then we can get our progressbar :) 
        for fname in tqdm.tqdm(data, desc='Looping over .bin files', disable=args.quiet):
            reader = go.io.TelemetryPacketReader(fname, dedup=args.ground) 
            for pack in reader:
                if pack.is_event_packet:
                    try:
                        ev     = go.events.TelemetryEvent.from_telemetrypacket(pack) 
                    except Exception as e:
                        print (e)
                        continue # deliberately fail silently. Deal with broken packets 
                                 # at some other time. For now, these will simply be 
                                 # missing
                    tof        = ev.tof
                    run_id     = tof.run_id 
                    gpstime    = tof.timestamp48*1e-8
                    event_id   = ev.event_id
                    gcutime    = pack.header.gcutime 
                    if run_id == 0:
                        continue # just the packet is broken, this can be passed on 
                                 # silently 
                    if run_id < 10000 and not run_id < 10000 and args.ground:
                        print("-> [WARN} There is a run id < 10000. This might be ground data?")
                        continue 
                    if args.ground and run_id >= 10000:
                        if args.verbose:
                            print ('-> [WARN] Supposed to process ground data, however, this is run {run_id}!')
                        continue
                    meta      = runs[run_id] 
                    if gcutime < meta.start_gcu_time or meta.start_gcu_time == 0:
                        meta.start_gcu_time = gcutime 
                    if gcutime > meta.stop_gcu_time:
                        meta.stop_gcu_time  = gcutime
                    if gpstime < meta.start_gps_time or meta.start_gps_time == 0:
                        meta.start_gps_time = gpstime 
                    if gpstime > meta.stop_gps_time:
                        meta.stop_gps_time  = gpstime 
                    if event_id < meta.start_event_id or meta.start_event_id == 0:
                        meta.start_event_id = event_id 
                    if event_id > meta.stop_event_id:
                        meta.stop_event_id  = event_id
                    meta.n_events += 1 
        # clean out non-populated runs 
        clean_runs = dict() 
        for r in runs:
            # if it doesn't have a stop event id, it is 
            # probably borked
            if runs[r].stop_event_id != 0:
                clean_runs[r] = runs[r]
                print (runs[r])
        print (f'-> Retrieved meta information for {len(clean_runs)} runs!')
        
        # create directories 
        for r in clean_runs:    
            run_dir = args.outdir / f'{r}' 
            run_dir.mkdir(parents=True, exist_ok=True) 
            metadata_file = Path(f'{run_dir}/run{r}.meta.toml') 
            with open(metadata_file,'w') as meta_f:
                meta = clean_runs[r]
                if args.ground:
                    # trypically, the gps is not available 
                    meta.runtime_h      = (meta.stop_gcu_time - meta.start_gcu_time)/3600 
                else:
                    meta.runtime_h      = (meta.stop_gps_time - meta.start_gps_time)/3600 
                meta.missing_evids  = (meta.stop_event_id  - meta.start_event_id) - meta.n_events
                meta.avg_rate       = meta.n_events / (3600*meta.runtime_h)
                meta.to_toml(meta_f)
        if args.bootstrap_only:
            print('-> Selceted to bootstrap only, concluding!') 
            sys.exit(0) 

    else: # if not args.reuse_existing 
        # we have to load the run meta data in outdir 
        clean_runs = dict() 
        run_dirs   = args.outdir.glob('*') 
        for rd in run_dirs: 
            meta = go.run.RunMeta.load(rd / f'run{rd.name}.meta.toml') 
            clean_runs[meta.run_id] = meta 

    print (f'-> {len(clean_runs)} runs available for processing!') 
    if not clean_runs:
        print (f'[ERROR] - no runs found!') 
        sys.exit(1) 
    
    #-- in any case, clean runs must contain our run id even if chose to 
    #-- process a single run only. Get start/stop times from there 
    first_time = clean_runs[args.only_run].start_gcu_time


    # -- the actual processing. Load the telemetry files and convert them 
    #     to caraspace files 1-1 
    # gcu time guard. Allow for a few seconds +- before and after start/stop times 
    seconds_pre, seconds_post = 120,120 
    for run_id in sorted(clean_runs):
        if args.only_run != -1:
            if not args.only_run in clean_runs:
                print (f'-> [ERROR] run to be requested is not available (run id {args.only_run})')
            if args.only_run != run_id:
                continue 
        print (f'-> Working on run {run_id}') 
        meta = clean_runs[run_id] 
        # set up a writer for the output    
        cr_timestamp = go.io.get_utc_timestamp_from_unix(float(meta.start_gcu_time) - seconds_pre)  
        writer       = go.io.CRWriter(str(args.outdir / Path(str(run_id))), run_id, timestamp=cr_timestamp, subrun_id = 0, file_len_gcu_sec = 60)
        # disable the automaitc splitting feature of the writer by setting 
        # an arbitrary, large number for the file size
        writer.set_mbytes_per_file(100000) # 100GB
        bin_files = go.io.grace_get_telemetry_binaries(float(meta.start_gcu_time) - seconds_pre, float(meta.stop_gcu_time) + seconds_post, args.telemetry_dir)
        # has been processed - use to avoid duplicate 
        seen = [] 
        nfile = 0
        for bfname in tqdm.tqdm(bin_files, desc='Reading telemetry',disable=args.quiet):
            ## load each file individually + gcu safeguard 
            ## this means, first we have to find out first
            ## and last gcutime 
            
            # We load all events from the file first, so taht we can sort it 
            # by event id
            treader = go.io.TelemetryPacketReader(str(bfname), dedup=args.ground) 
            tevents = []
            for pack in treader: 
                # bootstrapping - only select merged events for now!
                if (not pack.is_event_packet) or pack.packet_type == go.packets.TelemetryPacketType.NoTofDataEvent:
                    continue 
                # get event id, run _id 
                try:
                    if not pack.run_id == run_id:
                        continue 
                except Exception as e:
                    print (e) 
                    print (pack) 
                    raise
                # FIXME - in the future, the unpack step won't be necessary
                #ev = go.events.TelemetryEvent.from_telemetrypacket(pack) 
                #tevents.append((pack,ev)) 
                tevents.append(pack)
            tevents = sorted(tevents, key=lambda x : x.event_id) 
            if args.verbose:
                print (f'-> Extracted {len(tevents)} telemetry events from {bfname}!')
            # now we are writing the bin files to L0
            for pack in tevents:
                frame = go.io.CRFrame() 
                frame.put_telemetrypacket(pack, name='TelemetryEvent') 
                writer.add_frame(frame)
                
                if pack.header.gcutime - first_time > 50:
                    nfile += 1
                    # start a new file 
                    cr_timestamp = go.io.get_utc_timestamp_from_unix(pack.header.gcutime)  
                    writer       = go.io.CRWriter(str(args.outdir / Path(str(run_id))), run_id, subrun_id = nfile,  timestamp=cr_timestamp)
                    # we don't want the writer to start a new file basically at all. The respective option 
                    # where it starts new files automatically based on gcu time does not work and might 
                    # get removed
                    writer.set_mbytes_per_file(100000)
                    first_time = pack.header.gcutime 
    print(f'-> Finished!')
    sys.exit(0) 
