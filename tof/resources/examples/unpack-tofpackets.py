#! /usr/bin/env python3

"""
Example: How to read files with tofpackets from the python API
"""

# make sure gaps-online-software/build (or whereever you built the software) is in your 
# PYTHONPATH and it was build with pybind11 (BUILD_PYBINDINGS=ON)
import gaps_tof as gt

if __name__ == '__main__':
    import argparse
 
    parser = argparse.ArgumentParser(description='Example on how to read TofPackets from a file.')
    parser.add_argument('tofpacketfile', metavar='packetfile', type=str,
                        help='A file with tofpackets in it')
    parser.add_argument('--calibration', '-c', dest='calibration',
                        default='', type=str,
                        help='Calibration txt file for this specific readout board.')
    
    args = parser.parse_args()
    if args.calibration:
        calib = gt.RBCalibration.from_txtfile(args.calibration)
    
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
                print (calib.nanoseconds(rb_ev))
                print (calib.voltages(rb_ev, spike_cleaning=True))

