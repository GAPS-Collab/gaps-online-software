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
    parser.add_argument('--reuse-existing', action='store_true',\
                        help='Use a directory with an already existing directory structure and expand on that')
    parser.add_argument('--bootstrap-only', action='store_true',\
                        help='Only create the directory structure and quit after')
    parser.add_argument('-v','--verbose', action='store_true',\
                        help='More verbose output')
    #parser.add_argument('--packet-tag',\
    #                    help='When packing the TelemetryPackets in CRFrames, tag them with this prefix',
    #                    type=str,
    #                    default='SL')
    
    #parser.add_argument('--no-gps', action='store_true', \
    #                    help='Ignore the GPS to find matching telemetry and tof timestamps. Only use the tof file timestamps')
    #parser.add_argument('--reprocess', action='store_true', \
    #                    help='Recalculate tof packets with latest version of the code')
    args = parser.parse_args()

    if not args.reuse_existing: 
        # the binary files here need to live in a flat directory (no subdirectories) 
        data = Path(args.telemetry_dir) 
        data = [str(k) for k in sorted(data.glob("*.bin"))]
        print (f'-> We found {len(data)} .bin files in {args.telemetry_dir}!') 
        # store the run informaton in a dict runid -> metadata 
        # we seem to have 545 runs, add some for good measure 
        # the first run is 10000
        runs        = {10000 + k : RunMeta() for k in range(600)}
        for k in runs:
            runs[k].run_id = k
        # tof configuration packets - keep that for the future for now, 
        # since the configuratoin itself does not know which run id it is 
        # it is easy to confuse them. Rather get the config files from the 
        # tof disk drives 
        #tof_configs = dict()
        
        # go with the one-file, one reader even though it is 
        # a bit slower, but then we can get our progressbar :) 
        for fname in tqdm.tqdm(data, desc='Looping over .bin files'):
            reader = go.io.TelemetryPacketReader(fname) 
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
                    if run_id < 10000:
                        print("-> [WARN} There is a run id < 10000. This might be ground data?")
                        continue 
                    meta      = runs[run_id] 
                    if gcutime < meta.start_gcu_time:
                        meta.start_gcu_time = gcutime 
                    if gcutime > meta.stop_gcu_time:
                        meta.stop_gcu_time  = gcutime
                    if gpstime < meta.start_gps_time:
                        meta.start_gps_time = gpstime 
                    if gpstime > meta.stop_gps_time:
                        meta.stop_gps_time  = gpstime 
                    if event_id < meta.start_event_id:
                        meta.start_event_id = event_id 
                    if event_id > meta.stop_event_id:
                        meta.stop_event_id  = event_id
                    meta.n_events += 1 
                    # will be filled in later
                    #meta.missing_evids  
                    #meta.runtime_h      
                    #meta.n_events       
                    #meta.avg_rate       
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
            meta = RunMeta.load(rd / f'run{rd.name}.meta.toml') 
            clean_runs[meta.run_id] = meta 

    print (f'-> {len(clean_runs)} runs available for processing!') 
    
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
        # load all binary files! 
        bin_files = go.io.grace_get_telemetry_binaries(float(meta.start_gcu_time) - seconds_pre, float(meta.stop_gcu_time) + seconds_post, args.telemetry_dir)
        # has been processed - use to avoid duplicate 
        seen = [] 
        nfile = 0
        for bfname in tqdm.tqdm(bin_files, desc='Reading telemetry'):
            ## load each file individually + gcu safeguard 
            ## this means, first we have to find out first
            ## and last gcutime 
            #first_gcutime = np.inf 
            #last_gcutime  = -np.inf 
            treader = go.io.TelemetryPacketReader(str(bfname)) 
            tevents = []
            for pack in treader: 
                # bootstrapping - only select merged events for now!
                if (not pack.is_event_packet) or pack.packet_type == go.packets.TelemetryPacketType.NoTofDataEvent:
                    continue 
                # get event id, run _id 
                try:
                    if not pack.get_runid() == run_id:
                        continue 
                except Exception as e:
                    print (e) 
                    print (pack) 
                    raise
                # FIXME - in the future, the unpack step won't be necessary
                ev = go.events.TelemetryEvent.from_telemetrypacket(pack) 
                tevents.append((pack,ev)) 
            # FIXME - we can gain MASSIVELY here if we can extract the event id 
            #         from the packet as well 
            tevents = sorted(tevents, key=lambda x : x[1].event_id) 
            if args.verbose:
                print (f'-> Extracted {len(tevents)} telemetry events from {bfname}!')
            # if something useful has been extracted, write it to L0files 
            if tevents:
                first_time = tevents[0][0].header.gcutime 
            for pack,__ in tevents:
                frame = go.io.CRFrame() 
                frame.put_telemetrypacket(pack, name='TelemetryEvent') 
                writer.add_frame(frame)
                
                if pack.header.gcutime - first_time > 50:
                    nfile += 1
                    # start a new file 
                    cr_timestamp = go.io.get_utc_timestamp_from_unix(pack.header.gcutime)  
                    writer       = go.io.CRWriter(str(args.outdir / Path(str(run_id))), run_id, subrun_id = nfile,  timestamp=cr_timestamp)
                    first_time = pack.header.gcutime 
    sys.exit(0) 

            #for pack in treader:
            #    p_gcutime = pack.header.gcutime 
            #    if p_gcutime > last_gcutime:
            #        last_gcutime = p_gcutime 
            #    if p_gcutime < first_gcutime: 
            #        first_gcutime = p_gcutime 
            ## reload files with the safeguard 
            #bfs_for_l0 = go.io.grace_get_telemetry_binaries(first_gcutime - seconds_pre, last_gcutime + seconds_post, args.telemetry_dir)
            #treader = go.io.TelemetryPacketReader(bfs_for_l0) 
            #packs   = [pack for pack in treader if pack.is_event_packet] 
                         

            #print (f'-> Loaded {bfs_for_l0} for {bfname}') 
    raise   

    # one file is approximately 1min, many runs are ~1h
    # we always want to grab some overlap
    data_chunks = [data[i:i + 100] for i in range(0, len(data), 100)]
    print (f'-> We split the data up into {len(data_chunks)} chunks a 100 files each!')
    # just to make sure, let's check the start/stop times 
    first = go.io.TelemetryPacketReader(data_chunks[0][0]) 
    for pack in first:
        if pack.is_event_packet:
            break 
    first_gcu_time = pack.header.gcutime  
    first_event    = go.events.TelemetryEvent.from_telemetrypacket(pack)
    first_run_id   = first_event.tof.run_id 

    # check the last 6h for the last event 
    print ('-> Searching for the last event...')
    last_k = None 
    #for k in tqdm.tqdm(range(2)):
    #    k = -1*k
    for n in range(len(data_chunks[-1])):
        last = go.io.TelemetryPacketReader(data_chunks[-1][n])
        #print (last.count_packets())
        ev_pack = None 
        for pack in last:
            if pack.is_event_packet:
                #last_k = k 
                try:
                    last_event    = go.events.TelemetryEvent.from_telemetrypacket(pack)
                    last_gcu_time = pack.header.gcutime 
                except:
                    continue
    #print (f'-> Last index of chunks with events {last_k}')
    last_run_id   = last_event.tof.run_id 
    
    delta = (last_gcu_time - first_gcu_time) / (3600*24) 
    first_gcu_time = go.io.get_utc_timestamp_from_unix(first_gcu_time) 
    last_gcu_time  = go.io.get_utc_timestamp_from_unix(last_gcu_time) 
    print (f'-> Found telemetry files from {first_gcu_time} - {last_gcu_time} ({delta:.2f} days)')
    print (f'-> Found first run {first_run_id}, last {last_run_id}')
    
    run_meta     = {k : RunMeta() for k in range(first_run_id, last_run_id + 1)} 
    run_meta[10000] = RunMeta()
    run_meta[10000].run_id = 10000
    for k in run_meta:
        run_meta[k].run_id = k 
    tof_settings = [] 
    #print (run_meta)
    #sys.exit()
    #create directory structure 
    run_dirs = dict()
    for r in range(first_run_id, last_run_id + 1):
        run_dir = args.outdir / f'{r}' 
        run_dir.mkdir(parents=True, exist_ok=True) 
        run_dirs[r] = run_dir


    #--------------------------------------------------------------------- 
    
    # progressively find run/start stop information 
    for chunk in tqdm.tqdm(data_chunks):
        chunk  = [str(k) for k in chunk]
        reader = go.io.TelemetryPacketReader(chunk)
        packs  = [pack for pack in reader]
        last_run_id    = 0
        for pack in packs:
            if pack.is_tof_toml_packet:
                tof_settings.append((pack.header.gcutime, pack))
                print ('-> TOF .toml settigns found!')
            if pack.is_event_packet:
                this_ev = go.events.TelemetryEvent.from_telemetrypacket(pack) 
                current_run_id = this_ev.tof.run_id 
                try:
                    run_meta[current_run_id] 
                except KeyError:
                    print(f'-> Can not find meta information for run id {current_run_id}') 
                    continue
                if run_meta[current_run_id].start_gcu_time > pack.header.gcutime:
                    run_meta[current_run_id].start_gcu_time = pack.header.gcutime 
                if run_meta[current_run_id].stop_gcu_time < pack.header.gcutime:
                    run_meta[current_run_id].stop_gcu_time  = pack.header.gcutime 
                if run_meta[current_run_id].start_event_id > this_ev.tof.event_id :
                    run_meta[current_run_id].start_event_id = this_ev.tof.event_id
                if run_meta[current_run_id].stop_event_id < this_ev.tof.event_id:
                    run_meta[current_run_id].stop_event_id  = this_ev.tof.event_id
                if run_meta[current_run_id].start_gps_time > this_ev.tof.timestamp48:
                    run_meta[current_run_id].start_gps_time = this_ev.tof.timestamp48 
                if run_meta[current_run_id].stop_gps_time < this_ev.tof.timestamp48:
                    run_meta[current_run_id].stop_gps_time = this_ev.tof.timestamp48
                #break
    
    for r in run_dirs:
        if not r in run_meta:
            print (f'ERROR - can not find {r} in run_meta dict!')
            continue
        metadata_file = Path(f'{run_dirs[r]}/run{r}.meta.toml') 
        with open(metadata_file,'w') as meta_f:
            meta = run_meta[r] 
            meta.runtime_h      = (meta.stop_gcu_time - meta.start_gcu_time)/3600 
            meta.to_toml(meta_f)
        #if metadata_file.exists():
        #    prev_meta = RunMeta.load(metadata_file) 

        #    # modify it 
        #    if prev_meta.run_id != r:
        #        raise ValueError("Attempting to merge meta data for different runs!")
        #    if prev_meta.start_event_id < meta.start_event_id:
        #        meta.start_event_id = prev_meta.start_event_id 
        #    if prev_meta.stop_event_id > meta.stop_event_id:
        #        meta.stop_event_id  = prev_meta.stop_event_id 
        #    meta.n_events      += prev_meta.n_events
        #    meta.missing_evids += prev_meta.missing_evids
        #    if prev_meta.start_gcu_time < meta.start_gcu_time:
        #        meta.start_gcu_time = prev_meta.start_gcu_time 
        #    if prev_meta.stop_gcu_time > meta.stop_gcu_time:
        #        meta.stop_gcu_time = prev_meta.stop_gcu_time 
        #    if prev_meta.start_gps_time < meta.start_gps_time:
        #        meta.start_gps_time = prev_meta.start_gps_time 
        #    if prev_meta.stop_gps_time > meta.stop_gps_time:
        #        meta.stop_gps_time = prev_meta.stop_gps_time 
        #    meta.runtime_h += prev_meta.runtime_h 
        #    meta.avg_rate  += prev_meta.avg_rate 
        #    meta.avg_rate   = meta.avg_rate/2 # make sure it is still 
                                              # the average
    # write out the rundicts 


    #sys.exit(1) 

    #if args.reprocess:
    #    settings = go.liftof.LiftofSettings()
    #    settings = settings.from_file('settings.toml')
    run_id = 0
    # dealing with input files 
    #infile = args.infile[0] 
    #print(f'--> Will process {infile}')
    #print (args.telemetry_dir)
    #infiles          = sorted([k for k in args.telemetry_dir.glob('*.bin')])
    
    # get a telemetry reader 
    tel_reader = go.io.TelemetryPacketReader(args.telemetry_dir) 
    # in a first step, let's cache everything 
    moni_packets       = []
    merged_events      = {}
    tracker_daq_events = [] 

    # this is only for a single file - so just read all packets 
    packs = [pack for pack in tel_reader]
    # search for the toml packets 
    print(f'--> Searching run configuratoins ..')
    toml_packs   = [pack for pack in packs if pack.is_tof_toml_packet]
    run_settings = []
    for toml in toml_packs:
        toml_gcu       = toml.header.gcutime
        toml_timestamp = go.io.get_utc_timestamp_from_unix(toml_gcu)
        tp = go.packets.TofPacket.from_bytestream(toml.payload, 0)
        run_settings.append((toml_gcu,toml_timestamp,tp))
    print (f'--> Found {len(toml_packs)} run configuration settings!')
    n_packs_total = len(packs)
    print (f'--> Working with {n_packs_total:_} packets!')
    # kick out events without TofData since these don't have the gps 
    # or run id
    # FIXME - check how many events these are
    event_packets   = [k for k in packs if k.is_event_packet and k.header.packet_type != go.packets.TelemetryPacketType.NoTofDataEvent]
    tracker_packets = [k for k in packs if k.header.packet_type == go.packets.TelemetryPacketType.Tracker]

    # sort packets by time - if we go to more files than a single one, 
    # we might rethink that strategy
    # this is the reason we nned to load a bunch of them first
    event_packets = sorted(event_packets, key=lambda x : (x.get_runid(),x.get_gpstime_tof()))
    all_run_ids   = set([k.get_runid() for k in event_packets])
    print (f'--> Found the following runids {all_run_ids}')
    for k in all_run_ids:
        merged_events[k] = {}
    unpacking_errors              = 0
    associated_tracker_daq_events = 0
    total_tracker_daq_packets     = 0 
    total_tracker_daq_events      = 0 
    print('--> Sorting merged event packets!')
    for pack in tqdm.tqdm(event_packets):
        # merged events 
        try:
            ev = go.events.TelemetryEvent.from_telemetrypacket(pack) 
        except Exception as e:
            
            unpacking_errors += 1
            continue
        # instead of merging/deleting hits here,
        # we first aggregate the different event sources 
        #if args.remerge:
        #    ev.delete_all_tracker_hits() 
        # if there is a duplicate, we don't care, this will 
        # actually automatically eliminate them 
        merged_events[ev.tof.run_id][ev.event_id] = [ev]
    print(f'--> All merged events sorted by run!')
    print('================================================') 
    # tracker standalone (packet 80)
    print(f'--> Will try and associate {len(tracker_packets):_} (type 80) with the merged events. This is basically a re-merging') 
   
    if args.load_remainder is not None:
        print(f'--> Configured to load remainder (.bin) files from {args.load_remainder}') 
        prev_reader = go.io.TelemetryPacketReader(args.load_remainder) 
        print('--> We have a remainder with tracker packets from last time!')
        print('--> Loading packets...')
        prev_remainder = [k for k in tqdm.tqdm(prev_reader)] 
        print('--> done!')
        print(f'--> We found {len(prev_remainder)} packets!')
        tracker_packets += prev_remainder    
        print(f'--> UPDATED: Will try and associate {len(tracker_packets):_} (type 80) with the merged events. This is basically a re-merging') 
    
    tracker_unpack_errors = 0
    n_multiple_evids      = 0
    smallest_delta_time   = np.inf
    this_delta            = 0
    remainder             = []

    for pack in tqdm.tqdm(tracker_packets):
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
            for r in all_run_ids:
                if evid in merged_events[r].keys():
                    # the first packet is always the merged event
                    delta_gcutime = pack.header.gcutime - merged_events[r][evid][0].header.gcutime
                    delta_gcutime = abs(delta_gcutime)
                    candidates.append((delta_gcutime,r,ev)) 
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
                __,r,ev   = candidate
                # we put the event in the now empty TrackerPacket 
                empty_ev.add_event( ev.get_event_for_evid(evid))
                merged_events[r][evid].append(empty_ev)
                associated_tracker_daq_events += 1
                #ev.remove_event(evid) 
            # all events which have undergone any processing
            total_tracker_daq_events += 1

        # only packets with events still left count as 
        # "unassociable" for this processing and will 
        # be treated as a remainder
        if len(ev.events) > 0:
            remainder.append(ev.pack())
            #tracker_daq_events.append((ev.gcutime,ev.get_event_for_evid(evid)))
            #remaining -= len(candidates)
    print(f'--> We saw {tracker_unpack_errors} unpacking errors for tracker event packets!')
    print(f'--> We got {100*n_multiple_evids/len(tracker_packets):.3f}% multiple evids!')
    print(f'--> The smallest delta gcutime for multiple events was {smallest_delta_time}')
    print(f'--> There were {unpacking_errors} errors when unpacking merged events!')
    print(f'--> We saw {total_tracker_daq_packets} tracker daq events')
    print(f'--> We associated {associated_tracker_daq_events} tracker events with the merged events!')
    print(f'--> That is ~{float(associated_tracker_daq_events)/len(tracker_packets):.5f} events/packet')
    print(f'--> There are {len(remainder)} unsassociable events, that is {len(remainder)/(associated_tracker_daq_events + len(remainder)):.2f}%')
    print (f'--> There are {len(remainder)} ({100*len(remainder)/total_tracker_daq_events:.2f}%) tracker daq events which could not be added to a merged event!')
    #print (len(remainder), len(tracker_daq_events))
    #print (merged_events)
    print('=============================================') 
    print('--> Performing checks....!') 
    print('--> Check for missing tracker hits..!')
    events_with_missing_hits = 0
    for r in all_run_ids:
        for ev in merged_events[r].values():
            if not ev[0].has_at_least_expected_trk_hits:
               #print (ev)
               events_with_missing_hits += 1
               #raise
        print(f'--> The check resulted in seeing {events_with_missing_hits}/{len(merged_events[r])} ({100*events_with_missing_hits/len(merged_events[r]):.2f}%) of merged events for run {r} which do not have enough hits!')
    print('=============================================') 
    print('--> Writing data....!') 
    if args.outdir is None:
        cr_outdir =  Path('.')
    else:
        #cr_outdir = args.outdir / f'{run_id}'
        cr_outdir = args.outdir
    # make directories for runs and left-over tracker events 
    run_dirs = dict()
    for r in all_run_ids:
        run_dir = cr_outdir / f'{r}' 
        run_dir.mkdir(parents=True, exist_ok=True) 
        run_dirs[r] = run_dir
    
    if args.load_remainder is not None:
        print(f'--> (!!) Loaded previous remainder. Will delete it!') 
        shutil.rmtree(args.load_remainder) 
    
    # create new remainder!
    remainder_dir = cr_outdir / 'remainder' 
    remainder_dir.mkdir(parents=True, exist_ok=True) 
    
    # write the remaing tracker events
    remainder = sorted(remainder, key=lambda x : x.header.gcutime) 
    first_remaining_pack = remainder[0]
    remainder_writer = go.io.TelemetryPacketWriter(str(remainder_dir), first_remaining_pack)
    print(f'--> Writing remaining {len(remainder)} tracker events ..')
    for pack in remainder[1:]:
        remainder_writer.add_telemetry_packet(pack)
    del remainder_writer # trigger disk sync
    print('--> ... done!')
    #sys.exit() 
    print('=======================================')
    #for r in all_run_ids:
    #    print(f'--> Missing event id check for run {r}!')
    #    #print (min([k.event_id for k in remainder]), max([k.event_id for k in remainder]))
    #    
    #    # review the merged events and make them a list, checking for missing event ids:
    #    min_evid = min(merged_events[r].keys())
    #    max_evid = max(merged_events[r].keys())
    #    print (f'-> Scanning for missing evids in [{min_evid}, {max_evid}]')
    #    n_missing_evids = 0
    #    #writer             = go.io.CRWriter('new-processing', 0)
    #    n_processed_events = 0
    #    DELTA_SECONDS      = 60
    #    prev_gpstime       = merged_events[r][min_evid].tof.timestamp48/100e6 
    #    #        prev_gcutime   = first_gcutime 
    #    first_time     = go.io.get_utc_timestamp_from_unix(prev_gpstime)
    #    #print (first_time)
    #    run_id         = merged_events[r][min_evid].tof.run_id
    #    print (f'-> Will start next file! for {first_time}')
    #    writer         = go.io.CRWriter('new-processing', run_id, timestamp=first_time)
    #    n_files = 0
    #    # debugging only
    #    n_debug = 0
    #    timestamps = []
    #    event_ids  = []
     
    # write the merged events to disk again
    n_missing_evids    = 0 
    n_processed_events = 0
    n_files            = 0
    DELTA_SECONDS      = 60
    smallest_gcutime   = np.inf 
    largest_gcutime    = 0
    smallest_gpstime   = np.inf 
    largest_gpstime    = 0
    for r in all_run_ids:
        meta               = RunMeta()
        writer             = go.io.CRWriter(str(run_dir), r)
        n_processed_events = 0
        min_evid = min(merged_events[r].keys())
        max_evid = max(merged_events[r].keys())
        #print (f'-> Scanning for missing evids in [{min_evid}, {max_evid}]')
        prev_gpstime = merged_events[r][min_evid][0].tof.timestamp48/100e6 
        for evid in range(min_evid, max_evid+1): 
            frame = go.io.CRFrame() 
            try:
                event = merged_events[r][evid]
                n_processed_events += 1
            except KeyError as e:
                n_missing_evids += 1
                writer.add_frame(frame)
                continue
            # pack the merged event first 
            pack = event[0].pack() 
            name = f'TelemetryPacketType.{pack.header.packet_type}.SL'
            frame.put_telemetrypacket(pack, name=name)
            n_ev = 0
            for ev_to_pack in event[1:]:
                pack  = ev_to_pack.pack()
                # check if this is working
                #bs   = ev_to_pack.to_bytestream() 
                #foo_ = go.events.TrackerDAQEventPacket.from_bytestream(bs, 0)
                #try:
                #    foo_ = go.events.TrackerDAQEventPacket.from_telemetrypacket(pack)
                #except Exception as e:
                #    print (e)
                #    print (len(pack.payload), len(bs))
                name = f'TelemetryPacketType.{pack.header.packet_type}.{n_ev}.SL'
                frame.put_telemetrypacket(pack,name=name)
                newpack = frame.get_telemetrypacket(name)
                #print (pack) 
                #print (newpack)
                go.events.TrackerDAQEventPacket.from_telemetrypacket(newpack)
                n_ev += 1
            first_gpstime  = event[0].tof.timestamp48/100e6
            #first_gcutime = frame.get_first_gcutime()
            #print (first_gcutime, prev_gcutime)
            #timestamps.append(first_gpstime)
            #event_ids.append(event.tof.event_id)
            if first_gpstime - prev_gpstime < 0:
                prev_gpstime = first_gpstime
            # decide if we should start a new file
            if first_gpstime - prev_gpstime > DELTA_SECONDS:
                prev_gpstime = first_gpstime 
                first_time   = go.io.get_utc_timestamp_from_unix(first_gpstime)
                first_time   += 'UTC'
            #print (first_time)
                run_id         = event[0].tof.run_id
                print (f'-> Will start next file! for Run {run_id}, timestamp {first_time}')
                writer         = go.io.CRWriter(str(run_dirs[r]), run_id, timestamp=first_time)
                writer.add_frame(frame)
                n_files += 1
            else:
                writer.add_frame(frame)

            if first_gpstime < smallest_gpstime:
                smallest_gpstime = first_gpstime
            if first_gpstime > largest_gpstime:
                largest_gpstime = first_gpstime
                #if n_files > 24:
                #    print (f'{first_gpstime} {prev_gpstime} {first_gpstime - prev_pstime}')
                #if first_gpstime - prev_gpstime < -1:
                #    print (n_files)
                #    print (f'{first_gpstime} {prev_gpstime} {first_gpstime - prev_gpstime}')
                #    print (event.header)
                #    print (event.header.gcutime)
                #    print (first_time)
                #    print ('-----------------------')
                #    n_debug += 1 
                #if n_debug == 3:
                #    sys.exit()
        
        # Run metadata will provide a toml file with 
        # some statistics
        meta.run_id         = r
        meta.start_event_id = min_evid
        meta.stop_event_id  = max_evid 
        meta.n_events       = len(merged_events[r].keys())
        meta.missing_evids  = n_missing_evids 
        meta.start_gcu_time = merged_events[r][min_evid][0].header.gcutime
        meta.stop_gcu_time  = merged_events[r][min_evid][0].header.gcutime
        meta.start_gps_time = smallest_gpstime  
        meta.stop_gps_time  = largest_gpstime
        meta.runtime_h      = (largest_gpstime -smallest_gpstime)/3600 
        meta.avg_rate       = meta.n_events/(largest_gpstime - smallest_gpstime) 
        # in case file exists, read it first 
        metadata_file       = Path(f'{run_dirs[r]}/run{r}.meta.toml') 
        if metadata_file.exists():
            prev_meta = RunMeta.load(metadata_file) 

            # modify it 
            if prev_meta.run_id != r:
                raise ValueError("Attempting to merge meta data for different runs!")
            if prev_meta.start_event_id < meta.start_event_id:
                meta.start_event_id = prev_meta.start_event_id 
            if prev_meta.stop_event_id > meta.stop_event_id:
                meta.stop_event_id  = prev_meta.stop_event_id 
            meta.n_events      += prev_meta.n_events
            meta.missing_evids += prev_meta.missing_evids
            if prev_meta.start_gcu_time < meta.start_gcu_time:
                meta.start_gcu_time = prev_meta.start_gcu_time 
            if prev_meta.stop_gcu_time > meta.stop_gcu_time:
                meta.stop_gcu_time = prev_meta.stop_gcu_time 
            if prev_meta.start_gps_time < meta.start_gps_time:
                meta.start_gps_time = prev_meta.start_gps_time 
            if prev_meta.stop_gps_time > meta.stop_gps_time:
                meta.stop_gps_time = prev_meta.stop_gps_time 
            meta.runtime_h += prev_meta.runtime_h 
            meta.avg_rate  += prev_meta.avg_rate 
            meta.avg_rate   = meta.avg_rate/2 # make sure it is still 
                                              # the average
        with open(metadata_file,'w') as meta_f:
            meta.to_toml(meta_f)
        # check if we also have a toml file for this:
        for toml in run_settings:
            # the time in the toml-tuple is gcu time 
            print(f'-> Found toml {toml}!') 
            if abs(toml[0] - meta.start_gcu_time) < 5*60: # allow a discrepancy of 5 mins to run start
                tp = toml[2]
                go.io.decompress_toml(tp.payload, f'run{r}.{toml[1]}.toml')

        print(f'-> Found {n_missing_evids} ({n_missing_evids/n_processed_events:.2f}%) missing event ids in Run{r}!')
        #fig = plt.figure() 
        #ax  = fig.gca() 
        #ax.scatter(event_ids, timestamps)
        #fig.savefig(f'timestamps-evid-{r}.png')

    ## typically, the TOF data stream has less problems than the telemetry stream,
    ## and especially less dropped events
    ## so we will go by the TOF data stream
    #tof_reader = go.io.TofPacketReader(f'{args.tof_dir}/{args.run_id}')
    ## get the start/stop times from the tof_stream using GPS time. If that is not possible, 
    ## then we get them from the files
    #tof_times_reader = go.io.TofPacketReader(f'{args.tof_dir}/{args.run_id}', filter=go.packets.TofPacketType.TofEvent)
    #ev               = go.events.TofEvent()
    #if args.no_gps:
    #    #print ('-> Assume GPS is not connected, since --no-gps is given!')
    #    first_file       = tof_times_reader.filenames[0]
    #    last_file        = tof_times_reader.filenames[-1]
    #    tof_start_time   = go.io.get_unix_timestamp(go.io.get_rundata_from_file(first_file)['utctime'])
    #    tof_end_time     = go.io.get_unix_timestamp(go.io.get_rundata_from_file(last_file) ['utctime'])
    #    tof_duration     = tof_end_time - tof_start_time
    #    tof_times_reader.rewind()
    #    FIRST_TOF_EVENT  = None 
    #    print (f'-> Get start/stop times from TOF filenames')
    #    print (f'-> Adding +90s to the tof end file')
    #    tof_end_time += 90
    #    print (f'-> Found tof start/stop times of {tof_start_time:.1f}:{tof_end_time:.1f}, that is {tof_duration/3600:.2f} h!')
    #else:
    #    ev = go.events.TofEvent.from_tofpacket(tof_times_reader.first)
    #    tof_start_time   = 1e-5*ev.get_summary().timestamp48
    #    ev = go.events.TofEvent.from_tofpacket(tof_times_reader.last)
    #    FIRST_TOF_EVENT  = copy(ev)
    #    tof_end_time     = 1e-5*ev.get_summary().timestamp48
    #    tof_duration     = tof_end_time - tof_start_time
    #    print (f'-> Found tof start/stop times of {tof_start_time:.1f}:{tof_end_time:.1f}, that is {tof_duration/3600:.2f} h!')

    ## so here we do have start/stop times from the TOF from the tofstream data. We can overwrite them with 
    ## what has been given through the command line 
    #if args.start_time != -1: # default 
    #    tof_start_time = args.start_time 
    #if args.end_time   != -1: # default 
    #    tof_end_time   = args.stop_time

    #telemetry_files = go.io.grace_get_telemetry_binaries(tof_start_time,\
    #                                                     tof_end_time,\
    #                                                     data_dir=args.telemetry_dir)
    #telemetry_index = dict()
    #tof_index       = dict()
    #telly_errors    = 0

    ## check the telemetry stream for the run start time
    ## this is a bit tricky! The start is defined as 
    ## the first merged event after the run start.

    #start_found = False
    #first_telly_evid = -1
    #first_time  = -1
    #end_found   = False
    ## search for the start in the telemetry files 
    ##while not start_found:
    ##    if end_found: 
    ##        break 
    ##    # telemetry files are sorted, kick those out which are entirely 
    #n_skipped_files = 0
    #for f in telemetry_files:
    #    if not start_found:
    #        telly_reader = go.io.TelemetryPacketReader(str(f))
    #        #print (telly_reader.get_packet_index())
    #        for pack in telly_reader:
    #            #if pack.header.gcutime > tof_end_time:
    #            #    end_found = True
    #            #    break 
    #            if pack.header.gcutime < tof_start_time:
    #                continue
    #            else:
    #                # ln case this is monitoring information,
    #                # throw it away, so that we start with a 
    #                # merged event
    #                if not pack.packet_type in [go.packets.TelemetryPacketType.InterestingEvent,
    #                                            go.packets.TelemetryPacketType.BoringEvent,
    #                                            go.packets.TelemetryPacketType.NoTofDataEvent,
    #                                            go.packets.TelemetryPacketType.NoGapsTriggerEvent]:
    #                #if pack.packet_type != go.io.TelemetryPacketType.MergedEvent:
    #                    continue
    #                
    #                #ev = go.events.MergedEvent()
    #                try:
    #                    ev = go.events.TelemetryEvent.from_telemetrypacket(pack)
    #                    # trigger tof data unpacking
    #                    ev.tof
    #                except Exception as e:
    #                    print (f'-> While searching for the first event, we encountered an exception! {e}')
    #                    continue
    #                first_time = pack.header.gcutime
    #                first_telly_evid = ev.tof.event_id
    #                start_found = True
    #                FIRST_TELLY_EVENT = ev 
    #                FIRST_TELLY_READER = telly_reader
    #                break
    #        # if we reach here, the break statement did not trigger, so the start event was not 
    #        # in this file 
    #        n_skipped_files += 1
    #telemetry_files = telemetry_files[n_skipped_files:] 
    #assert len(telemetry_files) == len(set(telemetry_files) 

    #print(f'-> After cleaning for the run start we have {len(telemetry_files)} telemetry files')
    #print(f'-> The first event id {first_telly_evid} can be found at gcutime of {first_time}') 
    ##treader = go.io.TelemetryPacketReader(args.telemetry_dir, dedup = True, start_time = tof_start_time, end_time = tof_end_time) 
    ##print (len(treader.filenames))

    ##raise
    ##exit()
    ## fix the timestamp with the timestamp from the 
    ## tof file
    #current_filename = tof_reader.current_filename
    #file_timestamp   = go.io.get_rundata_from_file(current_filename)['utctime']
    ## now we have the telemetry and tof readers primed!
    #writer = go.io.CRWriter(cr_outdir, args.run_id, timestamp = file_timestamp)
 
    #telly_exhausted = False
    #telly_f_idx     = -1 # we start 1 before the filelist start
    #telly_errors    = 0
    #
    #toffy_exhausted = False
    #toffy_f_idx     = -1

    #tofevent_buffer_earlier = dict()
    #tofevent_buffer_later   = dict()
    #televent_buffer_earlier = dict()
    #televent_buffer_later   = dict()

    #frames_written = 0
    #print('-> Start merging!')
    #start_time = time.time()

    #first_event = True
    #
    #telly_f_idx  = 0
    ##telly_reader = go.io.TelemetryPacketReader(str(telemetry_files[telly_f_idx]))

    #n_telly_errors = 0
    #n_toffy_errors   = 0

    #done = False
    #for tofpack in tof_reader:
    #    if done:
    #        break
    #    # set the timestamp for the outputfile
    #    current_filename = tof_reader.current_filename
    #    #print (current_filename)
    #    #file_timestamp = get_timestamp(current_filename)
    #    file_timestamps = go.io.get_unix_timestamp(go.io.get_rundata_from_file(current_filename)['utctime'])
    #    writer.set_file_timestamp(file_timestamp)
    #    
    #    # in any case the L0 stream is that what is the 
    #    # tofstream
    #    frame = go.io.CRFrame()
    #    frame.put_tofpacket(tofpack, str(tofpack.packet_type))
    #    if frames_written % 10000 == 0 and not frames_written == 0: # or n_toffy_errors % 1000 == 0 or n_telly_errors % 1000 == 0:
    #        elapsed = (time.time() - start_time)/60
    #        print ('--------------------------------')
    #        #print (f'--> Read {telly_f_idx + 1} Telemetry files ({100*(telly_f_idx + 1)/len(telemetry_files):.2f}%), {read_tof_files} TOF files ({100*read_tof_files/len(tof_files):.2f})% in {elapsed:4.2f} minutes!')
    #        print (f'--> Read {telly_f_idx + 1} Telemetry files ({100*(telly_f_idx + 1)/len(telemetry_files):.2f}%) in {elapsed:4.2f} minutes!')
    #        print (f'--> Encountered {n_telly_errors} errors for TelemetryPackets, {n_toffy_errors} for TofPackets')
    #        print (f'--> Buffer size of telemetry events which are ahead of the TOF stream : {len(televent_buffer_earlier)}')
    #        print (f'--> Buffer size of telemetry events which are behind the   TOF stream : {len(televent_buffer_later)}')
    #        print (f'--> {frames_written} frames written!')
    #        print (f'--> {frame}')

    #    if tofpack.packet_type not in [go.packets.TofPacketType.TofEvent,go.packets.TofPacketType.TofEventDeprecated]:
    #        # tof hk in its own frames
    #        #go.packets.TofPacketTypeDeprecated
    #        #print (tofpack)
    #        #raise
    #        writer.add_frame(frame)
    #        frames_written += 1
    #        continue 
    #    else:
    #        #tofev   = go.events.TofEvent()
    #        try:
    #            #FIXME - think about making the constructor taking 
    #            # a tof packet/CRFrame and mark the necessary 
    #            # from_tofpacket as _from_tofpacket
    #            tofev = go.events.TofEvent.from_tofpacket(tofpack)
    #        except:
    #            n_toffy_errors += 1
    #            continue
    #        tofevid = tofev.event_id
    #        # we check if we have anything in the caches
    #        if tofevid in televent_buffer_earlier.keys():
    #            tp = televent_buffer_earlier.pop(tofevid)
    #            frame.put_telemetrypacket(tp, str(tp.packet_type))
    #            writer.add_frame(frame)
    #            frames_written += 1 
    #            continue;
    #        if tofevid in televent_buffer_later.keys():
    #            tp = televent_buffer_later.pop(tofevid)
    #            frame.put_telemetrypacket(tp, str(tp.packet_type))
    #            writer.add_frame(frame)
    #            frames_written += 1 
    #            continue;
    #            
    #        found = False
    #        while not found: # walk through the telemetry files until we find our event
    #            #print (frames_written, 'brah!')
    #            telly_exhausted = True
    #            #print (telly_reader)
    #            #exit()
    #            #n = 0
    #            for telpack in telly_reader:
    #                if telpack.header.gcutime < tof_start_time: 
    #                    continue
    #                telly_exhausted = False
    #                #n += 1
    #                #if n % 10000 == 0:
    #                #    print (n)
    #                #continue
    #                #break
    #                #exit()
    #                #continue
    #                #print (telpack)
    #                #print (telpack.packet_type)
    #                #print (telpack.header.counter)
    #                #print (telpack.header.checksum) 
    #                #continue
    #                if not telpack.packet_type in [go.packets.TelemetryPacketType.InterestingEvent,
    #                                               go.packets.TelemetryPacketType.BoringEvent,
    #                                               go.packets.TelemetryPacketType.NoTofDataEvent,
    #                                               go.packets.TelemetryPacketType.NoGapsTriggerEvent]:
    #                    # we add the housekeeping to the same frame
    #                    if telpack.packet_type == go.packets.TelemetryPacketType.Tracker:
    #                        continue # throw away tracker packets
    #                    success = False 
    #                    ntries  = 1
    #                    name    = str(telpack.packet_type)
    #                    nloop = 0
    #                    while not success:
    #                        nloop += 1
    #                        try:
    #                            #FIXME - speed this up by including an 
    #                            # auto-increment feature in put_telemetrypacket
    #                            frame.put_telemetrypacket(telpack, name)
    #                            success = True
    #                            break
    #                        except Exception as e:
    #                            #print (e) 
    #                            ntries += 1
    #                            name = str(telpack.packet_type) + f'_{ntries}' 
    #                            #print (frame)
    #                            continue
    #                    print (f'-> HK! {telpack.packet_type}, loop ran {nloop} times!')
    #                    print (frame)
    #                    continue
    #                else:
    #                    #ev = go.events.MergedEvent()
    #                    print ('found event')
    #                    break
    #                    try:
    #                        ev = go.events.TelemetryEvent.from_telemetrypacket(telpack)
    #                        televid = ev.tof.event_id
    #                    except:
    #                        n_telly_errors += 1
    #                        continue
    #                    #print (televid, tofevid)
    #                    if televid < tofevid:
    #                        televent_buffer_earlier[televid]   = telpack
    #                        continue
    #                    elif televid > tofevid:
    #                        televent_buffer_later[televid]   = telpack
    #                        # we only write the tofpacket and move on 
    #                        #frame.put_tofpacket(tofpack, str(tofpack.packet_type))
    #                        writer.add_frame(frame)
    #                        frames_written += 1
    #                        found = True # problem in telemetry stream, skipped event
    #                        break # break loop over telemetry packets
    #                    else:
    #                        # we are golden
    #                        frame.put_telemetrypacket(telpack, str(telpack.packet_type))
    #                        writer.add_frame(frame)
    #                        frames_written += 1
    #                        found = True
    #                        break
    #            if telly_exhausted: 
    #                telly_f_idx += 1
    #                if telly_f_idx == len(telemetry_files):
    #                    print ('-> We reached the last of the telemetry files!')
    #                    done = True
    #                    #sys.exit(0)
    #                    break

    #                telly_reader = go.io.TelemetryPacketReader(str(telemetry_files[telly_f_idx]))
    #                print (f'-> Primed new TelemetryPacketReader for file {telemetry_files[telly_f_idx]}')
    #                continue
    #
    #print (f'-> ===========================================================================')
    #print (f'-> Emptying early buffer')

    #def sort_by_subrun(f):
    #    return int(f.split('_')[1].split('.')[0])

    #inputfiles = sorted(glob(f'{cr_outdir}/*.gaps'), key=sort_by_subrun) 

    ## create a new directory, "clean" within the caraspace directory
    ## then have a reader/writer combo, for each file read the whole file and write it back to 
    ## the clean directory, adding the events from the "earlier" buffer
    #cr_outdir = Path(cr_outdir) / 'clean'
    #cr_outdir.mkdir(parents=True, exist_ok=True)
    #cr_outdir = str(cr_outdir)
    #
    #file_timestamp   = get_timestamp(inputfiles[0])
    #writer           = go.io.CRWriter(cr_outdir, args.run_id, timestamp = file_timestamp)
    #frames_written = 0
    #for f in inputfiles:
    #    reader = go.io.CRReader(str(f))
    #    file_timestamp   = get_timestamp(f)
    #    writer.set_file_timestamp(file_timestamp)
    #    for frame in reader:
    #        clean_frame = frame
    #        if frames_written % 10000 == 0: # or n_toffy_errors % 1000 == 0 or n_telly_errors % 1000 == 0:
    #            elapsed = (time.time() - start_time)/60
    #            print ('--------------------------------')
    #            #print (f'--> Read {telly_f_idx + 1} Telemetry files ({100*(telly_f_idx + 1)/len(telemetry_files):.2f}%), {read_tof_files} TOF files ({100*read_tof_files/len(tof_files):.2f})% in {elapsed:4.2f} minutes!')
    #            print (f'--> Buffer size of telemetry events which are ahead of the TOF stream : {len(televent_buffer_earlier)}')
    #            print (f'--> Buffer size of telemetry events which are behind the   TOF stream : {len(televent_buffer_later)}')
    #            print (f'--> {frames_written} frames written!')
    #            print (f'--> {frame}')
    #        if 'PacketType.TofEvent' in frame.index.keys():
    #            pack = frame.get_tofpacket('PacketType.TofEvent')
    #            ev   = go.events.TofEvent()
    #            ev.from_tofpacket(pack)
    #            evid = ev.event_id
    #            if evid in televent_buffer_earlier.keys():
    #                telpack = televent_buffer_earlier[evid]
    #                clean_frame.put_telemetrypacket(telpack, str(telpack.packet_type))
    #            if evid in televent_buffer_later.keys():
    #                telpack = televent_buffer_later[evid]
    #                clean_frame.put_telemetrypacket(telpack, str(telpack.packet_type))
    #        writer.add_frame(clean_frame)
    #        frames_written += 1
    #    os.remove(f)
    
