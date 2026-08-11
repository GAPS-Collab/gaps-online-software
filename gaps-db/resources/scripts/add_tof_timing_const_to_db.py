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
    parser.add_argument('--timing-const-file', type=Path, default='',\
                        help="File with data for pulser for each strip")
    parser.add_argument('--name', type=str, default='',\
                        help="Name as an identifier for the constants")
    parser.add_argument('--version', type=int, default=0,\
                        help="Additional version identifier for identification")
    parser.add_argument('--dry-run', action='store_true', default=False,\
                        help="Don't do anything, just print.")
    parser.add_argument('--recreate', action='store_true', default=False,\
                        help="Recreate all tables (delete them first)")
    args = parser.parse_args()

    # delete ?
    if args.recreate:
        tcs = m.TofPaddleTimingConstant.objects.all() 
        for k in tcs:
            k.delete()

    if not args.timing_const_file:
        raise ValueError("Need to specify a file with timing constants!")

    tcs     = m.TofPaddleTimingConstant.get_from_file(args.timing_const_file,\
                                                      utc_start = args.utc_start,
                                                      utc_stop  = args.utc_stop,
                                                      name      = args.name,
                                                      version   = args.version,
                                                      no_fail_on_vid_check = True)
    print (f'-> Extracted {len(tcs)} TofPaddleTimingConstants pedestals!')
    # fill the rest with active masks 
    if not tcs:
        sys.exit()

    if not args.dry_run:
        print (f'-> Will save timing constants to DB')
        for tc in tcs:
            tc.save()
        # Summer 26 -  a slight adjustement - fix the sign error 
        # as well as adjust the bottom panel 
        offsets = { 13 : 0.000, 14 : 0.129, 15: -0.196, 16 : -0.122,\
                    17 : 0.001, 18 : -0.84} # panel 2a
        offsets.update({20: 0.000, 21: -0.494, 22: -0.005, 23 : -0.281}) # panel 2b
        tcs     = m.TofPaddleTimingConstant.get_from_file(args.timing_const_file,\
                                                      utc_start = args.utc_start,
                                                      utc_stop  = args.utc_stop,
                                                      name      = args.name,
                                                      version   = args.version,
                                                      no_fail_on_vid_check = True)
        for tc in tcs:
            tc.name = "GraceV1.5" 
            if tc.paddle_id in offsets.keys():
                tc.paddle_constant = offsets[tc.paddle_id] 
                tc.panel_constant  = -0.898
            try:
                tc.timing_constant = tc.paddle_constant - tc.panel_constant 
            except:
                print (tc) 
                print (tc.paddle_constant)
            tc.save()
