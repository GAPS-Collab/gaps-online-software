#! /bin/sh 

#------------------------------------------
# This script assumes we have already run 
# over all the telemetry files and have 
# something which is bootstrapped in a 
# directory by individual run numbers 
#


#if [ -n "$_CONDOR_SCRATCH_DIR" ]; then
#  ## Setup local scratch paths on the compute node
#  LOCAL_CACHE="$_CONDOR_SCRATCH_DIR/.uv_cache"
#  LOCAL_TMP="$_CONDOR_SCRATCH_DIR/.tmp"
#  mkdir -p "$LOCAL_CACHE" "$LOCAL_TMP"
#  export TMPDIR="$LOCAL_TMP"
#else 
#  LOCAL_CACHE="/data/stoessl/"
#fi

LOCAL_CACHE="/data/stoessl/.cache"

# Directory with L0 files with telemetry data 
#L0_RUN_DIR="/data/stoessl/flight/GAPSI/dataset/L0/starlink"
L0_RUN_DIR="/data/stoessl/flight/GAPSI/dataset/L0/gcu_2_gcupool/"
# Direcotry with the initial set of telemetry files
TELEMETRY_DIR="/data1/nextcloud/cra_data/data/binaries_berkeley/gcu_2_gcupool"

# Run id we want to process (from input argument)
RUN_ID=$1

echo "--------------"
echo "-- starting processing for RUN $1"
echo "RUN ID $RUN_ID"

# bootstrap only
#uv run --isolated --cache-dir "$LOCAL_CACHE" python bootstrapl0.py -o $L0_RUN_DIR -t $TELEMETRY_DIR  --exclude-tracker-calib --bootstrap-only

# the actual processing in case we have the run meta files already
uv run --isolated --cache-dir "$LOCAL_CACHE" python bootstrapl0.py --quiet -o $L0_RUN_DIR -t $TELEMETRY_DIR --only-run $RUN_ID --reuse-existing

# no specific cache dir --- IGNORE
#uv run --isolated python bootstrapl0.py --quiet -o $L0_RUN_DIR --reuse-existing -t $TELEMETRY_DIR --only-run $RUN_ID

