from django.shortcuts import render

import hjson
import json
from tof_db.models import LTB
from collections import defaultdict

# Create your views here.

def create_ltb_to_rb_ch_json_mapping():
    """
    Query te db for all LTBs and create a map of
    dict[dsi][j][ltb_ch] -> (rb_id, rb_ch) and 
    save that as a json file
    """

    ltbs = LTB.objects.all()
    dsi_j_ch_map = { dsi : {j :\
            {ch : (0,0) \
            for ch in range(1,17)}\
            for j in range(1,6) }\
            for dsi in range(1,6)}
    for ltb in ltbs:
        dsi_j_ch_map[ltb.ltb_dsi][ltb.ltb_j] = ltb.get_channels_to_rb()
    for dsi in dsi_j_ch_map.keys():
        for j in dsi_j_ch_map[dsi].keys():
            for ch in dsi_j_ch_map[dsi][j].keys():
                if dsi_j_ch_map[dsi][j][ch] is None:
                    dsi_j_ch_map[dsi][j][ch] = 0

    json.dump(dsi_j_ch_map, open("dsi_j_ch_map.json", "w"),\
              indent=2)

