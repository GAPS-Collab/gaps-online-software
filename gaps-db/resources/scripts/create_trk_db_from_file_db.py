#! /usr/bin/env python 

"""
Populate the tracker db from db with calibration files and timestamps
as shipped with SimpleDet 
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

# try to suppress RUST logging
import logging
logging.getLogger('go').addHandler(logging.NullHandler())

#import charmingbeauty as cb 
#cb.visual.set_style_default()
#cb.visual.set_style_streamlit_dark()

# check gondola version
if not (go.get_version_minor() >= 12 and go.get_version_patch() >= 20):
    print(f'ERROR - got version {go.get_version_major()}.{go.get_version_minor()}.{go.get_version_patch()}')
    raise ImportError("gondola needs to be at least version 0.12.20!")

if __name__ == '__main__':

    import argparse
    #import sys

    description = """Read the calibration db as shipped with SimpleDet and populate (our) trk calibration db with the respec"""
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument('--sd-cali-db', default=Path('/srv/gaps/crane/v26.03/calibration/resources/CalibrationDB.db'),\
                        help='SimpleDet cali db with filenames and timestamps',\
                        type=Path,
                        )
    parser.add_argument('--gondola-trk-db', default=Path('gondola_trk_db_foo.db'),\
                        help='A file where to dump the trk part of the db used by gondola. For now, this should be the same as the rest of the gaps_flight.db',\
                        type=Path,
                        )
    parser.add_argument('--control-plots', action='store_true',\
                        default=False,
                        help='More verbose output')
    
    #parser.add_argument('-v','--verbose', action='store_true',\
    #                    help='More verbose output')
    args = parser.parse_args()

    trk_cali_files = go.db.load_calibration_db_elena(str(args.sd_cali_db))
    print(f'-> Found {len(trk_cali_files)} entries for calibration files in this database!')

    GONDOLA_TRK_DB = str(args.gondola_trk_db) 

    for cf in trk_cali_files:
        match cf.file_type:
            case go.db.TrackerCalibrationFileType.ChannelMask:
                masks = go.db.TrackerStripMask.parse_from_file(cf.path)
                name  = Path(cf.path).name
                for m in masks:
                    m.name                = name
                    m.utc_timestamp_start = cf.from_timestamp 
                    m.utc_timestamp_stop  = cf.to_timestamp
                go.db.create_trk_mask_table(GONDOLA_TRK_DB, masks)

            case go.db.TrackerCalibrationFileType.Gains:
                print(f'-> Will parse {cf.file_type} from {cf.path}!') 
                gains = go.db.TrackerStripGain.parse_from_file(cf.path)
                print(f'-- --> parsed {len(gains)}!') 
                name  = Path(cf.path).name
                for g in gains:
                    g.name                = name
                    g.utc_timestamp_start = cf.from_timestamp 
                    g.utc_timestamp_stop  = cf.to_timestamp
                go.db.create_trk_gain_table(GONDOLA_TRK_DB, gains)
            
            case go.db.TrackerCalibrationFileType.Pedestal:
                pedestals = go.db.TrackerStripPedestal.parse_from_file(cf.path)
                name  = Path(cf.path).name
                for ped in pedestals:
                    ped.name                = name
                    ped.utc_timestamp_start = cf.from_timestamp 
                    ped.utc_timestamp_stop  = cf.to_timestamp
                go.db.create_trk_pedestal_table(GONDOLA_TRK_DB, pedestals)

            case go.db.TrackerCalibrationFileType.PulsedChannels:
                pulses = go.db.TrackerStripPulse.parse_from_file(cf.path)
                name  = Path(cf.path).name
                for pulse in pulses:
                    pulse.name                = name
                    pulse.utc_timestamp_start = cf.from_timestamp 
                    pulse.utc_timestamp_stop  = cf.to_timestamp
                go.db.create_trk_pulse_table(GONDOLA_TRK_DB, pulses)
            
            case go.db.TrackerCalibrationFileType.TransferFn:
                # don't parse the files in root format
                name = Path(cf.path).name
                if name.endswith('.root'):
                    continue
                print(f'-> Will parse {cf.file_type} from {cf.path}!') 
                tfs  = go.db.TrackerStripTransferFunction.parse_from_file(cf.path)
                print(f'-- --> parsed {len(tfs)}!') 
                for tf in tfs:
                    tf.name                = name
                    tf.utc_timestamp_start = cf.from_timestamp 
                    tf.utc_timestamp_stop  = cf.to_timestamp
                go.db.create_trk_transfer_fn_table(GONDOLA_TRK_DB, tfs)
    

