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
  
  // Initialize some variables
  InitializeVariables(ch);

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
    delete wData[i];
    wData[i]  = NULL;
  }
  for (int i=0; i<NRB;  i++) {
    delete wData[i];
    wClock[i] = NULL;
  }
  
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////



////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::InitializeVariables(int no_acq) {
  
  // stuff related to the peaks

}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

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
    tdcCFD[b] = new TH1D(text, "", 400, 250.0, 350.0);
    tdcCFD[b]->GetXaxis()->SetTitle("Pulse Time (ns)");
    tdcCFD[b]->GetYaxis()->SetTitle("Counts");
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
    HitMask[b] = new TH1D(text, "", 10, -2.5, 7.5);
    HitMask[b]->GetXaxis()->SetTitle("Hit Mask (A=1,B=2)");
    HitMask[b]->GetYaxis()->SetTitle("Counts");
  }

  //rao  number of paddles hit upper, lower and outer
  NPaddlesUpper = new TH1D("NPaddles Hit Upper", "", 12, -1.5, 10.5);
  NPaddlesUpper->GetXaxis()->SetTitle("NPaddes Hit Upper");
  NPaddlesUpper->GetYaxis()->SetTitle("Counts");
  
  NPaddlesLower = new TH1D("NPaddles Hit Lower", "", 12, -1.5, 10.5);
  NPaddlesLower->GetXaxis()->SetTitle("NPaddes Hit Lower");
  NPaddlesLower->GetYaxis()->SetTitle("Counts");
  
  NPaddlesOuter = new TH1D("NPaddles Hit Outer", "", 12, -1.5, 10.5);
  NPaddlesOuter->GetXaxis()->SetTitle("NPaddes Hit Outer");
  NPaddlesOuter->GetYaxis()->SetTitle("Counts");

}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::WriteHistograms() {
  
  TFile *outfile = TFile::Open("/home/gaps/zweerink/anevis.root", "RECREATE"); 
  
  // For reasons I don't understand, the code to make subdirectories
  // is not compiling properly and gives an error (below) when run
  
  //./analyzeNevis: symbol lookup error: ./analyzeNevis: undefined
  //symbol: _ZN10TDirectory30GetSharedLocalCurrentDirectoryEv
  
  // For now, I am simply writing all the plots to the main directory.
  
  //create directories for the raw plots
  /*TDirectory *savdir = gDirectory;
    TDirectory *Peddir = savdir->mkdir("Pedestals");
    TDirectory *Peakdir = savdir->mkdir("VPeakplots");
    TDirectory *Chargedir = savdir->mkdir("Chargeplots");
    TDirectory *Hitmaskdir = savdir->mkdir("Hitmasks");
    
    TDirectory *TDCdir = savdir->mkdir("TDCplots");
  */
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
  
  //TDCdir->cd();
  for (int i = 0; i < NTOT; i++) tdcCFD[i]->Write();
  
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
      PedRMS[i] = wData[i]->GetPedsigma();

      //if ( PedRMS[i] > 15 ) printf("Channel %d: %8.1f\n", i, PedRMS[i]);
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
      wData[i]->SetThreshold(Threshold);
      wData[i]->SetCFDSFraction(CFDFraction);
      VPeak[i] = wData[i]->GetPeakValue(Pulse_low, Pulse_win);
      QInt[i]  = wData[i]->Integrate(Pulse_low, Pulse_win);
      wData[i]->FindPeaks(Pulse_low, Pulse_win);
      //if ( (wData[i]->GetNumPeaks() > 0) && (Qint[i] > 5.0) ) {
      if ( (wData[i]->GetNumPeaks() > 0) ) {
	wData[i]->FindTdc(0, GAPS::CFD_SIMPLE);     // Simple CFD
	TDC[i] = wData[i]->GetTdcs(0);
	printf("ch %3d: %.2f -- %.2f, %.2f\n", i, TDC[i], VPeak[i], QInt[i]);
      }
    }
  }
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
    if (QInt[i]>0.5) Charge_cut[i]->Fill(QInt[i]);

    tdcCFD[i]->Fill(TDC[i]);
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void EventGAPS::FillPaddleHistos(void) {
  
  float Vpeak_cut = 10.0;
  int ahit, bhit, hmask;

  int upper=0, outer=0, lower=0;
  
  for (int i=0; i<NPAD; i++) {
    QEnd2End[i]->Fill(QInt[i*2], QInt[i*2+1]);

    ahit = (VPeak[i*2] > Vpeak_cut) ? 1 : 0 ;   
    bhit = (VPeak[i*2+1] > Vpeak_cut) ? 2 : 0 ;   
    hmask = ahit + bhit;
    HitMask[i]->Fill(hmask);

    if (hmask>0) {
      if (i<40) upper++;
      else if (i>120) lower++;
      else outer++;
    }
  }
  NPaddlesUpper->Fill(upper);
  NPaddlesLower->Fill(lower);
  NPaddlesOuter->Fill(outer);
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

//==================END OF PEAK STUFF======================

void EventGAPS::Message(const char *s) {
  //cerr << s << endl;
}

