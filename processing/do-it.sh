#! /bin/sh

# Run 9125
#rye run python merge_tcfc.py -o /data/stoessl/mcmurdo-campaign-processing/L0/ -r 9125 --telemetry-dir /data1/nextcloud/cra_data/data/2024/gse5/ethernet/ --tof-dir /data1/nextcloud/cra_data/data/2024/tof/ 
rye run python merge_tcfc.py --no-gps -o /data2/gaps/L0/ -r 9111 --telemetry-dir /data2/gaps/gcu --tof-dir /data1/gaps/mcmurdo/tof 

# Run 134
#rye run python merge_tcfc.py -r 134 -s 1722723121 -e 1722791305  --tof-dir  /data0/gaps/csbf/csbf-data -v --reprocess --telemetry-dir /data0/gaps/csbf/csbf-data/binaries/ethernet

# Run 30141
#rye run python merge_tcfc.py --reprocess -r 30141 -s 1723693723 -e 1723728058  --tof-dir  /data0/gaps/csbf/csbf-data -v --telemetry-dir /data0/gaps/csbf/csbf-data/binaries/ethernet
