#! /usr/bin/env python 

"""
Convert L0 (caraspace) or (.bin) files from the GAPS experiment
to a ROOT formt whcih is used with SimpleDet, the 
analysis code used in GAPS.
"""

import sys
import tqdm
import gondola as go
import time
import numpy as np
import csv

from pathlib import Path
from dataclasses import dataclass

# try to suppress RUST logging
import logging
logging.getLogger('go').addHandler(logging.NullHandler())

# check gondola version
GON_VERSION_REQUIRED = '0.12.31' 
if not go.version_at_least(GON_VERSION_REQUIRED):
    print(f'ERROR - got version {go.get_version()} but need version {GON_VERSION_REQUIRED}')
    raise ImportError("gondola needs to be at least version {GON_VERSION_REQUIRED}!")


#v26.08 processing
#CRANE_INSTALL = "/srv/gaps/crane/v26.03/build/install/gaps-v26.3/resources/calibration/"
CRANE_VERSION = "v26.8" # it is stupid that sometimes we have the proceeding 0 here
CRANE_BASEDIR = f"/home/stoessl/crane/v26.08/"
CRANE_INSTALL = f"{CRANE_BASEDIR}build/install/gaps-{CRANE_VERSION}/resources/calibration/"
TRK_TRF   = f"{CRANE_INSTALL}trk-2025/TF_Fit_Coefficients_Calibration_1217_fit.txt" 
TRK_MASK  = f"{CRANE_INSTALL}trk-2025/tracker_channel_enables_100.txt"
TRK_PED   = f"{CRANE_INSTALL}trk-2025/ped_1217.txt"
TRK_PLS   = f"{CRANE_INSTALL}trk-2025/251217-calibration.root-888888-pulse-mask-cut.txt"
TRK_GAIN  = f"{CRANE_INSTALL}trk-2025/List-251216-NZS-0.txt-gains-cn2-mod2.txt"  
GEO       = f"{CRANE_INSTALL}/resources/geometry/geometry.v25.09.root"

GEO       = f"{CRANE_BASEDIR}/resources/geometry/geometry.v25.09.root"
CALI_DB   = F"{CRANE_BASEDIR}/calibration/resources/CalibrationDB.db"

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

def get_refined_edep(paddle_num,edep, cc, temperature_paddle):
    """
    Fix the temperature dependent gain by 
    applying Grace's constants
    """
    ## we need to apply the slope and y_intercept to each temperature value, to get a predicted MPV from the fit: 
    slope        = cc["slope"]
    y_int        = cc["y_intercept"]
    t_avg        = cc["T_avg"] 
    mpv_predicted  = temperature_paddle*slope + y_int
    mpv_reference  = t_avg*slope + y_int
    #print (mpv_reference) 
    #print (mpv_predicted)
    #print (edep)
    # temperature calibration formula:
    edep_corrected = edep*(mpv_reference/mpv_predicted)
    
    #print (edep,edep_corrected)
    #print ('-----------------')
    try:
        coefficient    = 1.22/cc[paddle_num]["measured_mip"]
    except KeyError:
        return edep
    return edep_corrected*coefficient

