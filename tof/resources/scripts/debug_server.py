#! /usr/bin/env python3

import time
import zmq
import sys
import logging

import gaps_tof as gt

logging.basicConfig(format="%(levelname)s: %(message)s", level=logging.INFO)

REQUEST_TIMEOUT = 20000
REQUEST_RETRIES = 6
SERVER_ENDPOINT = "tcp://10.0.1.151:40000"
from pico_hal import read_event_cnt

SERVER_ENDPOINT_DATA = "tcp://10.0.1.151:30000"


ctx = zmq.Context()

data_client = ctx.socket(zmq.SUB)
data_client.connect(SERVER_ENDPOINT_DATA)
data_client.setsockopt(zmq.SUBSCRIBE, b"")
client = ctx.socket(zmq.REQ)
client.connect(SERVER_ENDPOINT)
client.send_string("[Client] - connected")
resp = client.recv()
print (resp)
# let's create a tof packet for a command
tp  = gt.TofPacket()

event_cache = []
missing = []
time.sleep(10)
while True:
    # get a number of event ids and then work through 
    # the cache
    event_cache = missing
    if not event_cache:
        for n in range(10000):
            event_cache.append(read_event_cnt())
    # let the rb catch up
    # FIXME - this must be multithreaded
    time.sleep(5)
    for evid in event_cache:
        logging.info(f"Working on ev: {evid}")
        request = str(evid).encode()
        cmd = gt.CommandPacket(gt.TofCommand.RequestEvent, evid)
        #tp.set_packet_type(gt.Command)
        #tp.set_payload(cmd.to_bytestream())
        logging.info("Sending (%s)", request)
        client.send(bytes(cmd.to_bytestream()))
        #time.sleep(0.5)
        retries_left = REQUEST_RETRIES
        while True:
            if (client.poll(REQUEST_TIMEOUT) & zmq.POLLIN) != 0:
                reply = client.recv()
                if len(reply) > 5:
                    response = gt.ResponsePacket(gt.TofResponse.Unknown, 0);
                    response.from_bytestream([k for k in reply],0)
                    logging.info("Server replied OK (%s)", response)
                    if response.get_response() == gt.TofResponse.Success:
                        print (f"Yay, {evid}")
                        foo = data_client.recv()
                        foo = [ k for k in foo]
                        foo.extend([0,0,0])
                        print (foo[0:50])
                        print ("...")
                        print (foo[-10:])
                        event = gt.get_events_from_stream(foo,0)
                        print (event[0])
                        raise
                    if response.get_response() == gt.TofResponse.EventNotReady:
                        missing.append(evid)
                    retries_left = REQUEST_RETRIES
                    break
                else:
                    logging.error("Malformed reply from server: %s", reply)
                    continue

            retries_left -= 1
            logging.warning("No response from server")
            # Socket is confused. Close and remove it.
            client.setsockopt(zmq.LINGER, 0)
            client.close()
            if retries_left == 0:
                logging.error("Server seems to be offline, abandoning")
                sys.exit()

            logging.info("Reconnecting to server…")
            # Create new connection
            client = ctx.socket(zmq.REQ)
            client.connect(SERVER_ENDPOINT)
            logging.info("Resending (%s)", request)
            client.send(request)


#for n in range(200000):
#    if len(inwaiting) > 10000:
#        logging.warn(f"Cache too big, will get evid from cache!")
#        evid = inwaiting.pop()
#    else:
#        evid = read_event_cnt()
#    logging.info(f"Working on ev: {evid}")
#    request = str(evid).encode()
#    cmd = gt.CommandPacket(gt.TofCommand.RequestEvent, evid)
#    #tp.set_packet_type(gt.Command)
#    #tp.set_payload(cmd.to_bytestream())
#    logging.info("Sending (%s)", request)
#    client.send(bytes(cmd.to_bytestream()))
#    #time.sleep(0.5)
#    retries_left = REQUEST_RETRIES
#    while True:
#        if (client.poll(REQUEST_TIMEOUT) & zmq.POLLIN) != 0:
#            reply = client.recv()
#            if len(reply) > 5:
#                response = gt.ResponsePacket(gt.TofResponse.Unknown, 0);
#                response.from_bytestream([k for k in reply],0)
#                logging.info("Server replied OK (%s)", response)
#                if response.get_response() == gt.TofResponse.EventNotReady:
#                    inwaiting.append(evid)
#                retries_left = REQUEST_RETRIES
#                break
#            else:
#                logging.error("Malformed reply from server: %s", reply)
#                continue
#
#        retries_left -= 1
#        logging.warning("No response from server")
#        # Socket is confused. Close and remove it.
#        client.setsockopt(zmq.LINGER, 0)
#        client.close()
#        if retries_left == 0:
#            logging.error("Server seems to be offline, abandoning")
#            sys.exit()
#
#        logging.info("Reconnecting to server…")
#        # Create new connection
#        client = ctx.socket(zmq.REQ)
#        client.connect(SERVER_ENDPOINT)
#        logging.info("Resending (%s)", request)
#        client.send(request)

#sock.send_string("foo")
#while True:
#    now = time.time()
#    time.sleep(2)
#    print('..')
#    data = sock.recv()
#    print (data)
#    print (f"it took {time.time() - now} seconds till we got somethimg")
#    sock.send_string(str(evid))
