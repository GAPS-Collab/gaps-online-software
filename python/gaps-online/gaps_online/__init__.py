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
    import os
    os.environ['DJANGO_ALLOW_ASYNC_UNSAFE'] = '1'

except Exception as e:
    print(f"Can't load django environment, gaps_db will not be available. {e}")

try:
    import gaps_tof as cxx_api
except ImportError as e:
    print(f"Can't load CXX API! {e}")
try:
    import go_pybindings as rust_api
    try:
        rust_api.liftof
        liftof = rust_api.liftof
    except Exception as e:
        print (e)

except ImportError as e:
    print(f"Can't load RUST API! {e}")

# FIXME
from . import tof
from . import io
from . import events
from . import run

__version__ = "0.10"
