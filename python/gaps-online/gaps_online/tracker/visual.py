####

"""
Plot occupancy, spectra, projections etc.
"""

import numpy as np

SILI_RADIUS = 5 # mm, with guardring and all

##################################################

def strip_lines():
    """
    Assuming there ia a single SiLi-waver at 0,0, return positions and 
    line lengths to indicate the grooves which separate the strips with 
    lines. These can then be used by either hlines or vlines depending 
    on the orientation of the wafer
    """
    sw  =  [16.34907456/2,  10.32502464/2,\
             9.23299776/2,   8.84362752/2,\
            -8.84362752/2,  -9.23299776/2,\
           -10.32502464/2, -16.34907456/2]
    sw  = 0.2*np.array(sw)
    l_pos = [0]*7
    l_pos[0] = sw[1] + sw[2] + sw[3]
    l_pos[1] = sw[2] + sw[3]
    l_pos[2] = sw[3]
    l_pos[3] = 0
    l_pos[4] = -1*l_pos[2]
    l_pos[5] = -1*l_pos[1]
    l_pos[6] = -1*l_pos[0]
    radii = [SILI_RADIUS*0.8,\
             SILI_RADIUS*0.95, SILI_RADIUS,\
             SILI_RADIUS, SILI_RADIUS,\
             SILI_RADIUS*0.95,\
             SILI_RADIUS*0.8]
    return zip(radii, l_pos)

##################################################

def plot_strip_lines(ax,det,layer : int, color='k'):
    """
    Use lines to indicate detector strips. Automatically apply 
    correct orientation for even/odd layers.

    # Arguments:
        ax    : axis instance to plot on 
        det   : iterable providing x,y coordinates
        layer : tracker layer (0-9)
    """
    strip_widths = strip_lines()
    if layer % 2 == 0: # even layer
        for r,sw in strip_widths:
            ax.plot([det[0] + sw, det[0] + sw], [det[1] - r, det[1] + r], color=color, alpha=0.4, lw=0.5)
    else:
        for r,sw in strip_widths:
            ax.plot([det[0] - r, det[0]+ r], [det[1] + sw, det[1] + sw], color=color, alpha=0.4, lw=0.5)

