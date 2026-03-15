"""
Dedicated module for variaous algorithms 
"""

from . import _gondola_core 

get_max_value_idx = _gondola_core.algo.get_max_value_idx 
get_max_value_idx.__module__ = __name__ 

interpolate_time  = _gondola_core.algo.interpolate_time 
interpolate_time.__module__ = __name__ 

fit_sine_simple   = _gondola_core.algo.fit_sine_simple 
fit_sine_simple.__module__  = __name__
