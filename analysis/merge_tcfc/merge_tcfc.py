from glob import glob


import tqdm
import gaps_online as go
# will go away
import time

from collections import deque

def align_evids(evids_telly, evids_toffy, truncate = False):
    print (f'-> Got {len(evids_telly)} event ids for telemetry stream!')
    print (f'-> Got {len(evids_toffy)} event ids for TOF stream!')
    go.run.check_missing_events(evids_telly)
    go.run.check_missing_events(evids_toffy)
    min_telly = min(sorted(evids_telly))
    min_toffy = min(sorted(evids_toffy))
    max_telly = max(sorted(evids_telly))
    max_toffy = max(sorted(evids_toffy))
    print (f'-> Found event ids in range {min_telly} .. {max_telly} for telemetry stream')
    print (f'-> Found event ids in range {min_toffy} .. {max_toffy} for TOF stream')
    if min_telly < min_toffy:
        #get rid of the first events
        if truncate:
            new_evids_telly = []
            for k in evids_telly:
                if k >= min_toffy:
                    new_evids_telly.append(k)
            evids_telly = new_evids_telly
            print (f'-> Truncated first set of evids to {len(evids_telly)}')
    elif min_toffy < min_telly:
        if truncate:
            new_evids_toffy = []
            for k in evids_toffy:
                if k >= min_telly:
                    new_evids_toffy.append(k)
            evids_toffy = new_evids_toffy
            print (f'-> Truncated second set of evids to {len(evids_toffy)}')
    else: # events start is aligned
        pass
    return evids_telly, evids_toffy

