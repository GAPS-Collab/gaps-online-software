#ifndef BLOBROUTINES_H_INCLUDED
#define BLOBROUTINES_H_INCLUDED

// Header file for blobroutines

//-------------------------------------------------------------------------
// Constants                                                                
//-------------------------------------------------------------------------
// We use a #define statement, but NCHN must be 8 due to a number of
// hard-coded features below. It does make coding the for-loops easier
// These are now defined in TOFCommon.h
//#define MAX_BRDS 6       // How many Readout Boards
//#define NCHN 8           // How many ADC channels per board
//#define NWORDS 1024      // How many Words per channel

#include <TOFCommon.h>
#include <bitset>

#define FIRSTNUM 200

//---------------------------------------------------------------------------
// Prototypes
//---------------------------------------------------------------------------

void FillFirstEvents(FILE *fp, int board, struct Times_t *times );
void FindFirstEvents(struct Times_t *times);
void FindUTCReference(RunData_t *RunInfo); 
void FindUTCReference(struct Times_t *times); 
int  HaveEvents(int status[], int nbrds);
void BoardsInEvent(int status[], int evt_ID[], int inevent[], int nbrds);
void BoardsInEventTime(int status[], unsigned long long time[], int inevent[],
       int nbrds);
int  ReadEvent(FILE *fp, BlobEvt_t *evt, bool Print=false);
void VoltageCalibration(short traceIn[], double traceOut[],
                    unsigned int tCell, struct Calibrations_t cal);
void VoltageNonCalibration(short traceIn[], double traceOut[]);
void TimingCalibration(double times[],
                    unsigned int tCell, struct Calibrations_t cal);
void RemoveSpikes(double wf[NCHN][1024], unsigned int tCell, int spikes[]);
//void RemoveSpikes(short int wf[NCHN][1024], unsigned int tCell, int spikes[]);
unsigned long long Decode64(unsigned long long tb[]);
unsigned long long Decode48(unsigned long long tb[]);
unsigned long Decode32(unsigned long sb[]);
int  nthbit(unsigned long long number, int n);
void printbinary(unsigned long long number, int bit);

struct Times_t
{
  int nbrds;
  unsigned long long time[MAX_BRDS][FIRSTNUM]; // First XX event times
  unsigned long evt_ctr[MAX_BRDS][FIRSTNUM];   // First XX event ctrs
  int first_evt[MAX_BRDS];                     // First common event
  int first_evt_ID[MAX_BRDS];                  // ID of first common event
  unsigned long evtid_ref;
  double utc_ref;
  unsigned long long time_ref[MAX_BRDS];
  bool common;
};

struct Calibrations_t
{
  double vofs[NWORDS]; // voltage offset
  double vdip[NWORDS]; // voltage "dip" (time-dependent correction)
  double vinc[NWORDS]; // voltage increment (mV/ADC unit)
  double tbin[NWORDS]; // cell width (ns)
};

#endif
