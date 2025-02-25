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
* `make (-j <njobs>)` to build the project.
* `make install`,
* if you get an error here, you might need to build the docs seperatly with `make doc`
* then go to specified install directory.
* in the install directory, `source setup-env.sh` to set the necessary paths.
* A welcome banner should greet you.

_The rust pybindings are currently a bit experimental, and have to be built separatly. This is currently an expert option, so if you are interested, contact the maintainer._

## Build the docs

* docs can be built with `make doc`

## Using pybindings with pyo3 on MacOS

Building rust pybindings via pyo3 on MacOS encounters build problems, specifically "errno=21 for architecture x86_64
clang: error: linker command failed with exit code 1 ", even when not using clang as your default C compiler. 

To avoid this, you must create a file on your home system in the cargo directory called "/usr/bin/.cargo/config". You may need to create the config file if you don't have one already. 

Paste the following into the .cargo/config file: 
        [target.x86_64-apple-darwin]
        rustflags = [
        "-C", "link-arg=-undefined",
        "-C", "link-arg=dynamic_lookup",
        ]

        [target.aarch64-apple-darwin]
        rustflags = [
        "-C", "link-arg=-undefined",
        "-C", "link-arg=dynamic_lookup",
        ]
Now, your pyo3 pybindings in rust should build. 
