#! /bin/sh

# install script for corrosion, an interlink
# for cargo in cmake
cmake -Scorrosion -Bcorrosion/build -DCMAKE_BUILD_TYPE=Release  .
cmake --build corrosion/build --config Release 
