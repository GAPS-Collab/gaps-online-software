#! /bin/sh 

#------------------------------------------
# This script assumes we have already run 
# over all the telemetry files and have 
# something which is bootstrapped in a 
# directory by individual run numbers 
#

# Run id we want to process (from input argument)
RUN_ID=$1
L0_RUN_DIR="/data/stoessl/flight/GAPSI/dataset/L0/ground"
TELEMETRY_DIR="/data/stoessl/flight/GAPSI/telemetry-ethernet-ground"
echo "--------------"
echo "-- starting processing for $1"
echo "RUN ID $RUN_ID"

uv run --isolated python bootstrapl0.py -o $L0_RUN_DIR -t $TELEMETRY_DIR --only-run $RUN_ID --ground

