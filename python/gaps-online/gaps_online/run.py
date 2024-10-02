"""
Higher level functionality to work with entire runs
"""

from .io import TelemetryPacketReader,\
                TelemetryPacketType,\
                TofPacket,\
                TofPacketType,\
                get_telemetry_binaries,\
                safe_unpack_merged_event

from .tof.monitoring import MtbMoniData,\
                            RBMoniData

from .db import ReadoutBoard
from .events import MergedEvent

import tqdm

def load_run_from_telemetry(start, end, data_dir='/data0/gaps/csbf/csbf-data/binaries/ethernet') -> dict:
    """
    Load data from telemetry binaries within start/end range

    # Arguments:
        * start    : start time of the run in UNIX time since epoch
        * end      : end time of the run in UNIX time since epoch
    
    # Keyword Arguments:
        * data_dir : directory with .bin files ("Berkeley binaries")
    """
    files    = get_telemetry_binaries(start, end, data_dir=data_dir)
    nevents  = 0
    nmerged  = 0
    npackets = 0
    packets  = []

    mtbmoni = []
    rbmoni  = []

    for f in tqdm.tqdm(files, desc='Reading files..'):
        treader = TelemetryPacketReader(str(f))
        for pack in treader:
            npackets += 1
            if pack.header.packet_type == 90:
                nmerged += 1
                packets.append(pack)
            if pack.header.packet_type == 92: # AnyTofHK
                tp = TofPacket()
                tp.from_bytestream(pack.payload, 0)
                if tp.packet_type == TofPacketType.MonitorMtb:
                    moni = MtbMoniData()
                    moni.from_tofpacket(tp)
                    mtbmoni.append((pack.header.gcutime,moni))
                if tp.packet_type == TofPacketType.RBMoniData:
                    moni = RBMoniData()
                    moni.from_tofpacket(tp)
                    rbmoni.append((pack.header.gcutime, moni))

    events = [safe_unpack_merged_event(k) for k in tqdm.tqdm(packets, desc='Unpacking merged events')]
    errors = [k for k in tqdm.tqdm(events, desc='Error checking..') if k is None]
    errors = len(errors)
    events = [k for k in tqdm.tqdm(events, desc='Filtering....') if k is not None]

    # event ids
    evids   = [ev.tof.event_id for ev in events if not ev.broken]
    evids   = sorted(evids)
    missing = len([k for k in range(evids[0], evids[-1])]) - len(evids)

    print (f'--> Found {npackets} packets in the binary files')
    print (f'--> During unpacking, we encountered {errors} de-serialization errors!')
    print (f'--> We got {len(events)} merged events in this run')
    print (f'--> Compared to a consecutive rising event id, we lost {100*missing/(len(evids) + missing):4.2f}% events!')

    result = {'files'   : files,\
              'errors'  : errors,\
              'events'  : events,\
              'mtbmoni' : mtbmoni,\
              'rbmoni'  : rbmoni}
    return result

#############################################################

def get_file_tof_status(fname, check_rbs=False) -> dict:
    """
    Get the number of active TOF paddles and rbs for
    each file

    # Arguements:
        fname      : file name of telemetry file (RAW*.bin)
    # Keyword Arguments:
        check_rbs : If yes, translate the active paddles in
                    Readoutborad IDs and add them to the
                    returned dictionary
    """
    firsttime = None
    reader = TelemetryPacketReader(str(fname), filter=TelemetryPacketType.MergedEvent)
    all_paddles = []
    active_rbs  = []
    elapsed     = 0
    nerrors = 0
    if check_rbs:
        all_rbs = [k for k in ReadoutBoard.objects.all()]
    has_packet = False
    for pack in reader:
        has_packet = True
        if firsttime is None:
            firsttime = pack.header.gcutime
        ev = MergedEvent()
        try:
            ev.from_telemetrypacket(pack)
            for h in ev.tof.hits:
                all_paddles.append(h.paddle_id)
        except Exception as e:
            nerrors += 1
    if has_packet:
        lasttime = pack.header.gcutime
        elapsed  = lasttime - firsttime
        all_paddles = set(all_paddles)

    if check_rbs:
        for pdl in all_paddles:
            for rb in all_rbs:
                if pdl in rb.pids:
                    active_rbs.append(rb.rb_id)
    active_rbs = list(set(active_rbs))
    result = { 'npaddles' : len(all_paddles),\
               'nerrors'  : nerrors,\
               'elapsed'  : elapsed,\
               'all_paddles' : all_paddles,\
               'active_rbs'  : active_rbs}
    return result

#############################################################

def get_run_tof_status(files, check_rbs=True) -> dict:
    """
    Get the number of active TOF paddles and rbs for
    a whole run as represented by the list of telemetry
    files

    # Arguements:
        fname      : file name of telemetry file (RAW*.bin)
    # Keyword Arguments:
        check_rbs : If yes, translate the active paddles in
                    Readoutborad IDs and add them to the
                    returned dictionary
    """
    npaddles = []
    nrbs = []
    elapsed = 0
    all_paddles = []
    all_rbs = []
    for f in tqdm.tqdm(files, desc="Getting TOF status from files..."):
        pdls = get_file_tof_status(str(f), check_rbs=check_rbs)
        npaddles.append((elapsed, pdls['npaddles']))
        nrbs.append((elapsed, len(pdls['active_rbs'])))
        elapsed += pdls['elapsed']
        all_paddles.extend(pdls['all_paddles'])
        all_poaddles = list(set(all_paddles))  # kick out duplicates
        all_rbs.extend(pdls['active_rbs'])
        all_rbs = list(set(all_rbs))

    all_paddle = s = set(all_paddles)
    all_rbs = set(all_rbs)

    missing_pdls = [k for k in range(1, 161) if not k in all_paddles]
    missing_rbs = [k for k in range(1, 50) if not k in all_rbs]
    non_existing_rbs = [10, 12, 37, 38, 43, 45, 47, 48, 49]
    missing_rbs = [k for k in missing_rbs if not k in non_existing_rbs]
    print(f'-> For this run, we have {len(missing_pdls)} missing paddles!')
    print(f'-> For this run, we are missing the following RBs {missing_rbs}!')
    result = {'missing_paddels': missing_pdls, \
              'npaddles': npaddles, \
              'nrbs': nrbs, \
              'missing_rbs': missing_rbs, \
              'active_pdls': all_paddles, \
              'active_rbs': all_rbs}
    return result

#############################################################

def check_missing_events(evids : list) -> None:
    """
    Compare the list of event ids to a list with
    consecutive rising event ids for the same range

    # Arguments:
        evids  : A list of event ids
    """
    all_evids = range(evids[0], evids[-1])
    print (f'-> We found {len(evids)} event ids when expecting {len(all_evids)}')
    print (f'-> We are missing {100*(len(all_evids) - len(evids))/len(all_evids):.3f}%')
