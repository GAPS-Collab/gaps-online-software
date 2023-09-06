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
    spdlog::debug("Filename matches pattern for RB ID {}!", match[1].str());
    u32 number1 = std::stoi(match[1].str());
    return number1;
  } else {
    return 0; // Return an invalid pair if no match is found
  }
}


/************************************************/

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
  spdlog::info("Loaded calibration for RB {}!", rb_id);
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


