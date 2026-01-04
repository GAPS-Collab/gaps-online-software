
import numpy as np
import dashi as d
import json
d.visual()

from copy import deepcopy as copy

from .. import gondola_core as _gc

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
        self.PADDLE_PEAK_BINS   = np.linspace(0,200,     nbins)
        self.PADDLE_CHARGE_BINS = np.linspace(-2,60 ,     nbins)
        self.PADDLE_TIMING_BINS = np.linspace(0,250,     nbins)
        self.PADDLE_BL_BINS     = np.linspace(-2.5,2.5 ,     nbins)
        self.PADDLE_BLRMS_BINS  = np.linspace(0,2  ,     nbins)
        self.PADDLE_X0_BINS     = np.linspace(-0.1, 1.1, nbins)
        self.PADDLE_T0_BINS     = np.linspace(0,500,     nbins)
        self.PADDLE_EDEP_BINS   = np.linspace(0,30 ,     nbins)
        self.NHIT_BINS          = np.arange(-0.5,25.5,1)   
        self.PID_BINS           = np.arange(0.5,160.5,1)
        self.BETA_BINS          = np.linspace(0,2  ,     nbins)
        self.EDEP_BINS          = np.linspace(0,50,      nbins)
        self.TIMING_BINS        = np.linspace(-100, 300, nbins)
        self.PDELAY_BINS        = np.linspace(-60,60,    nbins)
        self.TDIFF_BINS         = np.linspace(-1, 10, nbins)
        self.DIST_BINS          = np.linspace(0,4, nbins)
        self.COS_T_BINS         = np.linspace(0,1,nbins)
        self.COS_T2_BINS        = np.linspace(0,1,int(nbins/3))
        self.X_BINS_OUTER       = np.linspace(-2000,2000,nbins)
        self.Y_BINS_OUTER       = np.linspace(-2000,2000,nbins)
        self.Z_BINS_OUTER       = np.linspace(-250, 2500,nbins)
        self.X_BINS_INNER       = np.linspace(-1000,1000,nbins)
        self.Y_BINS_INNER       = np.linspace(-1000,1000,nbins)
        self.Z_BINS_INNER       = np.linspace(-250, 1500,nbins)
        self.PID_BINS_INNER     = np.arange(0.5, 60.5, 1)
        self.PID_BINS_OUTER     = np.arange(61.5, 161.5,1)

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
        if self.cuts.void:
            if self.n_events != 0:
                _repr += f'\n  -- events with extra hits   : {100*self.extra_hits/self.n_events:.2f} %'
        else:
            if self.cuts.nevents > 0:
                _repr += f'\n  -- events with extra hits   : {100*self.extra_hits/self.cuts.nevents:.2f} %'
        
        if not self.cuts.void:
            _repr += '\n  -- -- applied cut:'
            _repr += f'\n\t -- -- {self.cuts}'
            _repr += f'\n\t  -- -- nevents (after cut) : {self.n_events}'
            if self.cuts.nevents > 0:
                _repr += f'\n\t  -- -- efficiency          : {100*self.n_events/self.cuts.nevents : .2f} %'
            _repr += f'\n\t  -- -- rate    (Hz)        : {self.rate: .2f} Hz'
        _repr += '\n  -- Event status statistics'
        nevents = self.n_events
        if not self.cuts.void:
            nevents = self.cuts.nevents 
        for k in self.event_stati:
            if nevents != 0:
                _repr += f'\n\t -- -- {k} : {self.event_stati[k]} ({100*self.event_stati[k]/nevents:.2f} (%)'
        
        return _repr

    def _timing_plots(self):
        # first tuple argument is the histogram, second the cache, 
        # since hist1d.fill takes a long time
        tmg_plots = {
          'beta'         : d.histogram.hist1d(self.BETA_BINS),
          't_inner'      : d.histogram.hist1d(self.TIMING_BINS),
          't_outer'      : d.histogram.hist1d(self.TIMING_BINS),
          # timing difference will not be larger than phase delay!
          't_diff'       : d.histogram.hist1d(self.TDIFF_BINS),
          'ph_delay'     : d.histogram.hist1d(self.PDELAY_BINS),
          'dist'         : d.histogram.hist1d(self.DIST_BINS),
          'dist_vs_beta' : d.histogram.hist2d((self.DIST_BINS, self.BETA_BINS)),
          'dist_vs_tdiff': d.histogram.hist2d((self.DIST_BINS, self.TDIFF_BINS)),
          'cos_theta'    : d.histogram.hist1d(self.COS_T_BINS), 
          'cos2_theta'   : d.histogram.hist1d(self.COS_T2_BINS),
          'x_outer'      : d.histogram.hist1d(self.X_BINS_OUTER),
          'y_outer'      : d.histogram.hist1d(self.Y_BINS_OUTER),
          'z_outer'      : d.histogram.hist1d(self.Z_BINS_OUTER),
          'x_inner'      : d.histogram.hist1d(self.X_BINS_INNER),
          'y_inner'      : d.histogram.hist1d(self.Y_BINS_INNER),
          'z_inner'      : d.histogram.hist1d(self.Z_BINS_INNER),
          'pid_inner'    : d.histogram.hist1d(self.PID_BINS_INNER),
          'pid_outer'    : d.histogram.hist1d(self.PID_BINS_OUTER),
          'beta_vs_theta': d.histogram.hist2d((self.BETA_BINS, self.COS_T_BINS)),
        }
        tmg_cache = dict()
        for k in tmg_plots.keys():
            tmg_cache[k] = []
        return tmg_plots, tmg_cache

    def _edep_plots(self):
        plots = {
          # total energy depostion
          'edep'         : d.histogram.hist1d(self.EDEP_BINS)
        }
        for k in range(1,22):
            plots[f'edep_pnl{k}'] = d.histogram.hist1d(self.EDEP_BINS)
        plots['edep_umb'] = d.histogram.hist1d(self.EDEP_BINS)
        plots['edep_cbe'] = d.histogram.hist1d(self.EDEP_BINS)
        plots['edep_cor'] = d.histogram.hist1d(self.EDEP_BINS)
        plots['umb_vs_cor_edep'] = d.histogram.hist2d((self.EDEP_BINS, self.EDEP_BINS))
        plots['cbe_vs_cor_edep'] = d.histogram.hist2d((self.EDEP_BINS, self.EDEP_BINS))
        plots['cbe_vs_umb_edep'] = d.histogram.hist2d((self.EDEP_BINS, self.EDEP_BINS))
        cache = dict()
        for k in plots.keys():
            cache[k] = []
        return plots, cache


    def _nhit_plots(self):
        nhit_plots = {
          'hit'        : d.histogram.hist1d(self.NHIT_BINS),
          'nhit_cbe'   : d.histogram.hist1d(self.NHIT_BINS),
          'nhit_umb'   : d.histogram.hist1d(self.NHIT_BINS),
          'nhit_cor'   : d.histogram.hist1d(self.NHIT_BINS),
          'i_vs_o_nhit': d.histogram.hist2d((self.NHIT_BINS, self.NHIT_BINS)),
          'umb_vs_cor_nhit': d.histogram.hist2d((self.NHIT_BINS, self.NHIT_BINS)),
          'cbe_vs_umb_nhit': d.histogram.hist2d((self.NHIT_BINS, self.NHIT_BINS)),
          'cbe_vs_cor_nhit': d.histogram.hist2d((self.NHIT_BINS, self.NHIT_BINS)),
          'thit'       : d.histogram.hist1d(self.NHIT_BINS),
          'rblink'     : d.histogram.hist1d(self.NHIT_BINS),
          'miss_hit'   : d.histogram.hist1d(self.PID_BINS),
          # these are non causal hits
          'nc_pdls'    : d.histogram.hist1d(self.PID_BINS),
        }
        return nhit_plots

    def _paddle_plots(self):
        """
        Charge and timing plots for each paddle
        """
        paddle_plots = {\
          # use cache, cache, histogram
          # explanation - in general, the cache won't be needed for 2d histograms which are 
          # created from other histograms
          'charge2d'  : d.histogram.hist2d((self.PADDLE_CHARGE_BINS, self.PADDLE_CHARGE_BINS)),
          'amp2d'     : d.histogram.hist2d((self.PADDLE_PEAK_BINS, self.PADDLE_PEAK_BINS)),
          'amp_a'     : d.histogram.hist1d(self.PADDLE_PEAK_BINS),
          'amp_b'     : d.histogram.hist1d(self.PADDLE_PEAK_BINS),
          'charge_a'  : d.histogram.hist1d(self.PADDLE_CHARGE_BINS),
          'charge_b'  : d.histogram.hist1d(self.PADDLE_CHARGE_BINS),
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
        all_paddle_plots = {k : copy(paddle_plots) for k in range(161)}
        paddle_caches = {k : dict() for k in range(161)}
        for pid in range(161):
            for k in paddle_plots.keys():
                paddle_caches[pid][k] = []
        return all_paddle_plots, paddle_caches

    @property
    def occupancy(self):
        return self._analysis.occupancy 

    @property
    def occupancy_t(self):
        return self._analysis.occupancy_t

    @property 
    def n_events(self):
        return self._analysis.n_events

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
        return 1e-5*(self._analysis.last_event_t - self._analysis.first_event_t)

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
                 skip_timeout       = True,\
                 beta_analysis      = True,\
                 nbins              = 90,
                 cuts               = _gc.tof.TofCuts(),
                 use_offsets        = False,
                 pid_inner          = None,
                 pid_outer          = None,
                 active             = False):
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
          pid_outer                : Select a specific paddle instead of the first on the outer TOF 
                                     for the beta/timing analysis
          pid_inner                : Select a specific paddle instead of the first on the inner TOF
                                     for the beta/timing analysis
          active                   : if True, this analysis will actually "do something"
                                     and acquire events
        """
        self._analysis       = _gc.tof.TofAnalysis() 

        # process kwargs
        self.skip_mangled    = skip_mangled
        self.skip_timeout    = skip_timeout
        self.beta_analysis   = beta_analysis
        self.use_offsets     = use_offsets
        self.nbins           = nbins
        self.cuts            = cuts
        self.active          = active 
        self.offsets         = None
        #self.event_stati     = dict() 
        if self.use_offsets:
            self.offsets = dict()
            offsets = json.load(open('offsets.json'))
            for k in offsets:
                k_int = int(k)
                # currently we only have intra-panel calibrations
                if k_int <= 12:
                    self.offsets[k_int] = offsets[k]
        
        self.define_bins(nbins = self.nbins)
        #self.first_ev_time = np.inf
        #self.last_ev_time  = 0
        self.finished      = False
        self.n_mangled     = 0
        self.n_timed_out   = 0
        #self.n_events      = 0
        pp_hist, pp_cache  = self._paddle_plots()
        self.paddle_plots  = pp_hist
        self.paddle_cache  = pp_cache
        self.nhit_plots    = self._nhit_plots()
        tmg_plots, tmg_cache = self._timing_plots()
        self.tmg_plots     = tmg_plots
        self.tmg_cache     = tmg_cache
        edep_plots, edep_cache = self._edep_plots()
        self.edep_plots    = edep_plots
        self.edep_cache    = edep_cache

        # FIXME - technically, this is not thread safe!!
        self.paddles       = _gc.db.TofPaddle.all_as_dict()
        self.hg_mapping    = _gc.db.get_dsi_j_ch_pid_map()
        # hit histogram
        self.nhit          = 0
        #self.no_hitmiss    = 0
        #self.one_hitmiss   = 0
        #self.two_hitmiss   = 0
        #self.extra_hits    = 0
        #self.occupancy     = {k : 0 for k in range(1,161)}
        #self.occupancy_t   = {k : 0 for k in range(1,161)}
        # beta analysis
        # select specific paddles for beta 
        self.pid_inner     = pid_inner
        self.pid_outer     = pid_outer
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
  
    @property
    def no_hitmiss(self):
        return self._analysis.no_hitmiss
    
    @property
    def one_hitmiss(self):
        return self._analysis.one_hitmiss
    
    @property
    def two_hitmiss(self):
        return self._analysis.two_hitmiss
    
    @property
    def extra_hits(self):
        return self._analysis.extra_hits

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

    @property
    def event_stati(self):
        return self._analysis.event_stati


    def _is_compatible(self, other):
        if self._analysis.skip_mangled  != other._analysis.skip_mangled:
            return False 
        if self._analysis.skip_timeout  != other._analysis.skip_timeout:
            return False
        if self._analysis.beta_analysis != other._analysis.beta_analysis:
            return False
        if self._analysis.pid_inner != other._analysis.pid_inner:
            return False
        if self._analysis.pid_outer != other._analysis.pid_outer:
            return False
        if not self._analysis.cuts.is_compatible(other._analysis.cuts):
            return False
        return True

    def __iadd__(self, other):
        if not self._is_compatible(other):
            raise ValueError("Analysis are not compatible! Both must have the same setup!")
        #return other
        if other._analysis.first_event_t < self._analysis.first_event_t:
            self._analysis.first_event_t = other._analysis.first_event_t
        if other._analysis.last_event_t > self._analysis.last_event_t:
            self._analysis.last_event_t = other._analysis.last_event_t
        self._analysis.cuts          += other._analysis.cuts
        #self._analysis.skip_mangled  += other._analysis.skip_mangled
        #self._analysis.skip_timeout  += other._analysis.skip_timeout
        self._analysis.n_mangled     += other._analysis.n_mangled 
        self._analysis.n_timed_out   += other._analysis.n_timed_out 
        self._analysis.n_events      += other._analysis.n_events
        for k in self.nhit_plots:
            self.nhit_plots[k] += other.nhit_plots[k]
        for k in self.tmg_plots:
            self.tmg_plots[k] += other.tmg_plots[k]
        
        #    self.tmg_cache[k].extend(other.tmg_cache[k])
        for k in self.edep_plots:
            self.edep_plots[k] += other.edep_plots[k]
        #    self.edep_cache[k].extend(other.edep_cache[k])

        ## hit histogram
        self._analysis.nhit          += other._analysis.nhit 
        self._analysis.no_hitmiss    += other._analysis.no_hitmiss
        self._analysis.one_hitmiss   += other._analysis.one_hitmiss
        self._analysis.two_hitmiss   += other._analysis.two_hitmiss
        self._analysis.extra_hits    += other._analysis.extra_hits
        self._analysis.add_other_occupancy(other._analysis.occupancy)
        self._analysis.add_other_occupancy_t(other._analysis.occupancy_t)
        self._analysis.add_other_hit_cache(other._analysis)
        self._analysis.add_other_event_stati(other._analysis)
        self._analysis.add_other_cache(other._analysis)
        for pid in range(1,161):
            for k in self.paddle_plots[pid]:
                self.paddle_plots[pid][k] += other.paddle_plots[pid][k]
                #self.paddle_cache[pid][k].extend(other.paddle_cache[pid][k])
        self._analysis.add_other_paddle_cache(other._analysis);
        return self

    def __add__(self, other):
        new_analysis = TofAnalysis(skip_mangled = self.skip_mangled,
                                   skip_timeout = self.skip_timeout,
                                   beta_analysis= self.beta_analysis)
        new_analysis._analysis = self._analysis
        new_analysis += self
        new_analysis += other
        return new_analysis

    def fill_histograms(self):
        """
        Fill the histograms with the cached values
        """
        if self._analysis.hit_cache_len >= self.event_cache_size: 
            # hit statistics
            self.nhit_plots['hit'     ].fill(self._analysis.c_hit) 
            self.nhit_plots['nhit_umb'].fill(self._analysis.c_hit_umb) 
            self.nhit_plots['nhit_cor'].fill(self._analysis.c_hit_cor) 
            self.nhit_plots['nhit_cbe'].fill(self._analysis.c_hit_cbe) 
            outer_n_hit = [self._analysis.c_hit_umb[k] + self._analysis.c_hit_cor[k] for k in range(len(self._analysis.c_hit_cor))]
            self.nhit_plots['i_vs_o_nhit'].fill((self._analysis.c_hit_cbe, outer_n_hit))
            self.nhit_plots['umb_vs_cor_nhit'].fill((self._analysis.c_hit_umb, self._analysis.c_hit_cor))
            self.nhit_plots['cbe_vs_umb_nhit'].fill((self._analysis.c_hit_cbe, self._analysis.c_hit_umb))
            self.nhit_plots['cbe_vs_cor_nhit'].fill((self._analysis.c_hit_cbe, self._analysis.c_hit_cbe))
            self.nhit_plots['thit'    ].fill(self._analysis.c_thit) 
            self.nhit_plots['rblink'  ].fill(self._analysis.c_rblink) 
            self.nhit_plots['miss_hit'].fill(self._analysis.c_miss_hit) 
            self.nhit_plots['nc_pdls'] .fill(self._analysis.c_nc_pid)
            self._analysis.clear_hit_stats()
            print (f'beta analysis {self._analysis.beta_analysis}')
            if self._analysis.beta_analysis:
            #    c_dist_vs_beta  = np.array([ k for k in zip(self.tmg_cache['dist'], self.tmg_cache['beta'])])
            #    c_dist_vs_tdiff = np.array([ k for k in zip(self.tmg_cache['dist'], self.tmg_cache['t_diff'])])
            #    c_beta_vs_theta = np.array([ k for k in zip(self.tmg_cache['beta'], self.tmg_cache['cos_theta'])])
            #    self.tmg_plots['dist_vs_beta'].fill(c_dist_vs_beta)
            #    self.tmg_plots['dist_vs_tdiff'].fill(c_dist_vs_tdiff)
            #    self.tmg_plots['beta_vs_theta'].fill(c_beta_vs_theta)
                for k in self.tmg_plots:
                    #print (k)
                    if k in ['dist_vs_beta', 'dist_vs_tdiff', 'beta_vs_theta']:
                        continue
                    if k in ['pid_inner','pid_outer']:
                        self.tmg_plots[k].fill(self._analysis.cache.get_u8_data(k))
                        print (self._analysis.cache.get_u8_data('pid_inner'))
                    else:
                        self.tmg_plots[k].fill(self._analysis.cache.get_f32_data(k))
                for k in self.edep_plots:
                    if not k in ['umb_vs_cor_edep', 'cbe_vs_cor_edep', 'cbe_vs_umb_edep', 'edep', 'edep_cor', 'edep_cbe', 'edep_cor']:
                        try:
                            pnl = int(k[8:])
                        except Exception as e:
                            print (f'(can not get panel edep for {k}')
                            continue
                        self.edep_plots[k].fill(self._analysis.cache.get_f32_data_panel("edep", pnl))
                        continue
                    if k == 'umb_vs_cor_edep':
                        self.edep_plots[k].fill(\
                                (self._analysis.cache.get_f32_data("edep_umb"),
                                self._analysis.cache.get_f32_data("edep_cor")))
                        continue
                    if k == 'cbe_vs_cor_edep':
                        self.edep_plots[k].fill(\
                                (self._analysis.cache.get_f32_data("edep_cbe"),
                                self._analysis.cache.get_f32_data("edep_cor")))
                        continue
                    if k == 'cbe_vs_umb_edep':
                        self.edep_plots[k].fill(\
                                (self._analysis.cache.get_f32_data("edep_cbe"),
                                self._analysis.cache.get_f32_data("edep_umb")))
                        continue
                    self.edep_plots[k].fill(self._analysis.cache.get_f32_data(k))
                self._analysis.cache.clear()

            for paddle_id in range(1,161):
                for k in self.paddle_plots[paddle_id]:
                    #print (f'getting {paddle_id} {k}')
                    if k == 'charge2d':#,'amp2d','pos_edep']:
                        datax = self._analysis.paddle_cache.get_f32_data('charge_a', paddle_id)
                        datay = self._analysis.paddle_cache.get_f32_data('charge_b', paddle_id)
                        data  = np.array([j for j in zip(datax,datay)])
                        #print (data)
                        self.paddle_plots[paddle_id]['charge2d'].fill((datax,datay))
                        continue
                    if k == 'amp2d':
                        datax = self._analysis.paddle_cache.get_f32_data('amp_a', paddle_id)
                        datay = self._analysis.paddle_cache.get_f32_data('amp_b', paddle_id)
                        #data  = np.array([j for j in zip(datax,datay)])
                        #print (data)
                        self.paddle_plots[paddle_id]['amp2d'].fill((datax,datay))
                        continue
                    if k == 'pos_edep':
                        datax = self._analysis.paddle_cache.get_f32_data('x0', paddle_id)
                        datay = self._analysis.paddle_cache.get_f32_data('edep', paddle_id)
                        #data  = np.array([j for j in zip(datax,datay)])
                        #print (data)
                        self.paddle_plots[paddle_id]['pos_edep'].fill((datax,datay))
                        continue
                    #if self._analysis.paddle_cache.cache_size(paddle_id) >= self.hit_cache_size:
                    #if len(self.paddle_cache[paddle_id][k]) >= self.hit_cache_size:
                    self.paddle_plots[paddle_id][k].fill(self._analysis.paddle_cache.get_f32_data(k, paddle_id))
                    self._analysis.paddle_cache.clear(k, paddle_id)
                    #print (f'filling cache for {paddle_id} {k}')

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
        
        self._analysis.add_event(ev)
        
        #if self.first_ev_time == np.inf:
        #    self.first_ev_time = ev.timestamp48
        #self.last_ev_time = ev.timestamp48
        #try:
        #    self.event_stati[ev.status]
        #    self.event_stati[ev.status] += 1
        #except KeyError:
        #    self.event_stati[ev.event_status] = 1 
        #except Exception as e:
        #    print (e)
        #    raise
        #if ev.status == _gc.events.EventStatus.AnyDataMangling:
        #    #logger.debug(f'Found mangled event with id {ev.event_id}')
        #    self.n_mangled += 1
        #    if self.skip_mangled:
        #        return
        #if ev.status == _gc.events.EventStatus.EventTimeOut:
        #    #logger.debug(f'Found timed out event with id {ev.event_id}')
        #    self.n_timed_out += 1
        #    if self.skip_timeout:
        #        return
        ## FIXME - speed these up
        #nhit_ev        = 0
        #nhit_t_ev      = 0
        #self.n_events += 1
        ## at the very first, add the timings if desired
        #if self.use_offsets:
        #    ev.set_timing_offsets(self.offsets)
        #    #print (ev)
        #ev.normalize_hit_times()

        ## before cutting, calculate missing hits
        ## the problem for removing hits right now
        ## is the fact that if we do a hit cleaning,
        ## it will be only for the HG hits and not 
        ## the LG hits, so if we do a missing hit calculation 
        ## after the hit cleaning, we will artificially 
        ## increase the number of missing hits
        ## FIXME - this is currently a bit inconsistent.
        #missing        = [int(k) for k in ev.get_missing_paddles_hg(self.hg_mapping)]
        #self.c_miss_hit.extend(missing)

        ## since we might do hit cleaning, for now 
        ## let's explicitly copy the event, see also
        ## issue #82
        #if not self.cuts.void:
        #    ev_for_cuts = ev.copy()
        #    if not self.cuts.accept(ev_for_cuts):
        #        return
        ## if desired, apply the cleanings
        #if self.cuts.only_causal_hits:
        #    rm_pids = ev.remove_non_causal_hits()
        #    self.c_nc_pid.extend(rm_pids)
        #    #hits_rmvd_csl  = len(rm_pids)
        #if self.cuts.ls_cleaning_t_err != np.inf:
        #    rm_pids = ev.lightspeed_cleaning(self.cuts.ls_cleaning_t_err)
        #    #hits_rmvd_ls   = len(rm_pids)

        #for h in ev.trigger_hits:
        #    #pid = find_paddle(h, self.paddles.values())
        #    try:
        #        pid = self.hg_mapping[h[0]][h[1]][h[2][0]] 
        #    except KeyError:
        #        pid = self.hg_mapping[h[0]][h[1]][h[2][1]] 

        #    #self.occupancy_t[pid] += 1
        #    nhit_t_ev += 1
        #if self.beta_analysis:
        #    outer_h = []
        #    inner_h = []
        #for h in ev.hits:
        #    # for gondola, the hits should have paddle information 
        #    # already
        #    pdl = self.paddles[h.paddle_id]
        #    #h.set_paddle(10*pdl.length, pdl.cable_len, pdl.coax_cable_time, pdl.harting_cable_time)
        #    if pdl.panel_id < 22:
        #        edep_key = f'edep_pnl{pdl.panel_id}'
        #        self.edep_cache[edep_key].append(h.edep)
        #        self.edep_cache['edep'].append(h.edep)
        #    if h.edep > 0:
        #        self.occupancy[h.paddle_id] += 1
        #    nhit_ev += 1
        #    if self.beta_analysis:
        #        if self.pid_outer is None:
        #            if h.paddle_id > 60:
        #                outer_h.append(h)
        #        else:
        #            if h.paddle_id == self.pid_outer:
        #                outer_h.append(h)
        #        if self.pid_inner is None:
        #            if h.paddle_id < 61:
        #                inner_h.append(h)
        #        else:
        #            if h.paddle_id == self.pid_inner:
        #                inner_h.append(h)
        #    # fill the caches
        #    #if h.charge_a < 0 or h.charge_b < 0:
        #    #    print (h)
        #    #    raise ValueError
        #    self.paddle_cache[h.paddle_id]['charge2d'].append([h.charge_a, h.charge_b])
        #    self.paddle_cache[h.paddle_id]['amp2d']   .append([h.peak_a, h.peak_b])
        #    self.paddle_cache[h.paddle_id]['amp_a']   .append(h.peak_a)
        #    self.paddle_cache[h.paddle_id]['amp_b']   .append(h.peak_b)
        #    self.paddle_cache[h.paddle_id]['time_a']  .append(h.time_a)
        #    self.paddle_cache[h.paddle_id]['time_b']  .append(h.time_b)
        #    self.paddle_cache[h.paddle_id]['charge_a'].append(h.charge_a)
        #    self.paddle_cache[h.paddle_id]['charge_b'].append(h.charge_b)
        #    self.paddle_cache[h.paddle_id]['bl_a']    .append(h.baseline_a)
        #    self.paddle_cache[h.paddle_id]['bl_b']    .append(h.baseline_b)
        #    self.paddle_cache[h.paddle_id]['bl_a_rms'].append(h.baseline_a_rms)
        #    self.paddle_cache[h.paddle_id]['bl_b_rms'].append(h.baseline_b_rms)
        #    self.paddle_cache[h.paddle_id]['x0']      .append(h.pos/h.paddle_len)
        #    self.paddle_cache[h.paddle_id]['t0']      .append(h.event_t0)
        #    self.paddle_cache[h.paddle_id]['edep']    .append(h.edep)
        #    self.paddle_cache[h.paddle_id]['pos_edep'].append([h.pos/h.paddle_len, h.edep])
        #
        ## hit counting 
        #n_rblink_ev    = len(ev.rb_link_ids)
        #self.nhit     += nhit_ev
        #if nhit_t_ev == nhit_ev:
        #    self.no_hitmiss += 1 
        #elif (nhit_t_ev - nhit_ev) == 1:
        #    self.one_hitmiss += 1
        #elif (nhit_t_ev - nhit_ev) > 1:
        #    self.two_hitmiss += 1
        #elif (nhit_ev > nhit_t_ev):
        #    self.extra_hits += 1
        #
        #self.c_hit.append(nhit_ev)
        #self.c_thit.append(nhit_t_ev)
        #self.c_rblink.append(n_rblink_ev)

        #if not self.beta_analysis:
        #    return
        #
        #outer_h = sorted(outer_h, key=lambda x: x.event_t0)
        #inner_h = sorted(inner_h, key=lambda x: x.event_t0)
        #if inner_h and outer_h:
        #    #first_hit = sorted([h for h in ev.hits], key=lambda x: x.phase_delay)
        #    #last_hit  = first_hit[-1].phase_delay
        #    #first_hit = first_hit[0].phase_delay
        #    #print (inner_h, outer_h)
        #    diff_h  = inner_h[0].event_t0 - outer_h[0].event_t0 
        #    dist = inner_h[0].distance(outer_h[0])/1000
        #    cos_theta = abs(outer_h[0].z - inner_h[0].z)/(1000*dist)  
        #    if diff_h == 0:
        #        beta = 0
        #    else:
        #        beta = dist/(diff_h*1e-9)/299792458
        #    self.tmg_cache['dist']   .append(dist)
        #    self.tmg_cache['x_outer'].append(outer_h[0].x)
        #    self.tmg_cache['y_outer'].append(outer_h[0].y)
        #    self.tmg_cache['z_outer'].append(outer_h[0].z)
        #    self.tmg_cache['x_inner'].append(inner_h[0].x)
        #    self.tmg_cache['y_inner'].append(inner_h[0].y)
        #    self.tmg_cache['z_inner'].append(inner_h[0].z)
        #    self.tmg_cache['pid_inner'].append(inner_h[0].paddle_id)
        #    self.tmg_cache['pid_outer'].append(outer_h[0].paddle_id)
        #    self.tmg_cache['cos_theta'].append(cos_theta)
        #    self.tmg_cache['cos2_theta'].append(cos_theta*cos_theta)
        #    if beta < 0:
        #        beta = -1*beta
        #    self.tmg_cache['beta']    .append(beta)
        #    self.tmg_cache['t_outer'] .append(outer_h[0].event_t0)
        #    self.tmg_cache['t_inner'] .append(inner_h[0].event_t0)  
        #    self.tmg_cache['t_diff']  .append(inner_h[0].event_t0 - outer_h[0].event_t0)  
        #    self.tmg_cache['ph_delay'].append(inner_h[0].phase_delay - outer_h[0].phase_delay)
 
        ## fill is the massive bottleneck here, thus let's try to reduce the amount of calls 
        self.fill_histograms()    
        return 

