#! /usr/bin/env python

# Read TOF data from the "official" gaps binary stream
from time import sleep
from glob import glob
from struct import Struct
from argparse import ArgumentParser

import gaps_online as go

p = ArgumentParser()
p.add_argument('glob')
p.add_argument('--fastmode',action='store_true')
args = p.parse_args()

tl = go.telemetry
E_idx = 0
T_idx = 0

fnames = glob(args.glob)
fnames.sort()

for fname in fnames:
    # open a packet reader
    treader = tl.TelemetryPacketReader(fname)
    while 1:
        for k in treader:
            # 90 is merged event packet (which is the only type 
            # besides interesting event (that is the same type)
            # which has tof data
            if k.header.packet_type == 90:
                E_idx += 1
                #print(k)
                me = tl.MergedEvent()
                try:
                    me.from_telemetrypacket(k)
                    #print (me.event_id)
                    #if me.tracker_events.len() > 0:
                    #   break
                    print (me)
                except Exception  as e: 
                    print (e)
                #me.from_telemetrypacket(k)
                #print(k)
                # TelemetryPacketReader emits TelemetryPacket
                sleep(1)
                break
                
            #continue
            # this is tracker data
            elif k.header.packet_type == 80:
                T_idx += 1
                #print (k, k.header.packet_type)
                #print (len(k.payload))
                tp = tl.TrackerPacket()
                try:
                    tp.from_telemetrypacket(k)
                    if len (tp.events) > 0:
                        break
                    #print (tp, tp.header.packet_id)
                    #pds.append(tp.header.packet_id)
                except Exception  as e: 
                    print (e)
                    #continue
                if T_idx > 200:
                    break
                
            else:
                print (k.header.packet_type)
                
