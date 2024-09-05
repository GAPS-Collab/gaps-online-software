#include <fstream>
#include <iostream>
#include <regex>

#include "logging.hpp"
#include "parsers.h"
#include "calibration.h"
#include "io.hpp"

u8 extract_rbid(const String& filename) {
  std::regex pattern("rb(\\d+)_cal"); // Match "RB" followed by digits, an underscore, and more digits
  std::smatch match;
  if (std::regex_search(filename, match, pattern)) {
    log_debug("Filename matches pattern for RB ID " << match[1].str() << "!");
    u32 number1 = std::stoi(match[1].str());
    return number1;
  } else {
    return 0; // Return an invalid pair if no match is found
  }
}

/************************************************/

/// A simple version of the spike cleaning, which does not rely on 
/// all channels being present and can work independently on each 
/// channel
void spike_cleaning_simple(Vec<Vec<f32>> &wf, bool calibrated) {
//    # TODO: make robust (symmetric, doubles, fixed/estimated spike height)
//    thresh = 360
//    if vcaldone:
//        thresh = 16
//    spikefilter = -wf[:,:-3]+wf[:,1:-2]+wf[:,2:-1]-wf[:,3:]
//    spikes = np.where(np.sum(spikefilter > thresh,axis=0) >= 2)[0]
//    for i in spikes:
//        dV = (wf[:,i+3]-wf[:,i])/3.0
//        wf[:,i+1] = wf[:,i] + dV
//        wf[:,i+2] = wf[:,i] + 2*dV
//    return wf
  int thresh = 360;
  if (calibrated) {
    thresh = 16;
  }

  std::vector<std::vector<double>> spikefilter(wf.size());
  for (size_t i = 0; i < wf.size(); ++i) {
    for (size_t j = 0; j < wf[i].size() - 3; ++j) {
      double value = -wf[i][j] + wf[i][j + 1] + wf[i][j + 2] - wf[i][j + 3];
      spikefilter[i].push_back(value);
    }
  }

  // Finding spikes
  std::vector<int> spikes;
  for (size_t j = 0; j < spikefilter[0].size(); ++j) {
    int count = 0;
    for (size_t i = 0; i < spikefilter.size(); ++i) {
      if (spikefilter[i][j] > thresh) {
        count++;
      }
    }
    if (count >= 2) {
      spikes.push_back(j);
    }
  }

  // Adjusting wf based on spikes
  for (int i : spikes) {
    for (size_t row = 0; row < wf.size(); ++row) {
      if (i + 3 < (int)wf[row].size()) {  // Check to avoid out-of-bounds
        double dV = (wf[row][i + 3] - wf[row][i]) / 3.0;
        wf[row][i + 1] = wf[row][i] + dV;
        wf[row][i + 2] = wf[row][i] + 2 * dV;
      }
    }
  }

  // Printing the adjusted wf for demonstration
  //for (const auto& row : wf) {
  //  for (double num : row) {
  //    std::cout << num << " ";
  //  }
  //  std::cout << "\n";
  //}
}

/************************************************/

