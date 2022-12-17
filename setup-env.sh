#! /usr/bin/zsh

# set environment variables for custom installed software

export ROOTSYS=/srv/root/root-6.26-patches-install
export PYTHONPATH=$PYTHONPATH:$ROOTSYS/lib:/srv/gaps/gfp-data/gfp_analysis:/srv/gaps/gaps-online-software/build/tof:/srv/gaps/gaps-online-software/build/dataclasses
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/srv/gaps/gaps-online-software/build/dataclasses


# start the mongo database
#sudo systemctl start mongodb
