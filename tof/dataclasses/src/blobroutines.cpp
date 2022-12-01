#include <fstream>
#include <ctime>
#include <iostream>
#include <cmath>
#include <filesystem>
#include <string.h>
#include <stdio.h>
#include <limits.h>
//#include <TString.h>
#include "blobroutines.h"

void FillFirstEvents(FILE *fp, int brd, struct Times_t *times ) {
  BlobEvt_t temp;

  temp.event_ctr = 1000;
  for (int i=0; i<FIRSTNUM; i++) {
    int stat = ReadEvent(fp, &temp, false); // Get timestamp of first event
    times->time[brd][i]   = temp.timestamp;
    times->evt_ctr[brd][i] = temp.event_ctr;
  }
}

void FindFirstEvents(struct Times_t *times) {

  unsigned long long curr[MAX_BRDS];
  int ctr = 1;

  times->common = false; // No common start event flag
  
  // The code in this for-loop uses time differences to find common events
  /*for (int i=1; i<50; i++) { // Loop through the times of the first board
    curr[0] = times->time[0][i] - times->time[0][i-1];
    for (int j=1; j<times->nbrds; j++) { // Loop through the other boards
      for (int k=1; k<FIRSTNUM; k++) { // and each event for the board
        curr[j] = times->time[j][k] - times->time[j][k-1];
        if ( abs((int)(curr[0]-curr[j])) < FIRSTNUM ) {
          ctr++; // found time that matches.
          //printf("ctr = %d: i=%d, j=%d, k=%d\n", ctr, i, j, k);
          times->first_evt[j] = k-1; 
          k=FIRSTNUM;
        }
      }
      }*/

  int i,j,k,m;
  // The code in this for-loop uses the evt_ctr to find common events
  for (i=1; i<FIRSTNUM; i++) { // Loop through the times of the first board
    for (j=1; j<times->nbrds; j++) { // Loop through the other boards
      for (k=1; k<FIRSTNUM; k++) { // and each event for the board
  if ( times->evt_ctr[0][i] == times->evt_ctr[j][k] ) {
          ctr++; // found time that matches.
          //printf("ctr = %d: i=%d, j=%d, k=%d\n", ctr, i, j, k);
          times->first_evt[j] = k; 
          k=FIRSTNUM;
        }
      }
    }
    
    if (ctr == times->nbrds) { // Found our common event
      times->common = true;
      //times->first_evt[0] = i-1; // If using times
      times->first_evt[0] = i;   // If using evt_ctr
      // Now assign each boards first_evt_ID value
      for (j=0;j<times->nbrds;j++)
	times->first_evt_ID[j] = times->evt_ctr[j][times->first_evt[j]];

      // And print out some info if desired
      //for(m=0;m<times->nbrds;m++) printf("%d(%d) ",m,times->first_evt[m]);
      //printf("\n");
      return;
    } else {
      ctr = 1; // Start with the next time of the first board
    }
  }
  // Did not find a common event. Reset all first_evts to -1 and return
  for (int j=0; j<times->nbrds; j++)
    times->first_evt[j] = -1;
  return;
}
  
void FindUTCReference(RunData_t *RunInfo) { 

  char pname[500];
  unsigned long evt_ref;
  double utc;

  // First, we need to get the UTC of the reference event. Since we
  // already have the evtID, use that

  unsigned long evt = RunInfo->firstEvent;
  // Get reference event UTC from database
  //sprintf(pname, "/home/gaps/bin/evt_id_query %ld", evt);
  //FILE *fp = popen(pname, "r");
  std::cout << "[WARN] - the lookup of event time from the database has been disabled. Using dummy time!" << std::endl;
  std::string get_event_utc = "date '+%s'";
  FILE *fp = popen(get_event_utc.c_str(), "r");
  fscanf(fp, "%ld %lf", &evt_ref, &utc);
  fclose(fp);
  
  // Now decode the output of the program
  if (evt_ref != -1) { // Good output from program
    RunInfo->UTCEvtID = evt_ref;             // Reference ID
    RunInfo->UTCSecs  = (unsigned long)utc;  // Reference UTCSecs
    RunInfo->UTCMSecs = (unsigned long)((utc - RunInfo->UTCSecs)*1e6);
    //printf("%ld: %ld  %lf (%ld : %ld)\n", evt, evt_ref, utc,
    //	   RunInfo->UTCSecs, RunInfo->UTCMSecs);
  }
}

