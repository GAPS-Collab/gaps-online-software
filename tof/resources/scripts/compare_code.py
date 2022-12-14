#! /usr/bin/env python


import pb_dataclasses as pbd
import gaps_tof as gt

import rich.console
console = rich.console.Console()

import numpy as np
import pylab as p
import tables

import dashi as d 
d.visual()



if __name__ == '__main__':
  
    filename = '/data0/gfp-data-aug/Aug/run4a/rb2.dat'
    console.print(f'=> Reading file {filename}')
    data = open(filename, 'rb')
    data = data.read()
    bytestream = []
    for k in data:
        bytestream.append(k)
    console.print('=> Getting events from datastream')
    #len(bytestream)
    allevents = gt.get_events_from_stream(bytestream, 0)
    console.print('=> Getting calibrations..')
    calibrations = gt.read_calibration_file('/srv/gaps/gfp-data/gaps-gfp/TOFsoftware/server/datafiles/rb2_cal.txt')

    threshold = 5
    alltimes = []
    for k in range(1000):
         
        ev = allevents[k]
        wave = gt.voltage_calibration(ev, calibrations)
        times = gt.timing_calibration(ev, calibrations) 
        for k in [0,2,4,6]:
            wf = np.array(wave[k])
            if wf[wf > threshold].any():
                tof_event = pbd.waveforms_to_hit(times[k], wave[k], times[k+1], wave[k+1])
                alltimes.extend([tof_event.t_at_cfdA, tof_event.t_at_cfdB])

    # compare to the rust code
    f = tables.open_file("/srv/gaps/gaps-online-software/tof/crusty_kraken/waveforms_2.hdf")
    f.root.waveforms.wf.read()
    hdfdata = f.root.waveforms.wf.read()
    tdcs = []
    for k in hdfdata:
        tdcs.append(k['tdcs'][0])

    hist = d.factory.hist1d(alltimes, 100)
    hist.line()
    #p.show()
        
