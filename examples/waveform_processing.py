#! /usr/bin/env python 

import gondola as gon
import tqdm as tqdm

cali_dir = '/data1/gaps/mcmurdo/tof/calib/241209_123432UTC/'

# analysis engine configuration
ana_set = gon._gondola_core.tof.AnalysisEngineSettings()

# RB44 as an example 
rb = gon.db.ReadoutBoard.all_as_dict()[44]
rb.load_calibration(cali_dir)

# Load some data, can also be loaded from L0
reader = gon.io.TofPacketReader('/data1/gaps/mcmurdo/tof/9125')
#npacks = reader.count_packets()
npacks = 10000
n      = 0
for pack in tqdm.tqdm(reader, total=npacks):
    if pack.packet_type != gon.packets.TofPacketType.TofEventDeprecated:
        continue 
    n += 1
    ev = gon.events.TofEvent.from_tofpacket(pack)
    for k in ev.rb_events:
        if k.header.rb_id == 44:
            try:
                gon.tof.waveform_analysis(k, rb, ana_set)
            except Exception as e:
                print (e)
    if n >= npacks:
        break
