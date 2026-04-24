#! /usr/bin/env python 

"""
Convert raw (.bin) files from the GAPS experiment
to a ROOT formt whcih is used with SimpleDet, the 
analysis code widely used in GAPS.
"""

import tqdm
import gondola as go
import time
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

# check gondola version
if not (go.get_version_minor() >= 12 and go.get_version_patch() >= 20):
    print(f'ERROR - got version {go.get_version_major()}.{go.get_version_minor()}.{go.get_version_patch()}')
    raise ImportError(f"gondola needs to be at least version 0.12.20!")

#TRK_MASK = "/srv/gaps/crane/v26.01/calib-unpack/cal_run_10047//tracker_channel_enables_100.txt"
#TRK_PED  = "/srv/gaps/crane/v26.01/calib-unpack/cal_run_10047/List-251216-NZS-0.txt-pedestals-cn2-mod2.txt"
#TRK_TRF  = "/srv/gaps/crane/v26.01/calib-unpack/cal_run_10047//TF_1216.txt"
#TRK_PLS  = "/srv/gaps/crane/v26.01/calib-unpack/cal_run_10047//pulch_10047.txt" 
#TRK_GAIN = "/srv/gaps/crane/v26.01/calib-unpack/cal_run_10047//List-251216-NZS-0.txt-gains-cn2-mod2.txt"

#v26.01 processing
TRK_TRF   = "/srv/gaps/crane/v26.03/build/install/gaps-v26.3/resources/calibration//trk-2025/TF_Fit_Coefficients_Calibration_1217_fit.txt" 
TRK_MASK  = "/srv/gaps/crane/v26.03/build/install/gaps-v26.3/resources/calibration//trk-2025/tracker_channel_enables_100.txt"
TRK_PED   = "/srv/gaps/crane/v26.03/build/install/gaps-v26.3/resources/calibration//trk-2025/ped_1217.txt"
TRK_PLS   = "/srv/gaps/crane/v26.03/build/install/gaps-v26.3/resources/calibration//trk-2025/251217-calibration.root-888888-pulse-mask-cut.txt"
TRK_GAIN  = "/srv/gaps/crane/v26.03/build/install/gaps-v26.3/resources/calibration//trk-2025/List-251216-NZS-0.txt-gains-cn2-mod2.txt"
  
GEO      = "/srv/gaps/crane/v26.01/resources/geometry/geometry.v25.09.root"

try:
    import gondola_cxx as gxx 
except ImportError:
    print ("Unable to import python/C++ bindings for gondola (gondola_cxx)")
    import sys
    sys.exit(1)

#TRK_MEV_CUT=0.4
TRK_MEV_CUT=0

# bridge the gap between the rust library and the C++ library. 
# the difference is not immediately obvious, it is just the 
# implementation. Since we can only deal with SD's root format 
# in C++ because the member of CTrackRec* is not supported in 
# either python (uproot) or any of the more popular rust root 
# libraries (as of 2026)
def rust_to_cxx_bridge(event): 
    """
    This will bridge between rust and C++ 
    implementations of the gondola-core library 

    # Args:
        event (gondola.events.TelemetryEvent) [rust library] 

    # Returns:
       gondola_cxx.gondola_cxx.TelemetryEvent [CXX library]

    """
    cxx_event      = gxx.gondola_cxx.TelemetryEvent()
    tof_event      = event.tof # important, since this produces a copy!
                               # otherwise, it will be slow if we access 
                               # the fields since it will always copy the 
                               # complete struct
    cxx_tof_event  = gxx.gondola_cxx.TofEventSummary()
    cxx_tof_event.event_id      = tof_event.event_id 
    cxx_event.event_id          = tof_event.event_id 
    cxx_tof_event.run_id        = tof_event.run_id
    cxx_tof_event.dsi_j_mask    = tof_event.dsi_j_mask
    cxx_tof_event.channel_masks = tof_event.channel_masks 
    cxx_tof_event.trigger_sources_bytes = tof_event.trigger_sources_bytes

    #print (cxx_tof_event.dsi_j_mask, tof_event.dsi_j_mask)
    #print (f"--> Will bridge {len(tof_event.hits)} TOF hits")
    cxx_hits = []
    for h in tof_event.hits:
        h_cxx = gxx.gondola_cxx.TofHit() 
        h_cxx.paddle_id  = h.paddle_id
        h_cxx.time_a     = h.time_a 
        h_cxx.time_b     = h.time_b 
        h_cxx.charge_a   = h.charge_a 
        h_cxx.charge_b   = h.charge_b 
        h_cxx.peak_a     = h.peak_a 
        h_cxx.peak_b     = h.peak_b 
        h_cxx.paddle_len = h.paddle_len/10
        h_cxx.event_t0   = h.event_t0
        cxx_hits.append(h_cxx)
    cxx_tof_event.hits = cxx_hits 
    cxx_event.tof = cxx_tof_event
    cxx_hits = [] 
    for h in event.tracker:
        cxx_trk_hit = gxx.gondola_cxx.TrkHit()
        cxx_trk_hit.layer   = h.layer 
        cxx_trk_hit.row     = h.row
        cxx_trk_hit.module  = h.module 
        cxx_trk_hit.channel = h.channel 
        cxx_trk_hit.energy  = h.energy
        cxx_trk_hit.adc     = h.adc
        # the "tracker" mev cut
        #if h.energy < TRK_MEV_CUT:
        #    #print (h) 
        #    #exit()
        #    pass
        #else:
        cxx_hits.append(cxx_trk_hit) 
    cxx_event.tracker = cxx_hits
    return cxx_event 

