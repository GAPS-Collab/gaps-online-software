"""
gondola - GAPS online software core suite for python.
"""

# GAPS online/offline software core suite for python

from . import gondola_core as _gondola_core

import importlib as _importlib
import os as _os 

__version__ = _gondola_core.get_version()

packets    = _gondola_core.packets 
#io         = _gondola_core.io 
monitoring = _gondola_core.monitoring 
stats      = _gondola_core.stats 
algo       = _gondola_core.algo 
#db         = _gondola_core.db 
#tracker    = _gondola_core.tracker 

# set up the python submodules
from . import events
from . import calibration 
from . import visual
from . import io
from . import tof
from . import db 
from . import reconstruction 
from . import tracker

__all__ = ['events', 'packets', 'io', 'monitoring', 'stats', 'algo', 'db',
           'calibration', 'visual']

# clean up the namespace, module still available as hidden through _gondola_core
del gondola_core

def init_database():
    """
    Returns the path to the included SQLite database file.
    """
    with _importlib.resources.path("gondola", "gaps_flight.db") as db_path:
        _os.environ['GONDOLA_DB_URL'] = str(db_path) 
        return db_path

#----------------------------------
# Initializing 

init_database() 

print (f'Welcome to gondola v{__version__}, a software suite for the \U0001F388 GAPS experiment! Bulld for \U0001F40D with the power of \U0001F980! \u2728')
print (f' -- The database has been set to GONDOLA_DB_URL {_os.environ["GONDOLA_DB_URL"]}')

