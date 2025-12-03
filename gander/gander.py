#! /usr/bin/env python

"""
A viewer for any type of file.
"""

import gondola as gon
import re
import sys
import tomllib 
import cmocean
import concurrent.futures as fut
import polars as pl

from enum import Enum
from copy import deepcopy as copy
from pathlib import Path
from loguru import logger
import pyvista as pv
from stpyvista import stpyvista
logger.add(sys.stdout, level="INFO")

import charmingbeauty as cb
import charmingbeauty.layout as lo
cb.set_style_default()

import matplotlib
from matplotlib import patches
import matplotlib.pyplot as plt
matplotlib.use('Agg')
import numpy as np
import polars as pl
import dashi as d
d.visual()

import tqdm

from copy import deepcopy as copy
# streamlit app!
import streamlit as st

from stqdm import stqdm

from gander_session import *

#------------------------------------------------

ELIGIBLE_RBS = [1,2,3,4,5,6,7,8,9,11,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,39,40,41,42,44,46]

#------------------------------------------------

# Retrieve the password from secrets
try:
    stored_password = st.secrets["password"]
except KeyError:
    st.error("Password not found in secrets.toml. Please configure the secrets file.")
    st.stop()

#------------------------------------------------

def emit_empty_session_state():
    """
    Prepare the session state dictionary
    """
    session_state = {
        'telemetry_connected'   : False,
        # update the app every interval, when 
        # connected to telemetry
        'update_every'          : 1,
        # app control 
        'use_dark_theme'        : True,
        'plot_theme_gaps'       : False,
        'mark_preliminary'      : False,
        'mplib_rcprams'         : copy(plt.rcParams),
        'write_to_disk'         : False,
        # plots and event data
        'waveform_figs'         : None,
        'no_plot_first_bins_wf' : False,
        'no_plot_first_bins_wf2': False,
        'no_plot_last_bins_wf'  : False,
        # input 
        'infiles'               : [],
        'files_loaded'          : [],
        'run_id'                : None, 
        'event_id'              : [],
        'load_moni_data'        : True,
        'moni_data'             : moni_data,
        'load_waveforms'        : False,
        #'reader'                : go.io.CRReader(DEFAULT_FILE),
        'current_event_in_file' : 0,
        #'current_run_path'      : DEFAULT_FILE,
        'tof_calib'             : calib,
        # event viewer          
        'ev_viewer_cache'       : [],
        'ev_viewer_cache_size'  : 1000,
        'ev_viewer_idx'         : 0,
        # cuts
        'apply_tracker_mask'    : False,
        'search_event_id'       : 0,
        # TOF analysis part
        'tof_analysis'          : gon.tof.TofAnalysis(),
        # TRK analysis part
        'trk_analysis'          : gon.tracker.analysis.TrackerAnalysis(),
        'reco'                  : gon.reconstruction.Reconstruction(),
        'trk_calibration'       : { 'mask' : None, 'pedestal'  : None,\
                                    'tf'   : None, 'cmn_noise' : None},
        'vtk_plotter'           : pv.Plotter(window_size=[800, 600], off_screen=True)

    }
    return session_state

#################################################

def reset_session_state():
    session_state = emit_empty_session_state()
    for k in session_state:
        st.session_state[k] = session_state[k]

#################################################

def read_config(toml_filepath : Path):
    with open(toml_filepath, 'rb') as toml_file:
        config_data = tomllib.load(toml_file)
    return config_data

#################################################

BINS_TIMING        = np.linspace(-100, 300, 70)
BINS_NHIT          = np.arange(-0.5, 30.5, 1)
BINS_BETA          = np.linspace(-0.1, 4, 70)
DEFAULT_FILE       = '/data2/gaps/L0/9125/Run9125_263.241213_163347UTC.gaps'
CAMPAIGN_MONI_DATA = '/data2/gaps/L0/all-moni/'

MERGED_EVENT_TYPES     = [\
    "TelemetryPacketType.NoGapsTriggerEvent",
    "TelemetryPacketType.BoringEvent",
    "TelemetryPacketType.InterestingEvent",
    "TelemetryPacketType.NoTofDataEvent"]


# bins
PADDLE_PEAK_BINS   = np.linspace(0,300,70)
PADDLE_CHARGE_BINS = np.linspace(0,50,70)
PADDLE_TIMING_BINS = np.linspace(0,500,70)
PADDLE_BL_BINS     = np.linspace(-5,5,70)
PADDLE_BLRMS_BINS  = np.linspace(0,5,70)

#################################################

class EventType(Enum):
    Unknown = 0
    Merged  = 10
    Tof     = 20
    Calib   = 30

#################################################

def check_password():
    """Returns `True` if the password is correct."""
    def password_entered():
        """Checks password and updates state."""
        if (st.session_state["password"] == stored_password):
            st.session_state["password_correct"] = True
            del st.session_state["password"]  # don't store password
        else:
            st.session_state["password_correct"] = False

    if "password_correct" not in st.session_state:
        # First run, show input for password.
        st.text_input(
            "Password", type="password", on_change=password_entered, key="password"
        )
        return False
    elif not st.session_state["password_correct"]:
        # Password not correct, show input + error.
        st.text_input(
            "Password", type="password", on_change=password_entered, key="password"
        )
        st.error("😕 Incorrect password")
        return False
    else:
        # Password correct.
        return True

#------------------------------------------------

def add_preliminary(fig):
    """
    Add a "Preliminary" Stamp to a plot
    """
    ax = fig.gca()
    ax.text(0.25, 0.25, 'Preliminary', fontsize=36, color='tab:red',  alpha=0.5, rotation=34, transform=fig.transFigure)
    
#------------------------------------------------

def set_plot_theme_gaps():
    if st.session_state.plot_theme_gaps:
        plt.rcParams.update(st.session_state.mplib_rcprams)
        logger.info("Will use official gaps theme!")
        st.session_state.use_dark_theme = False
        style_sheet = '/srv/gaps/gaps-online-software/resources/stylesheets/gaps-approved-style.mplstyle'
        plt.style.use(style_sheet)
    else:
        plt.rcParams.update(st.session_state.mplib_rcprams)

#------------------------------------------------

def set_plot_theme_streamlit_dark():
    if st.session_state.use_dark_theme:
        plt.rcParams['text.color']        = '#FAFAFA'
        plt.rcParams['axes.labelcolor']   = '#FAFAFA'
        plt.rcParams['xtick.color']       = '#FAFAFA'
        plt.rcParams['ytick.color']       = '#FAFAFA'
        plt.rcParams['text.color']        = '#FAFAFA'
        plt.rcParams['axes.labelcolor']   = '#FAFAFA'
        plt.rcParams['xtick.color']       = '#FAFAFA'
        plt.rcParams['ytick.color']       = '#FAFAFA'
        plt.rcParams['grid.color']        = '#FAFAFA'
        plt.rcParams['axes.edgecolor']    = '#FAFAFA'
        plt.rcParams['savefig.facecolor'] = '#0E1117'
        plt.rcParams['axes.facecolor']    = '#0E1117'
    elif st.session_state.plot_theme_gaps:
        pass
    else:
        plt.rcParams.update(st.session_state.mplib_rcprams)

#------------------------------------------------

##################################################
# Event viewer 

def _event_viewer_data() -> dict:
    return {
            'merged_event'            : None,
            'packet_type'             : None,
            'tof_xy'                  : None,
            'gaps_xy'                 : None,
            'gaps_xz'                 : None,
            'gaps_yz'                 : None,
            'tof_xy_all'              : None,
            'tof_cbe'                 : None,
            'tof_cor'                 : None,
            'tof_event'               : None,
            'tof_hits'                : [],
            'trk_hits'                : [],
            'trk_layers'              : [],
            'trk_pointcloud'          : [],
            'n_trk_hits_masked'       : None,
            'n_trk_hits_no_mask_info' : None}

##################################################

@st.fragment
def next_event(session):
    if st.session_state.ev_viewer_idx != len(st.session_state.ev_viewer_cache):
        st.session_state.ev_viewer_idx += 1

##################################################

@st.fragment
def prev_event(session):
    if st.session_state.ev_viewer_idx != 0:
        st.session_state.ev_viewer_idx -= 1

##################################################

def add_arrow(ax, p_arrow_h, p_arrow_t, c0=0, c1=1, fc = 'w', ec = 'w'):
    x_0 = p_arrow_h[c0]/10
    y_0 = p_arrow_h[c1]/10
    dx  = (p_arrow_t[c0] - p_arrow_h[c0])/10
    dy  = (p_arrow_t[c1] - p_arrow_h[c1])/10
    #print(x_0, y_0, dx, dy)
    length = 10
    alpha = 1
    width = 0.5
    head_starts_at_zero = True
    head_width = width*10
    head_length = length*1.2
    shape = 'full'
    arrow_params={'length_includes_head':False,\
     'shape':shape,\
     'head_starts_at_zero':head_starts_at_zero}
    #print (head_width)
    #print (x_0,y_0)
    arr = ax.arrow(x_0, y_0, dx*length,\
     dy*length, fc=fc, ec=ec,\
     alpha=alpha, width=width,\
     head_width=head_width,\
     head_length=head_length,\
     **arrow_params)

##################################################

@st.fragment
def create_event_plots(no_plot_first_bins_wf=False) -> dict:
    """
    Plots from the cahced events for an interactive view

    # Keyworkd Args:
        * no_plot_first_bins_wf : If True, don't plot the first bins in a waveform, since 
          these might just spikes.
    """
    if len(st.session_state.ev_viewer_cache) == 0:
        return dict()

    ev_filename , ptype, (ev, wf_ev) = st.session_state.ev_viewer_cache[st.session_state.ev_viewer_idx]
    data = _event_viewer_data()
    n_trk_hits_masked       = None 
    n_trk_hits_no_mask_info = None
    telemetry = False
    if isinstance(ev, gon.events.TelemetryEvent):
        ev_tof = ev.tof
        data['tof_event'] = ev_tof
        data['merged_event'] = ev
        data['packet_type']     = ptype
        telemetry = True
    else:
        ev_tof = ev
        data['tof_event'] = ev_tof
        if ev_tof.hits:
            paddle_style            = {'edgecolor' : 'w', 'lw' : 1.0}
            data['tof_xy']    , ax  = gon.visual.tof.tof_projection_xy(event=ev_tof, cmap=matplotlib.colormaps['seismic'])
            data['tof_cbe']   , ax2 = gon.visual.tof.unroll_cbe_sides  (event=ev_tof, cmap=matplotlib.colormaps['seismic'], paddle_style = paddle_style)
            data['tof_cor']   , ax3 = gon.visual.tof.unroll_cor        (event=ev_tof, cmap=matplotlib.colormaps['seismic'], paddle_style = paddle_style)
            data['tof_xy_all'], data['tof_xz_all'], data['tof_yz_all'] \
                    = gon.visual.tof.tof_2dproj(event=ev_tof, cmap=matplotlib.colormaps['seismic'])
        data['tof_hits'] = ev_tof.hits
    
    if telemetry:
        if ev.tof.hits:
            paddle_style            = {'edgecolor' : 'w', 'lw' : 1.0}
            data['tof_xy']    , ax  = gon.visual.tof.tof_projection_xy(event=ev_tof, cmap=matplotlib.colormaps['seismic'])
            data['tof_cbe']   , ax2 = gon.visual.tof.unroll_cbe_sides  (event=ev_tof, cmap=matplotlib.colormaps['seismic'], paddle_style = paddle_style)
            data['tof_cor']   , ax3 = gon.visual.tof.unroll_cor        (event=ev_tof, cmap=matplotlib.colormaps['seismic'], paddle_style = paddle_style)
            data['tof_xy_all'], data['tof_xz_all'], data['tof_yz_all'] \
                    = gon.visual.tof.tof_2dproj(event=ev_tof, cmap=matplotlib.colormaps['seismic'])
        data['tof_hits'] = ev.tof.hits
        for h in ev.tracker:
            data['trk_hits'].append(h)
        ## FIXME - the pointcloud needs masking
        data['trk_pointcloud']          = ev.tracker_pointcloud 
        data['n_trk_hits_masked']       = n_trk_hits_masked
        data['n_trk_hits_no_mask_info'] = n_trk_hits_no_mask_info
        #data['trk_plots'] = plot_tracker(data['trk_hits'], strip_dict)
        if wf_ev is not None:
            calib = st.session_state.tof_calib 
            if no_plot_first_bins_wf:
                data['waveform_figs'], __ = gon.visual.tof.plot_waveforms(wf_ev, calib=calib, with_hits=True, skip_bins=10)  
            else:
                data['waveform_figs'], __ = gon.visual.tof.plot_waveforms(wf_ev, calib=calib, with_hits=True)  
    return data

#    session['trk_hits'] = []
#    for h in session['tel_ev'].tracker_v2:
#        session['trk_hits'].append(h)
#    tracker_hits = session['tel_ev'].tracker_v2
#    if session['apply_tracker_mask']:
#        tracker_hits = []
#        for h in session['tel_ev'].tracker_v2:
#            mask = go.db.TrackerStripMask.filter(strip_id=h.strip_id, mask_name=session['tracker_mask'])
#            if len(mask) == 0:
#                print (f'!WARN - no mask for strip {h.strip_id} found!')
#                tracker_hits.append(h)
#            elif len(mask) == 1:
#                if mask.active:
#                    tracker_hits.append(h)
#            else:
#                print (f'!WARN - ambiguos mask for strip {h.strip_id} found!')
#                tracker_hits.append(h)

##################################################

