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
    AnalysisEngineConfig  = rtd.commands.AnalysisEngineConfig
    EVTBLDRHeartbeat      = rtd.commands.EVTBLDRHeartbeat
    HeartbeatDataSink     = rtd.commands.HeartbeatDataSink
    MTBHeartbeat          = rtd.commands.MTBHeartbeat
    TofCommand            = rtd.commands.TofCommand
    TOFEventBuilderConfig = rtd.commands.TOFEventBuilderConfig
    TriggerConfig         = rtd.commands.TriggerConfig
