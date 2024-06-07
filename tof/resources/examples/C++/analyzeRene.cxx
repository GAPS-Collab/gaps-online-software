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
#include "./include/EventRene.h"

void   GetPaddleInfo(struct PaddleInfo *pad, struct SiPMInfo *sipm);
double FitSine(std::vector<double> volts, std::vector<double> times);

int main(int argc, char *argv[]){
  spdlog::cfg::load_env_levels();
    
  cxxopts::Options options("unpack-tofpackets", "Unpack example for .tof.gaps files with TofPackets.");
  options.add_options()
  ("h,help", "Print help")
  ("c,calibration", "Calibration file (in txt format)", cxxopts::value<std::string>()->default_value("/mnt/tof-nas/nevis-data/tofdata/calibration/latest/"))
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
    if (fp != NULL) {
      while (fscanf(fp, "%s", tmpline) != EOF) fnames[j++] = tmpline;
      fclose(fp);
    } else {
      printf("Unable to open file %s\n", fname.c_str());
    }
  } else {
    fnames[j++] = fname;
  }

  // Print out the filenames as a sanity check
  //std::cout << fnames[0] << std::endl;
  //for(int k=1;k<j;k++) std::cout << fnames[k] << std::endl;
  //return (0);
  
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
  float CFDS_frac = 0.10;
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
  if (fp != NULL) { // Actually opened file
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
  } else
    printf("Using default Nevis parameters.\n");
  
  
  // Now, we want to store information about the SiPM channels and
  // paddle relationships for analysis purpose. Read all that info
  // into the relevant structures.
  struct PaddleInfo PadInfo;
  struct SiPMInfo   SipmInfo;
  GetPaddleInfo(&PadInfo, &SipmInfo);
      
  // Instantiate our class that holds analysis results and set some
  // initial values
  auto Event = EventGAPS();
  //Event.SetPaddleMap(paddle_map, vol_id, paddle_vid, paddle_location);
  Event.SetPaddleMap(&PadInfo, &SipmInfo);
  Event.SetThreshold(CThresh);
  Event.SetCFDFraction(CFDS_frac);
  Event.InitializeHistograms();
  //return (0);
  
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
	float Phi[NRB];
	for (int i=0;i<NRB;i++) Phi[i] = -999.0;
	
        auto ev = TofEvent::from_bytestream(p.payload, pos);
	unsigned long int evt_ctr = ev.mt_event.event_id;
	//printf("Event %ld: RBs -", evt_ctr);
	//printf("%ld.", evt_ctr);
	/*for (int k=0;k<NRB;k++) {
	  if (k%9==0) printf("\n");
	  int n = ev.rb_events[k].header.rb_id;
	  printf(" %3d(%3d)", n, ev.rb_events[k].header.channel_mask);
	}*/
	//Vec<std::tuple<u8,u8,u8>> ltbmap = ev.mt_event.get_dsi_j_ch();
	//std::cout << get<0>(ltbmap[1]) << get<1>(ltbmap[2])
	//	  << get<2>(ltbmap[1]) << std::endl;
	//for (auto const& ltbmapi : ltbmap) {
	//std::cout << std::get<1>(ltbmapi) <<" "<< std::get<2>(ltbmapi)<<" ";
	  //for (auto k = std::begin(ltbmap); k != std::end(ltbmap); ++k) {
	  //  std::cout << std::get<1>(*k) << " "<< std::get<2>(*k)<< " ";
	//}
	//printf(" %3d(%3d)", k, ev.mt_event.board_mask[k]);
	
	for (auto const &rbid : ev.get_rbids()) {
	  RBEvent rb_event = ev.get_rbevent(rbid);
	  // Now that we know the RBID, we can set the starting ch_no
	  // Eventually we will use a function to map RB_ch to GAPS_ch
	  usize ch_start = (rbid-1)*NCH; // first RB is #1
          // Let's also store the channel mask to use later.               
          int ch_mask = rb_event.header.channel_mask;
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
	    // Before making waveforms, lets calculate the ch9
	    // phase. For now, if we have ch9 data for this RB, we
	    // want to analyze it.
	    Phi[rbid] = FitSine(ch9_volts,ch9_times);
	    // Now, initialize the ch9 Waveform for this RB. 
	    wch9[rbid] = new GAPS::Waveform(ch9_volts.data(),
					    ch9_times.data(), rbid,0);
	    //printf(" %d", rbid);
	    
	    // Now, deal with all the SiPM data
	    for(int c=0;c<NCH;c++) {
	      usize cw = c+ch_start; 
              unsigned int inEvent = ch_mask & (1 << c);
              if (inEvent > 0 ) {
		Vec<f64> ch_volts(volts[c].begin(), volts[c].end());
		Vec<f64> ch_times(times[c].begin(), times[c].end());
		wave[cw] = new GAPS::Waveform(ch_volts.data(),
					      ch_times.data(), cw,0);
	      }
	    }
	  }
	}
	//printf("\n");

	// Now that we have the waveforms in place, analyze the event.
	Event.InitializeVariables(evt_ctr, CThresh, CHmin);
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
	// Now that we have TDC values available, process the ch9 phases
	Event.AnalyzePhases(Phi);
	
	// Analyze each paddle: position on paddle, hitmask, etc
	//Event.AnalyzePaddles(10.0, CHmin); //Args: Peak and Charge cuts
	Event.AnalyzePaddles();

	// Now calculate beta, charge, and inner/outer tof x,y,z, etc.
	Event.AnalyzeEvent();
	
	// Now fill out histograms
	Event.FillChannelHistos(0);
	Event.FillPaddleHistos();

	Event.UnsetWaveforms();
	for (int i=0;i<NTOT;i++) {delete wave[i]; wave[i] = NULL;}
	for (int i=0;i<NRB;i++)  {delete wch9[i]; wch9[i] = NULL;}
	
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

