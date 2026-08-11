#ifndef EVENTGAPS
#define EVENTGAPS
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

#define ERRVAL		(999999999)

// Just some variables that we will use later. Should make more robust
// Need 1 larger to account for no "zero" cube paddle
#define NCUBT 61  
#define NUMBT 48  
#define NCUBC 12  
#define NUMBC 12
#define NCUBS  8
#define NCORT 10

// Types of Thresholds to use in determining timing
enum THRTYPE { CONSTANT, CFD_ELEC, CFD_SIMPLE, PCONSTANT, PCFD };

//double Pulse(double *x, double *par);

//namespace Gaps = GAPS;

class EventGAPS {

public:
  
  EventGAPS (void);
  
  ~EventGAPS (void);
  
  // MEMBER FUNCTIONS
  
  void    InitializeVariables(int run_no, unsigned long int evt_ctr);
  void    InitializeWaveforms(GAPS::Waveform *wave[], GAPS::Waveform *wch9[]);
  void    UnsetWaveforms(void);
  void    SetPaddleMap(int paddle_map[NRB][NCH], int pad2volid[NPAD],
		       int padvid[NPAD], float padLocation[NPAD][4]);
  void    SetPaddleMap(struct PaddleInfo *pad, struct SiPMInfo *sipm);
  
  // Stuff related to the actual data
  void    FillEventValues(struct EventInfo *evt);
  void    FixPedRMSValues(void);
  void    AnalyzePedestals(float Ped_begin, float Ped_win);
  void    SetThreshold(float PmtThreshold);
  void    SetCFDFraction(float CFDS_frac);
  void    AnalyzePulses(float Pulse_low, float Pulse_win);
  void    AnalyzePhases(float phi[NRB]);
  void    AnalyzePaddles(float pk_cut, float ch_cut);
  void    AnalyzeEvent(void);
  
  float   GetTDC(int ch) {return TDC[ch];}
  
  // Stuff related to plotting
  void    InitializeHistograms(void);
  void    FillChannelHistos(int old);
  void    FillPaddleHistos(void);
  void    WriteHistograms(void);

  // Stuff related to calculating paddle offsets
  void    OffsetHistograms(bool flag);
  void    FillOffsetHistos(void);
  void    WriteOffsetHistos(void);
  
private:
  
  // DATA MEMBERS
  
  // Local pointers to waveforms
  GAPS::Waveform  *wData[NTOT];
  GAPS::Waveform  *wClock[NRB];       
  
  int     ch;                        // channel we are working with
  int     runno;                     // Run Number
  unsigned long int  evtno;          // Event Number
  float   sc_speed;                  // Speed(mm/s) in scintillator
  
  float   Threshold;                 // PMT Threshold in DC (for now...)
  float   CFDFraction;               // CFD Fraction for TDC calculation
  
  // SiPM channel info (index references NTOT value)
  int     max_sipm;                 // Largest SiPM channel
  int     RB[NTOT];
  int     RB_ch[NTOT];
  int     Paddle[NTOT];
  int     PadEnd[NTOT];
  
  // Since paddles start at 1, we include one extra value
  int     max_paddle;                // Largest paddle ID
  int     Paddle_A[NPAD];            // Channel for this PadddleA
  int     Paddle_B[NPAD];            // Channel for this PadddleB
  int     PadVID[NPAD];              // Volume ID
  float   PadX[NPAD];                // X detector location
  float   PadY[NPAD];                // Y detector location
  float   PadZ[NPAD];                // Z detector location
  int     PadO[NPAD];                // Orientation of Paddle
  float   Dimension[NPAD][3];        // Dimension of paddle (LxWxH)
  float   TCorrFixed[NPAD];          // Timing correction (cables, pad_len)
  float   TCorrEvent[NPAD];          // Timing correction (ch9)
  float   Offset[NPAD];              // Offset of this paddle
  bool    CalcOffset;                // Set true if calculating offsets
  int     EarlyPaddle;               // Which paddle is first hit
  float   EarlyTime;                 // Time of earliest hit
  
  double  FlightTime;                 // Since flight start, in seconds
  float   Pedestal[NTOT];             // Pedestal values
  float   PedRMS[NTOT];               // Pedestal RMS values
  bool    PedRMSFix;                  // PedRMS Fix applied?
  bool    RBInData[NRB];              // RB in data stream?
  float   ClockPedestal[NRB];         // Pedestal values
  float   ClockPedRMS[NRB];           // Pedestal RMS values
  float   Phi[NRB];                   // Phase of ch9 data
  float   TShift[NRB];                // Calculated timing shift
  
  float   VPeak[NTOT];                // Pulse peak value
  float   QInt[NTOT];                 // Pulse charge value
  float   TDC[NTOT];                  // TDC value (CFD method)
  float   TDC_Cor[NTOT];              // Corrected TDC value (CFD method)

