"""
Tof command & control. This provides basically 3 sets of 
functionality:

* configs    : Allow to configure the .toml settings file.
               The rationale behind configs is that these 
               are a highly compressible representation of 
               config files shards, but this makes them 
               easy to be sent over flight telemetry
* commands   : Command infrastructure. THis provides a TofCommand,
               together with a command code which allows to trigger
               different reactions remotely
* heartbeats : These are liftof-cc specific montioring containers, 
               providing valuable information about running program
               health 
               FIXME: They will be moved to 'monitoring' in the future

"""

import_rtd_success = False
try:
    import go_pybindings as rtd
    import_rtd_success = True
except ImportError as e:
    print(f'gaps_online.commands reports ImportError {e}')
    print('--> Pybindings for tof-dataclasses are missing!')
    print('--> Limited functionality available only!')
    print('--> If you want to mitigate this, check your build, e.g. with ccmake and make sure the respective features are turned ON')

if import_rtd_success:
    TofCommand            = rtd.commands.TofCommand
    TofCommandCode        = rtd.commands.TofCommandCode
    # configs
    AnalysisEngineConfig  = rtd.commands.AnalysisEngineConfig
    TOFEventBuilderConfig = rtd.commands.TOFEventBuilderConfig
    DataPublisherConfig   = rtd.commands.DataPublisherConfig
    TriggerConfig         = rtd.commands.TriggerConfig
    TofRunConfig          = rtd.commands.TofRunConfig
    TofRBConfig           = rtd.commands.TofRBConfig
    BuildStrategy         = rtd.commands.BuildStrategy
    # Heartbeats
    EVTBLDRHeartbeat      = rtd.commands.EVTBLDRHeartbeat
    HeartbeatDataSink     = rtd.commands.HeartbeatDataSink
    MTBHeartbeat          = rtd.commands.MTBHeartbeat
    # command factory
    factory               = rtd.commands.factory
    