/// A simple version of the spike cleaning, which does not rely on 
/// all channels being present and can work independently on each 
/// channel
void spike_cleaning_all(Vec<Vec<f32>> &wf, bool calibrated) {
//    # TODO: make robust (symmetric, doubles, fixed/estimated spike height)
//    thresh = 360
//    if vcaldone:
//        thresh = 16
//    spikefilter = -wf[:,:-3]+wf[:,1:-2]+wf[:,2:-1]-wf[:,3:]
//    spikes = np.where(np.sum(spikefilter > thresh,axis=0) >= 2)[0]
//    for i in spikes:
//        dV = (wf[:,i+3]-wf[:,i])/3.0
//        wf[:,i+1] = wf[:,i] + dV
//        wf[:,i+2] = wf[:,i] + 2*dV
//    return wf
  int thresh = 360;
  int thresh_single = 200;  // threshold for single spikes 

  if (calibrated) {
    thresh = 16;
    thresh_single = 20;
  }

  // First, remedy the known DRS4 spikes
  std::vector<std::vector<double>> spikefilter(wf.size());
  for (size_t i = 0; i < wf.size(); ++i) {
    for (size_t j = 0; j < wf[i].size() - 3; ++j) {
      double value = -wf[i][j] + wf[i][j + 1] + wf[i][j + 2] - wf[i][j + 3];
      spikefilter[i].push_back(value);
    }
  }

  // find spikes
  std::vector<int> spikes;
  for (size_t j = 0; j < spikefilter[0].size(); ++j) {
    int count = 0;
    for (size_t i = 0; i < spikefilter.size(); ++i) {
      if (spikefilter[i][j] > thresh) {
        count++;
      }
    }
    if (count >= 2) {
      spikes.push_back(j);
    }
  }

  // adjust wf based on spikes
  for (int i : spikes) {
    for (size_t row = 0; row < wf.size(); ++row) {
      if (i + 3 < (int)wf[row].size()) {  // Check to avoid out-of-bounds
        double dV = (wf[row][i + 3] - wf[row][i]) / 3.0;
        wf[row][i + 1] = wf[row][i] + dV;
        wf[row][i + 2] = wf[row][i] + 2 * dV;
      }
    }
  }
  
  // Now, deal with the single spikes separated by 32 bins. Since the
  // spikes are associated with the RB, we look for single-bin spikes,
  // separated by 32 bins, that show up on multiple channels in an RB
  // (excluding ch9) .
  size_t start = 5;  // First few bins of trace are often weird. 
  size_t nsegs = 20; // How many 32-bin segments to use
  double sum;

  std::vector<double> singles;
  // For each of the 32-bin series, sum the signals in ch 0-7;
  for (size_t j = 0; j < 32; j++) {
      sum = 0.0; // initialize our sum
      for (size_t k = 0; k < nsegs; k++) {
	size_t ind = start + j + k*32;
	for (size_t i=0; i<wf.size()-1; i++) 
	  sum += wf[i][ind] - (wf[i][ind-1]+wf[i][ind+1])/2.0;
      }
      //printf(" %4.1lf", sum); 
      singles.push_back(sum);
  }
  
  // Now check for spikes
  double combined, largest=0.0;
  size_t spike = 999;
  for (size_t j=0;j<singles.size(); j++) {
    if (j==0)
      combined = singles[j] - (singles[singles.size()-1] + singles[j+1]); 
    else if (j==singles.size()-1)
      combined = singles[j] - (singles[j-1] + singles[0]); 
    else 
      combined = singles[j] - (singles[j-1] + singles[j+1]); 
    if (fabs(combined) > thresh_single && fabs(combined) > largest) {
      spike = j; 
      largest = fabs(combined);
    }
  }
  //if (spike < 999) printf(" :%ld", spike+start);
  //printf("\n");
  
  // If we found spikes, remove them on each waveform present.
  if (spike < 999) {
    // First, determine which channels (0-7) have data
    for (size_t i=0; i<wf.size()-1; i++) {
        double test=0.0;
      for (size_t m=0;m<10;m++) test += wf[i][m+5];
      if ( fabs(test) > 0.001 ) { // Channel has data  
	size_t ind = spike+start; // This is the first bin to correct. 
	do {
	  wf[i][ind] = (wf[i][ind-1] + wf[i][ind+1])/2.0;
	  ind +=32;
	} while ( ind < wf[i].size()-2 ) ;
      }
    }
  }

  /* OLD ALGORITHM THAT WORKS WITH INDIVIDUAL CHANNELS
  // Now, deal with the single spikes separated by 32 bins. Currently,
  // I look at each individual trace to determine if spikes
  // exist. However, the spikes are associated with the readout
  // board. So, we could modify the routine to look at the pairs of
  // channels associated with each paddle to be more sensitive to
  // spikes.
  for (size_t i = 0; i < wf.size()-1; ++i) { // Don't use ch9
    double test=0.0;;
    for (size_t m=0;m<10;m++) test += wf[i][m+5];
    //printf("ch %ld: test = %.2lf\n", i, test);
    std::vector<double> singles;
    for (size_t j = 0; j < 32; j++) {
      sum = 0.0; // initialize our sum
      for (size_t k = 0; k < nsegs; k++) {
	size_t ind = start + j + k*32;
	sum += wf[i][ind] - (wf[i][ind-1]+wf[i][ind+1])/2.0;
      }
      //if (sum!=0) printf(" %4.1lf", sum); 
      singles.push_back(sum);
    }
    if (sum!=0) {
      
      // Now check for spikes
      double combined, largest=0.0;
      size_t spike = 999;
      for (size_t j=0;j<singles.size(); j++) {
	if (j==0)
	  combined = singles[j] - (singles[singles.size()-1] + singles[j+1]); 
	else if (j==singles.size()-1)
	  combined = singles[j] - (singles[j-1] + singles[0]); 
	else 
	  combined = singles[j] - (singles[j-1] + singles[j+1]); 
	if (fabs(combined) > thresh_single && fabs(combined) > largest) {
	  spike = j; 
	  largest = fabs(combined);
	}
      }
      // Found spikes in the waveform
      if (spike < 999) {
	size_t ind = spike+start;
	do {
	  //wf[i][ind] = (wf[i][ind-1] + wf[i][ind+1])/2.0;
	  ind +=32;
	} while ( ind < wf[i].size()-2 ) ;
	//printf("  Sp -> %ld", spike+start);
      }
      //printf("\n");
    }
  } */
  
  // Printing the adjusted wf for demonstration
  //for (const auto& row : wf) {
  //  for (double num : row) {
  //    std::cout << num << " ";
  //  }
  //  std::cout << "\n";
  //}
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
  for (i = 0; i < 10; i++) {
    rsp[i] = -1;
  }
  /* find spikes with special high-pass filters */
  for (j = 0; j < 1024; j++) {
    for (i = 0; i < nChn; i++) {
      filter = -wf[i][j] + wf[i][(j + 1) % 1024] + wf[i][(j + 2) % 1024] - wf[i][(j + 3) % 1024];
      dfilter = filter + 2 * wf[i][(j + 3) % 1024] + wf[i][(j + 4) % 1024] - wf[i][(j + 5) % 1024];
      //::info("filter {}, dfilter {}", filter, dfilter);
      if (filter > 20 && filter < 100) {
        if (n_sp[i] < 10)   // record maximum of 10 spikes
        {
          sp[i][n_sp[i]] = (j + 1) % 1024;
          n_sp[i]++;
        } else {                // too many spikes -> something wrong
          log_warn("Spike cleaning not possible, too many spikes (" << n_sp[i] << ") in ch " << i << "!");
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
          log_warn("Spike cleaning not possible, too many spikes (" << n_sp[i] << ") in ch " << i << "!");
          return;
        }
      }
    }
  }
  for (usize ch=0;ch<9;ch++) {
    // be less verbose for now, FIXME
    //log_info("Found " << n_sp[ch] << " spikes in channel " << ch << "!");
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
  for (k = 0; k < n_rsp; k++) {
    spikes[k] = rsp[k];
    for (i = 0; i < nChn; i++) {
      if (k < n_rsp && fabs(rsp[k] - rsp[k + 1] % 1024) == 2) {
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
bool RBCalibration::serialize_event_data;

RBCalibration::RBCalibration() {
  RBCalibration::serialize_event_data = true;
  rb_id = 0;
  for (usize ch=0;ch<NCHN;ch++) {
    v_offsets.push_back(Vec<f32>(NWORDS, 0));
    v_dips.push_back(Vec<f32>(NWORDS, 0));
    v_incs.push_back(Vec<f32>(NWORDS, 0));
    t_bin.push_back(Vec<f32>(NWORDS,0)) ;
  }
}

/************************************************/

void RBCalibration::disable_eventdata() {
  RBCalibration::serialize_event_data = false;
}

/************************************************/

Vec<Vec<f32>> RBCalibration::voltages    (const RBEvent &event,
                                          bool spike_cleaning) const {
  Vec<Vec<f32>> all_ch_voltages;
  for (u8 ch=1;ch<NCHN+1;ch++) {
    all_ch_voltages.push_back(voltages(event, ch));
  }
  if (spike_cleaning) {
    /*
    int spikes[NWORDS];
    for (usize n=0;n<NWORDS;n++) {
      spikes[n] = 0;
    }
    spike_cleaning_drs4(all_ch_voltages, event.header.stop_cell, spikes);
    */
    //spike_cleaning_simple(all_ch_voltages, true); // true -> calibrated
    spike_cleaning_all(all_ch_voltages, true); // true -> calibrated
  }
  return all_ch_voltages;
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

Vec<f32> RBCalibration::voltages(const RBEvent &event, const u8 channel) const {
  Vec<f32> voltages = Vec<f32>(NWORDS,0);
  if (!(channel_check(channel))) {
    return voltages;
  }
  Vec<u16> adc = event.get_channel_adc(channel);
  if (adc.size() == 0) {
    return voltages;
  }
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

RBCalibration RBCalibration::from_bytestream(const Vec<u8> &stream,
                                             u64 &pos,
                                             bool discard_events) {
  //::set_pattern("[%^%l%$] [%s - %!:%#] [%Y-%m-%d %H:%M:%S] -- %v");
  RBCalibration calibration = RBCalibration();
  log_debug("Start decoding at pos " << pos);
  u16 head = Gaps::parse_u16(stream, pos);
  if (head != RBCalibration::HEAD)  {
    log_warn("No header signature found!");  
    return calibration;
  }
  calibration.rb_id         = stream[pos]; pos += 1;
  calibration.d_v           = Gaps::parse_f32(stream, pos);
  calibration.timestamp     = Gaps::parse_u32(stream, pos);
  bool serialize_event_data = Gaps::parse_bool(stream, pos);
  serialize_event_data 
      = serialize_event_data && RBCalibration::serialize_event_data;  
  f32 value;
  for (usize ch=0; ch<NCHN; ch++) {
    for (usize k=0; k<NWORDS; k++) {
      value = Gaps::parse_f32(stream, pos);
      calibration.v_offsets[ch][k] = value;
      value = Gaps::parse_f32(stream, pos);
      calibration.v_dips[ch][k] = value;
      value = Gaps::parse_f32(stream, pos);
      calibration.v_incs[ch][k] = value;
      value = Gaps::parse_f32(stream, pos);
      calibration.t_bin[ch][k]  = value;
    }
  }
  // FIXME - streamline this
  serialize_event_data = !discard_events;
  u16 n_noi = Gaps::parse_u16(stream, pos);
  if (serialize_event_data) {
    //log_info("Decoding " << n_noi << " no input data events..");
    for (u16 k=0; k<n_noi; k++) {
      auto ev = RBEvent::from_bytestream(stream, pos);
      calibration.noi_data.push_back(ev); 
    }
  } else {
    // we have to advance the number of bytes
    // the events will have 8 channels + ch9
    // by definition
    // FIXME - this number should not be hardcoded!
    pos += n_noi * 18469;
  }
  u16 n_vcal = Gaps::parse_u16(stream, pos);
  if (serialize_event_data) {
    //log_info("Decoding " << n_vcal << " VCAL data events...");
    for (u16 k=0; k<n_vcal; k++) {
      auto ev = RBEvent::from_bytestream(stream, pos);
      calibration.vcal_data.push_back(ev); 
    }
  } else {
    pos += n_vcal * 18469;
  }
  u16 n_tcal = Gaps::parse_u16(stream, pos);
  if (serialize_event_data) {
    //log_info("Decoding " << n_tcal << " TCAL data events...");
    for (u16 k=0; k<n_tcal; k++) {
      auto ev = RBEvent::from_bytestream(stream, pos);
      calibration.tcal_data.push_back(ev); 
    }
  } else {
    pos += n_tcal * 18469;
  }
  u16 tail = Gaps::parse_u16(stream, pos);
  if (tail != RBEvent::TAIL) {
    log_error("After parsing, we found an invalid tail signature " << tail);
  }
  return calibration;
}

/************************************************/

RBCalibration RBCalibration::from_file(const String &filename, bool discard_events) {
    auto cali_pack = get_tofpackets(filename)[0];
    u64 pos = 0;
    RBCalibration cali = RBCalibration::from_bytestream(cali_pack.payload, pos, discard_events);
    return cali;
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
    log_fatal("Can't open " << filename);
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
  log_info("Loaded calibration for RB " << rb_id << "!");
  return calibration;
}

/************************************************/

bool RBCalibration::channel_check(u8 channel) const {
  if (channel == 0) {
    log_error("Remember, channels start at 1. 0 does not exist!");
    return false;
  }
  if (channel > 9) {
    log_error("Thera are no channels > 9!");
    return false;
  }
  return true;
}

/************************************************/

std::string RBCalibration::to_string() const {
  std::string repr = "<ReadoutboardCalibration:";  
  repr += "\n RB             : " + std::to_string(rb_id);
  bool has_data = false;
  if (vcal_data.size() > 0) {
    repr += "\n VCalData       : " + std::to_string(vcal_data.size()) + " (events)";
    has_data = true;
  }
  if (tcal_data.size() > 0) {
    repr += "\n TCalData       : " + std::to_string(tcal_data.size()) + " (events)";
    has_data = true;
  }
  if (noi_data.size() > 0) {
    repr += "\n NoInputData    : " + std::to_string(tcal_data.size()) + " (events)"; 
    has_data = true;
  }
  if (!has_data) {
    repr += "\n .. no calibration data (RBEvents) loaded/available ..";
  }
  repr += "\n V Offsets [ch0]: .. " + std::to_string(v_offsets[0][98]) + " " + std::to_string(v_offsets[0][99]) + ".."
          "\n V Incrmts [ch0]: .. " + std::to_string(v_incs   [0][98]) + " " + std::to_string(v_incs   [0][99]) + ".."
          "\n V Dips    [ch0]: .. " + std::to_string(v_dips   [0][98]) + " " + std::to_string(v_dips   [0][99]) + ".."
          "\n T Bins    [ch0]: .. " + std::to_string(t_bin    [0][98]) + " " + std::to_string(t_bin    [0][99]) + "..>";
  return repr;
}

/************************************************/

std::ostream& operator<<(std::ostream& os, const RBCalibration& cali){ 
  os << cali.to_string();
  return os;
}

