/*
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

#include "io.hpp"
#include "calibration.h"
#include "database.h"
#include "caraspace.hpp"

#include "constants.h"
#include "EventGAPS.h"
#include "./PacketMethods.h"
#include "./YourMethods.h"

namespace fs = std::filesystem;
namespace gt = Gaps::Telemetry;

// This shows how to add new subroutines/methods. Must prototype the
// method and then define it below.
void PrintNiceMessage(void);

void PrintNiceMessage(void) {
  std::cout << "Here is a nice message in a user-defined method." << std::endl;
}

////////////////////////////////////////////////////////////////////////////
// Default constructor
PacketMethods::PacketMethods(void) {
  // This should rarely be changed. Just putting it at the top so that
  // the initialization process is visible.

  // First, we want to store information about the SiPM channels and
  // paddle relationships for analysis purpose. Read all that info             
  // into the relevant structures.
  GetPaddleInfo();
  
  InitializeVariables();
}

////////////////////////////////////////////////////////////////////////////
void PacketMethods::BeginRun(int run=5) {
  printf("Beginning Run %d.\n", run); fflush(stdout);
  PrintNiceMessage();
}

////////////////////////////////////////////////////////////////////////////
void PacketMethods::ProcessTofEventSummary(TofEventSummary *Tes,
					   unsigned long int evt_no){
  //printf("Found TofEventSummary: evt = %ld\n", evt_no);
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

  // Print out the info in the packet.  
  if (0) 
    std::cout << *Tes << std::endl; 
  
  // Now, parse the TofEventSummary data into EvtInfo  
  for (TofHit const &h : Tes->hits) {
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
    
  } 
}

////////////////////////////////////////////////////////////////////////////
void PacketMethods::EndRun() {
  PrintNiceMessage();
  printf("Endning Run\n");
}

void PacketMethods::NothingYet(void) {
  
      
}

////////////////////////////////////////////////////////////////////////////
void PacketMethods::InitializeVariables(void) {

  

}

////////////////////////////////////////////////////////////////////////////
//void PacketMethods::GetPaddleInfo(struct PaddleInfo *pad, struct SiPMInfo *sipm) {
void PacketMethods::GetPaddleInfo(void) {
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
	PadInfo.Location[j][0]  = tmp_x;
	PadInfo.Location[j][1]  = tmp_y;
	PadInfo.Location[j][2]  = tmp_z;
	PadInfo.Orientation[j]  = tmp_o;
	PadInfo.Dimension[j][0] = tmp_dimx;
	PadInfo.Dimension[j][1] = tmp_dimy;
	PadInfo.Dimension[j][2] = tmp_dimz;
	PadInfo.VolumeID[j]     = tmp_vid;
      }
    }
  }
  fclose(fp); // Finished with file                                           
  
  float coax, harting;
  // One last task: Get cable timings                                         
  snprintf(fname, 500, "%s/%s/Jeff_paddle_cable.json", srcdir, codedir);
  fp = fopen(fname, "r");
  if ( fscanf(fp, "%s", label) != EOF ) { // Read in first "{"                
    while (fscanf(fp,"%*[^-0-9]%d  %*[^-0-9]%f  %*[^-0-9]%f ",
		  &tmp_pad, &coax, &harting) != EOF) {
      if (tmp_pad > 0) {
	PadInfo.CoaxLen[tmp_pad]     = coax;
	PadInfo.HardingLen[tmp_pad]  = harting;
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
	  SipmInfo.RB[ch_num] = rb_num;
	  SipmInfo.RB_ch[ch_num] = rb_ch;
	  SipmInfo.PaddleID[ch_num] = paddle;
	  if (pad_id > 2000) { // We have a paddle ID for B                   
	    PadInfo.SiPM_B[paddle] = ch_num;
	    SipmInfo.PaddleEnd[ch_num] = 1;
	  } else if (pad_id > 1000) { //We have a paddle ID for A             
	    PadInfo.SiPM_A[paddle] = ch_num;
	    SipmInfo.PaddleEnd[ch_num] = 0;
	  }
	}
	st = fscanf(fp, "%s", line); // read in the closing "}" for RB        
      }
    }
  }
  fclose(fp); // Finished with file                                           
}

////////////////////////////////////////////////////////////////////////////
// Default destructor
PacketMethods::~PacketMethods(void) {
}

