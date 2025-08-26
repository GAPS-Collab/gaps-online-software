# the speed of light in a tof paddle
C_LIGHT_PADDLE = 15.4

try:
    import django
    django.setup()
    from .. import db
    import os
    os.environ['DJANGO_ALLOW_ASYNC_UNSAFE'] = '1'

except Exception as e:
    print(f"Can't load django environment, gaps_db will not be available. {e}")

_pybind_imported = False

try:
    import go_pybindings
    create_mtb_connection_to_pid_map = go_pybindings.io.create_mtb_connection_to_pid_map
    _pybind_imported = True
except ImportError as e:
    print (f'-> Unable to import pybindings! {e}')

import numpy as np
import dashi as d
import tqdm
import sys
import json
import tomllib 
# for some reason, toml writing is not native
# in the standard library, needs third-party
import tomli_w 
#from loguru import logger
#logger.add(sys.stdout,level='INFO')

from copy import deepcopy as copy

from ..events import EventStatus
from ..db import get_tof_paddles

#########################################################

def find_paddle(hit, paddles):
    """
    Get a paddle id for a trigger hit
    where the trigger hit is (dsi, j, ch)
    """
    # hit is dsi, ch, channel
    for pdl in paddles:
        if pdl.dsi == hit[0]:
            if pdl.j_ltb == hit[1]:
                if pdl.ltb_chA == hit[2][0]:
                    return pdl.paddle_id
                elif pdl.ltb_chB == hit[2][0]:
                    return pdl.paddle_id
    print (f'No paddle found for {hit}')

#########################################################

def distance(h0, h1) -> float:
    """ The spatial distance between 2 TofHits """
    return np.sqrt((h0.x - h1.x)**2 + (h0.y - h1.y) ** 2 + (h0.z - h1.z) ** 2) 

#########################################################

def create_occupancy_dict(reader           = None,
                          events           = [],
                          normalize        = True,
                          use_trigger_hits = False,
                          mark_0_as_bad    = False,
                          cbe_side         = True,
                          cor_side         = True):
    """
    Create a dictionary of paddle id vs nhits

    This can either accept a reader or a list of events.
    Use reader when memory is sparse and events when time is
    of the essence
    
    # Arguments:
        * reader           - either TofPacket or TelemetryPacketReader. The reader should be primed in a way
                             that it only spits out MergedEvents, TofEventSummary or TofEvents
    
        * use_trigger_hits - instead of plotting TofHits, just use the triggered hits for the occupancy
        * cbe_side         - add the CBE sides to the occupancy dictionary. It might make sense to exclude 
                             them for normalization reasons
        * cor_side         - add the COR sides to the occupancy dictionary. It might make sense to exclude 
                             them for normalization reasons
    """

    if reader is not None and events:
        raise ValueError("Unable to use both, reader and events!")

    if use_trigger_hits:
        paddles = db.get_tof_paddles()

    if reader is not None:
        for ev in reader:
            ev0 = ev
            break;
    else:
        # events can be TofEventSummary or TofEvent
        ev0 = events[0]

    is_tes = False
    if hasattr(ev0,'trigger_hits'):
        is_tes = True

    is_merged_event = False
    if hasattr(ev0,'tof'):
        is_merged_event = True

    occu_per_paddle = {k : 0 for k in range(1,161)}
    if reader is not None:
        events = reader
    for ev in tqdm.tqdm(events, desc='Getting TOF occupancy data!'):
        if use_trigger_hits:
            if is_tes:
                for h in ev.trigger_hits:
                    pid = find_paddle(h, paddles)
                    occu_per_paddle[pid] += 1
            elif is_merged_event:
                try:
                    ev = ev.tof
                except:
                    continue
                if trigger_hits:
                    for h in ev.trigger_hits:
                        pid = find_paddle(h, paddles)
                        occu_per_paddle[pid] += 1
                else:
                    for h in ev.hits:
                        pid = find_paddle(h, paddles)
                        occu_per_paddle[pid] += 1

            else:
                for h in ev.mastertriggerevent.trigger_hits:
                    pid = find_paddle(h, paddles)
                    occu_per_paddle[pid] += 1

        else:
            for h in ev.hits:
                occu_per_paddle[h.paddle_id] += 1

    if not cbe_side:
        for pid in range(25, 61):
            occu_per_paddle.pop(pid)
    if not cor_side:
        for pid in range(109, 161):
            occu_per_paddle.pop(pid)
    # normalize it
    if normalize:
        max_val = max(occu_per_paddle.values())
        for k in occu_per_paddle.keys():
            occu_per_paddle[k] = occu_per_paddle[k]/max_val
    if mark_0_as_bad:
        for k in occu_per_paddle.keys():
            if occu_per_paddle[k] == 0.0:
                occu_per_paddle[k] = np.nan
    return occu_per_paddle

##################################################################

def calc_rms(data) -> float:
    """ root mean square calculation """
    return np.sqrt((data ** 2).sum() / len(data))


##################################################################

