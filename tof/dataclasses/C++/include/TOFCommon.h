#ifndef TOFCOMMON
#define TOFCOMMON

#include <stdint.h>
#include <vector>
#include <tof_typedefs.h>

// This include file defines anything that is needed by multiple threads. 


// Set to one larger than the actual number until we start using RB0
#define MAX_BRDS     40    
#define NCHN         9
#define CHNTOT       MAX_BRDS * NCHN
// To calculate NPADDLE, remember that CH9 on each RB is not SiPM data
#define NPADDLE      (MAX_BRDS * (NCHN - 1))/2 
#define NWORDS       1024


namespace GAPS {

  enum class PADDLE_END : u16 {
      A = 10,
      B = 20,
      UNKNOWN = 30
  };

}




// These quantities relate to the current run conditions
// The only thread that should write to these quantities is RBCommunication
typedef struct RUN_DATA
{
  // How many RBs are sending data
  int nBrdsExpected;
  int BrdFound[MAX_BRDS];
  // Stuff needed to find/fill the UTC for each event
  bool firstBlob;
  unsigned long firstEvent;
  unsigned long long RefTimestamp[MAX_BRDS];
  unsigned long UTCEvtID;
  unsigned long UTCSecs;
  unsigned long UTCMSecs;
} RunData_t;


// These quantities are read in from a blob
typedef struct BLOB_DATA
{
  unsigned short head; // Head of event marker
  unsigned short status;
  unsigned short len;
  unsigned short roi;
  unsigned long long dna;
  unsigned short fw_hash;
  unsigned short id;
  unsigned short ch_mask;
  unsigned long event_ctr;
  unsigned short dtap0;
  unsigned short dtap1;
  unsigned long long timestamp;
  unsigned short ch_head[NCHN];
  short ch_adc[NCHN][NWORDS];
  unsigned long ch_trail[NCHN];
  unsigned short stop_cell;
  unsigned long crc32;
  unsigned short tail; // End of event marker
} BlobEvt_t ;  



typedef struct BRUN_DATA {
  unsigned int   RunNum;
  unsigned int   RunStartTime;
  unsigned int   FileOpenTime;
} brun_t;

typedef struct ERUN_DATA {
  unsigned int   RunNum;
  unsigned int   FileCloseTime;
} erun_t;

#endif
