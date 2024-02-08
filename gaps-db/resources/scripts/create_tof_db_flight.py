#! /usr/bin/env python

import django
django.setup()

import json
import sys
import pandas
import re
# FIXME - move from pandas to polars!
import polars
import numpy as np

import tof_db.models as m

RB_IGNORELIST = [10,12,37,38,43,45,47,48,49,50,51]

SPREADSHEET_PADDLE_END = 'Paddle End Master Spreadsheet'
SPREADSHEET_PANELS     = 'Panels'
SPREADSHEET_RATS       = 'Boards in RATs'
SPREADSHEET_MTB        = 'MTB Hookup'

if __name__ == '__main__':

    import argparse

    parser = argparse.ArgumentParser(description="(Re)create tables in the global GAPS database from paddle mapping spreadsheets")
    parser.add_argument('input', metavar='input', type=str,\
                        help='Input XLS spreadsheet')
    parser.add_argument('--volid-map', default="",\
                        help=".json file with mapping pid->volid")
    parser.add_argument('--level0-geo', default="",\
                        help=".json file with mapping volid->l0 geo coord")
    parser.add_argument('--dry-run', action='store_true', default=False,\
                        help="Don't do anything, just print.")
    parser.add_argument('--create-paddle-end-table', action='store_true', default=False,\
                        help="(Re)create the paddle end table from the spreadsheet")
    parser.add_argument('--create-rat-table',        action='store_true', default=False,\
                        help="(Re)create the rat table from the spreadsheet")
    parser.add_argument('--create-dsi-table',        action='store_true', default=False,\
                        help="(Re)create the dsi card table from the spreadsheet")
    parser.add_argument('--create-panel-table',      action='store_true', default=False,\
                        help="(Re)create the panel table from the spreadsheet")
    parser.add_argument('--create-rb-table',      action='store_true', default=False,\
                        help="(Re)create the panel table from the spreadsheet")
    parser.add_argument('--create-ltb-table',      action='store_true', default=False,\
                        help="(Re)create the LTB table from the spreadsheet")
    parser.add_argument('--create-pid-table',      action='store_true', default=False,\
                        help="(Re)create the Paddle ID table from the spreadsheet")
    parser.add_argument('--create-mtbchannel-table',      action='store_true', default=False,\
                        help="(Re)create the MTB channel table")
    parser.add_argument('--create-all-tables',      action='store_true', default=False,\
                        help="(Re)create the complete DB")

    args = parser.parse_args()
    if args.create_all_tables:
        args.create_paddle_end_table = True
        args.create_rat_table        = True
        args.create_dsi_table        = True
        args.create_panel_table      = True
        args.create_rb_table         = True
        args.create_ltb_table        = True
        args.create_pid_table        = True
        args.create_mtbchannel_table = True
    
    if not args.volid_map or not args.level0_geo:
        args.create_pid_table = False
        print("Not creating PID tablew without volid map and level0 geo!")
    #sure = input(f'Whatever you have selected, it is likely that current values in the global GAPS DB will get overwriten. Are you certain that you want to proceed? (YES/<any>\n\t')
    #if not sure:
    #    print(f'Abort! Nothing happend.')
    #    sys.exit(0)
    if args.create_dsi_table:
        try:
            sheet = pandas.read_excel(args.input, sheet_name=SPREADSHEET_MTB)
        except Exception as e:
            print (f'Can not read spreadsheet with name {SPREADSHEET_MTB}. Exception {e} thrown. Abort!')
            sys.exit(1)
        dsi_card_header = sheet.loc[1,:]
        dsi_cards_row = [k for k in dsi_card_header.index if not k.startswith('Unnamed')]
        pattern = re.compile('DSI card (?P<dsi_id>1|2|3|4|5)')
        dsi_cards = dict()
        print (dsi_card_header)
        for k in dsi_cards_row:
            card_id = int(pattern.search(k).groupdict()['dsi_id'])
            dsi_cards[card_id] = m.DSICard()
            dsi_cards[card_id].dsi_id = card_id
            
        pattern = re.compile('RBs RAT(?P<rat_id>\d{1,2})')
        for row in range(2,len(sheet.index)):
            row_data = sheet.loc[row,:]
            cols_for_dsi = {1 : 'Unnamed: 1',\
                            2 : 'Unnamed: 4',\
                            3 : 'Unnamed: 7',\
                            4 : 'Unnamed: 10',\
                            5 : 'Unnamed: 13'}
            for k in dsi_cards.keys():
                key = f'DSI card {k}'
                print ('key',key)
                print (row_data[key])
                if not row_data[key].startswith('J'):
                    continue
                if row_data[key].endswith('_1'):
                    continue
                this_j   = int(row_data[key][1])
                print (cols_for_dsi, k)
                print (this_j)
                print (row_data)
                thiscol = cols_for_dsi[int(k)]
                try:
                    row_data[thiscol] 
                except KeyError as e:
                    print(f'Can not find key {key}! {e}, skipping..')
                    continue
                if row_data[thiscol] == 'X':
                    continue
                
                #if np.isnan(row_data[thiscol]):
                #    continue
                try:
                    rat_id = pattern.search(row_data[thiscol]).groupdict()['rat_id']
                except TypeError as e:
                    print (thiscol)
                    print (row_data[thiscol])
                    print (f"Error, can't parse! {e}")
                    continue
                dsi_cards[int(k)].add_rat_id_for_j(this_j, rat_id)
        for card in dsi_cards:
            print (f"Found DSI card {dsi_cards[card]}")
            if not args.dry_run:
                dsi_cards[card].save()

    if args.create_rat_table:
        try:
            sheet = pandas.read_excel(args.input, sheet_name=SPREADSHEET_RATS)
        except Exception as e:
            print (f'Can not read spreadsheet with name {SPREADSHEET_RATS}. Exception {e} thrown. Abort!')
            sys.exit(1)
        for row in range(1,len(sheet.index)):
            rat = m.RAT()
            row_data = sheet.loc[row,:]
            print (row_data.keys())
            try:
                rat.fill_from_spreadsheet(row_data)
            except ValueError as e:
                print (f'Can not convert row {row}. Exception {e} thrown. Row data {row_data}. Skipping this RAT')
                continue
            print (rat)
            if not args.dry_run:
                rat.save()

    if args.create_panel_table:
        try:
            sheet = pandas.read_excel(args.input, sheet_name=SPREADSHEET_PANELS)
        except Exception as e:
            print (f'Can not read spreadsheet with name {SPREADSHEET_PANELS}. Exception {e} thrown. Abort!')
            sys.exit(1)
        for row in range(1,len(sheet.index)):
            row_data = sheet.loc[row,:]
            panel = m.Panel()
            try:
                panel.fill_from_spreadsheet(row_data)
            except Exception as e:
                print (row_data)
                print (f"Can't parse panel! {e}")
                continue
            print (panel)
            if not args.dry_run:
                panel.save()

    if args.create_paddle_end_table:
        try:
            sheet = pandas.read_excel(args.input, sheet_name=SPREADSHEET_PADDLE_END)
        except Exception as e:
            print (f'Can not read spreadsheet with name {SPREADSHEET_PADDLE_END}. Exception {e} thrown. Abort!')
            sys.exit(1)
        try:
            sheet_plr = polars.read_excel(args.input, sheet_name=SPREADSHEET_PADDLE_END)
        except Exception as e:
            print (f'Can not read spreadsheet with name {SPREADSHEET_PADDLE_END}. Exception {e} thrown. Abort!')
            sys.exit(1)
        
        ploc_col = sheet_plr.get_column("Paddle Location in Panel ")
        for row in range(1,len(sheet.index)):
            paddle_end = m.PaddleEnd()
            row_data = sheet.loc[row,:]
            print ('++++++++')
            print(row_data)
            paddle_end.fill_from_spreadsheet(row_data)
            paddle_end.setup_unique_paddle_end_id()
            paddle_end.pos_in_panel = ploc_col[row]
            #print (row_data.keys())
            #print (row_data)
            #print ('----')
            if paddle_end.panel_id is None:
                print ('Error, no panel_id, setting 99')
                paddle_end.panel_id = 99
            print (paddle_end)
            if not args.dry_run:
                paddle_end.save()
    if args.create_pid_table:
        paddle_ends = m.PaddleEnd.objects.all()
        if len(paddle_ends) == 0:
            print (f'[FATAL] - need to create paddle end table first! Abort..')
            sys.exit(1)
        pid_dict = {k : m.Paddle() for k in range(1,161)}
        
        volid_map  = json.load(open(args.volid_map))
        level0_geo = json.load(open(args.level0_geo))
        for k in pid_dict.keys():
            pid_dict[k].paddle_id = k
            vid = int(volid_map[str(k)])
            pid_dict[k].volume_id = vid
            l0_coord = level0_geo[str(vid)]
            x,y,z = l0_coord['x'], l0_coord['y'], l0_coord['z']
            pid_dict[k].global_pos_x_l0 = x
            pid_dict[k].global_pos_y_l0 = y
            pid_dict[k].global_pos_z_l0 = z
            if not args.dry_run:
                print (f'{pid_dict[k]}')
                pid_dict[k].save()

    if args.create_rb_table:
        rbs = {k : m.RB() for k in range(1,51)}
        
        for k in rbs:
            if k in RB_IGNORELIST:
                continue
            rbs[k].rb_id = k
            for ch in range(1,9):
                try:
                    pend = m.PaddleEnd.objects.filter(\
                            rb_id = rbs[k].rb_id,\
                            rb_ch = ch)[0]
                except Exception as e:
                    print (f"Can't get info for {rbs[k].rb_id} {ch}")
                    raise
                rbs[k].set_channel(ch, pend)
            print (rbs[k])
            if not args.dry_run:
                rbs[k].save()

    if args.create_ltb_table:
        print ("Creating LTB table")
        paddle_ends = m.PaddleEnd.objects.all()
        dsi_cards   = m.DSICard.objects.all()
        #paddle_ends_rb = { : []}
        #for k in paddle_ends:
        print (f'We got {len(paddle_ends)} Paddle ends.')
        #ltbs = {k : m.LTB() for k in range(1,21)}
        #
        #for k in ltbs:
        #    ltbs[k].ltb_id = k
        ltbs = dict()
        for paddle_end in paddle_ends:
            ltb_id = paddle_end.ltb_id
            if not ltb_id in ltbs:
                print (f"Adding {ltb_id}")
                ltbs[ltb_id] = m.LTB()
                ltbs[ltb_id].ltb_id = ltb_id
            ltb_ch = paddle_end.ltb_ch
            data   = {ltb_ch : [paddle_end.rb_id, paddle_end.rb_ch]}
            ltbs[ltb_id].set_channels_to_rb(data)
        
        rats = m.RAT.objects.all()
        for ltb in ltbs:
            m.get_dsi_j_for_ltb(ltbs[ltb], rats, dsi_cards, dry_run=args.dry_run)
            if not ltbs[ltb].is_populated():
                continue
            print (ltbs[ltb])
            if not args.dry_run:
                ltbs[ltb].save()

    if args.create_mtbchannel_table:
        ltbs = m.LTB.objects.all()
        mtbch = 0
        for ltb in ltbs:
            channels = ltb.get_channels_to_rb()
            for ch in channels:
                mtb             = m.MTBChannel()
                mtb.mtb_channel = mtbch
                mtb.dsi         = ltb.ltb_dsi
                mtb.j           = ltb.ltb_j
                mtb.ltb_id      = ltb.ltb_id
                mtb.ltb_channel = ch
                mtb.rb_id       = channels[ch][0]
                mtb.rb_channel  = channels[ch][1] 
                mtb.set_hg_channel()
                mtb.set_lg_channel()
                mtbch += 1
                if mtb.hg_channel is None or mtb.lg_channel is None:
                    print (f"WARN - not enough information for {ltb}, {ch} => {channels[ch]}")
                    print (mtb)
                    continue

                ltb_pend = m.PaddleEnd.objects.filter(ltb_id = ltb.ltb_id, ltb_ch = ch)
                assert (len(ltb_pend) == 1)
                ltb_pend = ltb_pend[0]
                rb_pend  = m.PaddleEnd.objects.filter(rb_id  = mtb.rb_id, rb_ch = mtb.rb_channel)
                assert (len(rb_pend) == 1)
                rb_pend  = rb_pend[0]
                assert(ltb_pend.paddle_end_id == rb_pend.paddle_end_id)
                mtb.p_end_id = rb_pend.paddle_end_id
                if not args.dry_run:
                    print (mtb)
                    try:
                        mtb.save()
                    except Exception as e:
                        print (e)
                        foo = m.MTBChannel.objects.filter(lg_channel = mtb.lg_channel)
                        for k in foo:
                            print (k)
                        raise 
                else:
                    print (mtb)
