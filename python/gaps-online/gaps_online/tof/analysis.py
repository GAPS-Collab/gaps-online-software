# the speed of light in a tof paddle
C_LIGHT_PADDLE = 15.4

try:
    import django
    django.setup()
    from .. import db
    import os
    os.environ['DJANGO_ALLOW_ASYNC_UNSAFE'] = '1'

except Exception as e:
    print(f"Can't load django environment, gaps_db will not be available. {e}")

_pybind_imported = False

try:
    import go_pybindings
    create_mtb_connection_to_pid_map = go_pybindings.io.create_mtb_connection_to_pid_map
    _pybind_imported = True
except ImportError as e:
    print (f'-> Unable to import pybindings! {e}')

import numpy as np
import dashi as d
import tqdm
import sys
#from loguru import logger
#logger.add(sys.stdout,level='INFO')

from copy import deepcopy as copy

from ..events import EventStatus
from ..db import get_tof_paddles

#########################################################

def find_paddle(hit, paddles):
    """
    Get a paddle id for a trigger hit
    where the trigger hit is (dsi, j, ch)
    """
    # hit is dsi, ch, channel
    for pdl in paddles:
        if pdl.dsi == hit[0]:
            if pdl.j_ltb == hit[1]:
                if pdl.ltb_chA == hit[2][0]:
                    return pdl.paddle_id
                elif pdl.ltb_chB == hit[2][0]:
                    return pdl.paddle_id
    print (f'No paddle found for {hit}')

#########################################################

def distance(h0, h1) -> float:
    """ The spatial distance between 2 TofHits """
    return np.sqrt((h0.x - h1.x)**2 + (h0.y - h1.y) ** 2 + (h0.z - h1.z) ** 2) 

#########################################################

def create_occupancy_dict(reader           = None,
                          events           = [],
                          normalize        = True,
                          use_trigger_hits = False,
                          mark_0_as_bad    = False,
                          cbe_side         = True,
                          cor_side         = True):
    """
    Create a dictionary of paddle id vs nhits

    This can either accept a reader or a list of events.
    Use reader when memory is sparse and events when time is
    of the essence
    
    # Arguments:
        * reader           - either TofPacket or TelemetryPacketReader. The reader should be primed in a way
                             that it only spits out MergedEvents, TofEventSummary or TofEvents
    
        * use_trigger_hits - instead of plotting TofHits, just use the triggered hits for the occupancy
        * cbe_side         - add the CBE sides to the occupancy dictionary. It might make sense to exclude 
                             them for normalization reasons
        * cor_side         - add the COR sides to the occupancy dictionary. It might make sense to exclude 
                             them for normalization reasons
    """

    if reader is not None and events:
        raise ValueError("Unable to use both, reader and events!")

    if use_trigger_hits:
        paddles = db.get_tof_paddles()

    if reader is not None:
        for ev in reader:
            ev0 = ev
            break;
    else:
        # events can be TofEventSummary or TofEvent
        ev0 = events[0]

    is_tes = False
    if hasattr(ev0,'trigger_hits'):
        is_tes = True

    is_merged_event = False
    if hasattr(ev0,'tof'):
        is_merged_event = True

    occu_per_paddle = {k : 0 for k in range(1,161)}
    if reader is not None:
        events = reader
    for ev in tqdm.tqdm(events, desc='Getting TOF occupancy data!'):
        if use_trigger_hits:
            if is_tes:
                for h in ev.trigger_hits:
                    pid = find_paddle(h, paddles)
                    occu_per_paddle[pid] += 1
            elif is_merged_event:
                try:
                    ev = ev.tof
                except:
                    continue
                if trigger_hits:
                    for h in ev.trigger_hits:
                        pid = find_paddle(h, paddles)
                        occu_per_paddle[pid] += 1
                else:
                    for h in ev.hits:
                        pid = find_paddle(h, paddles)
                        occu_per_paddle[pid] += 1

            else:
                for h in ev.mastertriggerevent.trigger_hits:
                    pid = find_paddle(h, paddles)
                    occu_per_paddle[pid] += 1

        else:
            for h in ev.hits:
                occu_per_paddle[h.paddle_id] += 1

    if not cbe_side:
        for pid in range(25, 61):
            occu_per_paddle.pop(pid)
    if not cor_side:
        for pid in range(109, 161):
            occu_per_paddle.pop(pid)
    # normalize it
    if normalize:
        max_val = max(occu_per_paddle.values())
        for k in occu_per_paddle.keys():
            occu_per_paddle[k] = occu_per_paddle[k]/max_val
    if mark_0_as_bad:
        for k in occu_per_paddle.keys():
            if occu_per_paddle[k] == 0.0:
                occu_per_paddle[k] = np.nan
    return occu_per_paddle

