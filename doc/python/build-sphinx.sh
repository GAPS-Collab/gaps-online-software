#! /bin/sh

# setup rye
rye lock --update-all
rye sync

rye run sphinx-build -M html . build
