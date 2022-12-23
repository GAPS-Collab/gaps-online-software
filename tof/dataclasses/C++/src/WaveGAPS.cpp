#include <stdlib.h>
#include <stdio.h>
#include <math.h>
#include <iostream>

/* Waveform stuff. */
#include "WaveGAPS.h"

#include "CraneLogging.hh"

// Some useful macros
#define SQR(A)               ( (A) * (A) )
#define ABS(A)               ( ( (A<0) ? -(A) : (A) ) )

using namespace std;
using namespace GAPS;

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
// Default constructor
Waveform::Waveform(double *data, double *time, int chnl, int flag ) {

  ch = chnl;
  runno = 4;

  // Initialize our bank pointers
  InitializePointers();

  // Initialize some variables
  InitializeVariables(ch);

  // Now we copy the data to internal storage
  WaveData = new double[wf_size];
  WaveTime = new double[wf_size];
  for(int i=0; i<wf_size; i++) {
    WaveData[i] = data[i];
    WaveTime[i] = time[i];
    //WaveTime[i] = i*0.2;
  }
  bin_ns = (wf_size - 1.0) / (WaveTime[wf_size-1] - WaveTime[0]);
  //MeasurePeaksRms();
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
Waveform::Waveform (int size)
// This constructor is used to create a Waveform from scratch. The
// whole trace is flat at 0 mV, with 0 varinace and 0 pedestal.
{
  ch = -99; runno = -9999;
  InitializePointers();
  InitializeVariables(0);
  wf_size = size;
  WaveData = new double[wf_size];
  WaveTime = new double[wf_size];
  for (int bin = 0; bin < wf_size; bin++) {
    WaveData[bin] = 0;
    WaveTime[bin] = bin;
  }
}


/***************************************************************************/

void Waveform::SetTime(std::vector<double> times)
{
    for (uint j=0; j<wf_size; j++)
    { WaveTime[j] = times[j];}
}

/***************************************************************************/

void Waveform::SetWave(std::vector<double> wave)
{
    for (uint j=0; j<wf_size; j++)
    { WaveData[j] = wave[j];}
}

/***************************************************************************/

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
// Default destructor
Waveform::~Waveform(void) {
  //Delete trace;
  if (WaveData) 
    delete[] WaveData;
  if (WaveTime) 
    delete[] WaveTime;

  // If we saved the original waveform because we modified it, delete it.
  //if (WaveDataOrig) 
  //delete[] WaveDataOrig;

  // If we read in a baseline, delete it.
  if (wf_baseline) 
    delete[] wf_baseline;
  

  // Free up the peak measurement memory.
  CleanUpPeaks();
  
  /*if (tracefit)
    delete tracefit;
  if (pulsefit)
  delete pulsefit; */
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveform::InitializePointers() {

  WaveData     = NULL;
  WaveTime     = NULL;
  //WaveDataOrig = NULL;
  wf_baseline  = NULL;
  
  //tracefit    = NULL;
  //pulsefit = NULL;
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveform::InitializeVariables(int no_acq) {
  
  // stuff related to the peaks
  max_num_peaks   = 15;  // Some relatively small value to start so that we
                        // don't use a lot of memory.  Can be changed up
                        // to valued defined in Waveform.h.
  num_peaks       = 0;
  peaks_found     = 0;
  peaks_allocated = 0;
  peak_plot = 0;  // Turn off peak plotting by default.  Can be turned
                  // back on with SetPeakPlot() method
  cfds_frac       = 0.25;
  cfde_frac       = 0.50;
  cfde_offset     = 5.0;

  timing_corr     = 0.0;

  impedance       = 50.0;

  wf_pedestal     = 0;
  wf_pedsigma     = 0;

  run_pedestal    = 0.0;  // Set to zero until we know a better value.

  // Here's how we initialize our variables for now
  wf_size = 1000;
  acq_ch = ch;

  dc2mv  = 1.0;
  offset = 0.0;
  bin_ns = 1.0;     // Set to 1 for now. Will calculate later
  base   = 0.0;

  wf_pedshift = 0.0;
  
  // stuff related to the pedestals
  // Need to do after setting wf_size
  wf_ped_begin   = 100;
  wf_ped_range   = 400;
  //wf_ped_range   = wf_size - wf_ped_begin;
  //saturated_hi   =  1000.0;
  //saturated_lo   = -1000.0;
  // We use these to determine if some quantity is saturated or not.
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
// PmtThreshold should be set to the value used by vmedaq.
void Waveform::SetThreshold(float PmtThreshold){
  Threshold = PmtThreshold; 
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

//==================PEDESTAL RELATED STUFF=================
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveform::SetPedRange(float range) {
  // This is a little convoluted, but we must convert the range (in
  // ns) into bins
  int bin_range = Time2Bin(WaveTime[wf_ped_begin]+range) - wf_ped_begin;

  if (bin_range < 0 || bin_range > wf_size) {
    char txt[1000];
    sprintf(txt,"Invalid range for calculating pedestals--%4d.  Not set", 
            bin_range);
    log_trace(txt);
  } else if (wf_ped_begin+bin_range > wf_size) {
    char txt[1000];
    log_warn("SetPedRange:  Range goes beyond waveform.");
    sprintf(txt,"\twf_bin_range not set to %d",bin_range);
    log_warn(txt);
  } else {
    wf_ped_range = bin_range;
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveform::SetPedBegin(float begin)
{
  int begin_bin = Time2Bin(begin);
  if (begin_bin < 0)
  {
    log_trace("Unable to set a negative pedestal beginning.");
  }
  else if (begin_bin+wf_ped_range > wf_size)
  {
    char txt[1000];
    log_warn("Starting too far into waveform.");
    sprintf(txt,"\twf_ped_begin not set to %d(%d)",begin_bin,wf_ped_range);
    log_warn(txt);
  }
  else
  {
    wf_ped_begin = begin_bin;
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveform::CalcPedestalRange(void)
{
  double sum=0;
  double sum2=0;
  double average;

  /* Here we find the sum and sum^2 for each point of the waveform between
     the start point and the end point (determined from wf_ped_begin and
     wf_ped_range).  */
  int ctr=0;
  for(int i=wf_ped_begin ; i<(wf_ped_begin+wf_ped_range) ; i++ )
  {
    if (abs(WaveData[i]) < 10.0)
    {
      sum  += (double) WaveData[i];
      sum2 += (double) SQR(WaveData[i]);
      ctr++;
    }
  }

  //average = sum / (double) wf_ped_range;
  average = sum / (double) ctr;
  //if (ctr!=wf_ped_range) printf("%d:  %d   %d\n", ch, ctr, wf_ped_range);

  // Now set the pedestal and pedsigma values (in mV)
  wf_pedestal = average;
  //wf_pedsigma = sqrt( sum2/(double)wf_ped_range - SQR(average) );
  wf_pedsigma = sqrt( sum2/(double)ctr - SQR(average) );

}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveform::CalcPedestalDynamic(void)
{
  // algorithm to find pedestal window and remove darks - JLR 180821
  double sum = 0;
  double sum2 = 0;
  double average;
  double cmin = 1000; // minimum value of the cross-correlation
  int cmax_ind = 0;
  // magic numbers
  const int spe_lag = 80;
  double corr_cut = 1;
  const int offset = 5;
  // 2 GS/s spe
  double spe[20];
  //fill_n(spe, 20, -0.05);
  // cross-correlate with normalized single photoelectron trace
  double CrossCorr[wf_size-spe_lag];
  int ctr;
  while(true)
  {
    for (int i = 0; i < wf_size-spe_lag; i++)
    {
      CrossCorr[i] = 0;
      for (int j = 0; j < 20; j++)
        CrossCorr[i] += WaveData[i+j]*spe[j];
      if (CrossCorr[i] < cmin)
        cmin = CrossCorr[i];
      if (CrossCorr[i] > CrossCorr[cmax_ind])
        cmax_ind = i;
    }
    // subtract minimum
    for (int i = 0; i < wf_size-spe_lag; i++)
      CrossCorr[i] -= cmin;
    // set pedestal window
    wf_ped_begin = 10;
    wf_ped_range = cmax_ind-10;
    // for each index, add if previous 'spe_lag' bins are below threshold
    ctr = 0;
    for (int i = wf_ped_begin+offset; i < wf_ped_begin+wf_ped_range; i++)
    {
      int j;
      if (wf_ped_begin >= i-spe_lag)
        j = wf_ped_begin;
      else
        j = i-spe_lag;
      while (j <= i-offset) {
        if (CrossCorr[j] > corr_cut)
          break;
        j++;
      }
      if (j > i-offset)
      {
        sum  += (double) WaveData[i];
        sum2 += (double) SQR(WaveData[i]);
        ctr++;
      }
    }
    if (ctr > 0)
      break;
    else
      corr_cut += 0.1;
    if (corr_cut > 5)
    {
      ctr = 1;
      break;
    }
  }
  // get mean and r.m.s.
  average = sum / (double) ctr;
  wf_pedestal = average;
  wf_pedsigma = sqrt(sum2 / (double) ctr - SQR(average));
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveform::SubtractPedestal(void)
{
  // Subtract the calculated pedestal value from the whole trace
  for(int i = 0; i < wf_size; i++)
    WaveData[i] -= wf_pedestal;
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


//==================PEAK RELATED STUFF=====================
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveform::AllocatePeaks(void)
{
  // Now allocate pointers to the peak pos., tdc, width, height and charge 
  peaks   = new int[num_peaks];
  tdcs    = new double[num_peaks];
  charge  = new double[num_peaks];
  width   = new double[num_peaks];
  height  = new double[num_peaks];
  peaks_allocated = 1;
}

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveform::SetMaxPeaks(int max_num)
{
  char text[500];
  if (max_num > MAX_NUM_PEAKS)
  {
    sprintf(text,"Cannot set 'max_num_peaks' larger than %d. Nothing done.\n",
            MAX_NUM_PEAKS);
    Message(text);
    return;
  }
  else
  {
    max_num_peaks = max_num;
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveform::CleanUpPeaks(void)
{
  if (peaks_found)
  {
    delete[] begin_pk;
    delete[] end_pk;
    //delete[] sat_pk;
    delete[] spikes;
    peaks_found = 0;
  }
  if (peaks_allocated)
  {
    delete[] peaks;
    delete[] tdcs;
    delete[] width;
    delete[] height;
    delete[] charge;
    peaks_allocated = 0;
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
int Waveform::GetNumPeaks(void)
{
  if (peaks_found) {
    return num_peaks;
  } else {
    return (-1);
  }   
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveform::FindPeaks(float start, float size) {
  pulse_start = start;
  CleanUpPeaks();
  // Turn time values into bin numbers
  int start_bin = Time2Bin(start);
  int size_bin = Time2Bin(start + size) - start_bin;
  // Check limits
  if ((start_bin + size_bin) > wf_size)
    size_bin = wf_size - start_bin;

  // Modified FindNumPeaks
  int min_wid = 3;     // minimum peak width
  int pk_ctr = 0;      // current number of peaks
  int pos = start_bin; // current position
  int peak_bins = 0;   // current peak width (in bins)
  begin_pk  = new int[max_num_peaks];
  end_pk    = new int[max_num_peaks];
  spikes    = new int[max_num_peaks];
  // Step through trace until we are above threshold
  while ((WaveData[pos] < Threshold) && (pos < wf_size))
    pos++;
  for (int i = pos; i < start_bin + size_bin; i++) {
    if (WaveData[i] > Threshold) {
      peak_bins++;
      if (peak_bins == min_wid) { // new peak detected
        if (pk_ctr == max_num_peaks) {
          Message("Maximum number of peaks exceeded");
          break;
        }
	    begin_pk[pk_ctr] = i - (min_wid - 1);
        spikes[pk_ctr] = 0;
        end_pk[pk_ctr] = 0;
        pk_ctr++;
      } else if (peak_bins > min_wid) {
        // each "bump" counts as a separate peak
        int grad = 1;
        for (int k = 0; k < 3; k++) {
          if (WaveData[i-k] > WaveData[i-(k+1)])
            grad = 0;
        }
        if (grad == 0)
          continue;
        if (end_pk[pk_ctr-1] == 0)
          end_pk[pk_ctr-1] = i; // Set last bin included in peak
      }
    } else {
      peak_bins = 0;  // Reset bin counter at each bin not meeting requirement
    }
  }
  num_peaks = pk_ctr;
  begin_pk[num_peaks] = wf_size; // Need this to measure last peak correctly
  peaks_found = 1;
  
  // Alocate memory and get peak parameters
  AllocatePeaks();
  /* Commented out for now because they are unused (working) calculations
    for(int i = 0; i < num_peaks; i++) {
    peaks[i]  = GetMaxBin(begin_pk[i], end_pk[i]-begin_pk[i]);
    height[i] = WaveData[peaks[i]];
    width[i]  = CalculateWidth(i, 0.5*height[i]);
    charge[i] = Integrate( WaveTime[peaks[i]]+(int)PEAK_OFFSET, PEAK_LENGTH);
  }
  */
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
double Pulse(double *x, double *par) {
// Pulse function
  double xx = x[0];
  double f;
  if (xx < par[1]) {
    f = 0; 
  } else {
    f=1;
    /*
    f = par[0]*TMath::Power((xx-par[1])/par[2],par[3]); // power law
    f *= TMath::Exp(-(xx-par[1])/par[2]); // exponential
    f *= TMath::Power(TMath::E()/par[3],par[3]); // normalization
    */
  }
  return f;
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveform::FitPulse() {
  /*if (tracefit)
    delete tracefit;
  if (pulsefit)
    delete pulsefit;

  int lo = peaks[0]-50;
  int hi = peaks[0]+5;
  if (lo < 0)
    lo = 0;
  if (hi > wf_size)
    hi = wf_size - 1;
  
  tracefit = new TGraph(wf_size, WaveTime, WaveData);
  pulsefit = new TF1("pulse", Pulse, pulse_start, WaveTime[hi], 4);
  // TODO - arbitrary initial guess
  pulsefit->SetParameters(height[0], WaveTime[lo], 1.0, 8.0);
  tracefit->Fit(pulsefit, "WRQN");
  pulsepars[0] = pulsefit->GetParameter(0);
  pulsepars[1] = pulsefit->GetParameter(1);
  pulsepars[2] = pulsefit->GetParameter(2);
  pulsepars[3] = pulsefit->GetParameter(3);
  pulsechi2    = pulsefit->GetChisquare();
  ndf          = pulsefit->GetNDF();
  */
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveform::FindTdc(int pk_num, int th_type) {
  switch (th_type) {
    case CONSTANT:
      tdcs[pk_num] = FindInterpolatedTime(Threshold,begin_pk[pk_num]-1,1);
      break;
    case CFD_ELEC:
      tdcs[pk_num] = FindCFDElecTime(0.0,begin_pk[pk_num]-1,100);
      break;
    case CFD_SIMPLE:
      tdcs[pk_num] = FindCFDSimpTime(pk_num);
      break;
      /*case PCONSTANT: // TODO - only works for pk_num = 0
      tdcs[pk_num] = pulsefit->GetX(Threshold,pulse_start,WaveTime[peaks[0]]);
      break;
    case PCFD:
      double cfdval = cfds_frac * pulsepars[0];
      tdcs[pk_num] = pulsefit->GetX(cfdval,pulse_start,WaveTime[peaks[0]]);
      break;*/
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
double Waveform::FindInterpolatedTime(float thresh, int idx, int size){
  // idx is the last bin which is below threshold.  lval is the value of
  // the last bin below threshold.  hval is the value of the first value
  // above threshold and thresh is the desired threshold.

  double lval,hval=0;

  if(size<0){size = wf_size - idx; }
  if((idx<0) || (wf_size < idx + size)){ return(ERRVAL); }

  thresh = abs(thresh);
  lval = abs(WaveData[idx]);
  if(size == 1) { // We were passed the two bins around crossing
    hval = abs(WaveData[idx+1]);
  } else { // Find the bins around the crossing
    for(int i=idx+1; i<idx+size; i++){
      hval = abs(WaveData[i]);
      if( (hval>=thresh) && (thresh<=lval)) { // Threshold crossing?
        idx = i-1; // Reset idx to point before crossing
        break;
      }
      lval = hval;
    }
  }
  
  if ( lval > thresh && size != 1) 
    // This can occur when the trace stayed above threshold, but a
    // second peak was found.  In that case, we set the time to the
    // index value since it is the bottom of the valley.  (Don't forget
    // that our pulses are negative going.)
    return WaveTime[idx];
  else if( hval == lval ) 
    // Rare occurence if it will ever occur
    return WaveTime[idx];
  else {
    // Normal peak where we should interpolate.
    float time = WaveTime[idx] + 
      (thresh-lval)/(hval-lval) * (WaveTime[idx+1]-WaveTime[idx]) ;
    /*printf("%2d(%d-%d) -- %.2f, %.2f  -- %.2f, %.2f, %.2f    %.2f\n", ch, 
           num_peaks, idx, WaveTime[idx], WaveTime[idx+1], thresh, lval, 
           hval, time); */
    return time;
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
double Waveform::FindCFDElecTime(float thresh, int idx, int size){
  // idx is the last bin which is below threshold.  lval is the value of
  // the last bin below threshold.  hval is the value of the first value
  // above threshold and thresh is the desired threshold.

  double lval,hval=0;

  double ret_val = -99;

  if(size<0){size = wf_size - idx; }
  if((idx<0) || (wf_size < idx + size)){ return(ERRVAL); }

  // Set the initial value for our CFD calculation
  lval = cfde_frac*WaveData[idx] - WaveData[idx-cfde_offset];
    
  int minbin = idx;
  double minval = cfde_frac*WaveData[idx]-WaveData[idx - cfde_offset];
    
  for (int j = idx; j < idx + cfde_offset; j++){
      if (minval >= cfde_frac*WaveData[j]-WaveData[j - cfde_offset] ){
          minval = cfde_frac*WaveData[j]-WaveData[j - cfde_offset];
          minbin = j;
        } }

  for (int j=minbin; j<minbin + size; j++) {
    hval = cfde_frac*WaveData[j+1] - WaveData[j+1-cfde_offset];
    // Check if we had a zero crossing.  run_pedestal is our defined
    // zero value and all pulses are negative going (so we look for a
    // positive going zero crossing
    //printf("%d: %d  %d  %.2f  %.2f\n", ch, idx, j, lval, hval);
    if ( lval < thresh && hval > thresh) {
      ret_val = - (hval - ((hval - lval) / (WaveTime[j] - WaveTime[j-1])) * WaveTime[j]) * (WaveTime[j] - WaveTime[j-1])/(hval - lval);
      return (ret_val);
    }      
    lval = hval;
  }
  
  return (-101);  // Something went wrong, return -99 for cfde method failures to be included in ta-tb histogram
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
double Waveform::FindCFDSimpTime(int pk_num){
  // idx is the last bin which is below threshold.  lval is the value of
  // the last bin below threshold.  hval is the value of the first value
  // above threshold and thresh is the desired threshold.

  if( pk_num<0 || pk_num>MAX_NUM_PEAKS ) { return(WaveTime[wf_size]); }

  // Determine the threshold for finding the time. Use 25% of the
  // average of the bin of the highest peak and the two bins next to
  // it.
  int idx = GetMaxBin(begin_pk[pk_num], end_pk[pk_num]-begin_pk[pk_num]);
  
  double sum = 0.0;
  for (int i=idx-1; i<=idx+1; i++) sum += WaveData[i];
  double tmp_thresh = abs(cfds_frac * (sum / 3.0));

  // Now scan through the waveform around the peak to find the bin
  // crossing the calculated threshold. Bin idx is the peak so it is
  // definitely above threshold. So let's walk backwards through the
  // trace until we find a bin value less than the threshold.
  int lo_bin = wf_size;
  for (int i=idx; i>begin_pk[pk_num]-10; i--) {
    if ( abs(WaveData[i]) < tmp_thresh ) {
      lo_bin = i;
      i=0;
    }
  }

  double cfd_time;
  if (lo_bin < wf_size) 
    cfd_time = FindInterpolatedTime(tmp_thresh, lo_bin, 1);
  else 
    cfd_time = WaveTime[wf_size];
  
  return cfd_time;

}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
int Waveform::Time2Bin(float t_ns){
  // Given a time in ns, find the bin most closely corresponding to that time
  for (int i=0; i<wf_size; i++) 
    if (WaveTime[i] > t_ns) 
      return (i-1); 

  log_trace("-- " << t_ns);
  log_trace("Did not find a bin corresponding to the given time.");
  return (-1);
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
int Waveform::GetSpikes(int i = 0) {
  // Do some error checking first
  if ( i<0 ) return (-1);
  if ( i>num_peaks ) return (-2);
  if ( !peaks_allocated ) {
    Message("Peaks not measured yet.  Cannot return pulse spike value");
    return (-3);
  }    
  // Now return the appropriate value
  return spikes[i];
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
double Waveform::GetTdcs(int i = 0) {
  // Do some error checking first
  if ( i<0 ) return (-1);
  if ( i>num_peaks ) return (-2);
  if ( !peaks_allocated ) {
    Message("Peaks not measured yet.  Cannot return pulse tdc value");
    return (-3);
  }    
  // Now return the appropriate value.  Note: timing_corr is 0.0 for
  // Digi banks but non-zero for Dig0 banks.
  return tdcs[i] - timing_corr;
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
double Waveform::GetCharge(int i = 0) {
  // Do some error checking first
  if ( i<0 ) return (-1);
  if ( i>num_peaks ) return (-2);
  if ( !peaks_allocated ) {
    Message("Peaks not measured yet.  Cannot return pulse charge");
    return (-3);
  }    
  // Now return the appropriate value (in pC)
  return charge[i];
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
double Waveform::GetHeight(int i = 0) {
  // Do some error checking first
  if ( i<0 ) return (-1);
  if ( i>num_peaks ) return (-2);
  if ( !peaks_allocated ) {
    Message("Peaks not measured yet.  Cannot return pulse height");
    return (-3);
  }    
  // Now return the appropriate value (in mV)
  return height[i];
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
double Waveform::GetWidth(int i = 0) {
  // Do some error checking first
  if ( i<0 ) return (-1);
  if ( i>num_peaks ) return (-2);
  if ( !peaks_allocated ) {
    Message("Peaks not measured yet.  Cannot return pulse width");
    return (-3);
  }    
  // Now return the appropriate value (in ns)
  return width[i];
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

//==================END OF PEAK STUFF======================

void Waveform::Message(const char *s) {
  cout << s << endl;
}

//===============STUFF FROM DAN's CLASS======================

// --------------------------------------------------------------------------
// This method sets the value of a given bin in mV.
// --------------------------------------------------------------------------
double Waveform::SetBin(int idx, double val) {
  if ((idx < 0) || (wf_size <= idx)) { return(ERRVAL); }
  else { WaveData[idx] = val; return(0); }
}        

// --------------------------------------------------------------------------
// This method returns the value of a given bin in mV.
// --------------------------------------------------------------------------
double Waveform::GetBin(int idx) {
  if ((idx < 0) || (wf_size <= idx)) { return(ERRVAL); }
  else { return(WaveData[idx]); }
}  

// --------------------------------------------------------------------------
// This method returns the Time value of a given bin in ns.
// --------------------------------------------------------------------------
double Waveform::GetBinTime(int idx) {
  if ((idx < 0) || (wf_size <= idx)) { return(ERRVAL); }
  else { return(WaveTime[idx]); }
}  

// --------------------------------------------------------------------------
// This method returns the bin with the largest DC value.
// --------------------------------------------------------------------------
int Waveform::GetMaxBin(int lo = 0, int size = -1) {
  if(size < 0){ size = wf_size - lo; }
  if((lo < 0) || (wf_size < lo + size)){ return(ERRVAL); }
  double maxval = GetBin(lo);
  int    maxbin = lo;
  for(int idx = lo + 1 ; idx < lo + size ; idx++){
    if(maxval < GetBin(idx)){ maxval = GetBin(idx); maxbin = idx; }
  }
  return(maxbin);
}

// --------------------------------------------------------------------------
// This method returns the time with the largest DC value.
// --------------------------------------------------------------------------
double Waveform::GetMaxBinTime(int lo = 0, int size = -1) {
  if(size < 0){ size = wf_size - lo; }
  if((lo < 0) || (wf_size < lo + size)){ return(ERRVAL); }
  double maxval = GetBin(lo);
  int    maxbin = lo;
  for(int idx = lo + 1 ; idx < lo + size ; idx++){
    if(maxval < GetBin(idx)){ maxval = GetBin(idx); maxbin = idx; }
  }
  return(WaveTime[maxbin]);
}

// --------------------------------------------------------------------------
// This method returns the largest value in mV.
// --------------------------------------------------------------------------
double Waveform::GetMaxVal(int lo = 0, int size = -1) {
  if(size < 0){ size = wf_size - lo; }
  if((lo < 0) || (wf_size < lo + size)){ return(ERRVAL); }
  double maxval = GetBin(lo);
  for(int idx = lo + 1 ; idx < lo + size ; idx++){
    if(maxval < GetBin(idx)){ maxval = GetBin(idx); }
  }
  return(maxval);
}

// --------------------------------------------------------------------------
// This method returns the bin with the lowest DC value.
// --------------------------------------------------------------------------
int Waveform::GetMinBin(int lo = 0, int size = -1) {
  if(size < 0){ size = wf_size - lo; }
  if((lo < 0) || (wf_size < lo + size)){ return(ERRVAL); }
  double minval = GetBin(lo);
  int    minbin = lo;
  for(int idx = lo + 1 ; idx < lo + size ; idx++){
    if(GetBin(idx) < minval){ minval = GetBin(idx); minbin = idx; }
  }
  return(minbin);
}

// --------------------------------------------------------------------------
// This method returns the time with the lowest DC value.
// --------------------------------------------------------------------------
double Waveform::GetMinBinTime(int lo = 0, int size = -1) {
  if(size < 0){ size = wf_size - lo; }
  if((lo < 0) || (wf_size < lo + size)){ return(ERRVAL); }
  double minval = GetBin(lo);
  int    minbin = lo;
  for(int idx = lo + 1 ; idx < lo + size ; idx++){
    if(GetBin(idx) < minval){ minval = GetBin(idx); minbin = idx; }
  }
  return(WaveTime[minbin]);
}

// --------------------------------------------------------------------------
// This method returns the largest value in mV in a DC window
// --------------------------------------------------------------------------
double Waveform::GetMinVal(int lo = 0, int size = -1) {
  if(size < 0){ size = wf_size - lo; }
  if((lo < 0) || (wf_size < lo + size)){ return(ERRVAL); }
  double minval = GetBin(lo);
  for(int idx = lo + 1 ; idx < lo + size ; idx++){
    if(GetBin(idx) < minval){ minval = GetBin(idx); }
  }
  return(minval);
}

// --------------------------------------------------------------------------
// This method returns the largest (positive) value in mV in a time window
// --------------------------------------------------------------------------
double Waveform::GetPeakValue(float lo = 0.0, float size = -1.0) {
  if(size < 0.0){ size = WaveTime[wf_size-1] - lo; }
  if((lo < 0) || (WaveTime[wf_size-1] < lo + size)){ return(ERRVAL); }
  
  int lo_bin = Time2Bin(lo);
  int hi_bin = Time2Bin(lo+size);
  // Set the start point to the data value of the "lo" time bin
  double maxval = GetBin(lo_bin);
  
  for(int i = lo_bin ; i < hi_bin; i++){
    if(GetBin(i) > maxval){ maxval = GetBin(i); }
  }
  return(maxval);
}

void Waveform::Rescale(double factor = 1) {
  if(factor == 1){ return; }
  for(int idx = 0 ; idx < wf_size ; idx++){ WaveData[idx] *= factor; }
}

double Waveform::Integrate(float lo = 0, float size = -1) {
  // First, we need to convert our values of lo and size into bin values.
  int lo_bin   = Time2Bin(lo);
  int size_bin = Time2Bin(lo + size) - lo_bin;
  //printf("lo = %.2f; size = %.2f;  bin = %d\n", lo, size, lo_bin);
  
  if(size_bin < 0){ size_bin = wf_size - lo_bin; }
  if((wf_size < lo_bin + size_bin) || (lo_bin < 0)){ return(ERRVAL); }
  double sum = 0;
  int    max = lo_bin + size_bin;

  for(int idx = lo_bin ; idx < max ; idx++){ 
    //sum += (GetBin(idx) - wf_pedestal); 
    sum += ( GetBin(idx) * (WaveTime[idx] - WaveTime[idx-1]) ); 
  }

  // Return charge in pC...
  //sum *= (bin_ns / impedance); 
  sum /= (impedance); 

  return(sum);
}

