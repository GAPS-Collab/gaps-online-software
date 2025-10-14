"""
TOF related entities which did not fit anywhere else 
"""

from .. import _gondola_core  as _gc 

from . import analysis

RBPaddleID            = _gc.tof.RBPaddleID
RBPaddleID.__module__ = __name__
RBPaddleID.__name__   = "RBPaddleID"
#RBPaddleID.__doc__    = _gc.tof.RBPaddleID.__doc__
TofDetectorStatus   = _gc.tof.TofDetectorStatus
TofCommandCode      = _gc.tof.TofCommandCode
TofCommand          = _gc.tof.TofCommand
TofOperationMode    = _gc.tof.TofOperationMode
BuildStrategy       = _gc.tof.BuildStrategy
PreampBiasConfig    = _gc.tof.PreampBiasConfig
RBChannelMaskConfig = _gc.tof.RBChannelMaskConfig
TriggerConfig       = _gc.tof.TriggerConfig
TofRunConfig        = _gc.tof.TofRunConfig
TofCuts             = _gc.tof.TofCuts
to_board_id_string  = _gc.tof.to_board_id_string
TofAnalysis         = analysis.TofAnalysis

## command factories
#start_run            = _gc.tof.start_run 
#start_run.__module__ = __name__
#start_run.__name___  = 'start_run'
#
#stop_run            = _gc.tof.stop_run 
#stop_run.__module__ = __name__
#stop_run.__name___  = 'stop_run'
#
#enable_verification_run            = _gc.tof.enable_verification_run  
#enable_verification_run.__module__ = __name__
#enable_verification_run.__name___  = 'enable_verification_run'
#
#restart_liftofrb    = _gc.tof.restart_liftofrb 
#restart_liftofrb.___module__ = __name__ 
#restart_liftofrb.__name__ = 'restart_liftofrb'
#
#shutdown_all_rbs    = _gc.tof.shutdown_all_rbs
#shutdown_all_rbs.__module__ = __name__
#shutdown_all_rbs.__name__   = 'shutdown_all_rbs'
#
#shutdown_rat        = _gc.tof.shutdown_rat 
#shutdown_rat.__module__ = __name__ 
#shutdown_rat.__name__   = 'shutdown_rat' 
#
#shutdown_ratpair    = _gc.tof.shutdown_ratpair
#shutdown_ratpair.__module__ = __name__ 
#shutdown_ratpair.__name__ = 'shutdown_ratpair' 
#
#shutdown_rb         = _gc.tof.shutdown_rb 
#shutdown_rb.__module__  = __name__
#shutdown_rb.__name__    = 'shutdown_rb' 
#
#shutdown_tofcpu     = _gc.tof.shutdown_tofcpu 
#shutdown_tofcpu.__module__ = __name__ 
#shutdown_tofcpu.__name__   = 'shutdown_tofcpu'
#


