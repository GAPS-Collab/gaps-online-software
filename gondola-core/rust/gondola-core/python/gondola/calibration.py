"""
Calibration related methods
"""

from pathlib import Path
import re
import tqdm
import numpy as np

from . import _gondola_core 
from .visual.style import gander_line_plot 

RBCalibrations = _gondola_core.calibration.RBCalibrations
RBCalibrations.__module__ = __name__ 
RBCalibrations.__name__   = 'RBCalibrations'

TrackerOnlineCalibration = _gondola_core.tracker.TrackerOnlineCalibration 
TrackerOnlineCalibration.__module__ = __name__ 
TrackerOnlineCalibration.__name__   = 'TrackerOnlineCalibration'

TrackerOfflineCalibration = _gondola_core.calibration.TrackerOfflineCalibration 
TrackerOfflineCalibration.__module__ = __name__ 
TrackerOfflineCalibration.__name__   = 'TrackerOfflineCalibration'

## convenience functions
def load_rb_calibrations(cali_dir : Path, load_event_data = False):
    """
    Load all calibrations stored in a certain directory and
    return a dictionary rbid -> RBCalibration

    # Arguments:
        * cali_dir        : Path with calibration files, one per RB

    # Keyword Arguments: 

        * load_event_data : if True, also load the associated events
                            which went into the calculation of the
                            calibration constants.
    """
    pattern = re.compile('RB(?P<rb_id>[0-9]*)_')
    calib_files = [k for k in cali_dir.glob("*.tof.gaps")]
    calibs = dict()
    for fname in tqdm.tqdm(calib_files, desc="Loading calibration files"):
        fname = str(fname)
        try:
            rb_id = int(pattern.search(fname).groupdict()['rb_id'])
        except Exception as e:
            print(f'Failed to get RB ID from file {fname}')   
            continue
        cali = RBCalibrations.from_file(fname)
        calibs[rb_id] = cali 
    return calibs

#------------------------------------------------

def _plot_transfer_fn(self,strip_id):
    """
    Plot the transfer function for the respective strip over a 
    reasonable ADC range (0-1600) 
    """
    if strip_id in self.tf_map.keys():
        transfer_fn = self.tf_map[strip_id].transfer_fn 
        xs = np.arange(0,1600,1)
        ys = np.array([transfer_fn(x) for x in xs])
        fig = gander_line_plot(xs,ys, title=f'Transfer Fn strip {strip_id}', xlabel='ADC', ylabel='mV')
        return fig
    print(f"Unable to plot transfer fn for {strip_id}. Are the transfer functions loaded/set?")

TrackerOfflineCalibration.plot_transfer_fn = _plot_transfer_fn 

#------------------------------------------------

def apply_energy_calibration_tof(edep, cc, temperature_paddle):
    """
    The gain of the paddles is temperature dependent. 
    Correct the expected deposited energy by means of utilizing 
    the individual paddle temperature. 

    The correction is intended to unify the energy depositios accross 
    the paddles. It is an effect on the order of a few % and will 
    not deal with saturation. 

    # Arguments:
        * edep               - edep calcuated by whatever formula of choice. The 
                               value is for a single paddle 
        * cc                 - this is a dictionary, essentially provided by 
                               Grace's analysis, with linear fits to
                               temperature/edep data.
        * temperature_paddle - the temperature of this paddle at the time the 
                               energy deposition was supposed to be measured
    """
    ## we need to apply the slope and y_intercept to each temperature value, to get a predicted MPV from the fit: 
    slope          = cc["slope"]
    y_int          = cc["y_intercept"]
    t_avg          = cc["T_avg"] 
    mpv_predicted  = temperature_paddle*slope + y_int
    mpv_reference  = t_avg*slope + y_int
    # temperature calibration formula:
    edep_corrected = edep*(mpv_reference/mpv_predicted)
    # this coefficient of 1.22 is obtained by simulation 
    # there is a difference if either the total energy 
    # deposition is used or dE/dx
    coefficient    = 1.22/cc["coefficient"]
    return edep_corrected*coefficient


