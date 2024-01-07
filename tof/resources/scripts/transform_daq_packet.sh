#! /bin/sh

od -v --width=2 -t x2 --endian little  event.dat | awk -F" " '{if (length($2) > 0) print $2}' > daq_packet.txt