class TofCuts:
    """
    A simple container to hold a collection of TOF relevant cut 
    values
    """
    
    def __init__(self):
        self.min_hit_cor          = 0
        self.min_hit_cbe          = 0
        self.min_hit_umb          = 0
        self.max_hit_cor          = 161
        self.max_hit_cbe          = 161
        self.max_hit_umb          = 161
        self.min_hit_all          = 0
        self.max_hit_all          = 161
        self.min_cos_theta        = 0.0
        self.max_cos_theta        = 1.0 
        self.only_causal_hits     = False
        self.hit_cbe_acc          = 0 
        self.hit_umb_acc          = 0 
        self.hit_cor_acc          = 0
        self.hit_all_acc          = 0
        self.cos_theta_acc        = 0
        self.nevents              = 0
        self.hits_total           = 0
        self.hits_rmvd_csl        = 0
        self.hits_rmvd_ls         = 0
        # Require that the first hit MUST
        # be on the umbrella!
        self.fh_must_be_umb       = False
        self.fh_umb_acc           = 0
        self.ls_cleaning_t_err    = np.inf
        # the last hit of the event must be 
        # either on the cortina or on CBE BOT
        self.thru_going           = False
        self.thru_going_acc       = 0
        # the first hits on the inner can not be on 
        # the bottom panel
        self.fhi_not_bot          = False
        self.fhi_not_bot_acc      = 0
        # more criteria for first/last hit 
        self.fho_must_panel7      = False
        self.fho_must_panel7_acc  = 0
        self.lh_must_panel2       = False 
        self.lh_must_panel2_acc   = 0 
        # select only events with one hit which 
        # has a large energy deposition
        self.hit_high_edep        = False
        self.hit_high_edep_acc    = 0

    def to_toml(self):
        toml_dict = {
            'min_hit_cor'      : self.min_hit_cor    ,    
            'min_hit_cbe'      : self.min_hit_cbe    ,    
            'min_hit_umb'      : self.min_hit_umb    ,     
            'max_hit_cor'      : self.max_hit_cor    ,    
            'max_hit_cbe'      : self.max_hit_cbe    ,      
            'max_hit_umb'      : self.max_hit_umb    ,      
            'min_hit_all'      : self.min_hit_all    ,       
            'max_hit_all'      : self.max_hit_all    ,        
            'min_cos_theta'    : self.min_cos_theta  ,       
            'max_cos_theta'    : self.max_cos_theta  ,          
            'only_causal_hits' : self.only_causal_hits,      
            'fh_must_be_umb'   : self.fh_must_be_umb  ,    
            'ls_cleaning_t_err': self.ls_cleaning_t_err,    
            'thru_going'       : self.thru_going      ,   
            'fhi_not_bot'      : self.fhi_not_bot     ,   
            'fho_must_panel7'  : self.fho_must_panel7 ,   
            'lh_must_panel2'   : self.lh_must_panel2  , 
        }
        with open("tof_cuts.toml", "w") as f:
            tomli_w.dump(toml_dict , f)

    def from_toml(self, toml_filepath):
        with open(toml_filepath, 'rb') as toml_file:
            tomldict = tomllib.load(toml_file)

        self.min_hit_cor      = tomldict['min_hit_cor']        
        self.min_hit_cbe      = tomldict['min_hit_cbe']        
        self.min_hit_umb      = tomldict['min_hit_umb']         
        self.max_hit_cor      = tomldict['max_hit_cor']        
        self.max_hit_cbe      = tomldict['max_hit_cbe']          
        self.max_hit_umb      = tomldict['max_hit_umb']          
        self.min_hit_all      = tomldict['min_hit_all']           
        self.max_hit_all      = tomldict['max_hit_all']            
        self.min_cos_theta    = tomldict['min_cos_theta']         
        self.max_cos_theta    = tomldict['max_cos_theta']            
        self.only_causal_hits = tomldict['only_causal_hits']      
        self.fh_must_be_umb   = tomldict['fh_must_be_umb']      
        self.ls_cleaning_t_err= tomldict['ls_cleaning_t_err']   
        self.thru_going       = tomldict['thru_going']         
        self.fhi_not_bot      = tomldict['fhi_not_bot']        
        self.fho_must_panel7  = tomldict['fho_must_panel7']    
        self.lh_must_panel2   = tomldict['lh_must_panel2']   

    def clear_stats(self):
        """
        Zero out the event/hit counter variables
        """
        self.hit_cbe_acc         = 0 
        self.hit_umb_acc         = 0 
        self.hit_cor_acc         = 0
        self.hit_all_acc         = 0 
        self.cos_theta_acc       = 0
        self.nevents             = 0
        self.hits_total          = 0
        self.hits_rmvd_csl       = 0
        self.hits_rmvd_ls        = 0
        self.fh_umb_acc          = 0
        self.thru_going_acc      = 0
        self.fhi_not_bot_acc     = 0
        self.fho_must_panel7_acc = 0 
        self.lh_must_panel2_acc  = 0 
        self.hit_high_edep_acc   = 0

    @property
    def void(self):
        if self.min_hit_cor      != 0:
            return False
        if self.min_hit_cbe      != 0:
            return False
        if self.min_hit_umb      != 0:
            return False
        if self.max_hit_cor      != 161:
            return False
        if self.max_hit_cbe      != 161:
            return False
        if self.max_hit_umb      != 161:
            return False  
        if self.min_hit_all      != 0:
            return False 
        if self.max_hit_all      != 161:
            return False
        if self.only_causal_hits:
            return False
        if self.ls_cleaning_t_err != np.inf:
            return False
        if self.fh_must_be_umb != False:
            return False
        if self.thru_going != False:
            return False
        if self.fhi_not_bot != False:
            return False
        if self.min_cos_theta != 0:
            return False 
        if self.max_cos_theta != 1:
            return False 
        if self.fho_must_panel7:
            return False 
        if self.lh_must_panel2:
            return False 
        if self.hit_high_edep:
            return False
        return True

        
    def is_compatible(self, other):
        """
        Void cuts will autmaticaly be compatible
        """
        if self.only_causal_hits != other.only_causal_hits:
            return False
        if self.min_hit_cor  != other.min_hit_cor:
            return False
        if self.min_hit_cbe  != other.min_hit_cbe:
            return False
        if self.min_hit_umb  != other.min_hit_umb:
            return False
        if self.max_hit_cor  != other.max_hit_cor:
            return False
        if self.max_hit_cbe  != other.max_hit_cbe:
            return False
        if self.max_hit_umb  != other.max_hit_umb:
            return False
        if self.min_hit_all  != other.min_hit_all:
            return False
        if self.max_hit_all  != other.max_hit_all:
            return False
        if self.ls_cleaning_t_err != other.ls_cleaning_t_err:
            return False
        if self.fh_must_be_umb != other.fh_must_be_umb:
            return False
        if self.thru_going != other.thru_going:
            return False
        if self.fhi_not_bot != other.fhi_not_bot:
            return False
        if self.min_cos_theta != other.min_cos_theta:
            return False 
        if self.max_cos_theta != other.max_cos_theta:
            return False 
        if self.fho_must_panel7 != other.fho_must_panel7:
            return False 
        if self.lh_must_panel2 != other.lh_must_panel2:
            return False 
        if self.hit_high_edep != other.hit_high_edep:
            return False
        return True

    def __iadd__(self, other):
        if not self.is_compatible(other):
            raise ValueError("Cuts are not compatible!")
        self.nevents             += other.nevents
        self.hit_cbe_acc         += other.hit_cbe_acc 
        self.hit_umb_acc         += other.hit_umb_acc 
        self.hit_cor_acc         += other.hit_cor_acc
        self.hit_all_acc         += other.hit_all_acc
        self.cos_theta_acc       += other.cos_theta_acc 
        self.hits_total          += other.hits_total
        self.hits_rmvd_csl       += other.hits_rmvd_csl
        self.hits_rmvd_ls        += other.hits_rmvd_ls
        self.fh_umb_acc          += other.fh_umb_acc
        self.thru_going_acc      += other.thru_going_acc
        self.fhi_not_bot_acc     += other.fhi_not_bot_acc
        self.fho_must_panel7_acc += other.fho_must_panel7_acc 
        self.lh_must_panel2_acc  += other.lh_must_panel2_acc
        self.hit_high_edep_acc   += other.hit_high_edep_acc
        return self

    def __add__(self, other):
        new_cuts = TofCuts()
        if not self.is_compatible(other):
            raise ValueError("Cuts are not compatible!")
        new_cuts += self
        new_cuts += other
        return new_cuts

    @property
    def acc_frac_hit_cbe(self):
        if self.nevents == 0:
            return 0
        return self.hit_cbe_acc/self.nevents
    
    @property
    def acc_frac_hit_cor(self):
        if self.nevents == 0:
            return 0
        return self.hit_cor_acc/self.nevents
    
    @property
    def acc_frac_hit_umb(self):
        if self.nevents == 0:
            return 0
        return self.hit_umb_acc/self.nevents
    
    @property
    def acc_frac_fh_is_umb(self):
        if self.nevents == 0:
            return 0
        return self.fh_umb_acc/self.nevents

    @property
    def acc_frac_thru_going(self):
        if self.nevents == 0:
            return 0
        return self.thru_going_acc/self.nevents
    
    @property
    def acc_frac_fhi_not_bot(self):
        if self.nevents == 0:
            return 0
        return self.fhi_not_bot_acc/self.nevents

    @property
    def acc_frac_hit_all(self):
        if self.nevents == 0:
            return 0
        return self.hit_all_acc/self.nevents

    @property 
    def acc_frac_cos_theta(self):
        if self.nevents == 0:
            return 0 
        return self.cos_theta_acc/self.nevents 

    @property
    def acc_frac_fho_must_panel7(self):
        if self.nevents == 0:
            return 0
        return self.fho_must_panel7_acc/self.nevents 

    @property
    def acc_frac_lh_must_panel2(self):
        if self.nevents == 0:
            return 0
        return self.lh_must_panel2_acc/self.nevents 
    
    @property
    def acc_frac_hit_high_edep(self):
        if self.nevents == 0:
            return 0
        return self.hit_high_edep_acc/self.nevents 

    def pretty_print_efficiency(self):
        _repr =  f'-- -- -- -- -- -- -- -- -- -- --'
        _repr +=  f'\n TOTAL EVENTS : {self.nevents}'
        _repr += f'\n  {self.min_hit_umb} <= NHit(UMB) <= {self.max_hit_umb} : {100*self.acc_frac_hit_umb : .2f} %' 
        _repr += f'\n  {self.min_hit_cbe} <= NHit(CBE) <= {self.max_hit_cbe} : {100*self.acc_frac_hit_cbe : .2f} %' 
        _repr += f'\n  {self.min_hit_cor} <= NHit(COR) <= {self.max_hit_cor} : {100*self.acc_frac_hit_cor : .2f} %' 
        _repr += f'\n  {self.min_hit_all} <= NHit(TOF) <= {self.max_hit_all} : {100*self.acc_frac_hit_all : .2f} %' 
        _repr += f'\n  {self.min_cos_theta} <= COS(THET) <= {self.max_cos_theta} : {100*self.acc_frac_cos_theta : .2f} %'  
        if self.only_causal_hits:
            if self.hits_total > 0:
                _repr += f'\n Removed {100*self.hits_rmvd_csl/self.hits_total:.2f} % of hits due to causality cut!'
        if self.ls_cleaning_t_err != np.inf:
            if self.hits_total > 0:
                _repr += f'\n Removed {100*self.hits_rmvd_ls/self.hits_total:.2f} % of hits due to lightspeed cut!'
        if self.fh_must_be_umb:
            _repr += f'\n First hit must be on UMB!'
            _repr += f'\n   -- Accepted {100*self.acc_frac_fh_is_umb : .2f} %'
        if self.thru_going:
            _repr += '\n Require through-going track!'
            _repr += f'\n   -- Accepted {100*self.acc_frac_thru_going : .2f} %'
        if self.fhi_not_bot:
            _repr += '\n Require first hit on the inner TOF can not be on the Bottom 12PP'
            _repr += f'\n   -- Accepted {100*self.acc_frac_fhi_not_bot : .2f} %'
        if self.fho_must_panel7:
            _repr += '\n Require first hit on the outer TOF must be on panel7'
            _repr += f'\n   -- Accepted {100*self.acc_frac_fho_must_panel7 : .2f} %'
        if self.lh_must_panel2:
            _repr += '\n Require last hit must be on the bottom CBE panel'
            _repr += f'\n   -- Accepted {100*self.acc_frac_lh_must_panel2 : .2f} %'
        if self.hit_high_edep:
            _repr += '\n Require that one hit has an edep > 20MeV'
            _repr += f'\n   -- Accepted {100*self.acc_frac_hit_high_edep : .2f} %'

        _repr +=  f'\n-- -- -- -- -- -- -- -- -- -- --'
        return _repr 

    def __str__(self):
        return self.__repr__()

    def __repr__(self):
        _repr  = '<TofCuts:'
        if self.only_causal_hits:
            _repr += f'\n -- removes non-causal hits!'
        if self.ls_cleaning_t_err != np.inf:
            _repr += f'\n -- removes hits which are not correlated with the first hit!'
            _repr += f'\n --   assumed timing error {self.ls_cleaning_t_err}'
        if self.fh_must_be_umb:
            _repr += f'\n -- first hit must be on UMB'
        if self.thru_going:
            _repr += f'\n -- require last hit on CBE BOT or COR (thru-going tracks)'
        if self.fhi_not_bot:
            _repr += f'\n -- require that the first hit on the inner TOF is not on CBE BOT'
        if self.fho_must_panel7:
            _repr += f'\n -- require that the first hit on the outer TOF is on panel7'
        if self.lh_must_panel2:
            _repr += f'\n -- require that the last hit on the inner TOF is on CBE BOT'
        if self.hit_high_edep:
            _repr += f'\n -- require that at least one hit has an edep of > 29MeV'
        _repr += f'\n  {self.min_hit_umb} <= NHit(UMB) <= {self.max_hit_umb}' 
        _repr += f'\n  {self.min_hit_cbe} <= NHit(CBE) <= {self.max_hit_cbe}' 
        _repr += f'\n  {self.min_hit_cor} <= NHit(COR) <= {self.max_hit_cor}' 
        _repr += f'\n  {self.min_hit_all} <= NHit(TOF) <= {self.max_hit_all}' 
        _repr += f'\n  {self.min_cos_theta} <= COS(THET) <= {self.max_cos_theta}' 
        _repr += '>'
        return _repr
    
    def accept(self, ev):
        """
        Check if an event passes the selection
        and update the counters
        """
        # The order of events is important. Hit cleaning 
        # comes before the application of cuts.
        self.hits_total += ev.nhits
        self.nevents    += 1
        if self.only_causal_hits:
            rm_pids : list = ev.remove_non_causal_hits()
            self.hits_rmvd_csl  += len(rm_pids)
        if self.ls_cleaning_t_err != np.inf:
            rm_pids_ls = ev.lightspeed_cleaning(self.ls_cleaning_t_err)
            self.hits_rmvd_ls   += len(rm_pids_ls)

        # get number of cbe/umb/cor hits - only for valid hits
        nhits_cbe : int = ev.nhits_cbe
        nhits_umb : int = ev.nhits_umb
        nhits_cor : int = ev.nhits_cor
        
        # check for min/max hits on cbe, umb, cor
        # these cuts are combined with AND
        if not self.min_hit_all <= nhits_cbe + nhits_umb + nhits_cor <= self.max_hit_all:
            return False
        else:
            self.hit_all_acc += 1

        if not self.min_hit_cbe <= nhits_cbe <= self.max_hit_cbe:
            return False
        else:
            self.hit_cbe_acc += 1

        if not self.min_hit_umb <= nhits_umb <= self.max_hit_umb:
            return False
        else:
            self.hit_umb_acc += 1
        
        if not self.min_hit_cor <= nhits_cor <= self.max_hit_cor:
            return False
        else:
            self.hit_cor_acc += 1
       
        # at this point, it can still be that we don't have any TOF hits at all
        # the following set of cuts can only be calculated if there are hits
        #no_cos_possible = False 
        if self.fh_must_be_umb \
        or self.thru_going \
        or self.fhi_not_bot \
        or (self.min_cos_theta != 0) \
        or (self.max_cos_theta != 1) \
        or self.fho_must_panel7 \
        or self.lh_must_panel2 \
        or self.hit_high_edep:
            hits_sorted = sorted(ev.hits, key=lambda x: x.event_t0)
            if len(hits_sorted) == 0:
                # if we don't have hits, we also don't fulfill any of these conditions. simple.
                return False
            first_pid  = hits_sorted[0].paddle_id 
            last_pid   = hits_sorted[-1].paddle_id
            hits_inner = [k for k in hits_sorted if k.paddle_id < 61]
            hits_outer = [k for k in hits_sorted if k.paddle_id > 60] 
        # now we are sure that there are hits
        if self.fh_must_be_umb:
            if  (first_pid < 61 or first_pid > 108):
                return False
            else:
                self.fh_umb_acc += 1
        else:
            self.fh_umb_acc += 1

        if self.thru_going:
            if  (last_pid in range(13,25) or 108 < last_pid):
                self.thru_going_acc += 1
            else:
                return False
        else:
            self.thru_going_acc += 1
        
        if self.fhi_not_bot:
            if len(hits_inner) == 0:
                self.fhi_not_bot_acc += 1
            elif (12 < hits_inner[0].paddle_id < 25):
                return False
            else:
                self.fhi_not_bot_acc += 1
        else:
            self.fhi_not_bot_acc += 1
        
        if self.min_cos_theta != 0 or self.max_cos_theta != 1:
            # FIXME - this should not happen!
            if len(hits_inner) == 0 or len(hits_outer) == 0:
                return False
            dist = hits_inner[0].distance(hits_outer[0])/1000
            cos_theta = abs(hits_inner[0].z - hits_outer[0].z)/(1000*dist)  
            if not self.min_cos_theta <= cos_theta <= self.max_cos_theta:
                return False
            else:
                self.cos_theta_acc += 1
        if self.fho_must_panel7:
            if first_pid not in range(61, 73):
                return False 
            else:
                self.fho_must_panel7_acc += 1
        if self.lh_must_panel2:
            if last_pid not in range(13,25):
                return False 
            else:
                self.lh_must_panel2_acc += 1
        if self.hit_high_edep:
            found = False 
            for h in hits_sorted:
                if h.edep > 20:
                    self.lh_must_panel2_acc += 1
                    found = True
                    break
            if not found:
                return False 
        # if we arrive here, we passed everything
        return True

