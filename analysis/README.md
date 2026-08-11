## Analysis 

Keeps (private) analysis scripts for the use with 
gondola/gaps-online-software.

These can be in either language, most here are python.

We have the following:

* notebooks - general jupyter notebooks + some personal ones 
* zweerink  - JAZ' C++ scripts, formerly in tof/examples. These 
              use root and can be compiled with cmake.
              To do so, set `BUILD_CXX_PERSONAL_ANALYSIS_CODE=ON`
* kaoyama   - Kazu's waveform fitting. Currently this is for record 
              only and won't be compiled 
* scripts   - python scripts for general use 
 
### How to run the python scripts?

`pypoject.toml` files are provided for the use with [uv](https://docs.astral.sh/uv/) which is 
higly recommend and just a fantastic tool that solve all of your python woes. 
