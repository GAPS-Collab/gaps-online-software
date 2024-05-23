import tof_db.models as m


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

def get_paddle(rb_id, rb_ch):
    """
    Get information about a specific paddle end for 
    a RB ID/Channel
    """
    rbs = m.RB.object.filter(rb_id=rb_id)
    return rbs.get_channel(rb_ch)

def get_tof_paddles(panel_id=None):
    """
    Get all TOF paddles
    """
    if panel_id is None:
        paddles = [k for k in m.Paddle.objects.all()]
    else:
        paddles = [k for k in m.Paddle.objects.filter(panel_id=panel_id)]
    return paddles

def get_cube_paddles():
    paddles = [k for k in m.Paddle.objects.filter(panel_id__lt=7)]
    for pid in 57,58,59,60:
        paddle = m.Paddle.objects.filter(paddle_id=pid)
        paddles.append(paddle[0])
    return paddles

def get_umbrella_paddles():
    paddles = [k for k in m.Paddle.objects.filter(panel_id__gt=6).filter(panel_id__lt=14)]
    return paddles

