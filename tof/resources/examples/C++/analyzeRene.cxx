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
  ("f,files", "List of Files", cxxopts::value<bool>()->default_value("false"))
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
  bool files   = result["files"].as<bool>();
  bool verbose = result["verbose"].as<bool>();

  FILE *fp;
  char tmpline[500];
  std::string fnames[1000];
  int j=0;
  if (files) {
    fp = fopen(fname.c_str(), "r");
    while (fscanf(fp, "%s", tmpline) != EOF) fnames[j++] = tmpline;
    fclose(fp);
  } else {
    fnames[j++] = fname;
  }

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

  char label[50], line[500];
  int status;
  float value;
  // One last task before reading the data file processing events
  // -- read in some analysis parameters.
  // Doing this in a kludgy way since we will not use later.
  fp = fopen("paramNEVIS.txt", "r");
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

  // Another kludgy read is getting the RB-ch to paddle map from the
  // rbch-vs-paddle.json file. Achim has a way to do this via rust,
  // but I need the map for development purposes here.
  int paddle_map[NRB][NCH] = { 0 }; // Stored value will be paddle ID;
  int rb_num, ch_num;
  fp = fopen("/home/gaps/software/gaps-online-software/src/gaps-db/resources/master-spreadsheet/rbch-vs-paddle.json", "r");
  if ( fscanf(fp, "%s", label) != EOF ) { // Read in first "{"
    while (fscanf(fp, "%s %[^\n]", label, line) != EOF) { 
      if (strncmp(label, "\"", 1) == 0) { // Found an RB line
	int rb_len = strlen(label)-3;     // RB<=9 or RB>=10
	char tmp[6];
	if (rb_len == 1) snprintf(tmp, sizeof(tmp), "%.1s", label+1);
	if (rb_len == 2) snprintf(tmp, sizeof(tmp), "%.2s", label+1);
	rb_num = atoi(tmp);
	for(int i=0;i<NCH;i++) {
	  fscanf(fp, "%s %s", label, line);
	  snprintf(tmp, sizeof(tmp), "%.4s", line);
	  int pad_id = atoi(tmp);
	  //printf("RBnum = %s %s %d; %d %d\n",label,line,rb_num,i,pad_id);
	  paddle_map[rb_num][i] = pad_id;
	}
	fscanf(fp, "%s", line); // read in the closing "}" for RB
      }
    }
  }
  fclose(fp); // Finished with file
  
  // Another kludgy read is getting the Paddle to volume location from
  // the paddleid_vs_volid.json adn level0_coordinates.json
  // files. Achim has a way to do this via rust, but I need the map
  // for development purposes here.
  float paddle_location[NPAD][3] = { 0 }; // X, Y, Z coords in detector
  int   paddle_vid[NPAD] = { 0 }; // VID with same counter
  // First, read the paddle to volume ID map
  int tmp_pad, tmp_vol, vol_id[NPAD] = { 0 }; 
  int tmp_vid;
  float tmp_x, tmp_y, tmp_z;
  fp = fopen("/home/gaps/software/gaps-online-software/src/gaps-db/resources/master-spreadsheet/paddleid_vs_volid.json", "r");
  if ( fscanf(fp, "%s", label) != EOF ) { // Read in first "{"
    while (fscanf(fp,"%*[^-0-9]%d  %*[^-0-9] %d", &tmp_pad, &tmp_vol) != EOF) { 
      vol_id[tmp_pad] = tmp_vol;
      //printf("%d %d\n", tmp_pad, vol_id[tmp_pad]);
    }
  }
  fclose(fp); // Finished with file
  // Now that we have the vol_id for each paddle, map read in the
  // vol_id to location map.
  fp = fopen("/home/gaps/software/gaps-online-software/src/gaps-db/resources/master-spreadsheet/level0_coordinates.json", "r");
  int ctr=0;
  if ( fscanf(fp, "%s", label) != EOF ) { // Read in first "{"
    while (fscanf(fp,"%*[^-0-9]%d  %*[^-0-9]%f %*[^-0-9]%f  %*[^-0-9]%f ",
		  &tmp_vid, &tmp_x, &tmp_y, &tmp_z) != EOF) { 
      //printf("%d %.2f %.2f %.2f\n", tmp_vid, tmp_x, tmp_y, tmp_z);
      paddle_vid[ctr]         = tmp_vid;
      paddle_location[ctr][1] = tmp_x;
      paddle_location[ctr][2] = tmp_y;
      paddle_location[ctr++][3] = tmp_z;
    }
  }  
  fclose(fp); // Finished with file
  
  // Instantiate our class that holds analysis results and set some
  // initial values
  auto Event = EventGAPS();
  Event.SetPaddleMap(paddle_map, vol_id, paddle_vid, paddle_location);
  Event.SetThreshold(CThresh);
  Event.SetCFDFraction(CFDS_frac);
  Event.InitializeHistograms();

  // the reader is something for the future, when the 
  // files get bigger so they might not fit into memory
  // at the same time
  //auto reader = Gaps::TofPacketReader(fname); 
  // for now, we have to load the whole file in memory

  u32 n_rbcalib = 0;
  u32 n_rbmoni  = 0;
  u32 n_mte     = 0;
  u32 n_tcmoni  = 0;
  u32 n_mtbmoni = 0;
  u32 n_unknown = 0;
  u32 n_tofevents = 0;

  for (int k=0; k<j; k++) { 
    auto packets = get_tofpackets(fnames[k]);
    spdlog::info("We loaded {} packets from {}", packets.size(), fnames[k]);

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
	for (int i=0;i<NTOT;i++) wave[i] = NULL;
	for (int i=0;i<NRB;i++)  wch9[i] = NULL;
	
        auto ev = TofEvent::from_bytestream(p.payload, pos);
	unsigned long int evt_ctr = ev.mt_event.event_id;
	//printf("%ld.", evt_ctr);
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

	// Now that we have the waveforms in place, analyze the event.
	Event.InitializeVariables(evt_ctr);
	Event.InitializeWaveforms(wave, wch9);

	// Calculate and store pedestals/RMSs for each channel
	Event.AnalyzePedestals(Ped_low, Ped_win);

	// Analyze the pulses in each channel
	Event.SetThreshold(CThresh);
	Event.SetCFDFraction(CFDS_frac);
	Event.AnalyzePulses(Qwin_low, Qwin_size);
	for (int i=0;i<NTOT;i++) {
	  float tdc = Event.GetTDC(i);
	  //if (tdc > 5) printf("%ld: %d -> %.2f\n", evt_ctr, i, tdc);
	}
	
	// Analyze each paddle: position on paddle, hitmask, etc
	Event.AnalyzePaddles(10.0, 5.0); //Args: Peak and Charge cuts

	// Now calculate beta, charge, and inner/outer tof x,y,z, etc.
	Event.AnalyzeEvent();
	
	// Now fill out histograms
	Event.FillChannelHistos();
	Event.FillPaddleHistos();

	Event.UnsetWaveforms();
	
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
  }
  // Write histograms after analyzing all the files
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
