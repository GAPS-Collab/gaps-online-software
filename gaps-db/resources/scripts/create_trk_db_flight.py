#! /usr/bin/env python

import django
django.setup()

import json
import sys
import re
import polars
import numpy as np

import tof_db.models as m


if __name__ == '__main__':

    import argparse

    parser = argparse.ArgumentParser(description="(Re)create tables in the global GAPS database from json file for Si(Li) strip positions as used in te simulation")
    parser.add_argument('input', metavar='input', type=str,\
                        help='Input json file')
    parser.add_argument('--dry-run', action='store_true', default=False,\
                        help="Don't do anything, just print.")

    args = parser.parse_args()
    jsondata = json.load(open(args.input))
    for k in jsondata:
        sd = jsondata[k]
        strip = m.TrackerStrip()
        strip.layer     = int(sd['layer'])
        strip.row       = int(sd['row'])
        strip.module    = int(sd['module'])
        strip.channel   = int(sd['channel'])
        strip.volume_id = int(k)
        strip.global_pos_x_l0 = float(sd['x'])
        strip.global_pos_y_l0 = float(sd['y'])
        strip.global_pos_z_l0 = float(sd['z'])
        print (strip)
        if not args.dry_run:
            strip.save()

    
