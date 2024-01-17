#!/bin/bash

# Define Unicode symbols
responded_symbol="✓"
no_response_symbol="❌"

# Define the IP address range
start_ip="10.0.1.101"
end_ip="10.0.1.151"

# Loop through the IP addresses and ping each one
echo -e "Pinging all RBs in range 10.0.1.101 - 10.0.1.151"
echo -e "================================================"
for ip in $(seq -f "10.0.1.%g" $(echo $start_ip | cut -d'.' -f4) $(echo $end_ip | cut -d'.' -f4)); do
    packet_loss=$(ping -c 1 -W 1 "$ip" | awk '/packet loss/ {print $6}')

    # Check the packet loss value and determine the status
    if [ "$packet_loss" == "100%" ]; then
        status="$no_response_symbol"
    else
        status="$responded_symbol"
    fi

    # Output the result in a table format
    echo -e "\t -- $ip\t$status"
done | column -t
