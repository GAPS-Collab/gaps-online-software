#! /usr/bin/env python 

"""
Convert .tof.gaps files to .gaps files. That is the conversion from the 
bare TOF packet stream to caraspace compatible files
"""

import re
import tqdm
from pathlib import Path

import gaps_online as go

PATTERN = re.compile('Run(?P<runid>[0-9]*)_(?P<subrunid>[0-9]*).(?P<timestamp>([0-9_]*UTC)).(tofsum.gaps|tof.gaps|gaps)')

def get_timestamp(filename):
    ts = PATTERN.search(filename)
    if ts is None:
        raise  ValueError(f'Unable to extract timestamp from {filename}!')
    runid    = ts.groupdict()['runid']
    subrunid = ts.groupdict()['subrunid']
    ts       = ts.groupdict()['timestamp']
    return int(runid), int(subrunid), ts

if __name__ == '__main__':
    import argparse
    import sys

    parser = argparse.ArgumentParser(description='Walk over L0 data and transfer TofEvents into TofEventSummary (remove the waveforms)')
    parser.add_argument('indir', default='/data2/gaps/L0',\
                        help='A directory with telemetry binaries, as received from the telemetry stream',
                        type=Path
                        )
    parser.add_argument('-n','--nframes', type=int,\
                        default=-1, help='Limit readout to npackets, -1 for all packets (default)')
    parser.add_argument('-o','--outdir',\
                        help='Outdir for output .bin files',
                        type=Path,
                        default=None)
    parser.add_argument('-v','--verbose', action='store_true',\
                        help='More verbose output')
    args = parser.parse_args()
    
    infiles = args.indir.glob('*.tof.gaps')
    if len([k for k in infiles]) == 0:
        infiles = args.indir.glob('*.tofsum.gaps')
    # sort simply by subrun id (assuming this is for a single run)
    infiles = sorted([str(k) for k in infiles], key = lambda x : get_timestamp(x)[1])
    finished = False
    n_packs  = 0
    for f in  tqdm.tqdm(infiles, desc='Converting files...'):
        if finished:
            break 
        reader = go.io.TofPacketReader(f)
        cr_fname = args.outdir / Path(f).name.replace('.tof.gaps', '.gaps')
        runid,srunid,ts =  get_timestamp(str(cr_fname))
        #print (cr_fname)
        writer = go.io.CRWriter(str(args.outdir), runid, subrun_id=srunid, timestamp=ts)
        writer.set_mbytes_per_file(5000)
        for pack in reader:
            n_packs += 1
            frame = go.io.CRFrame()
            frame.put_tofpacket(pack, str(pack.packet_type))
            writer.add_frame(frame)
            if n_packs == args.nframes:
                finished = True
                break

