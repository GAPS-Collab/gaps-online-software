#include <stdlib.h>
#include <stdio.h>
#include <TF1.h>
#include <TGraph.h>
#include <TMath.h>

/* Waveform stuff. */
#include "../include/Waveplot.h"

// Some useful macros
#define SQR(A)               ( (A) * (A) )
#define ABS(A)               ( ( (A<0) ? -(A) : (A) ) )

using namespace std;

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
// Default constructor
Waveplot::Waveplot(double *data, double *time, int chnl, int flag ) {

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

  // 'flag' is set by default unless the waveform is constructed to the
  // contrary.  Thus, the default behavior is to provide immediate
  // access to pulse positions, times, heights, etc....  For someone who
  // wants to do a more specialized analysis, the constructor can be
  // called with flag=0 so that we do not waste time doing the following
  // calculations.
  if (flag &&  (ch != -1) ) {
    // Find the peaks and measure them using an RMS threshold cut.  Note
    // that you cannot use MeasurePeaksThresh() here since the
    // thresholds have not been set at this point. 
    MeasurePeaksRms();
  } else if (ch == -1){
    fprintf(stderr,"There is no Waveform data for channel %d.\n", ch);
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
Waveplot::Waveplot (int size)
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


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
// Default destructor
Waveplot::~Waveplot(void) {
  //Delete trace;
  if (WaveData) 
    delete[] WaveData;
  if (WaveTime) 
    delete[] WaveTime;

  // If we saved the original waveform because we modified it, delete it.
  if (WaveDataOrig) 
    delete[] WaveDataOrig;

  // If we read in a baseline, delete it.
  if (wf_baseline) 
    delete[] wf_baseline;
  

  // Free up the peak measurement memory.
  CleanUpPeaks();
  
  // Stuff related to plotting the peaks 
  if (pedestal)   delete pedestal;
  if (pedsigma)   delete pedsigma;
  if (thresh)     delete thresh;
  for(int i=0; i<MAX_NUM_PEAKS; i++) {
    if (gpeaks[i])  delete gpeaks[i];
    if (gwidth[i])  delete gwidth[i];
    if (gheight[i]) delete gheight[i];
  }

  if (tracefit)
    delete tracefit;
  if (pulsefit)
    delete pulsefit;
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveplot::InitializePointers() {

  WaveData     = NULL;
  WaveTime     = NULL;
  WaveDataOrig = NULL;
  wf_baseline  = NULL;
  
  // Stuff related to plotting the peaks 
  pedestal = NULL;
  pedsigma = NULL;
  thresh   = NULL;
  for(int i=0; i<MAX_NUM_PEAKS; i++) {
    gpeaks[i] = NULL;
    gwidth[i] = NULL;
    gheight[i] = NULL;
  }

  tracefit    = NULL;
  pulsefit = NULL;
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveplot::InitializeVariables(int no_acq) {
  
  // stuff related to the peaks
  max_num_peaks  = 20;  // Some relatively small value to start so that we
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

  wf_modified     = 0;  // Only original trace data as yet.

  impedance       = 50.0;

  wf_pedestal     = 0;
  wf_pedsigma     = 0;

  run_pedestal    = 0.0;  // Set to zero until we know a better value.

  // Here's how we initialize our variables for now
  wf_size = 1024;
  acq_ch = ch;
  added_waveforms = 1;

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
  saturated_hi   =  1000.0;
  saturated_lo   = -1000.0;
  // We use these to determine if some quantity is saturated or not.
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveplot::SaveOriginalData(void) {

  WaveDataOrig = new double[wf_size];

  for (int i=0; i<wf_size; i++)
    WaveDataOrig[i] = WaveData[i];

}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
int Waveplot::Add(Waveplot * rhs)
// This method adds one waveform to the other. 
{ 
  if (wf_size != rhs->wf_size)
  {
    cerr << "Error : Attempt to add waveforms of different size" << endl;
    return (-1);
  }
  for (int idx = 0 ; idx < wf_size ; idx++)
  {
    double nval = GetBin(idx) + rhs->GetBin(idx);
    SetBin(idx, nval);
  }
  added_waveforms++;
  return (1);
}

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveplot::FindAverageWaveform(void) {
  
  if ( added_waveforms > 1 ) { 
    // Divide by the number of waveforms we have added together
    for (int i=0; i<wf_size; i++) {
      WaveData[i] /= added_waveforms;
    }
  }
  
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
int Waveplot::RestoreOriginalTrace(int start, int size) {
  // 'start' is where in the current waveform we want to start restoring
  // the original trace and 'size' is the number of points to restore
  // If start is 9999 (default), restore the whole trace

  int i;

  // If start == 9999, restore the whole trace
  if (size == 9999) {
    start = 0;
    size = wf_size;
  }

  char txt[1000];
  if (start<0 || (start>wf_size && start!=9999) ) {
    sprintf(txt,"RestoreOriginalTrace: Start is out of range--%d\n",
            start);
    Message(txt);
    return (-1);
  }
  if (start+size>wf_size) {
    sprintf(txt,"RestoreOriginalTrace: Trying to restore beyond waveform\n");
    Message(txt);
    return (-1);
  }

  // Do we have an original trace to restore?
  if ( WaveDataOrig == NULL ) {
    sprintf(txt,"RestoreOriginalTrace: No original waveform to restore.\n");
    Message(txt);
    return (1);
  }
  
  // Restore the trace
  for(i=start; i<start+size; i++) {
    WaveData[i] = WaveDataOrig[i];
  }
  
  return (0);
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
// PmtThreshold should be set to the value used by vmedaq.
void Waveplot::SetThreshold(float PmtThreshold){
  Threshold = PmtThreshold; 
  /*
  if (PmtThreshold > 0){
    Threshold = -PmtThreshold ;
  }else {
    char txt[1000];
    sprintf(txt,"PMT Threshold is %.2f.  It must be a POSITIVE number!!!",
            PmtThreshold);
    Message(txt);
  }
  */
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveplot::PlotWaveform(int npoints, float x_lo, float x_hi, 
                            float y_lo, float y_hi) {

  // First, we make sure that we have a valid number of points to plot.
  // If not, we set the number to be the waveform size written to disk
  // by acqdaq.  
  if (npoints < 1 || npoints > wf_size) 
    npoints = wf_size;

  float x_sc_lo = ( x_lo < -9998 ? WaveTime[0] : x_lo ); 
  float x_sc_hi = ( x_hi < -9998 ? WaveTime[wf_size-1] : x_hi ); 
  float y_sc_lo = ( y_lo == -9999 ? -250.0 : y_lo ); 
  float y_sc_hi = ( y_lo == -9999 ?  30.0 : y_hi ); 
  
  // Make a histogram of our data and plot it in the current canvas
  TH1F *h1, *trace;
  h1 = (TH1F*)gROOT->FindObject("trace"); if (h1) h1->Delete(); h1=0;

  trace = new TH1F("trace", "", npoints-1, WaveTime); 
  for(int i=0;i<npoints-1;i++) {
    trace->Fill(WaveTime[i], WaveData[i]);
  }

  fflush(stdout);
  trace->SetAxisRange(x_sc_lo, x_sc_hi, "X");
  trace->SetAxisRange(y_sc_lo, y_sc_hi, "Y");
  trace->SetLineColor(1);
  trace->SetStats(kFALSE);
  trace->DrawCopy("HIST");

  float px[2],py[2];
  px[0]=0;
  px[1]=2048;
  py[0]=wf_pedestal;
  py[1]=wf_pedestal;
  pedestal = new TGraph(2,px,py);
  pedestal->SetLineColor(8);
  pedestal->Draw("L");
  //py[0]=wf_pedestal - 3*wf_pedsigma;
  //py[1]=wf_pedestal - 3*wf_pedsigma;
  py[0]=saturated_lo;
  py[1]=saturated_lo;
  pedsigma = new TGraph(2,px,py);
  pedsigma->SetLineColor(7);
  py[0] = Threshold;
  py[1] = Threshold;
  thresh = new TGraph(2,px,py);
  thresh->SetLineColor(6);
  thresh->Draw("L");
  
  if (peak_plot) {
    double x[2], y[2], h[2], w[2], p[2], t[2];
    // Just some code to check our peak finding algorithms
    if ( peaks_allocated && num_peaks > 0 ) {
      y[0] = -1e6;
      y[1] =  1e6;
      for(int i=0; i<num_peaks; i++) {
        //printf("ch %d--Peak %d: Peak = %d, TDC = %.2f, w = %.2f, h = %.2f\n", 
        //       ch, i, peaks[i], tdcs[i], width[i], height[i]);
        p[0] = WaveTime[peaks[i]];
        p[1] = WaveTime[peaks[i]];
        // Show the TDC values
        t[0] = tdcs[i];
        t[1] = tdcs[i];
        gpeaks[i] = new TGraph(2, t, y);
        gpeaks[i]->SetLineColor(2);
        gpeaks[i]->Draw("L");
        // Now show the peak widths...
        x[0] = WaveTime[peaks[i]] - width[i]/2.0;
        x[1] = WaveTime[peaks[i]] + width[i]/2.0;
        w[0] = (wf_pedestal - height[i])/2.0;
        w[1] = (wf_pedestal - height[i])/2.0;
        gwidth[i] = new TGraph(2, x, w);
        gwidth[i]->SetLineColor(4);
        gwidth[i]->Draw("L");
        // ..and the peak height...
        h[0] = wf_pedestal - height[i];
        h[1] = wf_pedestal;        
        gheight[i] = new TGraph(2, p, h);
        gheight[i]->SetLineColor(4);
        gheight[i]->Draw("L");
      }
    }
  }
  // Finally, we free up our memory
  //h_dum->Delete();
  //trace->Delete(); 
  // I still need to figure out how to free the memory associated with
  // the traces and the peaks that I plot that does not affect the
  // plot.  JAZ

}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveplot::PlotFit() {
  FitPulse();
  pulsefit->SetLineColor(2); // red
  pulsefit->Draw("SAME LP");
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

//==================PEDESTAL RELATED STUFF=================
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveplot::SetPedRange(float range) {
  // This is a little convoluted, but we must convert the range (in
  // ns) into bins
  int bin_range = Time2Bin(WaveTime[wf_ped_begin]+range) - wf_ped_begin;

  if (bin_range < 0 || bin_range > wf_size) {
    char txt[1000];
    sprintf(txt,"Invalid range for calculating pedestals--%4d.  Not set", 
            bin_range);
    Message(txt);
  } else if (wf_ped_begin+bin_range > wf_size) {
    char txt[1000];
    Message("WARNING--SetPedRange:  Range goes beyond waveform.");
    sprintf(txt,"\twf_bin_range not set to %d",bin_range);
    Message(txt);
  } else {
    wf_ped_range = bin_range;
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveplot::SetPedBegin(float begin)
{
  int begin_bin = Time2Bin(begin);
  if (begin_bin < 0)
  {
    Message("Unable to set a negative pedestal beginning.");
  }
  else if (begin_bin+wf_ped_range > wf_size)
  {
    char txt[1000];
    Message("WARNING--SetPedBegin:  Starting too far into waveform.");
    sprintf(txt,"\twf_ped_begin not set to %d(%d)",begin_bin,wf_ped_range);
    Message(txt);
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
void Waveplot::CalcPedestalRange(void)
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
    //if (abs(WaveData[i]) < 10000.0)
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
void Waveplot::CalcPedestalDynamic(void)
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
  fill_n(spe, 20, -0.05);
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
void Waveplot::SubtractPedestal(void)
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
void Waveplot::AllocatePeaks(void)
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
void Waveplot::SetMaxPeaks(int max_num)
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
void Waveplot::CleanUpPeaks(void)
{
  if (peaks_found)
  {
    delete[] begin_pk;
    delete[] end_pk;
    delete[] sat_pk;
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
int Waveplot::GetNumPeaks(void)
{
  if (peaks_found)
  {
    return num_peaks;
  }
  else
  {
    Message("Peaks not found yet.  Cannot return number of peaks");
    return (-1);
  }   
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
int Waveplot::FindNumPeaks(const char *method, int start_val, int size)
{
  /* This routine finds peaks based on the arguments with which we were
     called. */
  int j=0;
  int p_ctr=(-1);

  char text[500];
  int   min_wid;
  float min_sig = 3.0;
  float thresh  = 140.0;  // Initialize so we don't get compile warning
  double baseline;

#define INTERP 0

#if INTERP
  int   slope_pre, slope_post;
#endif

  int   start, tmp_ctr=0;

  // First, we allocate our *begin_pk and *end_pk pointers
  begin_pk  = new int[max_num_peaks];
  end_pk    = new int[max_num_peaks];
  sat_pk    = new int[max_num_peaks];
  spikes    = new int[max_num_peaks];
  
  // Then, figure out what method we are supposed to use to find the
  // peaks and set some local variables accordingly.
  if ( strncmp(method,"thr",3) == 0 ){ // Use a constant threshold
    // I simply require that 1 bin be above threshold
    min_wid = 1;
    thresh  = abs(Threshold); // We are looking at absolute values
    baseline = 0;
  } else if ( strncmp(method,"eff",3) == 0 ) { // Use effective threshold
    // I simply require that 1 bin be above threshold
    min_wid = 1;
    thresh  = abs(Threshold); // Looking at absolute values above run_pedestal
    baseline = run_pedestal - wf_pedshift*dc2mv;
  } else if ( strncmp(method,"rms",3) == 0 ) { // Use rms-based thresh
    // I require that 3 consecutive bins be more than 3*pedsigma below
    // the pedestal value. 
    min_wid = 3;
    thresh  = min_sig*wf_pedsigma;
    baseline = wf_pedestal;
  } else {
    sprintf(text,"FindNumPeaks: Invalid method(%s).  Use thr, eff or rms",
            method);
    return (-1);
  }

  // Now we determine what section of the trace we are going to look at.
  if (start_val == 9999) { // Use the full trace
    start = 1;
    size = wf_size-1;
  } else {
    start = ( start_val<1 ? 1 : start_val );
    if(start+size > wf_size) // Don't go beyone the end of the trace
      size = wf_size-start;
  }

  // To make sure that we are not starting in the middle of a peak, we
  // increment through the waveform until we are below threshold.
  //while ( baseline-WaveData[start+tmp_ctr] > thresh && 
  while ( abs(WaveData[start+tmp_ctr]-baseline) > thresh && 
          start+tmp_ctr < wf_size) 
    tmp_ctr++;
  
  for(int i=start+tmp_ctr+1; i<start+size; i++){
    if( (abs(WaveData[i]-baseline) > thresh ) ){
      j++;
      if(j == min_wid) {    // Do only once for each peak found
        p_ctr++;            // Increment the peak counter
        begin_pk[p_ctr] = i-(min_wid-1);  // Mark first bin of peak
        end_pk[p_ctr] = i; // Initialize last bin included in peak
        spikes[p_ctr] = 0;
        // Initialize and check for saturation here, also check below
        sat_pk[p_ctr] = (WaveData[i] < saturated_lo) ? SAT : NOT_SAT; 
        if(p_ctr+1 == max_num_peaks) {
          sprintf(text,"Found maximum number of peaks %d", p_ctr+1);
          Message(text);
          i=wf_size;
        }
      } else if(j > min_wid) { 
        
        // Are we saturated yet?
        if ( WaveData[i] < saturated_lo ) sat_pk[p_ctr] = SAT;
#if INTERP
        /* Now we find the slope of the trace before and after the
           point we are considering.  If the before slope is positive
           and the after slope is negative, we are in a valley, thus
           we have found a new peak.  Remember, we are dealing with
           negative going pulses. */
        
        // Slope before point
        slope_pre  = ( (WaveData[i-1] < WaveData[i]) ? 1 : 0 );
        // Slope after point
        slope_post = ( (WaveData[i+1] < WaveData[i]) ? -1 : 0 );
        // Check if we are at the bottom of the valley
        if((slope_pre == 1) && (slope_post == -1)){
          // Just keep track of the spikeyness (sp?) for now
          spikes[p_ctr]++;
        }
#endif
        end_pk[p_ctr] = i; // Set last bin included in peak
      }
    } else {
      j = 0;  // Reset bin counter at each bin not meeting requirement
    }
  }
  num_peaks = p_ctr+1;  // We start at zero
  
  begin_pk[num_peaks] = wf_size;  // Need this to measure last peak correctly

  peaks_found = 1;
  return num_peaks;
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveplot::MeasurePeaksRms(int start, int size) {
  /* This routine measures peaks based on the RMS of the trace */

  float cut_bound = 3.0;

  // Before we do anything, we need to clean up any previous peak
  // measurements so that we don't run into pointer problems.
  CleanUpPeaks();

  // Then we find how many peaks we have and their location in the
  // waveform.  Note that the position of the tdcs is determined by the
  // first bin where the trace is greater than n*pedsigma from the
  // pedestal value.  We will calculate a more reasonable determination 
  // of the peak position later.
  if (start+size > wf_size) size = wf_size-start;
  FindNumPeaks("rms", start, size);
  
  // Allocate memory for the peak measurements
  AllocatePeaks();
  
  for(int i=0; i<num_peaks; i++) {
    // First, initialize the pulse characteristics to zero
    peaks[i] = 0;
    tdcs[i] = charge[i] = width[i] = height[i] = 0;
    
    // Find TDC value by interpolating across threshold crossing
    float thresh = wf_pedestal - cut_bound*wf_pedsigma;
    tdcs[i] = FindInterpolatedTime(thresh, begin_pk[i]-1, 1);

    // Find the peak location
    peaks[i]  = GetMinBin(begin_pk[i], end_pk[i]-begin_pk[i] );
    // Extract the pulse height from the peak location.  
    height[i] = wf_pedestal - WaveData[peaks[i]];
    width[i] = CalculateWidth(i, -0.5*height[i]);
    // Now for the charge contained in the pulse.  The 'PEAK_OFFSET' and
    // 'PEAK_LENGTH' are arbitrary at this point.
    charge[i] = Integrate( peaks[i]+(int)PEAK_OFFSET, PEAK_LENGTH);
    charge[i] *= -1.0;
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveplot::MeasurePeaksThresh(float start, float size, int th_type) {
  /* This routine measures peaks using a constant threshold algorithm  */

  // Before we do anything, we need to clean up any previous peak
  // measurements so that we don't run into pointer problems.
  CleanUpPeaks();

  // All the algorithms below work in bin values so we need to convert
  // the arguments from ns to bin values
  int start_bin = Time2Bin(start);
  int size_bin  = Time2Bin(start+size) - start_bin;

  // Then we find how many peaks we have and their location in the
  // waveform.
  if (start_bin+size_bin > wf_size) size_bin = wf_size-start_bin;
  FindNumPeaks("thresh", start_bin, size_bin);
  
  // Now allocate pointers to the peak position, width, height and charge 
  AllocatePeaks();

  for(int i=0; i<num_peaks; i++) {

    peaks[i] = 0;
    tdcs[i] = charge[i] = width[i] = height[i] = 0.0;

    // Find TDC value by interpolating across threshold crossing
    if ( th_type == CONSTANT ) {
      tdcs[i]   = FindInterpolatedTime(Threshold, begin_pk[i]-1, 1);
    } else if ( th_type == CFD_ELEC ) {
      tdcs[i]   = FindCFDElecTime(0.0, begin_pk[i]-1, 100);
    } else if ( th_type == CFD_SIMPLE ) {
      tdcs[i]   = FindCFDSimpTime(i);        
    }
    // Find the peak location
    //peaks[i]  = (double) GetMinBin(begin_pk[i], end_pk[i]-begin_pk[i] );
    peaks[i]  = GetMinBin(begin_pk[i], end_pk[i]-begin_pk[i]);
    // Extract the pulse height from the peak location.
    //height[i] = wf_pedestal - WaveData[peaks[i]];
    height[i] = -WaveData[peaks[i]];
    width[i] = CalculateWidth(i, -0.5*height[i]);
    // Now for the charge contained in the pulse.  The 'PEAK_OFFSET' and
    // 'PEAK_LENGTH' are arbitrary at this point.
    charge[i] = Integrate( WaveTime[peaks[i]]+(int)PEAK_OFFSET, PEAK_LENGTH);
    charge[i] *= -1.0;
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveplot::FindPeaks(float start, float size) {
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
  sat_pk    = new int[max_num_peaks];
  spikes    = new int[max_num_peaks];
  // Step through trace until we are below threshold
  while ((WaveData[pos] < Threshold) && (pos < wf_size))
    pos++;
  for (int i = pos + 1; i < start_bin + size_bin; i++) {
    if (WaveData[i] < Threshold) {
      peak_bins++;
      if (peak_bins == min_wid) { // new peak detected
        if (pk_ctr == max_num_peaks) {
          Message("Maximum number of peaks exceeded");
          break;
        }
        begin_pk[pk_ctr] = i - (min_wid - 1);
        spikes[pk_ctr] = 0;
        end_pk[pk_ctr] = 0;
        sat_pk[pk_ctr] = (WaveData[i] < saturated_lo) ? SAT : NOT_SAT;
        pk_ctr++;
      } else if (peak_bins > min_wid) {
        if (WaveData[i] < saturated_lo)
          sat_pk[pk_ctr-1] = SAT;
        // each "bump" counts as a separate peak
        int grad = 1;
        for (int k = 0; k < 3; k++) {
          if (WaveData[i-k] < WaveData[i-(k+1)])
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
  for(int i = 0; i < num_peaks; i++) {
    peaks[i]  = GetMinBin(begin_pk[i], end_pk[i]-begin_pk[i]);
    height[i] = WaveData[peaks[i]];
    width[i]  = CalculateWidth(i, 0.5*height[i]);
    charge[i] = -Integrate( WaveTime[peaks[i]]+(int)PEAK_OFFSET, PEAK_LENGTH);
  }
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
    f = par[0]*TMath::Power((xx-par[1])/par[2],par[3]); // power law
    f *= TMath::Exp(-(xx-par[1])/par[2]); // exponential
    f *= TMath::Power(TMath::E()/par[3],par[3]); // normalization
  }
  return f;
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveplot::FitPulse() {
  if (tracefit)
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
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
void Waveplot::FindTdc(int pk_num, int th_type) {
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
    case PCONSTANT: // TODO - only works for pk_num = 0
      tdcs[pk_num] = pulsefit->GetX(Threshold,pulse_start,WaveTime[peaks[0]]);
      break;
    case PCFD:
      double cfdval = cfds_frac * pulsepars[0];
      tdcs[pk_num] = pulsefit->GetX(cfdval,pulse_start,WaveTime[peaks[0]]);
      break;
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
double Waveplot::CalculateWidth(int pk, float level) {
  /* This routine calculates the width of a peak */

  // The algorithm is this--Start at the peak.  Find the time before the
  // peak, Tb, where the trace drops to 'level'.  Do the same
  // after the peak, Ta.  Width = Ta -Tb.

  double Ta=0, Tb=0;
  double eps = 1e-3;;

  // We want to start a couple of bins before the start of the peak, but
  // want to insure that we don't start before the beginning of the trace
  int   lo = 10;
  int   hi = 15;
  int   start = (begin_pk[pk] < lo ? 0 : begin_pk[pk]-lo);
  int   end = (end_pk[pk] - wf_size < hi ? end_pk[pk]+hi : end_pk[pk]);
  int   peak = peaks[pk];

  // Find rising 'level' crossing time
  for (int i = peak; i>start; i--) {
    if ( (WaveData[i]-wf_pedestal > level) && 
         (WaveData[i+1]-wf_pedestal < level) ) {
      Tb = FindInterpolatedTime(level-wf_pedestal, i, 1);
      i = start;
    }
  }

  // Find falling 'level' crossing time
  for (int i = peak; i<end; i++) {
    if ( (WaveData[i]-wf_pedestal < level) && 
         (WaveData[i+1]-wf_pedestal > level) ) {
      Ta = FindInterpolatedTime(level-wf_pedestal, i, 1);
      i = end;
    }
  }

  if ( Ta<eps || Tb<eps ) // Unable to find level values
    return (1.0);
  else 
    return (Ta-Tb);
}

////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
double Waveplot::FindInterpolatedTime(float thresh, int idx, int size){
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
    /*printf("%2d(%d) -- %.2f, %.2f  -- %.2f, %.2f, %.2f    %.2f\n", ch, 
           num_peaks, WaveTime[idx], WaveTime[idx+1], thresh, lval, 
           hval, time);*/
    return time;
  }
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
double Waveplot::FindCFDElecTime(float thresh, int idx, int size){
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
double Waveplot::FindCFDSimpTime(int pk_num){
  // idx is the last bin which is below threshold.  lval is the value of
  // the last bin below threshold.  hval is the value of the first value
  // above threshold and thresh is the desired threshold.

  if( pk_num<0 || pk_num>MAX_NUM_PEAKS ) { return(WaveTime[wf_size]); }

  // Determine the threshold for finding the time. Use 25% of the
  // average of the bin of the highest peak and the two bins next to
  // it.
  int idx = GetMinBin(begin_pk[pk_num], end_pk[pk_num]-begin_pk[pk_num]);
  
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
int Waveplot::Time2Bin(float t_ns){
  // Given a time in ns, find the bin most closely corresponding to that time
  for (int i=0; i<wf_size; i++) 
    if (WaveTime[i] > t_ns) 
      return (i-1); 

  printf("--%.2f", t_ns);
  Message("Did not find a bin corresponding to the given time.");
  return (-1);
}
////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////
int Waveplot::GetSpikes(int i = 0) {
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
double Waveplot::GetTdcs(int i = 0) {
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
double Waveplot::GetCharge(int i = 0) {
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
double Waveplot::GetHeight(int i = 0) {
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
double Waveplot::GetWidth(int i = 0) {
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

void Waveplot::Message(const char *s) {
  cerr << s << endl;
}

//===============STUFF FROM DAN's CLASS======================

// --------------------------------------------------------------------------
// This method sets the value of a given bin in mV.
// --------------------------------------------------------------------------
double Waveplot::SetBin(int idx, double val) {
  if ((idx < 0) || (wf_size <= idx)) { return(ERRVAL); }
  else { WaveData[idx] = val; return(0); }
}        

// --------------------------------------------------------------------------
// This method returns the value of a given bin in mV.
// --------------------------------------------------------------------------
double Waveplot::GetBin(int idx) {
  if ((idx < 0) || (wf_size <= idx)) { return(ERRVAL); }
  else { return(WaveData[idx]); }
}  

// --------------------------------------------------------------------------
// This method returns the Time value of a given bin in ns.
// --------------------------------------------------------------------------
double Waveplot::GetBinTime(int idx) {
  if ((idx < 0) || (wf_size <= idx)) { return(ERRVAL); }
  else { return(WaveTime[idx]); }
}  

// --------------------------------------------------------------------------
// This method returns the bin with the largest DC value.
// --------------------------------------------------------------------------
int Waveplot::GetMaxBin(int lo = 0, int size = -1) {
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
double Waveplot::GetMaxBinTime(int lo = 0, int size = -1) {
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
double Waveplot::GetMaxVal(int lo = 0, int size = -1) {
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
int Waveplot::GetMinBin(int lo = 0, int size = -1) {
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
double Waveplot::GetMinBinTime(int lo = 0, int size = -1) {
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
double Waveplot::GetMinVal(int lo = 0, int size = -1) {
  if(size < 0){ size = wf_size - lo; }
  if((lo < 0) || (wf_size < lo + size)){ return(ERRVAL); }
  double minval = GetBin(lo);
  for(int idx = lo + 1 ; idx < lo + size ; idx++){
    if(GetBin(idx) < minval){ minval = GetBin(idx); }
  }
  return(minval);
}

// --------------------------------------------------------------------------
// This method returns the largest (negative) value in mV in a time window
// --------------------------------------------------------------------------
double Waveplot::GetPeakValue(float lo = 0.0, float size = -1.0) {
  if(size < 0.0){ size = WaveTime[wf_size-1] - lo; }
  if((lo < 0) || (WaveTime[wf_size-1] < lo + size)){ return(ERRVAL); }
  
  int lo_bin = Time2Bin(lo);
  int hi_bin = Time2Bin(lo+size);
  // Set the start point to the data value of the "lo" time bin
  double minval = GetBin(lo_bin);
  
  for(int i = lo_bin ; i < hi_bin; i++){
    if(GetBin(i) < minval){ minval = GetBin(i); }
  }
  return(minval);
}

void Waveplot::Rescale(double factor = 1) {
  if(factor == 1){ return; }
  for(int idx = 0 ; idx < wf_size ; idx++){ WaveData[idx] *= factor; }
}

double Waveplot::Integrate(float lo = 0, float size = -1) {
  // First, we need to convert our values of lo and size into bin values.
  int lo_bin   = Time2Bin(lo);
  int size_bin = Time2Bin(lo + size) - lo_bin;

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

