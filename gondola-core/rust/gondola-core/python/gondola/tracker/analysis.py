"""
Provides a standard, quick look mostly quality control focused analysis for 
the GAPS tracker system
"""


import numpy as np
import dashi as d

from copy import deepcopy as copy

#------------------------------------------------------------------------------

class TrackerCuts:
    """
    Tracker specific conditions
    """

    def __init__(self):
        self.nevents        : int = 0
        self.min_hits       : int = 0
        self.max_hits       : int = 12960
        self.min_hits_layer     = {k : 0 for k in range(10)}
        self.max_hits_layer = {k : 1296 for k in range(10)}
        self.hits_acc       = 0
        self.hits_layer_acc = {k : 0 for k in range(10)}

    def is_compatible(self, other):
        if self.min_hits != other.min_hits:
            return False 
        if self.max_hits != other.max_hits:
            return False
        for layer in self.min_hits_layer:
            if self.min_hits_layer[layer] != other.min_hits_layer[layer]:
                return False 
            if self.max_hits_layer[layer] != other.max_hits_layer[layer]:
                return False
        return True

    def __iadd__(self, other):
        if not self.is_compatible(other):
            raise ValueError("Cuts are not compatible!")
        self.hits_acc       += other.hits_acc
        for layer in self.hits_layer_acc:
            self.hits_layer_acc[layer] += other.hits_layer_acc[layer] 
        return self

    def __add__(self, other):
        new_cut = TrackerCuts()
        new_cut += self
        new_cut += other
        return new_cut
    
    @property
    def acc_hit(self):
        if self.nevents == 0:
            return 0
        return self.hits_acc/self.nevents

    @property 
    def acc_hit_layer(self):
        if self.nevents == 0:
            return {k: 0 for k in range(10)}
        acc = dict()
        for layer in self.hits_layer_acc:
            acc[layer] = self.hits_layer_acc[layer]/self.nevents
    
    def accept(self, tracker_hits): 
        """

        # Arguments: 
            tracker_hits :  A list of tracker hits (v2)
        """
        nhits = {layer : 0 for layer in range(10)}
        for h in tracker_hits:
            nhits[h.layer] += 1
        nhits_tot = sum(nhits.values())
        if not self.min_hits <= nhits_tot <= self.max_hits:
            return False
        for layer in self.min_hits_layer:
            if not self.min_hits_layer[layer] <= nhits[layer] <= self.max_hits_layer[layer]:
                return False
        return True 

    def __str__(self):
        return self.__repr__()

    def __repr__(self):
        _repr  = '<TrackerCuts:'
        #if self.only_causal_hits:
        #    _repr += f'\n -- removes non-causal hits!'
        #if self.ls_cleaning_t_err != np.inf:
        #    _repr += f'\n -- removes hits which are not correlated with the first hit!'
        #    _repr += f'\n --   assumed timing error {self.ls_cleaning_t_err}'
        #if self.fh_must_be_umb:
        #    _repr += f'\n -- first hit must be on UMB'
        #if self.thru_going:
        #    _repr += f'\n -- require last hit on CBE BOT or COR (thru-going tracks)'
        #if self.fhi_not_bot:
        #    _repr += f'\n -- require that the first hit on the inner TOF is not on CBE BOT'
        _repr += f'\n  {self.min_hits} <= NHIT <= {self.max_hits}' 
        for layer in self.min_hits_layer:
            _repr += f'\n  {self.min_hits_layer[layer]} <= NHIT(LAYER{layer}) <= {self.max_hits_layer[layer]}'
        _repr += '>'
        return _repr
        return _repr
    
    def pretty_print_efficiency(self):
        _repr =  f'-- -- -- -- -- -- -- -- -- -- --'
        _repr +=  f'\n TOTAL EVENTS : {self.nevents}'
        _repr += f'\n  {self.min_hits} <= NHit(TRK) <= {self.max_hits} : {100*self.acc_hit : .2f} %' 
        for layer in self.min_hits_layer:
            _repr += f'\n  {self.min_hits_layer[layer]} <= NHIT(LAYER{layer}) <= {self.max_hits_layer[layer]} : {100*self.acc_hit_layer[layer] : .2f} %'
        #_repr += f'\n  {self.min_hit_cbe} <= NHit(CBE) <= {self.max_hit_cbe} : {100*self.acc_frac_hit_cbe : .2f} %' 
        #_repr += f'\n  {self.min_hit_cor} <= NHit(COR) <= {self.max_hit_cor} : {100*self.acc_frac_hit_cor : .2f} %' 
        #_repr += f'\n  {self.min_hit_all} <= NHit(TOF) <= {self.max_hit_all} : {100*self.acc_frac_hit_all : .2f} %' 
        return _repr