void FindUTCReference(struct Times_t *times) {

  char pname[500];
  unsigned long evt_ref;
  double utc;

  // First, we need to get the UTC of the reference event. Since we
  // have already found the first common event, use that evt_id.
  unsigned long evt;
  if (times->common) { // Have a common event
    // If only one board exists, the first two events are often weird
    if (times->nbrds == 1) evt = times->evt_ctr[0][2]; 
    else evt = times->first_evt_ID[0];
    
    // Get reference event UTC from database
    std::cout << "[WARN] - the lookup of event time from the database has been disabled. Using dummy time!" << std::endl;
    std::string get_event_utc = "date '+%s'";
    FILE *fp = popen(get_event_utc.c_str(), "r");
    //sprintf(pname, "/home/gaps/bin/evt_id_query %ld", evt);
    //FILE *fp = popen(pname, "r");
    fscanf(fp, "%ld %lf", &evt_ref, &utc);
    fclose(fp);
    
    // Now decode the output of the program
    if (evt_ref != -1) { // Good output from program
      times->evtid_ref = evt_ref;      // Reference ID
      times->utc_ref = utc;            // Reference UTC
      for (int i=0;i<times->nbrds;i++) // Reference MHz clock value
	for (int j=0;j<FIRSTNUM;j++) { // Must find the proper event
	  if (times->evt_ctr[i][j] == times->evtid_ref)
	    times->time_ref[i] = times->time[i][j];
	}
      printf("%ld: %ld  %lf  %lld\n", evt, times->evtid_ref,
	     times->utc_ref, times->time_ref[0]);
    }
  }
}

int HaveEvents(int status[], int nbrds) {
  int ctr=0;
  for (int i=0; i<nbrds; i++) {
    ctr += (status[i] > 0 ? 1 : 0) ;
  }
  return (ctr);
}

void BoardsInEvent(int status[], int evt_ID[], int inevent[], int nbrds) {
  int low_evt = INT_MAX;
  int low_brd=-1;

  // First, find the lowest time for all boards
  for (int i=0; i<nbrds; i++) {
    if (status[i]==1) {
      if (evt_ID[i] < low_evt) {
        low_evt = evt_ID[i];
        low_brd = i;
      }
    }
  }

  // Now find all boards with a matching time.
  for (int i=0; i<nbrds; i++) {
    if (status[i]==1) {
      if ( evt_ID[i] == evt_ID[low_brd] )
        inevent[i] = 1;
      else
        inevent[i] = 0;
    }
  }
}

void BoardsInEventTime(int status[], unsigned long long time[], int inevent[],
                 int nbrds) {
  unsigned long long low_time = ULONG_MAX;
  int low_brd=-1;

  // First, find the lowest time for all boards
  for (int i=0; i<nbrds; i++) {
    if (status[i]==1) {
      if (time[i] < low_time) {
        low_time = time[i];
        low_brd = i;
      }
    }
  }

  // Now find all boards with a matching time.
  for (int i=0; i<nbrds; i++) {
    if (status[i]==1) {
      if ( abs((int)(time[i]-time[low_brd])) < 1000)
        inevent[i] = 1;
      else
        inevent[i] = 0;
    }
  }
}

