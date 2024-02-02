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

#include "./include/constants.h"
#include "./include/EventGAPS.h"


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
  bool RB_Calibrated[NRB] = { false };
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
	  //int ctr=0;
	  if (p.packet_type == PacketType::RBCalibration) {
	    // Should have the one calibration tofpacket stored in "packet".
	    usize pos = 0;
	    //if (++ctr == 4)  // 4th packet is the one we want
	    cali[i] = RBCalibration::from_bytestream(p.payload, pos); 
	    RB_Calibrated[i] = true;
	  }
	}
      } 
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

  // Some useful variables (some initialized to default values)
  // but overwritten from file (if it exists)
  float Ped_low   = 10;
  float Ped_win   = 90;
  float CThresh   = 5.0;
  float CFDS_frac = 0.40;
  float Qwin_low  = 100;
  float Qwin_size = 100;
  float CHmin     = 4.0;

  // Some useful analysis quantities
  float Ped[NTOT];
  float PedRMS[NTOT];
  float Qint[NTOT];
  float VPeak[NTOT];
  float TCFDS[NTOT];
  bool  IsHit[NTOT] = {false} ;

  // Instantiate our class that holds analysis results and set some
  // initial values
  auto Event = EventGAPS();
  Event.SetThreshold(CThresh);
  Event.SetCFDFraction(CFDS_frac);
  Event.InitializeHistograms();
  
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
	printf("%ld.", evt_ctr);
	for (auto const &rbid : ev.get_rbids()) {
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
	  //if (calname != "") { // For combined data all boards calibrated
	  if (RB_Calibrated[rbid]) { // Have cali data for this RBID
	    // Vec<f32> is a typedef for std::vector<float32>
	    volts = cali[rbid].voltages(rb_event, false); //second argument is for spike cleaning
	    // (C++ implementation causes a segfault sometimes when "true"
	    times = cali[rbid].nanoseconds(rb_event);
	    // volts and times are now ch 0-8 with the waveform for this event.

	    // First, store the waveform for channel 9
	    Vec<f64> ch9_volts(volts[8].begin(), volts[8].end());
	    Vec<f64> ch9_times(times[8].begin(), times[8].end());
	    wch9[rbid] = new GAPS::Waveform(ch9_volts.data(),
					    ch9_times.data(), rbid,0);
	    //printf(" %d", rbid);
	    
	    // Now, deal with all the SiPM data
	    for(int c=0;c<NCH;c++) {
	      usize cw = c+ch_start; 
	      
	      Vec<f64> ch_volts(volts[c].begin(), volts[c].end());
	      Vec<f64> ch_times(times[c].begin(), times[c].end());
	      wave[cw] = new GAPS::Waveform(ch_volts.data(),
					    ch_times.data(), cw,0);
	    }
	  }
	}
	//printf("\n");

	// Now that we have all the waveforms in place, we can analyze
	// the event.
	Event.InitializeWaveforms(wave, wch9);
	// Calculate and store pedestals/RMSs for each channel
	Event.AnalyzePedestals(Ped_low, Ped_win);
	// Analyze the pulses in each channel
	Event.SetThreshold(CThresh);
	Event.SetCFDFraction(CFDS_frac);
	Event.AnalyzePulses(Qwin_low, Qwin_size);
	
	// Now calculate beta, charge, and inner/outer tof x,y,z, etc.

	// Now fill out histograms
	Event.FillChannelHistos();
	Event.FillPaddleHistos();

	Event.UnsetWaveforms();
	// Run 54 bombs around event 2192924 so this provides a way to
	// write the file and exit before that.
	
	/*if(evt_ctr == 2192924) {
	Event.WriteHistograms();
	return (0);
	} */
	
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
  
  Event.WriteHistograms();
  
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
