#! /usr/bin/env python

# Read TOF data from the "official" gaps binary stream

import gondola as go

n_events = 0
# open a packet reader
treader = go.io.TelemetryPacketReader('example_data/RAW260101_000030.bin')
for pack in treader:
    # there is multiple packet types which can hold "merged" event informtion
    # (this is the "interesting event" mechanism
    if pack.is_event_packet():
        n_events += 1 
        ev        = go.events.TelemetryEvent.from_telemetrypacket(pack)
        # this calculates the variables which are used for the interesting 
        # event algorithm from the TOF side 
        ev.tof.calc_gcu_variables()
        print(ev)
        print(ev.tof.nhits_cbe)
        print(ev.tof.nhits_umb)
        print(ev.tof.nhits_cor)
        print(ev.tof.edep_cor)
        print(ev.tof.edep_umb)
        print(ev.tof.edep_cbe)
        break
