# data processing scripts

This can deal with the different source, e.g. binary files as well as the 
TOF computer data stream and produce a number of data products.

## L0 files

L0 is the merge of ALL "raw" data - the telemetry files as well as the TOF stream
from the tof-cpu disks. This contains all existing data. These files are sometimes 
also dubbed "caraspace" files, since the library is called "caraspace".

## dependencies

Make sure the flags `BUILD_PYBINDINGS`, `BUILD_CARASPACE` and `BUILD_TELEMETRY` are switched on 
in your build.

For the python dependencies, a `pyproject.toml` compatible with `rye` is provided.

### Example

To do a merge for a single run, do something like this
```rye run python merge_tcfc.py -r 134 -s 1722723121 -e 1722791305  --tof-dir /data0/gaps/csbf/csbf-data/ ```
