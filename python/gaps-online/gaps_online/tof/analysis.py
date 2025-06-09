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
import json

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

class TofCuts:
    """
    A simple container to hold a collection of TOF relevant cut 
    values
    """
    
    def __init__(self):
        self.min_hit_cor      = 0
        self.min_hit_cbe      = 0
        self.min_hit_umb      = 0
        self.max_hit_cor      = 161
        self.max_hit_cbe      = 161
        self.max_hit_umb      = 161
        self.only_causal_hits = False
        self.hit_cbe_acc      = 0 
        self.hit_umb_acc      = 0 
        self.hit_cor_acc      = 0
        self.nevents          = 0
        self.hits_total       = 0
        self.hits_rmvd_csl    = 0
        self.hits_rmvd_ls     = 0
        # Require that the first hit MUST
        # be on the umbrella!
        self.fh_must_be_umb   = False
        self.fh_umb_acc       = False
        self.ls_cleaning_t_err = np.inf

    def clear_stats(self):
        """
        Zero out the event/hit counter variables
        """
        self.hit_cbe_acc      = 0 
        self.hit_umb_acc      = 0 
        self.hit_cor_acc      = 0
        self.nevents          = 0
        self.hits_total       = 0
        self.hits_rmvd_csl    = 0
        self.hits_rmvd_ls     = 0
        self.fh_umb_acc       = 0


    @property
    def void(self):
        if self.min_hit_cor      != 0:
            return False
        if self.min_hit_cbe      != 0:
            return False
        if self.min_hit_umb      != 0:
            return False
        if self.max_hit_cor      != 161:
            return False
        if self.max_hit_cbe      != 161:
            return False
        if self.max_hit_umb      != 161:
            return False
        if self.only_causal_hits:
            return False
        if self.ls_cleaning_t_err != np.inf:
            return False
        if self.fh_must_be_umb != False:
            return False
        return True

        
    def is_compatible(self, other):
        """
        Void cuts will autmaticaly be compatible
        """
        if self.only_causal_hits != other.only_causal_hits:
            return False
        if self.min_hit_cor  != other.min_hit_cor:
            return False
        if self.min_hit_cbe  != other.min_hit_cbe:
            return False
        if self.min_hit_umb  != other.min_hit_umb:
            return False
        if self.max_hit_cor  != other.max_hit_cor:
            return False
        if self.max_hit_cbe  != other.max_hit_cbe:
            return False
        if self.max_hit_umb  != other.max_hit_umb:
            return False
        if self.ls_cleaning_t_err != other.ls_cleaning_t_err:
            return False
        if self.fh_must_be_umb != other.fh_must_be_umb:
            return False
        return True

    def __iadd__(self, other):
        if not self.is_compatible(other):
            raise ValueError("Cuts are not compatible!")
        self.nevents       += other.nevents
        self.hit_cbe_acc   += other.hit_cbe_acc 
        self.hit_umb_acc   += other.hit_umb_acc 
        self.hit_cor_acc   += other.hit_cor_acc
        self.hits_total    += other.hits_total
        self.hits_rmvd_csl += other.hits_rmvd_csl
        self.hits_rmvd_ls  += other.hits_rmvd_ls
        self.fh_umb_acc    += other.fh_umb_acc
        return self

    def __add__(self, other):
        new_cuts = TofCuts()
        if not self.is_compatible(other):
            raise ValueError("Cuts are not compatible!")
        new_cuts += self
        new_cuts += other
        #new_cuts.nevents = self.nevents + other.nevents
        #new_cuts.hit_cbe_acc      = self.hit_cbe_accn  + other.hit_cbe_acc 
        #new_cuts.hit_umb_acc      = self.hit_umb_accn  + other.hit_umb_acc 
        #new_cuts.hit_cor_acc      = self.hit_cor_accn  + other.hit_cor_acc
        #new_cuts.hits_total       = self.hits_total    + other.hits_total
        #new_cuts.hits_rmvd_csl    = self.hits_rmvd_csl + other.hits_rmvd_csl
        #new_cuts.hits_rmvd_ls     = self.hits_rmvd_ls  + other.hits_rmvd_ls
        
        return new_cuts

    @property
    def acc_frac_hit_cbe(self):
        if self.nevents == 0:
            return 0
        return self.hit_cbe_acc/self.nevents
    
    @property
    def acc_frac_hit_cor(self):
        if self.nevents == 0:
            return 0
        return self.hit_cor_acc/self.nevents
    
    @property
    def acc_frac_hit_umb(self):
        if self.nevents == 0:
            return 0
        return self.hit_umb_acc/self.nevents
    
    @property
    def acc_frac_fh_is_umb(self):
        if self.nevents == 0:
            return 0
        return self.fh_umb_acc/self.nevents

    def pretty_print_efficiency(self):
        _repr =  f'-- -- -- -- -- -- -- -- -- -- --'
        _repr +=  f'\n TOTAL EVENTS : {self.nevents}'
        _repr += f'\n  {self.min_hit_umb} <= NHit(UMB) <= {self.max_hit_umb} : {100*self.acc_frac_hit_umb : .2f} %' 
        _repr += f'\n  {self.min_hit_cbe} <= NHit(CBE) <= {self.max_hit_cbe} : {100*self.acc_frac_hit_cbe : .2f} %' 
        _repr += f'\n  {self.min_hit_cor} <= NHit(COR) <= {self.max_hit_cor} : {100*self.acc_frac_hit_cor : .2f} %' 
        if self.only_causal_hits:
            if self.hits_total > 0:
                _repr += f'\n Removed {100*self.hits_rmvd_csl/self.hits_total:.2f} % of hits due to causality cut!'
        if self.ls_cleaning_t_err != np.inf:
            if self.hits_total > 0:
                _repr += f'\n Removed {100*self.hits_rmvd_ls/self.hits_total:.2f} % of hits due to lightspeed cut!'
        if self.fh_must_be_umb:
            _repr += f'\n First hit must be on UMB!'
            _repr += f'\n   -- Removed {100*self.acc_frac_fh_is_umb : .2f} %'
        _repr +=  f'\n-- -- -- -- -- -- -- -- -- -- --'
        return _repr 

    #def accept(self, ev) -> bool: 
    #    return True

    def __str__(self):
        return self.__repr__()

    def __repr__(self):
        _repr  = '<TofCuts:'
        if self.only_causal_hits:
            _repr += f'\n -- removes non-causal hits!'
        if self.ls_cleaning_t_err != np.inf:
            _repr += f'\n -- removes hits which are not correlated with the first hit!'
            _repr += f'\n --   assumed timing error {self.ls_cleaning_t_err}'
        if self.fh_must_be_umb:
            _repr += f'\n -- first hit must be on UMB'
        _repr += f'\n  {self.min_hit_umb} <= NHit(UMB) <= {self.max_hit_umb}' 
        _repr += f'\n  {self.min_hit_cbe} <= NHit(CBE) <= {self.max_hit_cbe}' 
        _repr += f'\n  {self.min_hit_cor} <= NHit(COR) <= {self.max_hit_cor}' 
        _repr += '>'
        return _repr

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

    def define_bins(self, nbins = 70):
        """
        Set the bins for the different histograms for the 
        variables. Only the number of bins can be set
        """
        self.PADDLE_PEAK_BINS   = np.linspace(0,300,     nbins)
        self.PADDLE_CHARGE_BINS = np.linspace(0,50 ,     nbins)
        self.PADDLE_TIMING_BINS = np.linspace(0,500,     nbins)
        self.PADDLE_BL_BINS     = np.linspace(-5,5 ,     nbins)
        self.PADDLE_BLRMS_BINS  = np.linspace(0,5  ,     nbins)
        self.PADDLE_X0_BINS     = np.linspace(-0.1, 1.1, nbins)
        self.PADDLE_T0_BINS     = np.linspace(0,500,     nbins)
        self.PADDLE_EDEP_BINS   = np.linspace(0,10 ,     nbins)
        self.NHIT_BINS          = np.arange(-0.5,25.5,1)   
        self.PID_BINS           = np.arange(0.5,160.5,1)
        self.BETA_BINS          = np.linspace(0,2  ,     nbins)
        self.TIMING_BINS        = np.linspace(-100, 300, nbins)
        self.PDELAY_BINS        = np.linspace(-60,60,    nbins)

    def pretty_print_statistics(self):
        """
        A textual representation for some important numbers, e.g.
        seen events, cut efficiencies, etc.
        """
        _repr = "\n-- -- -- -- -- -- -- -- -- "
        _repr += "\n TOF analysis statistics"
        if not self.cuts.void:
            _repr += f'\n  -- nevents (no    cut)      : {self.cuts.nevents}'
        else:
            _repr += f'\n  -- nevents                  : {self.n_events}'
        _repr += f'\n  -- runtime (s)              : {self.run_time:1f} s'
        _repr += f'\n  -- rate    (Hz (nocut))     : {self.rate_nocut:.2f} Hz'
        _repr += f'\n  -- frac. of mangled events  : {100*self.n_mangled_frac : .2f} %'
        _repr += f'\n  -- frac. of timedout events : {100*self.n_timed_out_frac : .2f} %' 
        if not self.cuts.void:
            _repr += '\n  -- -- applied cut:'
            _repr += f'\n\t -- -- {self.cuts}'
            _repr += f'\n\t  -- -- nevents (after cut) : {self.n_events}'
            if self.cuts.nevents > 0:
                _repr += f'\n\t  -- -- efficiency          : {100*self.n_events/self.cuts.nevents : .2f} %'
            _repr += f'\n\t  -- -- rate    (Hz)        : {self.rate: .2f} Hz'
            
        return _repr

    def _timing_plots(self):
        tmg_plots = {
          'beta'     : d.histogram.hist1d(self.BETA_BINS),
          't_inner'  : d.histogram.hist1d(self.TIMING_BINS),
          't_outer'  : d.histogram.hist1d(self.TIMING_BINS),
          # timing difference will not be larger than phase delay!
          't_diff'   : d.histogram.hist1d(self.PDELAY_BINS),
          'ph_delay' : d.histogram.hist1d(self.PDELAY_BINS)
        }
        return tmg_plots

    def _nhit_plots(self):
        nhit_plots = {
          'hit'      : d.histogram.hist1d(self.NHIT_BINS),
          'thit'     : d.histogram.hist1d(self.NHIT_BINS),
          'rblink'   : d.histogram.hist1d(self.NHIT_BINS),
          'miss_hit' : d.histogram.hist1d(self.PID_BINS),
          # these are non causal hits
          'nc_pdls'  : d.histogram.hist1d(self.PID_BINS)
        }
        return nhit_plots

    def _paddle_plots(self):
        """
        Charge and timing plots for each paddle
        """
        paddle_plots = {\
          'charge2d'  : d.histogram.hist2d((self.PADDLE_CHARGE_BINS, self.PADDLE_CHARGE_BINS)),
          'amp_a'     : d.histogram.hist1d(self.PADDLE_PEAK_BINS),
          'amp_b'     : d.histogram.hist1d(self.PADDLE_PEAK_BINS),
          'time_a'    : d.histogram.hist1d(self.PADDLE_TIMING_BINS),
          'time_b'    : d.histogram.hist1d(self.PADDLE_TIMING_BINS),
          'bl_a'      : d.histogram.hist1d(self.PADDLE_BL_BINS),
          'bl_b'      : d.histogram.hist1d(self.PADDLE_BL_BINS),
          'bl_a_rms'  : d.histogram.hist1d(self.PADDLE_BLRMS_BINS),
          'bl_b_rms'  : d.histogram.hist1d(self.PADDLE_BLRMS_BINS),
          'x0'        : d.histogram.hist1d(self.PADDLE_X0_BINS),
          't0'        : d.histogram.hist1d(self.PADDLE_T0_BINS),
          'edep'      : d.histogram.hist1d(self.PADDLE_EDEP_BINS),
          'pos_edep'  : d.histogram.hist2d((self.PADDLE_X0_BINS, self.PADDLE_EDEP_BINS))
        }
        paddle_hists = {k : copy(paddle_plots) for k in range(1,161)}
        return paddle_hists

    @property
    def rate(self):
        if self.run_time == 0:
            return 0
        return self.n_events / self.run_time

    @property
    def rate_nocut(self):
        if self.run_time == 0:
            return 0
        if self.cuts.void:
            return self.n_events / self.run_time
        return self.cuts.nevents / self.run_time

    @property
    def run_time(self):
        """
        Get run time from last - first event in seconds
        """
        #print (f'LAST EV TIME  {self.last_ev_time}')
        #print (f'FIRST EV TIME {self.first_ev_time}')
        return 1e-5*(self.last_ev_time - self.first_ev_time)

    def reinit(self, nbins = 90):
        """
        Re-run the initialization routine. This will clear all plots, and 
        reset the binning. This needs to be run in case the binning has
        been changed
        """
        self.__init__(skip_mangled  = self.skip_mangled,
                      skip_timeout  = self.skip_timeout,
                      beta_analysis = self.beta_analysis,
                      nbins         = nbins)

    def __init__(self, skip_mangled = True,\
                 skip_timeout = True,\
                 beta_analysis = True,\
                 nbins = 90,
                 cuts : TofCuts  = TofCuts(),
                 use_offsets = False,
                 active = False):
        """
        Start a new TofAnalysis. This will add create histograms for 
        'interesting' variables and count mangled and timed out 
        events. While not complete, this can provide a conciese, 
        first look for a run.
        Events can be added to this analysis through the .add_event(ev)
        method. When all events are added, a call to .finish() is needed
        to make sure all events in the caches are added to the histograms.
        Caching is used to massively improve performance, since adding
        individual numbers to dashi.histograms is painfully slow.
        
        # Arguments:
        
          skip_mangled             : Ignore events which have the "AnyDataMangling" 
                                     flag set
          skip_timeout             : Ignore events which have the "EventTimedOut"
                                     flag set
          beta_analysis            : Look for first hit on outer tof/inner tof and 
                                     use these for a beta calculation. If pid_outer
                                     and pid_inner are given, use these paddles 
                                     instead.
          nbins                    : The number of bins for the histograms getting
                                     created
          cuts                     : Give a cut instance to reject events & hits.
                                     Default: None (no cuts)
          active                   : if True, this analysis will actually "do something"
                                     and acquire events
        """
        # process kwargs
        self.skip_mangled              = skip_mangled
        self.skip_timeout              = skip_timeout
        self.beta_analysis             = beta_analysis
        self.use_offsets               = use_offsets
        self.nbins                     = nbins
        self.cuts                      = cuts
        self.active                    = active 
        self.offsets     = None
        if self.use_offsets:
            self.offsets = dict()
            offsets = json.load(open('offsets.json'))
            for k in offsets:
                k_int = int(k)
                # currently we only have intra-panel calibrations
                if k_int <= 12:
                    self.offsets[k_int] = offsets[k]
        
        self.define_bins(nbins = self.nbins)
        self.first_ev_time = np.inf
        self.last_ev_time  = 0
        self.finished      = False
        self.n_mangled     = 0
        self.n_timed_out   = 0
        self.n_events      = 0
        self.paddle_plots  = self._paddle_plots()
        self.nhit_plots    = self._nhit_plots()
        self.tmg_plots     = self._timing_plots()
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
        self.pid_inner     = None
        self.pid_outer     = None
        #######################################################
        # caches - filling dashi histograms is very slow 
        # (it is not made for it). Work around that by only 
        # calling fill every 10000th event or so
        #######################################################
        # cache size for histograms with 1 entry/hit
        self.hit_cache_size     = int(1e6)
        # cache size for histograms with 1 entry/event
        self.event_cache_size   = int(1e6) 
        self.c_hit              = []
        self.c_thit             = []
        self.c_rblink           = []
        self.c_miss_hit         = []
        self.c_nc_pid           = []
        self.c_beta             = []
        self.c_t_inner          = []
        self.c_t_outer          = []
        self.c_t_diff           = []
        self.c_ph_delay         = []
        self.c_charges          = {k:[] for k in range(1,161)}
        self.c_peaks_a          = {k:[] for k in range(1,161)}
        self.c_peaks_b          = {k:[] for k in range(1,161)}
        self.c_times_a          = {k:[] for k in range(1,161)}
        self.c_times_b          = {k:[] for k in range(1,161)}
        self.c_baselines_a      = {k:[] for k in range(1,161)}
        self.c_baselines_b      = {k:[] for k in range(1,161)}
        self.c_baselines_a_rms  = {k:[] for k in range(1,161)}
        self.c_baselines_b_rms  = {k:[] for k in range(1,161)}
        self.c_positions        = {k:[] for k in range(1,161)}
        self.c_t0s              = {k:[] for k in range(1,161)}
        self.c_edeps            = {k:[] for k in range(1,161)}
        self.c_pos_edeps        = {k:[] for k in range(1,161)}
   
    @property
    def n_mangled_frac(self):
        if self.n_events > 0:
            return self.n_mangled/self.n_events
        else:
            return 0

    @property
    def n_timed_out_frac(self):
        if self.n_events > 0:
            return self.n_timed_out/self.n_events
        else:
            return 0

    def _is_compatible(self, other):
        if self.beta_analysis != other.beta_analysis:
            return False
        if self.pid_inner != other.pid_inner:
            return False
        if self.pid_outer != other.pid_outer:
            return False
        if not self.cuts.is_compatible(other.cuts):
            return False
        return True

    def __iadd__(self, other):
        if not self._is_compatible(other):
            raise ValueError("Analysis are not compatible! Both must have the same setup!")
        if other.first_ev_time < self.first_ev_time:
            self.first_ev_time = other.first_ev_time
        if other.last_ev_time > self.last_ev_time:
            self.last_ev_time = other.last_ev_time
        self.cuts          += other.cuts
        self.skip_mangled  += other.skip_mangled
        self.skip_timeout  += other.skip_timeout
        self.n_mangled     += other.n_mangled 
        self.n_timed_out   += other.n_timed_out 
        self.n_events      += other.n_events
        for pid in range(1,161):
            for k in self.paddle_plots[pid]:
                self.paddle_plots[pid][k] += other.paddle_plots[pid][k]
        for k in self.nhit_plots:
            self.nhit_plots[k] += other.nhit_plots[k]
        for k in self.tmg_plots:
            self.tmg_plots[k] += other.tmg_plots[k]
        # hit histogram
        self.nhit          += other.nhit 
        self.no_hitmiss    += other.no_hitmiss
        self.one_hitmiss   += other.one_hitmiss
        self.two_hitmiss   += other.two_hitmiss
        self.extra_hits    += other.extra_hits
        for pid in range(1,161):
            self.occupancy[pid]   += other.occupancy[pid] 
            self.occupancy_t[pid] += other.occupancy_t[pid] 
        self.c_hit             .extend(other.c_hit) 
        self.c_thit            .extend(other.c_thit) 
        self.c_rblink          .extend(other.c_rblink) 
        self.c_miss_hit        .extend(other.c_miss_hit) 
        self.c_nc_pid          .extend(other.c_nc_pid)
        self.c_t_inner         .extend(other.c_t_inner) 
        self.c_t_outer         .extend(other.c_t_outer) 
        self.c_t_diff          .extend(other.c_t_diff) 
        self.c_ph_delay        .extend(other.c_ph_delay) 
        if _pybind_imported: # FIXME - remove all these ifs
            self.c_beta            .extend(other.c_beta)
        for pid in range(1,161):
            self.c_charges[pid]        .extend(other.c_charges[pid]) 
            self.c_peaks_a[pid]        .extend(other.c_peaks_a[pid]) 
            self.c_peaks_b[pid]        .extend(other.c_peaks_b[pid]) 
            self.c_times_a[pid]        .extend(other.c_times_a[pid]) 
            self.c_times_b[pid]        .extend(other.c_times_b[pid]) 
            self.c_baselines_a[pid]    .extend(other.c_baselines_a[pid]) 
            self.c_baselines_b[pid]    .extend(other.c_baselines_b[pid]) 
            self.c_baselines_a_rms[pid].extend(other.c_baselines_a_rms[pid])
            self.c_baselines_b_rms[pid].extend(other.c_baselines_b_rms[pid])
            self.c_positions[pid]      .extend(other.c_positions[pid])
            self.c_t0s[pid]            .extend(other.c_t0s[pid])
            self.c_edeps[pid]          .extend(other.c_edeps[pid])
            self.c_pos_edeps[pid]      .extend(other.c_pos_edeps[pid])
        return self

    def __add__(self, other):
        new_analysis = TofAnalysis(skip_mangled = self.skip_mangled,
                                   skip_timeout = self.skip_timeout,
                                   beta_analysis= self.beta_analysis)
        new_analysis += self
        new_analysis += other
        return new_analysis

    def fill_histograms(self):
        """
        Fill the histograms with the cached values
        """
        if len(self.c_hit) >= self.event_cache_size: 
            self.nhit_plots['hit'     ].fill(np.array(self.c_hit)) 
            self.nhit_plots['thit'    ].fill(np.array(self.c_thit)) 
            self.nhit_plots['rblink'  ].fill(np.array(self.c_rblink)) 
            self.nhit_plots['miss_hit'].fill(np.array(self.c_miss_hit)) 
            self.nhit_plots['nc_pdls'] .fill(np.array(self.c_nc_pid))
            if self.beta_analysis:
                self.tmg_plots['beta'].fill(np.array(self.c_beta))   
                self.tmg_plots['t_inner'].fill(np.array(self.c_t_inner))
                self.tmg_plots['t_outer'].fill(np.array(self.c_t_outer))
                self.tmg_plots['t_diff'] .fill(np.array(self.c_t_diff))
                self.tmg_plots['ph_delay'].fill(np.array(self.c_ph_delay))
                self.c_beta.clear()
            self.c_t_inner .clear() 
            self.c_t_outer .clear() 
            self.c_t_diff  .clear() 
            self.c_ph_delay.clear() 
            self.c_hit     .clear()
            self.c_thit    .clear()
            self.c_rblink  .clear()
            self.c_miss_hit.clear()
            self.c_nc_pid  .clear()

        for paddle_id in range(1,161):
            if len(self.c_charges[paddle_id]) >= self.hit_cache_size:
                self.paddle_plots[paddle_id]['charge2d' ].fill(np.array(self.c_charges[paddle_id]))
                self.paddle_plots[paddle_id]['amp_a'    ].fill(np.array(self.c_peaks_a[paddle_id]))  
                self.paddle_plots[paddle_id]['amp_b'    ].fill(np.array(self.c_peaks_b[paddle_id]))  
                self.paddle_plots[paddle_id]['time_a'   ].fill(np.array(self.c_times_a[paddle_id]))  
                self.paddle_plots[paddle_id]['time_b'   ].fill(np.array(self.c_times_b[paddle_id]))  
                self.paddle_plots[paddle_id]['bl_a'     ].fill(np.array(self.c_baselines_a[paddle_id]))  
                self.paddle_plots[paddle_id]['bl_b'     ].fill(np.array(self.c_baselines_b[paddle_id]))  
                self.paddle_plots[paddle_id]['bl_a_rms' ].fill(np.array(self.c_baselines_a_rms[paddle_id]))  
                self.paddle_plots[paddle_id]['bl_b_rms' ].fill(np.array(self.c_baselines_b_rms[paddle_id]))  
                self.paddle_plots[paddle_id]['x0'       ].fill(np.array(self.c_positions[paddle_id]))
                self.paddle_plots[paddle_id]['t0'       ].fill(np.array(self.c_t0s[paddle_id]))
                self.paddle_plots[paddle_id]['edep'     ].fill(np.array(self.c_edeps[paddle_id]))
                self.paddle_plots[paddle_id]['pos_edep' ].fill(np.array(self.c_pos_edeps[paddle_id]))
                # clear the caches after filling
                self.c_charges        [paddle_id] .clear()
                self.c_peaks_a        [paddle_id] .clear()
                self.c_peaks_b        [paddle_id] .clear()
                self.c_times_a        [paddle_id] .clear()
                self.c_times_b        [paddle_id] .clear()
                self.c_baselines_a    [paddle_id] .clear()
                self.c_baselines_b    [paddle_id] .clear()
                self.c_baselines_a_rms[paddle_id] .clear()
                self.c_baselines_b_rms[paddle_id] .clear()
                self.c_positions      [paddle_id] .clear()
                self.c_t0s            [paddle_id] .clear()
                self.c_edeps          [paddle_id] .clear()
                self.c_pos_edeps      [paddle_id] .clear()

    def finish(self):
        """
        Ensure the remainder in the caches is histogrammed
        """
        if not self.active:
            return
        event_cache_size      = self.event_cache_size
        hit_cache_size        = self.hit_cache_size
        self.event_cache_size = 1
        self.hit_cache_size   = 1
        self.fill_histograms()
        # reset the cache sizes for the next run
        self.event_cache_size = event_cache_size
        self.hit_cache_size   = hit_cache_size
        self.finished = True

    def add_event(self, ev):
        """
        Fills the associated histograms
        
        # Arguments:
            * ev : Any kind of TofEvent or TofEventSummary
        """
        if not self.active:
            return

        if self.finished:
            print ("WARN: Analysis has been finished already. Not able to add more events.")
            return
        if self.first_ev_time == np.inf:
            self.first_ev_time = ev.timestamp48
        self.last_ev_time = ev.timestamp48
        if ev.status == EventStatus.AnyDataMangling:
            #logger.debug(f'Found mangled event with id {ev.event_id}')
            self.n_mangled += 1
            if self.skip_mangled:
                return
        if ev.status == EventStatus.EventTimeOut:
            #logger.debug(f'Found timed out event with id {ev.event_id}')
            self.n_timed_out += 1
            if self.skip_timeout:
                return
        # FIXME - speed these up
        nhit_ev        = 0
        nhit_t_ev      = 0
        self.n_events += 1
        # at the very first, add the timings if desired
        if self.use_offsets:
            ev.set_timing_offsets(self.offsets)
            #print (ev)
        if not self.cuts.void:
            self.cuts.nevents += 1
            nhits_cbe = ev.nhits_cbe
            nhits_umb = ev.nhits_umb
            nhits_cor = ev.nhits_cor
            
            # Count mising hits before (!) we remove non-causal hits
            # FIXME - this returns bytes and should return ints
            missing        = [int(k) for k in ev.get_missing_paddles_hg(self.hg_mapping)]
            self.c_miss_hit.extend(missing)

            # in case we reject non causal hits, do that here
            hits_rmvd_csl = 0
            hits_rmvd_ls  = 0
            if self.cuts.only_causal_hits or self.cuts.ls_cleaning_t_err != np.inf:
                event_hits            = ev.nhits
                #hits_total += event_hits
            if self.cuts.only_causal_hits:
                rm_pids = ev.remove_non_causal_hits()
                self.c_nc_pid.extend(rm_pids)
                hits_rmvd_csl  = len(rm_pids)
            if self.cuts.ls_cleaning_t_err != np.inf:
                rm_pids = ev.lightspeed_cleaning(self.cuts.ls_cleaning_t_err)
                hits_rmvd_ls   = len(rm_pids)
            cuts_passed = True
            if self.cuts.fh_must_be_umb:
                hits_sorted = sorted(ev.hits, key=lambda x: x.event_t0)
                if hits_sorted:
                    if  hits_sorted[0].paddle_id < 60 or hits_sorted[0].paddle_id > 108:
                        cuts_passed = False
                    else:
                        self.cuts.fh_umb_acc += 1
            if not self.cuts.min_hit_cbe <= nhits_cbe <= self.cuts.max_hit_cbe:
                cuts_passed = False
            else:
                self.cuts.hit_cbe_acc += 1
            if not self.cuts.min_hit_umb <= nhits_umb <= self.cuts.max_hit_umb:
                cuts_passed = False
            else:
                self.cuts.hit_umb_acc += 1
            if not self.cuts.min_hit_cor <= nhits_cor <= self.cuts.max_hit_cor:
                cuts_passed = False
            else:
                self.cuts.hit_cor_acc += 1
            if not cuts_passed:
                return
            if self.cuts.only_causal_hits or self.cuts.ls_cleaning_t_err != np.inf:
                self.cuts.hits_total    = event_hits
                self.cuts.hits_rmvd_ls  = hits_rmvd_ls
                self.cuts.hits_rmvd_csl = hits_rmvd_csl
        else:
            # Still count missing hits even without cutting
            # FIXME - this returns bytes and should return ints
            missing        = [int(k) for k in ev.get_missing_paddles_hg(self.hg_mapping)]
            self.c_miss_hit.extend(missing)

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
            # fill the caches
            self.c_charges[h.paddle_id].append([h.charge_a, h.charge_b])
            self.c_peaks_a[h.paddle_id].append(h.peak_a)
            self.c_peaks_b[h.paddle_id].append(h.peak_b)
            self.c_times_a[h.paddle_id].append(h.time_a)
            self.c_times_b[h.paddle_id].append(h.time_b)
            self.c_baselines_a[h.paddle_id].append(h.baseline_a)
            self.c_baselines_b[h.paddle_id].append(h.baseline_b)
            self.c_baselines_a_rms[h.paddle_id].append(h.baseline_a_rms)
            self.c_baselines_b_rms[h.paddle_id].append(h.baseline_b_rms)
            self.c_positions[h.paddle_id].append(h.pos/h.paddle_len)
            self.c_t0s[h.paddle_id].append(h.event_t0)
            self.c_edeps[h.paddle_id].append(h.edep)
            self.c_pos_edeps[h.paddle_id].append([h.pos/h.paddle_len, h.edep])
        
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
        
        self.c_hit.append(nhit_ev)
        self.c_thit.append(nhit_t_ev)
        self.c_rblink.append(n_rblink_ev)

        if not self.beta_analysis:
            return
        
        outer_h = sorted(outer_h, key=lambda x: x.event_t0)
        inner_h = sorted(inner_h, key=lambda x: x.event_t0)
        if inner_h and outer_h:
            #first_hit = sorted([h for h in ev.hits], key=lambda x: x.phase_delay)
            #last_hit  = first_hit[-1].phase_delay
            #first_hit = first_hit[0].phase_delay
            #print (inner_h, outer_h)
            diff_h  = inner_h[0].event_t0 - outer_h[0].event_t0 
            beta = (distance(inner_h[0],outer_h[0])/1000)/(diff_h*1e-9)/299792458
            if beta < 0:
                beta = -1*beta
            self.c_beta    .append(beta)
            self.c_t_outer .append(outer_h[0].event_t0)
            self.c_t_inner .append(inner_h[0].event_t0)  
            self.c_t_diff  .append(inner_h[0].event_t0 - outer_h[0].event_t0)  
            self.c_ph_delay.append(inner_h[0].phase_delay - outer_h[0].phase_delay)

            #self.tmg_plots['beta'].fill(np.array([beta]))   
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
        
        # fill is the massive bottleneck here, thus let's try to reduce the amount of calls 
        self.fill_histograms()    
        return 

##################################################################

