#! /bin/sh 

#------------------------------------------
# This script assumes we have already run 
# over all the telemetry files and have 
# something which is bootstrapped in a 
# directory by individual run numbers 
#

# Directory for new binary files
BIN_REMERGED_DIR="/data/stoessl/flight/L0-remerged-bin-plus-wastie-NEW"
# Directory with disk data which has wastie hits 
BIN_WASTIE_DIR="/data1/nextcloud/cra_data/data/binaries_berkeley/auxgcu/disk_a/"
# Directory with L0 files with telemetry data 
L0_RUN_DIR="/data/stoessl/flight/L0-remerged/"
# Directory for L0 files which also have wastie hits 
L0_WASTIE_OUTDIR="/data/stoessl/flight/L0-remerged-plus-wastie-NEW"
# Run id we want to process (from input argument)
RUN_ID=$1

echo "--------------"
echo "-- starting processing for $1"
echo "RUN ID $RUN_ID"

## make a directory for the output 
echo "-- creating $L0_WASTIE_OUTDIR/$RUN_ID"
mkdir -p $L0_WASTIE_OUTDIR/$RUN_ID
echo "-- making sure $BIN_REMERGED_DIR exists..."
mkdir -p $BIN_REMERGED_DIR 
echo "--------------"
echo "-- begin with add-wastie-hits "

## this creates new binary files with merged events 
uv run --isolated python add-wastie-hits.py --run-dir $L0_RUN_DIR/$RUN_ID --wastie-dir $BIN_WASTIE_DIR -o $L0_WASTIE_OUTDIR/$RUN_ID
uv run --isolated python cs2tel.py -o $BIN_REMERGED_DIR --add-wastie  --run-dir $L0_WASTIE_OUTDIR/$RUN_ID
#
##uv run python tracker-checks.py --telemetry-dir /data-ssd0/L0/binaries-remerged-wastie   
