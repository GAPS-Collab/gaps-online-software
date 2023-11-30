"""
Mapping tables for DSI/J/CH -> RB_ID RB_CH, 
paddle maps etc.
"""

import json

class DsiJChRBMap:
    """
    The mapping directly "engraved" in the MTB setup. 
    Each DSI slot has 5 J connections and each J hosts
    a LTB with 20 channels
    """

    def __init__(self, inputfile : str):
        self.data = json.load(open(inputfile))

    def get_rbid_rbch(self,dsi,j,ltb_ch):
        """
        Get a pair of (rb_id, rb_ch) for the given 
        triple of dsi card slot, J connection and 
        Channel on the LTB
        """
        return self.data[str(dsi)][str(j)][str(ltb_ch)]

