#! /bin/sh 

#------------------------------------------
# This script assumes we have already run 
# over all the telemetry files and have 
# something which is bootstrapped in a 
# directory by individual run numbers 
#

# Directory for new binary files
BIN_REMERGED_DIR=""
# Directory with disk data which has wastie hits 
BIN_WASTIE_DIR=""
# Directory with L0 files with telemetry data 
L0_RUN_DIR=""
# Directory for L0 files which also have wastie hits 
L0_WASTIE_OUTDIR=""
# Run id we want to process (from input argument)
RUN_ID=$1

# make a directory for the output 
mkdir -p $L0_WASTIE_OUTDIR/$RUN_ID

# this creates new binary files with merged events 
uv run add-wastie-hits.py --run-dir $L0_RUN_DIR/$RUN_ID --wastie-dir $BIN_WASTIE_DIR -o $L0_WASTIE_OUTDIR/$RUN_ID
uv run python cs2tel.py -o $BIN_REMERGED_DIR --add-wastie  --run-dir $L0_WASTIE_OUTDIR/$RUN_ID

#uv run python tracker-checks.py --telemetry-dir /data-ssd0/L0/binaries-remerged-wastie   
