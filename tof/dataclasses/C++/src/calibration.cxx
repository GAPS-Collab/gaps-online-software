#include <fstream>
#include <iostream>

#include "calibration.h"


std::vector<Calibrations_t> read_calibration_file (std::string filename) {
  std::vector<Calibrations_t> all_channel_calibrations
      = std::vector<Calibrations_t>{NCHN};
  std::fstream calfile(filename.c_str(), std::ios_base::in);
  if (calfile.fail()) {
    std::cerr << "[ERROR] Can't open " << filename << 
      " !" << std::endl;
    return all_channel_calibrations;
  }
  for (size_t i=0; i<NCHN; i++) {
    for (size_t j=0; j<NWORDS; j++)
      calfile >> all_channel_calibrations[i].vofs[j];
    for (size_t j=0; j<NWORDS; j++)
      calfile >> all_channel_calibrations[i].vdip[j];
    for (size_t j=0; j<NWORDS; j++)
      calfile >> all_channel_calibrations[i].vinc[j];
    for (size_t j=0; j<NWORDS; j++)
      calfile >> all_channel_calibrations[i].tbin[j];
  }
  return all_channel_calibrations;
}

void apply_tcal(const u16 stop_cell,
                const Calibrations_t &cal,
                f32 times[NWORDS]) {
  for (size_t k = 1; k < NWORDS; k++) {
    times[k] = times[k-1] + cal.tbin[(k-1+stop_cell) % NWORDS];
  }
}


void apply_vcal(const u16 stop_cell,
                const i16 adc[NWORDS],
                const Calibrations_t &cal,
                f32 waveform[NWORDS]) {
  for (int i = 0; i < NWORDS; i++) {
    waveform[i] = (f32) adc[i];
    ////if (i%100 == 0)
    //  //printf("%f\n", traceOut[i]);
    waveform[i] -= cal.vofs[(i + stop_cell)%NWORDS];
    waveform[i] -= cal.vdip[i];
    waveform[i] *= cal.vinc[(i+stop_cell)%NWORDS];
  }
}
  //vec_f32 voltages;
  //for (size_t k=0; k<NCHN; k++) {
  //  f64 trace_out[NWORDS];
  //  u32 n = sizeof(trace_out)/sizeof(trace_out[0]);
  //  VoltageCalibration(const_cast<short int*>(evt.ch_adc[k]),
  //                     trace_out,
  //                     evt.stop_cell,
  //                     cal[k]);
  //  result.push_back(std::vector<f64>(trace_out, trace_out + n));
  //}
  //return result;
  //}

