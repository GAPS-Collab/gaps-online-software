# GONDOLA - python wrapper for gaps-online-software 

This allows to easily access various dataclasses used throughout 
the [GAPS experiment](https://gaps1.astro.ucla.edu/gaps/)

# CHANGELOG 

* v0.11 is the first version to be published through pypi
* Database system: switch from django to pure-rust implementation with diesel + pyo3
    This gives a performance boost by 3x for `get_tofpaddles()`:
    Rust : 500mic sec, django 1.5 milli sec
* The caraspace reader is now optimized to not copy paddle information every time 
  it is emiiting a CRFrame, thus increasing performance by a huge amount  