int ReadEvent(FILE *fp, BlobEvt_t *evt, bool Print) {

  //Print = true;
  unsigned short head = 0xaaaa; // Head of event marker
  unsigned short tail = 0x5555; // End of event marker
  
  // Some temporary words for checking data
  unsigned short temp_byte_top, temp_byte_bottom;
  unsigned short temp_short;
  unsigned long long tb[8]; // To read out the longer words (>4bytes)
  unsigned long sb[4]; // To read out the 4-bytes words)

  //bool Print = true;
  
  // reads out 2 byte words one at a time from the stream, assigns
  // to temp_byte_top

  int eof_ck = -1;
  do { // Find the start of the next event
    eof_ck = 
      fread(&temp_byte_top, 2, 1, fp);
    //if (temp_byte_top != 0) 
    //std::cout << "[INFO] - searching for header bytes, found " << temp_byte_top << std::endl;
    //printf("head %x, %x--",temp_byte_top,head);printbinary(temp_byte_top,16);
    if (eof_ck==0)
      { 
          std::cout << "[INFO] <ReadEvent> eof found, returning -1" << std::endl;
          return (-1);
      }
  } while (temp_byte_top != head);
  
  //printf("Decoding event\n");
  evt->head = temp_byte_top;
  // Read in the status bytes
  fread(&evt->status, 2, 1, fp);
  // Read the packet length
  fread(&evt->len, 2, 1, fp);
  // Read the roi
  fread(&evt->roi, 2, 1, fp);
  // Read the dna, make sure we are only getting the proper 8 bits. 
  for (int i=0; i<8; i++) {fread(&tb[i], 1, 1, fp);tb[i] = tb[i] & 0xFF;}
  evt->dna = Decode64(tb);
  // Read the fw_hash
  fread(&evt->fw_hash, 2, 1, fp);
  // Read the id
  fread(&evt->id, 2, 1, fp);
  // Read the ch_mask
  fread(&evt->ch_mask, 2, 1, fp);
  // Read the event_ctr, make sure we are only getting the proper 8 bits. 
  for (int i=0; i<4; i++) {fread(&sb[i], 1, 1, fp); sb[i] = sb[i] & 0xFF;}
  evt->event_ctr = Decode32(sb);
  // Read the dtap0
  fread(&evt->dtap0, 2, 1, fp);
  // Read the dtap1
  fread(&evt->dtap1, 2, 1, fp);
  // Read the timestamp, make sure we are only getting the proper 8 bits. 
  for (int i=0; i<6; i++) {fread(&tb[i], 1, 1, fp);tb[i] = tb[i] & 0xFF;}
  evt->timestamp = Decode48(tb);
  
  // NOW WE READ IN THE ADC DATA
  std::bitset<16> ch_mask = evt->ch_mask;
  int nchan = ch_mask.count();
  for (int i = 0; i < nchan; i++) {
    // Read the channel header
    fread(&evt->ch_head[i], 2, 1, fp);
    // Read the channel data
    for (int j=0; j<NWORDS; j++) {
      fread(&temp_short, 2, 1, fp);
      evt->ch_adc[i][j] = temp_short & 0x3FFF; // Only 14-bit ADCs
    }
    // Read the channel trailer
    for (int k=0; k<4; k++) {fread(&sb[k], 1, 1, fp);sb[k] = sb[k] & 0xFF;}
    evt->ch_trail[i] = Decode32(sb);
  }          
  // Read the stop_cell
  fread(&evt->stop_cell, 2, 1, fp);
  // Read the crc32
  for (int i=0; i<4; i++) {fread(&sb[i], 1, 1, fp);sb[i] = sb[i] & 0xFF;}
  evt->crc32 = Decode32(sb);
  
  //read end bytes into temp_byte_bottom
  eof_ck = fread(&temp_byte_bottom, 2, 1, fp);

  if (Print) {
    printf("status: %d--", evt->status); printbinary(evt->status,16);
    printf("Packet Length = %d--", evt->len); printbinary(evt->len, 16); 
    printf("ROI = %d--", evt->roi); printbinary(evt->roi, 16); 
    printf("DNA = %llu--", evt->dna); printbinary(evt->dna, 64); 
    printf("FW_hash = %d--",evt->fw_hash); printbinary(evt->fw_hash,16); 
    printf("ID = %d--", evt->id); printbinary(evt->id, 16); 
    printf("CH_MASK = %d--",evt->ch_mask); printbinary(evt->ch_mask,16); 
    printf("EVT_CTR = %ld ",evt->event_ctr); printbinary(evt->event_ctr, 32); 
    printf("DTAP0 = %d--", evt->dtap0); printbinary(evt->dtap0, 16); 
    printf("DTAP1 = %d--", evt->dtap1); printbinary(evt->dtap1, 16); 
    printf("TIMESTAMP = %llu--", evt->timestamp); printbinary(evt->timestamp, 64);

    for (int i=0; i<NCHN; i++) {
      printf("ch_head[%i] = %d--", i, evt->ch_head[i]);
      printbinary(evt->ch_head[i], 16); 
      //for (int j=0; j<NWORDS; j++) 
      //  if (j%10 == 0 && i == 8) {
      //    printf("Ch %d: ADC %d = %d\n",i,j,evt->ch_adc[i][j]);
      //    printf("ch_trail[%i] = %ld--", i, evt->ch_trail[i]);
      //    printbinary(evt->ch_trail[i], 32); 
      //  }
    }
    printf("STOP_CELL = %d--", evt->stop_cell); printbinary(evt->stop_cell, 16);
    printf("CRC32 = %ld--", evt->crc32); printbinary(evt->crc32, 32);
  }

  //std::cout << "[INFO] <ReadEvent> top bytes " << temp_byte_top << std::endl;
  //std::cout << "[INFO] <ReadEvent> bottom bytes " << temp_byte_bottom << std::endl;

  if (temp_byte_bottom==tail) { // Verify the event ended properly
    evt->tail = temp_byte_bottom;
    return (1);
  } else if (eof_ck == 0) {
    return (-1);
  } else {
    return (0);
  }
}

