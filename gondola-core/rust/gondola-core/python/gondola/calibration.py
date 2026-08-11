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
