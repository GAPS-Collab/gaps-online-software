"""
Python API for gaps_online_software. Extended
pybdindings for the C++ and Rust API. 

- tof-dataclasses
- plotting


"""
try:
    import django
    django.setup()
    from . import db
except Exception as e:
    print(f"Can't load django environment, gaps_db will not be available. {e}")

try:
    import gaps_tof as cxx_api
except ImportError as e:
    print(f"Can't load CXX API! {e}")
try:
    import rpy_tof_dataclasses as rust_api
except ImportError as e:
    print(f"Can't load RUST API! {e}")

try:
    import rust_telemetry as telemetry
except ImportError as e:
    print(f"Can't load RUST TELEMETRY API {e}")

from . import tof
from . import io
from . import events

__version__ = "0.10"
