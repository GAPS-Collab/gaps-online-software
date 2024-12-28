#! /bin/sh

# setup rye
rye sync

source rye run sphinx-build -M html . build
