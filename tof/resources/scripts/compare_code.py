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

    threshold = 10
    alltimes   = []
    allcharges = []
    allwaves   = []
    event_ids_over_thr = []
    for ev in allevents[:-105]:
        wave  = gt.voltage_calibration(ev, calibrations)
        times = gt.timing_calibration(ev, calibrations) 
        wave  = gt.remove_spikes(wave, ev)
        ch_over_thr = []
        ch_waves    = []
        ch_tdcs     = []
        ch_charges  = []

        for k in [0,2,4,6]:
            wf = np.array(wave[k])
            if wf[wf > threshold].any():
                tof_event = pbd.waveforms_to_hit(times[k], wave[k], times[k+1], wave[k+1])
                alltimes.extend([tof_event.t_at_cfdA])#, tof_event.t_at_cfdB])
                allcharges.extend([tof_event.chargeA])#, tof_event.chargeB])
                allwaves.append(wave[k])
                ch_over_thr.append(k)
                ch_waves.append(wf)
                ch_tdcs.append(tof_event.t_at_cfdA)
                ch_charges.append(tof_event.chargeA)
        if ch_over_thr:
           event_ids_over_thr.append((ev.event_ctr,\
                                      ch_over_thr,\
                                      ch_waves,\
                                      ch_tdcs,\
                                      ch_charges))
    console.print(f'=> We found {len(event_ids_over_thr)} events with waveforms over threashold in the raw file data')


    # compare to the rust code
    f = tables.open_file("/srv/gaps/gaps-online-software/tof/crusty_kraken/waveforms_2.hdf")
    f.root.waveforms.wf.read()
    hdfdata = f.root.waveforms.wf.read()

    # first, lets check if we see the same events which go
    # over threashold
    hdf_ev_over_thr = []
    for k in hdfdata:
        if k['wave'][k['wave'] > threshold].any():
            hdf_ev_over_thr.append(k['event_ctr'])
    
    # make sure we have the same events in the two samples
    console.print(f"=> We found {len(set(hdf_ev_over_thr))} events with waveforms over threshold in the hdf data")
    assert set([k[0] for k in event_ids_over_thr]) == set(hdf_ev_over_thr)

    tdcs    = []
    charges = []
    for evid, channels, __, __, __ in event_ids_over_thr:
        event_hdfdata = hdfdata[hdfdata['event_ctr'] == evid]
        if not event_hdfdata['event_ctr'].any():
            continue
        for ch in channels:
            try:
                tdcs.append(event_hdfdata['tdcs'][ch])
                charges.append(event_hdfdata['charge'][ch][0])
            except:
                continue
#hist = d.factory.hist1d(alltimes, 100)
    #hist.line()
    #p.show()
    console.print(f'=> We found {len(tdcs)} cfd times for the RUST code')
    console.print(f'=> We found {len(alltimes)} cfd times for the C++ code')
