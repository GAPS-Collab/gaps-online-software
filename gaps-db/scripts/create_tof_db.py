#! /usr/bin/env python

import django
django.setup()

import sys
import pandas

import tof_db.models as m

SPREADSHEET_PADDLE_END = 'Paddle End Master Spreadsheet'
SPREADSHEET_PANELS     = 'Panels'
SPREADSHEET_RATS       = 'Boards in RATs'
SPREADSHEET_MTB        = 'MTB Hookup'

if __name__ == '__main__':

    import argparse

    parser = argparse.ArgumentParser(description="(Re)create tables in the global GAPS database from paddle mapping spreadsheets")
    parser.add_argument('input', metavar='input', type=str,\
                        help='Input XLS spreadsheet')
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
    parser.add_argument('--create-all-tables',       action='store_true', default=False,\
                        help="(Re)create all tables")

    args = parser.parse_args()
    if args.create_all_tables:
        args.create_paddle_end_table = True
        args.create_rat_table        = True
        args.create_dsi_table        = True
        args.create_panel_table      = True
        args.create_rb_table         = True
        args.create_ltb_table        = True
    #sure = input(f'Whatever you have selected, it is likely that current values in the global GAPS DB will get overwriten. Are you certain that you want to proceed? (YES/<any>\n\t')
    #if not sure:
    #    print(f'Abort! Nothing happend.')
    #    sys.exit(0)
    if args.create_dsi_table:
        # We know we have 5 DSI cards. Only 4 populated
        dsi_cards = { k + 1 : m.DSICard() for k in range(2)}
        for k in dsi_cards:
            dsi_cards[k].dsi_id = k
        try:
            sheet = pandas.read_excel(args.input, sheet_name=SPREADSHEET_MTB)
        except Exception as e:
            print (f'Can not read spreadsheet with name {SPREADSHEET_NTB}. Exception {e} thrown. Abort!')
            sys.exit(1)
        for row in range(1,len(sheet.index)):
            row_data = sheet.loc[row,:]
            for dsi in dsi_cards:
                try:
                    dsi_cards[dsi].add_from_spreadsheet(row_data, dsi)
                except Exception as e:
                    print(f'Can not add row\n\t {row_data}\n to dsi card {dsi}. Exception {e}')

            #print (row)
            #print (row_data)
            #print (row_data.keys())
        for card in dsi_cards:
            print (dsi_cards[card])
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
            panel.fill_from_spreadsheet(row_data)
            print (panel)
            if not args.dry_run:
                panel.save()

    if args.create_paddle_end_table:
        try:
            sheet = pandas.read_excel(args.input, sheet_name=SPREADSHEET_PADDLE_END)
        except Exception as e:
            print (f'Can not read spreadsheet with name {SPREADSHEET_PADDLE_END}. Exception {e} thrown. Abort!')
            sys.exit(1)
        for row in range(1,len(sheet.index)):
            paddle_end = m.PaddleEnd()
            row_data = sheet.loc[row,:]
            paddle_end.fill_from_spreadsheet(row_data)
            paddle_end.setup_unique_paddle_end_id()
            #print (row_data.keys())
            #print (row_data)
            #print ('----')
            print (paddle_end)
            if not args.dry_run:
                paddle_end.save()
    if args.create_rb_table:
        paddle_ends = m.PaddleEnd.objects.all()
        #paddle_ends_rb = { : []}
        #for k in paddle_ends:

        print (f'We got {len(paddle_ends)} Paddle ends.')
        rbs = {k : m.RB() for k in range(1,41)}
        
        for k in rbs:
            rbs[k].rb_id = k
            rbs[k].get_designated_ip()
            try:
                rbs[k].ch1_paddle = [j for j in paddle_ends if j.rb_id == k and j.rb_ch == 1][0]
            except Exception as e: 
                print (f"Can't add paddle end for ch1, exception {e}")
            try:
                rbs[k].ch2_paddle = [j for j in paddle_ends if j.rb_id == k and j.rb_ch == 2][0]
            except Exception as e: 
                print (f"Can't add paddle end for ch2, exception {e}")
            try:
                rbs[k].ch3_paddle = [j for j in paddle_ends if j.rb_id == k and j.rb_ch == 3][0]
            except Exception as e: 
                print (f"Can't add paddle end for ch3, exception {e}")
            try:
                rbs[k].ch4_paddle = [j for j in paddle_ends if j.rb_id == k and j.rb_ch == 4][0]
            except Exception as e: 
                print (f"Can't add paddle end for ch4, exception {e}")
            try:
                rbs[k].ch5_paddle = [j for j in paddle_ends if j.rb_id == k and j.rb_ch == 5][0]
            except Exception as e: 
                print (f"Can't add paddle end for ch5, exception {e}")
            try:
                rbs[k].ch6_paddle = [j for j in paddle_ends if j.rb_id == k and j.rb_ch == 6][0]
            except Exception as e: 
                print (f"Can't add paddle end for ch6, exception {e}")
            try:
                rbs[k].ch7_paddle = [j for j in paddle_ends if j.rb_id == k and j.rb_ch == 7][0]
            except Exception as e: 
                print (f"Can't add paddle end for ch7, exception {e}")
            try:
                rbs[k].ch8_paddle = [j for j in paddle_ends if j.rb_id == k and j.rb_ch == 8][0]
            except Exception as e: 
                print (f"Can't add paddle end for ch8, exception {e}")
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
        ltbs = {k : m.LTB() for k in range(1,21)}
        
        for k in ltbs:
            ltbs[k].ltb_id = k
            #ltbs[k].get_designated_ip()
            #try:
            #    lbts[k].ltb
            #    pass
            #except Exception as e:
            #    print (f'Can not get DSI card for LTB {k}')


            try:
                ltbs[k].ltb_ch1_rb = [j.rb_id for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 1][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch1, exception {e}")
                #print (f'Filled {ltbs[k].ltb_ch1_rb}')
            try:
                ltbs[k].ltb_ch1_rb_ch = [j.rb_ch for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 1][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch1 RB, exception {e}")
            
            # ch2
            try:
                ltbs[k].ltb_ch2_rb = [j.rb_id for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 2][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch2, exception {e}")
                #print (f'Filled {ltbs[k].ltb_ch1_rb}')
            try:
                ltbs[k].ltb_ch2_rb_ch = [j.rb_ch for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 2][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch2 RB, exception {e}")


            # ch3
            try:
                ltbs[k].ltb_ch3_rb = [j.rb_id for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 3][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch3, exception {e}")
                #print (f'Filled {ltbs[k].ltb_ch1_rb}')
            try:
                ltbs[k].ltb_ch3_rb_ch = [j.rb_ch for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 3][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch3 RB, exception {e}")
            
            # ch4
            try:
                ltbs[k].ltb_ch4_rb = [j.rb_id for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 4][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch4, exception {e}")
                #print (f'Filled {ltbs[k].ltb_ch1_rb}')
            try:
                ltbs[k].ltb_ch4_rb_ch = [j.rb_ch for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 4][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch4 RB, exception {e}")

            # ch5
            try:
                ltbs[k].ltb_ch5_rb = [j.rb_id for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 5][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch5, exception {e}")
                #print (f'Filled {ltbs[k].ltb_ch1_rb}')
            try:
                ltbs[k].ltb_ch5_rb_ch = [j.rb_ch for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 5][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch5 RB, exception {e}")
            
            # ch6
            try:
                ltbs[k].ltb_ch6_rb = [j.rb_id for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 6][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch6, exception {e}")
                #print (f'Filled {ltbs[k].ltb_ch1_rb}')
            try:
                ltbs[k].ltb_ch6_rb_ch = [j.rb_ch for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 6][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch6 RB, exception {e}")
            
            # ch7
            try:
                ltbs[k].ltb_ch7_rb = [j.rb_id for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 7][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch7, exception {e}")
                #print (f'Filled {ltbs[k].ltb_ch1_rb}')
            try:
                ltbs[k].ltb_ch7_rb_ch = [j.rb_ch for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 7][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch7 RB, exception {e}")
           
            # ch8
            try:
                ltbs[k].ltb_ch8_rb = [j.rb_id for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 8][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch8, exception {e}")
                #print (f'Filled {ltbs[k].ltb_ch1_rb}')
            try:
                ltbs[k].ltb_ch8_rb_ch = [j.rb_ch for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 8][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch8 RB, exception {e}")
           
            # ch9
            try:
                ltbs[k].ltb_ch9_rb = [j.rb_id for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 9][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch9, exception {e}")
                #print (f'Filled {ltbs[k].ltb_ch1_rb}')
            try:
                ltbs[k].ltb_ch9_rb_ch = [j.rb_ch for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 9][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch9 RB, exception {e}")
            
            # ch10
            try:
                ltbs[k].ltb_ch10_rb = [j.rb_id for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 10][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch10, exception {e}")
                #print (f'Filled {ltbs[k].ltb_ch1_rb}')
            try:
                ltbs[k].ltb_ch10_rb_ch = [j.rb_ch for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 10][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch10 RB, exception {e}")
            
            # ch11
            try:
                ltbs[k].ltb_ch11_rb = [j.rb_id for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 11][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch11, exception {e}")
                #print (f'Filled {ltbs[k].ltb_ch1_rb}')
            try:
                ltbs[k].ltb_ch11_rb_ch = [j.rb_ch for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 11][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch11 RB, exception {e}")
            
            # ch12 
            try:
                ltbs[k].ltb_ch12_rb = [j.rb_id for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 12][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch12, exception {e}")
                #print (f'Filled {ltbs[k].ltb_ch1_rb}')
            try:
                ltbs[k].ltb_ch12_rb_ch = [j.rb_ch for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 12][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch12 RB, exception {e}")
            
            # ch13
            try:
                ltbs[k].ltb_ch13_rb = [j.rb_id for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 13][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch13, exception {e}")
                #print (f'Filled {ltbs[k].ltb_ch1_rb}')
            try:
                ltbs[k].ltb_ch13_rb_ch = [j.rb_ch for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 13][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch13 RB, exception {e}")
            
            # ch14
            try:
                ltbs[k].ltb_ch14_rb = [j.rb_id for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 14][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch14, exception {e}")
                #print (f'Filled {ltbs[k].ltb_ch1_rb}')
            try:
                ltbs[k].ltb_ch14_rb_ch = [j.rb_ch for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 14][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch14 RB, exception {e}")
            
            # ch15
            try:
                ltbs[k].ltb_ch15_rb = [j.rb_id for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 15][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch15, exception {e}")
                #print (f'Filled {ltbs[k].ltb_ch1_rb}')
            try:
                ltbs[k].ltb_ch15_rb_ch = [j.rb_ch for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 15][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch15 RB, exception {e}")
            
            # ch16
            try:
                ltbs[k].ltb_ch16_rb = [j.rb_id for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 16][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch16, exception {e}")
                #print (f'Filled {ltbs[k].ltb_ch1_rb}')
            try:
                ltbs[k].ltb_ch16_rb_ch = [j.rb_ch for j in paddle_ends if j.ltb_id == k and j.ltb_ch == 16][0]
                #print (f'Filled {ltbs[k].ch1_rb}')
            except Exception as e: 
                print (f"Can't add paddle end for ltb ch16 RB, exception {e}")
        
        rats = m.RAT.objects.all()
        for ltb in ltbs:
            if not args.dry_run:
                ltbs[ltb].save()
            m.get_dsi_j_for_ltb(ltbs[ltb], rats, dsi_cards, dry_run=args.dry_run)
            print (ltbs[ltb])
