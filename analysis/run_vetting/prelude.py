#############################################################
# Run this with %run prelude.rc in a jupyter notebook
#############################################################


import gaps_online as go
import gaps_online.db as db
import gaps_online.telemetry as tel

import polars as pl
import numpy as np
import scipy.integrate as integrate
import HErmes as he
import HErmes.fitting as fit
import scipy.stats as st
import matplotlib
import re

from scipy.spatial.transform import Rotation as rot
from datetime import datetime, UTC, timezone
from glob import glob

from pathlib import Path
import dashi as d
d.visual()
import tqdm

import matplotlib.pyplot as plt
import charmingbeauty as cb
lo = cb.layout
cb.visual.set_style_default()

import re
get_ipython().system('export DJANGO_ALLOW_ASYNC_UNSAFE=1')
import os
os.environ['DJANGO_ALLOW_ASYNC_UNSAFE'] = '1'

# this is for gse machines which do not have latex installed
plt.rcParams.update({'text.usetex' : False})

# the speed of light in a tof paddle
C_LIGHT_PADDLE = 15.4; 

def get_ts_from_toffile(fname):
    """
    Get the timestamp from a .gaps.tof file
    """
    pattern = re.compile('Run[0-9]*_[0-9]*.(?P<tdate>[0-9_]*)')
    ts = pattern.search(str(fname)).groupdict()['tdate']
    #print (ts)
    ts = datetime.strptime(ts, '%y%m%d_%H%M%S')
    ts = ts.replace(tzinfo=timezone.utc)
    return ts

def get_ts_from_binfile(fname):
    """
    Get the timestamp from a .gaps.tof file
    """
    pattern = re.compile('RAW(?P<tdate>[0-9_]*).bin')
    ts = pattern.search(str(fname)).groupdict()['tdate']
    ts = datetime.strptime(ts, '%y%m%d_%H%M%S')
    ts = ts.replace(tzinfo=timezone.utc)
    return ts
    
def calc_rms(data):   
    """ root mean square calculation """
    return np.sqrt((data**2).sum()/len(data))

def get_t0(cfd_a, cfd_b, paddle_len):
    return 0.5*(cfd_a + cfd_b - (paddle_len/(10.0*C_LIGHT_PADDLE)))
    
def get_pos(cfd_a, t0):
    return (cfd_a - t0)*C_LIGHT_PADDLE*10.0 

def construct_plen_table():
    """
    Get the paddle lengths from the database
    """
    paddles = db.Paddle.objects.all()
    plen = dict()
    for pdl in paddles:
        plen[pdl.paddle_id] = pdl.length
    return plen

def get_hit_paddles_rbs(ev, paddles):
    """
    Get the hit paddle ids when looking at the 
    fired channels of the readout boards
    """
    hit_paddles = []
    for rbev in ev.rbevents:
        chnls = np.array(rbev.header.get_channels())
        chnls += 1
        for pdl in paddles:
            if pdl.rb_id == rbev.header.rb_id:
                if pdl.rb_chA in chnls:
                    hit_paddles.append((pdl.paddle_id, pdl.panel_id))
                    continue
    return hit_paddles

def get_hit_paddles(hits, paddles):
    """
    Get the hit paddles from the hit mask in 
    TofEventSummary or TofEvent 
    """
    hit_paddles = []
    for h in hits:
        for pdl in paddles:
            if pdl.dsi == h[0]:
                if pdl.j_ltb == h[1]:
                    ch = (pdl.ltb_chA, pdl.ltb_chB)
                    if sorted(ch) == sorted(h[2]):
                        hit_paddles.append((pdl.paddle_id, pdl.panel_id, (pdl.global_pos_x_l0, pdl.global_pos_y_l0, pdl.global_pos_z_l0)))
                        break
    return hit_paddles

def charge_dist(charges, bins):
    """
    """   

    def Landau(xs, scale, mu, eta):
        return scale*st.moyal.pdf(xs, loc=mu, scale=eta )

    fit = he.fitting.model.Model(Landau)

    fig = plt.figure(figsize=cb.layout.FIGSIZE_A4_LANDSCAPE)
    ax = fig.gca()
    h = d.factory.hist1d(charges, bins)

    spectral = h.bincenters, h.bincontent
    fit.startparams = (max(spectral[1]), 1000 ,1111.15)
    fit.add_data(h.bincontent, xs=h.bincenters, create_distribution=False)
    fit.fit_to_data(silent=True)
    h.line(filled=True, alpha=0.7,color='r')
    ax.plot(bins, fit(bins, *fit.best_fit_params), color='r', label='Landau fit')
    ax.set_xlabel('ccharge [pC]', loc='right')
    ax.set_ylabel('entries', loc='top')
    ax.legend()
    return fig

def get_rot(axis, theta):
    if axis == 'x':
        mat = np.array([[1,0,0],
                       [0,np.cos(theta),-np.sin(theta)],
                       [0,np.sin(theta), np.cos(theta)]])

    if axis == 'y':
        mat = np.array([[np.cos(theta),0 ,np.sin(theta)],
                       [0,1,0],
                       [-np.sin(theta), 0, np.cos(theta)]])

    if axis == 'z':
        mat = np.array([[np.cos(theta),-np.sin(theta),0],
                       [np.sin(theta),np.cos(theta),0],
                       [0,0, 1]])
    return mat

def get_binaries(unix_time_start, unix_time_stop, data_dir='/gaps_binaries/live/raw/ethernet'):
    # file format is something like RAW240712_094325.bin
    t_start = datetime.fromtimestamp(unix_time_start, UTC)
    t_stop = datetime.fromtimestamp(unix_time_stop, UTC)
    print(t_start)
    all_files = sorted([k for k in Path(f'{data_dir}').glob('*.bin')])
    print(f'-> Found {len(all_files)} files in {data_dir}')
    ts = [get_ts_from_binfile(f) for f in all_files]
    #print (ts[0])
    files = [f for f,ts in zip(all_files, ts) if t_start <= ts <= t_stop]
    ts = [get_ts_from_binfile(f) for f in files]
    print (f'-> Run duration {ts[-1] - ts[0]}')
    if files:
        print (f'-> Found {len(files)} files within range of {t_start} - {t_stop}')
        print (f'--> Earliest file {files[0]}')
        print (f'--> Latest file {files[-1]}')
    else:
        print (f'! No files have been found within {t_start} and {t_stop}!')
    

    #print(datetime.utcfromtimestamp(ts).strftime('%Y-%m-%d %H:%M:%S'))
    return files

#print (get_rot('x', np.pi/2))



