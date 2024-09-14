import_tl_success  = False
import_rtd_success = False
try:
    import rust_telemetry as tl
    import_tl_success = True
except ImportError as e:
    print('--> gaps_online.events reports ImportError {e}')
    print('--> Pybindings for telemetry-dataclasses are missing!')
    print('--> Limited functionality available only!')
    print('--> If you want to mitigate this, check your build, e.g. with ccmake and make sure the respective features are turned ON')

try:
    import rpy_tof_dataclasses as rtd
    import_rtd_success = True
except ImportError as e:
    print(f'gaps_online.events reports ImportError {e}')
    print('--> Pybindings for tof-dataclasses are missing!')
    print('--> Limited functionality available only!')
    print('--> If you want to mitigate this, check your build, e.g. with ccmake and make sure the respective features are turned ON')


if import_tl_success:
    MergedEvent = tl.MergedEvent

if import_rtd_success:
    TofEvent = rtd.events.TofEvent