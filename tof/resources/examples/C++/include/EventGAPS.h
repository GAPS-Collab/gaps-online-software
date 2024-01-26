/* ROOT Stuff for plotting traces */
#include <TGraph.h>
#include <TGraphErrors.h>
#include <TGraphAsymmErrors.h>
#include <TH1F.h>
#include <TH2F.h>
#include <TROOT.h>

#include <legacy.h>

#ifndef EVENTGAPS
#define EVENTGAPS

#define ERRVAL		(999999999)

// Types of Thresholds to use in determining timing
enum THRTYPE { CONSTANT, CFD_ELEC, CFD_SIMPLE, PCONSTANT, PCFD };

//double Pulse(double *x, double *par);

class EventGAPS {

public:

  EventGAPS (GAPS::Waveform *wave[], GAPS::Waveform *wch9[]);
  // Constructor 'flag' is set by default unless the waveform is
  // constructed with a call to the contrary.  Thus, the default
  // behavior is to provide immediate access to pulse positions,
  // times, heights, etc....  For someone who wants to do a more
  // specialized analysis, the constructor can be called with flag=0
  // so that we do not waste time doing the ped and peak calculations.

  EventGAPS (int size);
  ~EventGAPS (void);

  // MEMBER FUNCTIONS

  // Stuff related to the actual data
  void    SetThreshold(float PmtThreshold);
  //int     GetWaveSize(void){return wf_size;}

private:

  // DATA MEMBERS

  int     ch;                        // STACEE channel we are working with
  int     runno;                     // Run Number
  float   Threshold;                 // PMT Threshold in DC (for now...)

  double  wf_pedestal;               // Pedestal value
  int     *peaks;          // Bin values of the actual peak positions

  // MEMBER FUNCTIONS
  void    InitializeVariables(int no_acq);
  void    Message(const char *s);           // Print out messages as needed
  // Stuff related to the peaks
};

#endif
