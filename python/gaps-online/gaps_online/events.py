import_tl_success  = False
import_rtd_success = False
try:
    import go_pybindings
    tl = go_pybindings.telemetry
    import_tl_success = True
except ImportError as e:
    print('--> gaps_online.events reports ImportError {e}')
    print('--> Pybindings for telemetry-dataclasses are missing!')
    print('--> Limited functionality available only!')
    print('--> If you want to mitigate this, check your build, e.g. with ccmake and make sure the respective features are turned ON')

try:
    import go_pybindings as rtd
    import_rtd_success = True
except ImportError as e:
    print(f'gaps_online.events reports ImportError {e}')
    print('--> Pybindings for tof-dataclasses are missing!')
    print('--> Limited functionality available only!')
    print('--> If you want to mitigate this, check your build, e.g. with ccmake and make sure the respective features are turned ON')


if import_tl_success:
    MergedEvent = tl.MergedEvent

if import_rtd_success:
    TofEvent        = rtd.events.TofEvent
    TofEventSummary = rtd.events.TofEventSummary
    TofHit          = rtd.events.TofHit
    RBCalibration   = rtd.events.RBCalibration
    TriggerType     = rtd.events.TriggerType
