# gaps-online-software 

![build-docs-badge](https://github.com/GAPS-Collab/gaps-online-software/workflows/BuildBot/badge.svg)

This is Version AULEPE-0.12 <a href="https://en.wikipedia.org/wiki/Sailfish">Aulepe are sailfish!</a><img src="resources/assets/aulepe_luma.png" align="right" width="15%">
<br clear="right"/>

>[!NOTE] 
>The fastest non-airborne species is actually not the Cheetah! Marine life can be even faster. While the sailfish seems to be a little less fast then the [fastest non-airborne animal on the planet (Black Marlin with speeds up to 80MpH)](https://en.wikipedia.org/wiki/Fastest_animals). Thus the Black Marlin is faster than a Cheetah.
>Sailfish have been observed to swim up to 68 MpH, and while that's a little less fast than their world-record cousin, their displays when they leap out of the water at these speed are quite astonishing. 
>Marlin can be found around the Hawaiian islands and are actually quite tasty!

## CHANGELOG/Migration guide  
[Since v0.12 we are keeping a global CHANELOG.MD](CHANGELOG.md)

* Antarctic RBWaveform data from telemetry might not be able to be read be read with this version, use v0.11 instead 
## API docs 

The documentation supports different release versions of the code and is hosted on github-pages.

[software documentation](https://gaps-collab.github.io/gaps-online-software/)

## Installation

### Installation of the python library 

The python code is called `gondola` and hosted on [pypi](https://pypi.org/project/gondola/) and can 
be installed with `uv/pip`.

## From source

### Software repository

The code is organized in a public github repository at 
* [github](https://github.com/GAPS-Collab/gaps-online-software)

### Branches and how to get updates

The branches/releases are named after fish in Hawaii. A fish 
identification card can be found [here](https://www.honolulu.gov/rep/site/dpr/dpr_docs/hbep_fish_id_card.pdf).
You can switch branches with `git checkout <branch>`. To get updates, use `git pull`

Usually, each branch has a specific purpose, everything with version numbers < 1.0.0 will be unstable, meaning there is no guarantee for code to work even after a pull.
The branches following the naming scheme "FISHNAME-X.X" are dedicated to specific tasks, 
e.g. the NTS campaign, during flight I (0.11) or after flight I (>=0.12). Please see the dedicated README for the specific branch.

### Clone the repository wit submodules

We are using git submodules to pull in some of the dependencies.
To automatically check them out when clone te repository, use
`git clone --recurse-submodules`

### Prerequisites for compilation from source

* rust toolchain - to compile `liftof` flight software suite as well as the 
  core library `gondola-core` with the pybindings. Rust edtion 2024 is 
  required
* `cmake` is used as a build system for the C++ part.
* The C++ API uses the C++20 standard and thus wants gcc-13 or later.
* Doxygen to build the C++ documentation locally.

### Building the C++ implementation of the `gondola-core` library

The installation uses `cmake`. Create a build directory and execute
`cmake <gaps-online-software source directory> --install-prefix <install_dir>`

After that, you can have a look at the `cmake` cache with 
`ccmake .` in your build directory. If everything seems ok, execute:

`make`
`make doc`
`make install`

After that, the `build` directory can be discarded, but might be kept for 
a quicker build when there are updates. Important is the `<install_dir>`.

In `<install_dir>` there is a `setup-env.sh` script, this needs to be sourced 
in order to set the necessary variables for `PYTHONPATH`, `PATH` and `LD_LIBRARY_PATH`.
Do so with 
`source setup-env.sh`
It will greet you with a banner.

After that, you can either write your own C++ code, linking to the gaps-online-software
C++ API, or use the included pybindings

Example code on how to use them can be found in 
`<install_dir>/examples/`

[More detailed installation instructions can be found in INSTALL.MD](INSTALL.md)

## Running the tests (rust only)

`cargo` provides unit and integration tests. Without going into further detail here,
please note that some care is needed that all tests are run when using `cargo test`. 
In general, there is 

* `cargo test --features=random` to run the unit tests
* `cargo test --features=random --test=test` to run the integration tests

The command `cargo test --features=random -- list` will list all tests. Further usefule
is the addition of the `--no-capture` flag, e.g. `cargo test --features=random -- --no-captuer` in case the output of the tests shall be printed as well.

## Getting help

Please see the README.md in the individual subfolders. 

## Maintainer

* A. Stoessl <stoessl@hawaii.edu>

* G. Tytus <gtytus@hawaii.edu>
