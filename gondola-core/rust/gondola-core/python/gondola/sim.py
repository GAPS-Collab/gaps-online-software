"""
Classes to interface with custom geant4 simulation
"""

from . import _gondola_core 

McTree                           =  _gondola_core.events.McTree             
McTree.__module__                = __name__
McTree.__name__                  = 'McTree'

