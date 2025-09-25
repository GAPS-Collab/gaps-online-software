#! /usr/bin/env python

"""
Adds mask data files for Tracker to database
"""
import django
django.setup()

import sys
import tqdm
import tof_db.models as m

if __name__ == '__main__':

    import argparse

    parser = argparse.ArgumentParser(description="(Re)create tables in the global GAPS database from txt file for Si(Li) strip mask data.")
    parser.add_argument('--utc-start', type=int, default=0,\
                        help='First UTC Timestamp of timespan for which these values shall be applied to')
    parser.add_argument('--utc-stop', type=int, default=0,\
                        help='Last UTC Timestamp of timespan for which these values shall be applied to')
    parser.add_argument('--mask-file', type=str, default='',\
                        help="File with masks for each strip")
    parser.add_argument('--dry-run', action='store_true', default=False,\
                        help="Don't do anything, just print.")
    parser.add_argument('--recreate', action='store_true', default=False,\
                        help="Recreate all tables (delete them first)")
    args = parser.parse_args()

    # delete ?
    if args.recreate:
        gains = m.TrackerStripMask.objects.all() 
        for k in gains:
            k.delete()

    # get a list of strip ids - we will need this to feed mean 
    # values to those strips for which we don't have any data
    strips     = m.TrackerStrip.objects.all()
    strips     = {k.strip_id : k for k in strips}
    print (f'In total, we have {len(strips)} tracker strips!') 

    if not args.mask_file:
        raise ValueError("Needs file with tracker masks!")
    hid_vid_map = m.TrackerStrip.get_hid_vid_map()
    trk_mask    = m.TrackerStripMask.get_from_file(args.mask_file, args.utc_start, args.utc_stop)
    print (f'-> Extracted {len(trk_mask)} TRK masks!')
    # fill the rest with active masks 
    if not trk_mask:
        sys.exit()

    for k in trk_mask:
        blueprint_mask = trk_mask[k]
        break 

    for k in strips:
        try:
            trk_mask[k]
        except KeyError:
            empty_mask  = m.TrackerStripMask()
            empty_mask.strip_id = k
            vid      = hid_vid_map[k]
            empty_mask.active    = True
            empty_mask.volume_id = vid
            empty_mask.name      = blueprint_mask.name
            trk_mask[k]          = empty_mask 

    #n_entries = 0
    trk_mask = [trk_mask[k] for k in trk_mask] 
    trk_mask = [k for k in sorted(trk_mask, key=lambda x: x.strip_id)]

    if not args.dry_run: 
        m.TrackerStripMask.objects.bulk_create(trk_mask)
    print (f'-> Inserted {len(trk_mask)} TRK masks in DB!')
    #for k in tqdm.tqdm(trk_mask, desc="Inserting TRK masks in DB.."):
    #    n_entries += 1
    #    if n_entries % 100:
    #        print (trk_mask[k])
    #    if not args.dry_run:
    #        trk_mask[k].save() 

