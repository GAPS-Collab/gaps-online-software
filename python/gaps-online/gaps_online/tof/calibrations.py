"""
Calibration related convenience functions
"""

cxx_api_loaded  = False
rust_api_loaded = False
try:
    import gaps_tof as gt
    cxx_api_loaded = True
except ImportError as err:
    print(f"Unable to load CXX API! {err}")

try:
    import go_pybindings as go
    RBCalibration = go.events.RBCalibration
    rust_api_loaded = True
except ImportError as err:
    print (f'Unable to load RUST API! {err}')

import matplotlib.pyplot as p
import numpy as np
import dashi as d
import re
import tqdm

from pathlib import Path

d.visual()

try:
    import charmingbeauty as cb
    FIGSIZE=cb.layout.FIGSIZE_A4  
except ImportError as e:
    print(f"Can't find charmingbeauty for nice looking plots! {e}")
    GOLDEN_RATIO = (1 + np.sqrt(5))/2.
    WIDTH_A4 = 5.78851 # inch
    FIGSIZE_A4_LANDSCAPE = (WIDTH_A4, WIDTH_A4/GOLDEN_RATIO)
    FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT = (WIDTH_A4, (WIDTH_A4/(2*GOLDEN_RATIO)))
    FIGSIZE_A4 = (WIDTH_A4, (WIDTH_A4/GOLDEN_RATIO) + WIDTH_A4)
    FIGSIZE=FIGSIZE_A4



def _plot_constants(rb_id, data,var='offset', bins=20):
    fig, axes = \
      p.subplots(5, 1, sharex=True, figsize=cb.layout.FIGSIZE_A4)# layout='constrained', sharex=True)
    fig.subplots_adjust(hspace=0)
    ylabel = 'counts'
    xpos   = 0.75
    fig.text(-0.05, 0.85, ylabel,\
             rotation=90,\
             transform=fig.transFigure,\
             size=14)
    fig.text(0.75, 0.9,\
            f'RB{rb_id:02}',\
             transform=fig.transFigure,\
             size=18) 
    for k in axes:
        k.spines['top'].set_visible(True)
        k.spines['right'].set_visible(True)
        k.grid(True)
    match var:
        case 'offset':
            xlabel = 'V Offsets'
            fig.text(xpos, 0.05,\
                     xlabel,\
                     transform=fig.transFigure,\
                     size=14)
            for ch in range(9):
                _const_plotter(axes, ch, data, bins=bins)
        case 'inc':
            xlabel = 'V Incs'
            fig.text(xpos, 0.05,\
                     xlabel,\
                     transform=fig.transFigure,\
                     size=14)
            for ch in range(9):
                _const_plotter(axes, ch, data, bins=bins)
        case 'dip':
            xlabel = 'V Dips'
            fig.text(xpos, 0.05,\
                     xlabel,\
                     transform=fig.transFigure,\
                     size=14)
            for ch in range(9):
                _const_plotter(axes, ch, data, bins=bins)
        case 'tbin':
            xlabel = 'T Bins'
            fig.text(xpos, 0.05,\
                     xlabel,\
                     transform=fig.transFigure,\
                     size=14)
            for ch in range(9):
                _const_plotter(axes, ch, data, bins=bins)

def _const_plotter(axes, ch, data, bins=20):

    if ch == 9:
        ax_j  = 4
        color = 'k'
        alpha = 1.0
        lw    = 1.5
        fill  = False
    elif ch % 2 == 0:
        ax_j  = int ((ch - 2) / 2)
        color = 'tab:red'
        alpha = 0.9
        lw    = 0.9
        fill  = False
    else:
        ax_j  = int ((ch - 1) / 2)
        color = 'tab:blue'
        alpha = 0.9
        lw    = 0.9
        fill  = True
    p.sca(axes[ax_j])
    h = d.factory.hist1d(data[ch], bins)
    h.line(color  = color,\
           alpha  = alpha,\
           filled = fill,\
           lw     = lw)

def plot_offsets(self, bins=20):
    data = self.v_offsets
    return _plot_constants(self.rb_id,data,var='offset', bins=bins)

def plot_dips(self, bins=20):
    data = self.v_dips
    return _plot_constants(self.rb_id,data,var='inc', bins=bins)

def plot_incs(self, bins=20):
    data = self.v_incs
    return _plot_constants(self.rb_id,data,var='dip', bins=bins)

def plot_tbins(self, bins=20):
    data = self.t_bin
    return _plot_constants(self.rb_id,data,var='tbin', bins=bins)

if cxx_api_loaded:
    # monkey patch the C++ API RBEvent
    gt.RBCalibration.plot_offsets = plot_offsets
    gt.RBCalibration.plot_dips    = plot_dips
    gt.RBCalibration.plot_incs    = plot_incs
    gt.RBCalibration.plot_tbins   = plot_tbins
    RBCalibration_cxx = gt.RBCalibration

## convenience functions
def load_calibrations(cali_dir : Path, load_event_data = False):
    """
    Load all calibrations stored in a certain directory and
    return a dictionary rbid -> RBCalibration

    # Arguments:

        * load_event_data : if True, also load the associated events
                            which went into the calculation of the
                            calibration constants.
    """
    pattern = re.compile('RB(?P<rb_id>[0-9]*)_')
    calib_files = [k for k in cali_dir.glob("*.tof.gaps")]
    calibs = dict()
    for fname in tqdm.tqdm(calib_files, desc="Loading calibration files"):
        fname = str(fname)
        try:
            rb_id = int(pattern.search(fname).groupdict()['rb_id'])
        except Exception as e:
            print(f'Failed to get RB ID from file {fname}')   
            continue
        cali = RBCalibration()
        cali.from_file(fname)
        calibs[rb_id] = cali
    
    return calibs


## convenience functions
def load_calibrations_cxx(cali_dir : Path, load_event_data = False):
    """
    DEPRECATED - this function is deprecated and we discourage using 
                 the CXX/pybind11 API! Use the RUST API instead!

    Load all calibrations stored in a certain directory and
    return a dictionary rbid -> RBCalibration

    # Arguments:

        * load_event_data : if True, also load the associated events
                            which went into the calculation of the
                            calibration constants.
    """
    pattern = re.compile('RB(?P<rb_id>[0-9]*)_')
    calib_files = [k for k in cali_dir.glob("*.tof.gaps")]
    calibs = dict()
    for fname in tqdm.tqdm(calib_files, desc="Loading calibration files"):
        fname = str(fname)
        try:
            rb_id = int(pattern.search(fname).groupdict()['rb_id'])
        except Exception as e:
            print(f'Failed to get RB ID from file {fname}')   
            continue
        calibs[rb_id] = RBCalibration_cxx.from_file(fname)
    
    return calibs
