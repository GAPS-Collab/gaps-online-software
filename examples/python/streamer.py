#! /usr/bin/env python

"""
Stream packets with a certain rate for testing
"""

import gondola as gon
reader = gon.io.TofPacketReader('/data1/gaps/mcmurdo/tof/9125')
streamer = gon.io.streamers.PacketStreamer(reader, "tcp://127.0.0.1:33333")
streamer.stream()
