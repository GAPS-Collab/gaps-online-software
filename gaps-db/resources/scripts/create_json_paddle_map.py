#! /usr/bin/env python

import django
django.setup()

import json
import tof_db.models as m

RB_IGNORELIST = [10,12,37,38,43,45,47,48,49,50,51]

if __name__ == '__main__':

    import argparse

    parser = argparse.ArgumentParser(description="(Re)create tables in the global GAPS database from paddle mapping spreadsheets")
    args = parser.parse_args()

    rbs = m.RB.objects.all()
    mapping = dict()
    for rb in rbs:
        mapping[rb.rb_id] = dict()
        for ch in range(1,9):
            print (rb.get_channel(ch))
            mapping[rb.rb_id][ch] = rb.get_channel(ch).paddle_end_id
        print (rb)
    print (mapping)
    
    with open('rbch-vs-paddle.json','w') as f:
        json.dump(mapping,f, indent=2)

