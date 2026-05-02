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

#include "constants.h"
#include "EventRene.h"
#include "PacketMethods.h"

namespace fs = std::filesystem;

double FitSine(std::vector<double> volts, std::vector<double> times);

////////////////////////////////////////////////////////////////////////////
// Default constructor
PacketMethods::PacketMethods(void) {
  // This should rarely be changed. Just putting it at the top so that
  // the initialization process is visible.                                  

  // First, we want to store information about the SiPM channels and
  // paddle relationships for analysis purpose. Read all that info             
  // into the relevant structures.
  InitPaddleInfo();
  GetPaddleInfo();
  
  InitializeVariables();
}

////////////////////////////////////////////////////////////////////////////
void PacketMethods::BeginRun(int run=5) {
  printf("Beginning Run %d.\n", run); fflush(stdout);

  // Just for utility, set the initial time from the timestamp48 value
  // in the first TofEventSummary Packet of the first flight .bin
  // file--RAW251215_101836.bin
  timeInit = 94575234425295;

  // Some useful variables (some initialized to default values)
  // but overwritten from file (if it exists)
  float Ped_low   = 10;
  float Ped_win   = 90;
  float CThresh   = 5.0;
  float CFDS_frac = 0.10;
  float Qwin_low  = 100;
  float Qwin_size = 100;
  float CHmin     = 4.0;

  // Instantiate our class that holds analysis results and set some           
  // initial values
  //EventGAPS Event = EventGAPS();
  Event.SetPaddleMap(&PadInfo, &SipmInfo);
  Event.SetThreshold(CThresh);
  Event.SetCFDFraction(CFDS_frac);
  Event.InitializeHistograms();
  Event.OffsetHistograms(false);
}

////////////////////////////////////////////////////////////////////////////
void PacketMethods::ProcessTofEvent(TofEvent *Tev,
				    std::map<u8,gondola::RBCalibration>& cali){

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

  usize pos = 0;
  // We need a structure to hold the waveforms for an event. We
  // initialize and delete them with each new event
  GAPS::Waveform *wave[NTOT];
  GAPS::Waveform *wch9[NRB];
  for (int i=0;i<NTOT;i++) wave[i] = NULL;
  for (int i=0;i<NRB;i++)  wch9[i] = NULL;
  float Phi[NRB];
  for (int i=0;i<NRB;i++) Phi[i] = -999.0;
  
  //unsigned long int evt_ctr = ev.mt_event.event_id;
  unsigned long int evt_ctr = Tev->event_id;

  //std::cout << &Tev << std::endl;

  //printf("Event %ld: RBs -", evt_ctr);
  //printf("%ld.", evt_ctr);
  
  for (auto const &rbid : Tev->get_rbids()) {
    RBEvent rb_event = Tev->get_rbevent(rbid);
    // Now that we know the RBID, we can set the starting ch_no
    // Eventually we will use a function to map RB_ch to GAPS_ch
    usize ch_start = (rbid-1)*NCH; // first RB is #1
    // Let's also store the channel mask to use later. 
    int ch_mask = rb_event.header.channel_mask;
    
    if (0) {
      std::cout << rb_event << std::endl;
    }
    
    Vec<Vec<f32>> volts;
    Vec<Vec<f32>> times;
    if (cali.contains(rbid)) {
      // Vec<f32> is a typedef for std::vector<float32>
      volts = cali[rbid].voltages(rb_event, true); //2nd arg -> spike cleaning
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
  
  // Now that we have the waveforms in place, analyze the event.
  Event.InitializeVariables(evt_ctr);
  Event.InitializeWaveforms(wave, wch9);
  
  // Calculate and store pedestals/RMSs for each channel
  Event.AnalyzePedestals(Ped_low, Ped_win);
  
  // Analyze the pulses in each channel
  Event.SetThreshold(CThresh);
  Event.SetCFDFraction(CFDS_frac);
  Event.AnalyzePulses(Qwin_low, Qwin_size);
  
  // Now that we have TDC values available, process the ch9 phases
  Event.AnalyzePhases(Phi);
  
  // Analyze each paddle: position on paddle, hitmask, etc
  Event.AnalyzePaddles(10.0, CHmin); //Args: Peak and Charge cuts
  
  // Now calculate beta, charge, and inner/outer tof x,y,z, etc.
  Event.AnalyzeEvent();
  
  // Now fill out histograms
  Event.FillChannelHistos(0);
  Event.FillPaddleHistos();
  Event.FillOffsetHistos();
  //}
  Event.UnsetWaveforms();
  for (int i=0;i<NTOT;i++) 
    if ( wave[i] != NULL ) { delete wave[i]; wave[i] = NULL; }
  for (int i=0;i<NRB;i++)  
    if ( wch9[i] != NULL ) { delete wch9[i]; wch9[i] = NULL; }
}

////////////////////////////////////////////////////////////////////////////
void PacketMethods::EndRun() {
  printf("Ending Run\n");
  Event.WriteHistograms();
  Event.WriteOffsetHistos();
}

////////////////////////////////////////////////////////////////////////////
void PacketMethods::InitializeVariables(void) {

}

////////////////////////////////////////////////////////////////////////////
void PacketMethods::InitPaddleInfo(void) {
  for (int i=0;i<NTOT;i++) { // First, init all values to zero.
    SipmInfo.PB[i]         = 0;
    SipmInfo.PB_ch[i]      = 0;
    SipmInfo.LTB[i]        = 0;
    SipmInfo.LTB_ch[i]     = 0;
    SipmInfo.RB[i]         = 0;
    SipmInfo.RB_ch[i]      = 0;
    SipmInfo.PaddleID[ch]  = 0;
    SipmInfo.PaddleEnd[ch] = 0;
  }

  for (int i=0;i<NPAD;i++) { // First, init all values to zero.
    PadInfo.VolumeID[i]       = 0;
    for (int j=0;j<3;j++) {
      PadInfo.Location[i][j]  = 0.0;
      PadInfo.Dimension[i][j] = 0.0;
    }
    PadInfo.Orientation[i]    = 0;
    PadInfo.CoaxLen[i]        = 0.0;
    PadInfo.HardingLen[i]     = 0.0;
    PadInfo.SiPM_A[i]         = 0;
    PadInfo.SiPM_B[i]         = 0;
    PadInfo.IsUmbrella[i]     = false;
    PadInfo.IsCube[i]         = false;
    PadInfo.IsCortina[i]      = false;
  }
}

////////////////////////////////////////////////////////////////////////////   
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
  double c;
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

  //amplitude parameter
  double amp2 = pow(a,2)+pow(b,2);

  //printf(" %.4f %.4f %.4f\n", sqrt(amp2), 0.020,  phi);
  return phi;

  
  //return all three params
  //std::vector<double> v;
  //v.push_back(phi);
  //v.push_back(amp2);
  //v.push_back(c);

  //return v;
}
