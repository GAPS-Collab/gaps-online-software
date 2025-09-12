"""
Gondola I/O system. Read files and connect to sockets to 
obtain various flavors of data used trhoughout GAPS.
"""

from .. import _gondola_core  as _gc 

# shortcut for import
get_all_telemetry_event_names = _gc.io.get_all_telemetry_event_names
read_example                  = _gc.io.read_example
get_runfilename               = _gc.io.get_runfilename 
get_califilename              = _gc.io.get_califilename
CRFrameObject                 = _gc.io.CRFrameObject 
DataSourceKind                = _gc.io.DataSourceKind 
CRReader                      = _gc.io.CRReader
list_path_contents_sorted     = _gc.io.list_path_contents_sorted
get_utc_now                   = _gc.io.get_utc_timestamp 
get_utc_date                  = _gc.io.get_utc_date
get_datetime                  = _gc.io.get_datetime
get_rundata_from_file         = _gc.io.get_rundata_from_file

get_unix_timestamp            = _gc.io.get_unix_timestamp 
