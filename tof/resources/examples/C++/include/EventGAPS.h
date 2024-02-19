/* ROOT Stuff for plotting traces */
#include <TGraph.h>
#include <TGraphErrors.h>
#include <TGraphAsymmErrors.h>
#include <TH1F.h>
#include <TH2F.h>
#include <TROOT.h>

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

  void    InitializeVariables(void);
  void    InitializeWaveforms(GAPS::Waveform *wave[], GAPS::Waveform *wch9[]);
  void    UnsetWaveforms(void);
  void    SetPaddleMap(int paddle_map[NRB][NCH], int pad2volid[NPAD],
		       int padvid[NPAD], float padLocation[NPAD][3]);
  
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
  float   Threshold;                 // PMT Threshold in DC (for now...)
  float   CFDFraction;               // CFD Fraction for TDC calculation

  // Since paddles start at 1, we include one extra value
  int     Paddle_A[NPAD];            // Channel for this PadddleA
  int     Paddle_B[NPAD];            // Channel for this PadddleB
  int     ChnlMap[NRB][NCH];         // Maps SiPM channel to Paddle
  int     PadVID[NPAD];              // Volume ID
  float   PadX[NPAD];                // X detector location
  float   PadY[NPAD];                // Y detector location
  float   PadZ[NPAD];                // Z detector location

  
  float   Pedestal[NTOT];             // Pedestal values
  float   PedRMS[NTOT];               // Pedestal RMS values
  float   ClockPedestal[NRB];         // Pedestal values
  float   ClockPedRMS[NRB];           // Pedestal RMS values
 
  float   VPeak[NTOT];                // Pulse peak value
  float   QInt[NTOT];                 // Pulse charge value
  float   TDC[NTOT];                  // TDC value (CFD method)

  int     Hits[NPAD];                 // Hit mask for paddle 
  float   HitX[NPAD];                 // X location in detector
  float   HitY[NPAD];                 // Y location in detector
  float   HitZ[NPAD];                 // Z location in detector
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
  TH1D    *HitMask[NPAD];              // Hit mask of paddle
  TH1D    *NPaddlesCube;
  TH1D    *NPaddlesUpper;
  TH1D    *NPaddlesLower;
  TH1D    *NPaddlesOuter;
  
  // MEMBER FUNCTIONS
  void    Message(const char *s);           // Print out messages as needed
  // Stuff related to the peaks
};

#endif
