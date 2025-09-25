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
from copy import deepcopy as copy
from pathlib import Path

if __name__ == '__main__':

    import argparse

    parser = argparse.ArgumentParser(description="(Re)create tables in the global GAPS database from json file for Si(Li) strip positions as used in te simulation")
    parser.add_argument('--coordinates', type=str, default='',\
                        help='Input json file with strip coordinates')
    parser.add_argument('--pedestals', type=str, default='',\
                        help="Filename for SiLi pedestal values")
    parser.add_argument('--tracker-mask', type=str, default='',\
                        help="Filename for a file with tracker masks (1 mask per strip)")
    parser.add_argument('--dry-run', action='store_true', default=False,\
                        help="Don't do anything, just print.")
    parser.add_argument('--transfer-fns', type=str, default='',\
                        help="Filename with the polynomial fit versions of the transfer functions from e.g. Riccardo")
    

    args = parser.parse_args()
    if args.coordinates:
        coord_data = json.load(open(args.coordinates))
        nstrips = 0 
        strips_to_create = []
        for k in tqdm.tqdm(coord_data, desc="Inserting TRK strips in DB..."):
            sd = coord_data[k]
            strip           = m.TrackerStrip()
            strip.layer     = int(sd['layer'])
            strip.row       = int(sd['row'])
            strip.module    = int(sd['module'])
            strip.channel   = int(sd['channel'])
            strip.volume_id = int(k)
            # we have a global shift between simulation 
            # geometry and that what we get from Erik
            delta_z = -20.22628 # cm

            strip.global_pos_x_det_l0 = float(sd['det_x'])/10
            strip.global_pos_y_det_l0 = float(sd['det_y'])/10
            strip.global_pos_z_det_l0 = float(sd['det_z'])/10 + delta_z
            strip.global_pos_x_l0     = float(sd['x'])/10
            strip.global_pos_y_l0     = float(sd['y'])/10
            strip.global_pos_z_l0     = float(sd['z'])/10 + delta_z
            # FIXME - this should be included in a save hook
            strip.strip_id        = strip.get_id()
            nstrips += 1
            if nstrips % 1000 == 0:
                print (strip)
            if not args.dry_run:
                strips_to_create.append(strip)
            #strip.save()
        # Saving strips to db
        m.TrackerStrip.objects.bulk_create(strips_to_create)
        print (f'-> We processed {nstrips} strips!')

    #if args.pedestals:
    #    pedf = open(args.pedestals, 'r')
    #    # HACK - file not properly named
    #    if args.pedestals.endswith('LDB_9December.txt'):
    #        timestamp = 241209000000

    #    # first, bootstrap the pedestal table
    #    strips = m.TrackerStrip.objects.all()
    #    for strip in tqdm.tqdm(strips, desc='Bootstrapping pedestal table...'):
    #        pedestal           = m.TrackerStripPedestal()
    #        pedestal.volume_id = strip.volume_id
    #        pedestal.strip_id  = strip.strip_id
    #        pedestal.save()
    #    print ('-> Pedestal table bootstrapped!')
    #    all_pedestal_mean  = 0
    #    all_pedestal_sigma = 0
    #    n_pedestal         = 0
    #    nstrips            = 0 
    #    for line in pedf.readlines():
    #        nstrips += 1
    #        data = [float(k) for k in line.split()]
    #        layer = int(data[0])
    #        row   = int(data[1])
    #        mod   = int(data[2])
    #        chn   = int(data[3])
    #        strip_id = m.TrackerStrip.create_id(layer, row, mod, chn)
    #        if nstrips % 100 == 0:
    #            print (f'-> Layer {layer}, row {row}, mod {mod}, chn {chn}')
    #            print (f'-> Getting pedestal for strip {strip_id}')
    #        try:
    #            pedestal = m.TrackerStripPedestal.objects.filter(strip_id=strip_id)[0]
    #        except:
    #            print ("WARNING! No pedestal for this strip!")
    #            continue
    #        pedestal.utc_timestamp = timestamp
    #        pedestal.pedestal_mean  = data[4]
    #        pedestal.pedestal_sigma = data[5]
    #        n_pedestal += 1
    #        all_pedestal_mean  += pedestal.pedestal_mean
    #        all_pedestal_sigma += pedestal.pedestal_sigma
    #        pedestal.is_mean_value = False
    #        pedestal.save()
    #        print (pedestal) 
    #    # set pedestal values for the strips which did not have 
    #    # a value set by the file above
    #    all_pedestal_mean  /= n_pedestal
    #    all_pedestal_sigma /= n_pedestal
    #    mean_peds = m.TrackerStripPedestal.objects.filter(is_mean_value=True)
    #    for ped in mean_peds:
    #        pedestal.pedestal_mean  = all_pedestal_mean
    #        pedestal.pedestal_sigma = all_pedestal_sigma
    #        if not args.dry_run:
    #            pedestal.save()
    #        print (pedestal)
    #    print (f'--> We set mean pedestal values for {len(mean_peds)}')   
    #if args.tracker_mask:
    #    maskmap = dict()
    #    nstrip = 0
    #    with open(args.tracker_mask, 'r') as maskf:
    #        for line in maskf.readlines():
    #            nstrip += 1
    #            module_id, mask = line.split()
    #            mask = int(mask, base=16)
    #            #print (module_id, mask)
    #            layer  = int(module_id[0])
    #            row    = int(module_id[1])
    #            module = int(module_id[2])
    #            for k in range(32):
    #                strip_mask = mask >> k & 0x1
    #                tsmask = m.TrackerStripMask()
    #                tsmask.strip_id = m.TrackerStrip.create_id(layer, row, module, k)
    #                vid = m.TrackerStrip.objects.filter(strip_id=tsmask.strip_id)[0].volume_id
    #                tsmask.active = bool(strip_mask)
    #                tsmask.volume_id = vid
    #                tsmask.mask_name = str(Path(args.tracker_mask).stem)
    #                if nstrip % 100 == 0:
    #                    print(f'-> Getting tracker strip for id {tsmask.strip_id}') 
    #                    print (tsmask)
    #                if not args.dry_run:
    #                    tsmask.save()

    if args.transfer_fns:
        transfer_fns = m.TrackerStripTransferFunction.get_from_file(args.transfer_fns)
        nstrip = 0
        strips = m.TrackerStrip.objects.all()
        strips = {k.strip_id : k for k in strips}
        hid_vid_map = m.TrackerStrip.get_hid_vid_map()   
        for k in transfer_fns:
            default_tf = transfer_fns[k]
            break 
        for k in strips:
            try:
                transfer_fns[k]
            except KeyError:
                default_tf.strip_id  = k
                default_tf.volume_id = hid_vid_map[k]
                default_tf.pol_a2_0
                default_tf.pol_a2_1   
                default_tf.pol_a2_2

                default_tf.pol_b3_0
                default_tf.pol_b3_1
                default_tf.pol_b3_2
                default_tf.pol_b3_3

                default_tf.pol_c3_0
                default_tf.pol_c3_1
                default_tf.pol_c3_2
                default_tf.pol_c3_3
                transfer_fns[k] = copy(default_tf) 

        transfer_fns = [transfer_fns[k] for k in transfer_fns] 
        transfer_fns = [k for k in sorted(transfer_fns, key=lambda x: x.strip_id)]
        if not args.dry_run: 
            m.TrackerStripTransferFunction.objects.bulk_create(transfer_fns)
        print (f'-> Inserted {len(transfer_fns)} TRK transfer fns in DB!')

        #if not args.dry_run:
        #    for k in transfer_fns:
        #        nstrip += 1
        #        if nstrip % 100 == 0:
        #            print (transfer_fns[k])
        #        vid = m.TrackerStrip.objects.filter(strip_id=transfer_fns[k].strip_id)[0].volume_id
        #        transfer_fns[k].volume_id = vid
        #        transfer_fns[k].save()

        
