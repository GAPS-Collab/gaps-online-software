"""
Disctribution and calculation helpers 
"""

from . import _gondola_core 

mean      = _gondola_core.stats.mean 
mean.__module__ = __name__

gamma_pdf = _gondola_core.stats.gamma_pdf 
gamma_pdf.__module__ = __name__



