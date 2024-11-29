# the speed of light in a tof paddle
C_LIGHT_PADDLE = 15.4

try:
    import django
    django.setup()
    from .. import db
    import os
    os.environ['DJANGO_ALLOW_ASYNC_UNSAFE'] = '1'

except Exception as e:
    print(f"Can't load django environment, gaps_db will not be available. {e}")

import numpy as np
import tqdm

def find_paddle(hit, paddles):
    """
    Get a paddle id for a trigger hit
    where the trigger hit is (dsi, j, ch)
    """
    # hit is dsi, ch, channel
    for pdl in paddles:
        if pdl.dsi == hit[0]:
            if pdl.j_ltb == hit[1]:
                if pdl.ltb_chA == hit[2][0]:
                    return pdl.paddle_id
                elif pdl.ltb_chB == hit[2][0]:
                    return pdl.paddle_id
    print (f'No paddle found for {hit}')


def create_occupancy_dict(reader           = None,
                          events           = [],
                          normalize        = True,
                          use_trigger_hits = False,
                          mark_0_as_bad    = False,
                          cbe_side         = True,
                          cor_side         = True):
    """
    Create a dictionary of paddle id vs nhits

    This can either accept a reader or a list of events.
    Use reader when memory is sparse and events when time is
    of the essence
    
    # Arguments:
        * reader           - either TofPacket or TelemetryPacketReader. The reader should be primed in a way
                             that it only spits out MergedEvents, TofEventSummary or TofEvents
    
        * use_trigger_hits - instead of plotting TofHits, just use the triggered hits for the occupancy
        * cbe_side         - add the CBE sides to the occupancy dictionary. It might make sense to exclude 
                             them for normalization reasons
        * cor_side         - add the COR sides to the occupancy dictionary. It might make sense to exclude 
                             them for normalization reasons
    """

    if reader is not None and events:
        raise ValueError("Unable to use both, reader and events!")

    if use_trigger_hits:
        paddles = db.get_tof_paddles()

    if reader is not None:
        for ev in reader:
            ev0 = ev
            break;
    else:
        # events can be TofEventSummary or TofEvent
        ev0 = events[0]

    is_tes = False
    if hasattr(ev0,'trigger_hits'):
        is_tes = True

    is_merged_event = False
    if hasattr(ev0,'tof'):
        is_merged_event = True

    occu_per_paddle = {k : 0 for k in range(1,161)}
    if reader is not None:
        events = reader
    for ev in tqdm.tqdm(events, desc='Getting TOF occupancy data!'):
        if use_trigger_hits:
            if is_tes:
                for h in ev.trigger_hits:
                    pid = find_paddle(h, paddles)
                    occu_per_paddle[pid] += 1
            elif is_merged_event:
                try:
                    ev = ev.tof
                except:
                    continue
                if trigger_hits:
                    for h in ev.trigger_hits:
                        pid = find_paddle(h, paddles)
                        occu_per_paddle[pid] += 1
                else:
                    for h in ev.hits:
                        pid = find_paddle(h, paddles)
                        occu_per_paddle[pid] += 1

            else:
                for h in ev.mastertriggerevent.trigger_hits:
                    pid = find_paddle(h, paddles)
                    occu_per_paddle[pid] += 1

        else:
            for h in ev.hits:
                occu_per_paddle[h.paddle_id] += 1

    if not cbe_side:
        for pid in range(25, 61):
            occu_per_paddle.pop(pid)
    if not cor_side:
        for pid in range(109, 161):
            occu_per_paddle.pop(pid)
    # normalize it
    if normalize:
        max_val = max(occu_per_paddle.values())
        for k in occu_per_paddle.keys():
            occu_per_paddle[k] = occu_per_paddle[k]/max_val
    return occu_per_paddle


def calc_rms(data):
    """ root mean square calculation """
    return np.sqrt((data ** 2).sum() / len(data))


def get_t0(cfd_a, cfd_b, paddle_len):
    """
    Get the particle interaction time for a paddle
    """
    return 0.5 * (cfd_a + cfd_b - (paddle_len / (10.0 * C_LIGHT_PADDLE)))


def get_pos(cfd_a, t0):
    """
    Position along a paddle, measured from the
    A-side
    """
    return (cfd_a - t0) * C_LIGHT_PADDLE * 10.0



