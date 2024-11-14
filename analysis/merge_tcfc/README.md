# merge-tcfc

Merges the separate stream from the TOF computer with the telemetry binary stream 
utilizing the caraspace serialization libary.


## dependencies

Make sure the flags `BUILD_PYBINDINGS`, `BUILD_CARASPACE` and `BUILD_TELEMETRY` are switched on 
in your build.

For the python dependencies, a `pyproject.toml` compatible with `rye` is provided.

### Example

To do a merge for a single run, do something like this
```rye run python merge_tcfc.py -r 134 -s 1722723121 -e 1722791305  --tof-dir /data0/gaps/csbf/csbf-data/ ```
