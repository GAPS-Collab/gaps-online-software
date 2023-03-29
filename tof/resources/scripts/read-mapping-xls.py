#! /usr/bin/env python

"""
Read a paddle mapping xls file from Sydney and change 
a tof manifest accordingly
"""
import json
import pprint
try :
    from python_arptable import get_arp_table
    def get_ip_for_mac(mac):
        arp = get_arp_table()
        for k in arp:
            if k['HW address'] == mac:
                return k['IP address']
    
    def get_mac_for_ip(ip):
        arp = get_arp_table()
        for k in arp:
            if k['IP address'] == ip:
                return k['HW address']
except ImportError:
    checks = False
from collections import OrderedDict

pp = pprint.PrettyPrinter(indent=2)

class LTB:
    """
    Representation of a local trigger board
    """

    def __init__(self):
        self.id  = 0
        self.DSI = 0
        self.J   = 0
        self.channels_to_rb = []

    def update(self, data):
        self.id  = int(data['id'])
        self.DSI = int(data['DSI'])
        self.J   = int(data['J'])
    
    def to_dict(self):
        data = OrderedDict()
        data['id']  = self.id
        data['DSI'] = self.DSI
        data['J']   = self.J
        data['ch_to_rb'] = dict()
        for ch in self.channels_to_rb:
            data['ch_to_rb'][str(ch[0])] = [int(ch[1]), int(ch[2])]
        return data

    def to_json(self):
        #return hjson.dumps(self.to_dict(), use_decimal=False)
        return json.dumps(self.to_dict())


class RB:
    """
    Representation of a ReadoutBoard
    """

    def __init__(self):
        self.id        = 0
        self.ch_to_pid = dict()
        self.dna       = 0
        self.port      = 42000
        self.calibration_file = ""
        self.mac_address = ""
        self.ip_address = ""
   
    def update(self, data):
        self.id   = int(data['id'])
        self.dna  = int(data['dna'])
        self.port = int(data['port'])
        self.calibration_file = data['calibration_file']
        self.mac_address = data['mac_address']
        self.guess_ip()

    def guess_ip(self):
        self.ip_address = "10.0.1.1" + str(self.id).zfill(2)
        return self.ip_address

    def to_dict(self):
        data = OrderedDict()
        data['id']   = self.id
        data['dna']  = self.dna
        data['port'] = self.port
        data['calibration_file'] = self.calibration_file
        data['mac_address'] = self.mac_address
        data['ip_address']  = self.ip_address
        data['ch_to_pid'] = self.ch_to_pid
        return data

    def to_json(self):
        #return hjson.dumps(self.to_dict(), use_decimal=False)
        return json.dumps(self.to_dict())

class PaddleEnd:
    
    def __init__(self, row):
        """

        Args:
            row (pandas.Series) : A row in the spreadsheet
        """
        self.id     = row['Paddle Number']
        self.end    = row['Paddle End (A/B)']
        self.panel  = row['Panel Number']
        self.cable  = row['Cable length (cm)']
        self.rat    = row['RAT Number']
        self.ltb_ch = row['LTB Number-Channel']
        self.rb_ch  = row['RB Number-Channel']
        self.pb_ch  = row['PB Number-Channel']


    def get_ltb_ch(self):
        ltb = int(self.ltb_ch.split('-')[0])
        ch  = int(self.ltb_ch.split('-')[1])
        return [ltb, ch]

    def get_rb_ch(self):
        rb = int(self.rb_ch.split('-')[0])
        ch = int(self.rb_ch.split('-')[1])
        return [rb, ch]

    def get_pb_ch(self):
        pb = int(self.pb_ch.split('-')[0])
        ch = int(self.pb_ch.split('-')[1])
        return [pb, ch]

    def __repr__(self):
        str_repr =  f'<PaddleEnd : \tPaddle Number {self.id}\n '
        str_repr += f'\t\tPaddle End (A/B) {self.end}\n '
        str_repr += f'\t\tPanel Number {self.panel}\n '
        str_repr += f'\t\tCable length (cm) {self.cable}\n '
        str_repr += f'\t\tRAT Number {self.rat}\n '
        str_repr += f'\t\tLTB Number-Channel {self.ltb_ch}\n ' 
        str_repr += f'\t\tRB Number-Channel {self.rb_ch}\n '
        str_repr += f'\t\tPB Number-Channel {self.pb_ch}>'
        return str_repr
    #Index(['Paddle Number', 'Paddle End (A/B)', 'Panel Number', 'Panel Center',
    #       'Paddle Location in Panel ', 'Cable length (cm)', 'RAT Number',
    #              'LTB Number-Channel', 'LTB Harting Connection', 'RB Number-Channel',
    #                     'RB Harting Connection', 'PB Number-Channel'],
    

