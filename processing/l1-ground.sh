#! /bin/sh 

#------------------------------------------
# --- produces .root files for further 
# --- processing (reconstruction) 


#if [ -n "$VARIABLE" ]; then
#    echo "Variable is not empty"
#fi

SCRATCH_DIR="/data/stoessl/scratch"

## Setup local scratch paths on the compute node
LOCAL_CACHE="$SCRATCH_DIR/.uv_cache"
LOCAL_TMP="$SCRATCH_DIR/.tmp"
mkdir -p "$LOCAL_CACHE" "$LOCAL_TMP"

export TMPDIR="$LOCAL_TMP"

L1_OUTDIR="/data/stoessl/flight/GAPSI/dataset/L1/ground/"
L0_DIR="/data/stoessl/flight/GAPSI/dataset/L0/ground" 

echo "--------------------------------"
echo "-- starting processing for $1"
echo "-- ** GROUND DATA **"

uv run --isolated --cache-dir "$LOCAL_CACHE" python l1processing.py --remove-cmn --input-dir $L0_DIR/$1 -o $L1_OUTDIR --quiet --ground --run-id $1

