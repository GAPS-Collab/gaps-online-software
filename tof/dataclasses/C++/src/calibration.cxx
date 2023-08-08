#include <fstream>
#include <iostream>
#include <regex>

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "calibration.h"

u8 extract_rbid(const String& filename) {
  std::regex pattern("rb(\\d+)_cal"); // Match "RB" followed by digits, an underscore, and more digits
  std::smatch match;
  if (std::regex_search(filename, match, pattern)) {
    spdlog::info("Found {} for RB id in filename!", match[1].str());
    //spdlog::info("{}", match[2].str());

    u32 number1 = std::stoi(match[1].str());
    //u32 number2 = std::stoi(match[2].str());
    //u32 two_digit_number = (number1 % 100) * 100 + (number2 % 100);
    return number1;
  } else {
    return 0; // Return an invalid pair if no match is found
  }
}


/************************************************/

void spike_cleaning_jeff(Vec<f32> &voltages) {
//  //let mut spikes  : [i32;10] = [0;10];
//  f32 filter;
//  f32 dfilter;
//  //let mut n_symmetric : usize;
//  usize n_neighbor;
//
//  usize  mut n_rsp = 0;
//  Vec<f32> input_voltages = voltages;
//
//  Vec<i32> rsp = Vec<i32>(10, -1);
//  Vec<Vec<usize>> sp = Vec<Vec<usize>>(NCHN, Vec<usize>(10,0);
//  Vec<usize>    n_sp = Vec<usize>(10, 0);
//  //let mut sp   : [[usize;10];NCHN] = [[0;10];NCHN];
//  //let mut n_sp : [usize;10]      = [0;10];
//  for (usize j=0;j<NWORDS;j++) {
//    for (usize i=0;i<NCHN; i++) {
//  //for j in 0..NWORDS as usize {
//  //  for i in 0..NCHN as usize {
//  //    filter = -self.voltages[i][j] + self.voltages[i][(j + 1) % NWORDS] + self.voltages[i][(j + 2) % NWORDS] - self.voltages[i][(j + 3) % NWORDS];
//      dfilter = filter + 2.0 * input_voltages[i][(j + 3) % NWORDS] + self.voltages[i][(j + 4) % NWORDS] - self.voltages[i][(j + 5) % NWORDS];
//      if filter > 20.0  && filter < 100.0 {
//        if n_sp[i] < 10 {   // record maximum of 10 spikes
//          sp[i][n_sp[i] as usize] = (j + 1) % NWORDS ;
//          n_sp[i] += 1;
//        // FIXME - error checking
//        } else {return;}            // too many spikes -> something wrong
//      }// end of if
//      else if dfilter > 40.0 && dfilter < 100.0 && filter > 10.0 {
//        if n_sp[i] < 9 {  // record maximum of 10 spikes
//          sp[i][n_sp[i] as usize] = (j + 1) % NWORDS ;
//          sp[i][(n_sp[i] + 1) as usize] = (j + 3) % NWORDS ;
//          n_sp[i] += 2;
//        } else { return;} // too many spikes -> something wrong
//      } // end of else if
//
//    }// end loop over NCHN
//  } // end loop over NWORDS
//
//  // go through all spikes and look for neighbors */
//  for i in 0..NCHN {
//    for j in 0..n_sp[i] as usize {
//      //n_symmetric = 0;
//      n_neighbor = 0;
//      for k in 0..NCHN {
//        for l in 0..n_sp[k] as usize {
//        //check if this spike has a symmetric partner in any channel
//          if (sp[i][j] as i32 + sp[k][l] as i32 - 2 * self.stop_cell as i32) as i32 % NWORDS as i32 == 1022 {
//            //n_symmetric += 1;
//            break;
//          }
//        }
//      } // end loop over k
//      // check if this spike has same spike is in any other channels */
//      //for (k = 0; k < nChn; k++) {
//      for k in 0..NCHN {
//        if i != k {
//          for l in 0..n_sp[k] {
//            if sp[i][j] == sp[k][l] {
//            n_neighbor += 1;
//            break;
//            }
//          } // end loop over l   
//        } // end if
//      } // end loop over k
//
//      if n_neighbor >= 2 {
//        for k in 0..n_rsp {
//          if rsp[k] == sp[i][j] as i32 {break;} // ignore repeats
//          if n_rsp < 10 && k == n_rsp {
//            rsp[n_rsp] = sp[i][j] as i32;
//            n_rsp += 1;
//          }
//        }  
//      }
//
//    } // end loop over j
//  } // end loop over i
//
//  // recognize spikes if at least one channel has it */
//  //for (k = 0; k < n_rsp; k++)
//  let magic_value : f64 = 14.8;
//  let mut x : f64;
//  let mut y : f64;
//
//  let mut skip_next : bool = false;
//  for k in 0..n_rsp {
//    if skip_next {
//      skip_next = false;
//      continue;
//    }
//    spikes[k] = rsp[k];
//    //for (i = 0; i < nChn; i++)
//    for i in 0..NCHN {
//      if k < n_rsp && i32::abs(rsp[k] as i32 - rsp[k + 1] as i32 % NWORDS as i32) == 2
//      {
//        // remove double spike 
//        let j = if rsp[k] > rsp[k + 1] {rsp[k + 1] as usize}  else {rsp[k] as usize};
//        x = self.voltages[i][(j - 1) % NWORDS];
//        y = self.voltages[i][(j + 4) % NWORDS];
//        if f64::abs(x - y) < 15.0
//        {
//          self.voltages[i][j % NWORDS] = x + 1.0 * (y - x) / 5.0;
//          self.voltages[i][(j + 1) % NWORDS] = x + 2.0 * (y - x) / 5.0;
//          self.voltages[i][(j + 2) % NWORDS] = x + 3.0 * (y - x) / 5.0;
//          self.voltages[i][(j + 3) % NWORDS] = x + 4.0 * (y - x) / 5.0;
//        }
//        else
//        {
//          self.voltages[i][j % NWORDS] -= magic_value;
//          self.voltages[i][(j + 1) % NWORDS] -= magic_value;
//          self.voltages[i][(j + 2) % NWORDS] -= magic_value;
//          self.voltages[i][(j + 3) % NWORDS] -= magic_value;
//        }
//      }
//      else
//      {
//        // remove single spike 
//        x = self.voltages[i][((rsp[k] - 1) % NWORDS as i32) as usize];
//        y = self.voltages[i][(rsp[k] + 2) as usize % NWORDS];
//        if f64::abs(x - y) < 15.0 {
//          self.voltages[i][rsp[k] as usize] = x + 1.0 * (y - x) / 3.0;
//          self.voltages[i][(rsp[k] + 1) as usize % NWORDS] = x + 2.0 * (y - x) / 3.0;
//        }
//        else
//        {
//          self.voltages[i][rsp[k] as usize] -= magic_value;
//          self.voltages[i][(rsp[k] + 1) as usize % NWORDS] -= magic_value;
//        }
//      } // end loop over nchn
//    } // end loop over n_rsp
//    if k < n_rsp && i32::abs(rsp[k] - rsp[k + 1] % NWORDS as i32) == 2
//      {skip_next = true;} // skip second half of double spike
//  } // end loop over k
//}
//
}

