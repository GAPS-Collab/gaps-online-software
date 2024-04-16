#! /usr/bin/env python

import django
django.setup()

import json
import sys
import re
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
    parser.add_argument('--create-rat-table',        action='store_true', default=False,\
                        help="(Re)create the rat table from the spreadsheet")
    parser.add_argument('--create-dsi-table',        action='store_true', default=False,\
                        help="(Re)create the dsi card table from the spreadsheet")
    parser.add_argument('--create-paddle-table',      action='store_true', default=False,\
                        help="(Re)create the Paddle ID table from the spreadsheet")
    parser.add_argument('--create-panel-table',      action='store_true', default=False,\
                        help="(Re)create the panel table from the spreadsheet")
    parser.add_argument('--create-ltb-table',      action='store_true', default=False,\
                        help="(Re)create the LTB table from the spreadsheet")
    parser.add_argument('--create-rb-table',      action='store_true', default=False,\
                        help="(Re)create the panel table from the spreadsheet")
    parser.add_argument('--create-mtbchannel-table',      action='store_true', default=False,\
                        help="(Re)create the MTB channel table")
    parser.add_argument('--create-all-tables',      action='store_true', default=False,\
                        help="(Re)create the complete DB")

    args = parser.parse_args()
    if args.create_all_tables:
        args.create_rat_table        = True
        args.create_dsi_table        = True
        args.create_paddle_table     = True
        args.create_panel_table      = True
        args.create_rb_table         = True
        args.create_ltb_table        = True
        args.create_mtbchannel_table = True
    
    if not args.volid_map or not args.level0_geo:
        args.create_paddle_table = False
        print("Not creating PID table without volid map and level0 geo!")
    
    if args.create_rat_table:
        try:
            sheet = polars.read_excel(args.input, sheet_name=SPREADSHEET_RATS)
        except Exception as e:
            print (f'Can not read spreadsheet with name {SPREADSHEET_RATS}. Exception {e} thrown. Abort!')
            sys.exit(1)
        rows = [k for k in sheet.rows()][1:] # first 2 rows are garbage, as are the last 2
        rows = [k for k in map(lambda x : (int(x[0]), int(x[1]), int(x[2]), int(x[3]), int(x[4]), float(x[5])*30.48), rows)]
        n_rats = 0
        for row in rows:
            rat = m.RAT()
            rat.rat_id = row[0]
            rat.pb_id  = row[1]
            # yes, rb2 comes first in the spreadsheet!
            rat.rb2_id = row[2]
            rat.rb1_id = row[3]
            rat.ltb_id = row[4]
            rat.ltb_harting_cable_length = row[5]
            print (rat)
            n_rats += 1
            if not args.dry_run:
                rat.save()
        print(f'-- {n_rats} RATs added to the DB!') 
    if args.create_dsi_table:
        try:
            # this NEEDS polars >= 0.20 and the calamine engine, otherwise this spreadsheet
            # won't be parsed correctly and misses DSI5
            sheet = polars.read_excel(args.input,engine='calamine', sheet_name=SPREADSHEET_MTB)
        except Exception as e:
            print (f'Can not read spreadsheet with name {SPREADSHEET_MTB}. Exception {e} thrown. Abort!')
            sys.exit(1)
        rows = sheet.rows()[2:]

        def format_row(r):
            pattern = re.compile('RAT(?P<ratid>[0-9]*)')
            js      = [k for k in map(lambda x :int(x[1]), [r[0],r[2],r[4],r[6],r[8]])]
            rats    = [int(pattern.search(k).groupdict()['ratid']) if k is not None else None for k in [r[1],r[3],r[5],r[7],r[9]]]
            return js, rats

        dsis = {k : m.DSICard() for k in range(1,6)}
        # take every other row, since rat and ltb js are the same
        for r in rows[::2]:
            js, rats = format_row(r)
            assert len(js) == len(rats) == 5
            js  = set(js)
            assert len(js) == 1
            j = list(js)[0]
            match j:
                case 1:
                    for k in range(5):
                        dsis[k+1].j1_rat_id = rats[k]
                case 2:
                    for k in range(5):
                        dsis[k+1].j2_rat_id = rats[k]
                case 3:
                    for k in range(5):
                        dsis[k+1].j3_rat_id = rats[k]
                case 4:
                    for k in range(5):
                        dsis[k+1].j4_rat_id = rats[k]
                case 5:
                    for k in range(5):
                        dsis[k+1].j5_rat_id = rats[k]
        for k in dsis.keys():
            dsis[k].dsi_id = k
            print (dsis[k])
            if not args.dry_run:
                dsis[k].save()

    if args.create_paddle_table:
        print('-- Creating paddle table!')
        volid_map  = json.load(open(args.volid_map))
        level0_geo = json.load(open(args.level0_geo))
        sheet      = polars.read_excel(args.input, sheet_name=SPREADSHEET_PADDLE_END)
        rows       = [r for r in sheet.rows()][1:321]
        # how this works is that we have line 0,1 for paddle 1, 2,3 for paddle 2 etc...
        paddle     = m.Paddle()
        for k,r in enumerate(rows):
            if k%2 == 0:
                paddle = m.Paddle()
            paddle.paddle_id           = int(r[0]) 
            #print (r)
            #print (paddle.paddle_id)
            assert paddle.paddle_id > 0
            assert paddle.paddle_id < 161
            paddle_end                 = r[1]
            if k%2==0:
                assert paddle_end == 'A'
            else:
                assert paddle_end == 'B'
            # FIXME - string look up
            paddle.volume_id           = int(volid_map[str(r[0])])
            panel_id                   = r[3]
            if panel_id.startswith('E'):
                # this are these individual edge paddles
                # we replace them with 1000 + the number 
                # after E-X
                panel_id = panel_id.replace("E-X","")
                paddle.panel_id = int(panel_id) + 1000
            else:
                paddle.panel_id = int(panel_id)
            paddle.cable_len           = float(r[6])
            ltb_nmb_ch                 = r[8].split('-')
            rb_nmb_ch                  = r[9].split('-')
            pb_nmb_ch                  = r[13].split('-')
            paddle.ltb_id              = int(ltb_nmb_ch[0]) 
            paddle.rb_id               = int(rb_nmb_ch[0])
            paddle.pb_id               = int(pb_nmb_ch[0])
            if paddle_end == 'A':
                paddle.ltb_chA         = int(ltb_nmb_ch[1])
                paddle.rb_chA          = int(rb_nmb_ch[1])
                paddle.pb_chA          = int(pb_nmb_ch[1])
            else:
                paddle.ltb_chB         = int(ltb_nmb_ch[1])
                paddle.rb_chB          = int(rb_nmb_ch[1])
                paddle.pb_chB          = int(pb_nmb_ch[1])
            paddle.dsi                 = int(r[10]) 
            paddle.j_ltb               = int(r[11][1])
            paddle.j_rb                = int(r[12][1])
            paddle.mtb_link_id         = int(r[18]) 
            l0_coord = level0_geo[str(paddle.volume_id)]
            x,y,z                      = l0_coord['x'], l0_coord['y'], l0_coord['z']
            length, width, height      = l0_coord['length'], l0_coord['width'], l0_coord['height']
            paddle.global_pos_x_l0     = float(x) 
            paddle.global_pos_y_l0     = float(y)
            paddle.global_pos_z_l0     = float(z)
            paddle.length              = float(length)
            paddle.height              = float(height)
            paddle.width               = float(width )
            # check in which direction the paddle is oriented, 
            # hopefully this is global coordinate
            paddle_end_loc             = r[2]
            match paddle_end_loc:
                case '+X':
                    paddle.global_pos_x_l0_A   = paddle.global_pos_x_l0 + paddle.length/2 
                    paddle.global_pos_y_l0_A   = paddle.global_pos_y_l0
                    paddle.global_pos_z_l0_A   = paddle.global_pos_z_l0
                case '-X':
                    paddle.global_pos_x_l0_A   = paddle.global_pos_x_l0 - paddle.length/2 
                    paddle.global_pos_y_l0_A   = paddle.global_pos_y_l0
                    paddle.global_pos_z_l0_A   = paddle.global_pos_z_l0
                case '+Y':
                    paddle.global_pos_x_l0_A   = paddle.global_pos_x_l0  
                    paddle.global_pos_y_l0_A   = paddle.global_pos_y_l0 + paddle.length/2
                    paddle.global_pos_z_l0_A   = paddle.global_pos_z_l0
                case '-Y':
                    paddle.global_pos_x_l0_A   = paddle.global_pos_x_l0 
                    paddle.global_pos_y_l0_A   = paddle.global_pos_y_l0 - paddle.length/2
                    paddle.global_pos_z_l0_A   = paddle.global_pos_z_l0
                case '+Z':
                    paddle.global_pos_x_l0_A   = paddle.global_pos_x_l0  
                    paddle.global_pos_y_l0_A   = paddle.global_pos_y_l0
                    paddle.global_pos_z_l0_A   = paddle.global_pos_z_l0 + paddle.length/2
                case '-Z':
                    paddle.global_pos_x_l0_A   = paddle.global_pos_x_l0  
                    paddle.global_pos_y_l0_A   = paddle.global_pos_y_l0
                    paddle.global_pos_z_l0_A   = paddle.global_pos_z_l0 + paddle.length/2
                case _:
                    raise ValueError("Can not parse {paddle_end_loc} for paddle end location!")
            if k%2 != 0:
                print (paddle)
                if not args.dry_run:
                    paddle.save()
    
    if args.create_panel_table:
        print ('-- Creating panel table!')
        try:
            sheet = polars.read_excel(args.input, sheet_name=SPREADSHEET_PANELS)
        except Exception as e:
            print (f'Can not read spreadsheet with name {SPREADSHEET_PANELS}. Exception {e} thrown. Abort!')
            sys.exit(1)
        rows    = sheet.rows()[:25]
        for r in rows:
            panel = m.Panel()
            panel_id    = r[0]
            if panel_id.startswith('E'):
                # this are these individual edge paddles
                # we replace them with 1000 + the number 
                # after E-X
                panel_id = panel_id.replace("E-X","")
                panel.panel_id = int(panel_id) + 1000
            else:
                panel.panel_id = int(panel_id)
            desc = r[1]
            panel.description = desc
            # hardcode the orientation, too tired for 
            # pattern matching, sorry...
            if 'umbrella' in desc or 'top' in desc or 'bottom' in desc:
                normal = [0,0,1]
            else:
                match panel.panel_id:
                    case 3 | 14:
                        normal = [1,0,0]
                    case 5 | 16:
                        normal = [-1,0,0]
                    case 4 | 15:
                        normal = [0,1,0]
                    case 6 | 17:
                        normal = [0,-1,0]
                    case 18 | 1045:
                        normal = [1,1,0]
                    case 19 | 1135:
                        normal = [-1,1,0]
                    case 20 | 1225:
                        normal = [-1,-1,0]
                    case 21 | 1315:
                        normal = [1,-1,0]
            panel.normal_x = normal[0]
            panel.normal_y = normal[1]
            panel.normal_z = normal[2]

            paddles = m.Paddle.objects.filter(panel_id=panel.panel_id)
            if not len (paddles):
                raise ValueError("Need to create Paddle table first!")
            paddles = sorted(paddles, key=lambda x: x.paddle_id) 
            for k,pdl in enumerate(paddles):
                match k:
                    case 0:
                        panel.paddle0 = pdl
                    case 1:
                        panel.paddle1 = pdl
                    case 2:
                        panel.paddle2 = pdl
                    case 3:
                        panel.paddle3 = pdl
                    case 4:
                        panel.paddle4 = pdl
                    case 5:
                        panel.paddle5 = pdl
                    case 6:
                        panel.paddle6 = pdl
                    case 7:
                        panel.paddle7 = pdl
                    case 8:
                        panel.paddle8 = pdl
                    case 9:
                        panel.paddle9 = pdl
                    case 10:
                        panel.paddle10 = pdl
                    case 11:
                        panel.paddle11 = pdl
                    case _:
                        ValueError("Too many paddles for this panel!")

            print (panel)
            #print (panel.description, panel.normal_x, panel.normal_y, panel.normal_z)
            if not args.dry_run:
                panel.save()


    if args.create_rb_table:
        # The readoutboard table can be generated completely from 
        # a list of eligible RBs and the paddle table
        rb_ids = [k for k in range(1,51) if not k in RB_IGNORELIST]
        print('-- creating RB table for ids {rb_ids}')
        for rid in rb_ids:
            rb = m.ReadoutBoard()
            paddles = m.Paddle.objects.filter(rb_id = rid)
            assert len(paddles) == 4
            dsi = set([pdl.dsi for pdl in paddles])
            assert len(dsi) == 1
            dsi = list(dsi)[0]
            j   = set([pdl.j_rb for pdl in paddles])
            assert len(j) == 1
            j   = list(j)[0] 
            mtb_link_id = set([pdl.mtb_link_id for pdl in paddles])
            assert len (mtb_link_id) == 1
            mtb_link_id    = list(mtb_link_id)[0]
            rb.rb_id       = rid
            rb.dsi         = dsi
            rb.j           = j
            rb.mtb_link_id = mtb_link_id 
            for pdl in paddles:
                match pdl.rb_chA:
                    case 1:
                        rb.paddle12     = pdl
                        rb.paddle12_chA = 1
                    case 2:
                        rb.paddle12     = pdl
                        rb.paddle12_chA = 2
                    case 3:
                        rb.paddle34     = pdl
                        rb.paddle34_chA = 3
                    case 4:
                        rb.paddle34     = pdl
                        rb.paddle34_chA = 4
                    case 5:
                        rb.paddle56     = pdl
                        rb.paddle56_chA = 5
                    case 6:
                        rb.paddle56     = pdl
                        rb.paddle56_chA = 6
                    case 7:
                        rb.paddle78     = pdl
                        rb.paddle78_chA = 7
                    case 8:
                        rb.paddle78     = pdl
                        rb.paddle78_chA = 8
            print (rb)
            if not args.dry_run:
                rb.save()


    if args.create_ltb_table:
        print ("-- Creating LTB table")
        dsi_cards   = m.DSICard.objects.all()
        rats        = m.RAT.objects.all()
        ltbs        = dict()
        # let's loop over the RAT table first, to find 
        # out which ltbs exist
        for rat in rats:
            ltb           = m.LocalTriggerBoard()
            ltb.rat       = rat.rat_id
            ltb.board_id  = rat.rat_id
            ltb.cable_len = rat.ltb_harting_cable_length
            # populate the dsi/j fields
            for dsi in dsi_cards:
                if not dsi.has_rat(ltb.rat):
                    continue
                ltb.dsi = dsi.dsi_id
                ltb.j   = dsi.get_j(ltb.rat)
            # for later
            paddles = m.Paddle.objects.filter(ltb_id = ltb.board_id)
            assert len(paddles) == 8
            paddles = sorted([k for k in paddles], key=lambda x : x.ltb_chA)  
            for k,pdl in enumerate(paddles):
                match k:
                    case 0:
                        ltb.paddle1 = pdl
                    case 1:
                        ltb.paddle2 = pdl
                    case 2:
                        ltb.paddle3 = pdl
                    case 3:
                        ltb.paddle4 = pdl
                    case 4:
                        ltb.paddle5 = pdl
                    case 5:
                        ltb.paddle6 = pdl
                    case 6:
                        ltb.paddle7 = pdl
                    case 7:
                        ltb.paddle8 = pdl

            ltbs[ltb.board_id]  = ltb

        for k in ltbs:
            print (ltbs[k])
            if not args.dry_run:
                ltbs[k].save()

    if args.create_mtbchannel_table:
        # mtbchannels go from 0-320
        #ltbs = m.LocalTriggerBoard.objects.all()
        #ifor ltb in ltbs:
        mtb_ch = 0
        for dsi in range(1,6):
            for j in range(1,6):
                print (dsi, j)
                try:
                    ltb = m.LocalTriggerBoard.objects.filter(dsi=dsi, j=j)[0]
                except Exception as e:
                    print (f"No LTB for {dsi}/{j}!")
                    mch = m.MTBChannel()
                    mch.mtb_ch = mtb_ch
                    for ch in range(1,17):
                        mtb_ch += 1
                        if not args.dry_run:
                            mch.save()
                    continue
                ltb_channels = dict()
                for ch in range(1,17):
                    pdl_isA = True
                    pdl = [k for k in filter(lambda x : x.ltb_chA == ch, ltb.paddles)]
                    if not pdl:
                        pdl_isA = False
                        pdl = [k for k in filter(lambda x : x.ltb_chB == ch, ltb.paddles)]
                    assert len(pdl) == 1
                    pdl = pdl[0]
                    print (pdl)
                    mch = m.MTBChannel()
                    mch.mtb_ch      = mtb_ch
                    mch.dsi         = dsi
                    mch.j           = j
                    mch.ltb_id      = ltb.board_id
                    mch.rb_id       = pdl.rb_id
                    if pdl_isA:
                        mch.ltb_ch      = pdl.ltb_chA
                        mch.rb_ch       = pdl.rb_chA
                    else:
                        mch.ltb_ch      = pdl.ltb_chB
                        mch.rb_ch       = pdl.rb_chB
                    mch.paddle_id   = pdl.paddle_id
                    mch.paddle_isA  = pdl_isA
                    rb = m.ReadoutBoard.objects.filter(rb_id = pdl.rb_id)[0]
                    mch.mtb_link_id = rb.mtb_link_id
                    mch.set_hg_channel()
                    mch.set_lg_channel()
                    mtb_ch += 1
                    print (mch)
                    if not args.dry_run:
                        mch.save()
        print (f"-- Added {mtb_ch} MTBChannels to the DB!")