void GetPaddleInfo(struct PaddleInfo *pad, struct SiPMInfo *sipm) {
  // Eventually we will call the db to get all this info. For now, I
  // will simple read the relevant files to get the info.

  FILE *fp;
  char label[50], line[500];
  char srcdir[200] = "/home/gaps/software/gaps-online-software/";
  char codedir[200] = "src/gaps-db/resources/master-spreadsheet/";
  char fname[501];
  int status;
  float value;

  // Another kludgy read is getting the Paddle to volume location from
  // the paddleid_vs_volid.json adn level0_coordinates.json
  // files. Achim has a way to do this via rust, but I need the map
  // for development purposes here.
  // First, read the paddle to volume ID map
  int tmp_pad, tmp_vol, vol_id[NPAD] = { 0 }; 
  int tmp_vid;
  float tmp_x, tmp_y, tmp_z;
  float tmp_dimx, tmp_dimy, tmp_dimz;
  snprintf(fname, 500, "%s/%s/paddleid_vs_volid.json", srcdir, codedir);
  fp = fopen(fname, "r");
  if ( fscanf(fp, "%s", label) != EOF ) { // Read in first "{"
    while (fscanf(fp,"%*[^-0-9]%d  %*[^-0-9] %d", &tmp_pad, &tmp_vol) != EOF) {
      vol_id[tmp_pad] = tmp_vol;
      pad->VolumeID[tmp_pad] = tmp_vol; // Assign the paddle volume ID
      //printf("%d %d\n", tmp_pad, vol_id[tmp_pad]);
    }
  }
  fclose(fp); // Finished with file
  
  // Now that we have the vol_id for each paddle, map read in the
  // vol_id to location map.
  snprintf(fname, 500, "%s/%s/level0_coordinates.json", srcdir, codedir);
  fp = fopen(fname, "r");
  int ctr=0;
  if ( fscanf(fp, "%s", label) != EOF ) { // Read in first "{"
    while (fscanf(fp,"%*[^-0-9]%d ", &tmp_vid) != EOF) { // Read VolID
      // For each paddle, we want to set the X, Y, Z locations. So,
      // index through the volume IDs to find a match, then set the
      // appropriate dimensions and locations.
      if (tmp_vid > 10000) { // Valid Volume ID
	for (int j=0; j<NPAD; j++) {
	  if (tmp_vid == pad->VolumeID[j]) { // Found a match, pad = j
	    status = fscanf(fp,"%*[^-0-9]%f %*[^-0-9]%f  %*[^-0-9]%f ",
			    &tmp_x, &tmp_y, &tmp_z);
	    status = fscanf(fp,"%*[^-0-9]%f %*[^-0-9]%f  %*[^-0-9]%f ",
			    &tmp_dimx, &tmp_dimy, &tmp_dimz);
	    pad->Location[j][0] = tmp_x;
	    pad->Location[j][1] = tmp_y;
	    pad->Location[j][2] = tmp_z;
	    pad->Dimension[j][0] = tmp_dimx;
	    pad->Dimension[j][1] = tmp_dimy;
	    pad->Dimension[j][2] = tmp_dimz;
	  }
	}
      }
    }
  }  
  fclose(fp); // Finished with file
  
  int tmp_o;
  float coax, harting;
  // One last task: Get the paddle orientation from paddle_to_orientation.json
  snprintf(fname, 500, "%s/%s/paddle_orient_cable.jaz", srcdir, codedir);
  fp = fopen(fname, "r");
  if ( fscanf(fp, "%s", label) != EOF ) { // Read in first "{"
    while (fscanf(fp,"%*[^-0-9]%d  %*[^-0-9]%d %*[^-0-9]%f  %*[^-0-9]%f ",
		  &tmp_pad, &tmp_o, &coax, &harting) != EOF) {
      if (tmp_pad > 0) {
	pad->Orientation[tmp_pad] = tmp_o;
	pad->CoaxLen[tmp_pad]     = coax;
	pad->HardingLen[tmp_pad]  = harting;
	//printf("%3d %2d %8.3f %8.3f\n", tmp_pad, tmp_o, coax, harting);
      }
    }
  }

  // Kludgy read to get the RB-ch to paddle map from the
  // rbch-vs-paddle.json file. Achim has a way to do this via rust,
  // but I need the map for development purposes here.
  int paddle_map[NRB][NCH] = { 0 }; // Stored value will be paddle ID;
  int rb_num, rb_ch, ch_num, pad_id;
  snprintf(fname, 500, "%s/%s/rbch-vs-paddle.json", srcdir, codedir);
  fp = fopen(fname, "r");
  if ( fscanf(fp, "%s", label) != EOF ) { // Read in first "{"
    while (fscanf(fp, "%*[^-0-9]%d  %[^\n]", &rb_num, line) != EOF) { 
      if (rb_num>0 && rb_num<50) {
	for(int i=0;i<NCH;i++) {
	  status = fscanf(fp, "%*[^-0-9]%d  %*[^-0-9]%d ", &rb_ch, &pad_id);
	  // Store the SiPM Channel for each Paddle end
	  int paddle = pad_id % 1000;
	  int ch_num = (rb_num-1)*NCH + (rb_ch)-1; // Map the value to NTOT
	  sipm->RB[ch_num] = rb_num;
	  sipm->RB_ch[ch_num] = rb_ch;
	  sipm->PaddleID[ch_num] = paddle;
	  if (pad_id > 2000) { // We have a paddle ID for B
	    pad->SiPM_B[paddle] = ch_num;
	    sipm->PaddleEnd[ch_num] = 1;
	    //printf("B -> %d %d %d %d %d\n", i,j,ch_num, paddle,paddle_map[i][j]);
	  } else if (pad_id > 1000) { //We have a paddle ID for A
	    pad->SiPM_A[paddle] = ch_num; 
	    sipm->PaddleEnd[ch_num] = 0;
	  }
	}
	status = fscanf(fp, "%s", line); // read in the closing "}" for RB
      }
    }
  }
  fclose(fp); // Finished with file
}

