#!/usr/bin/env python3

import socket
import sys
import random
import select
import time
import json
import math

from enum import Enum

regs = None

PACKET_ID = 0
#IPADDR = "192.168.36.121"
IPADDR = "10.0.1.10"
PORT = 50001

# Create a UDP socket and bind the socket to the port
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
target_address = ("", 0)

READ = 0
WRITE = 1
READ_NON_INCR = 2
WRITE_NON_INCR = 3
RMW = 4

typedef = {
    READ: "READ",
    WRITE: "WRITE",
    READ_NON_INCR: "READ (non incremental)",
    WRITE_NON_INCR: "WRITE (non incremental)",
    RMW: "Read Modify Write",
}

def get_lsb(n):
    return int(math.log2(n & -n))

# Recieve
def decode_ipbus(message, verbose=False):

    # Response
    ipbus_version = message[0] >> 4
    id = ((message[4] & 0xf) << 8) | message[5]
    size = message[6]
    type = (message[7] & 0xf0) >> 4
    info_code = message[7] & 0xf
    data = [None]*size

    # Read
    if type==READ or type==READ_NON_INCR:
        for i in range(size):
            data[i]=((message[8 + i * 4] << 24) | (message[9 + i * 4] << 16) | (message[10 + i * 4] << 8) | message[11 + i * 4])

    # Write
    if type==WRITE:
        data = [0]

    if (verbose):
        print("Decoding IPBus Packet:")
        print(f" > Msg = {message}")
        print(f" > IPBus version = {ipbus_version}")
        print(f" > ID = {id}")
        print(f" > Size = {size}")
        print(f" > Type = {type} %s" % typedef[type])
        print(f" > Info = {info_code}")
        print(f" > data = {data}")

    return (data)

def encode_ipbus(addr, packet_type, data):

    size = len(data)

    """Format a UDP packet from a byte array"""
    udp_data = [
        # Transaction Header
        0x20, # Protocol version & RSVD
        0x00, # Transaction ID (0 or bug)
        0x00, # Transaction ID (0 or bug)
        0xf0, # Packet order & packet_type
        # Packet Header
        (0x20 | ((PACKET_ID & 0xf00) >> 8)), # Protocol version & Packet ID MSB
        (PACKET_ID & 0xff), # Packet ID LSB,
        size, # Words
        (((packet_type & 0xf) << 4) | 0xf), # Packet_Type & Info code
        # Address
        ((addr & 0xff000000) >> 24),
        ((addr & 0x00ff0000) >> 16),
        ((addr & 0x0000ff00) >> 8),
        (addr & 0x000000ff),]

    if packet_type == WRITE or packet_type == WRITE_NON_INCR:
        for i in range(size):
            udp_data.append((data[i] & 0xff000000) >> 24)
            udp_data.append((data[i] & 0x00ff0000) >> 16)
            udp_data.append((data[i] & 0x0000ff00) >> 8)
            udp_data.append(data[i] & 0x000000ff)

    return bytes(udp_data)

def wReg(reg, data, verify=False):
    node = regs[reg]
    adr = node["adr"]
    mask = node["mask"]
    shift = get_lsb(mask)
    r = (0xffffffff ^ mask) & rAdr(adr)
    r |= data << shift
    wAdr(adr, r)

def rReg(reg, verify=False):
    node = regs[reg]
    adr = node["adr"]
    mask = node["mask"]
    shift = get_lsb(mask)
    return (rAdr(adr) & mask) >> shift

def wAdr(address, data, verify=False):
    s.sendto(encode_ipbus(addr=address, packet_type=WRITE, data=[data]), target_address)
    ready = select.select([s], [], [], 1)
    if ready[0]:
        data = s.recvfrom(4096)
        if verify:
            rdback = rAdr(address)
            if rdback != data:
                print("Readback error in wAdr!")
            return rdback
        return data
    else:
        print("timeout in wreg 0x%08X" % data)
        return wAdr(address, data, verify)

def rAdr(address):
    s.sendto(encode_ipbus(addr=address, packet_type=READ, data=[0x0]), target_address)
    ready = select.select([s], [], [], 1)
    if ready[0]:
        data, adr = s.recvfrom(4096)
        dec = decode_ipbus(data,False)
        if (len(dec) > 0):
            return dec[0]
        else:
            return rAdr(address)
    else:
        print("timeout in rreg")
        return rAdr(address)

c_addr = 0x1004
div_addr = 0x1005
ss_addr = 0x1006
d_addr = (0x1000, 0x1001, 0x1002, 0x1003)

def wSpiSS(ss):
    wAdr(ss_addr, ss)

