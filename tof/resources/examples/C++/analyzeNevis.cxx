/**
 * Binary to unpack tofpackets/raw rb data to illustrate 
 * how to work with teh API
 * 
 * September 2023, gaps-online-sw V0.7
 * The API will not be stable until V1.0 and is thus 
 * subject to change. Please refer to the respective 
 * README.md
 *
 */

#include <iostream>
#include "cxxopts.hpp"

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "io.hpp"
#include "calibration.h"

#include "legacy.h"
#include <vector>

#include "./include/EventGAPS.h"

const int NRB   = 50; // Technically, it is 49, but we don't use 0
const int NCH   = 8;
const int NTOT  = NCH * NRB; // NTOT is the number of SiPMs
const int NPADS = NTOT/2;        // NPAD: 1 per 2 SiPMs

int main(int argc, char *argv[]){
  spdlog::cfg::load_env_levels();
    
  cxxopts::Options options("unpack-tofpackets", "Unpack example for .tof.gaps files with TofPackets.");
  options.add_options()
  ("h,help", "Print help")
  ("c,calibration", "Calibration file (in txt format)", cxxopts::value<std::string>()->default_value(""))
  ("file", "A file with TofPackets in it", cxxopts::value<std::string>())
  ("v,verbose", "Verbose output", cxxopts::value<bool>()->default_value("false"))
  ;
  options.parse_positional({"file"});
  auto result = options.parse(argc, argv);
  if (result.count("help")) {
    std::cout << options.help() << std::endl;
    exit(EXIT_SUCCESS);
  }
  if (!result.count("file")) {
    spdlog::error("No input file given!");
    std::cout << options.help() << std::endl;
    exit(EXIT_FAILURE);
  }
  auto fname   = result["file"].as<std::string>();
  bool verbose = result["verbose"].as<bool>();
  // -> Gaps relevant code starts here
 
  auto calname = result["calibration"].as<std::string>();
  RBCalibration cali[NRB]; // "cali" stores values for one RB

  // To read calibration data from individual binary files, when -c is
  // given with the directory of the calibration files
  if (calname != "") {
    for (int i=1; i<NRB; i++) {
      // First, determine the proper RB filename from its number
      std::string f_str;
      if (i<10) // Little Kludgy, but it works
	f_str = calname + "rb_0" + std::to_string(i) + ".cali.tof.gaps";
      else
	f_str = calname + "rb_" + std::to_string(i) + ".cali.tof.gaps";
      //spdlog::info("Extracting RB data from file {}", f_str);
      
      // Read the packets from the file
      //if ( std::filesystem::exists(f_str) ) {
      //printf("%s file exists\n", f_str.c_str() );
      //}
      // Before proceeding, check that the file exists. 
      struct stat buffer; 
      if ( stat(f_str.c_str(), &buffer) != -1 ) {
	auto packet = get_tofpackets(f_str);
	spdlog::info("We loaded {} packets from {}", packet.size(), f_str);
	// Loop over the packets (should only be 1) and read into storage
	for (auto const &p : packet) {
	  int ctr=0;
	  if (p.packet_type == PacketType::RBCalibration) {
	    // Should have the one calibration tofpacket stored in "packet".
	    usize pos = 0;
	    if (++ctr == 4)  // 4th packet is the one we want
	      cali[i] = RBCalibration::from_bytestream(p.payload, pos); 
	  }
	}
      } //else {printf("File does not exist: %s\n", f_str.c_str());}
    }
  }
  
  // To read calibration data from individual text files, when -c is
  // given with the directory of the calibration files
  /*if (calname != "") {
    // obviously here we have to get all the calibration files, 
    // but for the sake of the example let's use only one
    // Ultimatly, they will be stored in the stream.
    for (int i=1; i<NRB; i++) {
      std::string f_str;
      if (i<10) // Little Kludgy, but it works
	f_str = calname + "/txt-files/rb0" + std::to_string(i) + "_cal.txt";
      else
	f_str = calname + "/txt-files/rb" + std::to_string(i) + "_cal.txt";
      
      //spdlog::info("Will use calibration file {}", calname);
      //cali[i] = RBCalibration::from_txtfile(calname);
      spdlog::info("Will use calibration file {}", f_str);
      cali[i] = RBCalibration::from_txtfile(f_str);
    }
    }*/

  // Some useful variables (some initialized to default values
  float Ped_low   = 10;
  float Ped_win   = 90;
  float CThresh   = 5.0;
  float CFDS_frac = 0.40;
  float Qwin_low  = 100;
  float Qwin_size = 100;
  float CHmin     = 4.0;
  float Ped[NTOT];
  float PedRMS[NTOT];
  float Qint[NTOT];
  float VPeak[NTOT];
  float TCFDS[NTOT];
  bool  IsHit[NTOT] = {false} ;

  char label[50], line[500];
  int status;
  float value;
  // One last task before reading the data file processing events
  // -- read in some analysis parameters.
  // Doing this in a kludgy way since we will not use later.
  FILE *fp = fopen("paramNEVIS.txt", "r");
  while (fscanf(fp, "%s %f", label, &value) != EOF) {
    if (strcmp(label,"ped_lo") ==0 )     Ped_low = value; 
    if (strcmp(label,"ped_win") ==0 )    Ped_win = value;
    if (strcmp(label,"pulse_lo") ==0 )   Qwin_low = value;
    if (strcmp(label,"pulse_win") ==0 )  Qwin_size = value;
    if (strcmp(label,"charge_min") ==0 ) CHmin = value;
    if (strcmp(label,"thresh") ==0 )     CThresh = value;
    if (strcmp(label,"cfd_frac") ==0 )   CFDS_frac = value;
    status = fscanf(fp,"%[^\n]",line); // Scan the rest of the line
  }
  fclose(fp); 

  // the reader is something for the future, when the 
  // files get bigger so they might not fit into memory
  // at the same time
  //auto reader = Gaps::TofPacketReader(fname); 
  // for now, we have to load the whole file in memory
  auto packets = get_tofpackets(fname);
  spdlog::info("We loaded {} packets from {}", packets.size(), fname);

  u32 n_rbcalib = 0;
  u32 n_rbmoni  = 0;
  u32 n_mte     = 0;
  u32 n_tcmoni  = 0;
  u32 n_mtbmoni = 0;
  u32 n_unknown = 0;
  u32 n_tofevents = 0;

  for (auto const &p : packets) {
    // print it
    //std::cout << p << std::endl;
    // there will be a more generic way to unpack TofPackets in the future
    // for now we have to use the packet_type field
    switch (p.packet_type) {
      case PacketType::RBCalibration : {
	// if you have the packet payload, the second argument 
	// (position in stream) will always be 0
	//
	// pos keeps track of the current position in bytestream, 
	// thus passed by reference so we need an rvalue
	//
	// the usize is a typedef from tof_typedefs.h and used
	// to make the rust and C++ code look more similar, so that 
	// is easier to compare them.
	usize pos = 0;
	auto cali = RBCalibration::from_bytestream(p.payload, pos);
	if (verbose) {
	  std::cout << cali << std::endl;
	}
	n_rbcalib++;
      break;
    }
      // this only works for the data I combined
      // recently, NOT for the "stream" kind of data
      // THe format will change as well soon.
      case PacketType::TofEvent : {

        usize pos = 0;
	// We need a structure to hold the waveforms for an event. We
	// initialize and delete them with each new event
	GAPS::Waveform *wave[NTOT];
	GAPS::Waveform *wch9[NRB];
	
        auto ev = TofEvent::from_bytestream(p.payload, pos);
	unsigned long int evt_ctr = ev.mt_event.event_id;
	//printf("Event %ld: RBs -", evt_ctr);
	for (auto const &rbid : ev.get_rbids()) {
	  printf("Getting RB %d event data\n", rbid); fflush(stdout);
	  RBEvent rb_event = ev.get_rbevent(rbid);
	  // Now that we know the RBID, we can set the starting ch_no
	  // Eventually we will use a function to map RB_ch to GAPS_ch
	  usize ch_start = (rbid-1)*NCH; // first RB is #1
	  if (verbose) {
	    std::cout << rb_event << std::endl;
          }
	  Vec<Vec<f32>> volts;
	  Vec<Vec<f32>> times;
	  //if ((calname != "") && cali.rb_id == rbid ){
	  if (calname != "") { // For combined data all boards calibrated
	    // Vec<f32> is a typedef for std::vector<float32>
	    volts = cali[rbid].voltages(rb_event, false); //second argument is for spike cleaning
	    // (C++ implementation causes a segfault sometimes when "true"
	    times = cali[rbid].nanoseconds(rb_event);
	    // volts and times are now ch 0-8 with the waveform for this event.

	    // First, store the waveform for channel 9
	    Vec<f64> ch9_volts(volts[8].begin(), volts[8].end());
	    Vec<f64> ch9_times(times[8].begin(), times[8].end());
	    //wch9[rbid] = new GAPS::Waveform(ch9_volts.data(),ch9_times.data(),rbid,0);
	    //wch9[rbid]->SetPedBegin(Ped_low);
	    //wch9[rbid]->SetPedRange(Ped_win);
	    //wch9[rbid]->CalcPedestalRange(); 
	    //float ch9RMS = wch9[rbid]->GetPedsigma();
	    //printf(" %d(%.1f)", rbid, ch9RMS);
	    // printf(" %d", rbid);
	      
	    // Now, deal with all the SiPM data
	    for(int c=0;c<NCH;c++) {
	      usize cw = c+ch_start; 

	      Vec<f64> ch_volts(volts[c].begin(), volts[c].end());
	      Vec<f64> ch_times(times[c].begin(), times[c].end());
	      //wave[cw] = new GAPS::Waveform(ch_volts.data(),ch_times.data() ,cw,0);
	      
	      // Calculate the pedestal
	      /*wave[cw]->SetPedBegin(Ped_low);
	      wave[cw]->SetPedRange(Ped_win);
	      wave[cw]->CalcPedestalRange(); 
	      wave[cw]->SubtractPedestal(); 
	      Ped[cw] = wave[cw]->GetPedestal();
	      PedRMS[cw] = wave[cw]->GetPedsigma();
	      if ( c==0 && (PedRMS[cw] > 15) && (ch9RMS < 190) ) {
		// RMS_ch1 has ch9 data && RMS_ch9 has normal data        
		printf(" %ld Row %d: %8.1f %8.1f\n", evt_ctr, rbid, ch9RMS, PedRMS[cw]);
		}*/

	      // Set thresholds and find pulses
	      /*wave[cw]->SetThreshold(CThresh);
	      wave[cw]->SetCFDSFraction(CFDS_frac);
	      VPeak[cw] = wave[cw]->GetPeakValue(Qwin_low, Qwin_size);
	      Qint[cw]  = wave[cw]->Integrate(Qwin_low, Qwin_size);
	      wave[cw]->FindPeaks(Qwin_low, Qwin_size);
	      //if ( (wave[cw]->GetNumPeaks() > 0) && (Qint[cw] > 5.0) ) {
	      if ( (wave[cw]->GetNumPeaks() > 0) ) {
		wave[cw]->FindTdc(0, GAPS::CFD_SIMPLE);       // Simple CFD
		TCFDS[cw] = wave[cw]->GetTdcs(0);
		printf("EVT %12ld - ch %3ld: %10.5f -- %.2f\n", evt_ctr, cw, TCFDS[cw], wave[cw]->GetPedsigma());
	      }	*/	
	    }
	  }
	}
	printf("\n");
	// Now that we have all the waveforms in place, we can analyze
	// the event.
	//auto Event = EventGAPS(wave, wch9);
	
	for (int k=0; k<NPADS; k++) {
	  //First, check with the MTB data to see if paddle is hit

	  // Then find the pulse information from each SiPM
	  int ch0 = k*2, ch1 = k*2+1;
	  /*
	  Paddle[k].time_a = wave[ch0].FindTdc(1,CFD_SIMPLE);
	  Paddle[k].time_b = wave[ch1].FindTdc(1,CFD_SIMPLE);
	  Paddle[k].peak_a = wave[ch0].GetPeakValue(100, 200);
	  Paddle[k].peak_b = wave[ch1].GetPeakValue(100, 200);
	  Paddle[k].charge_a = wave[ch0].Integrate(100, 200);
	  Paddle[k].charge_b = wave[ch1].Integrate(100, 200);
	  // Also hit position, min_i, and t_avg
	  */
	}
	// Now calculate beta, charge, and inner/outer tof x,y,z

	n_tofevents++;
        break;
      }
      case PacketType::RBMoni : {
        usize pos = 0;
        auto moni = RBMoniData::from_bytestream(p.payload, pos);
        if (verbose) {
          std::cout << moni << std::endl;
        }
        n_rbmoni++;
        break;
      }
      case PacketType::MasterTrigger : {
        usize pos = 0;
        auto mte = MasterTriggerEvent::from_bytestream(p.payload, pos);
        if (verbose) {
          std::cout << mte << std::endl;
        }
        n_mte++;
        break;
      }
      case PacketType::MTBMoni : {
        usize pos = 0;
        auto mtbmoni = MtbMoniData::from_bytestream(p.payload, pos);
        if (verbose) {
          std::cout << mtbmoni << std::endl;
        }
        n_mtbmoni++;
        break;
      }
      default : {
        if (verbose) {
          std::cout << "-- nothing to do for " << p.packet_type << " --" << std::endl;
        }
        n_unknown++;
        break;
      }
    }
  }
  
  std::cout << "-- -- packets summary:" << std::endl;
  
  std::cout << "-- -- RBCalibration     : " << n_rbcalib << "\t (packets) " <<  std::endl;
  std::cout << "-- -- RBMoniData        : " << n_rbmoni  << "\t (packets) " <<  std::endl;
  std::cout << "-- -- MasterTriggerEvent: " << n_mte     << "\t (packets) " <<  std::endl;
  std::cout << "-- -- TofEvent          : " << n_tofevents  << "\t (packets) " <<  std::endl;
  std::cout << "-- -- TofCmpMoniData    : " << n_tcmoni  << "\t (packets) " <<  std::endl;
  std::cout << "-- -- MtbMoniData       : " << n_mtbmoni << "\t (packets) " <<  std::endl;
  std::cout << "-- -- undecoded         : " << n_unknown << "\t (packets) " <<  std::endl;

  spdlog::info("Finished");
  return EXIT_SUCCESS;
}
