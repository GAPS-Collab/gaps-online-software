#! /usr/bin/env python 

"""
As an example, get PAMoniData from bin files and make a simple plot
"""

from pathlib import Path
from re import I 

import tqdm
import gondola as go
import polars as pl 
import matplotlib.pyplot as plt

# check gondola version
if not (go.get_version_minor() >= 12 and go.get_version_patch() >= 8):
    print(f'ERROR - got version {go.get_version_major()}.{go.get_version_minor()}.{go.get_version_patch()}')
    raise ImportError("gondola needs to be at least version 0.12.8!")

if __name__ == '__main__':
    
    import sys
    # have some files with metadata at location 
    files = Path(sys.argv[1])
    files = sorted(files.glob('*bin')) 
    moni  = go.monitoring.PAMoniDataSeries()
    moni.max_size = int(10e6)
    for f in tqdm.tqdm(files):
        moni.add_telemetryfile(str(f)) 
    # get a polars dataframe 
    df = moni.get_dataframe() 
    
    # the 'board_id' in the dataframe are the board id of the RB 
    # the PB with the respective temperature are connected to
    # For example, look at board 46
    df_board46 = df.filter(pl.col('board_id') == 46)
    
    fig  = plt.figure()
    ax   = fig.gca()
    # channel 1 on the PB 
    ax.scatter(df_board46['timestamp'], df_board46['temps1'])
    fig.savefig('pamonidataseries_example.png')

    

