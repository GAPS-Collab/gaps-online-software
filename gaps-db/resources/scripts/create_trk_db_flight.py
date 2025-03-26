#! /usr/bin/env python

import django
django.setup()

import json
import sys
import re
import polars
import numpy as np
import tqdm
import tof_db.models as m


if __name__ == '__main__':

    import argparse

    parser = argparse.ArgumentParser(description="(Re)create tables in the global GAPS database from json file for Si(Li) strip positions as used in te simulation")
    parser.add_argument('--coordinates', type=str, default='',\
                        help='Input json file with strip coordinates')
    parser.add_argument('--pedestals', type=str, default='',\
                        help="Filename for SiLi pedestal values")
    parser.add_argument('--dry-run', action='store_true', default=False,\
                        help="Don't do anything, just print.")

    args = parser.parse_args()
    if args.coordinates:
        coord_data = json.load(open(args.coordinates))
        nstrips = 0 
        for k in coord_data:
            sd = coord_data[k]
            strip = m.TrackerStrip()
            strip.layer     = int(sd['layer'])
            strip.row       = int(sd['row'])
            strip.module    = int(sd['module'])
            strip.channel   = int(sd['channel'])
            strip.volume_id = int(k)
            strip.global_pos_x_det_l0 = float(sd['det_x'])
            strip.global_pos_y_det_l0 = float(sd['det_y'])
            strip.global_pos_z_det_l0 = float(sd['det_z'])
            strip.global_pos_x_l0 = float(sd['x'])
            strip.global_pos_y_l0 = float(sd['y'])
            strip.global_pos_z_l0 = float(sd['z'])
            # FIXME - this should be included in a save hook
            strip.strip_id        = strip.get_id()
            print (strip)
            nstrips += 1
            if not args.dry_run:
                strip.save()
            print (f'-> We processed {nstrips} strips!')
    if args.pedestals:
        pedf = open(args.pedestals, 'r')
        # HACK - file not properly named
        if args.pedestals.endswith('LDB_9December.txt'):
            timestamp = 241209000000

        # first, bootstrap the pedestal table
        strips = m.TrackerStrip.objects.all()
        for strip in tqdm.tqdm(strips, desc='Bootstrapping pedestal table...'):
            pedestal           = m.TrackerStripPedestal()
            pedestal.volume_id = strip.volume_id
            pedestal.strip_id  = strip.strip_id
            pedestal.save()
        print ('-> Pedestal table bootstrapped!')
        all_pedestal_mean  = 0
        all_pedestal_sigma = 0
        n_pedestal         = 0
        for line in pedf.readlines():
            data = [float(k) for k in line.split()]
            layer = int(data[0])
            row   = int(data[1])
            mod   = int(data[2])
            chn   = int(data[3])
            strip_id = m.TrackerStrip.create_id(layer, row, mod, chn)
            print (f'-> Layer {layer}, row {row}, mod {mod}, chn {chn}')
            print (f'-> Getting pedestal for strip {strip_id}')
            pedestal = m.TrackerStripPedestal.objects.filter(strip_id=strip_id)[0]
            pedestal.utc_timestamp = timestamp
            pedestal.pedestal_mean  = data[4]
            pedestal.pedestal_sigma = data[5]
            n_pedestal += 1
            all_pedestal_mean  += pedestal.pedestal_mean
            all_pedestal_sigma += pedestal.pedestal_sigma
            pedestal.is_mean_value = False
            pedestal.save()
            print (pedestal) 
        # set pedestal values for the strips which did not have 
        # a value set by the file above
        all_pedestal_mean  /= n_pedestal
        all_pedestal_sigma /= n_pedestal
        mean_peds = m.TrackerStripPedestal.objects.filter(is_mean_value=True)
        for ped in mean_peds:
            pedestal.pedestal_mean  = all_pedestal_mean
            pedestal.pedestal_sigma = all_pedestal_sigma
            pedestal.save()
            print (pedestal)
        print (f'--> We set mean pedestal values for {len(mean_peds)}')   
