"""
Calibration routines and helpers for the tracker
"""

import matplotlib.pyplot as plt
import numpy as np
import tqdm

from copy import deepcopy as copy
from pathlib import Path

from ..db import TrackerStrip

#----------------------------------------------------------------

def get_transfer_functions(filename : Path) -> dict:
    """
    Load the transfer functions from a file and 
    provide them in a dictionary

    # Arguments:
        * filename : A path to a text file with the fitted constants from Riccardo's polyfit
    """
    transfer_fn = dict()
    total_lines = 0
    with open(filename) as f:
        for line in f.readlines():
            total_lines += 1
    
    with open(filename) as f:
        for line in tqdm.tqdm(f.readlines(), total=total_lines):
            if line.startswith('#'):
                continue
            line = line.split(',')
            #Layer Row Module Channel p0a p1a p2a p0b p1b p2b p3b p0c p1c p2c p3c p0d p1d p2d p3d   
            #p0a p1a p2a is pol2 fit between 0-190 ADC ch 
            #p0b p1b p2b p3b is pol3 fit between 190-500 ADC ch 
            #p0c p1c p2c p3c is pol3 fit between 500-900 ADC ch 
            #p0d p1d p2d p3d is pol3 fit between 900-1600 ADC ch 
            layer, row, module, channel = int(line[0]), int(line[1]), int(line[2]), int(line[3])
            strip_id = TrackerStrip.create_id(layer, row, module, channel)
            pol_a2_0, pol_a2_1, pol_a2_2 = [float(k) for k in line[4:7]]
            #print (pol_a2_0, pol_a2_1, pol_a2_2)
            #print ('----')
            pol_b3_0, pol_b3_1, pol_b3_2, pol_b3_3 = [float(k) for k in line[7:11]]
            pol_c3_0, pol_c3_1, pol_c3_2, pol_c3_3 = [float(k) for k in line[11:15]]
            pol_d3_0, pol_d3_1, pol_d3_2, pol_d3_3 = [float(k) for k in line[15:19]]
    
            def poly_a(xs):
                ys = np.zeros(len(xs))
                mask = xs <= 190 
                ys[mask] = pol_a2_0 + pol_a2_1*xs[mask] + pol_a2_2*(xs[mask]**2) 
                return ys
    
            def poly_b(xs):
                ys = np.zeros(len(xs))
                mask = np.logical_and(190 < xs, xs <= 500)
                ys[mask] = pol_b3_0 + pol_b3_1*xs[mask] + pol_b3_2*(xs[mask]**2) + pol_b3_3*(xs[mask]**3) 
                return ys
    
            def poly_c(xs):
                ys = np.zeros(len(xs))
                mask = np.logical_and(500 <  xs, xs <= 900)
                ys[mask] = pol_c3_0 + pol_c3_1*xs[mask] + pol_c3_2*(xs[mask]**2) + pol_c3_3*(xs[mask]**3) 
                return ys
    
            def poly_d(xs):
                ys = np.zeros(len(xs))
                mask = np.logical_and(900 <  xs, xs <= 1600)
                ys[mask] = pol_d3_0 + pol_d3_1*xs[mask] + pol_d3_2*(xs[mask]**2) + pol_d3_3*(xs[mask]**3) 
                return ys
            
            def trafo(xs):
                if isinstance(xs, float) or isinstance(xs, int):
                    xs = np.array(xs)
                a = poly_a(xs)
                b = poly_b(xs)
                c = poly_c(xs)
                d = poly_d(xs) 
                ys = a + b + c + d
                return ys
    
            transfer_fn[strip_id] = copy(trafo)
            #xs = np.arange(0,1600,1)
            #fig = plt.figure()
            #ax = fig.gca()
            ##ax.plot(xs, trafo(xs))
            #ax.plot(trafo(xs),xs)
            #fig.savefig(f'trafotest{strip_id}.png')
            ##trafos[stripid] = trafo        
            ##print (strip_id)
            ##print (line)
    return transfer_fn

#--------------------------------------------------------------

def get_energy(adc, fn):
    """
    Calculate the energy based on the given transfer functions

    # Arguments:
        * adc : adc value of the strip 
        * fn  : The actual transfer function
    """
    energy = 0
    if adc <=0:
        return energy
    if adc > 1600:
        adc = 1600
    mV2keV  = 0.841
    # the max and min range are basically defined by the 
    # polynominal [0-1600]
    adc     = np.array([adc])
    voltage = fn(adc)[0] # FIXME
    energy  = voltage*mV2keV
    energy /= 1000
    return energy
