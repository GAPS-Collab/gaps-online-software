#! /usr/bin/env python3 


import zmq
import time

import gaps_tof as gt

def print_event(event):
    print (f'event : {event.event_id} n_paddles {event.n_paddles}')

ctx = zmq.Context()
sock = ctx.socket(zmq.SUB)
#sock.connect("tcp://127.0.0.1:30000")
sock.connect("tcp://192.168.36.20:40000")
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
    if packet.packet_type == gt.PacketType.TofEvent:
        #print ("Got tof packet")
        event = gt.REventPacket()
        data = [k for k in packet.payload]
        event.from_bytestream(data,0)
        #print (f".. event {event.event_id}, .. no paddle packets")
        #if len(data) > 9:
        #    print (event)
        #    for k in event.paddle_packets:
        #        print (k)
        all_events.append(event)
        #if len(all_events) % 100 == 0:
        #    print ([k.event_id for k in all_events])
        npackets += 1
        #print (f"received {npackets} packets, delta t {time.time() - now}")

    if npackets % 100 == 0:
        print (f'=======')
        print (f' last event {event.event_id}')
        print (f"received {npackets} packets, delta t {time.time() - now}")
        now = time.time()

        #print (len([k for k in packet.payload]))
        #print (event)
        #print (len(data))
        #print_event(event)
