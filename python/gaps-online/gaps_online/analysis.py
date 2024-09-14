# the speed of light in a tof paddle
C_LIGHT_PADDLE = 15.4

import numpy as np

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