/************************************************/

RBCalibration::RBCalibration() {
  rb_id = 0;
  for (usize ch=0;ch<NCHN;ch++) {
    v_offsets.push_back(Vec<f32>(NWORDS, 0));
    v_dips.push_back(Vec<f32>(NWORDS, 0));
    v_incs.push_back(Vec<f32>(NWORDS, 0));
    t_bin.push_back(Vec<f32>(NWORDS,0)) ;
  }
}

/************************************************/

Vec<f32> RBCalibration::voltages(const RBEventMemoryView &event, const u8 channel) const {
  Vec<f32> voltages = Vec<f32>(NWORDS,0);
  if (!(channel_check(channel))) {
    return voltages;
  }
  Vec<u16> adc = event.get_channel_adc(channel);
  for (usize i = 0; i < NWORDS; i++) {
    voltages[i] = (f32) adc[i];
    ////if (i%100 == 0)
    //  //printf("%f\n", traceOut[i]);
    voltages[i] -= v_offsets[channel - 1][(i + event.stop_cell)%NWORDS];
    voltages[i] -= v_dips[channel - 1][i];
    voltages[i] *= v_incs[channel - 1][(i+ event.stop_cell)%NWORDS];
  }
  return voltages;
}

/************************************************/

Vec<f32> RBCalibration::nanoseconds(const RBEventMemoryView &event, const u8 channel) const {
  Vec<f32> nanoseconds = Vec<f32>(NWORDS,0);
  if (!(channel_check(channel))) {
    return nanoseconds;
  }
  for (usize k = 1; k < NWORDS; k++) {
    nanoseconds[k] = nanoseconds[k-1] + t_bin[channel - 1][(k-1+event.stop_cell) % NWORDS];
  }
  return nanoseconds;
}

