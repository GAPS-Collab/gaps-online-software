# `gondola_cxx` - C++ compatibility layer for gondola 

C++ bridge for `gaps-online-software` core library, `gondola`

While the core `gondola` library is written in rust (pybindings are 
available on pypi, see `gondola`), a secondary implementation exists 
in C++. It's main purpose is to interface with existing C++ code, 
notably, "SimpleDet", which is a pre-existing analysis framework 
used widely within the GAPS collaboration. 
The `gondola_cxx` package includes the pybindings for the C++ 
implementation for the gondola C++ library as well as a 
compatibility layer, to interface with SimpleDet. 

The project is still in an early alpha stage and can not be used for 
producion. 

Example: Read SimpleDet files: 

```
import gondola_cxx as gxx 
import tqdm

# the address will be cleaned up more nicely in future releases
reader = gxx.gondola_cxx.SDRootReader('/path/to/your/favorite/sd-file.root') 

# right now, looping over the events is a bit clumsy
for k in tqdm.tqdm(range(reader.nevents_total)):
    # not a lot is currently supported, but this returns 
    # many of the primary properties
    primary = reader.get_primary(k) 
    print (primary) # primary is a gxx.gondola_cxx.Tracklet 
```




The CMakeLists.txt file in this directory is seperate from the entire 
gaps-online-software build system.