class TrackerAnalysis:
    """
    Yet another container holding plots for tracker specific analysis
    """

    def _is_compatible(self, other):
        return True

    def pretty_print_statistics(self):
        _repr = "\n-- -- -- -- -- -- -- -- -- "
        _repr += '\n TRK analysis statistics'
        _repr += f'\n  -- events                    : {self.n_events}'
        _repr += f'\n  -- nhits                     : {self.n_hits}'
        #_repr += f'\n  -- runtime (s)              : {self.run_time:1f} s'
        #_repr += f'\n  -- rate    (Hz (nocut))     : {self.rate_nocut:.2f} Hz'
        #_repr += f'\n  -- frac. of mangled events  : {100*self.n_mangled_frac : .2f} %'
        #_repr += f'\n  -- frac. of timedout events : {100*self.n_timed_out_frac : .2f} %' 
        return _repr

    def __iadd__(self, other):
        if not self._is_compatible(other):
            return ValueError("Tracker analysis are not compatible!")

        self.n_events += other.n_events
        self.n_hits   += other.n_hits
        for k in self.nhit_plots:
            self.nhit_plots[k] += other.nhit_plots[k]
            self.nhit_cache[k].extend(other.nhit_cache[k])
        for k in self.edep_plots:
            self.edep_plots[k] += other.edep_plots[k]
            self.edep_cache[k].extend(other.edep_cache[k])
        #self.strip_mask = other.strip_mask
        return self

    def __add__(self, other):
        new_ana = TrackerAnalysis(nbins = self.nbins, active = self.active)
        new_ana += self
        new_ana += other
        return new_ana
    
    def _init_edep_plots(self):
        self.define_bins(nbins = self.nbins)
        self.edep_plots   = dict()
        self.edep_cache   = dict()
        # energy bins can either be adc or energy
        e_bins = self.ADC_BINS
        if self.is_calibrated:
            e_bins = self.EDEP_BINS
        self.edep_plots['edep'] = d.histogram.hist1d(e_bins)
        self.edep_cache['edep'] = []
        for layer in range(10):
            self.edep_plots[f'edep_layer{layer}']     = d.histogram.hist1d(e_bins)
            self.edep_cache[f'edep_layer{layer}']     = []

    def _init_nhit_plots(self):
        """
        Plots for nhit distributions, total, different layers, etc.
        """
        self.define_bins(nbins = self.nbins)
        self.nhit_plots   = dict()
        self.nhit_cache   = dict()
        self.nhit_counter = dict()
        self.nhit_plots['nhit'] = d.histogram.hist1d(self.NHIT_BINS)
        self.nhit_cache['nhit'] = []
        for layer in range(10):
            self.nhit_plots[f'nhit_layer{layer}']     = d.histogram.hist1d(self.NHIT_BINS)
            self.nhit_cache[f'nhit_layer{layer}']     = []
            self.nhit_counter[f'nhit_counter{layer}'] = 0

    def fill_histograms(self):
        """
        Transfer cache data into the histograms and delete
        the cache data
        """
        if len(self.nhit_cache['nhit']) <  self.event_cache_size:
            return
        for k in self.nhit_cache:
            self.nhit_plots[k].fill(np.array(self.nhit_cache[k]))
            self.nhit_cache[k].clear()
        for k in self.edep_cache:
            self.edep_plots[k].fill(np.array(self.edep_cache[k]))
            self.edep_cache[k].clear()

    def finish(self):
        """
        Flush the caches to the histograms
        """
        if not self.active:
            return
        event_cache_size = self.event_cache_size
        self.event_cache_size = 1
        self.fill_histograms()
        self.event_cache_size = event_cache_size
        self.finished = True

    def define_bins(self,nbins = 90):
        self.NHIT_BINS = np.arange(-0.5,25.5,1)
        self.ADC_BINS  = np.arange(0,1800, 1)
        # this is energy deposition in MIP (or actually MeV)
        self.EDEP_BINS = np.linspace(0,6,nbins)
        self.event_cache_size = 1000000

    def emit_kwargs(self) -> dict:
        """
        Provide the kwargs which were used to 
        create this instance
        """
        kwargs = dict()
        kwargs['nbins']              = copy(self.nbins)
        kwargs['active']             = copy(self.active)
        kwargs['cuts']               = copy(self.cuts)
        kwargs['is_calibrated']      = copy(self.is_calibrated)
        kwargs['exclude_empty_hits'] = copy(self.exclude_empty_hits)
        return kwargs

    def __init__(self,\
                 nbins              = 90,\
                 active             = False,\
                 cuts               = TrackerCuts(),
                 is_calibrated      = False,
                 exclude_empty_hits = False):

        self.nbins               = nbins
        self.n_events            = 0
        self.n_hits              = 0
        self.cuts                = cuts
        self.exclude_empty_hits  = exclude_empty_hits
        self.subtract_cmnnoise   = False
        self.apply_transfer_fn   = False
        self.subtract_pedestals  = False
        self.is_calibrated       = is_calibrated
        # a switch to indicate if we currently 
        # want to use this or not. 
        # (as for use in gander)
        self.active   = active
        self.define_bins(nbins = nbins)
        # the order is important here, since edep plots 
        # depend on the decision if the transfer functions
        # should be applied
        self._init_nhit_plots()
        self._init_edep_plots()
        # strip mask indicates active strips
        # by default we don't set any
        self.total_masked_strips = 0
        self.n_hits_not_in_mask  = 0
        self.finished = False

    def add_event(self,ev):
        """
        # Arguments:
            * ev : A merged event
        """
        if not self.active:
            return

        self.n_events += 1
        hits           = ev.tracker
        clean_hits     = []
        if self.exclude_empty_hits:
            for h in hits:
                if h.adc > 0:
                    clean_hits.append(h)
        else:
            for h in hits:
                clean_hits.append(h)
        hits = clean_hits
        nhits = len(hits)
        self.n_hits   += nhits
        self.nhit_cache['nhit'].append(nhits)
        # count hits in individual layers    
        for h in hits:
            energy = h.adc 
            if self.is_calibrated:
                energy = h.energy
            self.nhit_counter[f'nhit_counter{h.layer}'] += 1
            self.edep_cache['edep'].append(energy)
            self.edep_cache[f'edep_layer{h.layer}'].append(energy)
        for layer in range(10):
            self.nhit_cache[f'nhit_layer{layer}'].append(self.nhit_counter[f'nhit_counter{layer}'])
            self.nhit_counter[f'nhit_counter{layer}'] = 0
        self.fill_histograms()


