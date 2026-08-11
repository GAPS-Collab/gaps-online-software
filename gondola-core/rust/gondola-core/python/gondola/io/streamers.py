import zmq 
import time

from .. import _gondola_core  as _gc 

class TelemetrySocketReader:

    def __init__(self, address):
        ctx  = zmq.Context()
        sock = ctx.socket(zmq.SUB)
        sock.subscribe("")
        sock.connect(address)
        self.socket = sock 
    
    def __iter__(self):
        return self  # Return self, as this class is also the iterator

    def __next__(self):
        data  = self.socket.recv()
        pack  = _gc.packets.TelemetryPacket.from_bytestream(data, 0)
        frame = _gc.io.CRFrame()
        frame.put_telemetrypacket(pack) 
        return frame 


class PacketStreamer:

    def __init__(self, reader, address, rate = 500):
        self.reader  = reader 
        ctx          = zmq.Context()
        self.socket  = ctx.socket(zmq.PUB)
        self.socket.bind(address)
        self.rate    = rate 

    def stream(self):
        """
        This will block!
        """
        while True:
            pack = self.reader.__next__()
            if pack is None:
                self.reader.rewind()
                continue 
            self.socket.send(pack.to_bytestream())
            time.sleep(1/self.rate)
