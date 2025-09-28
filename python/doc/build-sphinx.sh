#! /bin/sh
rye init 
# setup rye
rye sync

rye run sphinx-build -M html . build