@st.fragment
def file_loader(f, event_type,\
                merged_event_types   = MERGED_EVENT_TYPES,
                write_to_basket      = False,
                n_event_samples      = 10,
                search_event_id      = 0,
                tof_analysis_kwargs  = {},
                trk_analysis_kwargs  = {},
                reco_analysis_kwargs = {}) -> (gon.tof.TofAnalysis, gon.tracker.analysis.TrackerAnalysis, list):
    """
    Run over a file and extract variables to fill the histograms
    in the assigned analyisises

    # Arguments:
        f                     : filename
        merged_event_types    : If event_type == EventType.Merged, we can furhter
                                select the sub-types of the merged packets.
                                Default = all

    # Returns:
        (tof_analysis,[(filename, packet_type, event),..]) : The events are sampled from the 
                                                             the beginning of the file
    """
    # detect automagically if the source is a file or a socket 
    stream_mode = False
    if str(f).startswith('tcp') and st.session_state.telemetry_active:
        stream_mode = True


    event_samples = []
    is_tof_file   = False
    if stream_mode:
        reader   = gon.io.streamer.TelemetrySocketReader(f)
        pass
    elif str(f).endswith('tof.gaps'):
        reader   = gon.io.TofPacketReader(f)
        is_tof_file = True
    else:
        reader   = gon.io.CRReader(f)
        # set the calibration files 
        reader.set_tracker_calibrations_from_fnames(mask        = st.session_state.trk_calibration['mask'],
                                                    pedestal    = st.session_state.trk_calibration['pedestal'],
                                                    transfer_fn = st.session_state.trk_calibration['tf'],
                                                    cmn_noise   = st.session_state.trk_calibration['cmn_noise'])
    if write_to_basket:
        if stream_mode:
            # FIXME
            runid, srunid, ts = 0,0,0
        else:
            f_rundata = gon.io.get_rundata_from_file(str(f)) 
            runid, srunid, ts = f_rundata['run'], f_rundata['subrun'], f_rundata['utctime']
        writer = gon.io.CRWriter('/prestaging/basket',runid, subrun_id=srunid, timestamp=ts)
        logger.info(f'Will use writer : {writer}')
    # FIXME - this whole 'do_tof_analysis' stuff needs to go
    #if do_tof_analysis:
    tof_analysis = None
    if tof_analysis_kwargs:
        if tof_analysis_kwargs['active']:
            tof_analysis = gon.tof.TofAnalysis(**tof_analysis_kwargs) 
            tof_analysis.cuts.clear_stats()
    trk_analysis = None
    if trk_analysis_kwargs:
        if trk_analysis_kwargs['active']:
            trk_analysis = gon.tracker.analysis.TrackerAnalysis(**trk_analysis_kwargs)     
    reco_analysis = None
    if reco_analysis_kwargs:
        if reco_analysis_kwargs['active']:
            reco_analysis = gon.reconstruction.Reconstruction(**reco_analysis_kwargs)
    nframes = 0
    exclude_event_types = [k for k in MERGED_EVENT_TYPES if k not in merged_event_types] 
    
    for frame in reader:
        if is_tof_file:
            pack = frame
            if pack.packet_type == gon.packets.TofPacketType.TofEvent:
                ev_tof = gon.events.TofEvent.unpack(pack)
                ev_tof.move_hits()
            else:
                continue
        else:
            match event_type:
                case EventType.Unknown:
                    raise ValueError("Unknown event type!")
                case EventType.Merged:
                    try:
                        ev = frame.get_telemetryevent(always_exclude=exclude_event_types)
                        m_type = ev.header.packet_type 
                    except Exception as e:
                        logger.warning(f'Merged event is corrupt! {e}')
                        continue
                    ev_tof = ev.tof
                case EventType.Tof:
                    m_type = 'TofEvent'

                    if not 'PacketType.TofEvent' in frame.index:
                        if 'PacketType.TofEventSummary' in frame.index:
                            m_type = 'TofEventSummary'
                        else:
                            continue
                    # we also do have TofEventSummary data (.tofsum.gaps)
                    # let's double check
                    
                    try:
                        if m_type == 'TofEventSummary':
                            #print (frame)
                            ev_tof = frame.get_tofeventsummary('PacketType.TofEventSummary')
                            ev = ev_tof
                        else:
                            ev = frame.get_tofevent('PacketType.TofEvent')
                    except Exception as e:
                        logger.warning(f'TofEvent is corrupt! {e}')
                        continue
                    #if m_type == 'TofEvent':
                    #    ev_tof = ev.get_summary()
                    #raise ValueError("TofEvent analysis not implemented yet, use merged event!")
                case _:
                    raise ValueError("Unable to digest this input!")

        # this will actually only add events when 
        # it is active
        trk_passed = False
        tof_passed = False
        if search_event_id != 0:
            if ev_tof.event_id != search_event_id:
                continue
        if tof_analysis is not None:
            if tof_analysis.cuts.accept(ev_tof):
                #tof_analysis.add_event(ev_tof)
                tof_passed = True
            if tof_analysis.skip_mangled and ev_tof.status == gon.events.EventStatus.AnyDataMangling:
                tof_analysis.n_mangled += 1
                tof_passed = False
            if tof_analysis.skip_timeout and ev_tof.status == gon.events.EventStatus.EventTimeOut:
                tof_analysis.n_timed_out += 1
                tof_passed = False
        else:
            tof_passed = True
        if trk_analysis is not None:
            #masked_hits = trk_analysis.mask_hits(ev)
            if trk_analysis.cuts.accept(ev.tracker):
                trk_passed = True
        else:
            trk_passed = True

        if tof_passed and trk_passed:
            if trk_analysis is not None:
                trk_analysis.add_event(ev)
            if tof_analysis is not None:
                tof_analysis.add_event(ev_tof)
            if write_to_basket:
                writer.add_frame(frame)
            nframes += 1
        else:
            nframes += 1
            continue

        if len(event_samples) != n_event_samples:
            # let's also get the tof event with the waveforms for this
            wf_event = None
            if is_tof_file:
                wf_event = ev_tof
                ev       = ev_tof
                m_type   = 'TofEvent'
            elif 'PacketType.TofEvent' in frame.index:
                wf_event = frame.get_tofevent('PacketType.TofEvent') 
            event_samples.append((f,m_type,(ev, wf_event)))
        
        if reco_analysis is not None:
            reco =reco_analysis.reco(ev)
            if nframes % 100 == 0:
                print (nframes)
        if stream_mode:
            if nframes >= st.session_state.telemetry_chunk_size:
                break            
    return (tof_analysis, trk_analysis,reco_analysis, event_samples)

##-----------------------------------------
#
#@st.fragment 
#def clear_loaded_runs():
#    st.session_state.event_cache = []
#
##-----------------------------------------

@st.fragment
def clear_analysis():
    """
    Reset the low level tof analysis
    """
    st.session_state.tof_analysis          = gon.tof.TofAnalysis()
    st.session_state.tof_analysis.finished = False
    st.session_state.trk_analysis          = gon.tracker.analysis.TrackerAnalysis()
    st.session_state.trk_analysis.finished = False
    st.session_state.reco                  = gon.reconstruction.Reconstruction()
    st.session_state.reco.finished         = False
    st.session_state.ev_viewer_cache       = []
    st.session_state.moni_data             = moni_data

#-----------------------------------------

@st.fragment
def clear_cuts():
    st.session_state.tof_analysis.cuts = gon.tof.analysis.TofCuts()
    st.session_state.trk_analysis.cuts = gon.tracker.analysis.TrackerCuts()

#-----------------------------------------

@st.fragment
def telemetry_loader(address,
                     update_interval,
                     nbins):
    pass


#-----------------------------------------

@st.fragment
def load_run(event_type         = EventType.Merged,\
             merged_event_types = MERGED_EVENT_TYPES,\
             nbins              = 70,\
             ncpus              = 20):
    """
    Load a complete run
    """
    nfiles   = len(st.session_state.infiles)
    # load the files!
    write_to_basket = st.session_state.write_to_disk
    st.session_state.files_loaded = copy(st.session_state.infiles)
    tof_analysis_kwargs = {'skip_mangled'  : copy(st.session_state.tof_analysis.skip_mangled),
                           'skip_timeout'  : copy(st.session_state.tof_analysis.skip_timeout),
                           'beta_analysis' : True,
                           'nbins'         : nbins,
                           'use_offsets'   : copy(st.session_state.tof_analysis.use_offsets),
                           'cuts'          : st.session_state.tof_analysis.cuts.copy(),
                           'active'        : copy(st.session_state.tof_analysis.active),
                           'pid_outer'     : copy(st.session_state.tof_analysis.pid_outer),
                           'pid_inner'     : copy(st.session_state.tof_analysis.pid_inner)}
    trk_analysis_kwargs  = st.session_state.trk_analysis.emit_kwargs()
    reco_analysis_kwargs = {'active'       : copy(st.session_state.reco.active)}
    search_event_id = copy(st.session_state.search_event_id)
    if ncpus > 1:
        with fut.ThreadPoolExecutor(max_workers=ncpus) as exe:
            future_to_ana = {exe.submit(file_loader, f, event_type,\
                                        merged_event_types   = merged_event_types,\
                                        tof_analysis_kwargs  = tof_analysis_kwargs,\
                                        trk_analysis_kwargs  = trk_analysis_kwargs,
                                        reco_analysis_kwargs = reco_analysis_kwargs,
                                        search_event_id      = search_event_id,
                                        write_to_basket      = write_to_basket) : f for f in st.session_state.infiles}
            for future in stqdm(fut.as_completed(future_to_ana), desc="Loading run data, this might take a while...", total = len(st.session_state.infiles)):
                try:
                    tof_a, trk_a, event_data = future.result()
                    if tof_a is not None:
                        st.session_state.tof_analysis += tof_a
                    if trk_a is not None:
                        st.session_state.trk_analysis += trk_a
                    st.session_state.ev_viewer_cache.extend(event_data)
                except Exception as e:
                    logger.warning(f"{future_to_ana[future]} caused exception {e}!")
                
    else:
        for f in stqdm(st.session_state.infiles, desc="Loading run data, this might take a while...", total = len(st.session_state.infiles)):
            if st.session_state.load_moni_data:
                if str(f).endswith('tof.gaps'):
                    st.session_state.moni_data['mtb'] .add_toffile(str(f))
                    st.session_state.moni_data['rb']  .add_toffile(str(f))
                    st.session_state.moni_data['pa']  .add_toffile(str(f))
                    st.session_state.moni_data['evbh'].add_toffile(str(f))
                    st.session_state.moni_data['ltb'] .add_toffile(str(f))
                else:    
                    st.session_state.moni_data['mtb'] .add_crfile(str(f))
                    st.session_state.moni_data['rb']  .add_crfile(str(f))
                    st.session_state.moni_data['pa']  .add_crfile(str(f))
                    st.session_state.moni_data['evbh'].add_crfile(str(f))
                    st.session_state.moni_data['ltb'] .add_crfile(str(f))
            result = file_loader(f, event_type,
                                 merged_event_types   = merged_event_types,
                                 search_event_id      = search_event_id,
                                 tof_analysis_kwargs  = tof_analysis_kwargs,\
                                 trk_analysis_kwargs  = trk_analysis_kwargs,
                                 reco_analysis_kwargs = reco_analysis_kwargs,\
                                 write_to_basket = write_to_basket)
            if result[0] is not None:
                st.session_state.tof_analysis += result[0]
            if result[1] is not None:
                st.session_state.trk_analysis += result[1]
            if result[2] is not None:
                st.session_state.reco         += result[2]
            st.session_state.ev_viewer_cache.extend(result[3])
            #if st.session_state.abort_run_loader:
            #    st.session_state.abort_run_loader = False
            #    break
    with st.spinner("Creating plots...", show_time=True):
        st.session_state.tof_analysis.finish()
        st.session_state.trk_analysis.finish()
        st.session_state.reco.finish()
    #st.session_state.tof_analysis_done = True



#@st.fragment
#def create_timing_distributions(infiles,
#                                load_bar,
#                                pid0=None,
#                                pid1=None,
#                                #result=ST_SESSION_PLOTS,
#                                bins=np.linspace(-100, 300, 70),
#                                beta_analysis=True):
#    """
#    Get missing HG hits from a reader 
#    as provided by the session dict
#    """
#    print ('-> create_timing_distributions')
#    # FIXME - make this better, this should not 
#    # reset other plots than the timing plots
#    result       = create_new_session_plots()
#    tofevent_key = 'PacketType.TofEvent'
#    nhits        = 0
#    infiles      = st.session_state.infiles
#    print (infiles)
#    if infiles:
#        infiles= infiles[:10]
#    nfiles = len(infiles)
#    f_cntr = 0
#    print (f'-> Picking paddles for {pid0} and {pid1}')
#    print (f'-> Will loop over {nfiles} files!')
#    for f in tqdm.tqdm(infiles, desc="Creating timing distributions...", total = len(infiles)):
#        load_bar.progress(f_cntr/nfiles, 'Creating timing distributions...')
#        f_cntr += 1
#        reader  = go.io.CRReader(str(f))
#        for frame in reader:
#            if tofevent_key in frame.index:
#                # new API :)
#                tof_ev = frame.get_tofevent(tofevent_key)
#                ev = tof_ev.get_summary()
#                #print (ev)
#                if ev.status == go.events.EventStatus.AnyDataMangling:
#                    continue
#                #missing = [k for k in ev.get_missing_paddles_hg(mapping) if not k in dead_paddles]
#                hits = ev.hits
#                # fill the paddle related histograms
#                for h in hits:
#                    #result['paddle_plots']['charge2d'] : d.histogram.hist2d((PADDLE_CHARGE_BINS, PADDLE_CHARGE_BINS)),
#                    result['paddle_plots'][h.paddle_id]['amp_a'   ].fill(np.array([h.peak_a]))  
#                    result['paddle_plots'][h.paddle_id]['amp_b'   ].fill(np.array([h.peak_b]))  
#                    result['paddle_plots'][h.paddle_id]['time_a'  ].fill(np.array([h.time_a]))  
#                    result['paddle_plots'][h.paddle_id]['time_b'  ].fill(np.array([h.time_b]))  
#                    result['paddle_plots'][h.paddle_id]['bl_a'    ].fill(np.array([h.baseline_a]))  
#                    result['paddle_plots'][h.paddle_id]['bl_b'    ].fill(np.array([h.baseline_b]))  
#                    result['paddle_plots'][h.paddle_id]['bl_a_rms'].fill(np.array([h.baseline_a_rms]))  
#                    result['paddle_plots'][h.paddle_id]['bl_b_rms'].fill(np.array([h.baseline_b_rms]))  
#
#                if not beta_analysis:
#                    continue
#                #outer_h = sorted([h for h in hits if h.paddle_id > 60 and h.paddle_id < 73], key=lambda x: x.t0)
#                #inner_h = sorted([h for h in hits if h.paddle_id < 13], key=lambda x: x.t0)
#                if pid0 is None:
#                    outer_h = sorted([h for h in hits if h.paddle_id > 60], key=lambda x: x.t0)
#                else:
#                    outer_h = sorted([h for h in hits if h.paddle_id == pid0], key=lambda x:x.t0)
#                if pid1 is None:
#                    inner_h = sorted([h for h in hits if h.paddle_id < 61], key=lambda x: x.t0)
#                else:
#                    inner_h = sorted([h for h in hits if h.paddle_id == pid1], key=lambda x: x.t0)
#
#                #outer_h = sorted([h for h in hits if h.paddle_id > 60], key=lambda x: x.t0)
#                #inner_h = sorted([h for h in hits if h.paddle_id < 61], key=lambda x: x.t0)
#                #if len(hits) > 2:
#                #    continue
#                if inner_h and outer_h:
#                    first_hit = sorted([h for h in hits], key=lambda x: x.phase_delay)
#                    last_hit  = first_hit[-1].phase_delay
#                    first_hit = first_hit[0].phase_delay
#                    diff_h  = inner_h[0].t0 - outer_h[0].t0 
#                    #if diff_h > 50:
#                    #    print (inner_h[0])
#                    #    print (outer_h[0])
#                    #    print (inner_h[0].phase_delay)
#                    #    print (outer_h[0].phase_delay)
#                    #    print (inner_h[0].phase)
#                    #    print (outer_h[0].phase)
#                    #    print (inner_h[0].phase - outer_h[0].phase)
#                    #    print (inner_h[0].phase - outer_h[0].phase < -np.pi/2)
#                    #    print (inner_h[0].phase - outer_h[0].phase > np.pi/2)
#                    #    print (diff_h)
#                    #    h0_foo = inner_h[0].t0_uncorrected + inner_h[0].cable_delay
#                    #    h1_foo = outer_h[0].t0_uncorrected + outer_h[0].cable_delay
#                    #    t_shift = 50*(inner_h[0].phase - outer_h[0].phase)/(2*np.pi)
#                    #    print (t_shift)
#                    #    print (h0_foo - h1_foo + t_shift)
#                    #    diff_h -= 50
#
#                    #    raise
#                    #if diff_h < -50:
#                    #    print (inner_h[0])
#                    #    print (outer_h[0])
#                    #    print (inner_h[0].phase_delay)
#                    #    print (outer_h[0].phase_delay)
#                    #    print (inner_h[0].phase)
#                    #    print (outer_h[0].phase)
#                    #    print (inner_h[0].phase - outer_h[0].phase < -np.pi/2)
#                    #    print (inner_h[0].phase - outer_h[0].phase > np.pi/2)
#                    #    print (diff_h)
#                    #    h0_foo = inner_h[0].t0_unocrrected + inner_h[0].cable_delay
#                    #    h1_foo = outer_h[0].t0_uncorrected + outer_h[0].cable_delay
#                    #    t_shift = 50*(inner_h[0].phase - outer_h[0].phase)/(2*np.pi)
#                    #    print (h0_foo - h1_foo + t_shift)
#                    #    diff_h += 50
#                    #    
#                    #    raise
#                    phase_0 = hits[0].phase
#                    h0_foo = inner_h[0].t0_uncorrected + inner_h[0].cable_delay
#                    h1_foo = outer_h[0].t0_uncorrected + outer_h[0].cable_delay
#                    #p_shift = inner_h[0].phase - outer_h[0].phase 
#                    #p_shift = inner_h[0].phase - outer_h[0].phase 
#                    #p0 = inner_h[0].phase - phase_0
#                    #p1 = inner_h[1].phase - phase_0
#                    #if p0 < -np.pi/2:
#                    #    p_shift += 2*np.pi
#                    #if p_shift > np.pi/2:
#                    #    p_shift -= 2*np.pi
#                    #t_shift = 50*(inner_h[0].phase - outer_h[0].phase)/(2*np.pi)
#                    #t_shift = 50*p_shift/(2*np.pi)
#                    #diff_h = h0_foo -h1_foo - t_shift
#                    beta = (inner_h[0].distance(outer_h[0])/1000)/(diff_h*1e-9)/299792458
#                    #print (beta)
#                    if beta < 0:
#                        beta = -1*beta
#                    ##if beta < 0.3:
#                    phase_delay = inner_h[0].phase_delay - outer_h[0].phase_delay
#                    #if beta < 0.3:
#                        #if last_hit - first_hit > 18:
#                        #    tof_elena(hits)
#                        #    raise
#                    if True:
#                    #if (phase_delay > 20 or phase_delay < -20):
#                        #    if inner_h[0].phase_delay - outer_h[0].phase_delay > 40:
#                        result['histo_beta'].fill(np.array([beta])) 
#                        result['last_pd_outer'] = outer_h[0].phase_delay
#                        result['last_pd_inner'] = inner_h[0].phase_delay
#                        for wf in tof_ev.waveforms:
#                            if wf.paddle_id == outer_h[0].paddle_id:
#                                rbid = wf.rb_id
#                                for rbev in tof_ev.rbevents:
#                                    if rbev.header.rb_id == rbid:
#                                        result['wf_outer'] = rbev.get_waveform(8)
#                            if wf.paddle_id == inner_h[0].paddle_id:
#                                rbid = wf.rb_id
#                                for rbev in tof_ev.rbevents:
#                                    if rbev.header.rb_id == rbid:
#                                        result['wf_inner'] = rbev.get_waveform(8)
#                        #print (ev)
#                        result['histo_t_diff_fst'].fill(np.array([last_hit - first_hit]))
#                        result['histo_nhit'].fill(np.array([len(hits)]))
#                        result['histo_t_diff'  ].fill(np.array([diff_h]))
#                        result['histo_t_inner' ].fill(np.array([inner_h[0].t0]))
#                        result['histo_t_outer' ].fill(np.array([outer_h[0].t0]))
#                        result['histo_dist'].fill(np.array([distance(inner_h[0], outer_h[0])/1000]))
#                        
#                        result['histo_pdelay'].fill(np.array([phase_delay]))
#                        result['histo_ph_out'].fill(np.array([outer_h[0].phase_delay]))
#                        result['histo_ph_in'].fill(np.array([inner_h[0].phase_delay]))
#
#                        result['histo_hit_pid'].fill(np.array([inner_h[0].paddle_id]))
#                        result['histo_hit_pid'].fill(np.array([outer_h[0].paddle_id]))
#    ST_SESSION_PLOTS = result
#    return result                

