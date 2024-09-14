import_success = False

try:
    import rust_telemetry as tl
    import_success = True
except ImportError as e:
    print('--> gaps_online.events reports ImportError {e}')
    print('--> Pybindings for telemetry-dataclasses are missing!')
    print('--> Limited functionality available only!')
    print('--> If you want to mitigate this, check your build, e.g. with ccmake and make sure the respective features are turned ON')

if import_success:
    MergedEvent = tl.MergedEvent