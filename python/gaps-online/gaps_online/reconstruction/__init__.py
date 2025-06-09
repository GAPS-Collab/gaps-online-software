"""
GAPS first guess & prototype event reconstructions

"""

from enum import Enum
import numpy as np

import iminuit
from iminuit.cost import LeastSquares as ls

#######################
#
## Errors from spatial dimensions of paddles/strips
## FIXME 
#ERR_X_TOF = 1
#ERR_Y_TOF = 1
#ERR_Z_TOF = 1
#
class FitStatus(Enum):
    Unknown        = 0
    DidNotConverge = 10
    Success        = 42

##############################################

def line3d(z, x_a, y_a, z_a, dx, dy, dz):
    """
    describe line depending on z since that is our
    best constrained value

    This model has 6 free parameters, 3 for 
    the anchor point and 3 for the direction
    """
    x = x_a + ((dx/dz) * (z - z_a))
    y = y_a + ((dy/dz) * (z - z_a))
    return x,y,z

####################################################

def line3d_2pts(z, tu_x, tu_y, tu_z, tl_x, tl_y, tl_z):
    """
    describe line depending on z since that is our
    best constrained value. Version with 2 points 
    instead of direction vector, so we can constrain
    the point on the tof better
    """

    dx = tu_x - tl_x
    dy = tu_y - tl_y
    dz = tu_z - tl_z
    # a little cheating here
    if dz == 0: dz = 1e-3
    if dy == 0: dy = 1e-3
    if dx == 0: dx = 1e-3
    x  = tu_x + ((dx/dz) * (z - tu_z))
    y  = tu_y + ((dy/dz) * (z - tu_z))
    return x,y,z

##############################################

class LeastSquares:
    """
    Generic least-squares cost function with error.
    """

    errordef = ls.errordef # for Minuit to compute errors correctly

    def __init__(self, model, x, y, z, x_err, y_err):
        self.model = model  # model predicts y for given x
        self.x     = x
        self.y     = y
        self.z     = z
        self.x_err = x_err
        self.y_err = y_err

    def __call__(self, *par):  # we accept a variable number of model parameters
        xm,ym,zm = self.model(self.z, *par)
        value    = np.sum(np.sqrt( ((self.x - xm ) ** 2 /self.x_err ** 2) + ((self.y - ym)**2 / self.y_err ** 2))) 
        #thesum = np.sum(((self.y - ym) ** 2 / self.y_err ** 2) + ((self.z - zm) ** 2 / self.z_err **2 ))
        #return thesum
        return value

#########################################################

def line_fit(xs, ys, zs, errs_x=None, errs_y=None):
    
    if errs_x is None:
        errs_x = 10*np.ones(len(xs)) 
        errs_y = 10*np.ones(len(ys))

    # the line3d takes z values and needs 6 parameters
    model = LeastSquares(line3d, xs, ys, zs, errs_x, errs_y)

    # start values
    if len(xs) < 2:
        print("Not enough points!")
        return
    x,y,z = xs[0],ys[0],zs[0]
    dx    = xs[1] - xs[0]
    dy    = ys[1] - ys[0]
    dz    = zs[1] - zs[0]
    m = iminuit.Minuit(model, x, y, z, dx, dy, dz)
    # force one of the points on the upper paddle
    # (and we know that this is 16 cm wide
    #dwidth = 0.635/2
    
    #m.limits = [(xs[0] - 8, xs[0] + 8), (-90,90), (zs[0]-dwidth, zs[0]+dwidth),\
    #             (None, None), (None, None), (None, None)]
    m.migrad()
    m.migrad()

    def line(line_z_vals):
        return line3d(line_z_vals, *m.values)
    return line, m.fval

#########################################################3

class Reconstruction:

    def __init__(self):
        pass

    def reco(self, ev):
        xs = [k[0] for k in ev.tracker_pointcloud]
        xs.extend([h.x for h in ev.tof.hits])

        ys = [k[1] for k in ev.tracker_pointcloud]
        ys.extend([h.y for h in ev.tof.hits])

        zs = [k[2] for k in ev.tracker_pointcloud]
        zs.extend([h.z for h in ev.tof.hits])

        xs = np.array(xs)
        ys = np.array(ys)
        zs = np.array(zs)

        reco = line_fit(xs, ys, zs)
        return reco

