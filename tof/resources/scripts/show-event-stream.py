#! /usr/bin/env python

import zmq
import gaps_tof as gt

if __name__ == '__main__':
    ctx = zmq.Context()
    sock = ctx.socket(zmq.SUB)
    sock.connect("tcp://100.96.207.91:42000")
    #sock.connect('tcp://localhost:42000')
    sock.subscribe('')
    nevents = 0
    while True:    
        data = sock.recv()
        #pack = gt.TofPacket.from_bytestream([k for k in data], 4)
        #print (data)
        pack = gt.TofPacket.from_bytestream([k for k in data],0)
        print (f' --> nevents {nevents} next packet {pack}')
        if pack.packet_type == gt.PacketType.MasterTrigger:
            ev = gt.MasterTriggerEvent.from_bytestream(pack.payload, 0)
            print (ev)
            continue
        if pack.packet_type == gt.PacketType.PT_RBEvent:
            ev = gt.RBEvent.from_bytestream(pack.payload, 0)
            #if ev.header.channel_mask < 500
            print (ev)
            nevents += 1
            #if ev.header.channel_mask != 0:
            #    raise
        if pack.packet_type == gt.PacketType.Monitor:
            moni = gt.RBMoniData.from_bytestream(pack.payload, 0)
            print (moni)
