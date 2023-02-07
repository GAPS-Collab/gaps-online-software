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
        data = [k for k in packet.payload]
        event.from_bytestream(data,0)
        print (f".. event {event.event_id}, .. no paddle packets")
        if len(data) > 9:
            print (event)
            for k in event.paddle_packets:
                print (k)
        #print (len([k for k in packet.payload]))
        #print (event)
        #print (len(data))
        #print_event(event)
