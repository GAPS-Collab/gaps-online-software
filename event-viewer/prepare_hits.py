"""
Get the hits in a format so the event viewer can understand
THIS IS TEMPORARY CODE AND SHOULD GO AWAY ASAP
"""

import gsequery
import gaps_tof as gt
from bind.merged_event_bindings import merged_event
import tqdm
from collections import defaultdict
from copy import deepcopy as copy

class TrackerHit:
    """
    This class will go away as soon as
    we have the layer information in the hit
    TEMPORARY CODE
    """
    def __init__(self):
        row     = 0
        hit     = 0
        layer   = 0
        channel = 0
        adc     = 0
        asic    = 0

def prepare_hits_helper(dbfile, t0, t1):
    """
    Get hits from the SQLite DB and bring them
    in a form suitable for the event viewer

    Args:
        dbfile:
        t0:
        t1:

    Returns:

    """
    data = gsequery.GSEQuery(path=dbfile)
    #data = gsequery.GSEQuery(path="/srv/gaps/gaps-online-software/data/gsedb.sqlite")
    T0 = 1678501414
    T1 = 1678502662
    events = data.get_rows1("mergedevent", T0, T1)
    print (f"We got {len (events)} events for this run")
    all_rows = []
    evs   = defaultdict(list)
    me = merged_event()
    failed = 0
    for ev in tqdm.tqdm(events):
        del me
        me = merged_event()
        if len(me.tracker_events) != 0:
            raise
        success = me.unpack_str(bytes(ev[10]), 0)
        if success < 0:
            failed += 1
            continue

        for k in me.tracker_events:

            for j in k.hits:
                # print (dir(j))
                h = TrackerHit()
                h.row = j.row
                h.mod = j.module
                h.channel = j.channel
                h.asic = j.asic_event_code
                h.layer = k.layer - 128
                h.adc = j.adc
                # j.layer   = k.layer
                # if h.adc < 10:
                #    continue
                if (h.asic == 3):
                    continue
                # if (h.asic == 0) or (h.asic == 2):
                #    continue
                evs[me.event_id].append(h)
    return evs