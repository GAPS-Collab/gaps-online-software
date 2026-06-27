#! /bin/sh 

#------------------------------------------
# This script assumes we have already run 
# over all the telemetry files and have 
# something which is bootstrapped in a 
# directory by individual run numbers 
#

# Directory with L0 files with telemetry data 
#L0_RUN_DIR="/data/stoessl/flight/GAPSI/dataset/L0/starlink"
L0_RUN_DIR="/data/stoessl/flight/GAPSI/dataset/L0/gcu_2_gcupool/"
# Direcotry with the initial set of telemetry files
TELEMETRY_DIR="/data1/nextcloud/cra_data/data/binaries_berkeley/gcu_2_gcupool"
# Run id we want to process (from input argument)
RUN_ID=$1

echo "--------------"
echo "-- starting processing for $1"
echo "RUN ID $RUN_ID"

uv run --isolated python bootstrapl0.py --quiet -o $L0_RUN_DIR --reuse-existing -t $TELEMETRY_DIR --only-run $RUN_ID

