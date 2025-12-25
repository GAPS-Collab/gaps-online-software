"""
Gondola I/O system. Read files and connect to sockets to 
obtain various flavors of data used trhoughout GAPS.
"""

from .. import _gondola_core  as _gc 

from . import streamers

import sys
if sys.version_info.minor <= 10:
    from datetime import datetime, timezone
    UTC = timezone.utc
else:
    from datetime import datetime, UTC, timezone

from pathlib import Path
import numpy as np

# shortcut for import
get_all_telemetry_event_names = _gc.io.get_all_telemetry_event_names
#read_example                  = _gc.io.read_example
get_runfilename               = _gc.io.get_runfilename 
get_califilename              = _gc.io.get_califilename
CRFrameObject                 = _gc.io.CRFrameObject
CRFrameObject.__module__      = __name__ 
CRFrameObject.__name__        = 'CRFrameObject'
DataSourceKind                = _gc.io.DataSourceKind 
CRReader                      = _gc.io.CRReader
CRReader.__module__           = __name__ 
CRReader.__name__             = 'CRReader' 


CRWriter                      = _gc.io.CRWriter
CRWriter.__module__           = __name__ 
CRWriter.__name__             = 'CRWriter' 
CRFrame                       = _gc.io.CRFrame
CRFrame.__module__            = __name__ 
CRFrame.__name__              = 'CRFrame' 
TofPacketReader               = _gc.io.TofPacketReader
TofPacketReader.__module__    = __name__ 
TofPacketReader.__name__      = 'TofPacketReader'
TofPacketWriter               = _gc.io.TofPacketWriter
TofPacketWriter.__module__    = __name__ 
TofPacketWriter.__name__      = 'TofPacketWriter'

TelemetryPacketReader         = _gc.io.TelemetryPacketReader
TelemetryPacketReader.__module__ = __name__ 
TelemetryPacketReader.__name__  = 'TelemetryPacketReader' 

list_path_contents_sorted     = _gc.io.list_path_contents_sorted
get_utc_now                   = _gc.io.get_utc_timestamp 
get_utc_date                  = _gc.io.get_utc_date
get_datetime                  = _gc.io.get_datetime
get_rundata_from_file         = _gc.io.get_rundata_from_file
get_unix_timestamp            = _gc.io.get_unix_timestamp 
apply_diff_to_file            = _gc.io.apply_diff_to_file 
compress_toml                 = _gc.io.compress_toml 
decompress_toml               = _gc.io.decompress_toml
create_compressed_diff        = _gc.io.create_compressed_diff 


def get_ts_from_binfile(fname):
    """ 
    Get a timestamp from a '.bin' file as written 
    by telemetry

    # Arguments:
        * fname  : name of the file 
    
    # Returns:
        datetime.datetime (UTC) 
    """
    ts = str(fname)[-17:-4]
    ts = get_unix_timestamp(ts)
    return datetime.fromtimestamp(ts, UTC)

#---------------------------------------------------

def get_telemetry_binaries(unix_time_start, unix_time_stop,\
                           data_dir='/gaps_binaries/live/raw/ethernet'):
    """
    Get the relevant telemetry data files for time period from a directory

    # Arguments
        * unix_time_start : seconds since epoch for run start
        * unix_time_end   : seconds since epoch for run end

    # Keyword Arguments
        * data_dir        : folder with telemetry binaries ('.bin')
    """
    # file format is something like RAW240712_094325.bin
    t_start = datetime.fromtimestamp(unix_time_start, UTC)
    t_stop  = datetime.fromtimestamp(unix_time_stop, UTC)
    all_files = sorted([k for k in Path(f'{data_dir}').glob('*.bin')])
    #print(f'-> Found {len(all_files)} files in {data_dir}')
    # heres some new logic yikes
    ts = np.array([get_ts_from_binfile(f) for f in all_files])
    all_files = np.array(all_files)[np.argsort(ts)]
    ts = np.sort(ts)
    i = 0
    while(ts[i]<t_start):
        i+=1
    j=i
    while(ts[j]<t_stop):
        j+=1
    i-=1
    j+=1
    # FiXME - this might throw away 1 file
    #files = [f for f, ts in zip(all_files, ts) if t_start <= ts <= t_stop]
    files = all_files[i:j]
    ts = [get_ts_from_binfile(f) for f in files]
    print(f'-> Run duration {ts[-1] - ts[0]}')
    if len(files)>0:
        print(f'-> Found {len(files)} files within range of {t_start} - {t_stop}')
        print(f'--> Earliest file {files[0]}')
        print(f'--> Latest file {files[-1]}')
    else:
        print(f'! No files have been found within {t_start} and {t_stop}!')
    return files

def grace_get_telemetry_binaries(unix_time_start, unix_time_stop,\
                                 data_dir='/gaps_binaries/live/raw/ethernet'):
    """
    Get the relevant telemetry data files for time period from a directory. 
    This function is preferred over get_telemetry_binaries, since it is more 
    precise in constricting the time range

    # Arguments
        * unix_time_start : seconds since epoch for run start
        * unix_time_end   : seconds since epoch for run end

    # Keyword Arguments
        * data_dir        : folder with telemetry binaries ('.bin')
    """

    # file format is something like RAW240712_094325.bin
    t_start = datetime.fromtimestamp(unix_time_start, UTC)
    t_stop  = datetime.fromtimestamp(unix_time_stop, UTC)
    all_files = sorted([k for k in Path(f'{data_dir}').glob('*.bin')])
    print(f'-> Found {len(all_files)} files in {data_dir}')
    #ts = [get_ts_from_binfile(f) for f in all_files]
    # FiXME - this might throw away 1 file
    #files = [f for f, ts in zip(all_files, ts) if t_start <= ts <= t_stop]
    #ts = [get_ts_from_binfile(f) for f in files]
    time_stamps = [get_ts_from_binfile(f) for f in all_files]
    # I think there is probably a more elegant way to do this with numpy sorting
    # but dont know how it handles datetime object, ona  side note
    # why tf are we using datetime objects instead of like floats...
    sorted_inds = np.argsort(time_stamps)
    sorted_files = np.array(all_files)[sorted_inds]
    sorted_times = np.array(time_stamps)[sorted_inds]
    i = np.argmin(np.abs(t_start-sorted_times))-1
    j = np.argmin(np.abs(t_stop-sorted_times))+1
    if i < 1:
        i = 1
    if j >= len(sorted_files):
        j = len(sorted_files)-1
    files = sorted_files[i-1:j+1]
    print(f'-> Run duration {sorted_times[j] - sorted_times[i-1]}')
    if len(files) > 0:
        print(f'-> Found {len(files)} files within range of {t_start} - {t_stop}')
        print(f'--> Earliest file {files[0]}')
        print(f'--> Latest file {files[-1]}')
    else:
        print(f'! No files have been found within {t_start} and {t_stop}!')
    return files


