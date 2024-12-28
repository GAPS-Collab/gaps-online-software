#! /bin/sh

# setup rye
export RYE_NO_AUTO_INSTALL=1

curl -sSf https://rye.astral.sh/get | bash
rye sync

source rye run sphinx-build -M html . build
