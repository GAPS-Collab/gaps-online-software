#! /usr/bin/env python

"""
Adds pedestal data files for Tracker to database
"""
import django
django.setup()

import tqdm
import tof_db.models as m

from pathlib import Path
import sys

if __name__ == '__main__':

    import argparse

    parser = argparse.ArgumentParser(description="(Re)create tables in the global GAPS database from txt file for Si(Li) strip pedestal data.")
    parser.add_argument('--utc-start', type=int, default=0,\
                        help='First UTC Timestamp of timespan for which these values shall be applied to')
    parser.add_argument('--utc-stop', type=int, default=0,\
                        help='Last UTC Timestamp of timespan for which these values shall be applied to')
    parser.add_argument('--pedestal-file', type=Path, default='',\
                        help="File with data for pulser for each strip")
    parser.add_argument('--dry-run', action='store_true', default=False,\
                        help="Don't do anything, just print.")
    parser.add_argument('--recreate', action='store_true', default=False,\
                        help="Recreate all tables (delete them first)")
    args = parser.parse_args()

    # delete ?
    if args.recreate:
        ped = m.TrackerStripPedestal.objects.all() 
        for k in ped:
            k.delete()

    # get a list of strip ids - we will need this to feed mean 
    # values to those strips for which we don't have any data
    strips     = m.TrackerStrip.objects.all()
    strips     = {k.strip_id : k for k in strips}
    print (f'In total, we have {len(strips)} tracker strips!') 
    if len(strips) == 0:
        raise ValueError("Running this script requires that there is a GAPS db with strip information already!")

    if not args.pedestal_file:
        raise ValueError("Need to specify a file for pedestals!")

    hid_vid_map = m.TrackerStrip.get_hid_vid_map()
    trk_ped     = m.TrackerStripPedestal.get_from_file(args.pedestal_file, args.utc_start, args.utc_stop)
    print (f'-> Extracted {len(trk_ped)} TRK pedestals!')
    # fill the rest with active masks 
    if not trk_ped:
        sys.exit()

    n_pedestal          = 0
    all_pedestal_mean   = 0
    all_pedestal_sigma  = 0

    for k in trk_ped:
        pedestal = trk_ped[k]
        n_pedestal += 1
        all_pedestal_mean  += pedestal.pedestal_mean
        all_pedestal_sigma += pedestal.pedestal_sigma
    
    all_pedestal_mean  /= n_pedestal
    all_pedestal_sigma /= n_pedestal

    for k in strips:
        try:
            trk_ped[k]
        except KeyError:
            empty_ped  = m.TrackerStripPedestal()
            empty_ped.strip_id = k
            vid      = hid_vid_map[k]
            empty_ped.volume_id = vid
            empty_ped.name      = pedestal.name
            trk_ped[k]           = empty_ped 
            trk_ped[k].pedestal_mean  = all_pedestal_mean
            trk_ped[k].pedestal_sigma = all_pedestal_sigma

    #n_entries = 0
    trk_ped = [trk_ped[k] for k in trk_ped] 
    trk_ped = [k for k in sorted(trk_ped, key=lambda x: x.strip_id)]

    if not args.dry_run: 
        m.TrackerStripPedestal.objects.bulk_create(trk_ped)
    print (f'-> Inserted {len(trk_ped)} TRK pedestals in DB!')


