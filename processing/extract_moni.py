#! /usr/bin/env python 

"""
Walk over all telemetry files and write out anything but 
merged events into a single file.
"""

import time
from pathlib import Path

import gaps_online as go

if __name__ == '__main__':
    import argparse
    import sys

    parser = argparse.ArgumentParser(description='Walk over binary data files and extract anything thais not a merged event')
    parser.add_argument('--telemetry-dir', default='/data0/gaps/csbf/csbf-data/binaries/ethernet',\
                        help='A directory with telemetry binaries, as received from the telemetry stream',
                        )
    parser.add_argument('-n','--npackets', type=int,\
                        default=-1, help='Limit readout to npackets, -1 for all packets (default)')
    parser.add_argument('-s','--start-time',\
                        type=int, default=-1,\
                        help='The run start time, e.g. as taken from the elog')
    parser.add_argument('-e','--end-time',
                        type=int, default=-1,\
                        help='The run end time, e.g. as taken from the elog')
    parser.add_argument('-o','--outdir',\
                        help='Outdir for caraspace output files',
                        type=Path,
                        default=None)
    parser.add_argument('-v','--verbose', action='store_true',\
                        help='More verbose output')
    args = parser.parse_args()

    # create a fake run id for the writer
    runid  = 1
    reader = go.io.TelemetryPacketReader(args.telemetry_dir)
    file_timestamp = str(int(time.time())) 
    writer = go.io.CRWriter(str(args.outdir), runid, timestamp = file_timestamp)
    frames_written = 0
    start_time = time.time()
    for pack in reader:
        frame = go.io.CRFrame()
        if pack.packet_type in [go.io.TelemetryPacketType.InterestingEvent,
                                go.io.TelemetryPacketType.BoringEvent,
                                go.io.TelemetryPacketType.NoGapsTriggerEvent,
                                go.io.TelemetryPacketType.NoTofDataEvent,
                                go.io.TelemetryPacketType.Tracker]:
            continue
        frame.put_telemetrypacket(pack, str(pack.packet_type))
        if frames_written % 1e6 == 0 and frames_written != 0: # or n_toffy_errors % 1000 == 0 or n_telly_errors % 1000 == 0:
            elapsed = (time.time() - start_time)
            print ('--------------------------------')
            #print (f'--> Read {telly_f_idx + 1} Telemetry files ({100*(telly_f_idx + 1)/len(telemetry_files):.2f}%), {read_tof_files} TOF files ({100*read_tof_files/len(tof_files):.2f})% in {elapsed:4.2f} minutes!')
            print (f'--> {frames_written} frames written in {elapsed:.1f}s')
            print (f'--> {frame}')
        writer.add_frame(frame)
        frames_written += 1 
        #break
