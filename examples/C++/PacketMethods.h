#ifndef PACKETMETHODS
#define PACKETMETHODS

#include <legacy.h>
#include "constants.h"
#include "events.h"
#include "telemetry_dataclasses.hpp"


#define ERRVAL		(999999999)

using namespace GAPS;

class PacketMethods {

public:
  
  PacketMethods (void);
  
  ~PacketMethods (void);
  
  EventGAPS  Event;

  // MEMBER FUNCTIONS
  
  //void    InitializeVariables(unsigned long int evt_ctr);
  void    InitializeVariables(void);
  void    BeginRun(int run);
  void    EndRun(void);
  //void    SetPaddleMap(struct PaddleInfo *pad, struct SiPMInfo *sipm);
  void    NothingYet(void);
  void    InitPaddleInfo(void);
  void    GetPaddleInfo(void);
  void    ProcessTofEventSummary(TofEventSummary *Tes, unsigned long int);
  
  // Stuff related to the actual data
  /*
  void    FillEventValues(struct EventInfo *evt);
  void    AnalyzePedestals(float Ped_begin, float Ped_win);
  void    SetThreshold(float PmtThreshold);
  void    SetCFDFraction(float CFDS_frac);
  void    AnalyzePulses(float Pulse_low, float Pulse_win);
  void    AnalyzePhases(float phi[NRB]);
  void    AnalyzePaddles(float pk_cut, float ch_cut);
  void    AnalyzeEvent(void);
  */
  
private:
  
  // DATA MEMBERS
  
  int     ch;                        // channel we are working with
  int     runno;                     // Run Number
  unsigned long int  evtno;          // Event Number
  
  struct PaddleInfo PadInfo;
  struct SiPMInfo   SipmInfo;
  
  // MEMBER FUNCTIONS
};

#endif
