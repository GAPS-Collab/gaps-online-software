#!/bin/bash

# Define Unicode symbols
responded_symbol="✓"
no_response_symbol="❌"

# Define the IP address range
start_ip="10.0.1.101"
end_ip="10.0.1.146" # inclusive

declare -i n_boards=0

# Loop through the IP addresses and ping each one
echo -e "Pinging all RBs in range $start_ip - $end_ip"
echo -e "================================================"
for ip in $(seq -f "10.0.1.%g" $(echo $start_ip | cut -d'.' -f4) $(echo $end_ip | cut -d'.' -f4)); do
    packet_loss=$(ping -c 1 -W 1 "$ip" | awk '/packet loss/ {print $6}')

    # Check the packet loss value and determine the status
    if [ "$packet_loss" == "100%" ]; then
        status="$no_response_symbol"
        case $ip in
	  "10.0.1.110")
	  continue
	  #status="$no_response_symbol (expected, does not exist)"
	  ;;
	  "10.0.1.112")
	  continue
	  #status="$no_response_symbol (expected, does not exist)"
	  ;;
	  "10.0.1.137")
	  #status="$no_response_symbol (expected, does not exist)"
	  continue
	  ;;
	  "10.0.1.138")
	  #status="$no_response_symbol (expected, does not exist)"
	  continue
	  ;;
	  "10.0.1.143")
	  #status="$no_response_symbol (expected, does not exist)"
	  continue
	  ;;
	  "10.0.1.145")
	  continue
	  #status="$no_response_symbol (expected, does not exist)"
	  ;;
	  * )
          status="$no_response_symbol (DOWN!)"  
	esac 
    else
	status="$responded_symbol";
	#n_boards=$((n_boards+1))
	n_boards+=1
    fi
    # Output the result in a table format
    echo -e "-- $ip\t$status"
done 
echo "==================================="
if [ $n_boards -lt 40 ]; then
    echo -e "=> \033[31m[WARNING] only $n_boards of 40 boards responded to ping 😔\033[0m"
else
    echo -e "=> \033[34mAll boards have responded to ping 🤩 \033[0m"
fi