unsigned long long Decode64(unsigned long long tb[]) {
  unsigned long long val = (tb[1]<<56 | tb[0]<<48 | tb[3]<<40 | tb[2]<<32 |
                        tb[5]<<24 | tb[4]<<16 | tb[7]<<8 | tb[6]);
  return (val);
}

unsigned long long Decode48(unsigned long long tb[]) {
  unsigned long long val = (tb[1]<<40 | tb[0]<<32 |
                        tb[3]<<24 | tb[2]<<16 | tb[5]<<8 | tb[4]);
  return (val);
}

unsigned long Decode32(unsigned long tb[]) {
  unsigned long val = (tb[1]<<24 | tb[0]<<16 | tb[3]<<8 | tb[2]);
  return (val);
}

int nthbit(unsigned long long number, int n){ //n start with 0
  long long bitselector = (long long)pow((float)2, (float)n);
  unsigned long long temp = number & bitselector;
  int bitcontent;
  if (temp==bitselector) bitcontent=1;
  else bitcontent=0;
  return bitcontent;
}

void printbinary(unsigned long long number, int bit){
  int printspace=0;
  for (int n=bit-1; n>=0; n--){
    if (printspace==4) {
      printf(" ");
      printspace=0;
    }
    printf("%1d", nthbit(number, n));
    printspace++;
  }
  printf("\n");
}

//---------------------------------------------------------------------------
// VoltageCalibration :: translate ADC units into voltage measurement
//---------------------------------------------------------------------------
void VoltageCalibration(short traceIn[], double_t traceOut[],
                    unsigned int tCell, struct Calibrations_t cal)
{
  for (int i = 0; i < 1024; i++) {
    traceOut[i] = (double_t) traceIn[i];
    //if (i%100 == 0)
      //printf("%f\n", traceOut[i]);
    traceOut[i] -= cal.vofs[(i+tCell)%1024];
    traceOut[i] -= cal.vdip[i];
    traceOut[i] *= cal.vinc[(i+tCell)%1024];
  }
}

void VoltageNonCalibration(short traceIn[], double_t traceOut[])
{
  for (int i = 0; i < 1024; i++) {
    traceOut[i] = (double_t) traceIn[i];
  }
}

//---------------------------------------------------------------------------
// TimingCalibration :: determine calibrated readout time for each cell
//---------------------------------------------------------------------------
void TimingCalibration(double_t times[],
                        unsigned int tCell, struct Calibrations_t cal)
{
  times[0] = 0.0;
  for (int i = 1; i < 1024; i++) {
    times[i] = times[i-1] + cal.tbin[(i-1+tCell)%1024];
  }
}