##################################################################

class TofAnalysis:
    """
    A container (yeah I know, don't like it either) to keep a 
    bunch of plots together.

    This does have some use as a pre-compiled analysis for 
    gander, and as a quick look kind of thing.

    The gist here it is independent of the data source, as 
    long as some kind of TofEvent can be plugged in.
    """

    def define_bins(self, nbins = 70):
        """
        Set the bins for the different histograms for the 
        variables. Only the number of bins can be set
        """
        self.PADDLE_PEAK_BINS   = np.linspace(0,200,     nbins)
        self.PADDLE_CHARGE_BINS = np.linspace(-2,60 ,     nbins)
        self.PADDLE_TIMING_BINS = np.linspace(0,250,     nbins)
        self.PADDLE_BL_BINS     = np.linspace(-2.5,2.5 ,     nbins)
        self.PADDLE_BLRMS_BINS  = np.linspace(0,2  ,     nbins)
        self.PADDLE_X0_BINS     = np.linspace(-0.1, 1.1, nbins)
        self.PADDLE_T0_BINS     = np.linspace(0,500,     nbins)
        self.PADDLE_EDEP_BINS   = np.linspace(0,100 ,     nbins)
        self.NHIT_BINS          = np.arange(-0.5,25.5,1)   
        self.PID_BINS           = np.arange(0.5,160.5,1)
        self.BETA_BINS          = np.linspace(0,2  ,     nbins)
        self.EDEP_BINS          = np.linspace(0,50,      nbins)
        self.TIMING_BINS        = np.linspace(-100, 300, nbins)
        self.PDELAY_BINS        = np.linspace(-60,60,    nbins)
        self.TDIFF_BINS         = np.linspace(-1, 10, nbins)
        self.DIST_BINS          = np.linspace(0,4, nbins)
        self.COS_T_BINS         = np.linspace(0,1,nbins)
        self.COS_T2_BINS        = np.linspace(0,1,int(nbins/3))
        self.X_BINS_OUTER       = np.linspace(-2000,2000,nbins)
        self.Y_BINS_OUTER       = np.linspace(-2000,2000,nbins)
        self.Z_BINS_OUTER       = np.linspace(-250, 2500,nbins)
        self.X_BINS_INNER       = np.linspace(-1000,1000,nbins)
        self.Y_BINS_INNER       = np.linspace(-1000,1000,nbins)
        self.Z_BINS_INNER       = np.linspace(-250, 1500,nbins)
        self.PID_BINS_INNER     = np.arange(0.5, 60.5, 1)
        self.PID_BINS_OUTER     = np.arange(61.5, 161.5,1)

    def pretty_print_statistics(self):
        """
        A textual representation for some important numbers, e.g.
        seen events, cut efficiencies, etc.
        """
        _repr = "\n-- -- -- -- -- -- -- -- -- "
        _repr += "\n TOF analysis statistics"
        if not self.cuts.void:
            _repr += f'\n  -- nevents (no    cut)      : {self.cuts.nevents}'
        else:
            _repr += f'\n  -- nevents                  : {self.n_events}'
        _repr += f'\n  -- runtime (s)              : {self.run_time:1f} s'
        _repr += f'\n  -- rate    (Hz (nocut))     : {self.rate_nocut:.2f} Hz'
        _repr += f'\n  -- frac. of mangled events  : {100*self.n_mangled_frac : .2f} %'
        _repr += f'\n  -- frac. of timedout events : {100*self.n_timed_out_frac : .2f} %' 
        if not self.cuts.void:
            _repr += '\n  -- -- applied cut:'
            _repr += f'\n\t -- -- {self.cuts}'
            _repr += f'\n\t  -- -- nevents (after cut) : {self.n_events}'
            if self.cuts.nevents > 0:
                _repr += f'\n\t  -- -- efficiency          : {100*self.n_events/self.cuts.nevents : .2f} %'
            _repr += f'\n\t  -- -- rate    (Hz)        : {self.rate: .2f} Hz'
            
        return _repr

    def _timing_plots(self):
        # first tuple argument is the histogram, second the cache, 
        # since hist1d.fill takes a long time
        tmg_plots = {
          'beta'         : d.histogram.hist1d(self.BETA_BINS),
          't_inner'      : d.histogram.hist1d(self.TIMING_BINS),
          't_outer'      : d.histogram.hist1d(self.TIMING_BINS),
          # timing difference will not be larger than phase delay!
          't_diff'       : d.histogram.hist1d(self.TDIFF_BINS),
          'ph_delay'     : d.histogram.hist1d(self.PDELAY_BINS),
          'dist'         : d.histogram.hist1d(self.DIST_BINS),
          'dist_vs_beta' : d.histogram.hist2d((self.DIST_BINS, self.BETA_BINS)),
          'dist_vs_tdiff': d.histogram.hist2d((self.DIST_BINS, self.TDIFF_BINS)),
          'cos_theta'    : d.histogram.hist1d(self.COS_T_BINS), 
          'cos2_theta'   : d.histogram.hist1d(self.COS_T2_BINS),
          'x_outer'      : d.histogram.hist1d(self.X_BINS_OUTER),
          'y_outer'      : d.histogram.hist1d(self.Y_BINS_OUTER),
          'z_outer'      : d.histogram.hist1d(self.Z_BINS_OUTER),
          'x_inner'      : d.histogram.hist1d(self.X_BINS_INNER),
          'y_inner'      : d.histogram.hist1d(self.Y_BINS_INNER),
          'z_inner'      : d.histogram.hist1d(self.Z_BINS_INNER),
          'pid_inner'    : d.histogram.hist1d(self.PID_BINS_INNER),
          'pid_outer'    : d.histogram.hist1d(self.PID_BINS_OUTER),
          'beta_vs_theta': d.histogram.hist2d((self.BETA_BINS, self.COS_T_BINS)),
        }
        tmg_cache = dict()
        for k in tmg_plots.keys():
            tmg_cache[k] = []
        return tmg_plots, tmg_cache

    def _edep_plots(self):
        plots = {
          # total energy depostion
          'edep'         : d.histogram.hist1d(self.EDEP_BINS)
        }
        for k in range(1,22):
            plots[f'edep_pnl{k}'] = d.histogram.hist1d(self.EDEP_BINS)
        cache = dict()
        for k in plots.keys():
            cache[k] = []
        return plots, cache


    def _nhit_plots(self):
        nhit_plots = {
          'hit'      : d.histogram.hist1d(self.NHIT_BINS),
          'thit'     : d.histogram.hist1d(self.NHIT_BINS),
          'rblink'   : d.histogram.hist1d(self.NHIT_BINS),
          'miss_hit' : d.histogram.hist1d(self.PID_BINS),
          # these are non causal hits
          'nc_pdls'  : d.histogram.hist1d(self.PID_BINS),
        }
        return nhit_plots

    def _paddle_plots(self):
        """
        Charge and timing plots for each paddle
        """
        paddle_plots = {\
          # use cache, cache, histogram
          # explanation - in general, the cache won't be needed for 2d histograms which are 
          # created from other histograms
          'charge2d'  : d.histogram.hist2d((self.PADDLE_CHARGE_BINS, self.PADDLE_CHARGE_BINS)),
          'amp_a'     : d.histogram.hist1d(self.PADDLE_PEAK_BINS),
          'amp_b'     : d.histogram.hist1d(self.PADDLE_PEAK_BINS),
          'charge_a'  : d.histogram.hist1d(self.PADDLE_CHARGE_BINS),
          'charge_b'  : d.histogram.hist1d(self.PADDLE_CHARGE_BINS),
          'time_a'    : d.histogram.hist1d(self.PADDLE_TIMING_BINS),
          'time_b'    : d.histogram.hist1d(self.PADDLE_TIMING_BINS),
          'bl_a'      : d.histogram.hist1d(self.PADDLE_BL_BINS),
          'bl_b'      : d.histogram.hist1d(self.PADDLE_BL_BINS),
          'bl_a_rms'  : d.histogram.hist1d(self.PADDLE_BLRMS_BINS),
          'bl_b_rms'  : d.histogram.hist1d(self.PADDLE_BLRMS_BINS),
          'x0'        : d.histogram.hist1d(self.PADDLE_X0_BINS),
          't0'        : d.histogram.hist1d(self.PADDLE_T0_BINS),
          'edep'      : d.histogram.hist1d(self.PADDLE_EDEP_BINS),
          'pos_edep'  : d.histogram.hist2d((self.PADDLE_X0_BINS, self.PADDLE_EDEP_BINS))
        }
        all_paddle_plots = {k : copy(paddle_plots) for k in range(161)}
        paddle_caches = {k : dict() for k in range(161)}
        for pid in range(161):
            for k in paddle_plots.keys():
                paddle_caches[pid][k] = []
        return all_paddle_plots, paddle_caches

    @property
    def rate(self):
        if self.run_time == 0:
            return 0
        return self.n_events / self.run_time

    @property
    def rate_nocut(self):
        if self.run_time == 0:
            return 0
        if self.cuts.void:
            return self.n_events / self.run_time
        return self.cuts.nevents / self.run_time

    @property
    def run_time(self):
        """
        Get run time from last - first event in seconds
        """
        #print (f'LAST EV TIME  {self.last_ev_time}')
        #print (f'FIRST EV TIME {self.first_ev_time}')
        return 1e-5*(self.last_ev_time - self.first_ev_time)

    def reinit(self, nbins = 90):
        """
        Re-run the initialization routine. This will clear all plots, and 
        reset the binning. This needs to be run in case the binning has
        been changed
        """
        self.__init__(skip_mangled  = self.skip_mangled,
                      skip_timeout  = self.skip_timeout,
                      beta_analysis = self.beta_analysis,
                      nbins         = nbins)

    def __init__(self, skip_mangled = True,\
                 skip_timeout    = True,\
                 beta_analysis   = True,\
                 nbins           = 90,
                 cuts : TofCuts  = TofCuts(),
                 use_offsets     = False,
                 pid_inner       = None,
                 pid_outer       = None,
                 active          = False):
        """
        Start a new TofAnalysis. This will add create histograms for 
        'interesting' variables and count mangled and timed out 
        events. While not complete, this can provide a conciese, 
        first look for a run.
        Events can be added to this analysis through the .add_event(ev)
        method. When all events are added, a call to .finish() is needed
        to make sure all events in the caches are added to the histograms.
        Caching is used to massively improve performance, since adding
        individual numbers to dashi.histograms is painfully slow.
        
        # Arguments:
        
          skip_mangled             : Ignore events which have the "AnyDataMangling" 
                                     flag set
          skip_timeout             : Ignore events which have the "EventTimedOut"
                                     flag set
          beta_analysis            : Look for first hit on outer tof/inner tof and 
                                     use these for a beta calculation. If pid_outer
                                     and pid_inner are given, use these paddles 
                                     instead.
          nbins                    : The number of bins for the histograms getting
                                     created
          cuts                     : Give a cut instance to reject events & hits.
                                     Default: None (no cuts)
          pid_outer                : Select a specific paddle instead of the first on the outer TOF 
                                     for the beta/timing analysis
          pid_inner                : Select a specific paddle instead of the first on the inner TOF
                                     for the beta/timing analysis
          active                   : if True, this analysis will actually "do something"
                                     and acquire events
        """
        # process kwargs
        self.skip_mangled              = skip_mangled
        self.skip_timeout              = skip_timeout
        self.beta_analysis             = beta_analysis
        self.use_offsets               = use_offsets
        self.nbins                     = nbins
        self.cuts                      = cuts
        self.active                    = active 
        self.offsets     = None
        if self.use_offsets:
            self.offsets = dict()
            offsets = json.load(open('offsets.json'))
            for k in offsets:
                k_int = int(k)
                # currently we only have intra-panel calibrations
                if k_int <= 12:
                    self.offsets[k_int] = offsets[k]
        
        self.define_bins(nbins = self.nbins)
        self.first_ev_time = np.inf
        self.last_ev_time  = 0
        self.finished      = False
        self.n_mangled     = 0
        self.n_timed_out   = 0
        self.n_events      = 0
        pp_hist, pp_cache  = self._paddle_plots()
        self.paddle_plots  = pp_hist
        self.paddle_cache  = pp_cache
        self.nhit_plots    = self._nhit_plots()
        tmg_plots, tmg_cache = self._timing_plots()
        self.tmg_plots     = tmg_plots
        self.tmg_cache     = tmg_cache
        edep_plots, edep_cache = self._edep_plots()
        self.edep_plots    = edep_plots
        self.edep_cache    = edep_cache
        self.paddles       = get_tof_paddles()
        self.hg_mapping    = create_mtb_connection_to_pid_map()
        # hit histogram
        self.nhit          = 0
        self.no_hitmiss    = 0
        self.one_hitmiss   = 0
        self.two_hitmiss   = 0
        self.extra_hits    = 0
        self.occupancy     = {k : 0 for k in range(1,161)}
        self.occupancy_t   = {k : 0 for k in range(1,161)}
        # beta analysis
        # select specific paddles for beta 
        self.pid_inner     = pid_inner
        self.pid_outer     = pid_outer
        #######################################################
        # caches - filling dashi histograms is very slow 
        # (it is not made for it). Work around that by only 
        # calling fill every 10000th event or so
        #######################################################
        # cache size for histograms with 1 entry/hit
        self.hit_cache_size     = int(1e6)
        # cache size for histograms with 1 entry/event
        self.event_cache_size   = int(1e6) 
        self.c_hit              = []
        self.c_thit             = []
        self.c_rblink           = []
        self.c_miss_hit         = []
        self.c_nc_pid           = []
   

    @property
    def n_mangled_frac(self):
        if self.n_events > 0:
            return self.n_mangled/self.n_events
        else:
            return 0

    @property
    def n_timed_out_frac(self):
        if self.n_events > 0:
            return self.n_timed_out/self.n_events
        else:
            return 0

    def _is_compatible(self, other):
        if self.beta_analysis != other.beta_analysis:
            return False
        if self.pid_inner != other.pid_inner:
            return False
        if self.pid_outer != other.pid_outer:
            return False
        if not self.cuts.is_compatible(other.cuts):
            return False
        return True

    def __iadd__(self, other):
        if not self._is_compatible(other):
            raise ValueError("Analysis are not compatible! Both must have the same setup!")
        if other.first_ev_time < self.first_ev_time:
            self.first_ev_time = other.first_ev_time
        if other.last_ev_time > self.last_ev_time:
            self.last_ev_time = other.last_ev_time
        self.cuts          += other.cuts
        self.skip_mangled  += other.skip_mangled
        self.skip_timeout  += other.skip_timeout
        self.n_mangled     += other.n_mangled 
        self.n_timed_out   += other.n_timed_out 
        self.n_events      += other.n_events
        for k in self.nhit_plots:
            self.nhit_plots[k] += other.nhit_plots[k]
        for k in self.tmg_plots:
            self.tmg_plots[k] += other.tmg_plots[k]
            self.tmg_cache[k].extend(other.tmg_cache[k])
        for k in self.edep_plots:
            self.edep_plots[k] += other.edep_plots[k]
            self.edep_cache[k].extend(other.edep_cache[k])

        # hit histogram
        self.nhit          += other.nhit 
        self.no_hitmiss    += other.no_hitmiss
        self.one_hitmiss   += other.one_hitmiss
        self.two_hitmiss   += other.two_hitmiss
        self.extra_hits    += other.extra_hits
        for pid in range(1,161):
            self.occupancy[pid]   += other.occupancy[pid] 
            self.occupancy_t[pid] += other.occupancy_t[pid] 
        # nhit plots
        self.c_hit             .extend(other.c_hit) 
        self.c_thit            .extend(other.c_thit) 
        self.c_rblink          .extend(other.c_rblink) 
        self.c_miss_hit        .extend(other.c_miss_hit) 
        self.c_nc_pid          .extend(other.c_nc_pid)
        
        # timing plots
        #self.c_t_inner         .extend(other.c_t_inner) 
        #self.c_t_outer         .extend(other.c_t_outer) 
        #self.c_t_diff          .extend(other.c_t_diff) 
        #self.c_ph_delay        .extend(other.c_ph_delay) 
        #self.c_dist            .extend(other.c_dist)
        #self.c_cos_theta       .extend(other.c_cos_theta)
        #self.c_x_outer         .extend(other.c_x_outer)
        #self.c_y_outer         .extend(other.c_y_outer)
        #self.c_z_outer         .extend(other.c_z_outer)
        #self.c_x_inner         .extend(other.c_x_inner)
        #self.c_y_inner         .extend(other.c_y_inner)
        #self.c_z_inner         .extend(other.c_z_inner)
        #self.c_pid_inner       .extend(other.c_pid_inner)
        #self.c_pid_outer       .extend(other.c_pid_outer)
        #self.c_beta            .extend(other.c_beta)
        # add paddle plots
        #for pid in range(1,161):
        #    for k in self.paddle_plots[pid]:
        #        self.paddle_plots[pid][k] += other.paddle_plots[pid][k]
        for pid in range(1,161):
            for k in self.paddle_plots[pid]:
                self.paddle_plots[pid][k] += other.paddle_plots[pid][k]
                self.paddle_cache[pid][k].extend(other.paddle_cache[pid][k])
        return self

    def __add__(self, other):
        new_analysis = TofAnalysis(skip_mangled = self.skip_mangled,
                                   skip_timeout = self.skip_timeout,
                                   beta_analysis= self.beta_analysis)
        new_analysis += self
        new_analysis += other
        return new_analysis

    def fill_histograms(self):
        """
        Fill the histograms with the cached values
        """
        if len(self.c_hit) >= self.event_cache_size: 
            self.nhit_plots['hit'     ].fill(np.array(self.c_hit)) 
            self.nhit_plots['thit'    ].fill(np.array(self.c_thit)) 
            self.nhit_plots['rblink'  ].fill(np.array(self.c_rblink)) 
            self.nhit_plots['miss_hit'].fill(np.array(self.c_miss_hit)) 
            self.nhit_plots['nc_pdls'] .fill(np.array(self.c_nc_pid))
            # nhit  
            self.c_hit     .clear()
            self.c_thit    .clear()
            self.c_rblink  .clear()
            self.c_miss_hit.clear()
            self.c_nc_pid  .clear()
            if self.beta_analysis:
                c_dist_vs_beta  = np.array([ k for k in zip(self.tmg_cache['dist'], self.tmg_cache['beta'])])
                c_dist_vs_tdiff = np.array([ k for k in zip(self.tmg_cache['dist'], self.tmg_cache['t_diff'])])
                c_beta_vs_theta = np.array([ k for k in zip(self.tmg_cache['beta'], self.tmg_cache['cos_theta'])])
                self.tmg_plots['dist_vs_beta'].fill(c_dist_vs_beta)
                self.tmg_plots['dist_vs_tdiff'].fill(c_dist_vs_tdiff)
                self.tmg_plots['beta_vs_theta'].fill(c_beta_vs_theta)
                for k in self.tmg_plots:
                    if k in ['dist_vs_beta', 'dist_vs_tdiff', 'beta_vs_theta']:
                        continue
                    self.tmg_plots[k].fill(np.array(self.tmg_cache[k]))
                    self.tmg_cache[k].clear()
                for k in self.edep_plots:
                    self.edep_plots[k].fill(np.array(self.edep_cache[k]))
                    self.edep_cache[k].clear()

        for paddle_id in range(1,161):
            for k in self.paddle_plots[paddle_id]:
                if len(self.paddle_cache[paddle_id][k]) >= self.hit_cache_size:
                    self.paddle_plots[paddle_id][k].fill(np.array(self.paddle_cache[paddle_id][k]))
                    self.paddle_cache[paddle_id][k].clear()

    def finish(self):
        """
        Ensure the remainder in the caches is histogrammed
        """
        if not self.active:
            return
        event_cache_size      = self.event_cache_size
        hit_cache_size        = self.hit_cache_size
        self.event_cache_size = 1
        self.hit_cache_size   = 1
        self.fill_histograms()
        # reset the cache sizes for the next run
        self.event_cache_size = event_cache_size
        self.hit_cache_size   = hit_cache_size
        self.finished = True

    def add_event(self, ev):
        """
        Fills the associated histograms
        
        # Arguments:
            * ev : Any kind of TofEvent or TofEventSummary
        """
        if not self.active:
            return

        if self.finished:
            print ("WARN: Analysis has been finished already. Not able to add more events.")
            return
        
        if self.first_ev_time == np.inf:
            self.first_ev_time = ev.timestamp48
        self.last_ev_time = ev.timestamp48
        if ev.status == EventStatus.AnyDataMangling:
            #logger.debug(f'Found mangled event with id {ev.event_id}')
            self.n_mangled += 1
            if self.skip_mangled:
                return
        if ev.status == EventStatus.EventTimeOut:
            #logger.debug(f'Found timed out event with id {ev.event_id}')
            self.n_timed_out += 1
            if self.skip_timeout:
                return
        # FIXME - speed these up
        nhit_ev        = 0
        nhit_t_ev      = 0
        self.n_events += 1
        # at the very first, add the timings if desired
        if self.use_offsets:
            ev.set_timing_offsets(self.offsets)
            #print (ev)


        # before cutting, calculate missing hits
        # the problem for removing hits right now
        # is the fact that if we do a hit cleaning,
        # it will be only for the HG hits and not 
        # the LG hits, so if we do a missing hit calculation 
        # after the hit cleaning, we will artificially 
        # increase the number of missing hits
        # FIXME - this is currently a bit inconsistent.
        missing        = [int(k) for k in ev.get_missing_paddles_hg(self.hg_mapping)]
        self.c_miss_hit.extend(missing)

        # since we might do hit cleaning, for now 
        # let's explicitly copy the event, see also
        # issue #82
        if not self.cuts.void:
            ev_for_cuts = ev.copy()
            if not self.cuts.accept(ev_for_cuts):
                return
        # if desired, apply the cleanings
        if self.cuts.only_causal_hits:
            rm_pids = ev.remove_non_causal_hits()
            self.c_nc_pid.extend(rm_pids)
            hits_rmvd_csl  = len(rm_pids)
        if self.cuts.ls_cleaning_t_err != np.inf:
            rm_pids = ev.lightspeed_cleaning(self.cuts.ls_cleaning_t_err)
            hits_rmvd_ls   = len(rm_pids)

        for h in ev.trigger_hits:
            pid = find_paddle(h, self.paddles.values())
            self.occupancy_t[pid] += 1
            nhit_t_ev += 1
        if self.beta_analysis:
            outer_h = []
            inner_h = []
        for h in ev.hits:
            pdl = self.paddles[h.paddle_id]
            h.set_paddle(10*pdl.length, pdl.cable_len, pdl.coax_cable_time, pdl.harting_cable_time)
            if pdl.panel_id < 22:
                edep_key = f'edep_pnl{pdl.panel_id}'
                self.edep_cache[edep_key].append(h.edep)
                self.edep_cache['edep'].append(h.edep)
            if h.edep > 0:
                self.occupancy[h.paddle_id] += 1
            nhit_ev += 1
            if self.beta_analysis:
                if self.pid_outer is None:
                    if h.paddle_id > 60:
                        outer_h.append(h)
                else:
                    if h.paddle_id == self.pid_outer:
                        outer_h.append(h)
                if self.pid_inner is None:
                    if h.paddle_id < 61:
                        inner_h.append(h)
                else:
                    if h.paddle_id == self.pid_inner:
                        inner_h.append(h)
            # fill the caches
            #if h.charge_a < 0 or h.charge_b < 0:
            #    print (h)
            #    raise ValueError
            self.paddle_cache[h.paddle_id]['charge2d'].append([h.charge_a, h.charge_b])
            self.paddle_cache[h.paddle_id]['amp_a']   .append(h.peak_a)
            self.paddle_cache[h.paddle_id]['amp_b']   .append(h.peak_b)
            self.paddle_cache[h.paddle_id]['time_a']  .append(h.time_a)
            self.paddle_cache[h.paddle_id]['time_b']  .append(h.time_b)
            self.paddle_cache[h.paddle_id]['charge_a'].append(h.charge_a)
            self.paddle_cache[h.paddle_id]['charge_b'].append(h.charge_b)
            self.paddle_cache[h.paddle_id]['bl_a']    .append(h.baseline_a)
            self.paddle_cache[h.paddle_id]['bl_b']    .append(h.baseline_b)
            self.paddle_cache[h.paddle_id]['bl_a_rms'].append(h.baseline_a_rms)
            self.paddle_cache[h.paddle_id]['bl_b_rms'].append(h.baseline_b_rms)
            self.paddle_cache[h.paddle_id]['x0']      .append(h.pos/h.paddle_len)
            self.paddle_cache[h.paddle_id]['t0']      .append(h.event_t0)
            self.paddle_cache[h.paddle_id]['edep']    .append(h.edep)
            self.paddle_cache[h.paddle_id]['pos_edep'].append([h.pos/h.paddle_len, h.edep])
        
        # hit counting 
        n_rblink_ev    = len(ev.rb_link_ids)
        self.nhit     += nhit_ev
        if nhit_t_ev == nhit_ev:
            self.no_hitmiss += 1 
        elif (nhit_t_ev - nhit_ev) == 1:
            self.one_hitmiss += 1
        elif (nhit_t_ev - nhit_ev) > 1:
            self.two_hitmiss += 1
        elif (nhit_ev > nhit_t_ev):
            self.extra_hits += 1
        
        self.c_hit.append(nhit_ev)
        self.c_thit.append(nhit_t_ev)
        self.c_rblink.append(n_rblink_ev)

        if not self.beta_analysis:
            return
        
        outer_h = sorted(outer_h, key=lambda x: x.event_t0)
        inner_h = sorted(inner_h, key=lambda x: x.event_t0)
        if inner_h and outer_h:
            #first_hit = sorted([h for h in ev.hits], key=lambda x: x.phase_delay)
            #last_hit  = first_hit[-1].phase_delay
            #first_hit = first_hit[0].phase_delay
            #print (inner_h, outer_h)
            diff_h  = inner_h[0].event_t0 - outer_h[0].event_t0 
            dist = distance(inner_h[0],outer_h[0])/1000
            cos_theta = abs(outer_h[0].z - inner_h[0].z)/(1000*dist)  
            beta = dist/(diff_h*1e-9)/299792458
            self.tmg_cache['dist']   .append(dist)
            self.tmg_cache['x_outer'].append(outer_h[0].x)
            self.tmg_cache['y_outer'].append(outer_h[0].y)
            self.tmg_cache['z_outer'].append(outer_h[0].z)
            self.tmg_cache['x_inner'].append(inner_h[0].x)
            self.tmg_cache['y_inner'].append(inner_h[0].y)
            self.tmg_cache['z_inner'].append(inner_h[0].z)
            self.tmg_cache['pid_inner'].append(inner_h[0].paddle_id)
            self.tmg_cache['pid_outer'].append(outer_h[0].paddle_id)
            self.tmg_cache['cos_theta'].append(cos_theta)
            self.tmg_cache['cos2_theta'].append(cos_theta*cos_theta)
            if beta < 0:
                beta = -1*beta
            self.tmg_cache['beta']    .append(beta)
            self.tmg_cache['t_outer'] .append(outer_h[0].event_t0)
            self.tmg_cache['t_inner'] .append(inner_h[0].event_t0)  
            self.tmg_cache['t_diff']  .append(inner_h[0].event_t0 - outer_h[0].event_t0)  
            self.tmg_cache['ph_delay'].append(inner_h[0].phase_delay - outer_h[0].phase_delay)
 
        # fill is the massive bottleneck here, thus let's try to reduce the amount of calls 
        self.fill_histograms()    
        return 

##################################################################