def wSpiWord(word):
    wAdr(d_addr[0], word, verify=False)

def wSpiCtrl(char_len=None, go=None, rx_neg=None, tx_neg=None, lsb=None, ie=None, ass=None):

    reg = rAdr(c_addr)

    if (char_len is not None):
        reg &= (0xffffffff ^ 0x7f)
        reg |= (0x7f & char_len)
    if (go is not None):
        reg &= (0xffffffff ^ 0x100)
        reg |= (0x1 & go) << 8
    if (rx_neg is not None):
        reg &= (0xffffffff ^ 0x200)
        reg |= (0x1 & rx_neg) << 9
    if (tx_neg is not None):
        reg &= (0xffffffff ^ 0x400)
        reg |= (0x1 & tx_neg) << 10
    if (lsb is not None):
        reg &= (0xffffffff ^ 0x800)
        reg |= (0x1 & lsb) << 11
    if (ie is not None):
        reg &= (0xffffffff ^ 0x1000)
        reg |= (0x1 & ie) << 12
    if (ass is not None):
        reg &= (0xffffffff ^ 0x2000)
        reg |= (0x1 & ass) << 13

    wAdr(c_addr, reg, verify=False)

def rSpi(channel=0, adc=0):
    wAdr(div_addr, 127, verify=False)
    # 0x18 start bit + for single ended
    wSpiWord((0x18 | (0x7 & channel)) << 16)
    wSpiCtrl(go=0)
    wSpiCtrl(ass=1, tx_neg=1, char_len=24)
    wSpiCtrl(go=1)
    wSpiCtrl(go=0)
    wSpiSS(0x1 << adc)
    return rAdr(d_addr[0])

def enable_tiu_emulation_mode():
    wReg("MT.TIU_EMULATION_MODE", 1)

def disable_tiu_emulation_mode():
    wReg("MT.TIU_EMULATION_MODE", 0)

def read_adc(adc, ch, shift=2):
    data = rSpi(ch, adc)
    data = (data >> shift) & 0xfff
    return data

def reset_event_cnt():
    wReg("MT.EVENT_CNT_RESET", 1, verify=False)

def read_event_cnt(output=False):
    cnt = rReg("MT.EVENT_CNT")
    if output:
        print("Event counter = %d" % cnt)

    return cnt

def reset_hit_cnt(output=False):
    wReg("MT.HIT_COUNTERS.RESET", 1)

def read_hit_cnt():
    wReg("MT.HIT_COUNTERS.SNAP", 1) # snap
    base_address = regs["MT.HIT_COUNTERS.LT0"]["adr"]
    for i in range (20):
        print("LTB%2d hit counts = %d" % (i, rAdr(base_address+i)))
    wReg("MT.HIT_COUNTERS.SNAP", 0) # unsnap

def read_adcs():

    from tabulate import tabulate

    channels = [
        {"function": "NC"           , "conversion": 1,   "unit": "V"},
        {"function": "DSI1 Current" , "conversion": 1,   "unit": "A"},
        {"function": "DSI2 Current" , "conversion": 1,   "unit": "A"},
        {"function": "DSI3 Current" , "conversion": 1,   "unit": "A"},
        {"function": "DSI4 Current" , "conversion": 1,   "unit": "A"},
        {"function": "DSI5 Current" , "conversion": 1,   "unit": "A"},
        {"function": "NC"           , "conversion": 1,   "unit": "V"},
        {"function": "CCB Current"  , "conversion": 1,   "unit": "A"},
        {"function": "12V Voltage"  , "conversion": 0.1, "unit": "V"},
        {"function": "3.3V Voltage" , "conversion": 0.5, "unit": "V"},
        {"function": "2.5V Voltage" , "conversion": 0.5, "unit": "V"},
        {"function": "Misc 0"       , "conversion": 1,   "unit": "V"},
        {"function": "Misc 1"       , "conversion": 1,   "unit": "V"},
        {"function": "Misc 2"       , "conversion": 1,   "unit": "V"},
        {"function": "Misc 3"       , "conversion": 1,   "unit": "V"},
        {"function": "NC"           , "conversion": 1,   "unit": "V"},]

    headers = ["Ch", "Reading", "Value", "Function"]
    table = []

    for adc in range(2):
        for channel in range(8):
            ichn = adc*8 + channel
            data = sum(read_adc(adc, channel) for _ in range(5)) / 5.0
            value = 2.5 * data / (2**12-1)

            table.append([ichn,
                        "0x%03X" % int(data),
                        "%4.2f %s" % (value / channels[ichn]["conversion"], channels[ichn]["unit"]),
                        channels[ichn]["function"],
                        ])

    channels = [
        {"function": "FPGA TEMP"    , "conversion": 1,   "unit": "C", "reg": "MT.XADC.TEMP"},
        {"function": "FPGA VCCINT"  , "conversion": 3,   "unit": "V", "reg": "MT.XADC.VCCINT"},
        {"function": "FPGA VCCAUX"  , "conversion": 3,   "unit": "V", "reg": "MT.XADC.VCCAUX"},
        {"function": "FPGA VCCBRAM" , "conversion": 3,   "unit": "V", "reg": "MT.XADC.VCCBRAM"}]

    for (ichn,channel) in enumerate(channels):

        data = rReg(channel["reg"])

        if (channel["function"] == "FPGA TEMP"):
            value = data * 503.975 / 4096 - 273.15
        else:
            value = channel["conversion"] * data / (2**12-1)

        table.append([ichn,
                    "0x%03X" % int(data),
                    "%4.2f %s" % (value, channels[ichn]["unit"]),
                    channels[ichn]["function"]])

    print(tabulate(table, headers=headers,  tablefmt="simple_outline"))

    return([headers]+table)

