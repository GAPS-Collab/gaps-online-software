#! /usr/inb/env python

"""
Simple example on how to read hdf files
"""

import h5py

f = h5py.File('nts_moni_3.h5','r')
f.keys()
data = f.get('RBMoniData')
for k in data:
    print ('<RBMoniData>')
    for name in k.dtype.names:
        print (f'{name} - {k[name]}')
