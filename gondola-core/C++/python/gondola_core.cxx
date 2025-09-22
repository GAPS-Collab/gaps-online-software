#include <nanobind/nanobind.h>
#include "sd_legacy.hpp" 

int add(int a, int b) { return a + b; }


NB_MODULE(gondola_cxx, m) {
  m.def("add", &add);
  #ifdef BUILD_WITH_ROOT
  m.def("read_sd_legacy_example",&gondola::read_sd_legacy_example); 
  #endif 
}


