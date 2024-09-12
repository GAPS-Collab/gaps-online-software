"""
GAPS online software TOF part

* pybindings for cxx and rust API
* enhanced functionality for plotting
  and analysis
"""


from .mapping import DsiJChRBMap

from . import sensors
try:
    from . import converters
except ImportError as e:
    print(f"HDF converter tools not available! {e}")
