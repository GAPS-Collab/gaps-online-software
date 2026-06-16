#! /usr/bin/env python 

"""
Convert raw (.bin) files from the GAPS experiment
to a ROOT formt whcih is used with SimpleDet, the 
analysis code widely used in GAPS.
"""

import sys
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
if not (go.get_version_minor() >= 12 and go.get_version_patch() >= 25):
    print(f'ERROR - got version {go.get_version_major()}.{go.get_version_minor()}.{go.get_version_patch()}')
    raise ImportError(f"gondola needs to be at least version 0.12.25!")

#TRK_MASK = "/srv/gaps/crane/v26.01/calib-unpack/cal_run_10047//tracker_channel_enables_100.txt"
#TRK_PED  = "/srv/gaps/crane/v26.01/calib-unpack/cal_run_10047/List-251216-NZS-0.txt-pedestals-cn2-mod2.txt"
#TRK_TRF  = "/srv/gaps/crane/v26.01/calib-unpack/cal_run_10047//TF_1216.txt"
#TRK_PLS  = "/srv/gaps/crane/v26.01/calib-unpack/cal_run_10047//pulch_10047.txt" 
#TRK_GAIN = "/srv/gaps/crane/v26.01/calib-unpack/cal_run_10047//List-251216-NZS-0.txt-gains-cn2-mod2.txt"

#v26.01 processing
CRANE_INSTALL = "/srv/gaps/crane/v26.03/build/install/gaps-v26.3/resources/calibration/"
#CRANE_INSTALL = "/home/stoessl/crane/v26.03/build/install/gaps-v26.3/resources/calibration/"
TRK_TRF   = f"{CRANE_INSTALL}trk-2025/TF_Fit_Coefficients_Calibration_1217_fit.txt" 
TRK_MASK  = f"{CRANE_INSTALL}trk-2025/tracker_channel_enables_100.txt"
TRK_PED   = f"{CRANE_INSTALL}trk-2025/ped_1217.txt"
TRK_PLS   = f"{CRANE_INSTALL}trk-2025/251217-calibration.root-888888-pulse-mask-cut.txt"
TRK_GAIN  = f"{CRANE_INSTALL}trk-2025/List-251216-NZS-0.txt-gains-cn2-mod2.txt"  
GEO       = f"{CRANE_INSTALL}/resources/geometry/geometry.v25.09.root"

GEO       = f"/srv/gaps/crane/v26.03/resources/geometry/geometry.v25.09.root"

for k in TRK_TRF, TRK_MASK, TRK_PED, TRK_PLS, TRK_GAIN, GEO:
    if not Path(k).exists():
        print (f'{k} does not exist! Aborting!')
        sys.exit(1) 
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

    parser      = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--telemetry-dir', default=Path('/data0/gaps/csbf/csbf-data/binaries/ethernet'),\
                        help='A directory with telemetry binaries, as received from the telemetry stream',\
                        type=Path,
                        )
    parser.add_argument('-n', '--n-events', type=int,\
                        default=0, help='Only process -n number of events')
    parser.add_argument('--remove-cmn', action='store_true',\
                        default=False,
                        help='Remove the common noise as identifier by the tracker team')
    parser.add_argument('--control-plots', action='store_true',\
                        default=False,
                        help='More verbose output')
    parser.add_argument('-o','--outdir',\
                        help='Outdir for .root output files',
                        type=Path,
                        default=None)
    
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
    # paddle offsets as calculated by Grace
    tof_timing_offsets = go.db.TofPaddleTimingConstant.as_dict_by_name('GraceV1')
    # these are the broken ones
    #tof_timing_offsets = {k : tof_timing_offsets[k].timing_constant for k in tof_timing_offsets}
    # fix the timing constants by subtracting the panel constant 
    tof_timing_offsets = {k : tof_timing_offsets[k].paddle_constant - tof_timing_offsets[k].panel_constant for k in tof_timing_offsets}

    print (f'--> Loaded TOF timing constants for  {len(tof_timing_offsets)} paddles from db!')
    if args.telemetry_dir.is_dir():
        files   = [k for k in sorted(args.telemetry_dir.glob('*.bin'))]
    if args.telemetry_dir.is_file():
        files   = [args.telemetry_dir]
    print (f'--> Found {len(files)} telemetry files!')
    nth_event = 0
    done      = False 
    # do some benchmarking 
    #benchfile = open('benchmarking-l1py1.dat', 'w')
    #start_time = time.time()
    for f in tqdm.tqdm(files, total=len(files)):
        if done:
            break
        outfile = args.outdir / f.name
        outfile = str(outfile)
        root_writer = gxx.gondola_cxx.SDRootWriter(outfile.replace('.bin','.root'), GEO) 
        root_writer.write_sdpar(0, "uhcra", "v23.03")
        #reader  = go.io.TelemetryPacketReader(args.telemetry_dir) 
        reader   = go.io.TelemetryPacketReader(str(f))
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
                ev.tof_set_timing_constants(tof_timing_offsets)
                ev.tof_normalize_hit_times()
                #tracker_cali.calibrate_event(ev) 
                #print (ev)
                cxx_ev = rust_to_cxx_bridge(ev)
                #print (cxx_ev.tof.dsi_j_mask, "cxx dsi j mask")
                root_writer.add_event(cxx_ev, pack.header.packet_type, pack.header.gcutime)
                #nth_event += 1
                #if nth_event % 500 == 0:
                #    timedelta = time.time() - start_time
                #    benchfile.write(f'\n {nth_event} {timedelta}')
                #    start_time = time.time()
        # FIXME - if this is missing, all but the last root file will be garbage
        del root_writer # explicetly delete it here, there is some memory issue
        #break

