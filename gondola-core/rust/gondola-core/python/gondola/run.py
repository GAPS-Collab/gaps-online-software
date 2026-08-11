"""
Deal with data & meta data. Data in the GAPS experiment is 
grouped by "run", where one run is a consecutive time of 
ininterrupted data taking with the same configuration 
"""

from dataclasses import dataclass
from fancy_dataclass import TOMLDataclass

@dataclass 
class RunMeta(TOMLDataclass): 
    """
    Write out some statistics for this run and 
    write it ultimately to a .toml file on disk
    """
    start_gps_time : float = 0 
    stop_gps_time  : float = 0
    start_gcu_time : float = 0
    stop_gcu_time  : float = 0
    run_id         : int   = 0
    start_event_id : int   = 0
    stop_event_id  : int   = 0
    missing_evids  : int   = 0
    runtime_h      : float = 0
    n_events       : int   = 0
    avg_rate       : float = 0


