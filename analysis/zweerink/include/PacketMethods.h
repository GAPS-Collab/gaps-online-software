#ifndef PACKETMETHODS
#define PACKETMETHODS

#include <legacy.h>
#include "constants.h"
#include "events.h"
#include "telemetry_dataclasses.hpp"


#define ERRVAL		(999999999)

using namespace GAPS;
namespace g = gondola;

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
  void    ProcessTofEventSummary(g::TofEventSummary *Tes, unsigned long int);
  void    ProcessTofEvent(g::TofEvent *Tev, std::map<u8,g::RBCalibration>& cali);
  
private:
  
  // DATA MEMBERS
  
  int     ch;                        // channel we are working with
  int     runno;                     // Run Number
  unsigned long int  evtno;          // Event Number
  unsigned long int  timeInit;       // Oscillator value of first Event
  
  struct PaddleInfo PadInfo;
  struct SiPMInfo   SipmInfo;
  
  // MEMBER FUNCTIONS
};

#endif
