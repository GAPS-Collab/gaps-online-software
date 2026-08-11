#!/bin/bash

# --- Configuration ---

if [ "$(hostname)" = "valkyrie" ]; then
    echo "=> Executing on host $(hostname)"
    #START=10100
    START=10510
    END=10546
    CORES=1  # Change this to the number of parallel processes you want
    BASE_RUN_DIR="/data-ssd0/L0/new-try"
    TELEMETRY_DIR="/data-ssd0/flight-starlink"
else
    echo "=> Executing on host $(hostname)"
    START=10000
    END=10546 
    CORES=1  # Change this to the number of parallel processes you want
    BASE_RUN_DIR="/data/stoessl/flight/L0-remerged"
    TELEMETRY_DIR="/data1/nextcloud/cra_data/data/binaries_berkeley/starlink/"
fi

SCRIPT_PATH="l0processing-single-run.py"

# --- Execution ---

# We use seq to generate the numbers, then pipe them into GNU Parallel
# {%} is the job slot number, {} is the current number from the sequence
seq $START $END | parallel -j $CORES \
    uv run $SCRIPT_PATH \
    --telemetry-dir $TELEMETRY_DIR \
    --run-dir "$BASE_RUN_DIR/{}"