def check_clocks():

    def check_clock(freq, desc, spec):
        if (spec < freq*1.01 and spec > freq*0.99):
            stat = "OK"
        else:
            stat = "BAD"
        print ("%19s: % 10d Hz (%s)" % (desc, freq, stat))

    for reg, spec in (["MT.CLOCK_RATE",      100000000],
                      ["MT.FB_CLOCK_RATE_0", 20000000],
                      ["MT.FB_CLOCK_RATE_1", 20000000],
                      ["MT.FB_CLOCK_RATE_2", 20000000],
                      ["MT.FB_CLOCK_RATE_3", 20000000],
                      ["MT.FB_CLOCK_RATE_4", 20000000]):
        check_clock(rReg(reg), reg, spec)

def force_trigger():
    wReg("MT.FORCE_TRIGGER", 1)

def en_ucla_trigger():
    set_trig("MT.TRIG_MASK_A", 0x000000f0)
    set_trig("MT.TRIG_MASK_B", 0x0000000f)

def en_ssl_trigger():
    set_trig("MT.TRIG_MASK_A", 0xfc3f0000)
    set_trig("MT.TRIG_MASK_B", 0x0000fc3f)

def set_any_trigger(val):
    wReg("MT.ANY_TRIG_EN", val)
    rd = rReg("MT.ANY_TRIG_EN")
    print("Any trigger mode set to %d" % rd)

def set_ssl_trig(trg, val):
    wReg("MT.SSL_TRIG_%s_EN" % trg, val)

def trig_stop():
    set_trig("MT.TRIG_MASK_A", 0x00000000)
    set_trig("MT.TRIG_MASK_B", 0x00000000)

def set_trig(reg, val):

    if not (reg == "MT.TRIG_MASK_A" or reg == "MT.TRIG_MASK_B"):
        raise Exception("invalid Trigger mask register!")

    if (isinstance(val, str)):
        val = int(val, 16)

    wReg(reg, val)

def set_trig_generate(val):
    wReg("MT.TRIG_GEN_RATE", val)

def set_trig_hz(rate):
    # rate = f_trig / 1E8 * 0xffffffff
    set_trig_generate(int((rate*0xffffffff)/1E8))

def read_ltb_link_status():
    for dsi in range(5):
        reg = f"MT.LT_LINK_READY{dsi}"
        ok_mask = rReg(reg)
        for ltb in range(5):
            link0  = 1 ^ (0x1 & (ok_mask >> (ltb*2 + 0)))
            link1  = 1 ^ (0x1 & (ok_mask >> (ltb*2 + 1)))
            status = "(BAD)" if link0 or link1 else "(maybe ok)"
            print(f"DSI {dsi} LTB {ltb} {link0=} {link1=} {status}")

def read_rates():
    rate = rAdr(0x17)
    lost = rAdr(0x18)
    print("Trigger rate      = %d Hz" % rate)
    print("Lost trigger rate = %d Hz" % lost)
    return rate,lost

