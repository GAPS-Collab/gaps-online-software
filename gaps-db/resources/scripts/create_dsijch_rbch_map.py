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
    dsi = m.DSICard.objects.all()
    mapping = dict()
    for card in dsi:
        print (card)
        mapping[card.dsi_id] = dict()
        for j in range(1,6):
            rat = card.get_rat(j)
            if rat is None:
                print(f"There is no RAT connected to DSI{card.dsi_id}/{j}")
                continue
            print (f"Got rat with id {rat}")
            ltb = m.LTB.objects.filter(ltb_id=rat)[0]
            mapping[card.dsi_id][j] = ltb.get_channels_to_rb()
            #print (ltb)
    print (mapping)
    
    with open('dsi_j_ch_map.gaps.json','w') as f:
        json.dump(mapping,f, indent=2)