if __name__ == '__main__':

    import argparse
    #import sys

    description = """Calibrate L0 data and write SimpleDet compatible ROOT files"""
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument('--run-dir', default=Path('/data0/gaps/csbf/csbf-data/binaries/ethernet'),\
                        help='A directory with telemetry binaries, as received from the telemetry stream',\
                        type=Path,
                        )
    parser.add_argument('--telemetry-dir', default=Path('/data0/gaps/csbf/csbf-data/binaries/ethernet'),\
                        help='A directory with telemetry binaries, as received from the telemetry stream',\
                        type=Path,
                        )
    parser.add_argument('-n', '--n-events', type=int,\
                        default=0, help='Only process -n number of events')
    parser.add_argument('--gcu-seconds-time-cutoff', type=int,\
                        default=100, help='How many seconds of the gcu clock are allowed to pass between tracker and merged event packets to still allow the merge? Typically the time gap is ~60s. If this number is too large, we risk merging event ids from different runs')
    parser.add_argument('--remove-cmn', action='store_true',\
                        default=False,
                        help='Remove the common noise as identifier by the tracker team')
    parser.add_argument('--control-plots', action='store_true',\
                        default=False,
                        help='More verbose output')
    
    #parser.add_argument('-v','--verbose', action='store_true',\
    #                    help='More verbose output')
    args = parser.parse_args()

    trk_mask = go.db.TrackerStripMask.parse_from_file            (TRK_MASK)
    trk_ped  = go.db.TrackerStripPedestal.parse_from_file        (TRK_PED)
    trk_trf  = go.db.TrackerStripTransferFunction.parse_from_file(TRK_TRF)
    trk_pls  = go.db.TrackerStripPulse.parse_from_file           (TRK_PLS)
    trk_gain = go.db.TrackerStripGain.parse_from_file            (TRK_GAIN)

    trk_mask = {k.strip_id : k for k in trk_mask} 
    trk_ped  = {k.strip_id : k for k in trk_ped} 
    trk_trf  = {k.strip_id : k for k in trk_trf} 
    trk_pls  = {k.strip_id : k for k in trk_pls} 
    trk_gain = {k.strip_id : k for k in trk_gain} 

    tracker_cali = go.calibration.TrackerOfflineCalibration() 
    tracker_cali.mask_map  = trk_mask 
    tracker_cali.ped_map   = trk_ped 
    tracker_cali.tf_map    = trk_trf 
    tracker_cali.pulse_map = trk_pls 
    tracker_cali.gain_map  = trk_gain
    tracker_cali.remove_cmn = args.remove_cmn
    tracker_cali.remove_pulsed = True
    print (tracker_cali)
   
    files   = [str(k) for k in sorted(args.telemetry_dir.glob('*.bin'))]
    print (f'--> Found {len(files)} telemetry files!')
    nth_event = 0
    done      = False 
    # do some benchmarking 
    benchfile = open('benchmarking-l1py1.dat', 'w')
    start_time = time.time()
    for f in tqdm.tqdm(files, total=len(files)):
        if done:
            break
        root_writer = gxx.gondola_cxx.SDRootWriter(f.replace('.bin','.root')) 
        root_writer.write_sdpar(10000, "valkyrie", "v23.03")
        #reader  = go.io.TelemetryPacketReader(args.telemetry_dir) 
        reader   = go.io.TelemetryPacketReader(f)
        n_packs, _n_err, _data = reader.count_packets()
        for pack in tqdm.tqdm(reader, total=n_packs):
            if pack.is_event_packet:
                if nth_event >= args.n_events and args.n_events > 0:
                    done = True
                    break
                ev = go.events.TelemetryEvent.from_telemetrypacket(pack) 
                # calibration steps 
                n_trk_hits_before = len(ev.tracker) 
                ev.calibrate_trk_hits(tracker_cali)
                #print(f'-> We masked {n_trk_hits_before - len(ev.tracker)} tracker hits!')
                ev.tof_normalize_hit_times()
                #tracker_cali.calibrate_event(ev) 
                #print (ev)
                cxx_ev = rust_to_cxx_bridge(ev)
                #print (cxx_ev.tof.dsi_j_mask, "cxx dsi j mask")
                root_writer.add_event(cxx_ev, pack.header.packet_type, pack.header.gcutime)
                nth_event += 1
                if nth_event % 500 == 0:
                    timedelta = time.time() - start_time
                    benchfile.write(f'\n {nth_event} {timedelta}')
                    start_time = time.time()
        # FIXME - if this is missing, all but the last root file will be garbage
        del root_writer # explicetly delete it here, there is some memory issue
        #break
    #run_meta = Path(args.run_dir)
    #run_meta = [k for k in run_meta.glob('*.meta.toml')]
    ##print (run_meta)
    #run_meta = run_meta[0]
    #run_meta_data = RunMeta.load(run_meta)
    #
    #max_packs_per_iter = int(4e6) # a typical 1h run has like 3.5M packets
    #packet_offset      = 0
    #remainder_m_ev     = [] 
    #remainder_tracker  = []
    #iteration          = 0
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
