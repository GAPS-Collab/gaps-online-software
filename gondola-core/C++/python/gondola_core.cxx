#include <nanobind/nanobind.h>

int add(int a, int b) { return a + b; }

NB_MODULE(gondola_cxx, m) {
  m.def("add", &add);
}


