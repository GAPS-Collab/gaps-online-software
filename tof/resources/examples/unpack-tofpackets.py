#! /usr/bin/env python

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
    
    packets      = gt.get_tofpackets(args.tofpacketfile)
     
    for pack in packets:
        print (pack)
        ev = gt.TofEvent.from_bytestream(pack.payload, 0)
        for k in ev.rbevents:
            # event header containing event id etc
            print (k.header)
            # get channel adcs
            k.get_channel_adc(1)