if __name__ == '__main__':

    import argparse
    import sys

    # pretty plots
    import charmingbeauty as cb
    cb.visual.set_style_default()

    from pathlib import Path

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
    #parser.add_argument('-o','--outdir',\
    #                    help='Outdir to save plots',
    #                    default='')
    parser.add_argument('-v','--verbose', action='store_true',\
                        help='More verbose output')
    parser.add_argument('--reprocess', action='store_true', \
                        help='Recalculate tof packets with latest version of the code')
    args = parser.parse_args()
   
    if args.reprocess:
        settings = go.liftof.LiftofSettings()
        settings = settings.from_file('settings.toml')

    cr_outdir = str(args.tof_dir / 'caraspace')

    telemetry_files = go.io.get_telemetry_binaries(args.start_time, args.end_time,\
                                                   data_dir=args.telemetry_dir)
    tof_files = go.io.get_tof_binaries(args.run_id, data_dir=args.tof_dir)
    if args.n_tof_files != -1:
        tof_files = tof_files[:args.n_tof_files]
    
    telemetry_index = dict()
    tof_index       = dict()
    evids_telly     = deque()
    evids_toffy     = deque()
    telly_errors    = 0

    # check the telemetry stream for the run start time
    start_found = False
    clean_files = []
    first_telly_evid = -1
    first_time  = -1
    while not start_found:
        # telemetry files are sorted, kick those out which are entirely 
        for f in telemetry_files:
            if not start_found:
                telly_reader = go.io.TelemetryPacketReader(str(f))
                #print (telly_reader.get_packet_index())
                for pack in telly_reader:
                    if pack.header.gcutime < args.start_time:
                        continue
                    else:
                        # ln case this is monitoring information,
                        # throw it away, so that we start with a 
                        # merged event
                        if pack.packet_type != go.io.TelemetryPacketType.MergedEvent:
                            continue
                        
                        ev = go.events.MergedEvent()
                        try:
                            ev.from_telemetrypacket(pack)
                            # trigger tof data unpacking
                            ev.tof
                        except Exception as e:
                            print (f'-> While searching for the first event, we encountered an exception! {e}')
                            continue
                        first_time = pack.header.gcutime
                        first_telly_evid = ev.tof.event_id
                        start_found = True
                        break
                    #if pack.packet_type == go.io.TelemetryPacketType.MergedEvent:
                    #    
                    #    ev = go.events.MergedEvent()
                    #    ev.from_telemetrypacket(pack)
                    #    try:
                    #        evid = ev.tof.event_id
                    #        evids_telly.append(evid)
                    #    except Exception as e:
                    #        print (e)
                    #        telly_errors += 1
                    #        continue
            if start_found:
                clean_files.append(f)
    telemetry_files = clean_files[1:] # re-use the already primed telly_reader, which 
                                      # is laoded with the first file already
    print(f'-> After cleaning we start have {len(telemetry_files)} telemetry files')
    print(f'-> The first event id {first_telly_evid} can be found at gcutime of {first_time}') 
    # the telly_reader is now at the correct position

    # now prime the tof reader
    start_found = False
    clean_files = []
    first_toffy_evid = []
    while not start_found:
        # telemetry files are sorted, kick those out which are entirely 
        for f in tof_files:
            if not start_found:
                toffy_reader = go.io.TofPacketReader(str(f))
                #print (telly_reader.get_packet_index())
                for pack in toffy_reader:
                    if pack.packet_type == go.io.TofPacketType.TofEvent:
                        ev = go.events.TofEvent()
                        try:
                            # FIXME - we only need the event id
                            # write partial unpack function?
                            ev.from_tofpacket(pack)
                        except Exception as e:
                            print (f'-> While searching for the first event, we encountered an exception! {e}')
                            continue
                        if ev.mastertriggerevent.event_id < first_telly_evid:
                            continue
                        else:
                            first_toffy_evid = ev.mastertriggerevent.event_id
                            start_found = True
                            break
                    else:
                        continue # throw away monitoring pre-run
            if start_found:
                clean_files.append(f)
    
    tof_files = clean_files[1:] # re-use the already primed telly_reader, which 
                                # is laoded with the first file already
    print(f'-> After cleaning we start have {len(tof_files)} tof files')
  
    # now we have the telemetry and tof readers primed!
    writer = go.io.CRWriter(cr_outdir, args.run_id)
    
    telly_exhausted = False
    telly_f_idx     = -1 # we start 1 before the filelist start
    telly_errors    = 0
    
    toffy_exhausted = False
    toffy_f_idx     = -1
    toffy_errors   = 0

    tofevent_buffer_earlier = dict()
    tofevent_buffer_later   = dict()

    frames_written = 0
    print('-> Start merging!')
    start_time = time.time()
    while telly_f_idx != len(telemetry_files) - 1: 
        if telly_exhausted:
            telly_exhausted = False
            telly_f_idx += 1
            f = telemetry_files[telly_f_idx]
            telly_reader = go.io.TelemetryPacketReader(str(f))
        # we will align the combined data by the telemetry stream
        for pack in telly_reader:
            # we create a single frame for each packet
            frame = go.io.CRFrame()
            frame.put_telemetrypacket(pack, str(pack.packet_type))
            if pack.packet_type != go.io.TelemetryPacketType.MergedEvent:
                # nothing to merge. Add to the joint file and move on
                writer.add_frame(frame)
                frames_written += 1
                continue
            else:
                telly_ev = go.events.MergedEvent()
                try:
                    telly_ev.from_telemetrypacket(pack)
                    telly_ev.tof # can trigger error
                except Exception as e:
                    telly_errors += 1
                    print (f'-> When reading from the telemetry stream, we encountered an exception! {e}')
                    continue 
                # check if there is an event in the tof_event_later buffer
                if telly_ev.tof.event_id in tofevent_buffer_later:
                    tpack = tofevent_buffer_later.pop(telly_ev.tof.event_id)
                    frame.put_tofpacket(tofpack, str(tpack.packet_type))
                    if args.reprocess:
                        new_tofev = go.events.TofEvent()
                        new_tofev.from_tofpacket(tpack)
                        new_tofev = go.liftof.waveform_analysis(new_tofev, settings)
                        new_tofpack = new_tofev.pack()
                        frame.put_tofpacket(new_tofpack, str('TofEvent.reprocessed'))
                    writer.add_frame(frame)
                    frames_written += 1
                    continue # get next telemetrypacket
                
                # check if there is an event in the tof_event_earlier buffer
                if telly_ev.tof.event_id in tofevent_buffer_earlier:
                    tpack = tofevent_buffer_earlier.pop(telly_ev.tof.event_id)
                    frame.put_tofpacket(tofpack, str(tpack.packet_type))
                    if args.reprocess:
                        new_tofev = go.events.TofEvent()
                        new_tofev.from_tofpacket(tpack)
                        new_tofev = go.liftof.waveform_analysis(new_tofev, settings)
                        new_tofpack = new_tofev.pack()
                        frame.put_tofpacket(new_tofpack, str('TofEvent.reprocessed'))
                    writer.add_frame(frame)
                    frames_written += 1
                    continue # get next telemetrypacket


                if toffy_exhausted:
                    toffy_exhausted = False
                    toffy_f_idx += 1
                    f = tof_files[toffy_f_idx]
                    toffy_reader = go.io.TofPacketReader(str(f))
                
                # this gets only executed if the toffy_reader is exhausted
                toffy_exhausted = True
                for tofpack in toffy_reader:
                    toffy_exhausted = False
                    # if there is monitoring data, we just add it to the same 
                    # frame
                    # FIXME - this needs a different name for each individual 
                    # RBs
                    if tofpack.packet_type != go.io.TofPacketType.TofEvent:
                        frame.put_tofpacket(tofpack, str(tofpack.packet_type))
                        # don't add the frame yet, we will add all the monitoring data 
                        # into the same frame
                        continue       
                    #if pa
                    else:
                        tofev = go.events.TofEvent()
                        try:
                            tofev.from_tofpacket(tofpack)
                        except Exception as e:
                            toffy_errors += 1
                            #print (f'-> When reading from the tof stream, we encountered an exception! {e}')
                            break
                        if tofev.mastertriggerevent.event_id == telly_ev.tof.event_id:
                            if args.reprocess:
                                new_tofev = go.liftof.waveform_analysis(tofev, settings)
                                new_tofpack = new_tofev.pack()
                                frame.put_tofpacket(new_tofpack, str('TofEvent.reprocessed'))
                            frame.put_tofpacket(tofpack, str(tofpack.packet_type))
                            break
                        # if it is larger, we buffer it for now
                        if tofev.mastertriggerevent.event_id >= telly_ev.tof.event_id:
                            tofevent_buffer_later[tofev.mastertriggerevent.event_id] = tofpack
                            #print (f'--> Buffer size of TOF events which are behind the telemetry stream  : {len(tofevent_buffer_later)}')
                            break
                            # if it is smaller, the event has been skipped in the telemetry stream, so we will 
                            # buffer it as well
                        if tofev.mastertriggerevent.event_id <= telly_ev.tof.event_id:
                            tofevent_buffer_earlier[tofev.mastertriggerevent.event_id] = tofpack
                            #print (f'--> Buffer size of TOF events which are ahead of the telemetry stream: {len(tofevent_buffer_earlier)}')
                            continue # get the next tofpacket
                    if args.verbose:
                        print (frame)
            writer.add_frame(frame)
            frames_written += 1
            # At this point, print a little summary
            
        telly_exhausted = True
        read_tof_files = toffy_f_idx + 2
        read_tel_files = telly_f_idx + 2
        elapsed = (time.time() - start_time)/60
        print ('--------------------------------')
        print (f'--> Read {read_tel_files} Telemetry files ({100*read_tel_files/len(telemetry_files):.2f}%), {read_tof_files} TOF files ({100*read_tof_files/len(tof_files):.2f})% in {elapsed:4.2f} minutes!')
        print (f'--> Encountered {telly_errors} errors for TelemetryPackets, {toffy_errors} for TofPackets')
        print (f'--> Buffer size of TOF events which are ahead of the telemetry stream: {len(tofevent_buffer_earlier)}')
        print (f'--> Buffer size of TOF events which are behind the telemetry stream  : {len(tofevent_buffer_later)}')
        print (f'--> {frames_written} frames written!')
        print (f'--> {frame}')
        #raise