def read_daq():

    def read_daq_word():
        while (True):
            if 0 == (rReg("MT.EVENT_QUEUE.EMPTY")):
                return rReg("MT.EVENT_QUEUE.DATA")

    def count_ones(n):
        count = 0
        while (n):
            count += n & 1
            n >>= 1
        return count

    state = "Idle"
    paddles_rxd = 0
    hit_paddles = -1

    wReg("MT.EVENT_QUEUE.RESET", 1)

    while (True):
        rd = read_daq_word()

        if (state=="Idle" and rd==0xAAAAAAAA):
            state = "Header"
            hit_paddles = 0
            paddles_rxd = 1

        if (state=="Mask"):
            hit_paddles = count_ones(rd)

        if (state=="Hits"):
            paddles_rxd += 1

        print("%08X (%s)" % (rd, state))

        if (state=="Header"):
            state="Event cnt"
        elif (state=="Event cnt"):
            state="Timestamp"
        elif (state=="Timestamp"):
            state="TIU Timestamp"
        elif (state=="TIU Timestamp"):
            state="GPS 32 bits"
        elif (state=="GPS 32 bits"):
            state="GPS 16 bits"
        elif (state=="GPS 16 bits"):
            state="Mask"
        elif (state=="Mask"):
            state="Hits"
        elif (state=="Hits" and paddles_rxd >= hit_paddles):
            state="CRC"
        elif (state=="CRC"):
            state="Trailer"
        elif (state=="Trailer"):
            state="Idle"


def loopback(nreads=100000):
    print(" > Running loopback test")
    from tqdm import tqdm
    for i in tqdm(range(nreads), colour='green'):
        write = random.randint(0, 0xffffffff)
        wAdr(0,write)
        read = rAdr(0)
        assert write==read, print("wr=0x%08X  rd=0x%08X" % (write, read))
        # if (i % 100 == 0):
        #     print(f"{i} reads, %f Mb" % ((i*32.0)/1000000.0))

def fw_info():

    fwdate = rReg("MT.HOG.GLOBAL_DATE")
    fwtime = rReg("MT.HOG.GLOBAL_TIME")
    fwver  = rReg("MT.HOG.GLOBAL_VER")
    fwsha  = rReg("MT.HOG.GLOBAL_SHA")

    if fwver == 0:
        fwver = "Local build."
    else:
        fwver  = "v%d.%d.%d" % (int("%04X" % ((fwver >> 24) & 0xff)),
                                int("%04X" % ((fwver >> 16) & 0xff)),
                                int("%04X" % ((fwver >>  0) & 0xffff)))

    fwdate = "%04x/%02x/%02x" % (fwdate & 0xffff, (fwdate>>16) & 0xff, (fwdate>>24) & 0xff)
    fwtime = "%02x:%02x:%02x" % ((fwtime >> 16) & 0xff, (fwtime>>8) & 0xff, (fwtime>>0) & 0xff)

    print("FW_DATE = %s" % fwdate)
    print("FW_TIME = %s" % fwtime)
    print("FW_VER  = %s" % fwver)
    print("FW_SHA  = %7X" % fwsha)

