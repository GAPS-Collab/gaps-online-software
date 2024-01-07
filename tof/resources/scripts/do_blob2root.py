#! /usr/bin/env python

"""
Apply blob2root to all the files in a certain folder
"""

import time
import re
import os
import shlex
import os.path
import subprocess as sub

from collections import defaultdict

if __name__ == '__main__':
    
    import argparse 

    from glob import glob
 
    parser = argparse.ArgumentParser(description='Apply the blob2root command to a directory, sorting the readout boards automatically.')
    parser.add_argument('--indir', type=str, default='',#default=[], nargs='+',
                        help='input directory with readout board blob files')
    args = parser.parse_args()
    files = sorted(glob(os.path.join(args.indir, '*.dat')))
    
    print (f'We found {len(files)} files')
    print (f'First filename is {files[0]}')
    print (files)

    pattern = re.compile('d20211222_(?P<board>[0-9]*)_data_(?P<id>1|2|3|4|5).dat')
    readout_group = defaultdict(lambda : [])
    for f in files:
        identifier = (pattern.search(f).groupdict())
        readout_group[int(identifier['board'])].append(f)

    print ('----------------------')
    for k in readout_group:
        print (f'--- {k} : {readout_group[k]}')
    #raise
    # compile commands
    ngroups = len(readout_group.keys())
    for i,k in enumerate(readout_group):
        command = './blob2root '
        for j in readout_group[k]:
            command += f'{j} '
        command = shlex.split(command)
        proc = sub.Popen(command).communicate()
        print (f'-- this was {i} of {ngroups} reaodut groups')
        time.sleep(20)   
