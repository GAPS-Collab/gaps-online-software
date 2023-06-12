#! /usr/bin/zsh

# set environment variables for custom installed software
THISDIR=`pwd`
echo  "Using gaps-online-software from $THISDIR"

export ROOTSYS=/srv/root/root-6.28-patches-install
export PYTHONPATH=$PYTHONPATH:$ROOTSYS/lib:/srv/gaps/gfp-data/gfp_analysis:$THISDIR/build/tof:$THISDIR/build/dataclasses
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$THISDIR/build/dataclasses
export PYTHONPATH=$PYTHONPATH:$THISDIR/tof 
export PYTHONPATH=$PYTHONPATH:$THISDIR/_deps/bfsw-build/pybfsw/gse:/srv/gaps/gaps-online-software/build/_deps/bfsw-build/pybfsw/
export PYTHONPATH=$PYTHONPATH:$THISDIR/_deps/bfsw-src/pybfsw/gse/
export PYTHONPATH=$PYTHONPATH:$THISDIR/_deps/bfsw-src/

# start the mongo database
#sudo systemctl start mongodb0
