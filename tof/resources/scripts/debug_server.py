#! /usr/bin/env python3

import time
import zmq
import sys
import logging

import gaps_tof as gt

logging.basicConfig(format="%(levelname)s: %(message)s", level=logging.INFO)

REQUEST_TIMEOUT = 10000
REQUEST_RETRIES = 6
SERVER_ENDPOINT = "tcp://10.0.1.151:38830"
from pico_hal import read_event_cnt

ctx = zmq.Context()
client = ctx.socket(zmq.REQ)
client.connect(SERVER_ENDPOINT)
client.send_string("foo")
client.recv()
# let's create a tof packet for a command
tp  = gt.TofPacket()
for n in range(10000):
    evid = read_event_cnt()
    print (f"Got {evid}")
    request = str(evid).encode()
    cmd = gt.CommandPacket(gt.TofCommand.RequestEvent, evid)
    tp.set_packet_type(gt.Command)
    tp.set_payload(cmd.to_bytestream())
    logging.info("Sending (%s)", request)
    client.send(bytes(tp.to_bytestream()))
    time.sleep(0.5)
    retries_left = REQUEST_RETRIES
    while True:
        if (client.poll(REQUEST_TIMEOUT) & zmq.POLLIN) != 0:
            reply = client.recv()
            if len(reply) > 5:
                logging.info("Server replied OK (%s)", reply)
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

#sock.send_string("foo")
#while True:
#    now = time.time()
#    time.sleep(2)
#    print('..')
#    data = sock.recv()
#    print (data)
#    print (f"it took {time.time() - now} seconds till we got somethimg")
#    sock.send_string(str(evid))
