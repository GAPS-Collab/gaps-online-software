#! /usr/bin/env python

"""
Read raw tof data stream with the rust/python API

* Read waveforms and do waveform calibration
* Sensor data from RBs
* Plot simple distributions
"""

from pathlib import Path
import os.path

import charmingbeauty as cb
import dashi as d

import gaps_online as go

d.visual()

if __name__ == '__main__':

    import argparse
    
    parser = argparse.ArgumentParser(description='Example to illustrate how to deal with TOF rawdata from .tof.gaps files. Read data, use RBcalibrations to get waveforms, read sensor data, etc...')
    parser.add_argument('rundir', metavar='rundir', type=str,
                        help='A directory with .tof.gaps files or a single .tof.gaps.file')
    parser.add_argument('--calibration', '-c', dest='calibration',
                        default='', type=Path,
                        help='Path to calibration txt files for all boards.')

    args = parser.parse_args()

    # read in calibration data
    cali = go.tof.calibrations.load_calibrations_rapi(args.calibration)
    if os.path.isfile(args.rundir):
        runfiles = [args.rundir]
    else:
        runfiles = Path(args.rundir)
        runfiles = runfiles.glob('*.tof.gaps')
    
    print(f"=> Will run over {len(runfiles)} files!")
    nevents_tot = 0
    nwfs_cali   = 0
    for f in runfiles:
        event_reader = go.rust_api.io.TofPacketReader(f, filter=go.rust_api.io.PacketType.TofEvent)
        rbmoni       = go.rust_api.moni.RBMoniSeries()
        # this converts the custom series into a polars data frame!
        rbmoni       = rbmoni.from_file(f)
        print(rbmoni)
        for pack in event_reader:
            ev = go.rust_api.events.TofEvent()
            ev.from_tofpacket(pack)
            nevents_tot += 1
            #print (ev)
            if len(ev.waveforms) > 0:
                wf = ev.waveforms[0] # assuming ev is the event from above
                wf.calibrate(cali[wf.rb_id])
                nwfs_cali += 1
    print(f'=> Walked over {nevents_tot} events!')
    print(f'=> Calibrated {nwfs_cali} waveforms!')
