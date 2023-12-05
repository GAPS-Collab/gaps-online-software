#! /usr/bin/env python3 


import zmq
import time
import numpy as np
import gaps_tof as gt

def print_event(event):
    print (f'event : {event.event_id} n_paddles {event.n_paddles}')

VERBOSE=False

ctx = zmq.Context()
sock = ctx.socket(zmq.SUB)
#sock.connect("tcp://127.0.0.1:30000")
sock.connect("tcp://192.168.37.20:40000")
sock.subscribe("")
all_events = []

npackets = 0
now = time.time()

while True:
    data  = sock.recv()
    packet = gt.TofPacket()
    data = [k for k in data]
    packet.from_bytestream(data, 0)
    #print (packet)
    
    if packet.packet_type != gt.PacketType.TofEvent:
        #print (packet.packet_type)
        continue
    if packet.packet_type == gt.PacketType.TofEvent:
        event = gt.REventPacket()
        data = [k for k in packet.payload]
        event.from_bytestream(data,0)
        all_events.append(event.event_id)
        #if len(all_events) % 100 == 0:
        #    print ([k.event_id for k in all_events])
        npackets += 1
        if VERBOSE:
            pass
            #print (f".. event {event.event_id}, .. no paddle packets")
            #if len(data) > 9:
            #    print (event)
            #    for k in event.paddle_packets:
            #        print (k)
            #print (f"received {npackets} packets, delta t {time.time() - now}")
    #if len(data) > 15:
    #    print (f'=======')
    #    print (f' last event {event.event_id} {event.n_paddles}')
    #    #print (data)
    #    print (event)
    #    for k in event.paddle_packets:
    #        print (k)
    #    #raise

    if npackets % 100 == 0:
        print (f'=======')
        print (f' last event {event.event_id} {event.n_paddles}')
        #print (data)
        #print (event)
        #for k in event.paddle_packets:
        #    print (k)
        ##if len(event.paddle_packets) > 4:
        ##    print(event)
        ##    for k in event.paddle_packets:
        ##        print (k)
        ##    #raise
        ##if event.n_paddles > 0:
        ##    print(event)
        ##    for k in event.paddle_packets:
        ##        print (k)
        ##    #if len(event.paddle_packets) > 1:
        ##    #    raise
        ##    #if not event.is_broken():
        ##    #     print(event.paddle_packets[0])
        ##    #raise
        print (f"received {npackets} packets, delta t {time.time() - now}")
        now = time.time()
        all_events = np.array(all_events)
        missing = all_events[1:] - all_events[:-1]
        print (missing)
        print (f'missing : {missing.sum() - 100}')
        all_events = []
        #print (len([k for k in packet.payload]))
        #print (event)
        #print (len(data))
        #print_event(event)
