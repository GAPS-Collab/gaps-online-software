#! /bin/sh 
# get the latest version of the database
# This must (!) be a copy, because the docker container obviously can't follow 
# links outside of the mounted directory
cp /srv/gaps/gaps-online-software/gaps-db/gaps_db/gaps_flight.db python/gondola

# ALTERNATIVE! BUILD WITH zig 
maturin build --release --zig

# FIXME - change python-source in pyproject.toml, otherwise it won't built with 
#         the maturin docker container

##cp ../../../gaps-db/gaps_db/gaps_flight.db python/gondola/
#docker run --rm -v $(pwd):/io ghcr.io/pyo3/maturin build --release
##docker run --rm -v $(pwd):/io  maturin-builder-glibc2_17 maturin build --release 
##docker run --rm -v $(pwd):/io ghcr.io/pyo3/maturin build --release
##twine upload target/wheels/gondola-0.11.24-cp311-abi3-manylinux_2_17_x86_64.manylinux2014_x86_64.whl  