if __name__ == '__main__':
    
    import pandas
   
    import argparse

    parser = argparse.ArgumentParser(description="Convert a paddle mapping xls spreadsheet into machine-readable JSON format")
    parser.add_argument('input', metavar='input', type=str,\
                        help='Input XLS spreadsheet')

    #mapping = pandas.read_excel('GAPS_Channel_mapping.xlsx', sheet_name="Paddle Ends")
    parser.add_argument('--update-existing', '-u', dest='update_existing',\
                           default='', type=str,\
                           help='update an existing tof manifest file by replacing RB and LTB information based on the information in the spreadsheet.')
    args = parser.parse_args()

    if args.update_existing:
        print (f'Will read {args.update_existing}')
        tof_manifest = json.load(open(args.update_existing))
        print (tof_manifest)
        print (type(tof_manifest))
    mapping = pandas.read_excel(args.input, sheet_name="Paddle End Master Spreadsheet")
    rbs  = dict()
    ltbs = dict()
    for k in range(1,len(mapping.index)):
        paddle_end = PaddleEnd(mapping.loc[k,:])
        print (paddle_end)
        print (paddle_end.get_ltb_ch())
        print (paddle_end.get_rb_ch())
        print (paddle_end.get_pb_ch())
        print ("=============")
        ltb = paddle_end.get_ltb_ch()
        rb  = paddle_end.get_rb_ch()
        # first RB, then LTB
        
        if rb[0] in rbs:
            rbs[rb[0]].ch_to_pid[str(rb[1])] =  paddle_end.id
        else:
            new_rb = RB()
            new_rb.id = rb[0]
            new_rb.ch_to_pid[str(rb[1])] = paddle_end.id
            rbs[rb[0]] = new_rb
            print (new_rb.guess_ip())
        
        if ltb[0] in ltbs:
            ltbs[ltb[0]].channels_to_rb.append([ltb[1],rb[0],rb[1]])
            #ltbs[ltb[0]] 
        else:
            new_ltb    = LTB()
            new_ltb.id = ltb[0]
            new_ltb.channels_to_rb.append([ltb[1],rb[0],rb[1]])
            ltbs[ltb[0]] = new_ltb
    print(f'We found {len(rbs.values())} ReadoutBoards')
    print(f'We found {len(ltbs.values())} LocalTriggerBoards')
    print ([k.to_json() for k in ltbs.values()][0])
    print ([k.to_json() for k in rbs.values()][0])
    

    if args.update_existing:
        for rb in tof_manifest['rbs']:
            try:
                rbs[rb['id']].update(rb)
            except KeyError:
                print (f'RB {rb["id"]} not found, adding this board...')
                rbs[rb['id']] = rb
            # checks
            checks = False
            mac = get_mac_for_ip(rb['ip_address'])
            rb['mac_address'] = mac
            if checks:
              mac = rb['mac_address']
              ip  = rb['ip_address']
              if not (get_ip_for_mac(mac) == ip) and (get_mac_for_ip(ip)):
                  print (rbs[rb['id']].to_json())
                  choice = input("We found an ip/mac mismatch! What do you want to do? 1) mac is correct, change ip, 2) ip is correct, change mac 3) abort")
                  if choice == "1":
                      rbs[rb['id']]['ip_address'] = ip
                      print (f'Have set ip to {ip}')
                  elif choice == "2":
                      rbs[rb['id']]['mac_address'] = mac
                      print (f'Have set mac to {mac}')
                  else:
                      print ('..skipping..')
            def to_dict(obj):
                if isinstance(obj, dict):
                    return obj
                else:
                    return obj.to_dict()
                    
            tof_manifest['rbs'] = [to_dict(k) for k in rbs.values()]

        for ltb in tof_manifest['ltbs']:
            ltbs[ltb['id']].update(ltb)
        tof_manifest['ltbs'] = [k.to_dict() for k in ltbs.values()]
        pp.pprint (tof_manifest)
        json.dump(tof_manifest,open('tof-manifest.updated.json','w'), indent=2)
