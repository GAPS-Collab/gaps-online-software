/* ROOT Stuff for plotting traces */
#include <TDirectory.h>
#include <TGraph.h>
#include <TGraphErrors.h>
#include <TGraphAsymmErrors.h>
#include <TH1F.h>
#include <TH2F.h>
#include <TH3F.h>
#include <TROOT.h>
#include <TProfile.h>

#include <legacy.h>
#include "./constants.h"

#ifndef EVENTGAPS
#define EVENTGAPS

#define ERRVAL		(999999999)

// Types of Thresholds to use in determining timing
enum THRTYPE { CONSTANT, CFD_ELEC, CFD_SIMPLE, PCONSTANT, PCFD };

//double Pulse(double *x, double *par);

class EventGAPS {

public:

  EventGAPS (void);

  ~EventGAPS (void);

  // MEMBER FUNCTIONS

  void    InitializeVariables(unsigned long int evt_ctr);
  void    InitializeWaveforms(GAPS::Waveform *wave[], GAPS::Waveform *wch9[]);
  void    UnsetWaveforms(void);
  void    SetPaddleMap(int paddle_map[NRB][NCH], int pad2volid[NPAD],
		       int padvid[NPAD], float padLocation[NPAD][4]);
  void    SetPaddleMap(struct PaddleInfo *pad, struct SiPMInfo *sipm);
  
  // Stuff related to the actual data
  void    AnalyzePedestals(float Ped_begin, float Ped_win);
  void    SetThreshold(float PmtThreshold);
  void    SetCFDFraction(float CFDS_frac);
  void    AnalyzePulses(float Pulse_low, float Pulse_win);
  void    AnalyzePaddles(float pk_cut, float ch_cut);
  void    AnalyzeEvent(void);

  float   GetTDC(int ch) {return TDC[ch];}
  
  // Stuff related to plotting
  void    InitializeHistograms(void);
  void    FillChannelHistos(void);
  void    FillPaddleHistos(void);
  void    WriteHistograms(void);

  
private:

  // DATA MEMBERS

  // Local pointers to waveforms
  GAPS::Waveform  *wData[NTOT];
  GAPS::Waveform  *wClock[NRB];       

  int     ch;                        // channel we are working with
  int     runno;                     // Run Number
  unsigned long int  evtno;          // Event Number
  float   Threshold;                 // PMT Threshold in DC (for now...)
  float   CFDFraction;               // CFD Fraction for TDC calculation

  // SiPM channel info (index references NTOT value)
  int     RB[NTOT];
  int     RB_ch[NTOT];
  int     Paddle[NTOT];
  int     PadEnd[NTOT];

  // Since paddles start at 1, we include one extra value
  int     Paddle_A[NPAD];            // Channel for this PadddleA
  int     Paddle_B[NPAD];            // Channel for this PadddleB
  int     PadVID[NPAD];              // Volume ID
  float   PadX[NPAD];                // X detector location
  float   PadY[NPAD];                // Y detector location
  float   PadZ[NPAD];                // Z detector location
  int     PadO[NPAD];                // Orientation of Paddle
  
  float   Pedestal[NTOT];             // Pedestal values
  float   PedRMS[NTOT];               // Pedestal RMS values
  float   ClockPedestal[NRB];         // Pedestal values
  float   ClockPedRMS[NRB];           // Pedestal RMS values
 
  float   VPeak[NTOT];                // Pulse peak value
  float   QInt[NTOT];                 // Pulse charge value
  float   TDC[NTOT];                  // TDC value (CFD method)

  bool    IsHit[NPAD];                // Do we have Hit info?
  int     Hits[NPAD];                 // Hit mask for paddle 
  float   HitX[NPAD];                 // X location in detector
  float   HitY[NPAD];                 // Y location in detector
  float   HitZ[NPAD];                 // Z location in detector
  float   delta[NPAD];                // displacement from center
  int     NPadCube;
  int     NPadUpper;
  int     NPadLower;
  int     NPadOuter;
  
  
  TH1D    *pedHist[NTOT];              // Pedestal histograms
  TH1D    *pedRMSHist[NTOT];           // Pedestal RMS histograms
  TH1D    *Peak[NTOT];                 // VPeak histograms
  TH1D    *Charge[NTOT];               // Charge histograms
  TH1D    *Charge_cut[NTOT];           // Charge (cut) histograms
  TH1D    *tdcCFD[NTOT];                  // TDC histograms

  TH2D    *QEnd2End[NPAD];             // End 2 End charge 
  TH1I    *HitMask[NPAD];              // Hit mask of paddle
  TH1D    *tDiff[NPAD];                // tdc diff of paddle ends
  TH1F    *HitPosition[NPAD];          // Hit Position along paddle (cm)
  TH3F    *HitGAPS;                    // Hit Position in detector (mm)
  TH3F    *HitCube;                    // Hit Position in detector (mm)
  TH3F    *HitCortina;                 // Hit Position in detector (mm)
  TH3F    *HitUmbrella;                // Hit Position in detector (mm)
  TProfile *QvPosition[NPAD];          // Avg Q vs position along paddle
  TH1I    *NPaddlesCube;
  TH1I    *NPaddlesUpper;
  TH1I    *NPaddlesLower;
  TH1I    *NPaddlesOuter;
  
  // MEMBER FUNCTIONS
  void    Message(const char *s);           // Print out messages as needed
  // Stuff related to the peaks
};

#endif