double FitSine(std::vector<double> volts, std::vector<double> times)
//if you want to get all three fit parameters:
//std::vector<double> FitSine(std::vector<double> volts, std::vector<double> times, float cm)
{
  //float ns_off = 0; //cm*0.08; //Harting cable signal propagation is supposed to be 5.13 ns/m or 0.0513 ns/cm. crude measurement gives  0.08 ns/cm
  int start_bin = 20;
  int size_bin = 900; //can probably make this smaller
  
  int data_size = 0;
  double pi = 3.14159265;
  double a;
  double b;
  //if you want to get all fit params
  //double c;
  double p[3]; // product of fitting equation
  double XiYi = 0.0;
  double XiZi = 0.0;
  double YiZi = 0.0;
  double XiXi = 0.0;
  double YiYi = 0.0;
  double Xi = 0.0;
  double Yi = 0.0;
  double Zi = 0.0;
  double xi = 0.0;
  double yi = 0.0;
  double zi = 0.0;

  for(int i=start_bin; i < start_bin+size_bin; i++)
  {

// condition left over from when the sine wave was truncated 
//    if (volts[i] > -80.0)
//    {

      xi = cos(2*pi*0.02*(times[i]));  //for this fit we know the frequency is 0.02 waves/ns
      yi = sin(2*pi*0.02*(times[i]));
      zi = volts[i];
      XiYi += xi*yi;
      XiZi += xi*zi;
      YiZi += yi*zi;
      XiXi += xi*xi;
      YiYi += yi*yi;
      Xi   += xi;
      Yi   += yi;
      Zi   += zi;
      data_size++;
//    }
  }

  double A[3][3];
  double B[3][3];
  double X[3][3];
  double x = 0;
  double n = 0; //n is the determinant of A

  //the matrix A is XTX where X is the matrix of dimensions (data_size x 3) <cos(2pifreq*time), sin(2pifreq*time),1>
  A[0][0] = XiXi;
  A[0][1] = XiYi;
  A[0][2] = Xi;
  A[1][0] = XiYi;
  A[1][1] = YiYi;
  A[1][2] = Yi;
  A[2][0] = Xi;
  A[2][1] = Yi;
  A[2][2] = data_size;

  n += A[0][0] * A[1][1] * A[2][2];
  n += A[0][1] * A[1][2] * A[2][0];
  n += A[0][2] * A[1][0] * A[2][1];
  n -= A[0][0] * A[1][2] * A[2][1];
  n -= A[0][1] * A[1][0] * A[2][2];
  n -= A[0][2] * A[1][1] * A[2][0];
  x = 1.0/n;

  //find cofactor matrix of A, call this B
  B[0][0] =  (A[1][1] * A[2][2]) - (A[2][1] * A[1][2]);
  B[0][1] = ((A[1][0] * A[2][2]) - (A[2][0] * A[1][2])) * (-1);
  B[0][2] =  (A[1][0] * A[2][1]) - (A[2][0] * A[1][1]);
  B[1][0] = ((A[0][1] * A[2][2]) - (A[2][1] * A[0][2])) * (-1);
  B[1][1] =  (A[0][0] * A[2][2]) - (A[2][0] * A[0][2]);
  B[1][2] = ((A[0][0] * A[2][1]) - (A[2][0] * A[0][1])) * (-1);
  B[2][0] =  (A[0][1] * A[1][2]) - (A[1][1] * A[0][2]);
  B[2][1] = ((A[0][0] * A[1][2]) - (A[1][0] * A[0][2])) * (-1);
  B[2][2] =  (A[0][0] * A[1][1]) - (A[1][0] * A[0][1]);

  //take the transpose of the cofactor matrix and divide by the determinant to get the inverse matrix X
  for(int i=0;i<3;i++)
  {
    for(int j=0;j<3;j++)
    {
      X[i][j] = B[j][i] * x;
    }
  }

  //multiply p = zTX by the result
  p[0] = XiZi;
  p[1] = YiZi;
  p[2] = Zi;
  a = X[0][0] * p[0] + X[1][0] * p[1] + X[2][0] * p[2];
  b = X[0][1] * p[0] + X[1][1] * p[1] + X[2][1] * p[2];
  //offset parameter
  //c = X[0][2] * p[0] + X[1][2] * p[1] + X[2][2] * p[2];
  
  double phi = atan2(a,b);
  
  return phi;

  //amplitude parameter
  //double amp2 = pow(a,2)+pow(b,2);
  
  //return all three params
  //std::vector<double> v;
  //v.push_back(phi);
  //v.push_back(amp2);
  //v.push_back(c);

  //return v;
}
