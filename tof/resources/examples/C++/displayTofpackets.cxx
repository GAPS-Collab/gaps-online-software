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

//#include "WaveGAPS.h"
#include <vector>

#include "./include/displayTofpackets.h"
#include <TSystem.h>
#include <TApplication.h>

#define MAX_EVENTS  1000
// A whole bunch of ROOT stuff for plotting. 
/* Global Variables */
int n_chan = 0;

const int NRB   = 50; // Technically, it is 49, but we don't use 0
const int NCH   = 8;
const int NTOT  = NCH * NRB; // NTOT is the number of SiPMs
const int NPADS = NTOT/2;        // NPAD: 1 per 2 SiPMs

// These are declared globally to make it easier to plot
//Vec<Waveform> wave;
//Vec<Waveform> wch9;
Waveplot *wave[NTOT];
Waveplot *wch9[NRB];

VoidFuncPtr_t initfuncs[] = { InitGui, 0};
int FirstEvent = 0;
int ALL_EVENTS = 1;
int plot_flag = ALLCH;  // Plot all channels by default
int plot_ch   = -1;
int restrict_range = 0;
float x_sc_lo, x_sc_hi, y_sc_lo, y_sc_hi;     // For full ranges
float x_scr_lo, x_scr_hi;
float y_scr_lo, y_scr_hi; // For restricted ranges
int Events[MAX_EVENTS];
TH2F    *h_dum2;
TROOT root("GUI","test",initfuncs);
TApplication theApp("FADC", 0, 0, 0, 0);    
MainFrame mainWin(gClient->GetRoot(), 600, 600);
TCanvas *cm = mainWin.GetCanvas();

