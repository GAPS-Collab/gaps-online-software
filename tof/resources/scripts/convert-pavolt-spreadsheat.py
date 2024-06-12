#! /usr/bin/env python

import polars as pl
import gaps_online.db as db

def get_pbch_for_pid(pid, ch='A'):
    paddle = db.Paddle.objects.filter(paddle_id=pid)[0]
    rat = (paddle.ltb_id)
    if ch == 'A':
        return (rat, paddle.pb_chA)
    if ch == 'B':
        return (rat, paddle.pb_chB)
    else:
        raise ValueError("Channel has to be either 'A' or 'B'")

volts = pl.read_excel('voltages_umb.xlsx')
rats = dict()
for k in range(1,21):
    rats[f'RAT{k:02}'] = [58.0]*16
#print (rats)
for  row in volts.rows():
    pid = row[1]
    pa_volt = row[3]
    if pid is not None:
        last_pid = pid
    if pid is None:
        pid = last_pid
    rat, pb_ch = get_pbch_for_pid(pid, row[2])
    key = f'RAT{k:02}'
    rats[key][pb_ch - 1] = pa_volt 

for k in rats:
    print (f'{k}={rats[k]}')

