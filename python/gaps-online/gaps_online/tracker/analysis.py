"""
Provides a standard, quick look mostly quality control focused analysis for 
the GAPS tracker system
"""

import numpy as np

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

    def _energy_dep_plots():
        pass

    def define_bins(nbins):
        pass

    def __init__(self):
        self.n_events = 0