int main(int argc, char *argv[]){
  spdlog::cfg::load_env_levels();
    
  cxxopts::Options options("unpack-tofpackets", "Unpack example for .tof.gaps files with TofPackets.");
  options.add_options()
  ("h,help", "Print help")
  ("c,calibration", "Calibration file (in txt format)", cxxopts::value<std::string>()->default_value("/home/gaps/nevis-data/tofdata/calibration/latest/"))
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
 
  read_events();  // List of event numbers to plot

  auto calname = result["calibration"].as<std::string>();
  RBCalibration cali[NRB]; // "cali" stores values for one RB

  // To read calibration data from individual binary files, when -c is
  // given with the directory of the calibration files. Since the
  // calibration files for each RB change with each calibration run,
  // this code reads the list of calibration files in the directory,
  // determines the RB number and copies the string into the relevant
  // array position. For RBs with no calibration file, the length of
  // the entry will be 0. We then use read the calibrations for all
  // RBs with files.
  bool RB_Calibrated[NRB] = { false };
  std::string cnames[NRB];
  if (calname != "") {
    char pname[500], line[500];
    snprintf(pname,450, "ls %s/RB*.cali.tof.gaps", calname.c_str());
    FILE *fp = popen(pname, "r");
    while (fscanf(fp,"%s", line) != EOF) {
      std::string c_name(line);          // Calib file found
      int position = c_name.find("RB");  // Find "RB" in the name
      std::string rbstr = c_name.substr(position+2, 2); // Extract RB num
      int rbnum = atoi(rbstr.data());    // Convert to integer
      //printf("%s %d %s\n", rbstr.c_str(), rbnum, line);
      cnames[rbnum] = c_name;            // Copy to proper place in array
    }
    pclose(fp);
    // Print out the calibration filenames as a sanity check
    //for (int i=0; i<NRB; i++) {
    //printf("%d: %lu %s\n", i, cnames[i].size(), cnames[i].c_str());
    //}
    
    for (int i=1; i<NRB; i++) {
      if (cnames[i].size() > 4) { // RB has a calibration file
	std::string f_str = cnames[i];
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
	      cali[i] = RBCalibration::from_bytestream(p.payload, pos); 
	    }
	  }
	}
      } //else {printf("File does not exist: %s\n", f_str.c_str());}
    }
  }

  // Some useful variables (some initialized to default values)
  // but overwritten from file (if it exists)
  float Ped_low   = 350;
  float Ped_win   = 100;
  float CThresh   = 5.0;
  float CFDS_frac = 0.10;
  float Qwin_low  = 100;
  float Qwin_size = 100;
  float CHmin     = 5.0;

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

  // Set the scale for the plots of the traces (this histogram is only  
  // used if the program is called with 'gui_flag' set to 1).
  x_sc_lo =    0.0;    // in ns
  x_sc_hi =  500.0;    // in ns
  y_sc_lo =  -20.0;    // in mV
  y_sc_hi =   40.0;    // in mV
  //y_sc_lo = -500.0;    // in mV
  //y_sc_hi = 1500.0;    // in mV
  //x_sc_lo =  -9999;    // in ns
  //x_sc_hi =  -9999;    // in ns

  float factor = 1.0;
  x_scr_lo =   50.0;    // in ns                                          
  x_scr_hi =  250.0;    // in ns                                          
  y_scr_lo = -270.0/factor;    // in mV                                   
  y_scr_hi =   35.0;    // in mV                                          

  int eventctr=0;
  
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
	//std::vector<GAPS::Waveform> wave;
	//wave.reserve(NTOT); // Number of SiPMs
	//wch9.reserve(NRB);  // Number of RBs
	usize ch_start;
	int nrbs=0;
	// Delete any waveforms 
	for (int i=0;i<NTOT;i++) wave[i] = NULL;
	for (int i=0;i<NRB;i++)  wch9[i] = NULL;
	//for(int c=0;c<NTOT;c++) 
	// if ( wave[c] != NULL ) { delete wave[c]; wave[c] = NULL; }
	//for(int c=0;c<NRB;c++) 
	//  if ( wch9[c] != NULL ) { delete wch9[c]; wch9[c] = NULL; }
	
	
        auto ev = TofEvent::from_bytestream(p.payload, pos);
	unsigned long int evt_ctr = ev.mt_event.event_id;
	printf("%ld.", evt_ctr); fflush(stdout);
	// Now, let's plot the data to see what it looks like
	int PLOT_EVENT = event_flag(evt_ctr);
	//ch_start = 0;
	if (PLOT_EVENT) {

	for (auto const &rbid : ev.get_rbids()) {
	  RBEvent rb_event = ev.get_rbevent(rbid);
	  if (verbose) {
	    std::cout << rb_event << std::endl;
          }
	  //printf(" %d (%d)\n", rbid, rb_event.header.channel_mask);
	  int ch_mask = rb_event.header.channel_mask;
	  // Now that we know the RBID, we can set the starting ch_no
	  // Eventually we will use a function to map RB_ch to GAPS_ch
	  ch_start = (rbid-1)*NCH; // first RB is #1
	  //printf("Event %ld: RB %d: start %ld\n", evt_ctr, rbid, ch_start);

	  Vec<Vec<f32>> volts;
	  Vec<Vec<f32>> times;
	  //if ((calname != "") && rbid < 44 ){
	  //if ((calname != "") && cali.rb_id == rbid ){
	  if (calname != "") { // For combined data all boards calibrated
	  /*if (calname != "" &&  // For combined data all boards calibrated
	      ( rbid==3  || rbid==14 || rbid==15 || rbid==32 || // Umb-cen
		rbid==16 || rbid==25 || rbid==44 || rbid==46 || // cube-top
		rbid==1  || rbid==11 || rbid==41 || rbid==42 ) ) {// cube-bot
	  */
	  /*if (calname != "" &&  // For combined data all boards calibrated
              ( rbid==17  || rbid==19 || rbid==13 || rbid==20 || 
                rbid==26) ) {// RBs with strange 
          */
	 // if (calname != "" &&  // For combined data all boards calibrated
         //     ( rbid==13 || rbid==20 || 
         //       rbid==26) ) {// RBs with strange 




	    nrbs++;
	    // Vec<f32> is a typedef for std::vector<float32>
	    //printf(" %d (%d)", rbid, rb_event.header.channel_mask);
	    volts = cali[rbid].voltages(rb_event, true); // second argument is for spike cleaning
	    // (C++ implementation causes a segfault sometimes when "true"
	    times = cali[rbid].nanoseconds(rb_event);
	    // volts and times are now ch 0-8 with the waveforms
	    // for this event.

	    // First, store the waveform for channel 9
	    Vec<f64> ch9_volts(volts[8].begin(), volts[8].end());
	    Vec<f64> ch9_times(times[8].begin(), times[8].end());
	    wch9[rbid] = new Waveplot(ch9_volts.data(),ch9_times.data(),rbid,0);
	    wch9[rbid]->SetPedBegin(10);
	    wch9[rbid]->SetPedRange(100);
	    wch9[rbid]->CalcPedestalRange(); 
	    float ch9RMS = wch9[rbid]->GetPedsigma();
	    //printf(" %d(%.1f)", rbid, ch9RMS);
	    
	    // Now, deal with all the SiPM data
	    for(int c=0;c<NCH;c++) {
	      //bool e = (ch_mask & (1 << c) > 0 ? true : false);
	      unsigned int inEvent = ch_mask & (1 << c);
	      if (inEvent > 0 ) {
		Vec<f64> ch_volts(volts[c].begin(), volts[c].end());
		Vec<f64> ch_times(times[c].begin(), times[c].end());
		
		usize cw = c+ch_start; 
		wave[cw] = new Waveplot(ch_volts.data(),ch_times.data(), cw,0);
		wave[cw]->SetThreshold(CThresh);
		wave[cw]->SetCFDSFraction(CFDS_frac);
		
		// Calculate the pedestal
		wave[cw]->SetPedBegin(Ped_low);
		wave[cw]->SetPedRange(Ped_win);
		wave[cw]->CalcPedestalRange(); 
		wave[cw]->SubtractPedestal(); 
		if (wave[cw]->GetPedsigma() < 60.6) {
		  wave[cw]->FindPeaks(Qwin_low, Qwin_size);
		  wave[cw]->FindTdc(0, CFD_SIMPLE);
		  //wave[cw]->FindTdc(0, CONSTANT);
		  //printf("%ld: %ld - %7.3f\n",evt_ctr,cw,wave[cw]->GetTdcs(0));
		}
		//if (c==0) printf("%ld(%.2f) ", cw, wave[cw]->GetPedestal());
	      } // Only for channels in ch_mask for this RB
	    }
	  }
	}
	
	n_chan = NTOT;
	// Now, let's plot the data to see what it looks like
	//int PLOT_EVENT = event_flag(evt_ctr);
	//ch_start = 0;
	//if (PLOT_EVENT) {
	  //printf(".%ld", evt_ctr); fflush(stdout);
	  //plotrb(n_chan, ch_start);
	  plotall(n_chan, nrbs);
	  
	  if (plot_flag != FINISH) {
	    while (gui_wait() != NEXT) {
	      switch (mainWin.status) {
	      case QUIT : {
		return (0);
		break;
	      }
	      case SELECT : {
		int chan=atoi(mainWin.GetText());
		if (chan < 0 || chan >= n_chan) {
		  std::cout << " Ch must be between 0 and " <<
		    n_chan-1 << std::endl;
		} else {
		  plot_flag = SELECT;
		  plot_ch = chan;
		  plotall(n_chan, nrbs);
		}
		break;
	      }
	      case ALLCH : {
		plot_flag = ALLCH;
		plotall(n_chan, nrbs);
		break;
	      }
	      case PRINT : {
		cm->Print("histo.pdf");
		break;
	      }
	      case FINISH : {
		plot_flag = FINISH;
		break;
	      }
	      case RESTRICT : {
		restrict_range = (restrict_range == 1 ? 0 : 1);
		break;
	      }
	      case FIT : {
		for (int c = 0; c < n_chan; c++) {
		  if (plot_flag == ALLCH ||
		      (plot_flag == SELECT && plot_ch == c)) {
		    cm->cd(c+1);
		    //wave[c]->FindPeaks(L.pulse_lo[c],L.pulse_win[c]);
		    //if (wave[c]->GetNumPeaks() > 0)
		    // wave[c]->PlotFit();
		  }
		}
		cm->Update();
		break;
	      }
	      }
	    }		  
	  } // Move on to the next RB
	} 
	for (int i=0;i<NTOT;i++) //{delete wave[i]; wave[i] = NULL;}
	  if ( wave[i] != NULL ) { delete wave[i]; wave[i] = NULL; }
	for (int i=0;i<NRB;i++)  //{delete wch9[i]; wch9[i] = NULL;}
	  if ( wch9[i] != NULL ) { delete wch9[i]; wch9[i] = NULL; }
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

