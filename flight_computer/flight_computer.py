#! /usr/bin/env python3 


import zmq

import gaps_tof as gt

def print_event(event):
    print (f'event : {event.event_id} n_paddles {event.n_paddles}')

ctx = zmq.Context()
sock = ctx.socket(zmq.SUB)
#sock.connect("tcp://127.0.0.1:30000")
sock.connect("tcp://192.168.36.20:40000")
sock.subscribe("")
while True:
    data  = sock.recv()
    packet = gt.TofPacket()
    data = [k for k in data]
    packet.from_bytestream(data, 0)
    #print (packet)
    if packet.packet_type == gt.PacketType.TofEvent:
        print ("Got tof packet")
        event = gt.REventPacket()
        event.from_bytestream([k for k in packet.payload],0)
        print (event)
        #print (len(data))
        #print_event(event)
