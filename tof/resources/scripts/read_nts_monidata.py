#! /usr/bin/env python

"""
Simplistic example about how to read h5 monitoring data
"""

import h5py
import numpy as np
f = h5py.File('nts_moni_10.h5')
moni = np.array(f['RBMoniData'])
moni['tmp_zynq'].flatten()
