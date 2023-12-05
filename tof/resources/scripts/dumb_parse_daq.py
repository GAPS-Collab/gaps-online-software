#!/usr/bin/env python3
from enum import Enum, auto
import libscrc

def read_test_packet():
    dna_length = 4
    event_cnt_length = 2
    ch_crc_length = 2
    crc32_length = 2
    timestamp_length = 3
    dna = 0
    ch_crc = 0
    ch_crc_calc = 0
    packet_crc_calc = 0
    crc32 = 0
    ch_cnt = 0
    event_cnt = 0
    timestamp = 0
    state_word_cnt = 0
    packet_length = 0
    roi_size = 0
    num_channels = 0

    class State(Enum):
        IDLE = auto()
        ERR = auto()
        HEAD = auto()
        STATUS = auto()
        LENGTH = auto()
        ROI = auto()
        DNA = auto()
        RSVD0 = auto()
        RSVD1 = auto()
        RSVD2 = auto()
        HASH = auto()
        ID = auto()
        CHMASK = auto()
        EVENT_CNT = auto()
        DTAP = auto()
        TEMPERATURE = auto()
        TIMESTAMP = auto()
        CALC_CH_CRC = auto()
        CH_CRC = auto()
        CH_HEADER = auto()
        PAYLOAD = auto()
        STOPCELL = auto()
        CRC32 = auto()
        TAIL = auto()

    with open("daq_packet.txt", "r") as daq_file:
        lines = daq_file.readlines()

    state = State.HEAD

    for line in lines:

        data = int(line, 16)

        if state not in (State.CRC32, State.TAIL):
            packet_crc_calc = libscrc.crc32((data).to_bytes(2, byteorder='little'), packet_crc_calc)
            #print("      > data = 0x%04X, Calculate crc = 0x%08X" % (data,packet_crc_calc))

        if state == State.HEAD:
            print("HEAD      : 0x%X" % data)
            state = State.STATUS
            continue

        if state == State.STATUS:
            print("STATUS    : 0x%X" % data)
            state = State.LENGTH
            continue

        if state == State.LENGTH:
            print("LENGTH    : 0x%X" % data)
            state = State.ROI
            packet_length = data
            continue

        if state == State.ROI:
            print("ROI       : 0x%X (payload size = %d)" % (data, data+1))
            state = State.DNA
            roi_size = data+1
            continue

        if state == State.DNA:
            dna = data
            state = State.RSVD0
            continue

        if state == State.RSVD0:
            state = State.RSVD1
            continue

        if state == State.RSVD1:
            state = State.RSVD2
            continue

        if state == State.RSVD2:
            state = State.HASH
            continue

        if state == State.HASH:
            print("HASH      : 0x%X" % data)
            state = State.ID
            hash = data
            continue

        if state == State.ID:
            print("ID        : 0x%X" % data)
            state = State.CHMASK
            continue

        if state == State.CHMASK:
            for i in range(8):
                if 0x1&(data>>i):
                    num_channels += 1
            if num_channels > 0:
                num_channels += 1

            state = State.EVENT_CNT
            print("CHMASK    : 0x%X (%d channels)" % (data, num_channels))
            continue

        if state == State.EVENT_CNT:
            if state_word_cnt == 0:
                event_cnt = 0
            event_cnt |= (data << (16*(event_cnt_length-state_word_cnt-1)))
            state_word_cnt += 1

            if state_word_cnt == event_cnt_length:
                print("EVENT_CNT : 0x%0*X" % (8, event_cnt))
                state = State.DTAP
                state_word_cnt = 0
                continue

        if state == State.DTAP:
            print("DTAP      : 0x%X" % data)
            state = State.TEMPERATURE
            dtap = data
            continue

        if state == State.TEMPERATURE:
            print("TEMPERATUR: 0x%X" % data)
            state = State.TIMESTAMP
            continue

        if state == State.TIMESTAMP:
            if state_word_cnt == 0:
                timestamp = 0
            timestamp |= (data << (16*(timestamp_length-state_word_cnt-1)))
            state_word_cnt += 1

            if state_word_cnt == timestamp_length:
                print("TIMESTAMP : 0x%0*X" % (12, timestamp))
                state = State.CH_HEADER
                state_word_cnt = 0
                continue

        if state == State.CH_HEADER:
            print("CH_HEAD   : 0x%X" % data)
            state = State.PAYLOAD
            continue

        if state == State.PAYLOAD:

            #print("Ch%d %04X!=%04X" % (ch_cnt+1, data, state_word_cnt))
            # if data != state_word_cnt:
            #     print("Ch%d ERROR %04X!=%04X" % (ch_cnt+1, data, state_word_cnt))
            #     return
            #print("%04x" % data)

            state_word_cnt += 1
            ch_crc_calc = libscrc.crc32((data).to_bytes(2, byteorder='little'), ch_crc_calc)

            if state_word_cnt >= (roi_size):
                state = State.CH_CRC
                state_word_cnt = 0
                continue

        if state == State.CH_CRC:
            if state_word_cnt == 0:
                ch_crc = 0
            ch_crc |= (data << (16*(ch_crc_length-state_word_cnt-1)))
            if state_word_cnt == ch_crc_length-1:
                if ch_crc_calc == ch_crc:
                    print("    > Ch%d Payload OK CRC=0x%08X" % ((ch_cnt+1), ch_crc))
                else:
                    print("    > Ch%d Payload CRC Fail" % (ch_cnt+1))
                    print("      > Calculate crc = 0x%08X" % ch_crc_calc)
                    print("      > Received  crc = 0x%0*X" % (8, ch_crc))
                if ch_cnt == num_channels - 1:
                    state = State.STOPCELL
                    ch_cnt = 0
                else:
                    state = State.CH_HEADER
                    ch_cnt += 1

                ch_crc_calc = 0
                state_word_cnt = 0
                continue
            state_word_cnt += 1

        if state == State.STOPCELL:
            print("STOPCELL  : 0x%X" % data)
            state = State.CRC32
            continue

        if state == State.CRC32:
            if state_word_cnt == 0:
                crc32 = 0
            crc32 |= (data << (16*(crc32_length-state_word_cnt-1)))
            state_word_cnt += 1

            if state_word_cnt == crc32_length:
                print("CRC32     : 0x%0*X" % (8, crc32))
                if crc32 == packet_crc_calc:
                    print("    > Packet CRC OK")
                else:
                    print("    > Packet CRC FAIL expect=0x%08X" % packet_crc_calc)
                state = State.TAIL
                state_word_cnt = 0
                continue

        if state == State.TAIL:
            print("TAIL      : 0x%X" % data)
            state = State.IDLE
            continue

read_test_packet()

