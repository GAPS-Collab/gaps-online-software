"""
GAPS online software TOF part

* pybindings for cxx and rust API
* enhanced functionality for plotting
  and analysis
"""


from .events import RBEvent
from .calibrations import RBCalibration
from .mapping import DsiJChRBMap
from gaps_tof import TofPacket,\
                     get_tofpackets

from . import sensors
try:
    from . import converters
except ImportError as e:
    print(f"HDF converter tools not available! {e}")