//---------------------------------------------------------------------------
// RemoveSpikes :: modified spike removal routine from drs-5.0.6/src/Osci.cpp
//---------------------------------------------------------------------------
void RemoveSpikes(double_t wf[NCHN][1024], unsigned int tCell, int spikes[])
//void RemoveSpikes(short int wf[NCHN][1024], unsigned int tCell, int spikes[])
{
  int i, j, k, l;
  double x, y;
  int sp[NCHN][10];
  int rsp[10];
  int n_sp[NCHN];
  int n_rsp;
  int nNeighbor, nSymmetric;
  int nChn = NCHN;
  double_t filter, dfilter;

  memset(sp, 0, sizeof(sp));
  memset(rsp, 0, sizeof(rsp));
  memset(n_sp, 0, sizeof(n_sp));
  n_rsp = 0;

  /* set rsp to -1 */
  for (i = 0; i < 10; i++)
  {
    rsp[i] = -1;
  }
  /* find spikes with special high-pass filters */
  for (j = 0; j < 1024; j++)
  {
    for (i = 0; i < nChn; i++)
    {
      filter = -wf[i][j] + wf[i][(j + 1) % 1024] + wf[i][(j + 2) % 1024] - wf[i][(j + 3) % 1024];
      dfilter = filter + 2 * wf[i][(j + 3) % 1024] + wf[i][(j + 4) % 1024] - wf[i][(j + 5) % 1024];
      if (filter > 20 && filter < 100)
      {
        if (n_sp[i] < 10)   // record maximum of 10 spikes
        {
          sp[i][n_sp[i]] = (j + 1) % 1024;
          n_sp[i]++;
        }
        else                // too many spikes -> something wrong
        {
          return;
        }
        // filter condition avoids mistaking pulse for spike sometimes
      }
      else if (dfilter > 40 && dfilter < 100 && filter > 10)
      {
        if (n_sp[i] < 9)   // record maximum of 10 spikes
        {
          sp[i][n_sp[i]] = (j + 1) % 1024;
          sp[i][n_sp[i] + 1] = (j + 3) % 1024;
          n_sp[i] += 2;
        }
        else                // too many spikes -> something wrong
        {
          return;
        }
      }
    }
  }

  /* find spikes at cell #0 and #1023
  for (i = 0; i < nChn; i++) {
    if (wf[i][0] + wf[i][1] - 2*wf[i][2] > 20) {
      if (n_sp[i] < 10) {
        sp[i][n_sp[i]] = 0;
        n_sp[i]++;
      }
    }
    if (-2*wf[i][1021] + wf[i][1022] + wf[i][1023] > 20) {
      if (n_sp[i] < 10) {
        sp[i][n_sp[i]] = 1022;
        n_sp[i]++;
      }
    }
  }
  */

  /* go through all spikes and look for neighbors */
  for (i = 0; i < nChn; i++)
  {
    for (j = 0; j < n_sp[i]; j++)
    {
      nSymmetric = 0;
      nNeighbor = 0;
      /* check if this spike has a symmetric partner in any channel */
      for (k = 0; k < nChn; k++)
      {
        for (l = 0; l < n_sp[k]; l++)
          if ((sp[i][j] + sp[k][l] - 2 * tCell) % 1024 == 1022)
          {
            nSymmetric++;
            break;
          }
      }
      /* check if this spike has same spike is in any other channels */
      for (k = 0; k < nChn; k++)
        if (i != k)
        {
          for (l = 0; l < n_sp[k]; l++)
            if (sp[i][j] == sp[k][l])
            {
              nNeighbor++;
              break;
            }
        }
      /* if at least two matching spikes, treat this as a real spike */
      if (nNeighbor >= 2)
      {
        for (k = 0; k < n_rsp; k++)
          if (rsp[k] == sp[i][j]) // ignore repeats
            break;
        if (n_rsp < 10 && k == n_rsp)
        {
          rsp[n_rsp] = sp[i][j];
          n_rsp++;
        }
      }
    }
  }

  /* recognize spikes if at least one channel has it */
  for (k = 0; k < n_rsp; k++)
  {
    spikes[k] = rsp[k];
    for (i = 0; i < nChn; i++)
    {
      if (k < n_rsp && fabs(rsp[k] - rsp[k + 1] % 1024) == 2)
      {
        /* remove double spike */
        j = rsp[k] > rsp[k + 1] ? rsp[k + 1] : rsp[k];
        x = wf[i][(j - 1) % 1024];
        y = wf[i][(j + 4) % 1024];
        if (fabs(x - y) < 15)
        {
          wf[i][j % 1024] = x + 1 * (y - x) / 5;
          wf[i][(j + 1) % 1024] = x + 2 * (y - x) / 5;
          wf[i][(j + 2) % 1024] = x + 3 * (y - x) / 5;
          wf[i][(j + 3) % 1024] = x + 4 * (y - x) / 5;
        }
        else
        {
          wf[i][j % 1024] -= 14.8f;
          wf[i][(j + 1) % 1024] -= 14.8f;
          wf[i][(j + 2) % 1024] -= 14.8f;
          wf[i][(j + 3) % 1024] -= 14.8f;
        }
      }
      else
      {
        /* remove single spike */
        x = wf[i][(rsp[k] - 1) % 1024];
        y = wf[i][(rsp[k] + 2) % 1024];
        if (fabs(x - y) < 15)
        {
          wf[i][rsp[k]] = x + 1 * (y - x) / 3;
          wf[i][(rsp[k] + 1) % 1024] = x + 2 * (y - x) / 3;
        }
        else
        {
          wf[i][rsp[k]] -= 14.8f;
          wf[i][(rsp[k] + 1) % 1024] -= 14.8f;
        }
      }
    }
    if (k < n_rsp && fabs(rsp[k] - rsp[k + 1] % 1024) == 2)
      k++; // skip second half of double spike
  }
}

