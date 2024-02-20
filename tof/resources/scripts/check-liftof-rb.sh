#!/bin/bash

# Command to be executed (start, stop, restart, status, reload)
COMMAND=$1

# Check if the command argument is provided
if [[ -z "$COMMAND" ]]; then
    echo "Usage: $0 [start|stop|status|restart|reload]"
    exit 1
fi

# Define the IP range
START_IP=101
END_IP=151
IP_PREFIX="10.0.1."

echo "Executing '$COMMAND' on the clients..."

# Initialize a variable to hold status results when needed
STATUS_RESULTS=""

# Loop through the IP range
for i in $(seq $START_IP $END_IP); do
    IP="${IP_PREFIX}${i}"

    echo "Processing $IP..."

    if [[ "$COMMAND" == "status" ]]; then
        # Execute the systemctl status command and parse the output
        RESULT=$(ssh $IP "systemctl status liftof" | grep -E 'Active:|Loaded:')
        
        # Append the result to STATUS_RESULTS
        STATUS_RESULTS+="$IP: $RESULT\n"
    elif [[ "$COMMAND" == "reload" ]]; then
        # Execute the systemctl daemon-reload command
        ssh $IP "sudo systemctl daemon-reload"
        
        # Output the result of the command
        if [[ $? -eq 0 ]]; then
            echo "$IP: daemon-reload successful."
        else
            echo "$IP: daemon-reload failed."
        fi
    else
        # Execute the systemctl command (start, stop, restart)
        ssh $IP "sudo systemctl $COMMAND liftof"
        
        # Output the result of the command
        if [[ $? -eq 0 ]]; then
            echo "$IP: $COMMAND successful."
        else
            echo "$IP: $COMMAND failed."
        fi
    fi
done

# If the command was 'status', display the results in a table format
if [[ "$COMMAND" == "status" ]]; then
    echo -e "Status of the clients:"
    echo -e "$STATUS_RESULTS" | column -t -s ':'
fi