##################################################################

def calc_rms(data) -> float:
    """ root mean square calculation """
    return np.sqrt((data ** 2).sum() / len(data))


##################################################################

class TofAnalysis:
    """
    A container (yeah I know, don't like it either) to keep a 
    bunch of plots together.

    This does have some use as a pre-compiled analysis for 
    gander, and as a quick look kind of thing.

    The gist here it is independent of the data source, as 
    long as some kind of TofEvent can be plugged in.
    """
    # bins
    NBINS              = 70
    PADDLE_PEAK_BINS   = np.linspace(0,300, NBINS)
    PADDLE_CHARGE_BINS = np.linspace(0,50 , NBINS)
    PADDLE_TIMING_BINS = np.linspace(0,500, NBINS)
    PADDLE_BL_BINS     = np.linspace(-5,5 , NBINS)
    PADDLE_BLRMS_BINS  = np.linspace(0,5  , NBINS)
    PADDLE_X0_BINS     = np.linspace(-0.1, 1.1,NBINS)
    PADDLE_T0_BINS     = np.linspace(0,500, NBINS)
    PADDLE_EDEP_BINS   = np.linspace(0,10 , NBINS)
    NHIT_BINS          = np.arange(-0.5,25.5,1)        
    PID_BINS           = np.arange(0.5,160.5,1)
    BETA_BINS          = np.linspace(0,4  , NBINS)

    @staticmethod
    def _timing_plots():
        tmg_plots = {
          'beta'  : d.histogram.hist1d(TofAnalysis.BETA_BINS),
        }
        return tmg_plots

    @staticmethod
    def _nhit_plots():
        nhit_plots = {
          'hit'      : d.histogram.hist1d(TofAnalysis.NHIT_BINS),
          'thit'     : d.histogram.hist1d(TofAnalysis.NHIT_BINS),
          'rblink'   : d.histogram.hist1d(TofAnalysis.NHIT_BINS),
          'miss_hit' : d.histogram.hist1d(TofAnalysis.PID_BINS)
        }
        return nhit_plots

    @staticmethod
    def _paddle_plots():
        """
        Charge and timing plots for each paddle
        """
        paddle_plots = {\
          'charge2d'  : d.histogram.hist2d((TofAnalysis.PADDLE_CHARGE_BINS, TofAnalysis.PADDLE_CHARGE_BINS)),
          'amp_a'     : d.histogram.hist1d(TofAnalysis.PADDLE_PEAK_BINS),
          'amp_b'     : d.histogram.hist1d(TofAnalysis.PADDLE_PEAK_BINS),
          'time_a'    : d.histogram.hist1d(TofAnalysis.PADDLE_TIMING_BINS),
          'time_b'    : d.histogram.hist1d(TofAnalysis.PADDLE_TIMING_BINS),
          'bl_a'      : d.histogram.hist1d(TofAnalysis.PADDLE_BL_BINS),
          'bl_b'      : d.histogram.hist1d(TofAnalysis.PADDLE_BL_BINS),
          'bl_a_rms'  : d.histogram.hist1d(TofAnalysis.PADDLE_BLRMS_BINS),
          'bl_b_rms'  : d.histogram.hist1d(TofAnalysis.PADDLE_BLRMS_BINS),
          'x0'        : d.histogram.hist1d(TofAnalysis.PADDLE_X0_BINS),
          't0'        : d.histogram.hist1d(TofAnalysis.PADDLE_T0_BINS),
          'edep'      : d.histogram.hist1d(TofAnalysis.PADDLE_EDEP_BINS),
          'pos_edep'  : d.histogram.hist2d((TofAnalysis.PADDLE_X0_BINS, TofAnalysis.PADDLE_EDEP_BINS))
        }
        paddle_hists = {k : copy(paddle_plots) for k in range(1,161)}
        return paddle_hists

    def __init__(self, skip_mangled = True, skip_timeout = True, beta_analysis = True):
        self.skip_mangled  = skip_mangled
        self.skip_timeout  = skip_timeout
        self.beta_analysis = beta_analysis
        self.n_mangled     = 0
        self.n_timed_out   = 0
        self.n_events      = 0
        self.paddle_plots  = TofAnalysis._paddle_plots()
        self.nhit_plots    = TofAnalysis._nhit_plots()
        self.tmg_plots     = TofAnalysis._timing_plots()
        self.paddles       = get_tof_paddles()
        if _pybind_imported:
            self.hg_mapping   = create_mtb_connection_to_pid_map()
        # hit histogram
        self.nhit          = 0
        self.no_hitmiss    = 0
        self.one_hitmiss   = 0
        self.two_hitmiss   = 0
        self.extra_hits    = 0
        self.occupancy     = {k : 0 for k in range(1,161)}
        self.occupancy_t   = {k : 0 for k in range(1,161)}
        # beta analysis
        self.pid_inner    = None
        self.pid_outer    = None


    def add_event(self, ev):
        """
        Fills the associated histograms
        
        # Arguments:
            * ev : Any kind of TofEvent or TofEventSummary
        """
        if ev.status == EventStatus.AnyDataMangling:
            #logger.debug(f'Found mangled event with id {ev.event_id}')
            self.n_mangled += 1
            if self.skip_mangled:
                return
        if ev.status == EventStatus.EventTimeOut:
            #logger.debug(f'Found timed out event with id {ev.event_id}')
            if self.skip_timeout:
                return
        # FIXME - speed these up
        nhit_ev        = 0
        nhit_t_ev      = 0
        self.n_events += 1
        
        # trigger occupancy histogram
        for h in ev.trigger_hits:
            pid = find_paddle(h, self.paddles.values())
            self.occupancy_t[pid] += 1
            nhit_t_ev += 1
        if self.beta_analysis:
            outer_h = []
            inner_h = []
        for h in ev.hits:
            pdl = self.paddles[h.paddle_id]
            h.set_paddle(10*pdl.length, pdl.cable_len, pdl.coax_cable_time, pdl.harting_cable_time)
            self.paddle_plots[h.paddle_id]['charge2d' ].fill(np.array([[h.charge_a,h.charge_b]]))
            self.paddle_plots[h.paddle_id]['amp_a'    ].fill(np.array([h.peak_a]))  
            self.paddle_plots[h.paddle_id]['amp_b'    ].fill(np.array([h.peak_b]))  
            self.paddle_plots[h.paddle_id]['time_a'   ].fill(np.array([h.time_a]))  
            self.paddle_plots[h.paddle_id]['time_b'   ].fill(np.array([h.time_b]))  
            self.paddle_plots[h.paddle_id]['bl_a'     ].fill(np.array([h.baseline_a]))  
            self.paddle_plots[h.paddle_id]['bl_b'     ].fill(np.array([h.baseline_b]))  
            self.paddle_plots[h.paddle_id]['bl_a_rms' ].fill(np.array([h.baseline_a_rms]))  
            self.paddle_plots[h.paddle_id]['bl_b_rms' ].fill(np.array([h.baseline_b_rms]))  
            self.paddle_plots[h.paddle_id]['x0'       ].fill(np.array([h.pos/h.paddle_len]))
            self.paddle_plots[h.paddle_id]['t0'       ].fill(np.array([h.t0_uncorrected]))
            self.paddle_plots[h.paddle_id]['edep'     ].fill(np.array([h.edep]))
            self.paddle_plots[h.paddle_id]['pos_edep' ].fill(np.array([[h.pos/h.paddle_len, h.edep]]))
            if h.edep > 0:
                self.occupancy[h.paddle_id] += 1
            nhit_ev += 1
            if self.beta_analysis:
                if self.pid_outer is None:
                    if h.paddle_id > 60:
                        outer_h.append(h)
                else:
                    if h.paddle_id == self.pid_outer:
                        outer_h.append(h)
                if self.pid_inner is None:
                    if h.paddle_id < 61:
                        inner_h.append(h)
                else:
                    if h.paddle_id == self.pid_inner:
                        inner_h = inner_h.append(h)
        
        # hit counting 
        n_rblink_ev    = len(ev.rb_link_ids)
        self.nhit     += nhit_ev
        if nhit_t_ev == nhit_ev:
            self.no_hitmiss += 1 
        elif (nhit_t_ev - nhit_ev) == 1:
            self.one_hitmiss += 1
        elif (nhit_t_ev - nhit_ev) > 1:
            self.two_hitmiss += 1
        elif (nhit_ev > nhit_t_ev):
            self.extra_hits += 1
        
        self.nhit_plots['hit'     ].fill(np.array([nhit_ev])) 
        self.nhit_plots['thit'    ].fill(np.array([nhit_t_ev])) 
        self.nhit_plots['rblink'  ].fill(np.array([n_rblink_ev])) 
        if _pybind_imported:
            # FIXME - this returns bytes and should return ints
            missing        = [int(k) for k in ev.get_missing_paddles_hg(self.hg_mapping)]
            self.nhit_plots['miss_hit'].fill(np.array(missing)) 

        if not self.beta_analysis:
            return
        
        outer_h = sorted(outer_h, key=lambda x: x.t0)
        inner_h = sorted(inner_h, key=lambda x: x.t0)
        if inner_h and outer_h:
            #first_hit = sorted([h for h in ev.hits], key=lambda x: x.phase_delay)
            #last_hit  = first_hit[-1].phase_delay
            #first_hit = first_hit[0].phase_delay
            #print (inner_h, outer_h)
            diff_h  = inner_h[0].t0 - outer_h[0].t0 
            beta = (distance(inner_h[0],outer_h[0])/1000)/(diff_h*1e-9)/299792458
            if beta < 0:
                beta = -1*beta
            #print (beta, diff_h)
            #raise
            self.tmg_plots['beta'].fill(np.array([beta]))   
             #phase_delay = inner_h[0].phase_delay - outer_h[0].phase_delay
             #if True:
             ##if (phase_delay > 20 or phase_delay < -20):
             #    #    if inner_h[0].phase_delay - outer_h[0].phase_delay > 40:
             #    result['histo_beta'].fill(np.array([beta])) 
             #    result['last_pd_outer'] = outer_h[0].phase_delay
             #    result['last_pd_inner'] = inner_h[0].phase_delay
             #    for wf in tof_ev.waveforms:
             #        if wf.paddle_id == outer_h[0].paddle_id:
             #            rbid = wf.rb_id
             #            for rbev in tof_ev.rbevents:
             #                if rbev.header.rb_id == rbid:
             #                    result['wf_outer'] = rbev.get_waveform(8)
             #        if wf.paddle_id == inner_h[0].paddle_id:
             #            rbid = wf.rb_id
             #            for rbev in tof_ev.rbevents:
             #                if rbev.header.rb_id == rbid:
             #                    result['wf_inner'] = rbev.get_waveform(8)
             #    #print (ev)
             #    result['histo_t_diff_fst'].fill(np.array([last_hit - first_hit]))
             #    result['histo_nhit'].fill(np.array([len(hits)]))
             #    result['histo_t_diff'  ].fill(np.array([diff_h]))
             #    result['histo_t_inner' ].fill(np.array([inner_h[0].t0]))
             #    result['histo_t_outer' ].fill(np.array([outer_h[0].t0]))
             #    result['histo_dist'].fill(np.array([distance(inner_h[0], outer_h[0])/1000]))
             #    
             #    result['histo_pdelay'].fill(np.array([phase_delay]))
             #    result['histo_ph_out'].fill(np.array([outer_h[0].phase_delay]))
             #    result['histo_ph_in'].fill(np.array([inner_h[0].phase_delay]))

             #    result['histo_hit_pid'].fill(np.array([inner_h[0].paddle_id]))
             #    result['histo_hit_pid'].fill(np.array([outer_h[0].paddle_id]))



##################################################################


