#!/bin/sh

kill `ps aux | grep liftof | awk NR==1'{print $2}'`
