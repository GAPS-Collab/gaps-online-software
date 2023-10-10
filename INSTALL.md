# Installation lnstructions

The repository has several components:

1. online (in-flight) software in `tof/liftof`

2. dataclasses in `tof/dataclasses` which themselves have a C++ API, 
a pybind11 API for te C++ API, a rust API and a pyO3 API for te rust
API

Everything can be built with cmake, sssuming g++ as well as a rust toolchain
is installed. Building the RB component for liftof also requires docker and 
cross, since we need to cross-compile that for ARM32. _Typically, it will not 
be necessary to rebuild that_

## Detailed instructions

* clone the repository with submodules `git clone --recurse-submodules <repository>`
* create a `build` directory wherever you like, for now we assume `gaps-online-software/build`
* cd `build` 
* `cmake ../` (or the path to te `gaps-online-software` src repository ceckout.
* inspect everything with `ccmake .`. There you can also set te path for the
install directory and switch on the different components. In case you want 
to build te pybindings, `pybind11` is required.
* `make install`, then go to specified install directory.
* in the install directory, `source setup-env.sh` to set the necessary paths.
* A welcome banner should greet you.

## BUild the docs

* docs can be built with `make doc`