int gui_wait() {

  mainWin.status = WAITING;
  while(mainWin.status == WAITING) {
    gSystem->Sleep(100);
    gSystem->ProcessEvents();
  }

  return(mainWin.status);
}

    
void read_events() {
  int i;
  for (i = 0; i < MAX_EVENTS; i++)
    Events[i] = -1; // Initialize the values                                   

  // Now let's read the events to be displayed                                 
  FILE *fp = fopen("event_list.txt","r");
  if (fp == NULL)
    return;

  ALL_EVENTS = 0;
  i=0;
  // Read events
  while ( (fscanf(fp,"%d",&Events[i++]) != EOF) && (i<MAX_EVENTS) );

  fclose (fp);
  return ;
}

int event_flag( int evno ) {
  if (ALL_EVENTS==1) return 1;

  for (int i=0;i<MAX_EVENTS;i++)
    if (evno == Events[i]) return 1;
  return 0;
}

void plotrb(int n_ch, int ch_start) {
  /*  int ch;
  int rbid = ch_start/NCH + 1;
  
  if (plot_flag == SELECT) {
    cm->Clear();
    cm->cd(0);
    ch = plot_ch + ch_start;
    //wave[ch]->SetPeakPlot(1); // Put some additional info on the plot
    if (restrict_range)
      wave[ch]->PlotWaveform(0, x_scr_lo, x_scr_hi, y_scr_lo, y_scr_hi);
    else
      if ( ch%9 == 8)
        wave[ch]->PlotWaveform(0, x_sc_lo, x_sc_hi, -400, +400);
      else
        wave[ch]->PlotWaveform(0, x_sc_lo, x_sc_hi, y_sc_lo, y_sc_hi);
  }
  
  if (plot_flag == ALLCH) {
    cm->Clear();
    int y = (int) sqrt(n_ch);
    if (sqrt(n_ch) > (float) y)
      y++;
    int x = ceil((float) n_ch / (float) y);
    x=9;
    y=(int)(n_ch/x);
    //cm->Divide(x, y, 1.0e-5, 1.0e-5);
    //cm->Divide(3, 3, 1.0e-5, 1.0e-5);
    cm->Divide(9, 16, 1.0e-5, 1.0e-5);
    for (int i = 0; i < n_ch; i++) {
      ch = ch_start+i;
      cm->cd( ch%8 + 1);
      //wave[ch]->SetPeakPlot(1); // Put some additional info on the plot  
      if (restrict_range)
        wave[ch]->PlotWaveform(0, x_scr_lo, x_scr_hi, y_scr_lo, y_scr_hi);
      else
        wave[ch]->PlotWaveform(0, x_sc_lo, x_sc_hi, y_sc_lo, y_sc_hi);
    }
    cm->cd(9);
        wch9[rbid]->PlotWaveform(0, x_sc_lo, x_sc_hi, -400, +400);
  }
  cm->Update(); */
}

