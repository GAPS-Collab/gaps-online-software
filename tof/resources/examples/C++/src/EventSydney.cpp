#include <stdlib.h>
#include <stdio.h>
#include <TFile.h>
#include <TTree.h>
#include <TF1.h>
#include <TGraph.h>
#include <TMath.h>
#include <fstream>

/* Waveform stuff. */
#include "../include/EventSydney.h"

// Some useful macros
#define SQR(A)               ( (A) * (A) )
#define ABS(A)               ( ( (A<0) ? -(A) : (A) ) )
#define PI                   3.14159265
// In units of mm/ns
#define CSPEED               299.792458


using namespace std;

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
// Default constructor
EventGAPS::EventGAPS(void) {

  // Initialize any values necessary for a new event
  InitializeVariables(0);

}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
// Default destructor
EventGAPS::~EventGAPS(void) {

}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::InitializeVariables(unsigned long int evt_ctr=0) {
  
  evtno    = evt_ctr;
  sc_speed = 154.0;   // in mm/s
  
  // Reset everything that is stored by SiPM channel number
  for (int i=0; i<NTOT; i++) {
    Pedestal[i] = -999.0;
    PedRMS[i]   = -999.0;
    VPeak[i]    = -999.0;
    QInt[i]     = -999.0;
    TDC[i]      = -999.0;
  }
  
  // Reset everything that is stored by RB number
  for (int i=0; i<NRB; i++) {
    ClockPedestal[i] = -999.0;
    ClockPedRMS[i]   = -999.0;
    RBInData[i]      = false;
    Phi[i]           = -999.0;
  }
  
  // Reset everything that is stored by Paddle number (1-160)
  for (int i=0; i<NPAD; i++) {
    Hits[i]   = -999;
    HitX[i]   = -999.0;
    HitY[i]   = -999.0;
    HitZ[i]   = -999.0;
    HitT[i]   = -999.0;
    delta[i]  = -999.0;
    IsHit[i]  = false;
  }
  
  // Reset everything that is stored by event
  beta         = -1.0;
  EarlyPaddle  = -1;
  NPadCube     = 0;
  NPadUmbrella = 0;
  NPadCortina  = 0;
  
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
// Set up our Waveforms
void EventGAPS::InitializeWaveforms(GAPS::Waveform *wave[],
				    GAPS::Waveform *wch9[]) {
  // Store pointers to the waveforms locally
  for (int i=0; i<NTOT; i++) wData[i]  = wave[i];
  for (int i=0; i<NRB;  i++) wClock[i] = wch9[i];
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
// Remove our Waveforms
void EventGAPS::UnsetWaveforms(void) {
  // Store pointers to the waveforms locally
  for (int i=0; i<NTOT; i++) {
    //delete wData[i];
    wData[i]  = NULL;
  }
  for (int i=0; i<NRB;  i++) {
    //delete wData[i];
    wClock[i] = NULL;
  }
  
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
// Set up our SiPM channel to Paddle map to Location in detector
void EventGAPS::SetPaddleMap(int paddle_map[NRB][NCH], int pad2volid[NPAD],
			     int padvid[NPAD], float padLocation[NPAD][4]) {
  // This was an older, more kludgey way of doing this where a bunch
  // of arrays were passed into the subroutine. The next method is
  // much cleaner where arguments are two structures with all the
  // mapping already done.
  
  // This subroutine stores the SiPM channel for each paddle end (A,B)
  for (int i=0; i<NRB; i++) {
    for (int j=0; j<NCH; j++) {
      // Store the paddle for each RB/CH
      //ChnlMap[i][j] = paddle_map[i][j];
      
      // Store the SiPM Channel for each Paddle end
      int paddle = paddle_map[i][j] % 1000;
      int ch_num = (i-1)*NCH+j; // Map the value to NTOT
      if (paddle_map[i][j] > 2000) { // We have a paddle ID for B
	Paddle_B[paddle] = ch_num; 
	//printf("B -> %d %d %d %d %d\n", i,j,ch_num, paddle,paddle_map[i][j]);
      } else if (paddle_map[i][j] > 1000) { //We have a paddle ID for A
	Paddle_A[paddle] = ch_num;
      }
    }
  }
  
  /*
  for (int i=0; i<NPAD; i++) {
    printf("PadID %3d  -> RB_A %3d %2d %2d; RB_B %3d %2d %2d\n", i,
	   Paddle_A[i], (int)Paddle_A[i]/NCH, Paddle_A[i]%NCH, 
	   Paddle_B[i], (int)Paddle_B[i]/NCH, Paddle_B[i]%NCH); 
  }
  */
  
  // For each paddle, we want to set the X, Y, Z locations. So, index
  // through the pad2volid[] array, find matching volid in
  // padLocation[][] array, and set x,y,z from appropriate values
  for (int i=0; i<NPAD; i++) { 
    int tmp_id = pad2volid[i];
    if (tmp_id > 0) { // Valid Volume ID
      for (int j=0; j<NPAD; j++) {
	if (tmp_id == padvid[j]) { // Found a match
	  PadVID[i] = tmp_id;
	  PadX[i]   = padLocation[j][0];
	  PadY[i]   = padLocation[j][1];
	  PadZ[i]   = padLocation[j][2];
	  PadO[i]   = (int)(padLocation[j][3]);
	  //printf("  Pad %d: %d  %.2f %.2f %.2f %.d\n", i, PadVID[i],
	  //	 PadX[i], PadY[i], PadZ[i], PadO[i]);
	}
      }
    }
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
// Set up our SiPM channel to Paddle map to Location in detector
void EventGAPS::SetPaddleMap(struct PaddleInfo *pad, struct SiPMInfo *sipm) {
  // This subroutine stores the SiPM channel for each paddle end (A,B)
  for (int i=1; i<NRB; i++) { // There is no RB0
    for (int j=0; j<NCH; j++) {
      int ch=(i-1)*NCH + j;  // Determine NTOT position
      RB[ch]     = sipm->RB[ch];
      RB_ch[ch]  = sipm->RB_ch[ch];
      Paddle[ch] = sipm->PaddleID[ch];
      PadEnd[ch] = sipm->PaddleEnd[ch];
      // If we have a valid RB, set the maximum sipm ch
      if (RB[ch] > 0 && RB[ch] < NRB) max_sipm = ch;
    }
  }
  if (0)
    for (int i=0;i<NTOT;i++)
      printf("%3d: %2d  %d  %3d  %d\n",i,RB[i],RB_ch[i],Paddle[i],PadEnd[i]);
  
  
  for (int i=0; i<NPAD; i++) {
    // Store the SiPM Channel for each Paddle end
    Paddle_A[i] = pad->SiPM_A[i]; 
    Paddle_B[i] = pad->SiPM_B[i]; 
    
    
    PadVID[i] = pad->VolumeID[i];
    // Store maximum paddle ID (real numbering, not C-numbering)
    if (PadVID[i] >= 100000000 && PadVID[i] < 120000000) max_paddle = i+1;
    PadO[i]   = pad->Orientation[i];
    PadX[i]   = pad->Location[i][0];
    PadY[i]   = pad->Location[i][1];
    PadZ[i]   = pad->Location[i][2];
    for (int j=0;j<1;j++) Dimension[i][j] = pad->Dimension[i][j];
    // Fixed timing correction for each paddle requires adding length
    // of the MTB-RB Harding cable, subtracting the SiPM coax length
    // and subtracting propagation time in the scintillator.
    TCorrFixed[i] = pad->HardingLen[i] - pad->CoaxLen[i];
    // Will correct for paddle dimension when calculating hit time
    
    if (0) {
      printf("Pad %3d: %d  %2d (%8.2f %8.2f %8.2f) %.1f (%.2f %.2f) %.2f\n",
	     i, PadVID[i], PadO[i], PadX[i], PadY[i], PadZ[i], Dimension[i][0],
	     pad->HardingLen[i], pad->CoaxLen[i], TCorrFixed[i]);
    }
  }
  
  /*  
  for (int i=0; i<NPAD; i++) {
    printf("PadID %3d  -> RB_A %3d %2d %2d; RB_B %3d %2d %2d\n", i,
	   Paddle_A[i], (int)Paddle_A[i]/NCH, Paddle_A[i]%NCH, 
	   Paddle_B[i], (int)Paddle_B[i]/NCH, Paddle_B[i]%NCH); 
  }
  */
}

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::InitializeHistograms(void) {

  char text[400];
  
  // Histograms for pedestals and pedestal RMSs
  for (int b = 0; b < NTOT; b++) {
      sprintf(text, "pedHist[%d]", b);
      pedHist[b] = new TH1D(text, "", 400, -10, 10);
      pedHist[b]->GetXaxis()->SetTitle("Pedestal (mV)");
      pedHist[b]->GetYaxis()->SetTitle("Counts");

      sprintf(text, "pedRMSHist[%d]", b);
      pedRMSHist[b] = new TH1D(text, "", 500, -1, 4);
      pedRMSHist[b]->GetXaxis()->SetTitle("Pedestal RMS (mV)");
      pedRMSHist[b]->GetYaxis()->SetTitle("Counts");
  }

  float lo_ch = -5.0;  // Low range of the charge plots (pC)
  float hi_ch =  60.0; // Hi range of the charge plots (pC)
  int   PeakBins = 100;
  float PeakLo   = -200;
  float PeakHi   = 40.0;
  
  //Histograms containing the charge distribution
  for (int b = 0; b < NTOT; b++) {
    //rao  change this histogram name and parameters
    sprintf(text, "Vpeak[%d]", b);
    //  Peak[b] = new TH1D(text, "", PeakBins, PeakLo, PeakHi);
    Peak[b] = new TH1D(text, "", 160, -10.0, 150.0);
    Peak[b]->GetXaxis()->SetTitle("Vpeak (mV)");
    Peak[b]->GetYaxis()->SetTitle("Counts");
  }
  
  //Histograms containing the charge distribution
  for (int b = 0; b < NTOT; b++) {
    //rao  change the parameters of this histogram
    sprintf(text, "Charge[%d]", b);
    // Charge[b] = new TH1D(text, "", 200, lo_ch, hi_ch);
    Charge[b] = new TH1D(text, "", 130, -10.0, 60.0);
    // Charge[b]->GetXaxis()->SetTitle("Charge(pC) (voltage*time/imp)");
    Charge[b]->GetXaxis()->SetTitle("Charge(pC)");
    Charge[b]->GetYaxis()->SetTitle("Counts");
  }

  for (int b = 0; b < NTOT; b++) {
    // rao  change the parameters of this histogram
    sprintf(text, "Charge Cut [%d]", b);
    // Charge_cut[b] = new TH1D(text, "", 200, lo_ch, hi_ch);
    Charge_cut[b] = new TH1D(text, "", 130, -10.0, 60.0);
    Charge_cut[b]->GetXaxis()->SetTitle("Charge(cut,pC)");
    Charge_cut[b]->GetYaxis()->SetTitle("Counts");
  }

  for (int b = 0; b < NTOT; b++) {
    sprintf(text, "tdcCFD[%d]", b);
    tdcCFD[b] = new TH1D(text, "", 400, 10.0, 200.0);
    tdcCFD[b]->GetXaxis()->SetTitle("Pulse Time (ns)");
    tdcCFD[b]->GetYaxis()->SetTitle("Counts");
  }
  
  //rao  TDC diffs
  for (int b = 0; b < NPAD; b++) {
    sprintf(text, "tDiff[%d]", b);
    tDiff[b] = new TH1D(text, "", 400, -5, 10);
    tDiff[b]->GetXaxis()->SetTitle("TDC Difference");
    tDiff[b]->GetYaxis()->SetTitle("Counts");
  }

  //Ch9 time shift
  for (int b = 0; b < NPAD; b++) {
    sprintf(text, "Ch9Shift[%d]", b);
    Ch9Shift[b] = new TH1D(text, "", 400, -5, 5);
    Ch9Shift[b]->GetXaxis()->SetTitle("Ch9 Shift");
    Ch9Shift[b]->GetYaxis()->SetTitle("Counts");
  }

  // Paddle Hit times
  for (int b = 0; b < NPAD; b++) {
    sprintf(text, "HitTime[%d]", b);
    HitTime[b] = new TH1F(text, "", 280, -10, 60);
    HitTime[b]->GetXaxis()->SetTitle("Hit Time (ns)");
    HitTime[b]->GetYaxis()->SetTitle("Counts");
  }

  // Earliest Paddle hit
  FirstPaddle = new TH1I("First Paddle Hit", "", 160, 0.5, 160.5);
  FirstPaddle->GetXaxis()->SetTitle("First Paddle Hit");
  FirstPaddle->GetYaxis()->SetTitle("Counts");
  // Earliest Paddle hit time
  FirstTime = new TH1F("First Hit Time", "", 260, 100.5, 360.5);
  FirstTime->GetXaxis()->SetTitle("First Hit Time");
  FirstTime->GetYaxis()->SetTitle("Counts");

  // Distribution of Beta
  BetaDist = new TH1F("Beta Distribution", "", 130, -0.05, 1.25);
  BetaDist->GetXaxis()->SetTitle("Beta Value");
  BetaDist->GetYaxis()->SetTitle("Counts");

  // Histograms comparing the charge measured at both ends of the paddle.
  for (int b = 0; b < NPAD; b++) {
    sprintf(text, "QEnd2Tdiff[%d]", b);
    QEnd2End[b] = new TH2D(text, "", 300, lo_ch, hi_ch,
                              300, lo_ch, hi_ch);
    QEnd2End[b]->GetXaxis()->SetTitle("End A");
    //QEnd2End[b]->GetYaxis()->SetTitle("End B");
    QEnd2End[b]->GetYaxis()->SetTitle("tdiff");
  }

  //rao  hit mask histograms
  for (int b = 0; b < NPAD; b++) {
    sprintf(text, "HitMask[%d]", b);
    HitMask[b] = new TH1I(text, "", 10, -2.5, 7.5);
    HitMask[b]->GetXaxis()->SetTitle("Hit Mask (A=1,B=2)");
    HitMask[b]->GetYaxis()->SetTitle("Counts");
  }

  float p_len;
  // Hit position along paddle
  for (int b = 0; b < NPAD; b++) {
    sprintf(text, "HitPosition[%d]", b);
    p_len = Dimension[b][0]/20.0; // Dimension in mm, position in cm
    //HitPosition[b] = new TH1F(text, "", 190, -95.0, 95.0);
    HitPosition[b] = new TH1F(text, "", 190, -1.2*p_len, 1.2*p_len);
    HitPosition[b]->GetXaxis()->SetTitle("Position (cm)");
    HitPosition[b]->GetYaxis()->SetTitle("Counts");
  }
  
  // Hit position in GAPS volume
  HitGAPS = new TH3F("HitGAPS", "", 180, -1800.0, 1800.0,
		     180, -1800.0, 1800.0,
		     110, 0.0, 2200.0 );
  HitGAPS->GetXaxis()->SetTitle("X Position (cm)");
  HitGAPS->GetYaxis()->SetTitle("Y Position (cm)");
  
  // Hit position in GAPS volume
  HitCube = new TH3F("HitCube", "", 180, -1800.0, 1800.0,
		     180, -1800.0, 1800.0,
		     110, 0.0, 2200.0 );
  HitCube->GetXaxis()->SetTitle("X Position (cm)");
  HitCube->GetYaxis()->SetTitle("Y Position (cm)");
  
  // Hit position in GAPS volume
  HitCortina = new TH3F("HitCortina", "", 180, -1800.0, 1800.0,
			180, -1800.0, 1800.0,
			110, 0.0, 2200.0 );
  HitCortina->GetXaxis()->SetTitle("X Position (cm)");
  HitCortina->GetYaxis()->SetTitle("Y Position (cm)");
  
  // Hit position in GAPS volume
  HitUmbrella = new TH3F("HitUmbrella", "", 180, -1800.0, 1800.0,
			 180, -1800.0, 1800.0,
			 110, 0.0, 2200.0 );
  HitUmbrella->GetXaxis()->SetTitle("X Position (cm)");
  HitUmbrella->GetYaxis()->SetTitle("Y Position (cm)");
  
  // Average Charge vs position along paddle
  for (int b = 0; b < NPAD; b++) {
    sprintf(text, "QvPosition[%d]", b);
    p_len = Dimension[b][0]/20.0; // Dimension in mm, position in cm
    //QvPosition[b] = new TProfile(text, "", 190, -95.0, 95.0);
    QvPosition[b] = new TProfile(text, "", 190, -1.2*p_len, 1.2*p_len);
    QvPosition[b]->GetXaxis()->SetTitle("Position (cm)");
    QvPosition[b]->GetYaxis()->SetTitle("Avg Charge");
    QvPosition[b]->SetMinimum(0);
    QvPosition[b]->SetMaximum(70);
    //QvPosition[b]->SetStats(false);
  }
  
  // Charge vs position along paddle (End A)
  for (int b = 0; b < NPAD; b++) {
    sprintf(text, "QvPositionA[%d]", b);
    p_len = Dimension[b][0]/20.0; // Dimension in mm, position in cm
    //QvPositionA[b] = new TProfile(text, "", 190, -95.0, 95.0);
    QvPositionA[b] = new TProfile(text, "", 190, -1.2*p_len, 1.2*p_len);
    QvPositionA[b]->GetXaxis()->SetTitle("Position (cm)");
    QvPositionA[b]->GetYaxis()->SetTitle("Charge - End A");
    QvPositionA[b]->SetMinimum(0);
    QvPositionA[b]->SetMaximum(70);
  }
  // Charge vs position along paddle (End B)
  for (int b = 0; b < NPAD; b++) {
    sprintf(text, "QvPositionB[%d]", b);
    p_len = Dimension[b][0]/20.0; // Dimension in mm, position in cm
    //QvPositionB[b] = new TProfile(text, "", 190, -95.0, 95.0);
    QvPositionB[b] = new TProfile(text, "", 190, -1.2*p_len, 1.2*p_len);
    QvPositionB[b]->GetXaxis()->SetTitle("Position (cm)");
    QvPositionB[b]->GetYaxis()->SetTitle("Charge - End B");
    QvPositionB[b]->SetMinimum(0);
    QvPositionB[b]->SetMaximum(70);
  }
  
  //rao  number of paddles hit Cube, Umbrella, Cortina
  NPaddlesCube = new TH1I("NPaddles Hit Cube", "", 152, -1.5, 150.5);
  NPaddlesCube->GetXaxis()->SetTitle("NPaddes Hit Cube");
  NPaddlesCube->GetYaxis()->SetTitle("Counts");
  
  NPaddlesUmbrella = new TH1I("NPaddles Hit Umbrella", "", 12, -1.5, 10.5);
  NPaddlesUmbrella->GetXaxis()->SetTitle("NPaddes Hit Umbrella");
  NPaddlesUmbrella->GetYaxis()->SetTitle("Counts");
  
  NPaddlesCortina = new TH1I("NPaddles Hit Cortina", "", 12, -1.5, 10.5);
  NPaddlesCortina->GetXaxis()->SetTitle("NPaddes Hit Cortina");
  NPaddlesCortina->GetYaxis()->SetTitle("Counts");
  
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::WriteHistograms() {
  
  TFile *outfile = TFile::Open("/home/gaps/sydney/outfile.root","RECREATE"); 
  
  // For reasons I don't understand, the code to make subdirectories
  // is not compiling properly and gives an error (below) when run
  
  //./analyzeNevis: symbol lookup error: ./analyzeNevis: undefined
  //symbol: _ZN10TDirectory30GetSharedLocalCurrentDirectoryEv
  
  // For now, I am simply writing all the plots to the main directory.
  
  //create directories for the raw plots
  //TDirectory *savdir = gDirectory; 
  //outfile->cd();
  //TDirectory *Peddir = outfile->mkdir("Pedestals");
  //TDirectory *Peakdir = outfile->mkdir("VPeakplots");
  //TDirectory *Chargedir = outfile->mkdir("Chargeplots");
  //TDirectory *Hitmaskdir = outfile->mkdir("Hitmasks");
  
  //TDirectory *TDCdir = outfile->mkdir("TDCplots");

  // Plots made using simple CFD timing
  //TDirectory *CFDTimingdir = savdir->mkdir("CFDTimingplots");
  //TDirectory *CFDTimeVsQdir = savdir->mkdir("CFDTimeVsQplots");
  //TDirectory *CFDTVsTdir = savdir->mkdir("CFDTvsTplots");

  int start = 1; // No Sipm ch = 0 or paddle = 0
  max_sipm = max_paddle*2-1;
  //write all the Trace plots to the root file
  //Peddir->cd();
  for (int i = start; i < max_sipm; i++) {
    pedHist[i]->Write();
    pedRMSHist[i]->Write();
  }
  
  //Peakdir->cd();
  for (int i = start; i < max_sipm; i++) Peak[i]->Write();
  
  //Chargedir->cd();
  for (int i = start; i < max_sipm; i++) {
    Charge[i]->Write();
    Charge_cut[i]->Write();
  }
  for (int j = start; j < max_paddle; j++) QEnd2End[j]->Write();
  HitGAPS->Write();
  HitCube->Write();
  HitCortina->Write();
  HitUmbrella->Write();
  for (int j = start; j < max_paddle; j++) HitPosition[j]->Write();
  for (int j = start; j < max_paddle; j++) {
    QvPosition[j]->Write();
    QvPositionA[j]->Write();
    QvPositionB[j]->Write();
  }
  
  //TDCdir->cd();
  FirstPaddle->Write();
  FirstTime->Write();
  BetaDist->Write();
  for (int j = start; j < max_paddle; j++) HitTime[j]->Write();
  for (int j = start; j < max_paddle; j++) Ch9Shift[j]->Write();
  for (int j = start; j < max_paddle; j++) tDiff[j]->Write();
  for (int i = start; i < max_sipm; i++) tdcCFD[i]->Write();
  
  //Hitmaskdir->cd();
  NPaddlesCube->Write();
  NPaddlesUmbrella->Write();
  NPaddlesCortina->Write();
  for (int j = start; j < max_paddle; j++) HitMask[j]->Write();
  
  outfile->Close();
  
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::AnalyzePedestals(float Ped_low, float Ped_win) {

  for (int i=0; i<NTOT; i++) {
    if (wData[i] != NULL) { 
      wData[i]->SetPedBegin(Ped_low);
      wData[i]->SetPedRange(Ped_win);
      wData[i]->CalcPedestalRange();    // Calculate pedestals
      wData[i]->SubtractPedestal();     // Subtract pedestals
      // Now store the values
      Pedestal[i] = wData[i]->GetPedestal(); 
      PedRMS[i]   = wData[i]->GetPedsigma();


      //if ( PedRMS[i] > 15 ) printf("Channel %d: %8.1f\n", i, PedRMS[i]);
      if ( PedRMS[i] > 3 ) {
	if ( i%NCH==7 && PedRMS[i-1]>3 && PedRMS[i-2]>3 && PedRMS[i-3]>3 ) {
	  //printf("Data Mangled Event %ld: RB %d\n", evtno, i/NCH);
	}
      }
    }
  }
  // This does the same thing for the clock data if so desired
  /*for (int i=0; i<NRB; i++) {
    if (wClock[i] != NULL) { 
      wClock[i]->SetPedBegin(Ped_begin);
      wClock[i]->SetPedRange(Ped_win);
      wClock[i]->CalcPedestalRange();    // Calculate pedestals
      wClock[i]->PedestalSubtract();     // Subtract pedestals
      // Now store the values
      ClockPedestal[i] = wClock[i]->GetPedestal(); 
      ClockPedRMS[i] = wClock[i]->GetPedsigma();

      if ( ClockRMS[i] > 190 ) printf("RB %d: %8.1f\n", i, ClockPedRMS[i]);
    }
  }*/
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::AnalyzePulses(float Pulse_low, float Pulse_win) {

  for (int i=0; i<NTOT; i++) {
    if (wData[i] != NULL) { 
      //if (wData[i] != NULL && PedRMS[i] < 3.0 ) { 
      // Verify that quantities are set correctly
      wData[i]->SetThreshold(Threshold);
      wData[i]->SetCFDSFraction(CFDFraction);
      // Find the pulse height
      VPeak[i] = wData[i]->GetPeakValue(Pulse_low, Pulse_win);
      // Find the charge around the peak
      double pk_time = wData[i]->GetPeakTime();
      float begin, size;
      if (pk_time < 25.0) {
	begin = 5.0; // Never use first 10 bins (which is t<5ns)
	size  = 80.0 + pk_time -5.0;
      } else {
	// Normal operation, integrate peak-20 to peak+80
	begin = pk_time - 20.0;
	size  = 100;
      }
      QInt[i]  = wData[i]->Integrate(begin, size);
      //if (size <100) printf("Evt %ld, %d: %.1f %.1f\n",evtno,i,begin,size);
      //QInt[i]  = wData[i]->Integrate(Pulse_low, Pulse_win);
      // If we have a pulse above threshold, find the TDC value
      wData[i]->FindPeaks(Pulse_low, Pulse_win);
      //if ( (wData[i]->GetNumPeaks() > 0) && (Qint[i] > 5.0) ) {
      if ( (wData[i]->GetNumPeaks() > 0) ) {
	wData[i]->FindTdc(0, GAPS::CFD_SIMPLE);     // Simple CFD
	TDC[i] = wData[i]->GetTdcs(0);
      }
    }
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::AnalyzePhases(float phi[NRB]) {

  float ref_phi = -999.0;
  int   ref_rb;
  
  // Set the local value of Phi and designate which RBs have data
  for (int i=0; i<NRB; i++) {
    if (phi[i] > -998.0) { // Have a calculated phi
      Phi[i] = phi[i];
      RBInData[i] = true;
      if (ref_phi < -998.0) { // For now, first legit value is reference
	ref_phi = Phi[i];
	ref_rb = i;
      }
    }
  }

  for (int i=0; i<NRB; i++) {
    // For each RB, subtract phi from reference value...
    if (RBInData[i]) {
      //float phi_shift = ref_phi - Phi[i];
      float phi_shift = Phi[i] - ref_phi;
      // Ensure the shift is in proper range: -Pi/3 < shift < Pi/3
      while (phi_shift < -PI/3.0) phi_shift += 2.0*PI;
      while (phi_shift >  PI/3.0) phi_shift -= 2.0*PI;
      // Store the timing shift for the ch9 correction
      TShift[i] = phi_shift/(2.0*PI*0.02);
    } else TShift[i] = -999;
  }

  if (0) { 
    for (int i=0; i<NRB; i++) {
      printf(" %.3f", TShift[i]);
      if (i%10 == 9) printf("\n");
    }
    printf("\n");
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::AnalyzePaddles(float pk_cut = -999, float ch_cut = -999.0) {
  // Assuming previous calls to AnalyzePedestals and AnalyzePulses,
  // you have access to Pedestals, charges, Peaks, TDCs etc so
  // calculate quantities related to paddles.

  float Vpeak_cut  = 10.0;
  float Charge_cut =  5.0;
  int ahit, bhit;
  
  int cube=0, upper=0, outer=0;
  
  // If passed non-negative values, use the arguments instead of defaults
  if (pk_cut > 0) Vpeak_cut  = pk_cut;
  if (ch_cut > 0) Charge_cut = ch_cut;
  
  for (int i=0; i<NPAD; i++) {
    int chA = Paddle_A[i]; // SiPM channel of A end
    int chB = Paddle_B[i]; // SiPM channel of B end
    ahit = (VPeak[chA] > Vpeak_cut) ? 1 : 0 ;   
    bhit = (VPeak[chB] > Vpeak_cut) ? 2 : 0 ;   
    Hits[i] = ahit + bhit;
    IsHit[i] = false; // Flag that we don't have Hit info
    
    if (Hits[i] == 3) { // We have hits on both ends of paddle
      float tdc_diff = TDC[chA] - TDC[chB];
      float delta_pos = tdc_diff*sc_speed/2.0;
      // Now, use position and orientation of paddle to find hit location
      int sign = (PadO[i] > 0 ? 1 : -1); 
      int orient = ABS(PadO[i]);
      delta[i] = delta_pos/10.0; // Convert mm to cm
      HitX[i] = PadX[i];
      HitY[i] = PadY[i];
      HitZ[i] = PadZ[i];
      if (orient == 1) HitX[i] += sign*delta_pos; 
      if (orient == 2) HitY[i] += sign*delta_pos;
      if (orient == 3) HitZ[i] += sign*delta_pos;
      // Find the ch9 timing shift for this paddle
      int rbnum = RB[chA];
      TCorrEvent[i] = TShift[rbnum];
      // Correct TDC from each end of the paddle. 
      TDC_Cor[chA] = TDC[chA] + TCorrEvent[i] + TCorrFixed[i];
      
TDC_Cor[chB] = TDC[chB] + TCorrEvent[i] + TCorrFixed[i];
      // Calculate hit time for the paddle
      HitT[i] = (TDC_Cor[chA]+TDC_Cor[chB])/2.0 - Dimension[i][0]/(2.0*sc_speed);
      if ( TDC[chA]>5 && TDC[chA]<220 && TDC[chB]>5 && TDC[chB]<220 ) {
	IsHit[i] = true;
      }
    }
    
    if (IsHit[i]>0) {
      if (i<61) cube++;         // Paddle in Cube
      else if (i<109) upper++;  // Paddle in Umbrella
      else if (i<161) outer++;  // Paddle in Cortina
    }
  }
  
  NPadCube     = cube;
  NPadUmbrella = upper;
  NPadCortina  = outer;
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::AnalyzeEvent(void) {
  // Assuming previous calls to AnalyzePedestals and AnalyzePulses,
  // you have access to Pedestals, charges, Peaks, TDCs etc so
  // calculate any interesting quantities here.

  float early = 1000;
  int   e_pad = -1;
  
  // Find the earliest hit time (and paddle) and demand that it is
  // either in the umbrella or cortina
  //for (int i=61; i<NPAD; i++) {
  for (int i=61; i<73; i++) {
    if (IsHit[i] ) {
      if (0) {
	printf(" %3d %7.3f %7.3f %7.2f -%7.2f (%7.2f) %8.2f %6.2f\n", i,
	       TCorrEvent[i], TCorrFixed[i], TDC[Paddle_A[i]], TDC[Paddle_B[i]],
	       TDC[Paddle_A[i]]+TDC[Paddle_B[i]], HitT[i], sc_speed);
      }
      if (HitT[i] < early) {
	early = HitT[i];
	e_pad = i;
      }
    }
  }
  EarlyTime = early;
  EarlyPaddle = e_pad;
  //printf("Earliest: %ld  %d (%.2f)\n", evtno, EarlyPaddle, early);
  
  // Now, subtract the earliest time from all other times
  if (e_pad > 0) { // Only if we found a umb/cor paddle hit
    for (int i=0; i<NPAD; i++)
      if (IsHit[i]) HitT[i] -= early;
  }

  // Now that we have the hit times and positions, calculate beta
  //for (int i=0; i<61; i++) { // Only calculate beta for cube hits
  for (int i=0; i<13; i++) { // Only calculate beta for cube-top hits
  //for (int i=5; i<9; i++) { // Only calculate beta for middle cube-top hits
    if (IsHit[i]) {
      float dist_sq = SQR(HitX[i]-HitX[e_pad]) + SQR(HitY[i]-HitY[e_pad]) +
	SQR(HitZ[i]-HitZ[e_pad]);
      float t_diff = HitT[i] - HitT[e_pad];
      float speed = sqrt(dist_sq) / (t_diff); // mm/ns
      beta = speed/(CSPEED);
      if (0 && e_pad>60 && e_pad<73) {
      //if (0 && (e_pad>64 && e_pad<69) ) {
	printf("Positions: (%d %d) (%.2f %.2f)  (%.2f %.2f)  (%.2f %.2f)\n",
	       i, e_pad, HitX[i], HitX[e_pad], HitY[i], HitY[e_pad],
	       HitZ[i], HitZ[e_pad]);
	printf("Speeds: (%.2f %.2f): %.2f   %.2f  %.2f\n", HitT[i],
	HitT[e_pad], t_diff,  speed, beta); 
	printf(" %3d %7.3f %7.3f %7.2f -%7.2f (%7.2f) (%7.2f %7.2f) %3d %6.2f\n",
	       i, TCorrEvent[i],TCorrFixed[i], TDC[Paddle_A[i]],TDC[Paddle_B[i]],
	       TDC[Paddle_A[i]]+TDC[Paddle_B[i]], HitT[i], HitT[e_pad], e_pad,
	       sc_speed);
      }
    }
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
// PmtThreshold should be set to the value used by vmedaq.
void EventGAPS::SetThreshold(float PmtThreshold){
  if (PmtThreshold > 0){
    Threshold = PmtThreshold ;
  } else {
    printf("PMT Threshold is %.2f.  It must be a POSITIVE number!!!",
	   PmtThreshold);
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
// CFDS_fraction for determining TDC value
void EventGAPS::SetCFDFraction(float CFDS_frac){
  if (CFDS_frac > 0){
    CFDFraction = CFDS_frac;
  } else {
    printf("CFD Fraction is %.2f.  It must be a POSITIVE number!!!",
	   CFDS_frac);
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::FillChannelHistos(int old=0) {
  // This section of code stores histos with channel numbers based on
  // RBs. Histo channel = SiPM Channel = (RB-1)*NCH+rbch  
  


	
  if (old) {
    for (int i=0; i<NTOT; i++) {
      pedHist[i]->Fill(Pedestal[i]);
      pedRMSHist[i]->Fill(PedRMS[i]);
      Peak[i]->Fill(VPeak[i]);
      Charge[i]->Fill(QInt[i]);
      if (QInt[i]>5.0) Charge_cut[i]->Fill(QInt[i]);
      
      tdcCFD[i]->Fill(TDC[i]);
    }
  } else {
    // This is the default way to store the histograms in the root file
    // This section of code stores histos with channel numbers based on
    // paddles. For paddle N, Histo[N/N+1] = PaddleA/B SiPM
    for (int i=0; i<NPAD; i++) {
      if (Paddle_A[i] > 0) { 
	int ch = 2*i;
	pedHist[ch-1]->Fill(Pedestal[Paddle_A[i]]);
	pedHist[ch]->Fill(Pedestal[Paddle_B[i]]);
	pedRMSHist[ch-1]->Fill(PedRMS[Paddle_A[i]]);
	pedRMSHist[ch]->Fill(PedRMS[Paddle_B[i]]);
	
	Peak[ch-1]->Fill(VPeak[Paddle_A[i]]);
	Peak[ch]->Fill(VPeak[Paddle_B[i]]);
	
	Charge[ch-1]->Fill(QInt[Paddle_A[i]]);
	Charge[ch]->Fill(QInt[Paddle_B[i]]);
	if (QInt[Paddle_A[i]]>5.0) Charge_cut[ch-1]->Fill(QInt[Paddle_A[i]]);
	if (QInt[Paddle_B[i]]>5.0) Charge_cut[ch]->Fill(QInt[Paddle_B[i]]);
	
	tdcCFD[ch-1]->Fill(TDC[Paddle_A[i]]);
	tdcCFD[ch]->Fill(TDC[Paddle_B[i]]);
      }
    }
  }




//calculate tdc times for all umbrella 12PP paddles to one particular cube top paddle: start with paddle 6
//

	//int cubePadd = 6;

  //Manually pick out events where conditions are met and calculate tdc times
        //let's isolate paddles in the umbrella top and cube top: paddle 66 and paddle 6
        //66 (+1X location) : RB-ch 15-01, 15-02. 295cm Coax. 20ft Harting. cw = (15-1)*8 + 0,1 = 112, 113 (A,B). 180cm paddle
        //6  (+1X location) : RB-ch 16-01, 16-02. 375cm Coax. 10ft Harting. cw = (16-1)*8 + 0,1 = 120, 121 (A,B). 180cm paddle

	//67 (-1X location) : RB-ch 14-01, 14-02. 295cm Coax. 20ft Harting. cw = (14-1)*8 + 0,1 = 104, 105 (A,B). 180cm paddle
        //7  (-1X location) : RB-ch 46-07, 46-08. 375cm Coax. 10ft Harting. cw = (46-1)*8 + 6,7 = 366, 367 (A,B). 180cm paddle
/*	
        int umbrA = 112;
        int umbrB = 113;
        int cubeA = 120;
        int cubeB = 121;

        int u66ped = 0;
        int u66t = 0;
	int u67ped = 0;
        int u67t = 0;

	int c6ped = 0;
        int c6t = 0;
        int c7ped = 0;
        int c7t = 0;

	int umbrA2 = 104;
        int umbrB2 = 105;
        int cubeA2 = 366;
        int cubeB2 = 367;

	int paddcount = 0;

        for (int i=0; i<NTOT; i++) {
	  if (i<160){
	  if (IsHit[i]) paddcount++;
	  }
	  NPaddlesCube->Fill(paddcount);
	  
          if ( i == umbrA || i == umbrB ){
                pedHist[i]->Fill(Pedestal[i]);
                pedRMSHist[i]->Fill(PedRMS[i]);
                if (PedRMS[i] < 2.0) u66ped++;
                Peak[i]->Fill(VPeak[i]);
                Charge[i]->Fill(QInt[i]);
                if (QInt[i]>5.0) Charge_cut[i]->Fill(QInt[i]);

                tdcCFD[i]->Fill(TDC[i]);
                if (TDC[i] < 220 && TDC[i] > 5) u66t++;

          }

	    if ( i == cubeA || i == cubeB ){
		pedHist[i]->Fill(Pedestal[i]);
                pedRMSHist[i]->Fill(PedRMS[i]);
                if (PedRMS[i] < 2.0) c6ped++;
                Peak[i]->Fill(VPeak[i]);
                Charge[i]->Fill(QInt[i]);
                if (QInt[i]>5.0) Charge_cut[i]->Fill(QInt[i]);

                tdcCFD[i]->Fill(TDC[i]);
                if (TDC[i] < 220 && TDC[i] > 5) c6t++;
	  }

	  if ( i == umbrA2 || i == umbrB2 ){
                pedHist[i]->Fill(Pedestal[i]);
                pedRMSHist[i]->Fill(PedRMS[i]);
                if (PedRMS[i] < 2.0) u67ped++;
                Peak[i]->Fill(VPeak[i]);
                Charge[i]->Fill(QInt[i]);
                if (QInt[i]>5.0) Charge_cut[i]->Fill(QInt[i]);

                tdcCFD[i]->Fill(TDC[i]);
                if (TDC[i] < 220 && TDC[i] > 5) u67t++;
	  }

	  if ( i == cubeA2 || i == cubeB2 ){
		pedHist[i]->Fill(Pedestal[i]);
                pedRMSHist[i]->Fill(PedRMS[i]);
                if (PedRMS[i] < 2.0) c7ped++;
                Peak[i]->Fill(VPeak[i]);
                Charge[i]->Fill(QInt[i]);
                if (QInt[i]>5.0) Charge_cut[i]->Fill(QInt[i]);

                tdcCFD[i]->Fill(TDC[i]);
                if (TDC[i] < 220 && TDC[i] > 5) c7t++;
          }
        }

	NPaddlesCube->Fill(paddcount);

       //   for (int j=0; j<6; j++){
       //     QEnd2End[40+j*10]->Fill(QInt[Paddle_A[18+j]],QInt[Paddle_B[18+j]]);
       //     QEnd2End[41+j*10]->Fill(QInt[Paddle_A[18+j]+1],QInt[Paddle_B[18+j]+1]);
       //     QEnd2End[42+j*10]->Fill(QInt[Paddle_A[18+j]-1],QInt[Paddle_B[18+j]-1]);
            //QEnd2End[43+j*10]->Fill(QInt[Paddle_A[18+j]+1],QInt[Paddle_B[18+j]]);
            //QEnd2End[44+j*10]->Fill(QInt[Paddle_A[18+j]-1],QInt[Paddle_B[18+j]]);
       //   }

	//umbrella 66
	
	if (paddcount < 3){
        if (u66t == 2){
        float tuA = TDC[umbrA];
        float tuB = TDC[umbrB];
	float tu = (tuA+tuB)/2 + TShift[15] + 6.096*4.46 - 2.95*4.15 - (180/(2*15.4));
	HitTime[66]->Fill(tu);

	if (c6t == 2){
        float tcA = TDC[cubeA];
        float tcB = TDC[cubeB];

        float tc = (tcA+tcB)/2 + TShift[16] + 3.048*4.46 - 3.75*4.15 - (180/(2*15.4));

        float phidiff = Phi[16] - Phi[15];
        float rawtdiff = (tcA+tcB)/2 - (tuA+tuB)/2;
        float adjtdiff = tc-tu;

            if (u66ped+c6ped == 4){
              HitTime[6]->Fill(tc);
              tDiff[1]->Fill(rawtdiff);
              Ch9Shift[160]->Fill(phidiff);
              tDiff[2]->Fill(adjtdiff);
	      if (tuA-tuB > -2.0 && tuA-tuB < 2.0 && tcA-tcB > -2.0 && tcA-tcB < 2.0){
	        tDiff[3]->Fill(rawtdiff);
	        tDiff[4]->Fill(adjtdiff);
	        if (adjtdiff > 0 && adjtdiff < 8) tDiff[5]->Fill(adjtdiff);
		QEnd2End[1]->Fill(QInt[umbrA], adjtdiff);
        	QEnd2End[2]->Fill(QInt[umbrB], adjtdiff);
        	QEnd2End[3]->Fill(QInt[umbrA]+QInt[umbrB], adjtdiff);
        	QEnd2End[4]->Fill(QInt[cubeA], adjtdiff);
        	QEnd2End[5]->Fill(QInt[cubeB], adjtdiff);
        	QEnd2End[6]->Fill(QInt[cubeA]+QInt[cubeB], adjtdiff);
		std::ofstream myfile;
        	myfile.open ("/home/gaps/sydney/evt_list_goodped.csv", std::ios::app);
        	myfile << evtno;
        	myfile << "," << "66" << "," << "6" << "," << tuA << "," << tuB << "," << tu << "," << TShift[15] << "," <<  tcA << "," << tcB << "," << tc << "," << TShift[16] << "," << adjtdiff <<  std::endl;
        	myfile.close();

	      }
	    }
	}

	if (c7t == 2){
        float tcA = TDC[cubeA2];
        float tcB = TDC[cubeB2];

        float tc = (tcA+tcB)/2 + TShift[46] + 3.048*4.46 - 3.75*4.15 - (180/(2*15.4));

        float phidiff = Phi[46] - Phi[15];
        float rawtdiff = (tcA+tcB)/2 - (tuA+tuB)/2;
        float adjtdiff = tc-tu;

            if (u66ped+c7ped == 4){

              HitTime[7]->Fill(tc);
              tDiff[6]->Fill(rawtdiff);
              Ch9Shift[159]->Fill(phidiff);
              tDiff[7]->Fill(adjtdiff);
              if (tuA-tuB > -2.0 && tuA-tuB < 2.0 && tcA-tcB > -2.0 && tcA-tcB < 2.0){
                tDiff[8]->Fill(rawtdiff);
                tDiff[9]->Fill(adjtdiff);
                if (adjtdiff > 0 && adjtdiff < 8) tDiff[10]->Fill(adjtdiff);
                QEnd2End[11]->Fill(QInt[umbrA], adjtdiff);
                QEnd2End[12]->Fill(QInt[umbrB], adjtdiff);
                QEnd2End[13]->Fill(QInt[umbrA]+QInt[umbrB], adjtdiff);
                QEnd2End[14]->Fill(QInt[cubeA2], adjtdiff);
                QEnd2End[15]->Fill(QInt[cubeB2], adjtdiff);
                QEnd2End[16]->Fill(QInt[cubeA2]+QInt[cubeB2], adjtdiff);
                std::ofstream myfile;
                myfile.open ("/home/gaps/sydney/evt_list_goodped.csv", std::ios::app);
                myfile << evtno;
                myfile << "," << "66" << "," << "7" << "," << tuA << "," << tuB << "," << tu << "," << TShift[15] << "," <<  tcA << "," << tcB << "," << tc << "," << TShift[46] << "," << adjtdiff << std::endl;
                myfile.close();

              }
            }
        }
	 */   /*
	    else{
	      HitTime[76]->Fill(tu);
              HitTime[16]->Fill(tc);
              tDiff[21]->Fill(rawtdiff);
              Ch9Shift[150]->Fill(phidiff);
              tDiff[22]->Fill(adjtdiff);
              if (tuA-tuB > -1.0 && tuA-tuB < 1.0 && tcA-tcB > -1.0 && tcA-tcB < 1.0){
                tDiff[23]->Fill(rawtdiff);
                tDiff[24]->Fill(adjtdiff);
                if (adjtdiff > 0 && adjtdiff < 8) tDiff[25]->Fill(adjtdiff);
		QEnd2End[21]->Fill(QInt[umbrA], adjtdiff);
        	QEnd2End[22]->Fill(QInt[umbrB], adjtdiff);
        	QEnd2End[23]->Fill(QInt[umbrA]+QInt[umbrB], adjtdiff);
        	QEnd2End[24]->Fill(QInt[cubeA], adjtdiff);
        	QEnd2End[25]->Fill(QInt[cubeB], adjtdiff);
        	QEnd2End[26]->Fill(QInt[cubeA]+QInt[cubeB], adjtdiff);
		std::ofstream myfile;
                myfile.open ("/home/gaps/sydney/evt_list_badped.csv", std::ios::app);
                myfile << evtno;
                myfile << "," << adjtdiff << "," << "6" << std::endl;
                myfile.close();
              }
	    }
	    */
	  }
/*	else{
          if (IsHit[6] && IsHit[66]){
            std::ofstream myfile;
            myfile.open ("/home/gaps/sydney/hit_but_badtime.csv", std::ios::app);
            myfile << evtno;
            myfile << "," << "6" << "," << TDC[umbrA] << "," << TDC[umbrB] << "," << TDC[cubeA] << "," << TDC[cubeB] << std::endl;
            myfile.close();
          }
        }
*/
/*	if (u67t == 2){
        float tuA = TDC[umbrA2];
        float tuB = TDC[umbrB2];
        float tu = (tuA+tuB)/2 + TShift[14] + 6.096*4.46 - 2.95*4.15 - (180/(2*15.4));
        HitTime[67]->Fill(tu);

        if (c6t == 2){
        float tcA = TDC[cubeA];
        float tcB = TDC[cubeB];

        float tc = (tcA+tcB)/2 + TShift[16] + 3.048*4.46 - 3.75*4.15 - (180/(2*15.4));

        float phidiff = Phi[16] - Phi[14];
        float rawtdiff = (tcA+tcB)/2 - (tuA+tuB)/2;
        float adjtdiff = tc-tu;

            if (u67ped+c6ped == 4){
              HitTime[6]->Fill(tc);
              tDiff[11]->Fill(rawtdiff);
              Ch9Shift[158]->Fill(phidiff);
              tDiff[12]->Fill(adjtdiff);
              if (tuA-tuB > -2.0 && tuA-tuB < 2.0 && tcA-tcB > -2.0 && tcA-tcB < 2.0){
                tDiff[13]->Fill(rawtdiff);
                tDiff[14]->Fill(adjtdiff);
		QEnd2End[21]->Fill(QInt[umbrA2], adjtdiff);
                QEnd2End[22]->Fill(QInt[umbrB2], adjtdiff);
                QEnd2End[23]->Fill(QInt[umbrA2]+QInt[umbrB2], adjtdiff);
                QEnd2End[24]->Fill(QInt[cubeA], adjtdiff);
                QEnd2End[25]->Fill(QInt[cubeB], adjtdiff);
                QEnd2End[26]->Fill(QInt[cubeA]+QInt[cubeB], adjtdiff);
                if (adjtdiff > 0 && adjtdiff < 8) tDiff[15]->Fill(adjtdiff);
		std::ofstream myfile;
                myfile.open ("/home/gaps/sydney/evt_list_goodped.csv", std::ios::app);
                myfile << evtno;
                myfile << "," << "67" << "," << "6" << "," << tuA << "," << tuB << "," << tu << "," << TShift[14] << "," <<  tcA << "," << tcB << "," << tc << "," << TShift[16] << "," << adjtdiff <<  std::endl;
                myfile.close();

              }
            }
        }

        if (c7t == 2){
        float tcA = TDC[cubeA2];
        float tcB = TDC[cubeB2];

        float tc = (tcA+tcB)/2 + TShift[46] + 3.048*4.46 - 3.75*4.15 - (180/(2*15.4));

        float phidiff = Phi[46] - Phi[14];
        float rawtdiff = (tcA+tcB)/2 - (tuA+tuB)/2;
        float adjtdiff = tc-tu;

	if (u66ped+c7ped == 4){

              HitTime[7]->Fill(tc);
              tDiff[16]->Fill(rawtdiff);
              Ch9Shift[157]->Fill(phidiff);
              tDiff[17]->Fill(adjtdiff);
              if (tuA-tuB > -2.0 && tuA-tuB < 2.0 && tcA-tcB > -2.0 && tcA-tcB < 2.0){
                tDiff[18]->Fill(rawtdiff);
                tDiff[19]->Fill(adjtdiff);
                if (adjtdiff > 0 && adjtdiff < 8) tDiff[20]->Fill(adjtdiff);
                QEnd2End[31]->Fill(QInt[umbrA2], adjtdiff);
                QEnd2End[32]->Fill(QInt[umbrB2], adjtdiff);
                QEnd2End[33]->Fill(QInt[umbrA2]+QInt[umbrB2], adjtdiff);
                QEnd2End[34]->Fill(QInt[cubeA2], adjtdiff);
                QEnd2End[35]->Fill(QInt[cubeB2], adjtdiff);
                QEnd2End[36]->Fill(QInt[cubeA2]+QInt[cubeB2], adjtdiff);
                std::ofstream myfile;
                myfile.open ("/home/gaps/sydney/evt_list_goodped.csv", std::ios::app);
                myfile << evtno;
                myfile << "," << "67" << "," << "7" << "," << tuA << "," << tuB << "," << tu << "," << TShift[14] << "," <<  tcA << "," << tcB << "," << tc << "," << TShift[46] << "," << adjtdiff << std::endl;
                myfile.close();

              }
            }
        }
    }
    }
*/
/*
	if (goodtime2 == 4){
        float tuA = TDC[umbrA2];
        float tuB = TDC[umbrB2];
        float tcA = TDC[cubeA2];
        float tcB = TDC[cubeB2];

        float tu = (tuA+tuB)/2 + TShift[14] + 6.096*4.46 - 2.95*4.15 - (180/(2*15.4));
        float tc = (tcA+tcB)/2 + TShift[46] + 3.048*4.46 - 3.75*4.15 - (180/(2*15.4));

        float phidiff = Phi[46] - Phi[14];
        float rawtdiff = (tcA+tcB)/2 - (tuA+tuB)/2;
        float adjtdiff = tc-tu;

	//std::ofstream myfile;
        //myfile.open ("/home/gaps/sydney/jeff_compare.csv", std::ios::app);
        //myfile << evtno;
        //myfile << "," << "7" << "," << TDC[umbrA2] << "," << TDC[umbrB2] << "," << TDC[cubeA2] << "," << TDC[cubeB2] << "," << TShift[46] << "," << TShift[14] << "," << adjtdiff <<  std::endl;
        //myfile.close();

            if (goodped2 == 4){
              HitTime[67]->Fill(tu);
              HitTime[7]->Fill(tc);
              tDiff[11]->Fill(rawtdiff);
              Ch9Shift[158]->Fill(phidiff);
              tDiff[12]->Fill(adjtdiff);
              if (tuA-tuB > -1.0 && tuA-tuB < 1.0 && tcA-tcB > -1.0 && tcA-tcB < 1.0){
                tDiff[13]->Fill(rawtdiff);
                tDiff[14]->Fill(adjtdiff);
                if (adjtdiff > 0 && adjtdiff < 8) tDiff[15]->Fill(adjtdiff);
		QEnd2End[11]->Fill(QInt[umbrA2], adjtdiff);
        	QEnd2End[12]->Fill(QInt[umbrB2], adjtdiff);
        	QEnd2End[13]->Fill(QInt[umbrA2]+QInt[umbrB2], adjtdiff);
        	QEnd2End[14]->Fill(QInt[cubeA2], adjtdiff);
        	QEnd2End[15]->Fill(QInt[cubeB2], adjtdiff);
        	QEnd2End[16]->Fill(QInt[cubeA2]+QInt[cubeB2], adjtdiff);
		std::ofstream myfile;
                myfile.open ("/home/gaps/sydney/evt_list_goodped.csv", std::ios::app);
                myfile << evtno;
                myfile << "," << adjtdiff << "," << "7" << std::endl;
                myfile.close();
              }
            }
	    else{
	      HitTime[77]->Fill(tu);
              HitTime[17]->Fill(tc);
              tDiff[31]->Fill(rawtdiff);
              Ch9Shift[148]->Fill(phidiff);
              tDiff[32]->Fill(adjtdiff);
              if (tuA-tuB > -1.0 && tuA-tuB < 1.0 && tcA-tcB > -1.0 && tcA-tcB < 1.0){
                tDiff[33]->Fill(rawtdiff);
                tDiff[34]->Fill(adjtdiff);
                if (adjtdiff > 0 && adjtdiff < 8) tDiff[35]->Fill(adjtdiff);
                QEnd2End[31]->Fill(QInt[umbrA2], adjtdiff);
                QEnd2End[32]->Fill(QInt[umbrB2], adjtdiff);
                QEnd2End[33]->Fill(QInt[umbrA2]+QInt[umbrB2], adjtdiff);
                QEnd2End[34]->Fill(QInt[cubeA2], adjtdiff);
                QEnd2End[35]->Fill(QInt[cubeB2], adjtdiff);
                QEnd2End[36]->Fill(QInt[cubeA2]+QInt[cubeB2], adjtdiff);
		std::ofstream myfile;
                myfile.open ("/home/gaps/sydney/evt_list_badped.csv", std::ios::app);
                myfile << evtno;
                myfile << "," << adjtdiff << "," << "7" << std::endl;
                myfile.close();
	      }
	    }
	  }
	else{
	  if (IsHit[7] && IsHit[67]){
	    std::ofstream myfile;
            myfile.open ("/home/gaps/sydney/hit_but_badtime.csv", std::ios::app);
            myfile << evtno;
            myfile << "," << "7" << "," << TDC[umbrA2] << "," << TDC[umbrB2] << "," << TDC[cubeA2] << "," << TDC[cubeB2] << std::endl;
            myfile.close();
	  }
	}
}*/
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::FillPaddleHistos(void) {
  
  for (int i=0; i<NPAD; i++) {
    if (Paddle_A[i] > 0) { // Paddle-channel map exists
      QEnd2End[i]->Fill(QInt[Paddle_A[i]], QInt[Paddle_B[i]]);
      HitMask[i]->Fill(Hits[i]);
      //if ( TDC[Paddle_A[i]] > 0 && TDC[Paddle_B[i]] > 0 ) {
	//tDiff[i]->Fill(TDC[Paddle_A[i]] - TDC[Paddle_B[i]]);
	//Ch9Shift[i]->Fill(TShift[i]);
      //}
      if (IsHit[i]) { // Both ends of paddle hit
	//if (EarlyPaddle>60) { // Demand a UMB or COR paddle hit
	if (EarlyPaddle>60 && EarlyPaddle<109) { // Demand a UMB paddle hit
	  HitTime[i]->Fill(HitT[i]);
	  FirstPaddle->Fill(EarlyPaddle);
	  FirstTime->Fill(EarlyTime);
	}
	HitPosition[i]->Fill(delta[i]);
	//HitGAPS->Fill(HitX[i], HitY[i], HitZ[i]);
	if (i<61) HitCube->Fill(HitX[i], HitY[i], HitZ[i]);
	else if (i<73){
	  if ((TDC[Paddle_A[i]] - TDC[Paddle_B[i]] < 2.0) && (TDC[Paddle_A[i]] - TDC[Paddle_B[i]] > -2.0)){
	  HitGAPS->Fill(HitX[i], HitY[i], HitZ[i]);
	  if (NPadCube+NPadUmbrella+NPadCortina == 2){
	    std::ofstream myfile;
            myfile.open ("run170_2HITS.csv", std::ios::app);
            myfile << evtno;
            myfile << "," << i << "," << IsHit[6] << "," << IsHit[7] << "," << IsHit[8] << "," << IsHit[9] << "," << IsHit[5] << std::endl;
            myfile.close();
	    if (IsHit[6] && (TDC[Paddle_A[6]] - TDC[Paddle_B[6]] < 2.0) && (TDC[Paddle_A[6]] - TDC[Paddle_B[6]] > -2.0)){
	      tDiff[i-60]->Fill(HitT[6]-HitT[i]);
            }
	    if (IsHit[7] && (TDC[Paddle_A[7]] - TDC[Paddle_B[7]] < 2.0) && (TDC[Paddle_A[7]] - TDC[Paddle_B[7]] > -2.0)){
              tDiff[i-61+20]->Fill(HitT[7]-HitT[i]);
            }
	    if (IsHit[8] && (TDC[Paddle_A[8]] - TDC[Paddle_B[8]] < 2.0) && (TDC[Paddle_A[8]] - TDC[Paddle_B[8]] > -2.0)){
              tDiff[i-61+40]->Fill(HitT[8]-HitT[i]);
            }
	    if (IsHit[9] && (TDC[Paddle_A[9]] - TDC[Paddle_B[9]] < 2.0) && (TDC[Paddle_A[9]] - TDC[Paddle_B[9]] > -2.0)){
              tDiff[i-1]->Fill(HitT[9]-HitT[i]);
            }
	    if (IsHit[5] && (TDC[Paddle_A[5]] - TDC[Paddle_B[5]] < 2.0) && (TDC[Paddle_A[5]] - TDC[Paddle_B[5]] > -2.0)){
              tDiff[i+19]->Fill(HitT[5]-HitT[i]);
            }
	  }
	}
	}
	else if (i<109) HitUmbrella->Fill(HitX[i], HitY[i], HitZ[i]);
	else if (i<161) HitCortina->Fill(HitX[i], HitY[i], HitZ[i]);
	float q_ave = (QInt[Paddle_A[i]] + QInt[Paddle_B[i]]) / 2.0;
	QvPosition[i]->Fill(delta[i], q_ave);
	if ( ABS(delta[i]) < 50 ) { // hit within middle meter
	  QvPositionA[i]->Fill(delta[i], QInt[Paddle_A[i]]);
	  QvPositionB[i]->Fill(delta[i], QInt[Paddle_B[i]]);
	}
      }
    }
  }
  if (beta > 0) BetaDist->Fill(beta); // Only when beta was calculated
  NPaddlesCube->Fill(NPadCube);
  NPaddlesUmbrella->Fill(NPadUmbrella);
  NPaddlesCortina->Fill(NPadCortina);
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

//==================END OF PEAK STUFF======================

void EventGAPS::Message(const char *s) {
  //cerr << s << endl;
}

