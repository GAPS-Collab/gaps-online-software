#! /bin/sh
ping 8.8.8.8 | ./parse_ping.pl | feedgnuplot --stream 1 --domain --lines --points --xlen 300 --extracmds 'set logscale y' --ymin 5 --ymax 1000
