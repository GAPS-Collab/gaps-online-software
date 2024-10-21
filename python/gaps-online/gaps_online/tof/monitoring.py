try:
    import go_pybindings as tdc

    PAMoniSeries      = tdc.moni.PAMoniSeries
    PBMoniSeries      = tdc.moni.PBMoniSeries
    RBMoniSeries      = tdc.moni.RBMoniSeries
    MtbMoniSeries     = tdc.moni.MtbMoniSeries
    CPUMoniSeries     = tdc.moni.CPUMoniSeries
    LTBMoniSeries     = tdc.moni.LTBMoniSeries
    RBMoniData        = tdc.moni.RBMoniData
    MtbMoniData       = tdc.moni.MtbMoniData
    PAMoniData        = tdc.moni.PAMoniData
    PBMoniData        = tdc.moni.PBMoniData
    EVTBLDRHeartbeat  = tdc.commands.EVTBLDRHeartbeat
    MTBHeartbeat      = tdc.commands.MTBHeartbeat
    HeartbeatDataSink = tdc.commands.HeartbeatDataSink

except ImportError as e:
    print(f'gaps_online.tof.monitoring ImportError {e}')
    print('--> Pybindings for tof-dataclasses are missing!')
    print('--> Limited functionality available only!')
    print('--> If you want to mitigate this, check your build, e.g. with ccmake and make sure the respective features are turned ON')

