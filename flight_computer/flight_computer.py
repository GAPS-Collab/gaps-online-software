#! /usr/bin/env python 


import zmq

import gaps_tof as gt

def print_event(event):
    print (f'event : {event.event_id} n_paddles {event.n_paddles}')

ctx = zmq.Context()
sock = ctx.socket(zmq.SUB)
sock.connect("tcp://127.0.0.1:30000")
sock.subscribe("")
while True:
    data  = sock.recv()
    event = gt.REventPacket()
    data = [k for k in data]
    event.deserialize(data, 0)
    print (len(data))
    print_event(event)
