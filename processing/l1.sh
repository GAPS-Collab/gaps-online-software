#! /bin/sh 

#------------------------------------------
# --- produces .root files for further 
# --- processing (reconstruction) 

SCRATCH_DIR="/data/stoessl/scratch"

## Setup local scratch paths on the compute node
LOCAL_CACHE="$SCRATCH_DIR/.uv_cache"
LOCAL_TMP="$SCRATCH_DIR/.tmp"
mkdir -p "$LOCAL_CACHE" "$LOCAL_TMP"

export TMPDIR="$LOCAL_TMP"

L1_OUTDIR="/data/stoessl/flight/GAPSI/dataset/L1/gcu_2_gcupool/"
L0_DIR="/data/stoessl/flight/GAPSI/dataset/L0/gcu_2_gcupool_auxgcu" 

echo "--------------"
echo "-- starting processing for $1"

## this creates new binary files with merged events 
uv run --isolated --cache-dir "$LOCAL_CACHE" python l1processing.py --remove-cmn --telemetry-dir $L0_DIR/$1 -o $L1_OUTDIR --quiet

