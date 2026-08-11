#! /usr/bin/env python 

# offsets by paddle

paddle_offsets = {\
'panel_1_offsets'  : [0.000,0.391,-0.069,0.279,-0.002,0.299,-0.430,-0.311,-0.201,-0.441,-0.434,-0.465],
'panel_2a_offsets' : [0.000,0.129,-0.196,-0.122,0.001,-0.84],
'panel_2b_offsets' : [0.000,-0.494,-0.005,-0.281],
'panel_3_offsets'  : [0.000,-0.397,0.291,0.305,0.463,0.463,0.799,0.781],
'panel_4_offsets'  : [0.000,0.384,0.391,0.523,0.187,0.876,0.544,0.833],
'panel_5a_offsets' : [0.000,0.113,0.828],
'panel_5b_offsets' : [0.000,0.460,0.737],
'panel_6_offsets'  : [0.000,0.366,0.172,0.610,0.479,0.755,0.956,1.034],
'panel_7_offsets'  : [0.000,0.024,0.091,0.092,0.165,0.247,-0.722,-0.048,-0.816,-0.158,-0.979,-0.099],
'panel_8_offsets'  : [0.000,0.023,-0.335,-0.031,-0.465,-0.039],
'panel_9_offsets'  : [0.000,-0.161,-0.051,-0.160,0.212,0.089],
'panel_10_offsets' : [0.000,0.095,-0.922,-0.362,-0.976,-0.233],
'panel_11_offsets' : [0.000,-0.773,0.788,1.622,1.041,1.297],
'panel_12_offsets' : [0.000,0.350,0.119,0.532,0.252],
'panel_13_offsets' : [0.000,0.528,0.495,0.696,0.724,.716],
'panel_14_offsets' : [0.000,-0.043,0.097,0.241,-0.254,-0.184,-0.135,-0.102,0.406,-0.044],
'panel_15_offsets' : [0.000,0.224,0.258,0.428,0.649,0.853,1.981,1.032,0.457,0.134],
'panel_16_offsets' : [0.000,-0.114,-0.017,0.534,0.339,0.242,0.429,0.180,-0.570,0.311],
'panel_17_offsets' : [0.000,-0.555,0.063,-0.419,0.345,-0.154,0.492,-0.129,0.681,0.102],
'panel_18_offsets' : [0.000,-0.042,-0.125],
'panel_19_offsets' : [0.000,-0.018,0.023],
'panel_20_offsets' : [0.000,0.610,-0.184],
'panel_21_offsets' : [0.000,0.610,-0.184],
'panel_57_offsets' : [0.000],
'panel_58_offsets' : [0.000],
'panel_59_offsets' : [0.000],
'panel_60_offsets' : [0.000]}

# panel offsets 
panel_to_panel_dt = [0.000,-0.898,-1.106,-0.161,0.278,0.136,-0.161,-0.026,0.472,0.438,-0.707,-0.726,0.542,1.505,1.544,-0.171,-0.280,-0.505,-0.098,-0.381,-0.928,-0.022,-0.412,-0.147,-0.430,0.408,-0.236]

