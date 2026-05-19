"""
gondola - GAPS online software core suite for python.
"""

# GAPS online/offline software core suite for python

from . import gondola_core as _gondola_core

import importlib as _importlib
import os as _os 

__version__ = _gondola_core.get_version()
get_version_major = _gondola_core.get_version_major
get_version_major.__module__ = __name__ 

get_version_minor = _gondola_core.get_version_minor
get_version_minor.__module__ = __name__ 

get_version_patch = _gondola_core.get_version_patch
get_version_patch.__module__ = __name__

version_at_least = _gondola_core.version_at_least 
version_at_least.__module__ = __name__

get_version     = _gondola_core.get_version 
get_version.__module__ = __name__ 

# set up the python submodules
# the python wrappers are needed to define 
# __module__ on each rust created class, 
# otherwise the documnetation won't work
from . import algo
from . import events
from . import calibration 
from . import visual
from . import io
from . import tof
from . import db 
from . import reconstruction 
from . import tracker
from . import packets 
from . import monitoring
from . import stats
from . import sim 
from . import run 

__all__ = ['events', 'packets',\
           'io', 'monitoring',\
           'stats', 'algo',\
           'db', 'calibration',\
           'visual','tracker',\
           'reconstruction', 'tof',\
           'sim', 'run']

# clean up the namespace, module still available as hidden through _gondola_core
# this might be depending on the build process if this actually exists
try:
    del gondola_core
except:
    pass # never fail silently, however, since this is an artefact anyway, 
         # it really should not matter

def init_database():
    """
    Returns the path to the included SQLite database file.
    """
    with _importlib.resources.path("gondola", "gaps_flight.db") as db_path:
        _os.environ['GONDOLA_DB_URL'] = str(db_path) 
        return db_path

def init_tracker_cal():
    """
    Returns the path to the data file used for the tracker online (in-flight) 
    calibration as it has been done on the GAPS instrument during flight
    """
    with _importlib.resources.path("gondola", "tracker_cal") as cal_path:
        _os.environ['GONDOLA_TRK_ONLINE_CAL'] = str(cal_path) 
        return cal_path

#----------------------------------
# Initializing 

init_database() 
init_tracker_cal()

print (f'Welcome to gondola v{__version__}, a software suite for the \U0001F388 GAPS experiment! Bulld for \U0001F40D with the power of \U0001F980! \u2728')
print (f' -- The database has been set to GONDOLA_DB_URL {_os.environ["GONDOLA_DB_URL"]}')

