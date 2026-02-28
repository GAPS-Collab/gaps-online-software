/**
 * Binary to illustrate how to read GAPS L0 file with the
 * caraspace library.
 * To use this example, the code has to be build with
 * BUILD_CARASPACE=ON
 * 
 * March 2025, gaps-online-sw V0.10
 * The API will not be stable until V1.0 and is thus 
 * subject to change. Please refer to the respective 
 * README.md
 *
 */

#include <iostream>
#include <filesystem>
#include <chrono>
#include "cxxopts.hpp"

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "io.hpp"
#include "calibration.h"
#include "database.h"
#include "caraspace.hpp"

//#include "../../analysis/zweerink/include/constants.h"
#include "constants.h"
#include "EventGAPS.h"
void GetPaddleInfo(struct PaddleInfo *pad, struct SiPMInfo *sipm);

namespace fs = std::filesystem;
namespace gt = Gaps::Telemetry;

int main(int argc, char *argv[]){
  spdlog::cfg::load_env_levels();
    
  cxxopts::Options options("read-caraspace", "Read GAPS L0 (caraspace) files. These files contain ALL information, including the TOF disk (waveform) stream and ALL telemetry packets");
  options.add_options()
  ("h,help", "Print help")
  //("c,calibration", "Folder with binary calibration files for each RB", cxxopts::value<std::string>()->default_value(""))
  ("file", "A Caraspace file", cxxopts::value<std::string>())
  ("directory", "A directory containing .gaps (caraspace) files, e.g. L0 Gaps files", cxxopts::value<std::string>())
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
  auto pathname   = result["file"].as<std::string>();
  bool verbose = result["verbose"].as<bool>();
  
  fs::path path(pathname);
  if (!fs::exists(path)) {
    spdlog::error("Path {} does not exist!", pathname);
    exit(EXIT_FAILURE);
  }
  
  Vec<std::string> filenames;
  if (fs::is_directory(path)) {
    for (const auto& entry : fs::directory_iterator(path)) {
      if (entry.is_regular_file()) {
        std::string filename = entry.path().string();
        filenames.push_back(filename);
      }
    }
  } else {
    filenames.push_back(path.string());
  }

  std::cout << "Will read " << filenames.size() << " files!" << std::endl; 
  std::string tp_name        = "PacketType.TofEvent";
  std::string tel_ev_nogaps  = "TelemetryPacketType.NoGapsTriggerEvent";
  std::string tel_ev_boring  = "TelemetryPacketType.BoringEvent";
  std::string tel_ev_intrst  = "TelemetryPacketType.InterestingEvent";
  std::string tel_ev_notof   = "TelemetryPacketType.NoTofDataEvent";

  std::string cooling_name   = "TelemetryPacketType.CoolingHK";
  std::string rbwf_name      = "TelemetryPacketType.RBWaveform";

  // First, we want to store information about the SiPM channels and          
  // paddle relationships for analysis purpose. Read all that info            
  // into the relevant structures.                                            
  struct PaddleInfo PadInfo;
  struct SiPMInfo   SipmInfo;
  GetPaddleInfo(&PadInfo, &SipmInfo);
  
  // Instantiate our class that holds analysis results and set some           
  // initial values                                                           
  auto Event = EventGAPS();
  Event.SetPaddleMap(&PadInfo, &SipmInfo);
  //Event.SetThreshold(CThresh);                                              
  //Event.SetCFDFraction(CFDS_frac);                                          
  Event.InitializeHistograms();
  Event.OffsetHistograms(true);

  u64 n_frames_processed  = 0;
  u64 n_telemetry_errors  = 0;
  u64 n_tof_telemetry_err = 0;
        
  // counters for merged event packets                                    
  u64 n_boring            = 0;
  u64 n_nogaps            = 0;
  u64 n_interest          = 0;
  u64 n_notof             = 0;

  auto start = std::chrono::high_resolution_clock::now();

  //auto trk_mask = Gaps::get_trackerstripmasks();
  //auto trk_ped  = Gaps::get_trackerstrippedestals();

  // as an example, count tracker hits
  u64 n_trk_hits        = 0;
  u64 n_trk_hits_masked = 0;
  u64 n_evt_no_trk_hits = 0;
  auto event_ids        = Vec<u32>();
  for (auto const &f : filenames) {
    auto start = std::chrono::high_resolution_clock::now();
    auto reader = gondola::CRReader(f);
    u64 n_frames_processed_file = 0;
    while (!reader.is_exhausted()) {
      auto frame = gondola::CRFrame();
      try {
        frame = reader.get_next_frame();
      } catch (const std::exception& e) {
        std::string emeesage = std::format("--> Exception '{}' caught!", e.what());
        std::string message = std::format("--> File {} with {} frames processed! In total, we proceseed {} frames", f, n_frames_processed_file, n_frames_processed);
        std::cout << emeesage << std::endl;
        std::cout << message << std::endl;
        break;
      }
      ++n_frames_processed;
      ++n_frames_processed_file;

      gt::Packet pack;
      // check for RBWaveform                                       
      if (frame.index.contains(rbwf_name)) {
        auto pack = frame.get_telemetrypacket(rbwf_name);
        usize pos = 0;
        auto tp_res   = TofPacket::from_bytestream(pack.payload, pos);
        if (!tp_res.is_ok()) {
          spdlog::error("Can't get tofpacket for rbwaveform from telemetrypacket!");
          continue;
        }
        auto tp   = tp_res.unwrap();
        pos       = 0;
	auto rbwf = RBWaveform::from_bytestream(tp.payload, pos);
        std::cout << rbwf.to_string() << std::endl;
        // just for now                                       
        //exit(0);
      }

      if (frame.index.contains(tel_ev_nogaps)) {
        ++n_nogaps;
        pack = frame.get_telemetrypacket(tel_ev_nogaps);
        //printf("Found NoGaps\n");
      } else if (frame.index.contains(tel_ev_boring)) {
        ++n_boring;
        pack = frame.get_telemetrypacket(tel_ev_boring);
        //printf("Found Boring\n");
      } else if (frame.index.contains(tel_ev_intrst)) {
        ++n_interest;
        pack = frame.get_telemetrypacket(tel_ev_intrst);
        //printf("Found Interesting\n");
     } else if (frame.index.contains(tel_ev_notof)) {
        ++n_notof;
        pack = frame.get_telemetrypacket(tel_ev_notof);
        //printf("Found NoTOF\n");
      } else {
        continue;
      }

      //if (frame.index.contains(cooling_name)) {
      //  pack = frame.get_telemetrypacket(cooling_name);
      //  std::cout << pack.to_string() << std::endl;
      //  usize pos = 0;
      //  auto cooling = gt::Cooling::from_bytestream(pack.payload, pos);
      //  if (cooling.is_ok()) {
      //    std::cout << cooling.unwrap().to_string() << std::endl;
      //  } else {
      //    std::cout << cooling.unwrap_err().reason << std::endl;
      //  }
      //  //std::exit(1);
      //}

      if (verbose) {
        std::cout << "---- TELEMETRY -----" << std::endl;
        std::cout << frame.to_string() << std::endl;
        std::cout << pack.to_string() << std::endl;
      }
      usize pos = 0;
      auto result = gt::MergedEvent::from_bytestream(pack.payload, pos);
      // in case of errors, we just move on
      if (result.is_err()) {
        std::string message = result.unwrap_err().reason;
        spdlog::error(message);
        ++n_telemetry_errors;
        continue;
      }
      auto m_ev = result.unwrap();
      for (gt::TrkHit const &h : m_ev.trk_hits) {
        auto strip_id = Gaps::TrackerStrip::create_id(h.layer, h.row, h.module, h.channel);
        //if (trk_mask[strip_id]) {
        //  // only count active strips
        //  ++n_trk_hits;
        //  // just as an example - subtract a pedestal
        //  auto adc_no_pedestal = h.adc - trk_ped[strip_id].pedestal_mean;
        //  //adc_no_pedestal;
        //} else {
        //  ++n_trk_hits_masked;
	//}
          //std::cout << h.to_string() << std::endl;
      }
      //printf("%d %d %d: %d TRK - ", (int)m_ev.header.counter,
      //     (int)m_ev.header.ptype, m_ev.event_id, (int)m_ev.trk_hits.size());
      if (m_ev.trk_hits.size() == 0) {
        ++n_evt_no_trk_hits;
      } else {
        n_trk_hits += m_ev.trk_hits.size();
      }
      
      //printf("%d TOF\n", (int)m_ev.tof_event.hits.size());
      struct EventInfo EvtInfo;
      // First, initialize all the event values properly              
      for (int i=0;i<NPAD;i++) {
	for (int j=0;j<2;j++) {
	  EvtInfo.Ped[i][j]    = -999.0;
	  EvtInfo.PedRMS[i][j] = -999.0;
	  EvtInfo.VPeak[i][j]  = -999.0;
	  EvtInfo.Charge[i][j] = -999.0;
	  EvtInfo.TDC[i][j]    = -999.0;
	  EvtInfo.TOTLo[i][j]  = -999.0;
	  EvtInfo.TOTHi[i][j]  = -999.0;
	}
      }
      for (int i=0;i<NRB;i++) {
	EvtInfo.Phi[i] = -999.0;
      }

      // Now, parse the TofEventSummary data into EvtInfo  
      for (TofHit const &h : m_ev.tof_event.hits) {
	int pad = h.paddle_id;
	EvtInfo.Ped[pad][0]    = h.baseline_a;
	EvtInfo.Ped[pad][1]    = h.baseline_b;
	EvtInfo.PedRMS[pad][0] = h.baseline_a_rms;
	EvtInfo.PedRMS[pad][1] = h.baseline_b_rms;
	EvtInfo.VPeak[pad][0]  = h.get_peak_a();
	EvtInfo.VPeak[pad][1]  = h.get_peak_b();
	EvtInfo.Charge[pad][0] = h.get_charge_a();
	EvtInfo.Charge[pad][1] = h.get_charge_b();
	EvtInfo.TDC[pad][0]    = h.get_time_a();
	EvtInfo.TDC[pad][1]    = h.get_time_b();
	EvtInfo.TOTLo[pad][0]  = h.get_tot_low_a();
	EvtInfo.TOTLo[pad][1]  = h.get_tot_low_b();
	EvtInfo.TOTHi[pad][0]  = h.get_tot_high_a();
	EvtInfo.TOTHi[pad][1]  = h.get_tot_high_b();
	int sipm_a = PadInfo.SiPM_A[pad];
	int rb     = SipmInfo.RB[sipm_a];
	EvtInfo.Phi[rb]       = h.phase;
	
	// Start the event analysis: initialize our variables           
	Event.InitializeVariables(m_ev.event_id);
	
	// Now, fill the appropriate quantities in EventGAPS            
	Event.FillEventValues(&EvtInfo);

	// Now, process the ch9 phases                                  
	Event.AnalyzePhases(EvtInfo.Phi);
	
	// Analyze each paddle: position on paddle, hitmask, etc        
	Event.AnalyzePaddles(10.0, 5.0); //Args: Peak and Charge cuts   
	
	// Now calculate beta, charge, and inner/outer tof x,y,z, etc.  
	Event.AnalyzeEvent();
	
	// Now fill out histograms                                      
	Event.FillChannelHistos(0);
	Event.FillPaddleHistos();
	Event.FillOffsetHistos();
	// do someting with h
        //std::cout << h.to_string() << std::endl;
      }
      
      if (n_frames_processed % 1000 == 0) {
        auto end = std::chrono::high_resolution_clock::now();
        std::chrono::duration<double> elapsed = end - start;
        std::cout << "--> ------------------------------" << std::endl;
        std::cout << "--> Processesd " << n_frames_processed << " frames in " << elapsed << std::endl;
        std::cout << "--> Saw " << n_telemetry_errors << " errores when reading telemetry files!" << std::endl;
        //std::cout << "--> Saw " << n_tofpacket_errors << " errores when reading tofstream files!" << std::endl;
        auto start = std::chrono::high_resolution_clock::now();
      }
    }
  }
  Event.WriteHistograms();
  Event.WriteOffsetHistos();

  auto end = std::chrono::high_resolution_clock::now();
  auto elapsed = end - start;
  std::cout << "--> ----FINISHED--------------" << std::endl;
  std::cout << "--> Processesd " << n_frames_processed << " frames in " << elapsed << std::endl;
  std::cout << "--> Saw " << n_telemetry_errors << " errores when reading telemetry files!" << std::endl;
  std::cout << "--> Saw " << n_tof_telemetry_err << " errores when reading tofdata from telemetry files!" << std::endl;
  std::cout << "--> Saw " << n_trk_hits << " valid tracker hits!" << std::endl;
  std::cout << "--> Saw " << n_trk_hits_masked << " inactive tracker hits!" << std::endl;
  std::cout << "--> Saw " << n_evt_no_trk_hits << " events without any tracker hits!" << std::endl;
  //std::cout << "--> Saw " << n_tofpacket_errors << " errores when reading tofstream files!" << std::endl;
  spdlog::info("Finished");
  return EXIT_SUCCESS;
}