# bridge the gap between the rust library and the C++ library. 
# the difference is not immediately obvious, it is just the 
# implementation. Since we can only deal with SD's root format 
# in C++ because the member of CTrackRec* is not supported in 
# either python (uproot) or any of the more popular rust root 
# libraries (as of 2026)
def rust_to_cxx_bridge(event, tof_paddles, cc = None, all_paddle_temps = None): 
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
    cxx_tof_event.set_timestamp48(tof_event.timestamp48)
    cxx_tof_event.run_id        = tof_event.run_id
    cxx_tof_event.dsi_j_mask    = tof_event.dsi_j_mask
    cxx_tof_event.channel_masks = tof_event.channel_masks 
    cxx_tof_event.trigger_sources_bytes = tof_event.trigger_sources_bytes
    cxx_tof_event.set_event_status(int(tof_event.status))
    #print (cxx_tof_event.dsi_j_mask, tof_event.dsi_j_mask)
    #print (f"--> Will bridge {len(tof_event.hits)} TOF hits")
    cxx_hits = []
    for h in tof_event.hits:
        #pdl = tof_paddles[h.paddle_id] 
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
        if cc is not None: 
            # energy correction - cc are the global 
            # energy conrrecton constants 
            print (h_cxx.edep, h_cxx.edep_att, h_cxx.edep_birk, h.edep, h.edep_att, h.edep_att_birk)
            print ('-- -- -- -- --')
            if h.paddle_id in cc.keys():
                temp = all_paddle_temps[h.paddle_id].get_for_ts(cxx_tof_event.timestamp48)
                if temp[0] == -273.0 or temp[1] == -273.0:
                    h_cxx.edep_corrected = h.edep_att_birk
                    #print (cxx_tof_event.timestamp48)
                    #print (f'Paddle temperature {temp} invalid!')
                else:
                    temp = (temp[0] + temp[1]) / 2
                    r_edep =  get_refined_edep(h.paddle_id, h.edep_att_birk, cc[h.paddle_id], temp)
                    h_cxx.edep_corrected = r_edep
                    print (h.edep_att, h.edep_att_birk)
            else:
                h_cxx.edep_corrected = h.edep_att_birk
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
    parser.add_argument('--input-dir', default=Path('/data0/gaps/csbf/csbf-data/binaries/ethernet'),\
                        help='A directory with telemetry binaries, as received from the telemetry stream',\
                        type=Path,
                        )
    parser.add_argument('--run-id', default=None,\
                        help='Run id (currently only relevant for ground data, either 240,243,251',\
                        type=int,
                        )
    parser.add_argument('-n', '--n-events', type=int,\
                        default=0, help='Only process -n number of events')
    parser.add_argument('--remove-cmn', action='store_true',\
                        default=False,
                        help='Remove the common noise as identifier by the tracker team')
    parser.add_argument('--ground', action='store_true',\
                        help='Advises the script that it is intended to deal with ground data. This will e.g. change the way how it deals with the tracker calibratoin files')
    parser.add_argument('--quiet', action='store_true',\
                        help='Suppress unnecessary output (e.g. progressbar) for use on cluster!')
    parser.add_argument('-o','--outdir',\
                        help='Outdir for .root output files',
                        type=Path,
                        default=None)

    
    #parser.add_argument('-v','--verbose', action='store_true',\
    #                    help='More verbose output')
    args = parser.parse_args()
    # Grace's energy corrections
    ## the calibration constants are stored in a .csv file: 
    calibration_constants = {}
    with open("/home/gtytus/analysis/grace/python/edeps/final/paddle_calibrations.csv", newline="") as f:
        reader = csv.DictReader(f)
    
        for row in reader:
            paddle = int(row["paddle"])
            
            ## the slope, y_intercept and T_avg are used for the intra-paddle temperature calibration, 
            ## and the normalization_coeff is used for the inter-paddle MIP MPV calibration.
            calibration_constants[paddle] = {
                "slope": float(row["slope"]),
                "y_intercept": float(row["y_intercept"]),
                "T_avg": float(row["T_avg"]),
                "coefficient": float(row["normalization_coeff"]),
            }
    if args.ground:
        print('-> ground data!')
        match args.run_id:
            case 240:
                TRK_MASK  = f"{CRANE_INSTALL}trk-2025/SiLi_mask_251204.txt"
                TRK_PED   = f"{CRANE_INSTALL}trk-2025/pedestal_24November2025.txt"
                TRK_TRF   = f"{CRANE_INSTALL}trk-2025/TF_Fit_Coefficients_Calibration_24November2025_Updated.txt" 
                TRK_PLS   = f"{CRANE_INSTALL}trk-2025/pulch_240.txt"
                TRK_GAIN  = f"{CRANE_INSTALL}trk-2025/List-251127-NZS.txt-gains-cn2-mod2.txt"  
            case 243:
                TRK_MASK  = f"{CRANE_INSTALL}trk-2025/SiLi_mask_251205.txt"
                TRK_PED   = f"{CRANE_INSTALL}trk-2025/pedestal_24November2025.txt"
                TRK_TRF   = f"{CRANE_INSTALL}trk-2025/TF_Fit_Coefficients_Calibration_24November2025_Updated.txt" 
                TRK_PLS   = f"{CRANE_INSTALL}trk-2025/pulch_243.txt"
                TRK_GAIN  = f"{CRANE_INSTALL}trk-2025/List-251127-NZS.txt-gains-cn2-mod2.txt"  
            case 251:
                # this really seems to be the same mask
                TRK_MASK  = f"{CRANE_INSTALL}trk-2025/SiLi_mask_251205.txt"
                TRK_PED   = f"{CRANE_INSTALL}trk-2025/pedestal_24November2025.txt"
                TRK_TRF   = f"{CRANE_INSTALL}trk-2025/TF_Fit_Coefficients_Calibration_24November2025_Updated.txt" 
                TRK_PLS   = f"{CRANE_INSTALL}trk-2025/pulch_251.txt"
                TRK_GAIN  = f"{CRANE_INSTALL}trk-2025/List-251127-NZS.txt-gains-cn2-mod2.txt"  
            case _:
                raise ValueError(f"Selected ground data, but for an uknown run. --run-id must be either 240, 243, or 251! We got {args.run_id} instead!")
        
        for k in TRK_MASK, TRK_PED, TRK_TRF, TRK_PLS, TRK_GAIN:
            print ('---- ---- ----')
            print (f'-- {k}')
        trk_mask = go.db.TrackerStripMask.parse_from_file            (TRK_MASK)
        trk_ped  = go.db.TrackerStripPedestal.parse_from_file        (TRK_PED)
        trk_trf  = go.db.TrackerStripTransferFunction.parse_from_file(TRK_TRF)
        trk_pls  = go.db.TrackerStripPulse.parse_from_file           (TRK_PLS)
        trk_gain = go.db.TrackerStripGain.parse_from_file            (TRK_GAIN)
        
        # currently, this is too annoying, hardcode the files here
        #cali_db = go.db.load_calibration_db_elena(CALI_DB)
        #if args.input_dir.is_dir():
        #    meta_files   = [k for k in sorted(args.input_dir.glob('*.toml'))]
        #    meta_files   = meta_files[0]
        #    run_meta     = go.run.RunMeta.load(meta_files)
        #    start_ts     = run_meta.start_gcu_time 
        #    stop_ts      = run_meta.stop_gcu_time 
        #    eligible_cali_files = []
        #    cali_files   = {'mask': [], 'ped' : [], 'tfn': [], 'pls':[], 'gain':[]}
        #    cft          = go.db.TrackerCalibrationFileType
        #    for cf in cali_db:
        #        # add a "safety margin" of 60 seconds around 
        #        # the calibration files to account for ELOG/gcu/gps
        #        # time mismatches
        #        if start_ts < cf.from_timestamp and abs(start_ts - cf.from_timestamp) > 600:
        #            print ('--- --- ---')
        #            print (f'-> {start_ts}')
        #            print (f'-> Removing {cf}!')
        #            continue
        #        match cf.file_type:
        #            case cft.ChannelMask:
        #                cali_files['mask'].append(cf)
        #            case cft.Pedestal:
        #                cali_files['ped'].append(cf) 
        #            case cft.TransferFn:
        #                cali_files['tfn'].append(cf)
        #            case cft.PulsedChannels:
        #                cali_files['pls'].append(cf)
        #            case cft.Gains:
        #                cali_files['gain'].append(cf)
        #            case _:
        #                print ('-> Unknonw Trk calibration file type! {cf}')
        #    for k in cali_files.keys():
        #        if len(cali_files[k]) == 0:
        #            raise ValueError(f"Missing Tracker calibratoin files for {name}!")
        #        if len(cali_files[k]) != 1:
        #            # iteratively clean the list to select the most suitable file 
        #            tmp_cali_files = []
        #            for cf in cali_files[k]:
        #                ts_start = abs(cf.from_timestamp - start_ts)  
        #                ts_stop  = abs(cf.to_timestamp - stop_ts)
        #                tmp_cali_files.append((ts_start, ts_stop, cf))
        #            if len(tmp_cali_files) != 1: 
        #                print ('Fixing cali files...')
        #                clean_cali_files = []
        #                smallest_window  = np.inf 
        #                best_idx         = -1 
        #                for idx,cf in enumerate(tmp_cali_files):
        #                    
        #                    if cf[0] + cf[1] <= smallest_window:
        #                        best_idx = idx
        #                        smallest_window = cf[0] + cf[1]
        #                
        #                clean_cali_files = [tmp_cali_files[best_idx][2]]
        #            else:
        #                clean_cali_files = [tmp_cali_files[0][0]]
        #            cali_files[k] = clean_cali_files
        #    for k in cali_files.keys():
        #        print (f'-> Using {k}: {cali_files[k]}')
        #        if len(cali_files[k]) != 1: # we just fixed it above
        #            raise ValueError(f'Ambiguous cali files {k}')
        #        cali_files[k] = cali_files[k][0]

        #    trk_mask = go.db.TrackerStripMask.parse_from_file(cali_files['mask'].path) 
        #    trk_ped  = go.db.TrackerStripPedestal.parse_from_file(cali_files['ped'].path) 
        #    trk_trf  = go.db.TrackerStripTransferFunction.parse_from_file(cali_files['tfn'].path) 
        #    trk_pls  = go.db.TrackerStripPulse.parse_from_file(cali_files['pls'].path) 
        #    trk_gain = go.db.TrackerStripGain.parse_from_file(cali_files['gain'].path) 
    else:
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
    print ('--- --- --- --- ---')
    print (tracker_cali)
    #if args.ground:
    #    sys.exit(0)
   
    # paddle offsets as calculated by Grace
    tof_timing_offsets = go.db.TofPaddleTimingConstant.as_dict_by_name('GraceV1.5')
    tof_paddles        = go.db.TofPaddle.all_as_dict()
    # these are the broken ones
    #tof_timing_offsets = {k : tof_timing_offsets[k].timing_constant for k in tof_timing_offsets}
    # fix the timing constants by subtracting the panel constant
    # update - this is no longer necessary for GraceV1.5, however, it 
    # does not hurt. However, for that version, the timing field could 
    # also be used directly 
    tof_timing_offsets = {k : tof_timing_offsets[k].paddle_constant - tof_timing_offsets[k].panel_constant for k in tof_timing_offsets}

    # temperatures 
    all_paddle_temps = go.db.TofPaddleTemp.all_data()

    print (f'--> Loaded TOF timing constants for  {len(tof_timing_offsets)} paddles from db!')
    is_caraspace = False
    print (f'--> Reading {args.input_dir}!')
    if args.input_dir.is_dir():
        files   = [k for k in sorted(args.input_dir.glob('*.bin'))]
        if not files:
            files = [k for k in sorted(args.input_dir.glob('*.gaps'))]
            is_caraspace = True
    if args.input_dir.is_file():
        files   = [args.input_dir]
        if args.input_dir.endswith('.gaps'):
            is_caraspace = True
    print (f'--> Found {len(files)} telemetry files!')

    nth_event = 0
    done      = False 
    # do some benchmarking 
    #benchfile = open('benchmarking-l1py1.dat', 'w')
    #start_time = time.time()
    for f in tqdm.tqdm(files, total=len(files)):
        if done:
            break
        fname = str(f.name) 
        if fname.endswith('.bin'):
            outfile_fname = fname.replace('.bin', '.root')
        if fname.endswith('.gaps'):
            outfile_fname = fname.split('.')[1] 
            outfile_fname = 'RAW' + outfile_fname + '.root' 

        outfile = args.outdir / outfile_fname
        outfile = str(outfile)
        print (f'-> Writing to {outfile}')
        root_writer = gxx.gondola_cxx.SDRootWriter(outfile, GEO) 
        root_writer.write_sdpar(0, "uhcra", CRANE_VERSION)
        #reader  = go.io.TelemetryPacketReader(args.input_dir) 
        is_event = lambda x : x.is_event_packet
        if is_caraspace:
            reader  = go.io.CRReader(str(f)) 
            # we can treat the packets/frames more 
            # or less interchangeably. 
            # one of them represent an event, that is 
            # important
            n_packs = reader.count_frames() 
            is_event = lambda x : True # all frames 
                                       # should be events
        else:
            reader   = go.io.TelemetryPacketReader(str(f))
            n_packs, _n_err, _data = reader.count_packets()
        for pack in tqdm.tqdm(reader, total=n_packs, disable=args.quiet):
            if is_event(pack):
                if nth_event >= args.n_events and args.n_events > 0:
                    done = True
                    break
                if is_caraspace:
                    frame = pack
                
                    pack  = frame.get_telemetrypacket('TelemetryEvent') 
                    pack_gcutime = pack.header.gcutime 
                    pack_ptype   = pack.header.packet_type 

                    #ev = frame.get_telemetryevent('TelemetryEvent')
                    ev = go.events.TelemetryEvent.from_telemetrypacket(pack)
                    extra_trk_hits = frame.get_tracker_hitseries('TrkAuxGcu')
                    len_all_extra_hits = len(extra_trk_hits)
                    event_trk_stripids = [h.strip_id for h in ev.tracker]
                    # the dictionary here will ensure that per strip we only 
                    # have a single hit. This should be guaranteed by design, 
                    # except if we have a problem with mixing runs or anything 
                    # else
                    # -- 
                    # the dictionary should have less hits than then list! 
                    extra_trk_hits     = {h.strip_id : h for h in extra_trk_hits}
                    if len_all_extra_hits != len(extra_trk_hits):
                        # we modify the error here, since we are lossing hits
                        # this is definitely not good, and needs to be investigated
                        # but currently we only warn and need to investigate later
                        print(f"[ERROR] Duplicate hits in extra hits! Before filtering per strip id, we have {len_all_extra_hits}, but after we have {len(extra_trk_hits)}")
                    extra_trk_hits     = [extra_trk_hits[h] for h in extra_trk_hits if not h in event_trk_stripids] 
                    ev.add_tracker_hits(extra_trk_hits, True)

                else:
                    # all tracker hits should be in the binary file already!
                    ev = go.events.TelemetryEvent.from_telemetrypacket(pack) 
                    pack_gcutime = pack.header.gcutime 
                    pack_ptype   = pack.header.packet_type 
                # calibration steps 
                n_trk_hits_before = len(ev.tracker)
                # sadly, in the curretn implemetnation, we have to do the 
                # calibration twice, once to calibrate all hits, the second
                # time to remove the pulsed hits 
                ev.calibrate_trk_hits(tracker_cali, False)

                #print(f'-> We masked {n_trk_hits_before - len(ev.tracker)} tracker hits!')
                # for reading from caraspace files, the paddles do not get set 
                # automatically (yet) in all cases
                ev.tof_set_paddles(tof_paddles)
                ev.tof_normalize_hit_times()
                ev.tof_set_timing_constants(tof_timing_offsets)
                #tracker_cali.calibrate_event(ev) 
                #print (ev)
                #print (calibration_constants)
                cxx_ev_for_raw = rust_to_cxx_bridge(ev, tof_paddles, calibration_constants, all_paddle_temps)
                # now we remove the pulsed hits for the rec part
                ev.calibrate_trk_hits(tracker_cali, True)
                cxx_ev_for_rec = rust_to_cxx_bridge(ev, tof_paddles, calibration_constants, all_paddle_temps)
            


                #print (cxx_ev.tof.dsi_j_mask, "cxx dsi j mask")
                #print (cxx_ev)
                root_writer.add_event(cxx_ev_for_rec, cxx_ev_for_raw, pack_ptype, pack_gcutime, True)
                #nth_event += 1
                #if nth_event % 500 == 0:
                #    timedelta = time.time() - start_time
                #    benchfile.write(f'\n {nth_event} {timedelta}')
                #    start_time = time.time()
        # FIXME - if this is missing, all but the last root file will be garbage
        del root_writer # explicetly delete it here, there is some memory issue
        #break

