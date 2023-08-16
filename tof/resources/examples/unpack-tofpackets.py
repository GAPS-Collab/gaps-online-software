#! /usr/bin/env python3

"""
Example: How to read files with tofpackets from the python API
"""

# make sure gaps-online-software/build (or whereever you built the software) is in your 
# PYTHONPATH and it was build with pybind11 (BUILD_PYBINDINGS=ON)
import gaps_tof as gt

from glob import glob
from pathlib import Path
import re


if __name__ == '__main__':
    import argparse
 
    parser = argparse.ArgumentParser(description='Example on how to read TofPackets from a file.')
    parser.add_argument('tofpacketfile', metavar='packetfile', type=str,
                        help='A file with tofpackets in it')
    parser.add_argument('--calibration', '-c', dest='calibration',
                        default='', type=Path,
                        help='Path to calibration txt files for all boards.')
    
    args = parser.parse_args()
    calib = None
    if args.calibration:
        calib = glob(str(args.calibration / '*.txt'))
        print (f'Found {len(calib)} calibration files!')
        pattern = 'rb(?P<id>[0-9]*)_'
        pattern = re.compile(pattern)
        all_calib = {}
        for cal in calib:
            all_calib[int(pattern.search(cal).groupdict()['id'])]\
                    = gt.RBCalibration.from_txtfile(cal)
        calib = all_calib

    packets      = gt.get_tofpackets(args.tofpacketfile)
     
    for pack in packets:
        print (pack)
        ev = gt.TofEvent.from_bytestream(pack.payload, 0)
        for rb_ev in ev.rbevents:
            # event header containing event id etc
            print (rb_ev.header)
            # get channel adcs
            rb_ev.get_channel_adc(1)
            if calib is not None:
                print (calib[rb_ev.header.rb_id].nanoseconds(rb_ev))
                print (calib[rb_ev.header.rb_id].voltages(rb_ev, spike_cleaning=True))