void GetPaddleInfo(struct PaddleInfo *pad, struct SiPMInfo *sipm) {
  // Eventually we will call the db to get all this info. For now, I          
  // will simple read the relevant files to get the info.                     
  
  FILE *fp;
  char label[50], line[500];
  char srcdir[200] = "/home/gaps/software/gaps_os_pro/";
  char codedir[200] = "gaps-db/resources/master-spreadsheet/";
  char fname[501];
  int st;
  float value;
  
  int tmp_pad, tmp_vid, vol_id[NPAD] = { 0 };
  int tmp_o;
  float tmp_x, tmp_y, tmp_z;
  float tmp_dimx, tmp_dimy, tmp_dimz;
  
  // For each paddle, read in the location, orientation, dims and volumeID    
  snprintf(fname, 500, "%s/%s/level0_coordinates.json", srcdir, codedir);
  fp = fopen(fname, "r");
  int ctr=0;
  if ( fscanf(fp, "%s", label) != EOF ) { // Read in first "{"                
    while (fscanf(fp,"%*[^-0-9]%d ", &tmp_pad) != EOF) { // Read paddle ID    
      // For each paddle, we want to set the X, Y, Z locations.               
      if (tmp_pad > 0 && tmp_pad < 161) { // Valid paddle ID                  
	int j = tmp_pad;
	st = fscanf(fp,"%*[^-0-9]%f %*[^-0-9]%f  %*[^-0-9]%f %*[^-0-9]%d ",
		    &tmp_x, &tmp_y, &tmp_z, &tmp_o);
	st = fscanf(fp,"%*[^-0-9]%f %*[^-0-9]%f  %*[^-0-9]%f %*[^-0-9]%d ",
		    &tmp_dimx, &tmp_dimy, &tmp_dimz, &tmp_vid);
	pad->Location[j][0]  = tmp_x;
	pad->Location[j][1]  = tmp_y;
	pad->Location[j][2]  = tmp_z;
	pad->Orientation[j]  = tmp_o;
	pad->Dimension[j][0] = tmp_dimx;
	pad->Dimension[j][1] = tmp_dimy;
	pad->Dimension[j][2] = tmp_dimz;
	pad->VolumeID[j]     = tmp_vid;
      }
    }
  }
  fclose(fp); // Finished with file                                           
  
  float coax, harting;
  // One last task: Get cable timings                                         
  snprintf(fname, 500, "%s/%s/paddle_cable.json", srcdir, codedir);
  fp = fopen(fname, "r");
  if ( fscanf(fp, "%s", label) != EOF ) { // Read in first "{"                
    while (fscanf(fp,"%*[^-0-9]%d  %*[^-0-9]%f  %*[^-0-9]%f ",
		  &tmp_pad, &coax, &harting) != EOF) {
      if (tmp_pad > 0) {
	pad->CoaxLen[tmp_pad]     = coax;
	pad->HardingLen[tmp_pad]  = harting;
	//printf("%3d %8.3f %8.3f\n", tmp_pad, coax, harting);                
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
	  st = fscanf(fp, "%*[^-0-9]%d  %*[^-0-9]%d ", &rb_ch, &pad_id);
	  // Store the SiPM Channel for each Paddle end                       
	  int paddle = pad_id % 1000;
	  int ch_num = (rb_num-1)*NCH + (rb_ch)-1; // Map the value to NTOT   
	  sipm->RB[ch_num] = rb_num;
	  sipm->RB_ch[ch_num] = rb_ch;
	  sipm->PaddleID[ch_num] = paddle;
	  if (pad_id > 2000) { // We have a paddle ID for B                   
	    pad->SiPM_B[paddle] = ch_num;
	    sipm->PaddleEnd[ch_num] = 1;
	  } else if (pad_id > 1000) { //We have a paddle ID for A             
	    pad->SiPM_A[paddle] = ch_num;
	    sipm->PaddleEnd[ch_num] = 0;
	  }
	}
	st = fscanf(fp, "%s", line); // read in the closing "}" for RB        
      }
    }
  }
  fclose(fp); // Finished with file                                           
}