# volume ids
panel_vids = {\
'panel_1_vids'  : [110000000, 110000100, 110000200, 110000300, 110000400, 110000500, 110000600, 110000700, 110000800, 110000900, 110001000, 110001100],
'panel_2a_vids' : [111000000, 111000100, 111000200, 111000300, 111000400, 111000500],

# FIXME - I guess 11100700 should be 111000700 ? 
# -- also paddle 24 is broken, right?
'panel_2b_vids' : [11100700, 111000800, 111000900, 111001000],
'panel_3_vids'  : [112000700, 112000600, 112000500, 112000400, 112000300, 112000200, 112000100, 112000000],
'panel_4_vids'  : [114000700, 114000600, 114000500, 114000400, 114000300, 114000200, 114000100, 114000000],
# -- the last volume id in panel 5a vids is paddle 6?
'panel_5a_vids' : [113000700, 113000600, 110000500],
'panel_5b_vids' : [113000200, 113000100, 113000000],
'panel_6_vids'  : [115000000, 115000100, 115000200, 115000300, 115000400, 115000500, 115000600, 115000700],
'panel_57_vids' : [116000000],
'panel_58_vids' : [116200000],
'panel_59_vids' : [116300000],
'panel_60_vids' : [116100000],
'panel_7_vids'  : [100000000, 100000100, 100000200, 100000300, 100000400, 100000500, 100000600, 100000700, 100000800, 100000900, 100001000, 100001100],
'panel_8_vids'  : [100300500, 100300400, 100300300, 100300200, 100300100, 100300000],
'panel_9_vids'  : [100200500, 100200400, 100200300, 100200200, 100200100, 100200000],
'panel_10_vids' : [100400000, 100400100, 100400200, 100400300, 100400400, 100400500],
'panel_11_vids' : [100600500, 100600400, 100600300, 100600200, 100600100, 100600000],
# I guess paddle 97 is just broken, right?
'panel_12_vids' : [100100400, 100100300, 100100200, 100100100, 100100000],
'panel_13_vids' : [100500500, 100500400, 100500300, 100500200, 100500100, 100500000],
'panel_14_vids' : [102000900, 102000800, 102000700, 102000600, 102000500, 102000400, 102000300, 102000200, 102000100, 102000000],
'panel_15_vids' : [104000000, 104000100, 104000200, 104000300, 104000400, 104000500, 104000600, 104000700, 104000800, 104000900],
'panel_16_vids' : [103000900, 103000800, 103000700, 103000600, 103000500, 103000400, 103000300, 103000200, 103000100, 103000000],
'panel_17_vids' : [105000900, 105000800, 105000700, 105000600, 105000500, 105000400, 105000300, 105000200, 105000100, 105000000],
'panel_18_vids' : [106000200, 106000100, 106000000],
'panel_19_vids' : [106200000, 106200100, 106200200],
'panel_20_vids' : [106300000, 106300100, 106300200],
'panel_21_vids' : [106100200, 106100100, 106100000]} 


import gondola as go 

v_map      = go.db.get_vid_hid_maps()[0]
# previous version of the offsets, as used before 
v1_offsets = go.db.TofPaddleTimingConstant.as_dict_by_name('GraceV1') 
# check the panel vids - just print them for now 
for panel in panel_vids:
    print (f' -- -- {panel}') 
    for j in panel_vids[panel]: 
        try:
            print (f' -- -- -- {j} => {v_map[j]}')
        except Exception as e:
            print (e)
            print (f'ERROR! vid {j} is unknown!') 

# print the weird ones, 24 and 97 
print ('== == Checking paddles 24 and 97')
print (v1_offsets[24]) # -> hm, we seemed to have one for 24 last time ... 
print (v1_offsets[97]) 

# compare the old to the new paddle-paddle contants 
for k in paddle_offsets:
    vids = k.replace('offsets', 'vids')
    vids = panel_vids[vids] 
    assert len(vids) == len(paddle_offsets[k]) 
    data = [(v_map[m],v1_offsets[v_map[m]], n) for m,n in zip(vids, paddle_offsets[k]) if not m == 11100700] 
    for el in data:
        if el[2] == 0:
            if el[1].paddle_constant == 0:
                continue
            else: 
                ratio = f'new {el[2]} -- v1 {el[1].paddle_constant}'
                print (f'-- discrepancy for paddle {el[0]} is {ratio}')
        else:
            ratio =  el[1].paddle_constant / el[2] 
            # we do expect discrepancies for paddle 13-24 (panel 2) 
            # anything else might be unexpected, but ok (?) 
            # however, I guess wareants a cross check?
            if abs(ratio - 1) > 0.01:
                print (f'-- discrepancy for paddle {el[0]} is {abs(ratio - 1):.2}%')
            #print (el[0].paddle_constant, el[1]) 


