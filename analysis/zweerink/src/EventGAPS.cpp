#include <stdlib.h>
#include <stdio.h>
#include <TFile.h>
#include <TTree.h>
#include <TF1.h>
#include <TGraph.h>
#include <TMath.h>

/* Waveform stuff. */
#include "../include/EventGAPS.h"

// Some useful macros
#define SQR(A)               ( (A) * (A) )
#define ABS(A)               ( ( (A<0) ? -(A) : (A) ) )
#define PI                   3.14159265
// In units of mm/ns
#define CSPEED               299.792458


using namespace std;
//using namespace GAPS;

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
  sc_speed = 154.0;   // Scintillator speed of light, in mm/ns
  
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

  float t_off[NPAD] = {
        0.269,  0.685,  0.223,  0.605,  0.293,  0.622, -0.160, -0.020,
        0.071, -0.117, -0.148, -0.153,  1.073,  1.184,  0.891,  0.935,
        1.078,  0.123,  0.000,  1.200,  0.799,  1.189,  0.964,  0.000,
        0.705,  0.205,  0.808,  0.771,  0.793,  0.798,  0.981,  0.952,
        0.205,  0.559,  0.506,  0.616,  0.000,  0.661,  0.272,  0.523,
        0.000,  0.011,  0.732,  0.000,  0.000,  0.385,  0.723,  0.000,
        0.447,  0.801,  0.505,  0.924,  0.652,  0.883,  0.988,  1.042,
        0.411,  0.541, -0.318,  0.446, -0.217, -0.223, -0.144, -0.169,
       -0.060, -0.000, -0.951, -0.291, -1.041, -0.395, -1.193, -0.302,
        0.424,  0.000,  0.757,  0.529,  0.947,  0.492, -0.571, -0.442,
       -0.619, -0.500, -0.933, -0.802, -0.636, -0.708,  0.000,  0.000,
        0.000,  0.000,  0.623,  1.638, -0.154,  0.000,  0.000,  0.000,
        0.000,  1.680,  1.227,  1.489,  1.014,  1.263,  1.614,  1.075,
        1.050,  0.832,  0.806,  0.000, -0.475, -0.306, -0.432, -0.520,
       -0.180, -0.808, -0.808, -3.932, -0.808, -0.808, -0.988, -1.133,
       -1.100, -1.077, -1.241, -3.921, -0.506, -0.506, -0.506, -0.506,
       -1.152, -0.943, -0.969, -1.323, -1.048, -3.513, -3.878, -4.161,
       -3.997, -0.732, -0.848, -0.133, -0.693, -0.031, -0.708, -2.737,
       -0.505, -3.544, -0.505, -0.505,  0.000,  0.000,  0.000,  0.000,
        0.000,  0.000,  0.000,  0.000,  0.000,  0.000,  0.000,  0.000 };

  
  for (int i=0;i<NPAD;i++) {
    if (i>0&&i<161) Offset[i] = t_off[i-1];
    else Offset[i] = 0.0;
    // If we want to calculate the offsets, set them all to zero.               
    //Offset[i] = 0.0;                                                          
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
    wData[i]  = NULL;
  }
  for (int i=0; i<NRB;  i++) {
    wClock[i] = NULL;
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
      // If we have a valid RB, set the appropriate values
      if (sipm->RB[ch] > 0 && sipm->RB[ch] < NRB) {
	RB[ch]     = sipm->RB[ch];
	RB_ch[ch]  = sipm->RB_ch[ch];
	Paddle[ch] = sipm->PaddleID[ch];
	PadEnd[ch] = sipm->PaddleEnd[ch];
      }
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
    sprintf(text, "Vpeak[%d]", b);
    //  Peak[b] = new TH1D(text, "", PeakBins, PeakLo, PeakHi);
    Peak[b] = new TH1D(text, "", 160, -10.0, 150.0);
    Peak[b]->GetXaxis()->SetTitle("Vpeak (mV)");
    Peak[b]->GetYaxis()->SetTitle("Counts");
  }
  
  //Histograms containing the charge distribution
  for (int b = 0; b < NTOT; b++) {
    sprintf(text, "Charge[%d]", b);
    // Charge[b] = new TH1D(text, "", 200, lo_ch, hi_ch);
    Charge[b] = new TH1D(text, "", 130, -10.0, 60.0);
    Charge[b]->GetXaxis()->SetTitle("Charge(pC)");
    Charge[b]->GetYaxis()->SetTitle("Counts");
  }

  for (int b = 0; b < NTOT; b++) {
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
  
  // TOT histos
  for (int b = 0; b < NTOT; b++) {
    sprintf(text, "totLo[%d]", b);
    totLo[b] = new TH1D(text, "", 300, -5, 70);
    totLo[b]->GetXaxis()->SetTitle("TOT - Lo Threshold");
    totLo[b]->GetYaxis()->SetTitle("Counts");
    sprintf(text, "totHi[%d]", b);
    totHi[b] = new TH1D(text, "", 300, -5, 70);
    totHi[b]->GetXaxis()->SetTitle("TOT - Hi Threshold");
    totHi[b]->GetYaxis()->SetTitle("Counts");
  }

  // TDC diffs
  for (int b = 0; b < NPAD; b++) {
    sprintf(text, "tDiff[%d]", b);
    tDiff[b] = new TH1D(text, "", 400, -100, 100);
    tDiff[b]->GetXaxis()->SetTitle("TDC Difference");
    tDiff[b]->GetYaxis()->SetTitle("Counts");
  }

  // Ch9 time shift
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

  // Distribution of Beta
  BetaDist = new TH1F("Beta Distribution", "", 560, -0.05, 5.55);
  BetaDist->GetXaxis()->SetTitle("Beta Value");
  BetaDist->GetYaxis()->SetTitle("Counts");

  // Histograms comparing the charge measured at both ends of the paddle.
  for (int b = 0; b < NPAD; b++) {
    sprintf(text, "QEnd2End[%d]", b);
    QEnd2End[b] = new TH2D(text, "", 300, lo_ch, hi_ch,
                              300, lo_ch, hi_ch);
    QEnd2End[b]->GetXaxis()->SetTitle("End A");
    QEnd2End[b]->GetYaxis()->SetTitle("End B");
  }

  // Hit mask histograms
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
  
  TFile *outfile = TFile::Open("./outfile.root","RECREATE"); 
  
  //create directories for the raw plots
  TDirectory *savdir = gDirectory; 
  outfile->cd();
  TDirectory *Peddir = outfile->mkdir("Pedestals");
  TDirectory *Peakdir = outfile->mkdir("VPeakplots");
  TDirectory *Chargedir = outfile->mkdir("Chargeplots");
  TDirectory *Hitmaskdir = outfile->mkdir("Hitmasks");
  
  TDirectory *TDCdir = outfile->mkdir("TDCplots");
  TDirectory *TOTdir = outfile->mkdir("TOTplots");

  int PEDS = 0;
  int PEAK = 0;
  int QVSP = 0;
  
  int start = 1; // No Sipm ch = 0 or paddle = 0
  max_sipm = max_paddle*2-1;

  if (PEDS) {
    Peddir->cd();
    for (int i = start; i < max_sipm; i++) {
      pedHist[i]->Write();
      pedRMSHist[i]->Write();
    }
  }

  if (PEAK) {
    Peakdir->cd();
    for (int i = start; i < max_sipm; i++) Peak[i]->Write();
    
    Chargedir->cd();
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
  
  TDCdir->cd();
  FirstPaddle->Write();
  FirstTime->Write();
  BetaDist->Write();
  for (int j = 0; j < 2; j++) {Ch9Good[j]->Write(); Ch9Bad[j]->Write();}
  for (int j = start; j < max_paddle; j++) HitTime[j]->Write();
  for (int j = start; j < max_paddle; j++) Ch9Shift[j]->Write();
  for (int j = start; j < max_paddle; j++) tDiff[j]->Write();
  for (int i = start; i < max_sipm; i++) tdcCFD[i]->Write();
  
  TOTdir->cd();
  //for (int j = start; j < max_paddle; j++) {
  for (int i = start; i < max_sipm; i++) {
    totLo[i]->Write();
    totHi[i]->Write();
  }
  
  Hitmaskdir->cd();
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
void EventGAPS::OffsetHistograms(bool flag=false) {

  char text[400];
  CalcOffset = flag;
  
  // Histos: Hit times between each UMB-Center and cube paddles
  for (int b = 0; b < NCUBT; b++) {
    for (int c = 0; c < NUMBC; c++) {
      sprintf(text, "H_OffCube[%d][%d]", b, c);
      H_OffCube[b][c] = new TH1F(text, "", 160, -5, 11);
      sprintf(text, "Hit Time (ns): Cube %d - UmbCen %d", b, c+61);
      H_OffCube[b][c]->GetXaxis()->SetTitle(text);
      H_OffCube[b][c]->GetYaxis()->SetTitle("Counts");
    }
  }

  // Histos: Hit times between each CUB-Center and umbrella paddles
  for (int b = 0; b < NUMBT; b++) {
    for (int c = 0; c < NCUBC; c++) {
      sprintf(text, "H_OffUmb[%d][%d]", b, c);
      H_OffUmb[b][c] = new TH1F(text, "", 160, -5, 11);
      sprintf(text, "Hit Time (ns): Umb %d - CubeTop %d", b, c+1);
      H_OffUmb[b][c]->GetXaxis()->SetTitle(text);
      H_OffUmb[b][c]->GetYaxis()->SetTitle("Counts");
    }
  }

  // Histos: Hit times between CUB-Side and umbrella paddles (North)
  for (int b = 0; b < NCORT; b++) {
    for (int c = 0; c < NCUBS; c++) {
      sprintf(text, "H_OffCorN[%d][%d]", b, c);
      H_OffCorN[b][c] = new TH1F(text, "", 160, -5, 11);
      sprintf(text, "Hit Time (ns): Cort %d - CubeN %d", b+109, c+25);
      H_OffCorN[b][c]->GetXaxis()->SetTitle(text);
      H_OffCorN[b][c]->GetYaxis()->SetTitle("Counts");
    }
  }

  // Histos: Hit times between CUB-Side and umbrella paddles (East)
  for (int b = 0; b < NCORT; b++) {
    for (int c = 0; c < NCUBS; c++) {
      sprintf(text, "H_OffCorE[%d][%d]", b, c);
      H_OffCorE[b][c] = new TH1F(text, "", 160, -5, 11);
      sprintf(text, "Hit Time (ns): Cort %d - CubeE %d", b+119, c+33);
      H_OffCorE[b][c]->GetXaxis()->SetTitle(text);
      H_OffCorE[b][c]->GetYaxis()->SetTitle("Counts");
    }
  }

  // Histos: Hit times between CUB-Side and umbrella paddles (South)
  for (int b = 0; b < NCORT; b++) {
    for (int c = 0; c < NCUBS; c++) {
      sprintf(text, "H_OffCorS[%d][%d]", b, c);
      H_OffCorS[b][c] = new TH1F(text, "", 160, -5, 11);
      sprintf(text, "Hit Time (ns): Cort %d - CubeS %d", b+129, c+41);
      H_OffCorS[b][c]->GetXaxis()->SetTitle(text);
      H_OffCorS[b][c]->GetYaxis()->SetTitle("Counts");
    }
  }

  // Histos: Hit times between CUB-Side and umbrella paddles (West)
  for (int b = 0; b < NCORT; b++) {
    for (int c = 0; c < NCUBS; c++) {
      sprintf(text, "H_OffCorW[%d][%d]", b, c);
      H_OffCorW[b][c] = new TH1F(text, "", 160, -5, 11);
      sprintf(text, "Hit Time (ns): Cort %d - CubeW %d", b+139, c+49);
      H_OffCorW[b][c]->GetXaxis()->SetTitle(text);
      H_OffCorW[b][c]->GetYaxis()->SetTitle("Counts");
    }
  }
}

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::FillEventValues(struct EventInfo *evt) {
  // All SiPM values in EventInfo structure are stored by Paddle
  // number. We need to find the proper SiPM number (NTOT) before
  // storing the SiPM values locally.
  
  int sipm[2];
  for (int i=0; i<NPAD; i++) {
    if (evt->Ped[i][0] > -900 && evt->Ped[i][1] > -900) { // Good Paddle
      sipm[0] = Paddle_A[i];
      sipm[1] = Paddle_B[i];
      for (int j=0; j<2; j++) {
	Pedestal[sipm[j]] = evt->Ped[i][j];
	PedRMS[sipm[j]]   = evt->PedRMS[i][j];
	VPeak[sipm[j]]    = evt->VPeak[i][j];
	QInt[sipm[j]]     = evt->Charge[i][j];
	TDC[sipm[j]]      = evt->TDC[i][j];
	TotLo[sipm[j]]    = evt->TOTLo[i][j];
	TotHi[sipm[j]]    = evt->TOTHi[i][j];
      }
      /*printf("%3d %7.2f %7.2f %7.2f %7.2f %6.2f %6.2f (%d %d)\n", i,
	     TDC[sipm[0]],TDC[sipm[1]],
	     VPeak[sipm[0]],VPeak[sipm[1]],
	     QInt[sipm[0]],QInt[sipm[1]],
	     Paddle_A[i], Paddle_B[i]);*/
    }
  }
  // Phi is already indexed by RB number and will be set by the
  // AnalyzePhases() call
}

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
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::AnalyzePulses(float Pulse_low, float Pulse_win) {

  for (int i=0; i<NTOT; i++) {
    if (wData[i] != NULL) { 
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

      // If we have a pulse above threshold, find the TDC value
      wData[i]->FindPeaks(Pulse_low, Pulse_win);
      if ( (wData[i]->GetNumPeaks() > 0) ) {
	wData[i]->FindTdc(0, GAPS::CFD_SIMPLE);     // Simple CFD
	//wData[i]->FindTdc(0, GAPS::CONSTANT);     // Simple CFD
	TDC[i] = wData[i]->GetTdcs(0);

	// Just for test purpose, fill the TOT values with dummies
	TotLo[i] = 20.0;
	TotHi[i] = 10.0;
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
      float phi_shift = Phi[i] - ref_phi;
      // Ensure the shift is in proper range: -Pi/3 < shift < Pi/3
      while (phi_shift < -PI/2.0) phi_shift += 2.0*PI;
      while (phi_shift >  PI/2.0) phi_shift -= 2.0*PI;
      // Store the timing shift for the ch9 correction 
     TShift[i] = phi_shift/(2.0*PI*0.02);
    } else TShift[i] = -999.0;
  }

  if (0) { // Print some diagnostics is useful
    printf("%ld - Phase Analysis: %d %6.3f\n", evtno, ref_rb, ref_phi);
    for (int i=0; i<NRB; i++) {
      if (Phi[i]>-998.0) printf("%2d : %6.3f %6.3f\n", i, TShift[i], Phi[i]);
    }
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
    //printf("%3d: %d ", i, IsHit[i]);

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
      
      // Check that the paddle has a good hit (TDCs in range, good peds)
      if ( TDC[chA]>5 && TDC[chA]<220 && TDC[chB]>5 && TDC[chB]<220 ) {
	if (PedRMS[chA]<2.0 && PedRMS[chB]<2.0) {
	  IsHit[i] = true;
	  // Next two lines are useful for printing values to compare
	  // with online quantities stored in the TofHits() class
	  if (0) {
	    printf("%3d %7.2f %7.2f %7.2f %7.2f %6.2f %6.2f", i, TDC[chA],
		   TDC[chB], VPeak[chA],VPeak[chB], QInt[chA],QInt[chB]);
	    printf(" %7.4f %5.2f %5.2f %4.2f %4.2f\n",
		   Phi[RB[chB]], Pedestal[chA],Pedestal[chB],
		   PedRMS[chA],PedRMS[chB]);
	  }
	}
	HitT[i] -= Offset[i];
      }
    }
    
    //printf("%d\n", IsHit[i]);
    if ( IsHit[i] ) {
      if (i<61) cube++;         // Paddle in Cube
      else if (i<109) upper++;  // Paddle in Umbrella
      else if (i<161) outer++;  // Paddle in Cortina
    }
  }
  
  NPadCube     = cube;
  NPadUmbrella = upper;
  NPadCortina  = outer;
  printf("NPad = %d %d %d\n", NPadCube, NPadUmbrella, NPadCortina); 
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
  for (int i=61; i<NPAD; i++) {
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
  
  // Now, subtract the earliest time from all other times
  if (e_pad > 0) { // Only if we found a umb/cor paddle hit
    for (int i=0; i<NPAD; i++)
      if (IsHit[i]) HitT[i] -= early;
  }
  int ctr=0;
  // Now that we have the hit times and positions, calculate beta
  for (int i=0; i<13; i++) { // Only calculate beta for cube-top hits
    if ( IsHit[i] ) {
      //printf("HitT: %f\t%f\t%f\n",HitX[i],HitY[i],HitZ[i]);fflush(stdout);
      float dist_sq = SQR(HitX[i]-HitX[e_pad]) + SQR(HitY[i]-HitY[e_pad]) +
	SQR(HitZ[i]-HitZ[e_pad]);
      float t_diff = HitT[i] - HitT[EarlyPaddle];
      float speed = sqrt(dist_sq) / (t_diff); // mm/ns
      ctr++;
      beta = speed/(CSPEED);
      //printf("Beta = %.2f  %.2f   %.2f\n",dist_sq,t_diff,beta); fflush(stdout);
      //BetaDist->Fill(1.0);
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
	
	// Fill TOT (lo/hi and A/B) values
	totLo[ch-1]->Fill(TotLo[Paddle_A[i]]);
	totLo[ch]->Fill(TotLo[Paddle_B[i]]);
	totHi[ch-1]->Fill(TotHi[Paddle_A[i]]);
	totHi[ch]->Fill(TotHi[Paddle_B[i]]);
	
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
	if ( (EarlyPaddle>60&&EarlyPaddle<73) && (i>0&&i<13) ) { 
	  if ( ABS(delta[i])< 15.4 && ABS(delta[EarlyPaddle])<15.4 &&
	       NHitPaddles < 4 ) {
	    int ind = (EarlyPaddle-61)*12;
	    HitTime[i+ind]->Fill(HitT[i]);
	    FirstPaddle->Fill(EarlyPaddle);
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
  // Histo for the number of paddles hit in parts of the detector
  //NPaddlesCube->Fill(NPadCube);
  //NPaddlesUmbrella->Fill(NPadUmbrella);
  //NPaddlesCortina->Fill(NPadCortina);
}

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::FillOffsetHistos(void) {
  
  for (int i=1; i<NCUBT; i++) { // Loop through CUB paddles
    if (Paddle_A[i] > 0) { // Paddle-channel map exists
      if ( IsHit[i] && (EarlyPaddle>60&&EarlyPaddle<73) ) { // Good hits
	// Here we use a much wider range on the paddle, which we can
	// do because we are calculating the residual compared to the
	// maximum time.
	if ( ABS(delta[i]) < 0.70*Dimension[i][0] &&
	     ABS(delta[EarlyPaddle]) < 0.70*Dimension[EarlyPaddle][0] &&
	     NPadCube+NPadUmbrella < 4 ) {
	  double dist = sqrt( SQR(HitX[i]-HitX[EarlyPaddle]) +
			      SQR(HitY[i]-HitY[EarlyPaddle]) +
			      SQR(HitZ[i]-HitZ[EarlyPaddle]) );
	  H_OffCube[i][EarlyPaddle-61]->Fill(HitT[i] - dist/CSPEED);
	}
      }
    }
  }

  for (int i=0; i<NUMBT; i++) { // Loop through UMB Paddles
    int umb = i+61;
    if (Paddle_A[umb] > 0 && IsHit[umb]) { // Pad-ch map exists, is hit
      for (int j=1; j<NCUBC+1; j++) { // Look for CUB-Top hit
	if ( IsHit[j] ) { 
	  // We have an UMB and CUB-Top hit -> fill H_OffUmb[][] 
	  
	  // Here we use a much wide range on the paddle, which we can
	  // do because we are calculating the residual compared to
	  // the maximum time.
	  if ( ABS(delta[umb]) < 0.70*Dimension[umb][0] &&
	       ABS(delta[j]) < 0.70*Dimension[j][0] &&
	       NPadCube+NPadUmbrella < 4 ) {
	    double dist = sqrt( SQR(HitX[umb]-HitX[j]) +
				SQR(HitY[umb]-HitY[j]) +
				SQR(HitZ[umb]-HitZ[j]) );
	    H_OffUmb[i][j]->Fill( (HitT[j] - HitT[umb]) - dist/CSPEED);
	  }
	}
      }
    }
  }
  
  // Now we fill the Cortina-Cube histos
  if (NPadCube+NPadCortina < 4) { // Only events with small nhits
    int cort_start = 109;
    for (int i=0; i<4*NCORT; i++) { // Loop through Cort paddles
      int cort_paddle = cort_start+i;
      if ( IsHit[cort_paddle] ) {
	// Loop through relevant CubeSide paddles
	int panel = i / (int)NCORT;
	int cube_start = 25+panel*NCUBS;
	for (int j=0; j<NCUBS; j++) { // Any Hit paddles?
	  int cube_paddle = cube_start+j;
	  if ( IsHit[cube_paddle] ) {
	    // Good paddle combo, calculate HitT and Tdist
	    if ( ABS(delta[cube_paddle]) < 0.70*Dimension[cube_paddle][0] &&
		 ABS(delta[cort_paddle]) < 0.70*Dimension[cort_paddle][0]) {
	      double dist = sqrt( SQR(HitX[cube_paddle]-HitX[cort_paddle]) +
				  SQR(HitY[cube_paddle]-HitY[cort_paddle]) +
				  SQR(HitZ[cube_paddle]-HitZ[cort_paddle]) );
	      double tdiff = HitT[cube_paddle] - HitT[cort_paddle];
	      double residual = tdiff - dist/CSPEED; 
	      if (panel==0) H_OffCorN[i%(int)NCORT][j]->Fill(residual);
	      if (panel==1) H_OffCorE[i%(int)NCORT][j]->Fill(residual);
	      if (panel==2) H_OffCorS[i%(int)NCORT][j]->Fill(residual);
	      if (panel==3) H_OffCorW[i%(int)NCORT][j]->Fill(residual);
	    }
	  }
	}
      }
    }
  }
}

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::WriteOffsetHistos(void) {
  TFile *outfile = TFile::Open("./offset.root","RECREATE"); 
  
  for (int i = 1; i < NCUBT; i++) {
    for (int j = 0; j < NUMBC; j++) {
      H_OffCube[i][j]->Write();
    }
  }

  for (int i = 0; i < NUMBT; i++) {
    for (int j = 0; j < NCUBC; j++) {
      H_OffUmb[i][j]->Write();
    }
  }
  
  for (int i = 0; i < NCORT; i++) {
    for (int j = 0; j < NCUBS; j++) {
      H_OffCorN[i][j]->Write();
    }
  }
  
  for (int i = 0; i < NCORT; i++) {
    for (int j = 0; j < NCUBS; j++) {
      H_OffCorE[i][j]->Write();
    }
  }
  
  for (int i = 0; i < NCORT; i++) {
    for (int j = 0; j < NCUBS; j++) {
      H_OffCorS[i][j]->Write();
    }
  }
  for (int i = 0; i < NCORT; i++) {
    for (int j = 0; j < NCUBS; j++) {
      H_OffCorW[i][j]->Write();
    }
  }
  
  outfile->Close();
}

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
