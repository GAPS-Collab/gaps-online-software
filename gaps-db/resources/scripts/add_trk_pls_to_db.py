#! /usr/bin/env python

"""
Adds pulse data files for Tracker to database
"""
import django
django.setup()

import tqdm
import tof_db.models as m

if __name__ == '__main__':

    import argparse

    parser = argparse.ArgumentParser(description="(Re)create tables in the global GAPS database from txt file for Si(Li) strip pulse data.")
    parser.add_argument('--utc-start', type=int, default=0,\
                        help='First UTC Timestamp of timespan for which these values shall be applied to')
    parser.add_argument('--utc-stop', type=int, default=0,\
                        help='Last UTC Timestamp of timespan for which these values shall be applied to')
    parser.add_argument('--pulse-file', type=str, default='',\
                        help="File with data for pulser for each strip")
    parser.add_argument('--gain-file', type=str, default='',\
                        help="File with gain data for each strip")
    parser.add_argument('--dry-run', action='store_true', default=False,\
                        help="Don't do anything, just print.")
    parser.add_argument('--recreate', action='store_true', default=False,\
                        help="Recreate all tables (delete them first)")
    args = parser.parse_args()

    # delete ?
    if args.recreate:
        gains = m.TrackerStripCmnNoise.objects.all() 
        for k in gains:
            k.delete()

    # get a list of strip ids - we will need this to feed mean 
    # values to those strips for which we don't have any data
    strips     = m.TrackerStrip.objects.all()
    all_strips = {k.strip_id : k for k in strips}
    print (f'In total, we have {len(all_strips)} tracker strips!') 

    if not args.gain_file or not args.pulse_file:
        raise ValueError("Needs both, pulse and gain file for this pulser run!")

    cmn_noise = m.TrackerStripCmnNoise.get_from_file(args.pulse_file, args.utc_start, args.utc_stop)
    m.TrackerStripCmnNoise.add_gains(args.gain_file, cmn_noise)

    print (f'-> Extracted {len(cmn_noise)} TrackerStripCmnNoise values')
    n_entries = 0
    cmn_noise = [cmn_noise[k] for k in cmn_noise] 
    cmn_noise = [k for k in sorted(cmn_noise, key=lambda x: x.strip_id)]

    if not args.dry_run: 
        m.TrackerStripCmnNoise.objects.bulk_create(cmn_noise)
    print (f'-> Inserted {len(cmn_noise)} TRK transfer fns in DB!')
        
        #n_entries += 1
        #if n_entries % 100:
        #    print (cmn_noise[k])
        #if not args.dry_run:
        #    cmn_noise[k].save() 

