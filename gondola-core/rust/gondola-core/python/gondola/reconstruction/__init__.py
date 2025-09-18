"""
GAPS first guess & prototype event reconstructions

"""

from enum import Enum
import numpy as np

import iminuit
from iminuit.cost import LeastSquares as ls

import dashi as d
import json

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

def line3d(z, x_a, y_a, z_a, dx, dy, dz):
    """
    describe line depending on z since that is our
    best constrained value

    This model has 6 free parameters, 3 for 
    the anchor point and 3 for the direction
    """
    # FIXME - proper treatment of ZeroDivisionError
    if dz == 0:
        dz = 1e-5
    x = x_a + ((dx/dz) * (z - z_a))
    y = y_a + ((dy/dz) * (z - z_a))
    return x,y,z

####################################################

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
        xm,ym,__ = self.model(self.z, *par)
        value    = np.sum(np.sqrt( ((self.x - xm ) ** 2 /self.x_err ** 2) + ((self.y - ym)**2 / self.y_err ** 2))) 
        #thesum = np.sum(((self.y - ym) ** 2 / self.y_err ** 2) + ((self.z - zm) ** 2 / self.z_err **2 ))
        #return thesum
        return value

#########################################################

def line_fit(xs, ys, zs, errs_x=None, errs_y=None, errs_z=None, search_anchor = False):
    """

    # Arguments:

        * search anchor : perform the fit multiple times, once per point. Select a different 
                          anchor point for the line each time (one of the hits) until all hits
                          have been used. Return the line with the best chi2.
    """

    assert len(xs) == len(ys) == len(zs)

    if errs_x is None:
        errs_x = 10*np.ones(len(xs)) 
    if errs_y is None:
        errs_y = 10*np.ones(len(ys))
    if errs_z is None:
        errs_z = 1*np.ones(len(zs))
    
    # rerutn a line for a convenience

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

    if search_anchor:
        #print ('RECO - iterative method (anchor search)')
        current_best = (m.fval, m.values)
        for k in range(1,len(xs)):
            x,y,z    = xs[k],ys[k],zs[k]
            dx,dy,dz = xs[k] - xs[k-1], ys[k] - ys[k-1], zs[k] - zs[k-1]
            m = iminuit.Minuit(model, x, y, z, dx, dy, dz)
            m.migrad()
            m.migrad()
            #recps.append((m.fval, m.values)
            if m.fval < current_best[0]:
                current_best = (m.fval, m.values)
            #print (f"RECO : Use anchor {x:.2f} {y:.2f} {z:.2f} ==> chi2 {m.fval:.2f}")
        def line(line_z_vals):
            return line3d(line_z_vals, *current_best[1])
        return line, current_best[0]  
    else:
        #print ('RECO - non-iterative method')
        def line(line_z_vals):
            return line3d(line_z_vals, *m.values)

        return line, m.fval

#########################################################3

class Reconstruction:

    def __init__(self, nbins = 100, active = False):
        self.active    = active
        self.CHIBINS   = np.linspace(0,500, nbins)
        self.COS2_BINS = np.linspace(0,1.1, nbins)
        self.BETA_BINS = np.linspace(-0.5,1.5, nbins)
        self.nevents   = 0
        self.event_cache_size = 1000000
        self.chi2_c    = []
        self.chi2      = d.histogram.hist1d(self.CHIBINS)
        self.cos2_c    = []
        self.cos2      = d.histogram.hist1d(self.COS2_BINS)
        self.beta_c    = []
        self.beta      = d.histogram.hist1d(self.BETA_BINS)
        self.finished  = False
        self.offsets = dict()
        offsets = json.load(open('offsets.json'))
        for k in offsets:
            k_int = int(k)
            # currently we only have intra-panel calibrations
            if k_int <= 12:
                self.offsets[k_int] = offsets[k]

    def __iadd__(self, other):
        self.chi2_c.extend(other.chi2_c)
        self.cos2_c.extend(other.cos2_c)
        self.beta_c.extend(other.beta_c)
        self.chi2 += other.chi2
        self.cos2 += other.cos2 
        self.beta += other.beta
        self.nevents += other.nevents
        return self

    def __add__(self, other):
        new_reco = Reconstruction()
        new_reco += self
        new_reco += other
        return new_reco

    def fill_histograms(self):
        if len(self.chi2_c) > self.event_cache_size:
            self.chi2.fill(np.array(self.chi2_c))
            self.chi2_c.clear()
            self.cos2.fill(np.array(self.cos2_c))
            self.cos2_c.clear()
            self.beta.fill(np.array(self.beta_c))
            self.beta_c.clear()

    def finish(self):
        ev_cache_size = self.event_cache_size
        self.event_cache_size = 1
        self.fill_histograms()
        self.finished = True
        self.event_cache_size = ev_cache_size


    def reco(self, ev):
        xs = [k[0] for k in ev.tracker_pointcloud]
        xs.extend([h.x for h in ev.tof.hits])

        ys = [k[1] for k in ev.tracker_pointcloud]
        ys.extend([h.y for h in ev.tof.hits])

        zs = [k[2] for k in ev.tracker_pointcloud]
        zs.extend([h.z for h in ev.tof.hits])

        ev.tof.normalize_hit_times()
        ev.tof.set_timing_offsets(self.offsets)
        ev.tof.remove_non_causal_hits()
        ev.tof.lightspeed_cleaning(0.35)
        ts = [(h.event_t0,h.x,h.y,h.z) for h in ev.tof.hits]
        ts = sorted(ts, key=lambda x: x[0])
        first_t = ts[0][0]
        last_t  = ts[-1][0]
        diff_h = last_t - first_t
        xs = np.array(xs)
        ys = np.array(ys)
        zs = np.array(zs)
        if len(xs) < 2:
            return
        reco = line_fit(xs, ys, zs)
        self.chi2_c.append(reco[1])
        
        # get the cosine by just selecting
        # two points on the line
        reco_f = reco[0]
        a = reco_f(0)
        b = reco_f(1000)
        d = np.sqrt((a[0] - b[0])**2 + (a[1] - b[1])**2 + (a[2] - b[2])**2)
        cos_theta = (b[2] - a[2])/d
        dist = np.sqrt((ts[-1][1] - ts[0][1])**2 + (ts[-1][2] - ts[0][2])**2 + (ts[-1][3] - ts[0][3])**2)
        beta = (dist/1000)/(diff_h*1e-9)/299792458  
        print (f'RECO BETA {beta}, DIST {dist}, TDIFF {diff_h}')
        self.beta_c.append(beta)
        self.cos2_c.append(cos_theta*cos_theta)
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
