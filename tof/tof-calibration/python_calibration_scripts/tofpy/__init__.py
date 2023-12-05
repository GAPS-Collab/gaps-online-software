# Licensed under a 3-clause BSD style license - see LICENSE.rst
'''tofpy: The GAPS TOF python package

* Code: https://gitlab.com/ucla-gaps-tof/software
* Docs: ...
'''

#from . import parsing  # allows tofpy.parsing.load()
from .parsing import *  # allows tofpy.load()
from .calibration import *
from .plotting import *