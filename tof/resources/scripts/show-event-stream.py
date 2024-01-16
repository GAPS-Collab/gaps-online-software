#! /usr/bin/env python

import zmq
import gaps_tof as gt

if __name__ == '__main__':
    ctx = zmq.Context()
    sock = ctx.socket(zmq.SUB)
    sock.connect("tcp://100.101.96.10:42000")
    #sock.connect("tcp://100.96.207.91:42001")
    #sock.connect('tcp://localhost:42000')
    sock.subscribe('')
    nevents = 0
    n_tofevents = 0
    while True:    
        data = sock.recv()
        if data.startswith(b"RB"):
            data = data[4:]
        #pack = gt.TofPacket.from_bytestream([k for k in data], 4)
        #print (data)
        pack = gt.TofPacket.from_bytestream([k for k in data],0)
        #print (f' --> nevents {nevents} next packet {pack}')
        match pack.packet_type:
            case gt.PacketType.MasterTrigger:
                ev = gt.MasterTriggerEvent.from_bytestream(pack.payload, 0)
                print (ev)
                continue
            case gt.PacketType.RBEvent:
                ev = gt.RBEvent.from_bytestream(pack.payload, 0)
                #if ev.header.channel_mask < 500
                print (ev)
                nevents += 1
                #if ev.header.channel_mask != 0:
                #    raise
            case gt.PacketType.TofEvent:
                ev = gt.TofEvent.from_bytestream(pack.payload, 0)
                n_tofevents += 1
            case gt.PacketType.RBMoniData:
                moni = gt.RBMoniData.from_bytestream(pack.payload, 0)
                print (moni)
            case gt.PacketType.MtbMoniData :
                moni = gt.MtbMoniData.from_bytestream(pack.payload, 0)
            case _ :
                print ([int(k) for k in data][:10])
                print (f"Unknown packet type! {pack}")
