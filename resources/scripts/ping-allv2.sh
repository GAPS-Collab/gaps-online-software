#!/bin/bash

# Define Unicode symbols
responded_symbol="✓"
no_response_symbol="❌"

# Define the IP address range
start_ip="10.0.1.101"
end_ip="10.0.1.146" # inclusive

declare -i n_boards=0
# Declare an associative array to store results: IP -> Status
declare -A results

# Define the list of IPs that are expected not to respond
# This simplifies the case/switch logic later
declare -a expected_down_ips=(
    "10.0.1.110"
    "10.0.1.112"
    "10.0.1.137"
    "10.0.1.138"
    "10.0.1.143"
    "10.0.1.145"
)

# Function to perform the ping and store the result
# This function is run in the background for each IP
ping_and_store() {
    local ip=$1
    local packet_loss=$(ping -c 1 -W 1 "$ip" | awk '/packet loss/ {print $6}')
    local status

    # Check the packet loss value and determine the status
    if [ "$packet_loss" == "100%" ]; then
        status="$no_response_symbol"
        
        # Check if the IP is in the list of expected down hosts
        local is_expected_down=0
        for expected_ip in "${expected_down_ips[@]}"; do
            if [[ "$ip" == "$expected_ip" ]]; then
                is_expected_down=1
                break
            fi
        done

        if [ "$is_expected_down" -eq 1 ]; then
            # We don't print "DOWN!" if it's an expected-down host, but still no_response_symbol
            status="$no_response_symbol (expected down)"
        else
            status="$no_response_symbol (DOWN!)"
        fi
    else
        status="$responded_symbol"
        # Increment the counter for boards that responded
        # We use a temporary file or global variable for a safe count in background processes
        # but for simplicity and to match your original logic, we'll count in the main thread
        # when we process the final results. For now, just store the status.
    fi

    # Store the result in a temporary file unique to this IP
    echo "$status" > "/tmp/ping_result_$ip"
}

## Main Script Execution ##
#-----------------------------------

echo -e "Pinging all RBs in range $start_ip - $end_ip (in parallel...)"
echo -e "================================================"

# Loop through the IP addresses and start pinging in the background
# The `seq` command generates the last octets, and we prepend '10.0.1.'
for last_octet in $(seq $(echo "$start_ip" | cut -d'.' -f4) $(echo "$end_ip" | cut -d'.' -f4)); do
    ip="10.0.1.$last_octet"
    # Call the function in the background
    ping_and_store "$ip" &
done

# Wait for all background jobs (pings) to complete
# This is crucial for parallel execution
wait

# Re-loop through the IPs to collect results and print the table
echo -e "IP Address\tStatus"
echo -e "-----------------------------------"
for last_octet in $(seq $(echo "$start_ip" | cut -d'.' -f4) $(echo "$end_ip" | cut -d'.' -f4)); do
    ip="10.0.1.$last_octet"
    # Read the status from the temporary file
    if [ -f "/tmp/ping_result_$ip" ]; then
        status=$(cat "/tmp/ping_result_$ip")
        
        # Count the boards that responded (status is just the '✓' symbol)
        if [[ "$status" == "$responded_symbol" ]]; then
            n_boards+=1
        fi
        
        # Output the result in a table format
        echo -e "$ip\t$status"
        
        # Clean up the temporary file
        rm "/tmp/ping_result_$ip"
    else
        # Fallback in case a ping failed to start or write a file (unlikely)
        echo -e "$ip\t$no_response_symbol (Error collecting result)"
    fi
done

#-----------------------------------
echo "==================================="
# Final summary report
if [ "$n_boards" -lt 40 ]; then
    echo -e "=> \033[31m[WARNING] only $n_boards of 40 boards responded to ping 😔\033[0m"
else
    echo -e "=> \033[34mAll boards have responded to ping 🤩 \033[0m"
fi
