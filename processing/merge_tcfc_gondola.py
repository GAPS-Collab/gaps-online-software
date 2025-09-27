#! /usr/bin/env python 

import os
import tqdm
import gondola as go
import time
import re
from pathlib import Path
from glob import glob
from copy import deepcopy

if __name__ == '__main__':

    import argparse
    import sys

    parser = argparse.ArgumentParser(description='Scrutinize run data for issues. Input can be either telemetry binary files, TOF run files (.tof.gaps) or both')
    parser.add_argument('--telemetry-dir', default='/data0/gaps/csbf/csbf-data/binaries/ethernet',\
                        help='A directory with telemetry binaries, as received from the telemetry stream',
                        )
    parser.add_argument('-n','--npackets', type=int,\
                        default=-1, help='Limit readout to npackets, -1 for all packets (default)')
    parser.add_argument('--n-tof-files', type=int,\
                        default=-1, help='Limit the readout to number of tif files, -1 for all files (default)')
    parser.add_argument('--tof-dir', default='', type=Path,\
                        help='A directory with tof data files (.tof.gaps)',)
    parser.add_argument('-s','--start-time',\
                        type=int, default=-1,\
                        help='The run start time, e.g. as taken from the elog')
    parser.add_argument('-e','--end-time',
                        type=int, default=-1,\
                        help='The run end time, e.g. as taken from the elog')
    parser.add_argument('-r','--run-id', default=-1, type=int,\
                        help='TOF Run id (only relevant when working with TOF files')
    parser.add_argument('-o','--outdir',\
                        help='Outdir for caraspace output files',
                        type=Path,
                        default=None)
    parser.add_argument('-v','--verbose', action='store_true',\
                        help='More verbose output')
    parser.add_argument('--no-gps', action='store_true', \
                        help='Ignore the GPS to find matching telemetry and tof timestamps. Only use the tof file timestamps')
    parser.add_argument('--reprocess', action='store_true', \
                        help='Recalculate tof packets with latest version of the code')
    args = parser.parse_args()
  

    if args.reprocess:
        settings = go.liftof.LiftofSettings()
        settings = settings.from_file('settings.toml')

    if args.outdir is None:
        cr_outdir = args.tof_dir / 'caraspace' / f'{args.run_id}'
    else:
        cr_outdir = args.outdir / f'{args.run_id}'
    cr_outdir.mkdir(parents=True, exist_ok=True)
    cr_outdir = str(cr_outdir)

    # typically, the TOF data stream has less problems than the telemetry stream,
    # and especially less dropped events
    # so we will go by the TOF data stream
    tof_reader = go.io.TofPacketReader(f'{args.tof_dir}/{args.run_id}')
    # get the start/stop times from the tof_stream using GPS time. If that is not possible, 
    # then we get them from the files
    tof_times_reader = go.io.TofPacketReader(f'{args.tof_dir}/{args.run_id}', filter=go.packets.TofPacketType.TofEvent)
    ev               = go.events.TofEvent()
    if args.no_gps:
        #print ('-> Assume GPS is not connected, since --no-gps is given!')
        first_file       = tof_times_reader.filenames[0]
        last_file        = tof_times_reader.filenames[-1]
        tof_start_time   = go.io.get_unix_timestamp(go.io.get_rundata_from_file(first_file)['utctime'])
        tof_end_time     = go.io.get_unix_timestamp(go.io.get_rundata_from_file(last_file) ['utctime'])
        tof_duration     = tof_end_time - tof_start_time
        tof_times_reader.rewind()
        FIRST_TOF_EVENT  = None 
        print (f'-> Get start/stop times from TOF filenames')
        print (f'-> Adding +90s to the tof end file')
        tof_end_time += 90
        print (f'-> Found tof start/stop times of {tof_start_time:.1f}:{tof_end_time:.1f}, that is {tof_duration/3600:.2f} h!')
    else:
        ev = go.events.TofEvent.from_tofpacket(tof_times_reader.first)
        tof_start_time   = 1e-5*ev.get_summary().timestamp48
        ev = go.events.TofEvent.from_tofpacket(tof_times_reader.last)
        FIRST_TOF_EVENT  = copy(ev)
        tof_end_time     = 1e-5*ev.get_summary().timestamp48
        tof_duration     = tof_end_time - tof_start_time
        print (f'-> Found tof start/stop times of {tof_start_time:.1f}:{tof_end_time:.1f}, that is {tof_duration/3600:.2f} h!')

    # so here we do have start/stop times from the TOF from the tofstream data. We can overwrite them with 
    # what has been given through the command line 
    if args.start_time != -1: # default 
        tof_start_time = args.start_time 
    if args.end_time   != -1: # default 
        tof_end_time   = args.stop_time

    telemetry_files = go.io.grace_get_telemetry_binaries(tof_start_time,\
                                                         tof_end_time,\
                                                         data_dir=args.telemetry_dir)
    telemetry_index = dict()
    tof_index       = dict()
    telly_errors    = 0

    # check the telemetry stream for the run start time
    # this is a bit tricky! The start is defined as 
    # the first merged event after the run start.

    start_found = False
    first_telly_evid = -1
    first_time  = -1
    end_found   = False
    # search for the start in the telemetry files 
    #while not start_found:
    #    if end_found: 
    #        break 
    #    # telemetry files are sorted, kick those out which are entirely 
    n_skipped_files = 0
    for f in telemetry_files:
        if not start_found:
            telly_reader = go.io.TelemetryPacketReader(str(f))
            #print (telly_reader.get_packet_index())
            for pack in telly_reader:
                #if pack.header.gcutime > tof_end_time:
                #    end_found = True
                #    break 
                if pack.header.gcutime < tof_start_time:
                    continue
                else:
                    # ln case this is monitoring information,
                    # throw it away, so that we start with a 
                    # merged event
                    if not pack.packet_type in [go.packets.TelemetryPacketType.InterestingEvent,
                                                go.packets.TelemetryPacketType.BoringEvent,
                                                go.packets.TelemetryPacketType.NoTofDataEvent,
                                                go.packets.TelemetryPacketType.NoGapsTriggerEvent]:
                    #if pack.packet_type != go.io.TelemetryPacketType.MergedEvent:
                        continue
                    
                    #ev = go.events.MergedEvent()
                    try:
                        ev = go.events.TelemetryEvent.from_telemetrypacket(pack)
                        # trigger tof data unpacking
                        ev.tof
                    except Exception as e:
                        print (f'-> While searching for the first event, we encountered an exception! {e}')
                        continue
                    first_time = pack.header.gcutime
                    first_telly_evid = ev.tof.event_id
                    start_found = True
                    FIRST_TELLY_EVENT = ev 
                    FIRST_TELLY_READER = telly_reader
                    break
            # if we reach here, the break statement did not trigger, so the start event was not 
            # in this file 
            n_skipped_files += 1
    telemetry_files = telemetry_files[n_skipped_files:] 
    assert len(telemetry_files) == len(set(telemetry_files) 

    print(f'-> After cleaning for the run start we have {len(telemetry_files)} telemetry files')
    print(f'-> The first event id {first_telly_evid} can be found at gcutime of {first_time}') 
    #treader = go.io.TelemetryPacketReader(args.telemetry_dir, dedup = True, start_time = tof_start_time, end_time = tof_end_time) 
    #print (len(treader.filenames))

    #raise
    #exit()
    # fix the timestamp with the timestamp from the 
    # tof file
    current_filename = tof_reader.current_filename
    file_timestamp   = go.io.get_rundata_from_file(current_filename)['utctime']
    # now we have the telemetry and tof readers primed!
    writer = go.io.CRWriter(cr_outdir, args.run_id, timestamp = file_timestamp)
 
    telly_exhausted = False
    telly_f_idx     = -1 # we start 1 before the filelist start
    telly_errors    = 0
    
    toffy_exhausted = False
    toffy_f_idx     = -1

    tofevent_buffer_earlier = dict()
    tofevent_buffer_later   = dict()
    televent_buffer_earlier = dict()
    televent_buffer_later   = dict()

    frames_written = 0
    print('-> Start merging!')
    start_time = time.time()

    first_event = True
    
    telly_f_idx  = 0
    #telly_reader = go.io.TelemetryPacketReader(str(telemetry_files[telly_f_idx]))

    n_telly_errors = 0
    n_toffy_errors   = 0

    done = False
    for tofpack in tof_reader:
        if done:
            break
        # set the timestamp for the outputfile
        current_filename = tof_reader.current_filename
        #print (current_filename)
        #file_timestamp = get_timestamp(current_filename)
        file_timestamps = go.io.get_unix_timestamp(go.io.get_rundata_from_file(current_filename)['utctime'])
        writer.set_file_timestamp(file_timestamp)
        
        # in any case the L0 stream is that what is the 
        # tofstream
        frame = go.io.CRFrame()
        frame.put_tofpacket(tofpack, str(tofpack.packet_type))
        if frames_written % 10000 == 0 and not frames_written == 0: # or n_toffy_errors % 1000 == 0 or n_telly_errors % 1000 == 0:
            elapsed = (time.time() - start_time)/60
            print ('--------------------------------')
            #print (f'--> Read {telly_f_idx + 1} Telemetry files ({100*(telly_f_idx + 1)/len(telemetry_files):.2f}%), {read_tof_files} TOF files ({100*read_tof_files/len(tof_files):.2f})% in {elapsed:4.2f} minutes!')
            print (f'--> Read {telly_f_idx + 1} Telemetry files ({100*(telly_f_idx + 1)/len(telemetry_files):.2f}%) in {elapsed:4.2f} minutes!')
            print (f'--> Encountered {n_telly_errors} errors for TelemetryPackets, {n_toffy_errors} for TofPackets')
            print (f'--> Buffer size of telemetry events which are ahead of the TOF stream : {len(televent_buffer_earlier)}')
            print (f'--> Buffer size of telemetry events which are behind the   TOF stream : {len(televent_buffer_later)}')
            print (f'--> {frames_written} frames written!')
            print (f'--> {frame}')

        if tofpack.packet_type not in [go.packets.TofPacketType.TofEvent,go.packets.TofPacketType.TofEventDeprecated]:
            # tof hk in its own frames
            #go.packets.TofPacketTypeDeprecated
            #print (tofpack)
            #raise
            writer.add_frame(frame)
            frames_written += 1
            continue 
        else:
            #tofev   = go.events.TofEvent()
            try:
                #FIXME - think about making the constructor taking 
                # a tof packet/CRFrame and mark the necessary 
                # from_tofpacket as _from_tofpacket
                tofev = go.events.TofEvent.from_tofpacket(tofpack)
            except:
                n_toffy_errors += 1
                continue
            tofevid = tofev.event_id
            # we check if we have anything in the caches
            if tofevid in televent_buffer_earlier.keys():
                tp = televent_buffer_earlier.pop(tofevid)
                frame.put_telemetrypacket(tp, str(tp.packet_type))
                writer.add_frame(frame)
                frames_written += 1 
                continue;
            if tofevid in televent_buffer_later.keys():
                tp = televent_buffer_later.pop(tofevid)
                frame.put_telemetrypacket(tp, str(tp.packet_type))
                writer.add_frame(frame)
                frames_written += 1 
                continue;
                
            found = False
            while not found: # walk through the telemetry files until we find our event
                #print (frames_written, 'brah!')
                telly_exhausted = True
                #print (telly_reader)
                #exit()
                #n = 0
                for telpack in telly_reader:
                    if telpack.header.gcutime < tof_start_time: 
                        continue
                    telly_exhausted = False
                    #n += 1
                    #if n % 10000 == 0:
                    #    print (n)
                    #continue
                    #break
                    #exit()
                    #continue
                    #print (telpack)
                    #print (telpack.packet_type)
                    #print (telpack.header.counter)
                    #print (telpack.header.checksum) 
                    #continue
                    if not telpack.packet_type in [go.packets.TelemetryPacketType.InterestingEvent,
                                                   go.packets.TelemetryPacketType.BoringEvent,
                                                   go.packets.TelemetryPacketType.NoTofDataEvent,
                                                   go.packets.TelemetryPacketType.NoGapsTriggerEvent]:
                        # we add the housekeeping to the same frame
                        if telpack.packet_type == go.packets.TelemetryPacketType.Tracker:
                            continue # throw away tracker packets
                        success = False 
                        ntries  = 1
                        name    = str(telpack.packet_type)
                        nloop = 0
                        while not success:
                            nloop += 1
                            try:
                                #FIXME - speed this up by including an 
                                # auto-increment feature in put_telemetrypacket
                                frame.put_telemetrypacket(telpack, name)
                                success = True
                                break
                            except Exception as e:
                                #print (e) 
                                ntries += 1
                                name = str(telpack.packet_type) + f'_{ntries}' 
                                #print (frame)
                                continue
                        print (f'-> HK! {telpack.packet_type}, loop ran {nloop} times!')
                        print (frame)
                        continue
                    else:
                        #ev = go.events.MergedEvent()
                        print ('found event')
                        break
                        try:
                            ev = go.events.TelemetryEvent.from_telemetrypacket(telpack)
                            televid = ev.tof.event_id
                        except:
                            n_telly_errors += 1
                            continue
                        #print (televid, tofevid)
                        if televid < tofevid:
                            televent_buffer_earlier[televid]   = telpack
                            continue
                        elif televid > tofevid:
                            televent_buffer_later[televid]   = telpack
                            # we only write the tofpacket and move on 
                            #frame.put_tofpacket(tofpack, str(tofpack.packet_type))
                            writer.add_frame(frame)
                            frames_written += 1
                            found = True # problem in telemetry stream, skipped event
                            break # break loop over telemetry packets
                        else:
                            # we are golden
                            frame.put_telemetrypacket(telpack, str(telpack.packet_type))
                            writer.add_frame(frame)
                            frames_written += 1
                            found = True
                            break
                if telly_exhausted: 
                    telly_f_idx += 1
                    if telly_f_idx == len(telemetry_files):
                        print ('-> We reached the last of the telemetry files!')
                        done = True
                        #sys.exit(0)
                        break

                    telly_reader = go.io.TelemetryPacketReader(str(telemetry_files[telly_f_idx]))
                    print (f'-> Primed new TelemetryPacketReader for file {telemetry_files[telly_f_idx]}')
                    continue
    
    print (f'-> ===========================================================================')
    print (f'-> Emptying early buffer')

    def sort_by_subrun(f):
        return int(f.split('_')[1].split('.')[0])

    inputfiles = sorted(glob(f'{cr_outdir}/*.gaps'), key=sort_by_subrun) 

    # create a new directory, "clean" within the caraspace directory
    # then have a reader/writer combo, for each file read the whole file and write it back to 
    # the clean directory, adding the events from the "earlier" buffer
    cr_outdir = Path(cr_outdir) / 'clean'
    cr_outdir.mkdir(parents=True, exist_ok=True)
    cr_outdir = str(cr_outdir)
    
    file_timestamp   = get_timestamp(inputfiles[0])
    writer           = go.io.CRWriter(cr_outdir, args.run_id, timestamp = file_timestamp)
    frames_written = 0
    for f in inputfiles:
        reader = go.io.CRReader(str(f))
        file_timestamp   = get_timestamp(f)
        writer.set_file_timestamp(file_timestamp)
        for frame in reader:
            clean_frame = frame
            if frames_written % 10000 == 0: # or n_toffy_errors % 1000 == 0 or n_telly_errors % 1000 == 0:
                elapsed = (time.time() - start_time)/60
                print ('--------------------------------')
                #print (f'--> Read {telly_f_idx + 1} Telemetry files ({100*(telly_f_idx + 1)/len(telemetry_files):.2f}%), {read_tof_files} TOF files ({100*read_tof_files/len(tof_files):.2f})% in {elapsed:4.2f} minutes!')
                print (f'--> Buffer size of telemetry events which are ahead of the TOF stream : {len(televent_buffer_earlier)}')
                print (f'--> Buffer size of telemetry events which are behind the   TOF stream : {len(televent_buffer_later)}')
                print (f'--> {frames_written} frames written!')
                print (f'--> {frame}')
            if 'PacketType.TofEvent' in frame.index.keys():
                pack = frame.get_tofpacket('PacketType.TofEvent')
                ev   = go.events.TofEvent()
                ev.from_tofpacket(pack)
                evid = ev.event_id
                if evid in televent_buffer_earlier.keys():
                    telpack = televent_buffer_earlier[evid]
                    clean_frame.put_telemetrypacket(telpack, str(telpack.packet_type))
                if evid in televent_buffer_later.keys():
                    telpack = televent_buffer_later[evid]
                    clean_frame.put_telemetrypacket(telpack, str(telpack.packet_type))
            writer.add_frame(clean_frame)
            frames_written += 1
        os.remove(f)
    
