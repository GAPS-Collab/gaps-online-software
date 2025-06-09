"""
Provides a standard, quick look mostly quality control focused analysis for 
the GAPS tracker system
"""

import numpy as np
import dashi as d

class TrackerCuts:
    """
    Tracker specific conditions
    """

    def __init__(self):
        self.min_nhits      = 0
        self.max_hits       = np.inf
        self.min_hits_layer = {k : 0 for k in range(10)}
        self.max_hits_layer = {k : np.inf for k in range(10)}



    def __iadd__(self, other):
        return self

    def __add__(self, other):
        new_cut = TrackerCuts()
        new_cut += self
        new_cut += other
        return new_cut

    def __str__(self):
        return self.__repr__()

    def __repr__(self):
        _repr = ''
        return _repr
    
    def pretty_print_efficiency(self):
        _repr = ''
        return _repr

class TrackerAnalysis:
    """
    Yet another container holding plots for tracker specific analysis
    """

    def _is_compatible(self, other):
        if self.strip_mask != other.strip_mask:
            return False
        return True

    def pretty_print_statistics(self):
        _repr = "\n-- -- -- -- -- -- -- -- -- "
        _repr += '\n TRK analysis statistics'
        _repr += f'\n  -- events                    : {self.n_events}'
        _repr += f'\n  -- nhits                     : {self.n_hits}'
        if self.strip_mask:
            _repr += '\n -- --  a strip mask has been applied!'

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
            self.nhit_cache[k] += other.nhit_cache[k]
        #self.strip_mask = other.strip_mask
        return self

    def __add__(self, other):
        new_ana = TrackerAnalysis(nbins = self.nbins, active = self.active)
        new_ana += self
        new_ana += other
        return new_ana

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

    def finish(self):
        if not self.active:
            return
        event_cache_size = self.event_cache_size
        self.event_cache_size = 1
        self.fill_histograms()
        self.event_cache_size = event_cache_size
        self.finished = True

    def _energy_dep_plots():
        pass

    def define_bins(self,nbins = 90):
        self.NHIT_BINS = np.arange(-0.5,25.5,1)
        self.event_cache_size = 1000000

    def __init__(self, nbins = 90, active = False, strip_mask = dict()):
        self.nbins    = nbins
        self.n_events = 0
        self.n_hits   = 0
        # a switch to indicate if we currently 
        # want to use this or not. 
        # (as for use in gander)
        self.active   = active
        self.define_bins(nbins = nbins)
        self._init_nhit_plots()
        # strip mask indicates active strips
        # by default we don't set any
        self.strip_mask = strip_mask
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
        hits           = ev.tracker_v2
        if self.strip_mask:
            masked_hits = []
            for h in hits:
                try:
                    if self.strip_mask[h.stripid]:
                        masked_hits.append(h)
                except KeyError:
                    masked_hits.append(h)
                    self.n_hits_not_in_mask += 1
            #masked_hits = [h for h in hits if self.strip_mask[h.strip_id]]
        else:
            masked_hits = hits
        self.total_masked_strips += len(hits) - len(masked_hits)
        hits = masked_hits
        nhits = len(hits)
        self.n_hits   += nhits
        self.nhit_cache['nhit'].append(nhits)
        # count hits in individual layers    
        for h in ev.tracker_v2:
            self.nhit_counter[f'nhit_counter{h.layer}'] += 1
        for layer in range(10):
            self.nhit_cache[f'nhit_layer{layer}'].append(self.nhit_counter[f'nhit_counter{layer}'])
            self.nhit_counter[f'nhit_counter{layer}'] = 0
        self.fill_histograms()


