#include <stdlib.h>
#include <stdio.h>
#include <TFile.h>
#include <TTree.h>
#include <TF1.h>
#include <TGraph.h>
#include <TMath.h>
#include <fstream>

/* Waveform stuff. */
#include "../include/EventGAPS.h"

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

  float umb[12] = {0.215, 0.187, 0.172, 0.154, 0.102, 0.000, 1.021, 0.279,
		   1.064, 0.431, 1.235, 0.390};
  float top[12] = {0.070, 0.537,-0.008, 0.471, 0.046, 0.504,-0.143, 0.050,
		  -0.166, 0.003,-0.089, 0.000};
  float bot[12] = {0.792, 0.000, 0.631, 0.751, 0.772, 0.615, 0.626, 0.942,
		   0.664, 0.000, 0.716, 0.000};
  for (int i=0;i<NPAD;i++) {
    if (i>0&&i<13) Offset[i] = top[i-1];
    else if (i>12&&i<25) Offset[i] = bot[i-13];
    else if (i>60&&i<73) Offset[i] = -1.0*umb[i-61];
    //if (i>0&&i<13) Offset[i] = umb[i-1];
    //else if (i>12&&i<25) Offset[i] = umb[i-1];
    ///else if (i>60&&i<73) Offset[i] = umb[i-1];
    else Offset[i] = 0.0;
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
  
  if (0)   
    for (int i=0; i<NPAD; i++) {
      printf("PadID %3d  -> RB_A %3d %2d %2d; RB_B %3d %2d %2d\n", i,
	     Paddle_A[i], (int)Paddle_A[i]/NCH, Paddle_A[i]%NCH, 
	     Paddle_B[i], (int)Paddle_B[i]/NCH, Paddle_B[i]%NCH); 
    }
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
    tDiff[b] = new TH1D(text, "", 400, -100, 100);
    tDiff[b]->GetXaxis()->SetTitle("TDC Difference");
    tDiff[b]->GetYaxis()->SetTitle("Counts");
  }

  //Ch9 time shift
  for (int b = 0; b < NPAD; b++) {
    sprintf(text, "Ch9Shift[%d]", b);
    Ch9Shift[b] = new TH1D(text, "", 400, -15, 15);
    Ch9Shift[b]->GetXaxis()->SetTitle("Ch9 Shift");
    Ch9Shift[b]->GetYaxis()->SetTitle("Counts");
  }
  
  for (int b = 0; b < 2; b++) {
    sprintf(text, "Ch9Good[%d]", b);
    Ch9Good[b] = new TH2F(text, "", 200, -10, 10, 200, -10, 10);
    Ch9Good[b]->GetXaxis()->SetTitle("Paddle 67");
    Ch9Good[b]->GetYaxis()->SetTitle("Paddle 6 or 7");
    sprintf(text, "Ch9Bad[%d]", b);
    Ch9Bad[b] = new TH2F(text, "", 200, -10, 10, 200, -10, 10);
    Ch9Bad[b]->GetXaxis()->SetTitle("Paddle 67");
    Ch9Bad[b]->GetYaxis()->SetTitle("Paddle 6 or 7");
  }

  // Paddle Hit times
  for (int b = 0; b < NPAD; b++) {
    sprintf(text, "HitTime[%d]", b);
    HitTime[b] = new TH1F(text, "", 250, -2, 14);
    HitTime[b]->GetXaxis()->SetTitle("Hit Time (ns)");
    HitTime[b]->GetYaxis()->SetTitle("Counts");
  }

  // Earliest Paddle hit
  FirstPaddle = new TH1I("First Paddle Hit", "", 160, 0.5, 160.5);
  FirstPaddle->GetXaxis()->SetTitle("First Paddle Hit");
  FirstPaddle->GetYaxis()->SetTitle("Counts");
  // Earliest Paddle hit time
  FirstTime = new TH1F("First Hit Time", "", 300, 10.5, 160.5);
  FirstTime->GetXaxis()->SetTitle("First Hit Time");
  FirstTime->GetYaxis()->SetTitle("Counts");
  // Earliest Paddle hit time
  FirstTimeBad = new TH1F("First Hit Time (Bad", "", 300, 10.5, 160.5);
  FirstTimeBad->GetXaxis()->SetTitle("First Hit Time");
  FirstTimeBad->GetYaxis()->SetTitle("Counts");

  // Distribution of Beta
  BetaDist1 = new TH1F("Beta6 Distribution", "", 560, -0.05, 5.55);
  BetaDist1->GetXaxis()->SetTitle("Beta Value");
  BetaDist1->GetYaxis()->SetTitle("Counts");
  BetaDist2 = new TH1F("Beta7 Distribution", "", 560, -0.05, 5.55);
  BetaDist2->GetXaxis()->SetTitle("Beta Value");
  BetaDist2->GetYaxis()->SetTitle("Counts");
  BetaDist3 = new TH1F("Beta18 Distribution", "", 560, -0.05, 5.55);
  BetaDist3->GetXaxis()->SetTitle("Beta Value");
  BetaDist3->GetYaxis()->SetTitle("Counts");
  BetaDist4 = new TH1F("Beta19 Distribution", "", 560, -0.05, 5.55);
  BetaDist4->GetXaxis()->SetTitle("Beta Value");
  BetaDist4->GetYaxis()->SetTitle("Counts");

  // Histograms comparing the charge measured at both ends of the paddle.
  for (int b = 0; b < NPAD; b++) {
    sprintf(text, "QEnd2End[%d]", b);
    QEnd2End[b] = new TH2D(text, "", 300, lo_ch, hi_ch,
                              300, lo_ch, hi_ch);
    QEnd2End[b]->GetXaxis()->SetTitle("End A");
    QEnd2End[b]->GetYaxis()->SetTitle("End B");
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
  NPaddlesCube = new TH1I("NPaddles Hit Cube", "", 12, -1.5, 10.5);
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
  
  TFile *outfile = TFile::Open("/home/gaps/userspace/sydney/outfile.root","RECREATE"); 
  
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

  int PEDS = 0;
  int PEAK = 0;
  int QVSP = 0;
  
  int start = 1; // No Sipm ch = 0 or paddle = 0
  max_sipm = max_paddle*2-1;
  //write all the Trace plots to the root file
  if (PEDS) {
    //Peddir->cd();
    for (int i = start; i < max_sipm; i++) {
      pedHist[i]->Write();
      pedRMSHist[i]->Write();
    }
  }

  if (PEAK) {
    //Peakdir->cd();
    for (int i = start; i < max_sipm; i++) Peak[i]->Write();
    
    //Chargedir->cd();
    for (int i = start; i < max_sipm; i++) {
      Charge[i]->Write();
      Charge_cut[i]->Write();
    }
  }
  
  for (int j = start; j < max_paddle; j++) QEnd2End[j]->Write();
  HitGAPS->Write();
  HitCube->Write();
  HitCortina->Write();
  HitUmbrella->Write();

  if (QVSP) {
    for (int j = start; j < max_paddle; j++) HitPosition[j]->Write();
    for (int j = start; j < max_paddle; j++) {
      QvPosition[j]->Write();
      QvPositionA[j]->Write();
      QvPositionB[j]->Write();
    }
  }
  
  //TDCdir->cd();
  FirstPaddle->Write();
  FirstTime->Write();
  FirstTimeBad->Write();
  BetaDist1->Write();
  BetaDist2->Write();
  BetaDist3->Write();
  BetaDist4->Write();
  for (int j = 0; j < 2; j++) {Ch9Good[j]->Write(); Ch9Bad[j]->Write();}
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

  float rms_m = 3.0;
  
  for (int i=0; i<NTOT; i++) {
    if (wData[i] != NULL) { 
      wData[i]->SetPedBegin(Ped_low);
      wData[i]->SetPedRange(Ped_win);
      wData[i]->CalcPedestalRange();    // Calculate pedestals
      wData[i]->SubtractPedestal();     // Subtract pedestals
      // Now store the values
      Pedestal[i] = wData[i]->GetPedestal(); 
      PedRMS[i]   = wData[i]->GetPedsigma();

      std::ofstream myfile;
      myfile.open ("Peds.csv", std::ios::app);
      myfile << evtno << "," << i << "," << Pedestal[i] << "," << PedRMS[i] << std::endl;
      myfile.close();

      //if ( PedRMS[i] > 15 ) printf("Channel %d: %8.1f\n", i, PedRMS[i]);
      // Check for data mangling
      
      if ( PedRMS[i] > rms_m ) {
	if ( i%NCH==7 )
	  if (PedRMS[i-1]>rms_m && PedRMS[i-2]>rms_m && PedRMS[i-3]>rms_m &&
	      PedRMS[i-4]>rms_m && PedRMS[i-5]>rms_m && PedRMS[i-6]>rms_m) {
	    printf("Data Mangled Event %ld: RB %d\n", evtno, i/NCH);
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
	//wData[i]->FindTdc(0, GAPS::CONSTANT);     // Simple CFD
	TDC[i] = wData[i]->GetTdcs(0);
	//printf("%ld: %d - %7.3f\n", evtno, i, TDC[i]);
	std::ofstream myfile;
        myfile.open ("TDC_QInt.csv", std::ios::app);
        myfile << evtno << "," << i << "," << TDC[i] << "," << QInt[i] << std::endl;
        myfile.close();
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

  for (int i=0; i<NRB-1; i++) {
    // For each RB, subtract phi from reference value...
    if (RBInData[i]) {
      //float phi_shift = ref_phi - Phi[i];
      float phi_shift = Phi[i] - ref_phi;
      // Ensure the shift is in proper range: -Pi/3 < shift < Pi/3
      while (phi_shift < -PI/2.0) phi_shift += 2.0*PI;
      while (phi_shift >  PI/2.0) phi_shift -= 2.0*PI;
      // Store the timing shift for the ch9 correction 
     TShift[i] = phi_shift/(2.0*PI*0.02);
    } else TShift[i] = -999.0;
  }

  if (0) { 
    printf("%ld - Phase Analysis: %d %6.3f\n", evtno, ref_rb, ref_phi);
    for (int i=0; i<NRB; i++) {
      if (Phi[i]>-998.0) printf("%2d : %6.3f %6.3f\n", i, TShift[i], Phi[i]);
      //printf(" %.3f (%.3f)", TShift[i], Phi[i]);
      //if (i%4 == 3) printf("\n");
    }
    //printf("%d %6.3f\n", ref_rb, ref_phi);
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
      //if (i<61 || i>72) // For all paddles except UMB
      
      if ( TDC[chA]>5 && TDC[chA]<220 && TDC[chB]>5 && TDC[chB]<220 ) {
	if (PedRMS[chA]<2.0 && PedRMS[chB]<2.0) 
	  IsHit[i] = true;
	HitT[i] -= Offset[i];
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
  for (int i=61; i<73; i++) { // UMB-Center
  //for (int i=66; i<68; i++) {
    if (IsHit[i] ) {
      if (0) {
	printf(" %3d %7.3f %7.3f %7.2f -%7.2f (%7.2f) %8.2f %6.2f\n", i,
	       TCorrEvent[i],TCorrFixed[i], TDC[Paddle_A[i]],TDC[Paddle_B[i]],
	       TDC[Paddle_A[i]]+TDC[Paddle_B[i]], HitT[i],
	       TShift[RB[Paddle_A[i]]]);
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
  int ctr=0;
  // Now that we have the hit times and positions, calculate beta
  //for (int i=0; i<61; i++) { // Only calculate beta for cube hits
  for (int i=0; i<13; i++) { // Only calculate beta for cube-top hits
  //for (int i=13; i<25; i++) { // Only calculate beta for cube-bot hits
  //for (int i=6; i<8; i++) { // Only calculate beta for middle cube-top hits
    //if (IsHit[i] && (i==6 || i==7 || i==8 || i==9) ) {
    if ( IsHit[i] ) {
      float dist_sq = SQR(HitX[i]-HitX[e_pad]) + SQR(HitY[i]-HitY[e_pad]) +
	SQR(HitZ[i]-HitZ[e_pad]);
      float t_diff = HitT[i] - HitT[EarlyPaddle];
      if ( 0 && (i==5) && t_diff < 2.29 ) {
	printf("Too Early: %ld -- %d - %5.2f (%6.2f %6.2f)\n",evtno,i,t_diff,
	       HitT[i], HitT[EarlyPaddle]);
      }
      float speed = sqrt(dist_sq) / (t_diff); // mm/ns
      ctr++;
      //if (ctr>1) printf("%ld: Previous beta = %.3f\n", evtno, beta);
      beta = speed/(CSPEED);
      //if (0 && e_pad>60 && e_pad<73) {
      if (0 && (EarlyPaddle>65 && EarlyPaddle<68) ) {
	printf("Positions: (%d %d) (%.2f %.2f)  (%.2f %.2f)  (%.2f %.2f)\n",
	       i, EarlyPaddle, HitX[i], HitX[EarlyPaddle], HitY[i],
	       HitY[EarlyPaddle], HitZ[i], HitZ[EarlyPaddle]);
	printf("Speeds: (%.2f %.2f): %.2f   %.2f  %.2f\n", HitT[i],
	       HitT[EarlyPaddle], t_diff,  speed, beta); 
	printf(" %3d %7.3f %7.3f %7.2f -%7.2f (%7.2f) (%7.2f %7.2f) %3d\n",
	       i,TCorrEvent[i],TCorrFixed[i],TDC[Paddle_A[i]],TDC[Paddle_B[i]],
	       TDC[Paddle_A[i]]+TDC[Paddle_B[i]], HitT[i],
	       HitT[EarlyPaddle], EarlyPaddle);
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
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::FillPaddleHistos(void) {

  int NHitPaddles = NPadCube + NPadUmbrella + NPadCortina;
  
  for (int i=0; i<NPAD; i++) {
    if (Paddle_A[i] > 0) { // Paddle-channel map exists
      QEnd2End[i]->Fill(QInt[Paddle_A[i]], QInt[Paddle_B[i]]);
      HitMask[i]->Fill(Hits[i]);
      if ( TDC[Paddle_A[i]] > 0 && TDC[Paddle_B[i]] > 0 ) {
	tDiff[i]->Fill(TDC[Paddle_A[i]] - TDC[Paddle_B[i]]);
	Ch9Shift[i]->Fill(TShift[RB[Paddle_A[i]]]); // Paddle ends in same RB
      }
      if (IsHit[i]) { // Both ends of paddle hit
	//if (EarlyPaddle>60) { // Demand a UMB or COR paddle hit
	//if (EarlyPaddle>60 && EarlyPaddle<109) { // Demand a UMB paddle hit
	// Only record HitTimes[] and beta for events which hit two
	// center umbrella paddles and two center Cube top paddles
	// (and within 10cm of center).
	//if ( (EarlyPaddle>65&&EarlyPaddle<68) && ((i>5&&i<8) || (i>17&&i<20))){ 
	if ( (EarlyPaddle>60&&EarlyPaddle<73) && (i>0&&i<13) ) { 
	//if ( (EarlyPaddle>60&&EarlyPaddle<73) && (i>12&&i<25) ) { 
	  if ( ABS(delta[i])< 15.4 && ABS(delta[EarlyPaddle])<15.4 &&
	       NHitPaddles < 4 ) {
	    int ind = (EarlyPaddle-61)*12;
	    //if (EarlyPaddle==66) 
	    HitTime[i+ind]->Fill(HitT[i]);
	    // HitTime[i+ind]->Fill(HitT[i] - Offset[EarlyPaddle]); // Subtract offset for UMB paddle
	      //else
	      //HitTime[i+10]->Fill(HitT[i]);
	    FirstPaddle->Fill(EarlyPaddle);
	    
	    /*
	    printf("\nEv %ld - %d ( %6.3f %6.3lf %6.3f ) ",
		   evtno, i,HitT[i],TShift[RB[Paddle_A[i]]],Phi[RB[Paddle_A[i]]]);
	    printf("- %d ( %6.3f %6.3lf %6.3f )\n",
		   EarlyPaddle, HitT[EarlyPaddle],
		   TShift[RB[Paddle_A[EarlyPaddle]]],
		   Phi[RB[Paddle_A[EarlyPaddle]]]);
	    printf("%ld TDC     %d %d - %.3f = %.3f : %d - %.3f = %.3f\n",
		   evtno, i, 
		   Paddle_A[i], TDC[Paddle_A[i]], TDC_Cor[Paddle_A[i]],
		   Paddle_B[i], TDC[Paddle_B[i]], TDC_Cor[Paddle_B[i]]);
	    printf("%ld HIT     %d ( %.3f %.3f %.3f ) : %.3f\n", evtno, i, 
		   TCorrEvent[i], TCorrFixed[i], Dimension[i][0], HitT[i]);
	    printf("%ld TDC_ref %d %d - %.3f = %.3f : %d - %.3f = %.3f\n",
		   evtno, EarlyPaddle, 
		   Paddle_A[EarlyPaddle], TDC[Paddle_A[EarlyPaddle]],
		   TDC_Cor[Paddle_A[EarlyPaddle]], Paddle_B[EarlyPaddle],
		   TDC[Paddle_B[EarlyPaddle]], TDC_Cor[Paddle_B[EarlyPaddle]]);
	    printf("%ld HIT_ref %d ( %.3f %.3f %.3f ) : %.3f\n",
		   evtno, EarlyPaddle,
		   TCorrEvent[EarlyPaddle], TCorrFixed[EarlyPaddle],
		   Dimension[EarlyPaddle][0], HitT[EarlyPaddle]);
	    */

	    if (beta>0 && (i==6||i==7)) {
	      //printf("Found Event %ld - %d: %.3f %.2f %.2f %.2f\n", evtno,i,
	      //     EarlyTime, beta, TShift[RB[Paddle_A[EarlyPaddle]]],
	      //     TShift[RB[Paddle_A[i]]]); 
	      if (beta > 1.0) {
		Ch9Bad[i-6]->Fill(TShift[RB[Paddle_A[EarlyPaddle]]],
				  TShift[RB[Paddle_A[i]]]);
		FirstTimeBad->Fill(EarlyTime);
	      } else { 
		Ch9Good[i-6]->Fill(TShift[RB[Paddle_A[EarlyPaddle]]],
				TShift[RB[Paddle_A[i]]]);
		FirstTime->Fill(EarlyTime);
	      }
	    }
	    if (beta>0 && i==6) BetaDist1->Fill(beta); 
	    if (beta>0 && i==7) BetaDist2->Fill(beta); 
	    if (beta>0 && i==8) BetaDist3->Fill(beta); 
	    if (beta>0 && i==9) BetaDist4->Fill(beta); 
	    if ( 1 && (i==5||i==6||i==18||i==19) && beta > 1.1 ) {
	      printf("Fast: %ld %d - %5.2f %6.2f %6.2f\n",evtno,i,beta,
	       HitT[i], HitT[EarlyPaddle]);
	    }
	  }
	}
	HitPosition[i]->Fill(delta[i]);
	HitGAPS->Fill(HitX[i], HitY[i], HitZ[i]);
	if (i<61) HitCube->Fill(HitX[i], HitY[i], HitZ[i]);
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
  //if (beta > 0) BetaDist1->Fill(beta); // Only when beta was calculated
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