/************************************************/

Vec<f32> RBCalibration::voltages(const RBEvent &event, const u8 channel) const {
  Vec<f32> voltages = Vec<f32>(NWORDS,0);
  if (!(channel_check(channel))) {
    return voltages;
  }
  Vec<u16> adc = event.get_channel_adc(channel);
  for (usize i = 0; i < NWORDS; i++) {
    voltages[i] = (f32) adc[i];
    ////if (i%100 == 0)
    //  //printf("%f\n", traceOut[i]);
    voltages[i] -= v_offsets[channel - 1][(i + event.header.stop_cell)%NWORDS];
    voltages[i] -= v_dips[channel - 1][i];
    voltages[i] *= v_incs[channel - 1][(i+ event.header.stop_cell)%NWORDS];
  }
  return voltages;
}

/************************************************/

Vec<f32> RBCalibration::nanoseconds(const RBEvent &event, const u8 channel) const {
  Vec<f32> nanoseconds = Vec<f32>(NWORDS,0);
  if (!(channel_check(channel))) {
    return nanoseconds;
  }
  for (usize k = 1; k < NWORDS; k++) {
    nanoseconds[k] = nanoseconds[k-1] + t_bin[channel - 1][(k-1+event.header.stop_cell) % NWORDS];
  }
  return nanoseconds;
}

/************************************************/

RBCalibration RBCalibration::from_bytestream(const Vec<u8> &bytestream,
                              u64 &pos) {
  spdlog::error("This is not implemented yet!");
  RBCalibration calibration = RBCalibration();
  return calibration;
}

/************************************************/

RBCalibration RBCalibration::from_txtfile(const String &filename) {
  //std::vector<Calibrations_t> all_channel_calibrations
  //    = std::vector<Calibrations_t>{NCHN};
  RBCalibration calibration;
  u8 rb_id = extract_rbid(filename);
  calibration.rb_id = rb_id;
  std::fstream calfile(filename.c_str(), std::ios_base::in);
  if (calfile.fail()) {
    spdlog::error("Can't open {}",filename);
    return calibration;
  }
  for (size_t i=0; i<NCHN; i++) {
    for (size_t j=0; j<NWORDS; j++)
      calfile >> calibration.v_offsets[i][j];
    for (size_t j=0; j<NWORDS; j++)
      calfile >> calibration.v_dips[i][j];
    for (size_t j=0; j<NWORDS; j++)
      calfile >> calibration.v_incs[i][j];
    for (size_t j=0; j<NWORDS; j++)
      calfile >> calibration.t_bin[i][j];
  }
  return calibration;
}

/************************************************/

bool RBCalibration::channel_check(u8 channel) const {
  if (channel == 0) {
    spdlog::error("Remember, channels start at 1. 0 does not exist!");
    return false;
  }
  if (channel > 9) {
    spdlog::error("Thera are no channels > 9!");
    return false;
  }
  return true;
}

/************************************************/

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

