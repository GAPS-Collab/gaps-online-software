#! /bin/sh 

#------------------------------------------
# This script assumes we have already run 
# over all the telemetry files and have 
# something which is bootstrapped in a 
# directory by individual run numbers 
# The individual files in these directories 
# shall already contain TelemetryEvents and 
# this will "vertically" stack more data to 
# the individual frames

# additional source to merge 
TELEMETRY_DIR="/data1/nextcloud/cra_data/data/binaries_berkeley/gcu_2_gcupool"
L0_RUN_DIR="/data/stoessl/flight/GAPSI/dataset/L0/gcu_2_gcupool/"
#L0_RUN_DIR="/data/stoessl/flight/GAPSI/dataset/L0/starlink"
#TELEMETRY_DIR="/data1/nextcloud/cra_data/data/binaries_berkeley/starlink"

# Run id we want to process (from input argument)
RUN_ID=$1

echo "--------------"
echo "-- starting processing for $1"
echo "RUN ID $RUN_ID"

## this creates new binary files with merged events 
uv run --isolated python l0-add-data-source.py --packet-tag Tracker --run-dir $L0_RUN_DIR/$RUN_ID --telemetry-dir $TELEMETRY_DIR 
