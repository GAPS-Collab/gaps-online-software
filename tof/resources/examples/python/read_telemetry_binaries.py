#! /usr/bin/env python

# Read TOF data from the "official" gaps binary stream

import gaps_online as go
tl = go.telemetry

idx = 0
# open a packet reader
treader = tl.TelemetryPacketReader('example-data/RAW240815_044946.bin')
for k in treader:
    # 90 is merged event packet (which is the only type 
    # besides interesting event (that is the same type)
    # which has tof data
    if k.header.packet_type == 90:
        idx += 1 
        me = rt.MergedEvent()
        # TelemetryPacketReader emits TelemetryPacket
        me.from_telemetrypacket(k)
        break
    continue
    # this is tracker data
    if k.header.packet_type == 80:
        idx += 1
        tp = rt.TrackerPacket()
        try:
            tp.from_telemetrypacket(k)
            if len (tp.events) > 0:
                break
            #print (tp)
            #pds.append(tp.header.packet_id)
        except Exception  as e: 
            print (e)
            continue
        if idx > 20:
            break
idx = 0
for k in treader:
    if k.header.packet_type == 90:
        idx += 1 
        print (k)
        me = rt.MergedEvent()
        me.from_telemetrypacket(k)
        break
    continue
    if k.header.packet_type == 80:
        idx += 1
        print (k)
        print (len(k.payload))
        tp = rt.TrackerPacket()
        try:
            tp.from_telemetrypacket(k)
            if len (tp.events) > 0:
                break
            #print (tp)
            #pds.append(tp.header.packet_id)
        except Exception  as e: 
            print (e)
            continue
        if idx > 20:
            break
for k in treader:
    if k.header.packet_type == 90:
        idx += 1 
        print (k)
        me = tl.MergedEvent()
        me.from_telemetrypacket(k)
        break
    continue
    if k.header.packet_type == 80:
        idx += 1
        print (k)
        print (len(k.payload))
        tp = tl.TrackerPacket()
        try:
            tp.from_telemetrypacket(k)
            if len (tp.events) > 0:
                break
            #print (tp)
            #pds.append(tp.header.packet_id)
        except Exception  as e: 
            print (e)
            continue
        if idx > 20:
            break
