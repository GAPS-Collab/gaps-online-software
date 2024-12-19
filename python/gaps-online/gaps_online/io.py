"""
Work with telemetry and .tof.gaps files
"""

import numpy as np
import re
import sys

if sys.version_info.minor <= 10:
    from datetime import datetime, timezone
    UTC = timezone.utc
else:
    from datetime import datetime, UTC, timezone
from pathlib import Path

rtd_import_success = False
rt_import_success = False
try:
    import go_pybindings as rt
    TelemetryPacketType   = rt.telemetry.TelemetryPacketType
    TelemetryPacketReader = rt.telemetry.TelemetryPacketReader
    TelemetryPacket       = rt.telemetry.TelemetryPacket
    rt_import_success = True
except ImportError as e:
    print(e)
    print('--> Pybindings for telemetry-dataclasses are missing!')
    print('--> Limited functionality available only!')
    print('--> If you want to mitigate this, check your build, e.g. with ccmake and make sure the respective features are turned ON')

try:
    import go_pybindings as rtd
    TofPacketType   = rtd.io.PacketType
    TofPacket       = rtd.io.TofPacket
    TofPacketReader = rtd.io.TofPacketReader
    try:
        # let's see if we can get the mtb connection map
        import os
        db_path = os.environ["DATABASE_URL"]
        dsi_j_pid_map = rtd.io.create_mtb_connection_to_pid_map(db_path)
    except Exception as e:
        print(f'-> Unable to create dsi_j_pid_map! {e}')
    try:
        rb_pid_map    = rtd.io.create_rb_ch_to_pid_map(db_path)
    except Exception as e:
        print(f'-> Unable to create dsi_j_pid_map! {e}')

    rtd_import_success = True
except ImportError as e:
    print(e)
    print('--> Pybindings for tof-dataclasses are missing!')
    print('--> Limited functionality available only!')
    print('--> If you want to mitigate this, check your build, e.g. with ccmake and make sure the respective features are turned ON')

# check for the caraspace system
try:
    import go_pybindings as rtd
    try:
        CRReader = rtd.caraspace.CRReader
        CRWriter = rtd.caraspace.CRWriter
        CRFrame  = rtd.caraspace.CRFrame
    except Exception as e:
        print(f'-> Pybdingins for the caraspace serializaztion library are missing! {e}')
        print('--> Limited functionality available only!')
        print('--> If you want to mitigate this, check your build, e.g. with ccmake and make sure the respective features are turned ON')

    rtd_import_success = True
except ImportError as e:
    print(e)
    print('--> Pybindings are missing!')
    print('--> Limited functionality available only!')
    print('--> If you want to mitigate this, check your build, e.g. with ccmake and make sure the respective features (BUILD_PYBIDINGIS) are turned ON')


##################################################################


def get_ts_from_toffile(fname):
    """
    Get the timestamp from a .tof.gaps file
    
    # Arguments:
        fname : Filename of .tof.gaps file
    """
    pattern = re.compile('Run[0-9]*_[0-9]*.(?P<tdate>[0-9_]*)')
    ts = pattern.search(str(fname)).groupdict()['tdate']
    #print (ts)
    ts = datetime.strptime(ts, '%y%m%d_%H%M%S')
    ts = ts.replace(tzinfo=timezone.utc)
    return ts

def get_ts_from_binfile(fname):
    """
    Get the timestamp from a .gaps.tof file
    """
    pattern = re.compile('RAW(?P<tdate>[0-9_]*).bin')
    ts = pattern.search(str(fname)).groupdict()['tdate']
    ts = datetime.strptime(ts, '%y%m%d_%H%M%S')
    ts = ts.replace(tzinfo=timezone.utc)
    return ts

def get_tof_binaries(run_id : int, data_dir='', ending='*.gaps') -> list[Path]:
    """
    This allows to get binary data written by either liftof
    ('.tof.gaps') files or through the caraspace library
    ('.gaps'). 
    In case of data written by liftof, this is the data written 
    in flight on the TOF CPU disks.
    For caraspace data, this is typically data merged after the 
    fact.

    # Arguments:
        * run_id    : TOF run id, as e.g. stated in the e-log

    # Keyword Arguments
        * data_dir  : directory with a directory for the specific
                      run in it
    """
    datapath = Path(data_dir) / f'{run_id}'
    files    = [f for f in datapath.glob(ending)]
    files    = sorted(files, key=get_ts_from_toffile)
    t_start  = get_ts_from_toffile(files[0])
    t_stop   = get_ts_from_toffile(files[-1])
    if files:
        print(f'-> Found {len(files)} files for run {run_id}!')
        print(f'-> Found {len(files)} files within range of {t_start} - {t_stop}')
        print(f'--> Earliest file {files[0]}')
        print(f'--> Latest file {files[-1]}')
    else:
        print(f'! No files have been found within {t_start} and {t_stop}!')
    #pattern = re.compile('Run(?P<runid>[0-9]*)_(?P<subrunid>[0-9]*).(?P<timestamp>[0-9_])UTC.tof.gaps')
    return files

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
    print(f'-> Found {len(all_files)} files in {data_dir}')
    ts = [get_ts_from_binfile(f) for f in all_files]
    # FiXME - this might throw away 1 file
    files = [f for f, ts in zip(all_files, ts) if t_start <= ts <= t_stop]
    ts = [get_ts_from_binfile(f) for f in files]
    print(f'-> Run duration {ts[-1] - ts[0]}')
    if files:
        print(f'-> Found {len(files)} files within range of {t_start} - {t_stop}')
        print(f'--> Earliest file {files[0]}')
        print(f'--> Latest file {files[-1]}')
    else:
        print(f'! No files have been found within {t_start} and {t_stop}!')
    return files


def grace_get_telemetry_binaries(unix_time_start, unix_time_stop,\
                           data_dir='/gaps_binaries/live/raw/ethernet'):

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

# if telemetry is available, we add additional
# functionality
if rt_import_success:
    def safe_unpack_merged_event(pack : TelemetryPacket):
        """
        Error checked unpack function for MergedEvents from telemetry packets

        # Arguments:

            pack : TelemetryPacket, as for example from a telemetry binary file as
                   read by TelemetryPacketReader

        # Returns:
            MergedEvent if successful
            False       for unpacking errors or if the packet does not
                        have packet_type == TelemetryPacketType.MergedEvent

        """
        if pack.header.packet_type != TelemetryPacketType.MergedEvent:
            return None
        ev = rt.telemetry.MergedEvent()
        try:
            ev.from_telemetrypacket(pack)
        except Exception:
            return None
        try:
            # this trigger the unpack check of
            # the tof payload, which is still
            # packed before this call.
            ev.tof
        except Exception:
            return None
        return ev
