#ifndef WAVEFORM_FOR_PYBIND_H_INCLUDED
#define WAVEFORM_FOR_PYBIND_H_INCLUDED

/***********************************
 * Waveform analysis related 
 * functions, translation
 * to TofHit
 *
 *
 **********************************/

#include "TofEvent.pb.h"

double get_t0(double tA, double tB);

double dist_from_A(double tA, double t0);

gaps::TofHit waveforms_to_hit(std::vector<double> tA,
                              std::vector<double> wfA,
                              std::vector<double> tB,
                              std::vector<double> wfB);

#endif
