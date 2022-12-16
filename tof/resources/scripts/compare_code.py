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
    data = data.read(gt.get_current_blobevent_size()*1000)
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

    event_ids_over_thr = []
    for k in range(len(allevents)):
         
        ev = allevents[k]
        wave = gt.voltage_calibration(ev, calibrations)
        times = gt.timing_calibration(ev, calibrations) 
        ch_over_thr = []
        for k in [0,2,4,6]:
            wf = np.array(wave[k])
            if wf[wf > threshold].any():
                tof_event = pbd.waveforms_to_hit(times[k], wave[k], times[k+1], wave[k+1])
                alltimes.extend([tof_event.t_at_cfdA, tof_event.t_at_cfdB])
                ch_over_thr.append(k)
        if ch_over_thr:
           event_ids_over_thr.append((ev.event_ctr, ch_over_thr))
    console.print(f'=> We found {len(event_ids_over_thr)} events with waveforms overthreashold')

    # compare to the rust code
    f = tables.open_file("/srv/gaps/gaps-online-software/tof/crusty_kraken/waveforms_2.hdf")
    f.root.waveforms.wf.read()
    hdfdata = f.root.waveforms.wf.read()
    tdcs = []
    thisevent = 0
    for k in event_ids_over_thr:
        event_hdfdata = hdfdata[hdfdata['event_ctr'] == k[0]]
        if not event_hdfdata['event_ctr'].any():
            continue
        for ch in k[1]:
            try:
                tdcs.append(event_hdfdata['tdcs'][ch])
            except:
                continue
#hist = d.factory.hist1d(alltimes, 100)
    #hist.line()
    #p.show()
        
