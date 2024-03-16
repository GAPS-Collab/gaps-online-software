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

  evtno = evt_ctr;
  
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
  }

  // Reset everything that is stored by Paddle number (1-160)
  for (int i=0; i<NPAD; i++) {
    Hits[i]   = -999;
    HitX[i]   = -999;
    HitY[i]   = -999;
    HitZ[i]   = -999;
  }

  // Reset everything that is stored by event
  NPadCube  = 0;
  NPadUpper = 0;
  NPadLower = 0;
  NPadOuter = 0;
       
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
  for (int i=0; i<NRB; i++) {
    for (int j=0; j<NCH; j++) {
      int ch=(i-1)*NCH + j;  // Determine NTOT position
      RB[ch]     = sipm->RB[ch];
      RB_ch[ch]  = sipm->RB_ch[ch];
      Paddle[ch] = sipm->PaddleID[ch];
      PadEnd[ch] = sipm->PaddleEnd[ch];
    }
  }
  //for (int i=0;i<NTOT;i++)
  //printf("%3d: %2d  %d  %3d  %d\n",i,RB[i],RB_ch[i],Paddle[i],PadEnd[i]);

  
  for (int i=0; i<NPAD; i++) {
    // Store the SiPM Channel for each Paddle end
    Paddle_A[i] = pad->SiPM_A[i]; 
    Paddle_B[i] = pad->SiPM_B[i]; 

    PadVID[i] = pad->VolumeID[i];
    PadO[i]   = pad->Orientation[i];
    PadX[i]   = pad->Location[i][0];
    PadY[i]   = pad->Location[i][1];
    PadZ[i]   = pad->Location[i][2];
    
    //printf("  Pad %d: %d  %8.2f %8.2f %8.2f %2d\n", i, PadVID[i],
    //	   PadX[i], PadY[i], PadZ[i], PadO[i]);
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
    tDiff[b] = new TH1D(text, "", 400, -100, 100);
    tDiff[b]->GetXaxis()->SetTitle("TDC Difference");
    tDiff[b]->GetYaxis()->SetTitle("Counts");
  }
  // Histograms comparing the charge measured at both ends of the paddle.
  //TH2D *QEnd2End[l->n_chan/2];
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

  // Hit position along paddle
  for (int b = 0; b < NPAD; b++) {
    sprintf(text, "HitPosition[%d]", b);
    HitPosition[b] = new TH1F(text, "", 190, -95.0, 95.0);
    HitPosition[b]->GetXaxis()->SetTitle("Position");
    HitPosition[b]->GetYaxis()->SetTitle("Counts");
  }

  // Hit position in GAPS volume
  HitGAPS = new TH3F("HitGAPS", "", 180, -1800.0, 1800.0,
		     180, -1800.0, 1800.0,
		     110, 0.0, 2200.0 );
  HitGAPS->GetXaxis()->SetTitle("X Position");
  HitGAPS->GetYaxis()->SetTitle("Y Position");

  // Hit position in GAPS volume
  HitCube = new TH3F("HitCube", "", 180, -1800.0, 1800.0,
		     180, -1800.0, 1800.0,
		     110, 0.0, 2200.0 );
  HitCube->GetXaxis()->SetTitle("X Position");
  HitCube->GetYaxis()->SetTitle("Y Position");

  // Hit position in GAPS volume
  HitCortina = new TH3F("HitCortina", "", 180, -1800.0, 1800.0,
		     180, -1800.0, 1800.0,
		     110, 0.0, 2200.0 );
  HitCortina->GetXaxis()->SetTitle("X Position");
  HitCortina->GetYaxis()->SetTitle("Y Position");

  // Hit position in GAPS volume
  HitUmbrella = new TH3F("HitUmbrella", "", 180, -1800.0, 1800.0,
		     180, -1800.0, 1800.0,
		     110, 0.0, 2200.0 );
  HitUmbrella->GetXaxis()->SetTitle("X Position");
  HitUmbrella->GetYaxis()->SetTitle("Y Position");

  // Average Charge vs position along paddle
  for (int b = 0; b < NPAD; b++) {
    sprintf(text, "QvPosition[%d]", b);
    QvPosition[b] = new TProfile(text, "", 190, -95.0, 95.0);
    QvPosition[b]->GetXaxis()->SetTitle("Position");
    QvPosition[b]->GetYaxis()->SetTitle("Avg Charge");
    QvPosition[b]->SetMinimum(0);
    QvPosition[b]->SetMaximum(70);
    //QvPosition[b]->SetStats(false);
  }

  //rao  number of paddles hit upper, lower and outer
  NPaddlesCube = new TH1I("NPaddles Hit Cube", "", 12, -1.5, 10.5);
  NPaddlesCube->GetXaxis()->SetTitle("NPaddes Hit Cube");
  NPaddlesCube->GetYaxis()->SetTitle("Counts");
  
  NPaddlesUpper = new TH1I("NPaddles Hit Upper", "", 12, -1.5, 10.5);
  NPaddlesUpper->GetXaxis()->SetTitle("NPaddes Hit Upper");
  NPaddlesUpper->GetYaxis()->SetTitle("Counts");
  
  NPaddlesLower = new TH1I("NPaddles Hit Lower", "", 12, -1.5, 10.5);
  NPaddlesLower->GetXaxis()->SetTitle("NPaddes Hit Lower");
  NPaddlesLower->GetYaxis()->SetTitle("Counts");
  
  NPaddlesOuter = new TH1I("NPaddles Hit Outer", "", 12, -1.5, 10.5);
  NPaddlesOuter->GetXaxis()->SetTitle("NPaddes Hit Outer");
  NPaddlesOuter->GetYaxis()->SetTitle("Counts");

}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::WriteHistograms() {
  
  TFile *outfile = TFile::Open("/home/gaps/zweerink/outfile.root","RECREATE"); 
  
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
  
  //write all the Trace plots to the root file
  //Peddir->cd();
  for (int i = 0; i < NTOT; i++) {
    pedHist[i]->Write();
    pedRMSHist[i]->Write();
  }
  
  //Peakdir->cd();
  for (int i = 0; i < NTOT; i++) Peak[i]->Write();
  
  //Chargedir->cd();
  for (int i = 0; i < NTOT; i++) {
    Charge[i]->Write();
    Charge_cut[i]->Write();
  }
  for (int j = 0; j < NPAD; j++) QEnd2End[j]->Write();
  HitGAPS->Write();
  HitCube->Write();
  HitCortina->Write();
  HitUmbrella->Write();
  for (int j = 0; j < NPAD; j++) HitPosition[j]->Write();
  for (int j = 0; j < NPAD; j++) QvPosition[j]->Write();
  
  //TDCdir->cd();
  for (int i = 0; i < NTOT; i++) tdcCFD[i]->Write();
  for (int j = 0; j < NPAD; j++) tDiff[j]->Write();
  
  //Hitmaskdir->cd();
  NPaddlesUpper->Write();
  NPaddlesLower->Write();
  NPaddlesOuter->Write();
  for (int j = 0; j < NPAD; j++) HitMask[j]->Write();
  
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
      // Find the charge
      QInt[i]  = wData[i]->Integrate(Pulse_low, Pulse_win);
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
void EventGAPS::AnalyzePaddles(float pk_cut = -999, float ch_cut = -999.0) {
  // Assuming previous calls to AnalyzePedestals and AnalyzePulses,
  // you have access to Pedestals, charges, Peaks, TDCs etc so
  // calculate quantities related to paddles.

  float Vpeak_cut  = 10.0;
  float Charge_cut =  5.0;
  int ahit, bhit;
  float sspeed = 154.0; // in mm/s
  
  int cube=0, upper=0, outer=0, lower=0;

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
      float delta_pos = tdc_diff*sspeed/2.0;
      // Now, use position and orientation of paddle to find hit location
      int sign = (PadO[i] > 0 ? 1 : -1); 
      int orient = ABS(PadO[i]);
      delta[i] = delta_pos/10.0;
      HitX[i] = PadX[i];
      HitY[i] = PadY[i];
      HitZ[i] = PadZ[i];
      if (orient == 1) HitX[i] += sign*delta_pos; 
      if (orient == 2) HitY[i] += sign*delta_pos;
      if (orient == 3) HitZ[i] += sign*delta_pos;
      if (TDC[chA]>100 && TDC[chA]<200 && TDC[chB]>100 && TDC[chB]<200) 
      //if (TDC[chA]>95 && TDC[chB]>95) 
	IsHit[i] = true;
    }
    
    if (Hits[i]>0) {
      // First determine if paddle is in the cube
      if (i>10 && i<26) cube++;
      if (i<40) upper++;
      else if (i>120) lower++;
      else outer++;
    }
  }

  NPadCube    = cube;
  NPadUpper   = upper;
  NPadLower   = lower;
  NPadOuter   = outer;
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::AnalyzeEvent(void) {
  // Assuming previous calls to AnalyzePedestals and AnalyzePulses,
  // you have access to Pedestals, charges, Peaks, TDCs etc so
  // calculate any interesting quantities here.
  
  
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
void EventGAPS::FillChannelHistos(void) {
  
  for (int i=0; i<NTOT; i++) {
    pedHist[i]->Fill(Pedestal[i]);
    pedRMSHist[i]->Fill(PedRMS[i]);
    Peak[i]->Fill(VPeak[i]);
    Charge[i]->Fill(QInt[i]);
    if (QInt[i]>5.0) Charge_cut[i]->Fill(QInt[i]);

    tdcCFD[i]->Fill(TDC[i]);
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::FillPaddleHistos(void) {
  
  for (int i=0; i<NPAD; i++) {
    if (Paddle_A[i] > 0) { // Paddle-channel map exists
      QEnd2End[i]->Fill(QInt[Paddle_A[i]], QInt[Paddle_B[i]]);
      HitMask[i]->Fill(Hits[i]);
      if ( TDC[Paddle_A[i]] > 0 && TDC[Paddle_B[i]] > 0 )
	tDiff[i]->Fill(TDC[Paddle_A[i]] - TDC[Paddle_B[i]]);
      if (IsHit[i]) { // Both ends of paddle hit
	HitPosition[i]->Fill(delta[i]);
	HitGAPS->Fill(HitX[i], HitY[i], HitZ[i]);
	if (i<61) HitCube->Fill(HitX[i], HitY[i], HitZ[i]);
	else if (i<109) HitUmbrella->Fill(HitX[i], HitY[i], HitZ[i]);
	else if (i<161) HitCortina->Fill(HitX[i], HitY[i], HitZ[i]);
	float q_ave = (QInt[Paddle_A[i]] + QInt[Paddle_B[i]]) / 2.0;
	QvPosition[i]->Fill(delta[i], q_ave);
      }
    }
  }
    
  NPaddlesCube->Fill(NPadCube);
  NPaddlesUpper->Fill(NPadUpper);
  NPaddlesLower->Fill(NPadLower);
  NPaddlesOuter->Fill(NPadOuter);
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

//==================END OF PEAK STUFF======================

void EventGAPS::Message(const char *s) {
  //cerr << s << endl;
}