  float   TotLo[NTOT];                // TOT values (lo threshold)
  float   TotHi[NTOT];                // TOT values (hi threshold)
  
  bool    IsHit[NPAD];                // Do we have Hit info?
  int     Hits[NPAD];                 // Hit mask for paddle 
  float   HitX[NPAD];                 // X location in detector
  float   HitY[NPAD];                 // Y location in detector
  float   HitZ[NPAD];                 // Z location in detector
  float   HitT[NPAD];                 // Time of hit in detector
  float   delta[NPAD];                // displacement from center
  float   beta;                       // beta of particle
  int     NPadCube;
  int     NPadUmbrella;
  int     NPadCortina;
  
  // Though we store the various values below by NTOT (since that is
  // the easiest way to index through the quantities, we want to write
  // them to histograms that are stored by NPAD.
  TH1D    *pedHist[NPAD][2];           // Pedestal histograms
  TH1D    *pedRMSHist[NPAD][2];        // Pedestal RMS histograms
  TH1D    *Peak[NPAD][2];              // VPeak histograms
  TH1D    *Charge[NPAD][2];            // Charge histograms
  TH1D    *Charge_cut[NPAD][2];        // Charge (cut) histograms
  TH1D    *tdcCFD[NPAD][2];            // TDC histograms
  TH1D    *totLo[NPAD][2];             // TOT (lo) histograms
  TH1D    *totHi[NPAD][2];             // TOT (hi) histograms
  TProfile *pedVtime[NPAD][2];         // Pedestal profile
  TProfile *pRMSVtime[NPAD][2];        // Pedestal RMS profile
  TProfile *peakVtime[NPAD][2];        // peak time profile
  TProfile *chVtime[NPAD][2];          // charge time profile
  TProfile *tCFDVtime[NPAD][2];        // tCFD time profile
  TProfile *totLVtime[NPAD][2];        // totLo time profile
  TProfile *totHVtime[NPAD][2];        // totHi time profile

  /*TH1D    *pedHist[NTOT];              // Pedestal histograms
  TH1D    *pedRMSHist[NTOT];           // Pedestal RMS histograms
  TProfile *pedVtime[NTOT];            // Pedestal profile
  TProfile *pRMSVtime[NTOT];           // Pedestal RMS profile
  TH1D    *Peak[NTOT];                 // VPeak histograms
  TH1D    *Charge[NTOT];               // Charge histograms
  TH1D    *Charge_cut[NTOT];           // Charge (cut) histograms
  TH1D    *tdcCFD[NTOT];               // TDC histograms
  TH1D    *totLo[NTOT];                // TOT (lo) histograms
  TH1D    *totHi[NTOT];                // TOT (hi) histograms
  */
  
  TH2D    *QEnd2End[NPAD];             // End 2 End charge 
  TH1I    *HitMask[NPAD];              // Hit mask of paddle
  TH1D    *tDiff[NPAD];                // tdc diff of paddle ends
  TH1D    *Ch9Shift[NPAD];             // T shift from ch9 analysis
  TH2F    *Ch9Good[2];                 // T shift from ch9 analysis
  TH2F    *Ch9Bad[2];                  // T shift from ch9 analysis
  TH1F    *HitTime[NPAD];              // Hit Time in detector
  TH1F    *HitPosition[NPAD];          // Hit Position along paddle (cm)
  TH3F    *HitGAPS;                    // Hit Position in detector (mm)
  TH3F    *HitCube;                    // Hit Position in detector (mm)
  TH3F    *HitCortina;                 // Hit Position in detector (mm)
  TH3F    *HitUmbrella;                // Hit Position in detector (mm)
  TProfile *QvPosition[NPAD];          // Avg Q vs position along paddle
  TProfile *QvPositionA[NPAD];         // Q vs position - End A
  TProfile *QvPositionB[NPAD];         // Q vs position - End B
  TH1I    *FirstPaddle;
  TH1F    *FirstTime;
  TH1F    *BetaDist;
  TH1I    *NPaddlesCube;
  TH1I    *NPaddlesUmbrella;
  TH1I    *NPaddlesCortina;

  TH1F    *H_OffCube[NCUBT][NUMBC]; // 60 Cube, 12 Umb-Center paddles
  TH1F    *H_OffUmb[NUMBT][NCUBC];  // 48 Umb, 12 Cube-Center(Top) paddles
  TH1F    *H_OffCorN[NCORT][NCUBS]; // 8 Cube, 10 Cortina paddles
  TH1F    *H_OffCorE[NCORT][NCUBS]; // 8 Cube, 10 Cortina paddles
  TH1F    *H_OffCorS[NCORT][NCUBS]; // 8 Cube, 10 Cortina paddles
  TH1F    *H_OffCorW[NCORT][NCUBS]; // 8 Cube, 10 Cortina paddles
  
  // MEMBER FUNCTIONS
};

#endif
