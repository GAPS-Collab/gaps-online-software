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


from . import tof

import gaps_tof as cxx_api
try:
    import rust_dataclasses as rust_api
except ImportError as e:
    print(f"Can't load RUST API! {e}")

__version__ = "0.9.x"