if __name__ == '__main__':

    import argparse

    argParser = argparse.ArgumentParser(description = "Argument parser")

    argParser.add_argument('--ip',                    action='store',      default=False, help="Set the IP Address of the target MTB.")
    argParser.add_argument('--status',                action='store_true', default=False, help="Print out hte status of the TMB.")
    argParser.add_argument('--ucla_trig_en',          action='store_true', default=False, help="Enable UCLA trigger")
    argParser.add_argument('--ssl_trig_en',           action='store_true', default=False, help="Enable SSL trigger")
    argParser.add_argument('--any_trig_en',           action='store_true', default=False, help="Enable ANY trigger")
    argParser.add_argument('--any_trig_dis',          action='store_true', default=False, help="Disable ANY trigger")
    argParser.add_argument('--ssl_top_bot_en',        action='store_true', default=False, help="Enable SSL trigger")
    argParser.add_argument('--ssl_top_bot_dis',       action='store_true', default=False, help="Disable SSL trigger")
    argParser.add_argument('--ssl_topedge_bot_en',    action='store_true', default=False, help="Enable SSL trigger")
    argParser.add_argument('--ssl_topedge_bot_dis',   action='store_true', default=False, help="Disable SSL trigger")
    argParser.add_argument('--ssl_top_botedge_en',    action='store_true', default=False, help="Enable SSL trigger")
    argParser.add_argument('--ssl_top_botedge_dis',   action='store_true', default=False, help="Disable SSL trigger")
    argParser.add_argument('--ssl_topmid_botmid_en',  action='store_true', default=False, help="Enable SSL trigger")
    argParser.add_argument('--ssl_topmid_botmid_dis', action='store_true', default=False, help="Disable SSL trigger")
    argParser.add_argument('--trig_rates',            action='store_true', default=False, help="Read the trigger rates")
    argParser.add_argument('--ltb_status',            action='store_true', default=False, help="Read LTB link status")
    argParser.add_argument('--trig_stop',             action='store_true', default=False, help="Stop all triggers.")
    argParser.add_argument('--trig_a',                action='store',                     help="Set trigger mask A")
    argParser.add_argument('--trig_b',                action='store',                     help="Set trigger mask B")
    argParser.add_argument('--trig_set_hz',           action='store',                     help="Set the poisson trigger generator rate in Hz")
    argParser.add_argument('--trig_generate',         action='store',                     help="Set the poisson trigger generator rate (f_trig = 1E8 * rate / 0xffffffff)")
    argParser.add_argument('--read_adc',              action='store_true', default=False, help="Read ADCs")
    argParser.add_argument('--loopback',              action='store_true', default=False, help="Ethernet Loopback Test")
    argParser.add_argument('--fw_info',               action='store_true', default=False, help="Print firmware version info")
    argParser.add_argument('--reset_event_cnt',       action='store_true', default=False, help="Reset the Event Counter")
    argParser.add_argument('--read_event_cnt',        action='store_true', default=False, help="Read the Event Counter")
    argParser.add_argument('--read_hit_cnt',          action='store_true', default=False, help="Read the LTB Hit Counters")
    argParser.add_argument('--reset_hit_cnt',         action='store_true', default=False, help="Reset the LTB Hit Counters")
    argParser.add_argument('--read_daq',              action='store_true', default=False, help="Stream the DAQ data to the screen")
    argParser.add_argument('--force_trig',            action='store_true', default=False, help="Force an MTB Trigger")
    argParser.add_argument('--check_clocks',          action='store_true', default=False, help="Check DSI loopback clock frequencies")
    argParser.add_argument('--enable_tiu_emulation_mode',  action='store_true', default=False, help="Write 1 to TIU_EMULATION_MODE register")
    argParser.add_argument('--disable_tiu_emulation_mode',  action='store_true', default=False, help="Write 0 to TIU_EMULATION_MODE register")

    args = argParser.parse_args()

    if args.ip:
        IPADDR = args.ip

    target_address = (IPADDR, PORT)

    with open('mt_addresses.json', 'r') as f:
        regs = json.load(f)

    if args.enable_tiu_emulation_mode:
        enable_tiu_emulation_mode()
    if args.disable_tiu_emulation_mode:
        disable_tiu_emulation_mode()
    if args.check_clocks:
        check_clocks()
    if args.ucla_trig_en:
        en_ucla_trigger()
    if args.trig_rates:
        read_rates()
    if args.ltb_status:
        read_ltb_link_status()
    if args.force_trig:
        force_trigger()
    if args.ssl_trig_en:
        en_ssl_trigger()
    if args.status:
        fw_info()
        print("")
        check_clocks()
        print("")
        read_rates()
        print("")
        read_adcs()
        print("")
        read_event_cnt(output=True)
        print("")
        read_hit_cnt()
        print("")
        read_ltb_link_status()

    if args.any_trig_en:
        set_any_trigger(1)
    if args.any_trig_dis:
        set_any_trigger(0)

    if args.ssl_top_bot_en:
        set_ssl_trig("TOP_BOT", 1)
    if args.ssl_top_bot_dis:
        set_ssl_trig("TOP_BOT", 0)
    if args.ssl_topedge_bot_en:
        set_ssl_trig("TOPEDGE_BOT", 1)
    if args.ssl_topedge_bot_dis:
        set_ssl_trig("TOPEDGE_BOT", 0)
    if args.ssl_top_botedge_en:
        set_ssl_trig("TOP_BOTEDGE", 1)
    if args.ssl_top_botedge_dis:
        set_ssl_trig("TOP_BOTEDGE", 0)
    if args.ssl_topmid_botmid_en:
        set_ssl_trig("TOPMID_BOTMID", 1)
    if args.ssl_topmid_botmid_dis:
        set_ssl_trig("TOPMID_BOTMID", 0)

    if args.trig_stop:
        trig_stop()
    if args.read_adc:
        read_adcs()
    if args.trig_generate:
        set_trig_generate(int(args.trig_generate))
    if args.trig_set_hz:
        set_trig_hz(int(args.trig_set_hz))
    if args.trig_a:
        set_trig("MT.TRIG_MASK_A", args.trig_a)
    if args.trig_b:
        set_trig("MT.TRIG_MASK_B", args.trig_b)
    if args.reset_event_cnt:
        reset_event_cnt()
    if args.read_event_cnt:
        read_event_cnt(output=True)
    if args.read_hit_cnt:
        read_hit_cnt()
    if args.reset_hit_cnt:
        reset_hit_cnt()
    if args.fw_info:
        fw_info()
    if args.loopback:
        loopback()
    if args.read_daq:
        read_daq()

    if len(sys.argv) == 1:
        argParser.print_help()

    # print(regs)
