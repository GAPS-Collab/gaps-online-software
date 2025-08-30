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

We are typically using rye to run everything within it's own virtual environment. A pyproject.toml is 
provided, so all you need to do is to run `rye sync`. 
Then scripts can be run through rye.
For the jupyter notebooks, we provide an alias `rye run jupy-lab` which opens a jupyter lab at port 9876.

