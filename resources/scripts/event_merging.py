#! /usr/bin/env python

import gsequery
import gaps_tof as gt
from bind.merged_event_bindings import merged_event

# this source is  the db with the merged events
data = gsequery.GSEQuery(path="/srv/gaps/gaps-online-software/data/gsedb.sqlite")
print (data.get_table_names())

# get all data for this run
events = data.get_rows1("mergedevent", 0, 999999999999999)
print (f"We got {len (events)} events for this run")

# this source is the stream with the master trigger events
streamfile = open('/srv/gaps/gaps-online-software/data/stream_0.tof.gaps',"rb")
stream = [k for k in streamfile.read()]
print("Stream read!")
merged_evids = []
master_evids = []

packets = gt.get_tofpackets_from_stream(stream, 0)
print (stream[:1000])
print (f"We got {len(packets)} packets from the stream")
mt_packets = [k for k in packets if k.packet_type == gt.PacketType.MasterTrigger]
print (f"{len(mt_packets)} of these are master trigger packets!")

raise

for ev in events:

    me = merged_event()
    me.unpack_str(ev[10],0)
    tp = gt.TofPacket()
    tp.from_bytestream([k for k in me.tof_data],0)
    if tp.packet_type == gt.PacketType.TofEvent:
        te = gt.REventPacket()
        te.from_bytestream([k for k in tp.payload],0)
        merged_evids.append(me.event_id)
        print (me.event_id, te.event_id)
    elif tp.packet_type == gt.PacketType.MasterTrigger:
        print ("Got MT event")  
        raise
        pass
    else:
        print (f"Got merged event, but tof packet type is {tp.packet_type}")
