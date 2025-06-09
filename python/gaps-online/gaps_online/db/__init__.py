"""
Higher level functions to access the sqlite database
shipped with this project.
"""

# FIXME - convoluted imports
import tof_db as tdb
import tof_db.models as m

from tof_db.models import *

##############################################

def get_HG_for_LG(dsi, j, ch):
    """
    Get the high gain (HG) connection for a given low gain 
    connection (LG)
    
    LG => LTB
    HG => RB
    For a dsi/j/channel connection as emitted
    by the MTB, get the respective RB/CH
    
    # Arguments:
    
      * dsi - LG DSI connection on the MTB
      *   j - LG DSI/J connection on the MTB
      *  ch - channel on the connected LTB
    
    # Returns:
    
      RB ID, RB channel
    """
    p_ends = m.PaddleEnd.objects.filter(dsi=dsi, ltb_harting_j=j, ltb_ch=ch)
    if len(p_ends) > 1:
        raise ValueError("Ambiguous result for {dsi,j,ch} mapping! More than one paddle end found! {p_ends}. Check the channel mapping!")
    p_end = p_ends[0]
    return p_end.rb_id, p_end.rb_ch

##############################################

def get_tof_paddles(panel_id=None) -> dict:
    """
    Get all TOF paddles
    """
    if panel_id is None:
        paddles = [k for k in m.Paddle.objects.all()]
    else:
        paddles = [k for k in m.Paddle.objects.filter(panel_id=panel_id)]
    pdict = {pdl.paddle_id : pdl for pdl in paddles}
    return pdict

##############################################

def get_cube_paddles():
    paddles = [k for k in m.Paddle.objects.filter(panel_id__lt=7)]
    for pid in 57,58,59,60:
        paddle = m.Paddle.objects.filter(paddle_id=pid)
        paddles.append(paddle[0])
    return paddles

##############################################

def get_umbrella_paddles():
    paddles = [k for k in m.Paddle.objects.filter(panel_id__gt=6).filter(panel_id__lt=14)]
    return paddles

##############################################

def get_tracker_strips() -> list[m.TrackerStrip]:
    """
    Get a list of all tracker strips
    """
    strips = [k for k in m.TrackerStrip.objects.all()]
    return strips

##############################################

def get_tracker_strip_mask(name) -> dict:
    """
    Get a tracker mask from the db and 
    return a dictionary strip_id -> bool
    
    # Arguments:
        * name : Each strip mask has a unique name
                 under which it can be retrieved from 
                 the db
    """
    #strips = get_tracker_strips()
    strip_mask = {k.strip_id : k.active for k in m.TrackerStripMask.objects.filter(mask_name = name)}
    return strip_mask

##############################################

def get_vid_hid_map() -> dict:
    """
    Return a map of volume id to hardware id. This is in case of tof paddles a number 
    from 1-160 and for the tracker it is a number per strip which contains layer, row, 
    module, strip
    """
    pdls   = get_tof_paddles()
    strips = get_tracker_strips()
    vid_hid_map = dict()
    for pdl in pdls.values():
        vid_hid_map[pdl.volume_id] = pdl.paddle_id
    for stp in strips:
        vid_hid_map[stp.volume_id] = stp.get_id()
    return vid_hid_map


