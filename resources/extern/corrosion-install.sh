#! /bin/sh

# install script for corrosion, an interlink
# for cargo in cmake
cmake -DCMAKE_INSTALL_PREFIX=corrosion/install -Scorrosion -Bcorrosion/build -DCMAKE_BUILD_TYPE=Release 
cmake --build corrosion/build --config Release
cmake --install corrosion/build --config Release