#
#
#def linefit_trust_the_tof(tof_event, tracker_hits):
#    """
#    Try to fit a straight line, but "trust" the tof values.
#    That means we force the first hit on the upper and the
#    second hit on the lower tof. We use the tof hits as 
#    start values and constrain them on the actual paddles 
#    with the limits of the minimization.
#    
#    Keyword Arguments:
#        hits_blacklist (list) : allows to exclude hits from the 
#                                fit
#    """
#    xs         = np.array([h.x     for h in gaps_event.hits])
#    ys         = np.array([h.y     for h in gaps_event.hits])
#    zs         = np.array([h.z     for h in gaps_event.hits])
#    adc_data   = np.array([h.edep  for h in gaps_event.hits]) 
#    errs_x     = np.array([h.x_err for h in gaps_event.hits]) 
#    errs_y     = np.array([h.y_err for h in gaps_event.hits]) 
#
#
#    model    = LeastSquares(line3d_2pts, xs, ys, zs, errs_x, errs_y)
#    # hit 0 MUST be the upper and hit 1 MUST be the lower!!
#    # give the two tof hits as startvalues
#    m        = iminuit.Minuit(model,xs[0], ys[0], zs[0], xs[1], ys[1], zs[1])
#    #print (m.fixed)
#    # fix the z values
#    m.fixed['x2'] = True
#    m.fixed['x5'] = True
#    # force one of the points on the upper paddle
#    # (and we know that this is 16 cm wide
#    dwidth   = 0.635/2
#    m.limits = [(xs[0] - 8, xs[0] + 8), (-90,90), (zs[0]-dwidth, zs[0]+dwidth),\
#                (xs[1] - 8, xs[1] + 8), (-90,90), (zs[1]-dwidth, zs[1]+dwidth)]
#    
#    #print(m.limits)
#    m.migrad()
#    m.migrad()
#    m.hesse()
#    try:
#        #chi2 = m.fval/(len(xs) - m.nfit)
#        chi2 = m.fval
#    except ZeroDivisionError:
#        chi2 = np.nan
#    gaps_event.chi2 = chi2
#    # residuals
#    for jj,h in enumerate(gaps_event.hits):
#        reco_x, reco_y, reco_z = line3d_2pts(h.z, *m.values)
#        gaps_event.hits[jj].reco_residual_x = h.x - reco_x 
#        gaps_event.hits[jj].reco_residual_y = h.y - reco_y
#        gaps_event.hits[jj].reco_residual_z = h.z - reco_z 
#
#    def line(line_z_vals):
#        return line3d_2pts(line_z_vals, *m.values)
#
#    return line, gaps_event
#
#########################################################3
#
#
#def tracker_only_linefit(gaps_event):
#    """
#    This will fit a line only through the tracker hits. 
#    """
#
#    xs         = np.array([h.x     for h in gaps_event.hits if not (h.v_id.startswith('U') or h.v_id.startswith('L'))])
#    ys         = np.array([h.y     for h in gaps_event.hits if not (h.v_id.startswith('U') or h.v_id.startswith('L'))])
#    zs         = np.array([h.z     for h in gaps_event.hits if not (h.v_id.startswith('U') or h.v_id.startswith('L'))])
#    adc_data   = np.array([h.edep  for h in gaps_event.hits if not (h.v_id.startswith('U') or h.v_id.startswith('L'))]) 
#    errs_x     = np.array([h.x_err for h in gaps_event.hits if not (h.v_id.startswith('U') or h.v_id.startswith('L'))]) 
#    errs_y     = np.array([h.y_err for h in gaps_event.hits if not (h.v_id.startswith('U') or h.v_id.startswith('L'))]) 
#
#    model    = LeastSquares(line3d_2pts, xs, ys, zs, errs_x, errs_y)
#
#    # we use the first two hits (randomly) as the seed for the fit and
#    # constrain them within the limits of the tracker
#    m        = iminuit.Minuit(model,xs[0], ys[0], zs[0], xs[1], ys[1], zs[1])
#
#    dwidth   = 0.125
#    m.limits = [(xs[0] - 0.5, xs[0] + 0.5), (ys[0]-8, ys[0] + 8), (zs[0]-dwidth, zs[0]+dwidth),\
#                (xs[1] - 0.5, xs[1] + 0.5), (ys[1]-8, ys[1] + 8), (zs[1]-dwidth, zs[1]+dwidth)]
#
#    m.migrad()
#    m.migrad()
#    m.hesse()
#    try:
#        #chi2 = m.fval/(len(xs) - m.nfit)
#        chi2 = m.fval
#    except ZeroDivisionError:
#        chi2 = np.nan
#    gaps_event.chi2 = chi2
#    # residuals
#    for jj,h in enumerate(gaps_event.hits):
#        reco_x, reco_y, reco_z = line3d_2pts(h.z, *m.values)
#        gaps_event.hits[jj].reco_residual_x = h.x - reco_x 
#        gaps_event.hits[jj].reco_residual_y = h.y - reco_y
#        gaps_event.hits[jj].reco_residual_z = h.z - reco_z 
#
#    def line(line_z_vals):
#        return line3d_2pts(line_z_vals, *m.values)
#
#    return line, gaps_event
#
#
#class LineFit:
#    """
#    A simple line fit between 2 points
#    """
#
#    def __init__(self):
#        self.chi2_last_event = None
#        self.last_anchor     = None
#        self.last_direction  = None
#        self.last_fitstatus  = FitStatus.Unknown
#
#
#    def add_event(self, tof_ev, tracker_hits = [])
#        """
#        """
#        pass
#
#    def fit(self):
#        """
#        """
#        pass
