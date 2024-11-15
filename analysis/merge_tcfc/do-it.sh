#! /bin/sh

# Run 134
#rye run python merge_tcfc.py -r 134 -s 1722723121 -e 1722791305  --tof-dir  /data0/gaps/csbf/csbf-data -v --reprocess --telemetry-dir /data0/gaps/csbf/csbf-data/binaries/ethernet

# Run 30141
rye run python merge_tcfc.py --reprocess -r 30141 -s 1723693723 -e 1723728058  --tof-dir  /data0/gaps/csbf/csbf-data -v --telemetry-dir /data0/gaps/csbf/csbf-data/binaries/ethernet
