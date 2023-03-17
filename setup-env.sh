#! /usr/bin/zsh

# set environment variables for custom installed software

export ROOTSYS=/srv/root/root-6.28-patches-install
export PYTHONPATH=$PYTHONPATH:$ROOTSYS/lib:/srv/gaps/gfp-data/gfp_analysis:/srv/gaps/gaps-online-software/build/tof:/srv/gaps/gaps-online-software/build/dataclasses
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/srv/gaps/gaps-online-software/build/dataclasses
export PYTHONPAHT=$PYTHONPATH:/srv/gaps/gaps-online-software/build/tof 
export PYTHONPATH=$PYTHONPATH:/srv/gaps/gaps-online-software/build/_deps/bfsw-build/pybfsw/gse:/srv/gaps/gaps-online-software/build/_deps/bfsw-build/pybfsw/
export PYTHONPATH=$PYTHONPATH:/srv/gaps/gaps-online-software/build/_deps/bfsw-src/pybfsw/gse/
export PYTHONPATH=$PYTHONPATH:/srv/gaps/gaps-online-software/build/_deps/bfsw-src/

# start the mongo database
#sudo systemctl start mongodb0
