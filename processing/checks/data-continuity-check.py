#! /usr/bin/env python 

"""
Check for processed L0 data if there are any gaps in the datastream
"""

from pathlib import Path
import tqdm
import pickle

import gondola as go

# check if gondola version satisfies required script version 
REQUIRED_VERSION = "0.12.25" 
if not go.version_at_least(REQUIRED_VERSION):
    print(f'ERROR - got version {go.get_version_major()}.{go.get_version_minor()}.{go.get_version_patch()}')
    raise ImportError(f"gondola needs to be at least version {REQUIRED_VERSION}! Please update your dependency (e.g. uv lock --upgrade")

if __name__ == '__main__':

    import argparse

    parser      = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--l0-dir', default=Path('/data/stoessl/flight/L0-remerged'),\
                        help='A directory with L0 (caraspace) files',\
                        type=Path,
                        )
    args = parser.parse_args() 
    input_fnames = []
    n_dir        = 0
    print (f'-> Looking for L0 files in {args.l0_dir}')
    for l0dir in tqdm.tqdm(args.l0_dir.glob('*'), desc='Gathering filenames...'):
        input_fnames.extend(l0dir.glob('*.gaps')) 
        n_dir += 1 

    input_fnames = sorted(input_fnames) 
    print (f'-> Found {len(input_fnames)} L0 files in total, distributed over {n_dir} directories!') 

    evids = []
    limit = 5e6 
    n     = 0
    n_pk  = 0
    # pickle the results so we don't overwhelm the memory
    for fname in tqdm.tqdm(input_fnames, desc='Checking data ...'): 
        reader = go.io.CRReader(str(fname)) 
        for frame in reader:
            if frame.has('TelemetryEvent'):
                gcutime = frame.get_first_gcutime()
                ev = frame.get_telemetryevent('TelemetryEvent') 
                evids.append((frame.get_first_gcutime(), ev.event_id, ev.tof.run_id))
                n += 1 
        if n >= limit:
            print (f'-> Write next .pickle file ({n_pk})')
            with open(f'/data/stoessl/dumps2/l0-evids-{n_pk}.pickle','wb') as f_out:
                pickle.dump(evids, f_out) 
            n_pk += 1 
            n    = 0 
            evids = [] 

#print (evids) 
