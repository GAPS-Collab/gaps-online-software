#! /bin/sh

# setup rye
rye sync

rye run sphinx-build -M html . build
