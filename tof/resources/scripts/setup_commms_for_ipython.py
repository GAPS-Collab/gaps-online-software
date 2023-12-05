#! /usr/bin/env python3

import time
import zmq
import sys
import logging

import concurrent.futures as fut
from threading import Thread

import gaps_tof as gt

logging.basicConfig(format="%(levelname)s: %(message)s", level=logging.INFO)

REQUEST_TIMEOUT = 20000
REQUEST_RETRIES = 6
SERVER_ENDPOINT = "tcp://10.0.1.151:40000"
from pico_hal import read_event_cnt

SERVER_ENDPOINT_DATA = "tcp://10.0.1.151:30000"



available_commands = { \
  'RunStart'       : gt.TofCommand.DataRunStart, 
  'RunStop'        : gt.TofCommand.DataRunEnd,
  'RequestEvent'   : gt.TofCommand.RequestEvent,
  'StreamAnyEvent' : gt.TofCommand.StreamAnyEvent
}



def prepare_socket():
    ctx = zmq.Context()
    client = ctx.socket(zmq.REQ)
    client.connect(SERVER_ENDPOINT)
    client.send_string("[Client] - connected")
    resp = client.recv()
    return client

client = prepare_socket()

def eternal_listener():
    ctx = zmq.Context()
    client = ctx.socket(zmq.SUB)
    client.connect(SERVER_ENDPOINT_DATA)
    client.setsockopt(zmq.SUBSCRIBE, b"")
    while True:
        data = client.recv()
        #print ("SUB socket got something!")
        # for now, we know that must be a monii packet
        tp = gt.TofPacket()
        bytestream = [k for k in data]
        #print (len(data))
        #print (data[0:20])
        tp.from_bytestream([k for k in data],0)
        #print ("---- RECEIVED")
        #print (tp.packet_type)
        #print (tp.payload[0:20])
        if tp.packet_type == gt.PacketType.RBEvent : 
            rbevent = gt.get_events_from_stream([k for k in tp.payload], 0)
            print (rbevent)
        if tp.packet_type == gt.PacketType.Monitor: 
            moni = gt.RBMoniPacket()
            moni.from_bytestream([k for k in tp.payload], 0)
            print (moni)
        #print (data)
        print ("-------")
        #time.sleep(1)

#with fut.ThreadPoolExecutor(max_workers=1) as executor:
#     executor.map(eternal_listener, [1])

daemon = Thread(target=eternal_listener, daemon=True, name='Monitor')
daemon.start()

def send_command(command_string, value=0, client=client):
    cmd    = available_commands[command_string]
    cmd_pk = gt.CommandPacket(cmd, value)
    client.send(bytes(cmd_pk.to_bytestream()))
    response = gt.ResponsePacket(gt.TofResponse.Unknown, 0);
    reply = client.recv()
    response.from_bytestream([k for k in reply],0)
    print (response)
#
#
#data_client = ctx.socket(zmq.SUB)
#data_client.connect(SERVER_ENDPOINT_DATA)
#data_client.setsockopt(zmq.SUBSCRIBE, b"")
#client = ctx.socket(zmq.REQ)
#client.connect(SERVER_ENDPOINT)
#client.send_string("[Client] - connected")
#resp = client.recv()
#print (resp)
## let's create a tof packet for a command
#tp  = gt.TofPacket()
#
#event_cache = []
#missing = []
#time.sleep(10)
#while True:
#    # get a number of event ids and then work through 
#    # the cache
#    event_cache = missing
#    if not event_cache:
#        for n in range(10000):
#            event_cache.append(read_event_cnt())
#    # let the rb catch up
#    # FIXME - this must be multithreaded
#    time.sleep(5)
#    for evid in event_cache:
#        logging.info(f"Working on ev: {evid}")
#        request = str(evid).encode()
#        cmd = gt.CommandPacket(gt.TofCommand.RequestEvent, evid)
#        #tp.set_packet_type(gt.Command)
#        #tp.set_payload(cmd.to_bytestream())
#        logging.info("Sending (%s)", request)
#        client.send(bytes(cmd.to_bytestream()))
#        #time.sleep(0.5)
#        retries_left = REQUEST_RETRIES
#        while True:
#            if (client.poll(REQUEST_TIMEOUT) & zmq.POLLIN) != 0:
#                reply = client.recv()
#                if len(reply) > 5:
#                    response = gt.ResponsePacket(gt.TofResponse.Unknown, 0);
#                    response.from_bytestream([k for k in reply],0)
#                    logging.info("Server replied OK (%s)", response)
#                    if response.get_response() == gt.TofResponse.Success:
#                        print (f"Yay, {evid}")
#                        foo = data_client.recv()
#                        foo = [ k for k in foo]
#                        foo.extend([0,0,0])
#                        print (foo[0:50])
#                        print ("...")
#                        print (foo[-10:])
#                        event = gt.get_events_from_stream(foo,0)
#                        print (event[0])
#                        raise
#                    if response.get_response() == gt.TofResponse.EventNotReady:
#                        missing.append(evid)
#                    retries_left = REQUEST_RETRIES
#                    break
#                else:
#                    logging.error("Malformed reply from server: %s", reply)
#                    continue
#
#            retries_left -= 1
#            logging.warning("No response from server")
#            # Socket is confused. Close and remove it.
#            client.setsockopt(zmq.LINGER, 0)
#            client.close()
#            if retries_left == 0:
#                logging.error("Server seems to be offline, abandoning")
#                sys.exit()
#
#            logging.info("Reconnecting to server…")
#            # Create new connection
#            client = ctx.socket(zmq.REQ)
#            client.connect(SERVER_ENDPOINT)
#            logging.info("Resending (%s)", request)
#            client.send(request)
#
#
