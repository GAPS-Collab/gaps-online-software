#! /usr/bin/env python

"""
Example: How to read plain RB files with the python API. These files are that what 
we formerly called "BlobFiles".
"""

# make sure gaps-online-software/build (or whereever you built the software) is in your 
# PYTHONPATH and it was build with pybind11 (BUILD_PYBINDINGS=ON).
# You can source the 'setup-env.sh' shell in the install directory.

import gaps_tof as gt

if __name__ == '__main__':
    import argparse
 
    parser = argparse.ArgumentParser(description='Read robin (ReadOutBoardBINary) ("Blob") data from a file.')
    parser.add_argument('filename', type=str,
                        help='A file with plain readoutboard data in it. NOT wrapped in TofPackets.')
    parser.add_argument('--calibration', '-c', dest='calibration',
                        default='', type=str,
                        help='Calibration txt file for this specific readout board.')
    parser.add_argument('--print-waveforms',
                        default=False, action='store_true',
                        help='Print waveform data. ADC, but if calibration is available, print ns/mV instead.')

    
    args = parser.parse_args()
    # in case we have duplicate events in the file (due to a bug in the buffer readout of the RB software)
    # the 'omit_duplicates' flag will eliminate them.
    events      = gt.get_rbeventsmemoryviews(args.filename, omit_duplicates = True)
    calib = None
    if args.calibration:
        calib = gt.RBCalibration.from_txtfile(args.calibration)
    
    for ev in events:
        print (ev)
        #print (f"Event: {ev.event_ctr}")
        if args.print_waveforms:
            if calib is not None:
                print (calib.nanoseconds(ev))
                print (calib.voltages(ev, spike_cleaning=True))
            else:
                print (ev.get_channel_adc())
