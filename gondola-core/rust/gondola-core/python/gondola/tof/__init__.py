"""
TOF related entities which did not fit anywhere else 
"""

from .. import _gondola_core  as _gc 

from . import analysis

__all__ = ['shutdown_tofcpu', 'shutdown_rb', 'shutdown_ratpair', 'shutdown_rat',\
'shutdown_all_rbs', 'restart_liftofrb', 'enable_verification_run', 'stop_run', \
'start_run', 'waveform_analysis', 'MasterTrigger', 'TofAnalysis', 'to_board_id_string',\
'TofCuts','TofRunConfig','TriggerConfig', 'RBChannelMaskConfig', 'PreampBiasConfig', \
'TofCommand', 'TofOperationMode', 'BuildStrategy', 'TofCommandCode', 'AnalysisEngineSettings', \
'TofDetectorStatus', 'RBPaddleID']

RBPaddleID            = _gc.tof.RBPaddleID
RBPaddleID.__module__ = __name__
RBPaddleID.__name__   = "RBPaddleID"
#RBPaddleID.__doc__    = _gc.tof.RBPaddleID.__doc__
TofDetectorStatus   = _gc.tof.TofDetectorStatus
TofDetectorStatus.__module__ = __name__ 
TofDetectorStatus.__name__   = 'TofDetectorStatus' 

AnalysisEngineSettings = _gc.tof.AnalysisEngineSettings 
AnalysisEngineSettings.__module__ = __name__ 
AnalysisEngineSettings.__name__   = 'AnalysisEngineSettings' 


# enums 
TofCommandCode      = _gc.tof.TofCommandCode
BuildStrategy       = _gc.tof.BuildStrategy
TofOperationMode    = _gc.tof.TofOperationMode

TofCommand          = _gc.tof.TofCommand
TofCommand.__module__ = __name__ 
TofCommand.__name__   = "TofCommand" 

PreampBiasConfig    = _gc.tof.PreampBiasConfig
PreampBiasConfig.__module__ = __name__ 
PreampBiasConfig.__name__ = "PreampBiasConfig"

RBChannelMaskConfig = _gc.tof.RBChannelMaskConfig
RBChannelMaskConfig.__module__ = __name__ 
RBChannelMaskConfig.__name__   = "RBChannelMaskConfig"

TriggerConfig       = _gc.tof.TriggerConfig
TriggerConfig.__module__ = __name__ 
TriggerConfig.__name__   = "TriggerConfig" 

TofRunConfig        = _gc.tof.TofRunConfig
TofRunConfig.__module__ = __name__ 
TofRunConfig.__name__   = "TofRunConfig" 


waveform_analysis   = _gc.tof.waveform_analysis 

TofCuts             = _gc.tof.TofCuts
TofCuts.__module__  = __name__ 
TofCuts.__name__    = 'TofCuts' 

to_board_id_string  = _gc.tof.to_board_id_string
TofAnalysis         = analysis.TofAnalysis

MasterTrigger       = _gc.tof.MasterTrigger 
MasterTrigger.__module__ = __name__
MasterTrigger.__name__   = 'MasterTrigger'


## command factories
start_run            = _gc.tof.start_run 
#start_run.__module__ = __name__
#start_run.__name___  = 'start_run'
#
stop_run            = _gc.tof.stop_run 
#stop_run.__module__ = __name__
#stop_run.__name___  = 'stop_run'
#
enable_verification_run            = _gc.tof.enable_verification_run  
#enable_verification_run.__module__ = __name__
#enable_verification_run.__name___  = 'enable_verification_run'
#
restart_liftofrb    = _gc.tof.restart_liftofrb 
#restart_liftofrb.___module__ = __name__ 
#restart_liftofrb.__name__ = 'restart_liftofrb'
#
shutdown_all_rbs    = _gc.tof.shutdown_all_rbs
#shutdown_all_rbs.__module__ = __name__
#shutdown_all_rbs.__name__   = 'shutdown_all_rbs'
#
shutdown_rat        = _gc.tof.shutdown_rat 
#shutdown_rat.__module__ = __name__ 
#shutdown_rat.__name__   = 'shutdown_rat' 
#
shutdown_ratpair    = _gc.tof.shutdown_ratpair
#shutdown_ratpair.__module__ = __name__ 
#shutdown_ratpair.__name__ = 'shutdown_ratpair' 
#
shutdown_rb         = _gc.tof.shutdown_rb 
#shutdown_rb.__module__  = __name__
#shutdown_rb.__name__    = 'shutdown_rb' 
#
shutdown_tofcpu     = _gc.tof.shutdown_tofcpu 
#shutdown_tofcpu.__module__ = __name__ 
#shutdown_tofcpu.__name__   = 'shutdown_tofcpu'
#
run_action_alfa     = _gc.tof.run_action_alfa 
run_action_bravo    = _gc.tof.run_action_bravo 
run_action_charlie  = _gc.tof.run_action_charlie 
run_action_whiskey  = _gc.tof.run_action_whiskey 
run_action_tango    = _gc.tof.run_action_tango 
run_action_foxtrott = _gc.tof.run_action_foxtrott
request_liftof_setitngs = _gc.tof.request_liftof_settings
apply_settings_diff     = _gc.tof.apply_settings_diff
restore_default_config  = _gc.tof.restore_default_config