void plotall(int n_ch, int nrbs) {
  int ch;
  
  if (plot_flag == SELECT) {
    cm->Clear();
    cm->cd(0);
    ch = plot_ch;
    if (wave[ch] != NULL) {
      //wave[ch]->SetPeakPlot(1); // Put some additional info on the plot
      if (restrict_range)
	wave[ch]->PlotWaveform(0, x_scr_lo, x_scr_hi, y_scr_lo, y_scr_hi);
      else
	wave[ch]->PlotWaveform(0, x_sc_lo, x_sc_hi, y_sc_lo, y_sc_hi);
    }
  }
  
  if (plot_flag == ALLCH) {
    cm->Clear();
    int y = (int) sqrt(n_ch);
    if (sqrt(n_ch) > (float) y)
      y++;
    int x = ceil((float) n_ch / (float) y);
    x=9;
    y=(int)(n_ch/x);
    //cm->Divide(x, y, 1.0e-5, 1.0e-5);
    //cm->Divide(3, 3, 1.0e-5, 1.0e-5);
    cm->Divide(9, nrbs, 1.0e-5, 1.0e-5);
    int ctr = 0, pl_num, ch_num=0;
    int rbid;
    for (int i = 0; i < n_ch; i++) {
	pl_num = ctr*(NCH+1)+1 + ch_num++;
	cm->cd( pl_num ); 
      if (wave[i] != NULL) {
	//wave[ch]->SetPeakPlot(1); // Put some additional info on the plot  
	//printf("Plotting (%d %d).", i, pl_num);fflush(stdout);
	if (restrict_range)
	  wave[i]->PlotWaveform(0, x_scr_lo, x_scr_hi, y_scr_lo, y_scr_hi);
	else
	  wave[i]->PlotWaveform(0, x_sc_lo, x_sc_hi, y_sc_lo, y_sc_hi);
	//printf("Done\n");fflush(stdout);
      }
      if (i%8 == 7) {  // finished with 8 SiPM channels on RB
	rbid = i/NCH + 1;
	cm->cd(ctr*(NCH+1)+9);
	if (wch9[rbid] != NULL) {
	  wch9[rbid]->PlotWaveform(0, x_sc_lo, x_sc_hi, -400, +400);
	  ctr++; // increment the number of RBs plotted
	  //printf("ch %d; rbid %d; ctr %d\n", i, rbid, ctr); fflush(stdout);
	}
	ch_num = 0;
      }
    }
  }
  cm->Update();
}
