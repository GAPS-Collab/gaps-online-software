"""
Calibration related methods
"""

from pathlib import Path
import re
import tqdm

from . import _gondola_core 
RBCalibrations = _gondola_core.calibration.RBCalibrations
RBCalibrations.__module__ = __name__ 
RBCalibrations.__name__   = 'RBCalibrations'

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