#def tof_elena(hits):
#    phase_0 = hits[0].phase
#    for h in hits:
#        phi_shift = h.phase - phase_0
#        while phi_shift < -np.pi/2:
#            phi_shift += 2.0*np.pi
#        while phi_shift > np.pi/2:
#            phi_shift -= 2.0*np.pi
#        t_shift    = 50.0*phi_shift/(2.0*np.pi);
#        print (h)
#        print(t_shift)
#        print (t_shift + h.cable_delay)

# this dubs as the __name__ == '__main__' hook
if check_password():
   
    config = read_config('config.toml') 
    # setup calibration and mappings - use latest calib as default calibration
    # for now
    # steamlit app
    calibs = [k for k in reversed(sorted(Path(config['data']['tof_calib']).glob('25*')))]
    #print (calibs)
    #raise
    calib = gon.calibration.load_rb_calibrations(calibs[0])
   
    moni_data = {'mtb'   : gon.monitoring.MtbMoniDataSeries(),
                 'rb'    : gon.monitoring.RBMoniDataSeries(),
                 'pa'    : gon.monitoring.PAMoniDataSeries(),
                 'pb'    : gon.monitoring.PBMoniDataSeries(),
                 'evbh'  : gon.monitoring.EventBuilderHBSeries(),
                 'mtbhb' : gon.monitoring.MasterTriggerHBSeries(),
                 'dshb'  : gon.monitoring.DataSinkHBSeries(),
                 'ltb'   : gon.monitoring.LTBMoniDataSeries()}
    
    session_state = emit_empty_session_state()
    
    def configure_app():
        logger.info('Configuring app!')
        st.set_page_config(page_title="GAPS event view event {session['event_id}",
                           page_icon=":balloon:",
                           layout="wide")
    # app configure has to be the first thing
    configure_app()
    for k in session_state:
        if not k in st.session_state:
            st.session_state[k] = session_state[k]

    # dark theme is default theme 
    set_plot_theme_streamlit_dark()

    # page layout, individual pages for different 
    # functionality
    @st.fragment
    def page_run():
        """
        General run overview. Present files and .toml files of the current run
        """
        if st.session_state.files_loaded:
            st.subheader('Loaded data - Run statistic')
            with st.expander(f"Loaded {len(st.session_state.files_loaded)} files!"):
                for k in st.session_state.files_loaded:
                    st.write(f'-- {k}')
        if st.session_state.tof_analysis.finished and st.session_state.tof_analysis.active:
            
            st.text(f'{st.session_state.tof_analysis.pretty_print_statistics()}')
            if not st.session_state.tof_analysis.cuts.void:
                st.divider()
                st.text('Cut efficiencies:')
                st.text(f'{st.session_state.tof_analysis.cuts.pretty_print_efficiency()}')
        
        if st.session_state.trk_analysis.finished and st.session_state.trk_analysis.active:
            st.divider()
            st.text(f'TRK analysis {st.session_state.trk_analysis.pretty_print_statistics()}')
        # universal kwargs for run loading
        load_run_kwargs = dict()
          
        # create tabbed interface
        tab_telemetry, tab_run, tab_reco, tab_tofanalysis, tab_trkanalysis, tab_tof_cali, tab_settings, = st.tabs(["Telemetry", "Load data", "Reconstruction", "TOF analysis", "TRK analysis", "TOF cali", "Other settings"])
       
        with tab_telemetry:
            st.session_state.telemetry_connected = st.checkbox('Connect to telemetry', value=st.session_state.telemetry_connected)
            address = st.text_input("TCP port for telemetry packet stream", value="tcp://")
            st.session_state.update_every        = st.number_input("Update interval (mins)", value=1)

        with tab_run:
            if st.session_state.telemetry_connected:
                st.warning('Disable telemetry first!', icon=":warning:") 
                return 
            if len(st.session_state.files_loaded) > 0:
                st.button("Clear loaded run!", on_click=reset_session_state) 
            else:
                #st.subheader('Available runs without waveforms')
                st.session_state.load_waveforms = st.checkbox('Load runs with waveforms (might be slower)', value=st.session_state.load_waveforms)
                
                l_col, m_col, r_col = st.columns(3, vertical_alignment="top")
                # sort by subrun
                # FIXME - ultimatly, we want to sort by time
                if st.session_state.load_waveforms:
                    all_runs     = [k for k in Path(config['data']['waveform']).glob('*')]
                    runids       = [k.name for k in all_runs if k.name.isdigit()]
                    selected_run = l_col.selectbox('Select a run (with wf)', tuple(runids))
                    selected_run = all_runs[0].parent / selected_run 
                    sr_infiles   = selected_run.glob('*.gaps')
                    # sort by subrun
                    # FIXME - ultimatly, we want to sort by time
                    sr_infiles = [k for k in sorted(sr_infiles, key=lambda x : gon.io.get_rundata_from_file(str(x))['utctime'])]
                    load_n_files = m_col.number_input(
                      "Number of files to load (with wf)",
                      value=len(sr_infiles),
                      min_value=0,
                      step=1,
                      placeholder="Load only a subset of files")
                    load_run_kwargs['event_type'] = EventType.Tof
                else:
                    all_runs     = [k for k in Path(config['data']['no_waveform']).glob('*')]
                    runids       = reversed(sorted([k.name for k in all_runs if k.name.isdigit()]))
                    selected_run = l_col.selectbox('Select a run (no wf)', tuple(runids))
                    selected_run = all_runs[0].parent / selected_run 
                    sr_infiles   = selected_run.glob('*.gaps')
                    sr_infiles   = [k for k in sorted(sr_infiles, key=lambda x : gon.io.get_rundata_from_file(str(x))['utctime'])]
                    load_n_files = m_col.number_input(
                      "Number of files to load",
                      value=len(sr_infiles),
                      min_value=0,
                      step=1,
                      placeholder="Load only a subset of files")

                st.session_state.infiles = sr_infiles[:load_n_files]
                st.divider()
                ncpus = r_col.number_input(
                      "Use number of CPUs (experimental, does not support monitoring right now)",
                      min_value=1,
                      max_value=config['system']['ncpu'],
                      step=1,
                      placeholder="Select number of CPUs to use for reading out the data. (experimental)")
                load_run_kwargs['ncpus'] = ncpus
                st.session_state.search_event_id = m_col.number_input(
                      "Search for a specific event id (0 for all events)",
                      value=st.session_state.search_event_id,
                      min_value=0,
                      step=1,
                      placeholder="Select a specific event id")
                st.session_state.load_moni_data = st.checkbox('Load all available monitoring data', value=st.session_state.load_moni_data)
                # show a mini overview of run information
                # the index should have - number of packets, number of events
                print (load_run_kwargs)
                #r_col.button('Abort!', on_click=abort_run_loader)    
                if config['data']['allow_write']:
                    st.session_state.write_to_disk = l_col.checkbox('Write selected data to disk', value=st.session_state.write_to_disk)
                    #load_run_kwargs['write_to_basket'] = st.session_state.write_to_disk 
                st.divider()
                st.write("Pick merged event types to load! All others will be dismissed!")
                merged_event_types = []
                # default is to use the MergedEvents 
                use_event = EventType.Merged 
                if not st.session_state.load_waveforms:
                    all_event_types = [EventType.Merged, EventType.Tof]
                    use_event = st.selectbox('Pick the event packet to use', 
                                                 all_event_types,
                                                 index=all_event_types.index(EventType.Merged))
                else:
                    use_event = EventType.Tof
                if use_event == EventType.Merged:
                    boring = st.checkbox('BoringEvent'       , value=True)                
                    nogaps = st.checkbox('NoGapsTriggerEvent', value=True)
                    inter  = st.checkbox('InterestingEvent'  , value=True)
                    notof  = st.checkbox('NoTofDataEvent (recommended to be excluded)'    , value=False)
                    if boring:
                        merged_event_types.append('TelemetryPacketType.BoringEvent')
                    if nogaps:
                        merged_event_types.append('TelemetryPacketType.NoGapsTriggerEvent')
                    if inter:
                        merged_event_types.append('TelemetryPacketType.InterestingEvent')
                    if notof:
                        merged_event_types.append('TelemetryPacketType.NoTofDataEvent')
                load_run_kwargs.update({
                        'event_type'         : use_event,
                        'merged_event_types' : merged_event_types})
                st.divider()
                l_col, m_col, r_col = st.columns(3, vertical_alignment="top")
               
                print (load_run_kwargs)
                l_col.button("Load run!", on_click=load_run, kwargs=load_run_kwargs) 
                m_col.button("Clear analysis!", on_click=clear_analysis) 
        
        with tab_reco:
            st.session_state.reco.active = st.checkbox('run linefit reconstruction')
            st.divider()

        with tab_tofanalysis:
            tof_analysis_active = st.checkbox("run low level TOF analysis", value=st.session_state.tof_analysis.active) 
            
            if tof_analysis_active:
                l_col, m_col, r_col = st.columns(3, vertical_alignment="bottom")
                cachesize = l_col.number_input(
                  "cache size",
                  value=st.session_state.tof_analysis.hit_cache_size,
                  min_value=0,
                  step=1000,
                  placeholder="Cachesize for creating histograms (impacts memory/performance) [larger -> faster]")
                st.session_state.tof_analysis.hit_cache_size = cachesize
                st.session_state.tof_analysis.event_cache_size = cachesize
                nbins = m_col.number_input(
                  "Number of bins for histograms. WARNING: if this is changed, it will reset the analysis!",
                  min_value=1,
                  value = st.session_state.tof_analysis.nbins,
                  step=1,
                  placeholder="Set the number of bins for the histograms")
                if nbins != st.session_state.tof_analysis.nbins:
                    st.session_state.tof_analysis.reinit(nbins = nbins)
                #if not st.session_state.load_waveforms:
                #    all_event_types = [EventType.Merged, EventType.Tof]
                #    use_event = l_col.selectbox('Pick the event packet to use', 
                #                                 all_event_types,
                #                                 index=all_event_types.index(EventType.Merged))
                #    if not isinstance(use_event, EventType):
                #        raise ValueError('boo!')
                #else:
                #    use_event = EventType.Merged
                #merged_event_types = []
                #if use_event == EventType.Merged:
                #    boring = st.checkbox('BoringEvent'       , value=True)                
                #    nogaps = st.checkbox('NoGapsTriggerEvent', value=True)
                #    inter  = st.checkbox('InterestingEvent'  , value=True)
                #    notof  = st.checkbox('NoTofDataEvent (recommended to be excluded)'    , value=False)
                #    if boring:
                #        merged_event_types.append('TelemetryPacketType.BoringEvent')
                #    if nogaps:
                #        merged_event_types.append('TelemetryPacketType.NoGapsTriggerEvent')
                #    if inter:
                #        merged_event_types.append('TelemetryPacketType.InterestingEvent')
                #    if notof:
                #        merged_event_types.append('TelemetryPacketType.NoTofDataEvent')

                st.session_state.tof_analysis.active = tof_analysis_active
                st.session_state.tof_analysis.nbins  = nbins
                st.session_state.tof_analysis.use_offsets = st.checkbox('Use timing offsets from .json')
                # FIXME - this needs to be specific to the TofAnalysis 
                load_run_kwargs['nbins'] = nbins
                st.divider()
                l_col_cuts, m_col_cuts, r_col_cuts = st.columns(3, vertical_alignment="top")
                st.text("Cuts on TOF hits in CBE, UMB, COR are combined with AND")
                st.session_state.tof_analysis.cuts.min_hit_umb = l_col_cuts.number_input(
                  f"Min Number of hits in TOF UMB",
                  value=st.session_state.tof_analysis.cuts.min_hit_umb,
                  min_value=0,
                  step=1,
                  placeholder="Require at least <x> hits in the TOF umbrella")
                st.session_state.tof_analysis.cuts.min_hit_cbe = m_col_cuts.number_input(
                  f"Min Number of hits in TOF CBE",
                  value=st.session_state.tof_analysis.cuts.min_hit_cbe,
                  min_value=0,
                  step=1,
                  placeholder="Require at least <x> hits in the TOF")
                st.session_state.tof_analysis.cuts.min_cos_theta = r_col_cuts.number_input(
                  f"Restrict minimum allowed abs(cos(theta))",
                  value=st.session_state.tof_analysis.cuts.min_cos_theta,
                  min_value=0.0,
                  max_value=1.0,
                  step=0.1,
                  placeholder="Min abs(cos(theta))")
                st.session_state.tof_analysis.cuts.max_hit_umb = l_col_cuts.number_input(
                  f"Max Number of hits in TOF UMB",
                  value=st.session_state.tof_analysis.cuts.max_hit_umb,
                  min_value=0,
                  step=1,
                  placeholder="Require at least <x> hits in the TOF umbrella")
                st.session_state.tof_analysis.cuts.max_hit_cbe = m_col_cuts.number_input(
                  f"Max Number of hits in TOF CBE",
                  value=st.session_state.tof_analysis.cuts.max_hit_cbe,
                  min_value=0,
                  step=1,
                  placeholder="Require at least <x> hits in the TOF")
                st.session_state.tof_analysis.cuts.max_cos_theta = r_col_cuts.number_input(
                  f"Restrict max allowed abs(cos(theta))",
                  value=st.session_state.tof_analysis.cuts.max_cos_theta,
                  min_value=0.0,
                  max_value=1.0,
                  step=0.1,
                  placeholder="Max abs(cos(theta))")
                st.session_state.tof_analysis.cuts.min_hit_all = l_col_cuts.number_input(
                  f"Min Number of hits in entire TOF ",
                  value=st.session_state.tof_analysis.cuts.min_hit_all,
                  min_value=0,
                  max_value=161,
                  step=1,
                  placeholder="Require at least <x> hits in the TOF")
                st.session_state.tof_analysis.cuts.max_hit_all = m_col_cuts.number_input(
                  f"Max Number of hits in entire TOF ",
                  value=st.session_state.tof_analysis.cuts.max_hit_all,
                  min_value=0,
                  max_value = 161,
                  step=1,
                  placeholder="Require at least <x> hits in the TOF")
                st.session_state.tof_analysis.cuts.min_hit_cor = r_col_cuts.number_input(
                  f"Min Number of hits in TOF COR",
                  value=st.session_state.tof_analysis.cuts.min_hit_cor,
                  min_value=0,
                  step=1,
                  placeholder="Require at least <x> hits in the TOF")
                st.session_state.tof_analysis.cuts.max_hit_cor = r_col_cuts.number_input(
                  f"Max Number of hits in TOF COR",
                  value=st.session_state.tof_analysis.cuts.max_hit_cor,
                  min_value=0,
                  step=1,
                  placeholder="Require at least <x> hits in the TOF")
                st.session_state.tof_analysis.cuts.only_causal_hits = r_col_cuts.checkbox("Reject hits where the light reaches paddle ends faster than with C_SPEED_LIGHT_PADDLE (causality cut)", value=st.session_state.tof_analysis.cuts.only_causal_hits)
                apply_lightspeed_cleaning = m_col_cuts.checkbox('Apply lightspeed cleaning for TOF hit')
                if apply_lightspeed_cleaning:
                    st.session_state.tof_analysis.cuts.ls_cleaning_t_err = l_col_cuts.number_input(
                      f"Allowed error in ns for lightspeed cleaning",
                      value=0.35,
                      min_value=0.0,
                      step=0.05,
                      placeholder="Perform lightspeed cleaning for TOF hits`")
                #else:
                #    st.session_state.tof_analysis.cuts.ls_cleaning_t_err = np.inf
                st.divider()
                st.text('Select specific paddles for the beta calculation. If none are selected, then the first outer hit and the first inner hit will be used to calculate beta')
                l_col_pair, m_col_pair, __ = st.columns(3, vertical_alignment="top")
                st.session_state.tof_analysis.pid_outer = l_col_pair.number_input(
                      f"Select the first paddle to form a pair for timing analysis",
                      value=None,
                      min_value=1,
                      max_value=160,
                      step=1,
                      placeholder="First hit paddle for beta calculation`")
                st.session_state.tof_analysis.pid_inner = m_col_pair.number_input(
                      f"Select the second paddle to form a pair for timing analysis",
                      value=None,
                      min_value=1,
                      max_value=160,
                      step=1,
                      placeholder="Second hit paddle for beta calculation`")

                st.session_state.tof_analysis.cuts.fh_must_be_umb = l_col_cuts.checkbox('First hit MUST be on UMB!', value = st.session_state.tof_analysis.cuts.fh_must_be_umb)
                st.session_state.tof_analysis.cuts.thru_going = l_col_cuts.checkbox('Last hit MUST be on CBE BOT or COR (thru-going)!', value = st.session_state.tof_analysis.cuts.thru_going)
                st.session_state.tof_analysis.cuts.fhi_not_bot = m_col_cuts.checkbox('First hit on inner TOF MUST NOT be on CBE BOT)!', value = st.session_state.tof_analysis.cuts.fhi_not_bot)
                st.session_state.tof_analysis.cuts.fho_must_panel7 = r_col_cuts.checkbox('First hit on outer TOF MUST be on panel 7)!', value = st.session_state.tof_analysis.cuts.fho_must_panel7)
                st.session_state.tof_analysis.cuts.lh_must_panel2 = r_col_cuts.checkbox('Last hit MUST be on CBE BOT)!', value = st.session_state.tof_analysis.cuts.lh_must_panel2)
                st.session_state.tof_analysis.cuts.hit_high_edep = l_col_cuts.checkbox('One hit must have edep > 20MeV!', value = st.session_state.tof_analysis.cuts.hit_high_edep)
                if not st.session_state.tof_analysis.cuts.void:
                    st.write(f'Will apply this cut')
                    st.text(f'{st.session_state.tof_analysis.cuts}')
                st.divider()
                st.text("Skip events with doubtful quality!")
                l_col_ev_stat, m_col_ev_stat, __ = st.columns(3, vertical_alignment="top")
                st.session_state.tof_analysis.skip_mangled = l_col_ev_stat.checkbox("Skip mangled events!", value=st.session_state.tof_analysis.skip_mangled)
                st.session_state.tof_analysis.skip_timeout = m_col_ev_stat.checkbox("Skip timed out events!", value=st.session_state.tof_analysis.skip_timeout)
                st.divider() 
                r_col_cuts.button('Reset cuts!', on_click=clear_cuts)
        
        with tab_trkanalysis:
            st.session_state.trk_analysis.active = st.checkbox("run low level TRK analysis", value=st.session_state.trk_analysis.active) 
            if st.session_state.trk_analysis.active:
                st.divider()
                l_col_cuts, m_col_cuts, r_col_cuts = st.columns(3, vertical_alignment="top")
                st.session_state.trk_analysis.cuts.min_hits = l_col_cuts.number_input(
                  f"Min Number of hits in TRK",
                  value=st.session_state.trk_analysis.cuts.min_hits,
                  min_value=0,
                  step=1,
                  placeholder="Require at least <x> hits in TRK")
                st.session_state.trk_analysis.cuts.max_hits = m_col_cuts.number_input(
                  f"Max Number of hits in TRK",
                  value=st.session_state.trk_analysis.cuts.max_hits,
                  min_value=0,
                  step=1,
                  placeholder="Require at max <x> hits in TRK")
                for layer in range(10): 
                    st.session_state.trk_analysis.cuts.min_hits_layer[layer] = l_col_cuts.number_input(
                      f"Min Number of hits in TRK Layer {layer}",
                      value=st.session_state.trk_analysis.cuts.min_hits_layer[layer],
                      min_value=0,
                      step=1,
                      placeholder="Require at least <x> hits in TRK Layer {layer}")
                    st.session_state.trk_analysis.cuts.max_hits_layer[layer] = m_col_cuts.number_input(
                      f"Max Number of hits in TRK Layer {layer}",
                      value=st.session_state.trk_analysis.cuts.max_hits_layer[layer],
                      min_value=0,
                      step=1,
                      placeholder="Require at max <x> hits in TRK Layer {layer}")
                
                st.divider()
                apply_stripmask = st.checkbox("Apply a strip mask", value=st.session_state.apply_tracker_mask)
                if apply_stripmask:
                    all_masks = gon.db.TrackerStripMask.all_names()
                    st.subheader('Available masks')
                    mask_option = st.selectbox('', tuple(all_masks))
                    strip_mask = gon.db.TrackerStripMask.as_dict_by_name(mask_option)
                    n_all_strips    = len(strip_mask.keys())
                    n_masked_strips = len([k for k in strip_mask if not strip_mask[k].active])
                    st.text(f'Masking {n_masked_strips}/{n_all_strips} ({100*n_masked_strips/n_all_strips:.2f}%) of the strips!')
                    #st.session_state.trk_analysis.strip_mask = strip_mask
                    st.divider()
                    st.session_state.trk_calibration['mask'] = mask_option
                else:
                    st.session_state.trk_calibration['mask'] = None
                
                st.session_state.trk_analysis.subtract_pedestals = st.checkbox("Subtract pedestal ADC", value=st.session_state.trk_analysis.subtract_pedestals)
                if st.session_state.trk_analysis.subtract_pedestals:
                    all_peds   = gon.db.TrackerStripPedestal.all_names()
                    ped_option = st.selectbox('', tuple(all_peds))
                    pedestals  = gon.db.TrackerStripPedestal.as_dict_by_name(ped_option)

                    h = d.factory.hist1d([pd.pedestal_mean for pd in pedestals.values()], np.linspace(0,500,70))
                    fig = gander_plot(h,\
                                               xlabel = 'ADC',
                                               title  = 'Tracker pedestals')
                    st.pyplot(fig)
                    st.divider()
                    #pedestals= {k.strip_id : k for k in pedestals}
                    st.session_state.trk_analysis.pedestals = pedestals
                    st.session_state.trk_calibration['pedestal'] = ped_option
                else:
                    st.session_state.trk_calibration['pedestal'] = None 

                st.session_state.trk_analysis.apply_transfer_fn = st.checkbox("Apply transfer functions. This will convert ADC in energy", value=st.session_state.trk_analysis.apply_transfer_fn)
                if st.session_state.trk_analysis.apply_transfer_fn:
                    all_tr_fn_files = gon.db.TrackerStripTransferFunction.all_names()
                    tr_file_option = st.selectbox('', tuple(all_tr_fn_files))
                    transfer_fn = gon.db.TrackerStripTransferFunction.as_dict_by_name(tr_file_option) 
                    print (transfer_fn)
                    st.session_state.trk_analysis.transfer_fn = transfer_fn
                    # this requires we change the binning and re-init the edep plots
                    # FIXME - implemente setter which does that
                    if not st.session_state.trk_analysis.finished:
                        st.session_state.trk_analysis._init_edep_plots()
                    l_col, m_col, __ = st.columns(3, vertical_alignment="top")
                    stripid = l_col.number_input(
                                   f"Plot a transfer function by specifying the strip id (LRMSS)",
                                   #value=st.session_state.trk_analysis.cuts.min_hits_layer[layer],
                                   min_value=0,
                                   max_value=max(transfer_fn.keys()),
                                   step=1,
                                   placeholder="Plot a transfer fn")
                    if stripid in transfer_fn:   
                        xs = np.arange(0,1600,1)
                        ys = np.array([transfer_fn[stripid].transfer_fn(x) for x in xs])
                        fig = gander_line_plot(xs,ys, title=f'Transfer Fn strip {stripid}', xlabel='ADC', ylabel='mV')
                        st.pyplot(fig)
                    else:
                       st.write(f'No transfer fn available for stripid {stripid}')
                    st.session_state.trk_analysis.is_calibrated = True
                    st.session_state.trk_calibration['tf'] = tr_file_option 
                else:
                    st.session_state.trk_calibration['tf'] = None

                    #st.session_state.trk_analysis.transfer_fn = dict()
                    if not st.session_state.trk_analysis.finished:
                        st.session_state.trk_analysis._init_edep_plots()
                
                # Remove hits with ADC
                st.checkbox("Remove hits with ADC = 0", value=st.session_state.trk_analysis.exclude_empty_hits)
                st.divider()

                #--------------------------------

                # the new common noise data
                st.write("Common noise subtraction")
                st.session_state.trk_analysis.subtract_cmnnoise = st.checkbox("Subtract CMN noise", value=st.session_state.trk_analysis.subtract_cmnnoise)
                if st.session_state.trk_analysis.subtract_cmnnoise:
                    all_cmn_files   = gon.db.TrackerStripCmnNoise.all_names()
                    cmn_file_option = st.selectbox('', tuple(all_cmn_files))
                    cmn_data        = gon.db.TrackerStripCmnNoise.as_dict_by_name(cmn_file_option) 
                    h = d.factory.hist1d([cmn.gain for cmn in cmn_data.values()], np.linspace(0,3,70))
                    fig = gander_plot(h,\
                                      xlabel = 'Gain',
                                      title  = 'Tracker CMN noise/Gain',
                                      log    = True)
                    st.pyplot(fig)
                    h = d.factory.hist1d([cmn.pulse_avg for cmn in cmn_data.values()], np.linspace(0,500,70))
                    fig = gander_plot(h,\
                                      xlabel = 'Pulse AVG (ADC)',
                                      title  = 'Tracker CMN noise/Gain',
                                      log    = True)
                    st.pyplot(fig)
                    st.write("Common Noise LVL for strip")
                    xs = np.array([k for k in range(1600)])
                    l_col, m_col, __ = st.columns(3, vertical_alignment="top")
                    stripid = l_col.number_input(
                      f"Plot the cmn noise level for a given strip id (LRMSS)",
                      #value=st.session_state.trk_analysis.cuts.min_hits_layer[layer],
                      min_value=0,
                      max_value=max(cmn_data.keys()),
                      step=1,
                      placeholder="Plot a transfer fn")
                    if stripid in cmn_data:   
                        xs = np.arange(0,1600,1)
                        ys = np.array([cmn_data[stripid].get_common_level(x) for x in xs])
                        fig = gander_line_plot(xs,ys, title=f'CMN noise level for strip {stripid}', xlabel='ADC', ylabel='cmn level')
                        st.pyplot(fig)
                    else:
                       st.write(f'No cmn noise data available for stripid {stripid}')
                    st.session_state.trk_calibration['cmn_noise'] = cmn_file_option
                else:
                    st.session_state.trk_calibration['cmn_noise'] = None

                st.divider()
                

        with tab_tof_cali:
            st.subheader('Select TOF calibrations')
            #st.write('Default path (non-changeable')
            #st.code(TOF_CALI_PATH)
            calibs = reversed(sorted(Path(config['data']['tof_calib']).glob('25*')))
            use_this_calib = st.selectbox('251120_093503UTC', tuple(calibs))
            calib = gon.calibration.load_rb_calibrations(use_this_calib)
            st.session_state['tof_calib'] = calib
            good  = sorted([cal for cal in calib.values() if cal.check()], key=lambda x: x.rb_id)
            bad   = sorted([cal for cal in calib.values() if not cal.check()], key=lambda x: x.rb_id)
            if st.session_state['tof_calib'] is not None:
                if bad:
                    st.warning(f"Some boards failed the calibration self-check! Failing boards:\t {" ".join([str(k.rb_id) for k in bad])}", icon=':material/add_alert:')
                    st.write("RBs with failed calibrations:")
                    for k in bad:
                        cali = st.session_state['tof_calib'][k.rb_id]
                        with st.expander(f"Calibration for RB {k.rb_id}"): 
                            st.text(f"Used TOF calibration files {cali}") 
                            plot_histograms = st.checkbox('Plot distributions', key=f'plot_cali_dist_{k.rb_id}_voffset')
                            if plot_histograms:
                                for ch in range(9):
                                    st.subheader(f'Channel {ch + 1}')
                                    st.divider()
                                    hist = d.factory.hist1d(cali.v_offsets[ch], 70) 
                                    fig  = gander_plot(hist,
                                                       xlabel = 'V Offsets',
                                                       title  = f'RB {k.rb_id}/ CH {ch+1}')
                                    st.pyplot(fig)
                                    
                                    hist = d.factory.hist1d(cali.v_inc[ch], 70) 
                                    fig  = gander_plot(hist,
                                                       xlabel = 'V Inc',
                                                       title  = f'RB {k.rb_id}/ CH {ch+1}')
                                    st.pyplot(fig)
                                    
                                    hist = d.factory.hist1d(cali.v_dips[ch], 70) 
                                    fig  = gander_plot(hist,
                                                       xlabel = 'V Dips',
                                                       title  = f'RB {k.rb_id}/ CH {ch+1}')
                                    st.pyplot(fig)
                                    
                                    hist = d.factory.hist1d(cali.v_offsets[ch], 70) 
                                    fig  = gander_plot(hist,
                                                       xlabel = 'V Offsets',
                                                       title  = f'RB {k.rb_id}/ CH {ch+1}')
                                    print (dir(cali)) 
                                    hist = d.factory.hist1d(cali.tbin[ch], 70) 
                                    fig  = gander_plot(hist,
                                                       xlabel = 'T bins',
                                                       title  = f'RB {k.rb_id}/ CH {ch+1}')
                                    st.pyplot(fig)
                                    st.divider()


                st.divider()
                for k in good:
                    with st.expander(f"Calibration for RB {k.rb_id}"):
                        st.text(f"{st.session_state['tof_calib'][k.rb_id]}") 
                        plot_histograms = st.checkbox('Plot distributions', key=f'plot_cali_dist_{k.rb_id}_voffset')
                        cali = st.session_state['tof_calib'][k.rb_id]
                        if plot_histograms:
                            for ch in range(9):
                                st.subheader(f'Channel {ch + 1}')
                                st.divider()
                                hist = d.factory.hist1d(cali.v_offsets[ch], 70) 
                                fig  = gander_plot(hist,
                                                   xlabel = 'V Offsets',
                                                   title  = f'RB {k.rb_id}/ CH {ch+1}')
                                st.pyplot(fig)
                                
                                hist = d.factory.hist1d(cali.v_inc[ch], 70) 
                                fig  = gander_plot(hist,
                                                   xlabel = 'V Inc',
                                                   title  = f'RB {k.rb_id}/ CH {ch+1}')
                                st.pyplot(fig)
                                
                                hist = d.factory.hist1d(cali.v_dips[ch], 70) 
                                fig  = gander_plot(hist,
                                                   xlabel = 'V Dips',
                                                   title  = f'RB {k.rb_id}/ CH {ch+1}')
                                st.pyplot(fig)
                                
                                hist = d.factory.hist1d(cali.v_offsets[ch], 70) 
                                fig  = gander_plot(hist,
                                                   xlabel = 'V Offsets',
                                                   title  = f'RB {k.rb_id}/ CH {ch+1}')
                                print (dir(cali)) 
                                hist = d.factory.hist1d(cali.tbin[ch], 70) 
                                fig  = gander_plot(hist,
                                                   xlabel = 'T bins',
                                                   title  = f'RB {k.rb_id}/ CH {ch+1}')
                                st.pyplot(fig)
                                st.divider()

        with tab_settings:
            st.subheader("Settings")
            st.session_state.use_dark_theme = st.checkbox('Use dark theme for plots', value = st.session_state.use_dark_theme,\
                                                          on_change=set_plot_theme_streamlit_dark())
            st.session_state.plot_theme_gaps = st.checkbox('Use the official plot theme for GAPS', value = st.session_state.plot_theme_gaps,\
                                                          on_change=set_plot_theme_gaps())
            st.session_state.mark_perliminary = st.checkbox('Add "Preliminary" watermark', value = st.session_state.mark_preliminary)
            if st.session_state.use_dark_theme:
                set_plot_theme_streamlit_dark()

    @st.fragment
    def page_reco():
        """
        PLot which show reconstruction success
        """
        if not st.session_state.reco.finished:
            st.write('-> Please go to Run -> Load Run to load some events for the reconstruction!')
            st.divider()
            return 
        r = st.session_state.reco 
        fig = gander_plot(r.chi2,
                          xlabel = '$\chi^2$',
                          title  = 'Linefit goodness-of-fit')
        st.pyplot(fig)
        fig = gander_plot(r.cos2,
                          xlabel = '$\cos^2$',
                          title  = 'Linefit $\cos^2(\theta)$')
        st.pyplot(fig)
        if st.session_state.use_dark_theme:
            fig = gander_plot(r.beta, xlabel=r'$\beta$', gauss_fit = True, title='TOF beta')
        else:
            kwargs = {'color'  : 'k',\
                      'filled' : True,\
                      'alpha'  : 0.4,\
                      'lw'     : 0.9}
            fig = gander_plot(beta, xlabel=r'$\beta$', gauss_fit = True, title='TOF beta', **kwargs)
        st.pyplot(fig)
    
    @st.fragment
    def page_tofpulses():
        """
        Plots of charge and timing for the TOF
        """
        st.subheader('Paddle pulses')
        st.write(f'{st.session_state.tof_analysis.n_events} events loaded!')
        if not st.session_state.tof_analysis.finished:
            st.write('-> Please go to Run -> Load Run to load some events for the TOF analysis!')
            st.divider()
            return 
        l_col, m_col,r_col,__,__,__,__,__,__,__,__,__ = st.columns(12, vertical_alignment="bottom")

        color_a = l_col.color_picker("Color A side", "#1f77b4") # tab:blue
        color_b = m_col.color_picker("Color B side", "#d62728") # tab:red
        r = st.session_state.tof_analysis.paddle_plots
        p_kwargs = {'color'  : color_a,
                    'filled' : True,
                    'alpha'  : 0.4,
                    'lw'     : 0.9}
        p_kwargs = [copy(p_kwargs), copy(p_kwargs)]
        p_kwargs[1]['color'] = color_b
        if r_col.button('Plot all paddles!'):
            for k in range(1,161):
                with st.expander(f"Paddle {k}"):

                    fig = gander_multi_plot([r[k]['amp_a'],
                                             r[k]['amp_b']],
                                             kwargs  = p_kwargs,
                                             use_gaps_theme = st.session_state.plot_theme_gaps,\
                                             labels = ['A', 'B'], 
                                             xlabel='mV', title=f'paddle {k}, amplitude')
                    st.pyplot(fig)
                    fig = gander_multi_plot([r[k]['time_a'],
                                             r[k]['time_b']],
                                             kwargs  = p_kwargs,
                                             labels = ['A', 'B'], 
                                             xlabel='ns', title=f'paddle {k}, pulse time')
                    st.pyplot(fig)
                    log_scale = st.checkbox('Use log scale!', key=f'logscale_paddle{k}')
                    fig = gander_multi_plot([r[k]['charge_a'],
                                             r[k]['charge_b']],
                                             kwargs  = p_kwargs,
                                             log     = log_scale,
                                             labels  = ['A', 'B'], 
                                             xlabel  ='pC', title=f'paddle {k}, charge')
                    st.pyplot(fig)
                    fig = gander_multi_plot([r[k]['bl_a'],
                                             r[k]['bl_b']],
                                             kwargs  = p_kwargs,
                                             labels = ['A', 'B'], 
                                             xlabel='mV', title=f'paddle {k}, baseline')
                    st.pyplot(fig)
                    fig = gander_multi_plot([r[k]['bl_a_rms'],
                                             r[k]['bl_b_rms']],
                                             kwargs  = p_kwargs,
                                             labels = ['A', 'B'], 
                                             xlabel='mV', title=f'paddle {k}, baseline RMS')
                    st.pyplot(fig)
                    fig = gander_2dplot(r[k]['charge2d'],
                                        xlabel = 'A [pC]',
                                        ylabel = 'B [pC]',
                                        title  = f'paddle {k} charge')
                    st.pyplot(fig)
                    fig = gander_2dplot(r[k]['amp2d'],
                                        xlabel = 'A [mV]',
                                        ylabel = 'B [mV]',
                                        title  = f'paddle {k} amplitude')
                    st.pyplot(fig)
                    logscale = st.checkbox('Log scale!', value=True,key=f'logscale_paddle_edep{k}')
                    fig = gander_plot(r[k]['edep'],
                                      xlabel = 'MeV',
                                      log    = logscale,
                                      title  = f'paddle {k} energy deposition')
                    st.pyplot(fig)
                    fig = gander_plot(r[k]['x0'],
                                      xlabel = 'rel. pos. (1 is B)',
                                      title  = f'paddle {k} reco. dist. from A side')
                    st.pyplot(fig)
                    fig = gander_plot(r[k]['t0'],
                                      xlabel = 'ns',
                                      title  = f'paddle {k} reco. inter. time [no correction]')
                    st.pyplot(fig)


                    fig = gander_2dplot(r[k]['pos_edep'],
                                        xlabel = 'rel. pos. (1 is B)',
                                        ylabel = 'edep [pJ]',
                                        xlim   = (0,1),
                                        title  = 'Rel. position vs. Energy Dep')
                    st.pyplot(fig)

                    #fig = gander_plot(r[k]['amp_b'], xlabel='mV', title=f'PID {k}-B, amplitude')
                    #st.pyplot(fig)

    @st.fragment
    def page_occupancy():
        """
        TOF/Tracker occupancy plots
        """
        def hg_occu_plots(occu, cmap):
            fig, __ = gon.visual.tof.tof_projection_xy(occu, cmap = cmap)
            if st.session_state.mark_perliminary:
                add_preliminary(fig)
            st.pyplot(fig)
            fig, __ = gon.visual.tof.unroll_cbe_sides(paddle_occupancy=occu, cmap = cmap)
            if st.session_state.mark_perliminary:
                add_preliminary(fig)
            st.pyplot(fig)
            fig, __ = gon.visual.tof.unroll_cor(paddle_occupancy=occu, cmap = cmap)
            if st.session_state.mark_perliminary:
                add_preliminary(fig)
            st.pyplot(fig)

        def lg_occu_plots(occu_t, occu_t_non_normalized, cmap, lognorm=False):
            fig, __ = gon.visual.tof.tof_projection_xy(occu_t, cmap = cmap)
            st.pyplot(fig)
            fig, __ = gon.visual.tof.tof_projection_xy(occu_t_non_normalized, cmap = cmap, overlay_panels = True, umbrella_only = True, lognorm = lognorm)
            ax = fig.gca()
            #ax.set_title('umbrella trigger hit occupancy', loc='right')
            #ax.spines['top'].set_visible(True)
            #ax.spines['right'].set_visible(True)
            #fig.savefig('tof-umb-occ.pdf')
            st.pyplot(fig)

            fig, __ = gon.visual.tof.unroll_cbe_sides(paddle_occupancy=occu_t, cmap = cmap)
            st.pyplot(fig)
            fig, __ = gon.visual.tof.unroll_cor(paddle_occupancy=occu_t, cmap = cmap)
            st.pyplot(fig)

        def plot_occu(occu, occu_t, occu_t_non_normalized, cmap, lognorm=False):
            st.subheader('HG occupancy')
            st.write('This is what the RB see. Paddles which have hits, but the energy deposition is 0 are excluded.')
            hg_occu_plots(occu, cmap)
            st.divider()
            st.subheader('LG occupancy')
            st.write('This is what the MTB sees over the LTB system')
            lg_occu_plots(occu_t, occu_t_non_normalized, cmap, lognorm=lognorm)
        # normalize occupancy
        occu    = copy(st.session_state.tof_analysis.occupancy)
        occu_t  = copy(st.session_state.tof_analysis.occupancy_t)
        occu_t_non_normalized = copy(st.session_state.tof_analysis.occupancy_t)
        if len(occu) > 0:
            max_occu   = max([occu[k] for k in occu])
        else:
            max_occu   = 1 
        if len(occu_t) > 0:
            max_occu_t = max([occu_t[k] for k in occu_t])
        else:
            max_occu_t = 1

        for k  in occu:
            if max_occu != 0:
                occu[k]   /= max_occu 
            if max_occu_t != 0:
                occu_t[k] /= max_occu_t
        # show page
        st.subheader('Relative TOF occupancey')
        st.write(f'{st.session_state.tof_analysis.n_events} events loaded!')
        st.write(f'The occupancy is normed. We also recommend a diverging colormap, e.g. seismic so in that way deviations from the average are highllighted')
        st.divider() 
        st.subheader('Options')
        all_colormaps = list(plt.colormaps())
        # omaps from cmocean
        all_cm_cmocean = cmocean.cm.__dict__.keys()
        all_cm_cmocean = [k for k in all_cm_cmocean if not k.startswith('_')]
        all_colormaps.extend(all_cm_cmocean)
        selected_cmap = st.selectbox('Choose a maptlotlib colormap', 
                                     all_colormaps,
                                     index=all_colormaps.index('seismic'))
        try:
            cmap = matplotlib.colormaps[selected_cmap]
        except KeyError:
            cmap = cmocean.cm.__dict__[selected_cmap] 
        use_lognorm = st.checkbox("Use logarithmic normalization of the colormap", value=False)
        st.divider()
        plot_occu(occu, occu_t,occu_t_non_normalized, cmap, lognorm=use_lognorm)
            

    @st.fragment
    def page_event_view():
        """
        A simple event viewer
        """
        ev_data = create_event_plots()
        if not ev_data:
            st.write('No events available!')
            st.write('-> Please go to Run -> Load Run to load some events for this analysis!')
            return
        
        #tracker_plots = plot_tracker(ev_data['trk_hits'], strip_dict)

        st.subheader(f"Event {ev_data['tof_event'].event_id}")
        l_col, r_col, __, __, __, __, __, __  = st.columns(8, vertical_alignment="bottom")
        l_col.button("PrevEvent", on_click=prev_event, args=[st.session_state]) 
        r_col.button("NextEvent", on_click=next_event, args=[st.session_state]) 
        tab_event, tab_tof_panels, tab_tof_waveforms, tab_tracker_layers, tab_2d, tab_3d = st.tabs(["Event", "Tof panels", "Tof waveforms", "Trk layers", "2d projections", "3d view"])
        with tab_event:
            if ev_data['packet_type'] is not None:
                st.badge(f"{ev_data['packet_type']}", color='blue')
            if ev_data['tof_event'].status == gon.events.EventStatus.AnyDataMangling:
                st.badge("AnyDataMangling", color='red')
            if ev_data['tof_event'].status == gon.events.EventStatus.EventTimeOut:
                st.badge("EventTimedOut", color='red')
            
            with st.expander('Event properties'):
                if ev_data['merged_event'] is not None:
                    st.text(f'{ev_data["merged_event"]}')
                    st.divider()
                if not ev_data['tof_event'].event_id == 0: 
                    ## the formatting here looks weird, but apperas nicely in the app
                    st.text(f'''           Trigger sources : {ev_data["tof_event"].trigger_sources}
                    Status                  : {ev_data["tof_event"].status}
                    Event ID              : {ev_data["tof_event"].event_id}
                    Timestamp        : {ev_data["tof_event"].timestamp48}''')
                    mapping = gon.db.get_dsi_j_ch_pid_map()
                    st.divider()
                    st.text(f'TRIGGER HITS : {[h for h in ev_data["tof_event"].trigger_hits]}')
                    st.text(f'RB LINK IDs : {[int(h) for h in ev_data["tof_event"].rb_link_ids]}')
                    if ev_data['tof_event'].get_missing_paddles_hg(mapping):
                        st.text(f'MISSING HG HITS: {[int(h) for h in ev_data["tof_event"].get_missing_paddles_hg(mapping)]}')
                    
                    st.subheader(f"{len(ev_data['tof_hits'])} TOF hits")
                    for h in ev_data['tof_hits']:
                        with st.expander(f"Paddle {h.paddle_id}"):
                            st.text(f'{h}')
                    st.subheader(f"{len(ev_data['trk_hits'])} TRK hits")
            
            for h in ev_data['trk_hits']:
                with st.expander(f'Strip {h.strip_id}'):
                    st.text(f'{h}')
            if ev_data['n_trk_hits_masked'] is None:
                st.text('No tracker strip mask applied!')
            else:
                n_strips_masked = ev_data['n_trk_hits_masked']
                if n_strips_masked > 0:
                    st.text(f'{n_strips_masked} hits have been masked due to marked as bad in the used strip mask!')

        with tab_tof_panels:
            if ev_data['tof_xy'] is not None:
                st.pyplot(ev_data['tof_xy'])
            if ev_data['tof_cbe'] is not None:
                st.pyplot(ev_data['tof_cbe'])
            if ev_data['tof_cor'] is not None:
                st.pyplot(ev_data['tof_cor'])

        with tab_tof_waveforms: 
            #st.subheader(f"Waveforms for {len(st.session_state['waveform_figs'])} paddles")
            st.session_state.no_plot_first_bins_wf = st.checkbox("Skip the first 10 bins (ns) when plotting waveforms", value=st.session_state.no_plot_first_bins_wf, on_change=create_event_plots)
            st.session_state.no_plot_first_bins_wf2 = st.checkbox("Skip the first 100 bins (ns) when plotting waveforms", value=st.session_state.no_plot_first_bins_wf2, on_change=create_event_plots)
            st.session_state.no_plot_last_bins_wf = st.checkbox("Skip the last 250 bins (ns) when plotting waveforms", value=st.session_state.no_plot_last_bins_wf, on_change=create_event_plots)
            st.divider()
            ev_data = create_event_plots(no_plot_first_bins_wf=st.session_state.no_plot_first_bins_wf)
            if not ev_data:
                st.write('No events available!')
                st.write('-> Please go to Run -> Load Run to load some events for this analysis!')
                return
            if 'waveform_figs' in ev_data:
                for wf_plot in ev_data['waveform_figs']:
                    if st.session_state.no_plot_last_bins_wf:
                        ax = wf_plot.gca()
                        ax.set_xlim(right=250)
                    if st.session_state.no_plot_first_bins_wf2:
                        ax = wf_plot.gca()
                        ax.set_xlim(left=100)
                    st.pyplot(wf_plot)

        with tab_tracker_layers:
            ev_filename , ptype, (ev, wf_ev) = st.session_state.ev_viewer_cache[st.session_state.ev_viewer_idx]
            if hasattr(ev,'tracker'):
                trk_plots = gon.visual.tracker.plot_tracker(hits = ev.tracker) 
            else:
                trk_plots = False
            if trk_plots:
                st.pyplot(trk_plots['trk_proj_xy'], use_container_width=False)
                st.pyplot(trk_plots['trk_proj_xz'], use_container_width=False)
                st.pyplot(trk_plots['trk_proj_yz'], use_container_width=False)
                layer_keys = [k for k in trk_plots.keys() if k not in ('trk_proj_xy', 'trk_proj_xz', 'trk_proj_yz')]
                for k in layer_keys:
                    layer = int(k[10:])
                    with st.expander(f"Layer {layer}"):
                        st.pyplot(trk_plots[k], use_container_width=False) 
            #if ev_data['trk_plots']:
            #    for k in ev_data['trk_plots'].keys():
            #        fig = ev_data['trk_plots'][k]
            #        st.pyplot(fig, use_container_width=False)

        with tab_2d:
            hitstyle={ 'edgecolor' : 'w', 'alpha' : 0.5 , 'marker' : 'o'} 
            circle_color = 'w'
            if not st.session_state.use_dark_theme:
                circle_color = 'k'
                hitstyle={'edgecolor' : 'k', 'alpha' : 0.5, 'marker' : 'o'} 
            #tof_ev = copy(ev_data['tof_event'])
            plot_tracker2d = st.checkbox('Add tracker projections', key='add_tracker_proj')
            cs_is_energy   = st.checkbox('Use color scale for energy instead of timing', key='cs_is_energy')
            viewer_apply_lightspeed_cleaning = st.checkbox("Apply lightspeed cleaning for TOF hit. (Can't be undone!)", key='lightspeed_cleaning_2dview')
            cleaning_tolerance = 0.35
            #if viewer_apply_lightspeed_cleaning:
            cleaning_tolerance= st.number_input(
              f"Allowed error in ns for lightspeed cleaning",
              value=0.35,
              min_value=0.0,
              step=0.05,
              key='lightspeed_cleaning_tolerance',
              placeholder="Perform lightspeed cleaning for TOF hits`")
            if viewer_apply_lightspeed_cleaning:
                ev_data['tof_event'].lightspeed_cleaning(t_err = cleaning_tolerance)
            color = 'w'
            paddle_style     = {'edgecolor' : 'w', 'lw' : 0.4}

            if not st.session_state.use_dark_theme:
                color = 'k'
                paddle_style     = {'edgecolor' : 'k', 'lw' : 0.4}
            show_linefit   = st.checkbox('Show a simple linefit')
            if show_linefit:
                search_anchor  = st.checkbox('Search anchor point iteratively (slower)')
                xs = [k[0] for k in ev_data['trk_pointcloud']]
                xs.extend([h.x for h in ev_data['tof_event'].hits])

                ys = [k[1] for k in ev_data['trk_pointcloud']]
                ys.extend([h.y for h in ev_data['tof_event'].hits])

                zs = [k[2] for k in ev_data['trk_pointcloud']]
                zs.extend([h.z for h in ev_data['tof_event'].hits])
                
                xs = np.array(xs)
                ys = np.array(ys)
                zs = np.array(zs)
                
                reco = gon.reconstruction.line_fit(xs, ys, zs, search_anchor = search_anchor)
                # plot in z from -25 to 250
                p0, chi2 = reco[0](2200),reco[1]
                #chi2/(len(xs) - 6)
                p1 = reco[0](-200)
                print ('RCONSTRUCTION!',p0, p1, chi2)
                p0 = np.array(p0)/10
                p1 = np.array(p1)/10
                p_arrow_h = reco[0](500) # somewhat close to the end
                p_arrow_t = reco[0](300) # somewhat close to the end
            all_colormaps = list(plt.colormaps())
            selected_cmap = st.selectbox('Choose a maptlotlib colormap for the times of the TOF hits', 
                                         all_colormaps,
                                         index=all_colormaps.index('seismic'),
                                         key = 'cmap_selectbox_2dproj')
            cmap  = matplotlib.colormaps[selected_cmap]
            no_ax = st.checkbox("Don't show axes", value=True)
            show_cbar = st.checkbox("Show a colorbar", value=True)
            print (ev_data['tof_event'])
            print (ev_data['tof_event'].rb_events)
            
            tof_xy_all, tof_xz_all, tof_yz_all \
                = gon.visual.tof.tof_2dproj(event=ev_data['tof_event'],\
                                            cmap=cmap,
                                            paddle_style   = paddle_style,
                                            no_ax_no_ticks = no_ax,
                                            show_cbar      = show_cbar,
                                            cnorm_max      = 2,
                                            cs_is_energy   = cs_is_energy)
            if plot_tracker2d or show_linefit:
                ax = tof_xy_all.gca()
                if show_linefit:
                    ax.plot([p0[0],p1[0]],[p0[1],p1[1]], color=color, lw=1.0)
                    ax.text(p0[0]+5,p0[1]+5,'$\mu$')
                    ax.text(p0[0]+5,p0[1]-4, '$\chi^2$ = ' + f'{chi2:.2f}', fontsize=5)
                    add_arrow(ax, p_arrow_h, p_arrow_t, fc=paddle_style['edgecolor'], ec=paddle_style['edgecolor'])
                if plot_tracker2d:    
                    gon.visual.tracker.plot_tracker_proj(
                        ax,\
                        ev_data['trk_hits'],\
                        projection='xy',\
                        use_energy=cs_is_energy,\
                        circle_color = circle_color,\
                        hitstyle = hitstyle,\
                        cmap = cmap,\
                        color_energy=cs_is_energy
                    )
                
                ax = tof_xz_all.gca()
                if show_linefit:
                    ax.plot([p0[0],p1[0]],[p0[2],p1[2]], color=color, lw=1.0)
                    ax.text(p0[0]+5,p0[2]+5,'$\mu$')
                    add_arrow(ax, p_arrow_h, p_arrow_t,c0=0,c1=2, fc=paddle_style['edgecolor'], ec=paddle_style['edgecolor'])
                if plot_tracker2d:
                    gon.visual.tracker.plot_tracker_proj(
                        ax,\
                        ev_data['trk_hits'],\
                        projection='xz',\
                        use_energy=cs_is_energy,\
                        circle_color = circle_color,\
                        hitstyle = hitstyle,\
                        cmap = cmap,\
                        color_energy=cs_is_energy
                    )
                
                ax = tof_yz_all.gca()
                if show_linefit:
                    ax.plot([p0[1],p1[1]],[p0[2],p1[2]], color=color, lw=1.0)
                    ax.text(p0[1]+5,p0[2]+5,'$\mu$')
                    add_arrow(ax, p_arrow_h, p_arrow_t, c0=1,c1=2, fc=paddle_style['edgecolor'], ec=paddle_style['edgecolor'])
                if plot_tracker2d:
                    gon.visual.tracker.plot_tracker_proj(
                        ax,\
                        ev_data['trk_hits'],\
                        projection='yz',\
                        use_energy=cs_is_energy,\
                        circle_color = circle_color,\
                        hitstyle = hitstyle,\
                        cmap = cmap,\
                        color_energy=cs_is_energy
                    )
            
            st.pyplot(tof_xy_all)
            st.pyplot(tof_xz_all)
            st.pyplot(tof_yz_all)
            st.subheader('TOF Noise identification - "lightspeed cleaning"') 
            time_evolution = gon.visual.tof.tof_hits_time_evolution(ev_data['tof_event'],line_color='w', t_err = cleaning_tolerance)
            st.pyplot(time_evolution)

        with tab_3d:
            # Load mesh
            show_3dplot = st.checkbox('Show 3d plot! (experimental, might take a while)')
            show_linefit   = st.checkbox('Show a simple linefit', key='lf_for_3d')
            xs = [k[0] for k in ev_data['trk_pointcloud']]
            xs.extend([h.x for h in ev_data['tof_event'].hits])

            ys = [k[1] for k in ev_data['trk_pointcloud']]
            ys.extend([h.y for h in ev_data['tof_event'].hits])

            zs = [k[2] for k in ev_data['trk_pointcloud']]
            zs.extend([h.z for h in ev_data['tof_event'].hits])
            sizes = [k[3] for k in ev_data['trk_pointcloud']]
            sizes.extend([h.edep for h in ev_data['tof_event'].hits])

            xs = np.array(xs)
            ys = np.array(ys)
            zs = np.array(zs)
            sizes = 5*np.array(sizes) 
            
            plotter = st.session_state.vtk_plotter
            if st.button("Reset"):
                del st.session_state.vtk_plotter 
                st.session_state.vtk_plotter = pv.Plotter(window_size=[800, 600], off_screen=True)
            
            if show_linefit:
                
                reco = gon.reconstruction.line_fit(xs, ys, zs, search_anchor = False)
                if reco is not None:
                    # plot in z from -25 to 250
                    p0, chi2 = reco[0](2200),reco[1]
                    #chi2/(len(xs) - 6)
                    p1 = reco[0](-200)
                    print ('RCONSTRUCTION!',p0, p1, chi2)
                    p0 = np.array(p0)
                    p1 = np.array(p1)
                    p_arrow_h = reco[0](500) # somewhat close to the end
                    p_arrow_t = reco[0](300) # somewhat close to the end
                else:
                    show_linefit = False

            if show_3dplot:
                
                #mesh = pv.read("/srv/gaps/gaps-online-software/event-viewer/sample.ply")
                #print ("Mesh loaded!") 
                # Create a PyVista plotter
                plotter.set_background("#0E1117")
                #print ("Plotter created")
                # Example: point cloud
                #points = np.random.rand(100, 3) * 10  # 100 random points in space
                points = np.array([k for k in zip(xs,ys,zs)])/10
                #print (points)
                #sizes = np.linspace(5, 20, len(points))       # point sizes
                colors = np.random.rand(len(points), 3)       # RGB colors in [0,1]
                
                point_cloud = pv.PolyData(points)
                point_cloud["colors"] = (colors * 255).astype(np.uint8)
                #point_cloud["sizes"] = sizes
                point_cloud["scales"] = sizes 
                # Create sphere source for glyphs
                sphere = pv.Sphere(radius=1.0)
                
                # Glyph each point with its own size
                glyphs = point_cloud.glyph(
                    geom=sphere,
                    scale="scales",   # use per-point scaling
                    orient=False
                )

                
                # Create PyVista plotter
                plotter = pv.Plotter(window_size=[800, 600], off_screen=True)
                plotter.set_background("#0E1117")
                #plotter.set_background("#000000")

                ## Add point cloud (with custom sizes & colors)
                plotter.add_mesh(glyphs, scalars="colors", rgb=True)
                #plotter.add_points(
                #    point_cloud,
                #    scalars="colors",
                #    rgb=True,
                #    render_points_as_spheres=True,
                #    point_size=10,  # global scale, sizes will modulate this
                #)
                #
                ## Add line
                # Example: line between two points
                if show_linefit:
                    line_points = (p0/10,p1/10)
                #line_points = np.array([[0, 0, 0], [5, 5, 5]])
                    line = pv.Line(line_points[0], line_points[1])
                    plotter.add_mesh(line, color="#F0F0F0", line_width=5)
                # Add wireframe mesh
                #plotter.add_mesh(mesh, color="#F0F0F0", style="wireframe", line_width=1)
                paddles = gon.db.TofPaddle.all()
                for pdl in paddles:
                    box = pv.wrap(pdl._create_box())
                #box = pv.Box(bounds=(0, 1, 0, 2, 0, 0.5)) 
                # Add mesh as wireframe
                #plotter.add_mesh(mesh, color="#F0F0F0", style="wireframe", line_width=1)
                    plotter.add_mesh(box, color="#F0F0F0", style="wireframe", line_width=1)
                #print ("mesh added")

                # Render inside Streamlit
                plotter.reset_camera()
                #col1, col2, col3, col4, __, __, __, __, __, __, __, __ = st.columns(12)
                #with col1:
                #    if st.button("🔄 Reset"):
                #        plotter.reset_camera()    
                #with col2:
                #    if st.button("X-Axis"):
                #        plotter.view_xz()
                #        plotter.reset_camera()
                #with col3:
                #    if st.button("Y-Axis"):
                #        plotter.view_yz()
                #        plotter.reset_camera() 
                #with col4:
                #    if st.button("Z-Axis"):
                #        plotter.view_xy()
                #        plotter.reset_camera()
                stpyvista(plotter, key="ply_viewer")
                print ("stpvista done")

    @st.fragment
    def page_nhit():
        st.subheader('TOF NHit distributions')
        if not st.session_state.tof_analysis.finished:
            st.write('-> Please go to Run -> Load Run to load some events for this analysis!')
            st.divider()
        else: 
            r = st.session_state.tof_analysis
            st.write(f"{r.n_events} were loaded in the analysis!")
            if r.n_events > 0:
                st.write(f"-- {r.no_hitmiss} [{100*r.no_hitmiss/r.n_events:.3f}%] events had no hit missing")
                st.write(f"-- {r.one_hitmiss} [{100*r.one_hitmiss/r.n_events:.3f}%] events had one hits missing")
                st.write(f"-- {r.two_hitmiss} [{100*r.two_hitmiss/r.n_events:.3f}%] events had two or more hits missing")
            print (r.nhit_plots['hit'].bincontent) 
            print (r.nhit_plots['thit'].bincontent) 

            fig, fig_ratio  = gon.visual.tof.plot_hg_lg_hits(r.nhit_plots['hit'],\
                                                             r.nhit_plots['thit'],\
                                                             n_events         = r.n_events,\
                                                             no_hitmissing    = r.no_hitmiss,\
                                                             one_hitmissing   = r.one_hitmiss,\
                                                             lttwo_hitmissing = r.two_hitmiss,\
                                                             extra_hits       = r.extra_hits)
            st.pyplot(fig)
            st.pyplot(fig_ratio)
            fig = gander_plot(r.nhit_plots['miss_hit'],\
                              xlabel=r'PID',\
                              title='Missing hits on paddle',
                              figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT)
       
            st.pyplot(fig)
            if st.session_state.tof_analysis.cuts.only_causal_hits:
                fig = gander_plot(r.nhit_plots['nc_pdls'],\
                                  xlabel=r'PID',\
                                  title='Non-causal hits on paddle',
                                  figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT)
       
                st.pyplot(fig)

            st.divider()
        if not st.session_state.trk_analysis.finished:
            st.write('-> Please go to Run -> Load Run to load some events for the TRK analysis!')
        if st.session_state.trk_analysis.active:
            r = st.session_state.trk_analysis.nhit_plots
            log_scale = st.checkbox('Use log scale!')
            fig = gander_plot(r['nhit'],\
                              xlabel='N',\
                              title='All TRK HITS',
                              log  = log_scale,\
                              figsize=lo.FIGSIZE_A4_LANDSCAPE)
       
            st.pyplot(fig)
            st.divider()
            for layer in range(10):
                with st.expander(f'Hits in layer {layer}'):
                    log_scale = st.checkbox('Use log scale!', key=f'logscale_layer{layer}')
                    fig = gander_plot(r[f'nhit_layer{layer}'],\
                                       xlabel='N',\
                                       title=f'TRK HITS LAYER {layer}',
                                       log  = log_scale,\
                                       figsize=lo.FIGSIZE_A4_LANDSCAPE)
                                        
                    st.pyplot(fig)
            if st.session_state.tof_analysis.finished and st.session_state.trk_analysis.finished:
                trk = st.session_state.trk_analysis.nhit_plots
                tof = st.session_state.tof_analysis.nhit_plots
                log_scale = st.checkbox('Use log scale!', key='use_logscale_nhit_trk_tof')
                p_kwargs = {'color'  : 'tab:blue',
                    'filled' : True,
                    'alpha'  : 0.4,
                    'lw'     : 0.9}
                p_kwargs = [copy(p_kwargs), copy(p_kwargs)]
                p_kwargs[1]['color'] = 'tab:red'
                fig = gander_multi_plot([tof['hit'],
                                         trk['nhit']],
                                         kwargs  = p_kwargs,
                                         log     = log_scale,
                                         labels = ['TOF', 'TRK'], 
                                         xlabel='NHit', title=f'TOF, TRK NHit')
                if st.session_state.mark_perliminary:
                    add_preliminary(fig)
                st.pyplot(fig)

    @st.fragment
    def page_energy():
        st.subheader("Energy deposition analysis")
        if st.session_state.use_dark_theme:
            kwargs = {'color'  : 'w',\
                      'filled' : True,\
                      'alpha'  : 0.4,\
                      'lw'     : 0.9}
        else:
            kwargs = {'color'  : 'k',\
                      'filled' : True,\
                      'alpha'  : 0.4,\
                      'lw'     : 0.9}


        if not st.session_state.tof_analysis.finished:
            st.write('-> Please go to Run -> Load Run/Tof analysis to load some events for the tof analysis!')
        else:
            #---------------------------------------
            r = st.session_state.tof_analysis
            log_scale = st.checkbox('Use log scale!', key=f'logscale_edep_tof')
            if st.session_state.use_dark_theme:
                fig = gander_plot(r.edep_plots['edep'], xlabel=r'EDep',\
                                  gauss_fit = False, title='TOF EDep',\
                                  log = log_scale)
            else:
                kwargs = {'color'  : 'k',\
                          'filled' : True,\
                          'alpha'  : 0.4,\
                          'lw'     : 0.9}
                fig = gander_plot(r.edep_plots['edep'],\
                                  xlabel=r'EDep',\
                                  gauss_fit = False,\
                                  log = log_scale,\
                                  title='TOF EDep', **kwargs)
            if st.session_state.mark_perliminary:
                add_preliminary(fig)
            st.pyplot(fig)
            
            #---------------------------------------
            
            for k in range(1,22):
                with st.expander(f"Panel {k}"):
                    log_scale_pnl = st.checkbox('Use log scale!', key=f'logscale_edep_tof{k}')
                    if st.session_state.use_dark_theme:
                        fig = gander_plot(r.edep_plots[f'edep_pnl{k}'],\
                                          xlabel=r'EDep', gauss_fit = False,\
                                          log = log_scale_pnl,
                                          title=f'TOF EDep Panel {k}')
                    else:
                        kwargs = {'color'  : 'k',\
                                  'filled' : True,\
                                  'alpha'  : 0.4,\
                                  'lw'     : 0.9}
                        fig = gander_plot(r.edep_plots[f'edep_pnl{k}'],\
                                          xlabel=r'EDep',
                                          gauss_fit = False,\
                                          log = log_scale_pnl,
                                          title=f'TOF EDep {k}', **kwargs)
                    if st.session_state.mark_perliminary:
                        add_preliminary(fig)
                    st.pyplot(fig)
        
        #---------------------------------------

        st.divider()
        if not st.session_state.trk_analysis.finished:
            st.write('-> Please go to Run -> Load Run/Tracker analysis to load some events for the tracker anlaysis!')
        else:
            xlabel = 'ADC'
            if st.session_state.trk_analysis.is_calibrated:
                xlabel = 'MeV'
            r = st.session_state.trk_analysis.edep_plots
            l_col, m_col, r_col = st.columns(3, vertical_alignment="top")
            log_scale_e1 = l_col.checkbox('Use log scale!')
            fit_landau   = m_col.checkbox('Fit Landau!', key='fit_landau_edep_trk')
            fig = gander_plot(r['edep'],
                              xlabel     = xlabel,
                              log        = log_scale_e1,
                              landau_fit = fit_landau,
                              title      = f'Tracker energy deposition',
                              **kwargs)
            if st.session_state.mark_perliminary:
                add_preliminary(fig)
            st.pyplot(fig)
            for layer in range(10):
                l_col, m_col, r_col = st.columns(3, vertical_alignment="top")
                log_scale  = l_col.checkbox('Use log scale!', key=f'log_scale_edep{layer}')
                fit_landau = m_col.checkbox('Fit Landau!', key=f'fit_landau_edep_trk{layer}')
                fig = gander_plot(r[f'edep_layer{layer}'],
                                  xlabel     = xlabel,
                                  log        = log_scale,
                                  landau_fit = fit_landau,
                                  title      = f'Tracker energy dep. layer {layer}',
                                  **kwargs)
                if st.session_state.mark_perliminary:
                    add_preliminary(fig)
                st.pyplot(fig)

    @st.fragment
    def page_beta():
        st.subheader("Beta analysis")
        if not st.session_state.tof_analysis.finished:
            st.write('-> Please go to Run -> Load Run to load some events for this analysis!')
            return 
        r = st.session_state.tof_analysis
        if st.session_state.use_dark_theme:
            fig = gander_plot(r.tmg_plots['beta'], xlabel=r'$\beta$', gauss_fit = True, title='TOF beta')
        else:
            kwargs = {'color'  : 'k',\
                      'filled' : True,\
                      'alpha'  : 0.4,\
                      'lw'     : 0.9}
            fig = gander_plot(r.tmg_plots['beta'], xlabel=r'$\beta$', gauss_fit = True, title='TOF beta', **kwargs)
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        st.pyplot(fig)
        
        p_kwargs = {'color'  : 'tab:blue',
                    'filled' : True,
                    'alpha'  : 0.4,
                    'lw'     : 0.9}
        p_kwargs = [copy(p_kwargs), copy(p_kwargs)]
        p_kwargs[1]['color'] = 'tab:red'
        fig = gander_multi_plot([r.tmg_plots['t_inner'],
                                 r.tmg_plots['t_outer']],
                                 kwargs  = p_kwargs,
                                 labels = ['Inner', 'Outer'], 
                                 xlabel='ns', title=f'First inner and outer TOF hit')
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        st.pyplot(fig)

        tmg_hist_title = 'First inner - outer'
        if (st.session_state.tof_analysis.pid_inner is not None) and \
        (st.session_state.tof_analysis.pid_outer is not None):
            tmg_hist_title = r'$\Delta_t($pdl$_{' + f'{st.session_state.tof_analysis.pid_outer}' + r'}$,pdl$_{' + f'{st.session_state.tof_analysis.pid_inner}' + r'})$'
        logger.info(f'Will use title {tmg_hist_title} for tmg histogram!')
        if st.session_state.use_dark_theme:
            fig = gander_plot(r.tmg_plots['t_diff'], xlabel=r'ns', title=tmg_hist_title, gauss_fit = True)
        else:
            kwargs = {'color'  : 'k',\
                      'filled' : True,\
                      'alpha'  : 0.4,\
                      'lw'     : 0.9}
            fig = gander_plot(r.tmg_plots['t_diff'],
                              xlabel=r'ns',
                              title=tmg_hist_title,
                              use_gaps_style=st.session_state.plot_theme_gaps,
                              gauss_fit = True, **kwargs)
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        st.pyplot(fig)
        fig = gander_plot(r.tmg_plots['ph_delay'], xlabel=r'phase differenc', title='Phase first inner - outer')
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        st.pyplot(fig)
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        fig = gander_plot(r.tmg_plots['dist'], xlabel='m', title='Distance (first inner - first outer)')
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        st.pyplot(fig)
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        fig = gander_plot(r.tmg_plots['cos_theta'], xlabel=r'$\cos(\theta))$', title=r'$\cos(\theta)$ (first inner - first outer)')
        ax = fig.gca()
        ax.invert_xaxis()
        st.pyplot(fig)
        fig = gander_plot(r.tmg_plots['cos2_theta'], xlabel=r'$\cos^2(\theta))$', title=r'$\cos^2(\theta)$ (first inner - first outer)')
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        ax = fig.gca()
        ax.invert_xaxis()
        st.pyplot(fig)
        fig = gander_2dplot(r.tmg_plots['dist_vs_tdiff'],
                            xlabel = 'm',
                            ylabel = r'T_{diff}',
                            title  = r'Distance vs $T_{inner} - T_{outer}$')
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        st.pyplot(fig)
        fig = gander_2dplot(r.tmg_plots['dist_vs_beta'],
                            xlabel = 'm',
                            ylabel = r'$\beta$',
                            title  = r'Distance vs $\beta$')
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        st.pyplot(fig)
        fig = gander_2dplot(r.tmg_plots['beta_vs_theta'],
                            xlabel = r'$\beta$',
                            ylabel = r'$\cos(\theta)$',
                            invert_yaxis = True,
                            title  = r'$\beta$ vs $\cos(\theta)$')
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        st.pyplot(fig)
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        fig = gander_plot(r.tmg_plots['x_inner'], xlabel='mm', title=r'X(first inner)')
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        st.pyplot(fig)
        fig = gander_plot(r.tmg_plots['y_inner'], xlabel='mm', title=r'Y(first inner)')
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        st.pyplot(fig)
        fig = gander_plot(r.tmg_plots['z_inner'], xlabel='mm', title=r'Z(first inner)')
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        st.pyplot(fig)
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        fig = gander_plot(r.tmg_plots['pid_inner'], xlabel='PID', title=r'first inner')
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        st.pyplot(fig)
        
        fig = gander_plot(r.tmg_plots['x_outer'], xlabel='mm', title=r'X(first outer)')
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        st.pyplot(fig)
        fig = gander_plot(r.tmg_plots['y_outer'], xlabel='mm', title=r'Y(first outer)')
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        st.pyplot(fig)
        fig = gander_plot(r.tmg_plots['z_outer'], xlabel='mm', title=r'Z(first outer)')
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        st.pyplot(fig)
        fig = gander_plot(r.tmg_plots['pid_outer'],\
                                   xlabel  = 'PID',\
                                   figsize = lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT,\
                                   title   = r'first outer')
        if st.session_state.mark_perliminary:
            add_preliminary(fig)
        st.pyplot(fig)

    #@st.fragment
    #def page_timing():
    #    st.subheader("Timing analysis")
    #    st.divider()
    #    pid0, pid1 = None, None
    #    pair_only = st.checkbox("Pick a paddle pair!")
    #    if pair_only:
    #        pid0 = st.number_input(
    #          "",
    #          value=0,
    #          min_value=0,
    #          step=1,
    #          placeholder="First paddle")
    #        pid1 = st.number_input(
    #          "",
    #          value=0,
    #          min_value=0,
    #          step=1,
    #          placeholder="Second paddle")
    #    
    #    infiles = []
    #    if not isinstance(st.session_state['current_run_path'], Path):
    #        infiles = Path(st.session_state['current_run_path'])
    #        if infiles.is_file():
    #            infiles = [infiles]
    #    
    #    load_bar = st.progress(0, text='Creating nhit distributions')
    #    #st.button("Create timing plots!", on_click=create_timing_distributions, args=[infiles, load_bar], kwargs={'pid0' : pid0, 'pid1' : pid1}) 
    #    r = create_timing_distributions(infiles, load_bar, pid0=pid0, pid1=pid1)
    #    load_bar.empty()
    #    
    #    # canvases for the plots
    #    fig = gander_plot(r['histo_beta'], xlabel=r'$\beta$', title='TOF beta')
    #    st.pyplot(fig)
    #    
    #    fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE)
    #    ax = fig.gca()
    #    r['histo_t_outer'].line(color='tab:red', filled=True, alpha=0.4,lw=0.9, label='outer TOF')
    #    r['histo_t_inner'].line(color='tab:blue', filled=True, alpha=0.4,lw=0.9, label='inner TOF')
    #    r['histo_t_diff'].line(color='w', filled=True, alpha=0.4, label='diff')
    #    ax.legend(loc='upper right', frameon=False)
    #    ax.set_title('Hit $t_0$', loc='right')
    #    cb.visual.adjust_minor_ticks(ax, which='x')
    #    ax.set_ylim(bottom=0)
    #    st.pyplot(fig)

    #    fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT)
    #    ax = fig.gca()
    #    r['histo_hit_pid'].line(color='w', filled=True, alpha=0.4, lw=0.9)
    #    ax.set_title('HIT PID', loc='right')
    #    ax.set_ylim(bottom=0)
    #    st.pyplot(fig)

    #    fig = gander_plot(r['histo_dist'], xlabel='m', title='Reco distance')
    #    #fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE)
    #    #ax = fig.gca()
    #    #r['histo_dist'].line(color='w', filled=True, alpha=0.4, lw=0.9)
    #    #ax.set_title('Reco distance', loc='right')
    #    #ax.set_ylim(bottom=0)
    #    st.pyplot(fig)

    #    fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE)
    #    ax = fig.gca()
    #    r['histo_pdelay'].line(color='w', filled=True, alpha=0.4,lw=0.9, label='delta')
    #    r['histo_ph_in'].line(color='tab:blue', filled=True, alpha = 0.4,lw=0.9, label='inner')
    #    r['histo_ph_out'].line(color='tab:red', filled=True, alpha = 0.4,lw=0.9, label='outer')
    #    ax.legend(loc='upper right', frameon=False)
    #    ax.set_title('Phase delay', loc='right')
    #    ax.set_ylim(bottom=0)
    #    cb.visual.adjust_minor_ticks(ax, which='x')
    #    ax.vlines(50,0,100,color='w', ls='dashed')
    #    st.pyplot(fig)
    #    
    #    fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE)
    #    ax = fig.gca()
    #    r['histo_nhit'].line(color='w', filled=True, alpha=0.4,lw=0.9, label='delta')
    #    #ax.legend(loc='upper right', frameon=False)
    #    ax.set_title('NHit', loc='right')
    #    ax.set_ylim(bottom=0)
    #    cb.visual.adjust_minor_ticks(ax, which='x')
    #    st.pyplot(fig)
    #    
    #    fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE)
    #    ax = fig.gca()
    #    r['histo_t_diff_fst'].line(color='w', filled=True, alpha=0.4,lw=0.9, label='delta')
    #    #ax.legend(loc='upper right', frameon=False)
    #    ax.set_title('Delta T last - first', loc='right')
    #    ax.set_ylim(bottom=0)
    #    cb.visual.adjust_minor_ticks(ax, which='x')
    #    st.pyplot(fig)
    #    
    #    fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT)
    #    ax = fig.gca()
    #    binshift_outer = 0.5*np.arange(r['last_pd_outer'], 1024 + r['last_pd_outer'], 1)[:200]
    #    binshift_inner = 0.5*np.arange(r['last_pd_inner'], 1024 + r['last_pd_outer'], 1)[:200]
    #    ax.plot(binshift_inner,r['wf_inner'][:200],color='tab:blue', lw=0.9, label='inner')
    #    ax.plot(binshift_outer,r['wf_outer'][:200],color='tab:red', lw=0.9, label='outer')
    #    ax.set_xlabel('ns', loc='right')
    #    #ax.plot(r['wf_inner'][:200],color='tab:blue', lw=0.9, label='inner')
    #    #ax.plot(r['wf_outer'][:200],color='tab:red', lw=0.9, label='outer')
    #    ax.vlines(r['last_pd_inner'],0,7000, color='tab:blue', lw=0.9)
    #    ax.vlines(r['last_pd_outer'],0,7000, color='tab:red', lw=0.9)

    #    ax.legend(loc='upper right', frameon=False)
    #    ax.set_title('Ch9 ADC', loc='right')
    #    ax.set_ylim(bottom=0)
    #    cb.visual.adjust_minor_ticks(ax, which='x')
    #    ax.vlines(50,0,100,color='w', ls='dashed')
    #    st.pyplot(fig)


    #@st.cache_data
    @st.fragment
    def page_monitoring():
        tab_hb, tab_mtb, tab_rb, tab_ltb, tab_pa, tab_pb = st.tabs(["Heartbeats", "MTB",  "RB","LTB","PA","PB"])
        kwargs = {'color'  : 'w',\
                  'alpha'  : 0.4,\
                  'ls'     : 'dashed'
                  }
                  #'lw'     : 0.9}
        if not st.session_state.use_dark_theme:
            kwargs['color'] = 'k'
        with tab_hb:
            with st.expander(f"Event builder HeartBeat"):
                df    = st.session_state.moni_data['evbh'].get_dataframe().filter((pl.col("timestamp") < 1200*3600))
                times = df['timestamp']/3600 
                
                fig   = gander_scatter_plot(times,100*df['data_mangled_ev']/df['n_mte_received_tot'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='TOF event builder - mangled events', xlabel='MET [h]', ylabel='Mangled events (\%)', **kwargs)
                st.pyplot(fig)
                
                fig   = gander_scatter_plot(times,100*df['n_timed_out']/df['n_mte_received_tot'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='TOF event builder - timed out events', xlabel='MET [h]', ylabel='Timed out events (\%)', **kwargs)
                st.pyplot(fig)
                
                fig   = gander_scatter_plot(times,df['event_id_cache_size'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='TOF event builder - Cache size (ids)', xlabel='MET [h]', ylabel='Cache size (event id)', **kwargs)
                st.pyplot(fig)
                
                fig   = gander_scatter_plot(times,df['event_cache_size'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='TOF event builder - Cache size (events)', xlabel='MET [h]', ylabel='Cache size (event id)', **kwargs)
                st.pyplot(fig)
                
                fig   = gander_scatter_plot(times,df['n_rbe_received_tot']/df['n_mte_received_tot'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='TOF event builder - RBEvents/TofEvent', xlabel='MET [h]', ylabel='N(RBEvents)/N(TofEvents)', **kwargs)
                st.pyplot(fig)
                
                fig   = gander_scatter_plot(times,100*df['n_rbe_orphan']/df['n_mte_received_tot'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='TOF event builder - Orphan RB events (\%)', xlabel='MET [h]', ylabel='N(RBEvents) lost', **kwargs)
                st.pyplot(fig) 
                
                fig   = gander_scatter_plot(times,100*df['n_rbe_discarded_tot']/df['n_mte_received_tot'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='TOF event builder - Number of discarded RB events (\%)', xlabel='MET [h]', ylabel='N(RBEvents) lost', **kwargs)
                st.pyplot(fig) 

                fig   = gander_scatter_plot(times,df['n_rbe_discarded_tot'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='TOF event builder - Number of discarded RB events', xlabel='MET [h]', ylabel='N(RBEvents) lost', **kwargs)
                st.pyplot(fig) 

                fig   = gander_scatter_plot(times,df['n_mte_skipped'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='TOF event builder - Number of discarded events from the MTB', xlabel='MET [h]', ylabel='N skipped', **kwargs)
                st.pyplot(fig)
                
                fig   = gander_scatter_plot(times,df['n_rbe_per_loop'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='TOF event builder - Number of RBEvents requested per MasterTriggerEvent', xlabel='MET [h]', ylabel='N skipped', **kwargs)
                st.pyplot(fig)
                
                fig   = gander_scatter_plot(times,df['rbe_wo_mte'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='TOF event builder - Number of RBEvents without MasterTrigger', xlabel='MET [h]', ylabel='N skipped', **kwargs)
                st.pyplot(fig)
                
                fig   = gander_scatter_plot(times,df['drs_bsy_lost_hg_hits'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='Lost Hits (DRS Busy)', xlabel='MET [h]', ylabel='N', **kwargs)
                st.pyplot(fig)
            
        with tab_rb:
            df = st.session_state.moni_data['rb'].get_dataframe().filter((pl.col("timestamp") < 1200*3600))
            available_boards   = set(df['board_id'])
            available_boards   = set([int(k) for k in available_boards])
            n_available_boards = len(available_boards) 
            if n_available_boards != len(ELIGIBLE_RBS):
                missing_boards = set(ELIGIBLE_RBS).difference(available_boards)
                for board in missing_boards:
                    st.warning(f"RB {board} has no monitoring data!", icon=":material/add_alert:")
            #average_rate =  df.select(pl.col('rate').mean())
            mean_rb_rate = df.group_by("board_id").agg(pl.col("rate").mean().alias("mean_rb_rate"))
            for board_id in available_boards:
                if (mean_rb_rate.filter((pl.col('board_id') == board_id)))['mean_rb_rate'].item() < 1:
                    st.warning(f"RB {board_id} has a rate of 0!", icon=":material/add_alert:")
            st.dataframe(mean_rb_rate)
            with st.expander('RBMoni data'):
                # sometimes the GCU time is weird, so we limit ourselves to a 50day flight
                # average rates
                for board in available_boards:
                    with st.expander(f"RB {board}"):
                        rbmoni  = df.filter(pl.col("board_id") == board)#.select("rate")
                        rbtimes = rbmoni['timestamp']/3600 # time in h

                        fig   = gander_scatter_plot(rbtimes,rbmoni['rate'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title=f'RB{int(board)} rate', xlabel='MET [h]', ylabel='Hz', **kwargs)
                        st.pyplot(fig)
                        
                        fig   = gander_scatter_plot(rbtimes,rbmoni['tmp_drs'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title=f'RB{int(board)} DRS TMP', xlabel='MET [h]', ylabel='C', **kwargs)
                        st.pyplot(fig)
                        
                        fig   = gander_scatter_plot(rbtimes,rbmoni['pressure'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title=f'Pressure', xlabel='MET [h]', ylabel='hPa', **kwargs)
                        st.pyplot(fig)
                        
                        fig   = gander_scatter_plot(rbtimes,rbmoni['humidity'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title=f'Humidity', xlabel='MET [h]', ylabel='rel. hum.', **kwargs)
                        st.pyplot(fig)
        
        with tab_ltb:
            df    = st.session_state.moni_data['ltb'].get_dataframe().filter((pl.col("timestamp") < 1200*3600))
            times = df['timestamp']/3600 
            
            # Last thresholds
            st.subheader(f"Last LTB moni data (MET {times[-1]:0.2}h)")
            st.divider()
            st.dataframe(df.tail(1))
            st.divider()  
            fig   = gander_scatter_plot(times,df['ltb_temp'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='LTB Temp', xlabel='MET [h]', ylabel='C', **kwargs)
            st.pyplot(fig)

        with tab_pa:
            with st.expander('PAMoni data'):
                df = st.session_state.moni_data['pa'].get_dataframe().filter((pl.col("timestamp") < 1200*3600))
                available_boards = set(df['board_id'])
                for board in available_boards:
                    rbmoni  = df.filter(pl.col("board_id") == board)#.select("rate")
                    rbtimes = rbmoni['timestamp']
                    rbtimes = rbtimes/3600
                    fig   = gander_scatter_plot(rbtimes,rbmoni['temps1'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title=f'PB for RB{int(board)} - T1', xlabel='MET [h]', ylabel='C', **kwargs)
                    st.pyplot(fig)

        with tab_mtb:
            with st.expander(f"MTB Data"):
                df = st.session_state.moni_data['mtb'].get_dataframe().filter((pl.col("timestamp") < 1200*3600))
                ##times = st.session_state.moni_data['mtb'].timestamps 
                times = df['timestamp']
                if times.eq(0).all():
                    times = np.array([k for k in range(len(times))])
                times =  times/3600.0
                fig   = gander_scatter_plot(times,df['rate'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='MTB rate', xlabel='MET [h]', ylabel='Hz', **kwargs)
                st.pyplot(fig)
                
                hist  = d.factory.hist1d(df['rate'], 45)
                fig   = gander_plot(hist,figsize=lo.FIGSIZE_A4_LANDSCAPE, title='MTB rate (from reg)', xlabel='Hz', gauss_fit=True)
                st.pyplot(fig)
                
                # lsot rate
                fig   = gander_scatter_plot(times,df['lost_rate'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='MTB rate (lost)', xlabel='MET [h]', ylabel='Hz', **kwargs)
                st.pyplot(fig)

                fig   = gander_scatter_plot(times,df['rb_lost_rate'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='MTB rate (lost, RB)', xlabel='MET [h]', ylabel='Hz', **kwargs)
                st.pyplot(fig)
                
                fig   = gander_scatter_plot(times,df['vccaux'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='VCCAUX', xlabel='MET [h]', ylabel='V', **kwargs)
                st.pyplot(fig)
                
                fig   = gander_scatter_plot(times,df['vccint'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='VCCINT', xlabel='MET [h]', ylabel='V', **kwargs)
                st.pyplot(fig)
                
                fig   = gander_scatter_plot(times,df['vccbram'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='VCCBRAM', xlabel='MET [h]', ylabel='V', **kwargs)
                st.pyplot(fig)
                
                fig   = gander_scatter_plot(times,df['daq_queue_len'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='DAQ LEN(QUEUE)', xlabel='MET [h]', ylabel='LEN', **kwargs)
                st.pyplot(fig)
                
                fig   = gander_scatter_plot(times,df['temp'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='MTB Temp', xlabel='MET [h]', ylabel='$^\circ$C', **kwargs)
                st.pyplot(fig)
                
                fig   = gander_scatter_plot(times,df['tiu_busy_len'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='Last reported trk busy in sample', xlabel='MET [h]', ylabel='$\mu$s/10', **kwargs)
                st.pyplot(fig)
            
                fig   = gander_scatter_plot(times,df['tiu_busy_len'],figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='Last reported trk busy in sample', xlabel='MET [h]', ylabel='$\mu$s/10', **kwargs)
                st.pyplot(fig)

                hist  = d.factory.hist1d(df['tiu_busy_len'], int(len(times)/15))
                fig   = gander_plot(hist,figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT, title='Sampled TRK BUSY', xlabel='$\mu$s/10')
                st.pyplot(fig)

    # The app
    st.logo('../resources/assets/GAPSLOGO_2023.png', size='large')
    pg = st.navigation([st.Page(page_run           , title="Data source & setup"), 
                        st.Page(page_event_view    , title="Event view"),
                        st.Page(page_reco          , title="Reconstruction"),
                        st.Page(page_nhit          , title="NHit distributions"),
                        st.Page(page_occupancy     , title="Occupancy"),
                        st.Page(page_energy        , title="Energy deposition"),
                        st.Page(page_beta          , title="Beta Analysis"),
                        st.Page(page_tofpulses     , title="Paddle Analysis"),
                        st.Page(page_monitoring    , title="Monitoring data")
                        ])
    
    pg.run()    
