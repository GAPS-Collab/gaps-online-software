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

//void RemoveSpikes(double_t wf[NCHN][1024], unsigned int tCell, int spikes[])
void spike_cleaning_drs4(Vec<Vec<f32>> &wf, u16 tCell, i32 spikes[]) {
  int i, j, k, l;
  double x, y;
  int sp[NCHN][10];
  int rsp[10];
  int n_sp[NCHN];
  int n_rsp;
  int nNeighbor, nSymmetric;
  int nChn = NCHN;
  double_t filter, dfilter;

  memset(sp, 0, sizeof(sp));
  memset(rsp, 0, sizeof(rsp));
  memset(n_sp, 0, sizeof(n_sp));
  n_rsp = 0;

  /* set rsp to -1 */
  for (i = 0; i < 10; i++)
  {
    rsp[i] = -1;
  }
  /* find spikes with special high-pass filters */
  for (j = 0; j < 1024; j++)
  {
    for (i = 0; i < nChn; i++)
    {
      filter = -wf[i][j] + wf[i][(j + 1) % 1024] + wf[i][(j + 2) % 1024] - wf[i][(j + 3) % 1024];
      dfilter = filter + 2 * wf[i][(j + 3) % 1024] + wf[i][(j + 4) % 1024] - wf[i][(j + 5) % 1024];
      if (filter > 20 && filter < 100)
      {
        if (n_sp[i] < 10)   // record maximum of 10 spikes
        {
          sp[i][n_sp[i]] = (j + 1) % 1024;
          n_sp[i]++;
        }
        else                // too many spikes -> something wrong
        {
          return;
        }
        // filter condition avoids mistaking pulse for spike sometimes
      }
      else if (dfilter > 40 && dfilter < 100 && filter > 10)
      {
        if (n_sp[i] < 9)   // record maximum of 10 spikes
        {
          sp[i][n_sp[i]] = (j + 1) % 1024;
          sp[i][n_sp[i] + 1] = (j + 3) % 1024;
          n_sp[i] += 2;
        }
        else                // too many spikes -> something wrong
        {
          return;
        }
      }
    }
  }

  /* find spikes at cell #0 and #1023
  for (i = 0; i < nChn; i++) {
    if (wf[i][0] + wf[i][1] - 2*wf[i][2] > 20) {
      if (n_sp[i] < 10) {
        sp[i][n_sp[i]] = 0;
        n_sp[i]++;
      }
    }
    if (-2*wf[i][1021] + wf[i][1022] + wf[i][1023] > 20) {
      if (n_sp[i] < 10) {
        sp[i][n_sp[i]] = 1022;
        n_sp[i]++;
      }
    }
  }
  */

  /* go through all spikes and look for neighbors */
  for (i = 0; i < nChn; i++)
  {
    for (j = 0; j < n_sp[i]; j++)
    {
      nSymmetric = 0;
      nNeighbor = 0;
      /* check if this spike has a symmetric partner in any channel */
      for (k = 0; k < nChn; k++)
      {
        for (l = 0; l < n_sp[k]; l++)
          if ((sp[i][j] + sp[k][l] - 2 * tCell) % 1024 == 1022)
          {
            nSymmetric++;
            break;
          }
      }
      /* check if this spike has same spike is in any other channels */
      for (k = 0; k < nChn; k++)
        if (i != k)
        {
          for (l = 0; l < n_sp[k]; l++)
            if (sp[i][j] == sp[k][l])
            {
              nNeighbor++;
              break;
            }
        }
      /* if at least two matching spikes, treat this as a real spike */
      if (nNeighbor >= 2)
      {
        for (k = 0; k < n_rsp; k++)
          if (rsp[k] == sp[i][j]) // ignore repeats
            break;
        if (n_rsp < 10 && k == n_rsp)
        {
          rsp[n_rsp] = sp[i][j];
          n_rsp++;
        }
      }
    }
  }

  /* recognize spikes if at least one channel has it */
  for (k = 0; k < n_rsp; k++)
  {
    spikes[k] = rsp[k];
    for (i = 0; i < nChn; i++)
    {
      if (k < n_rsp && fabs(rsp[k] - rsp[k + 1] % 1024) == 2)
      {
        /* remove double spike */
        j = rsp[k] > rsp[k + 1] ? rsp[k + 1] : rsp[k];
        x = wf[i][(j - 1) % 1024];
        y = wf[i][(j + 4) % 1024];
        if (fabs(x - y) < 15)
        {
          wf[i][j % 1024] = x + 1 * (y - x) / 5;
          wf[i][(j + 1) % 1024] = x + 2 * (y - x) / 5;
          wf[i][(j + 2) % 1024] = x + 3 * (y - x) / 5;
          wf[i][(j + 3) % 1024] = x + 4 * (y - x) / 5;
        }
        else
        {
          wf[i][j % 1024] -= 14.8f;
          wf[i][(j + 1) % 1024] -= 14.8f;
          wf[i][(j + 2) % 1024] -= 14.8f;
          wf[i][(j + 3) % 1024] -= 14.8f;
        }
      }
      else
      {
        /* remove single spike */
        x = wf[i][(rsp[k] - 1) % 1024];
        y = wf[i][(rsp[k] + 2) % 1024];
        if (fabs(x - y) < 15)
        {
          wf[i][rsp[k]] = x + 1 * (y - x) / 3;
          wf[i][(rsp[k] + 1) % 1024] = x + 2 * (y - x) / 3;
        }
        else
        {
          wf[i][rsp[k]] -= 14.8f;
          wf[i][(rsp[k] + 1) % 1024] -= 14.8f;
        }
      }
    }
    if (k < n_rsp && fabs(rsp[k] - rsp[k + 1] % 1024) == 2)
      k++; // skip second half of double spike
  }
  //spdlog::error("This is not implemented yet and does NUTHIN!");
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

Vec<Vec<f32>> RBCalibration::voltages    (const RBEventMemoryView &event, bool spike_cleaning) const {
  Vec<Vec<f32>> all_ch_voltages;
  for (u8 ch=1;ch<NCHN+1;ch++) {
    all_ch_voltages.push_back(voltages(event, ch));
  }
  if (spike_cleaning) {
    int spikes[NWORDS];
    for (usize n=0;n<NWORDS;n++) {
      spikes[n] = 0;
    }
    spike_cleaning_drs4(all_ch_voltages, event.stop_cell, spikes);
  }
  return all_ch_voltages;
}

/************************************************/

Vec<Vec<f32>> RBCalibration::voltages    (const RBEvent &event, bool spike_cleaning) const {
  Vec<Vec<f32>> all_ch_voltages;
  for (u8 ch=1;ch<NCHN+1;ch++) {
    all_ch_voltages.push_back(voltages(event, ch));
  }
  if (spike_cleaning) {
    int spikes[NWORDS];
    for (usize n=0;n<NWORDS;n++) {
      spikes[n] = 0;
    }
    spike_cleaning_drs4(all_ch_voltages, event.header.stop_cell, spikes);
  }
  return all_ch_voltages;
}

/************************************************/
  
Vec<Vec<f32>> RBCalibration::nanoseconds (const RBEventMemoryView &event) const {
  Vec<Vec<f32>> all_ch_nanoseconds;
  for (u8 ch=1;ch<NCHN+1;ch++) {
    all_ch_nanoseconds.push_back(nanoseconds(event, ch));
  }
  return all_ch_nanoseconds;
}

/************************************************/
  
Vec<Vec<f32>> RBCalibration::nanoseconds (const RBEvent &event) const {
  Vec<Vec<f32>> all_ch_nanoseconds;
  for (u8 ch=1;ch<NCHN+1;ch++) {
    all_ch_nanoseconds.push_back(nanoseconds(event, ch));
  }
  return all_ch_nanoseconds;
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

