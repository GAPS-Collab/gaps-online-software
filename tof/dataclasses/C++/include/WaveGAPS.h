#ifndef GAPS_WAVEFORM_H_INCLUDED
#define GAPS_WAVEFORM_H_INCLUDED

/**********************************
 * Code from Jeff's original 
 * waveform library
 */ 


#include <vector>

#define MAX_NUM_PEAKS   50  
#define ERRVAL		(999999999)

// These two #defines give the necessary information for integrating to
// find the charge in the peak (these should be in ns)
#define PEAK_OFFSET  -4
#define PEAK_LENGTH  12

namespace GAPS { 

// Types of Thresholds to use in determining timing
enum THRTYPE { CONSTANT, CFD_ELEC, CFD_SIMPLE, PCONSTANT, PCFD };

// I moved these to cfde_frac and cfde_offset below and made routines
// so that they can be set without recompiling.
//#define CFD_FRACTION  0.1
// CFD_OFFSET should be an integer so we don't have to interpolate our
// FADC data.  
//#define CFD_OFFSET    5.0

// Note:  The NORMAL entry is there to make it easier to fill the
// channel_status flag in the PMT0 bank.  A value of zero there
// indicates "no stacq data", so, it will be filled with 1 if the data
// is valid and not saturated.  Otherwise, it will be filled with zero
// if the data is not valid and with the saturated flag value if it is
// valid.  
enum SAT_FLAGS { NOT_SAT, NORMAL, SAT, NO_FIT, FIT_GOOD, FIT_BAD};

double Pulse(double *x, double *par);

class Waveform {

public:

  Waveform (double *data, double *time, int chnl, int flag = 0);
  // Constructor 'flag' is set by default unless the waveform is
  // constructed with a call to the contrary.  Thus, the default
  // behavior is to provide immediate access to pulse positions,
  // times, heights, etc....  For someone who wants to do a more
  // specialized analysis, the constructor can be called with flag=0
  // so that we do not waste time doing the ped and peak calculations.

  Waveform (int size);
  ~Waveform (void);

  // MEMBER FUNCTIONS

  void SetWave(std::vector<double>);
  void SetTime(std::vector<double>);

  // Stuff related to the actual data
  void    SetThreshold(float PmtThreshold);
  int     GetWaveSize(void){return wf_size;}
  double  SetBin(int idx, double val);
  double  GetBin(int idx);
  double  GetBinTime(int idx);
  int     GetBinDC(int idx);
  int     GetMaxBin(int lo, int size);
  double  GetMaxBinTime(int lo, int size);
  double  GetMaxVal(int lo, int size);
  int     GetMinBin(int lo, int size);
  double  GetMinBinTime(int lo, int size);
  double  GetMinVal(int lo, int size);
  double  GetPeakValue(float lo, float size);
  void    Rescale(double factor);
  double  Integrate(float lo, float size);

  // Stuff related to the pedestals
  void    SetPedestal(double pedestal){wf_pedestal = pedestal;}
  void    SetRunPedestal(double runped){run_pedestal = runped;}
  void    SetPedRange(float range);
  void    SetPedBegin(float begin);
  int     GetPedRange(void){return wf_ped_range;}
  int     GetPedBegin(void){return wf_ped_begin;}
  double  GetPedestal(void){return wf_pedestal;}
  double  GetPedsigma(void){return wf_pedsigma;}
  void    CalcPedestalRange(void);
  void    CalcPedestalDynamic(void);
  void    SubtractPedestal(void);

  // Stuff related to the peaks
  void    SetMaxPeaks(int max_num);
  int     GetMaxPeaks(void){return max_num_peaks;}
  void    CleanUpPeaks(void);
  int     GetNumPeaks(void);
  void    SetCFDSFraction(double fraction) {cfds_frac = fraction;}
  void    SetCFDEFraction(double fraction) {cfde_frac = fraction;}
  void    SetCFDEOffset(int offset) {cfde_offset = offset;}
  void    FindPeaks(float start, float size);
  void    FindTdc(int pk_num, int th_type = CFD_SIMPLE);
  int     GetSpikes(int i);
  double  GetTdcs(int i);
  double  GetCharge(int i);
  double  GetHeight(int i);
  double  GetWidth(int i);

  // Stuff related to the pulses
  double  GetPulsepars(int i) {return pulsepars[i];}
  double  GetPulsechi2() {return pulsechi2;}
  double  GetNDF() {return ndf;}
  void    FitPulse(void);

  // Stuff related to converting from mV to DC and vice versa
  double  GetNsPerBin(void)  {return bin_ns;}
  double  GetOffset(void)    {return offset;}
  double  GetTimingCorr(void){return timing_corr;}
  double  GetImpedance(void) {return impedance;}
  void    SetImpedance(double val) {if (val>0) impedance = val;}

private:

  // DATA MEMBERS

  int     ch;                        // STACEE channel we are working with
  int     runno;                     // Run Number
  float   Threshold;                 // PMT Threshold in DC (for now...)

  // Stuff related to the actual data
  int     wf_size;                   // How much data was written to disk
  double  base;                      // 0 mV baseline in FADC counts
  double  *WaveData;                 // 'in memory' waveform data
  double  *WaveTime;                 // 'in memory' waveform times
  double  *wf_baseline;              // To subtract reference baseline (mV)

  // Stuff related to the pedestals
  int     wf_ped_range;              // How much data used for ped calcs
  int     wf_ped_begin;              // Start of trace for ped calcs
  double  wf_pedestal;               // Pedestal value
  double  run_pedestal;              // Average pedestal for run
  double  wf_pedsigma;               // Deviation of pedestal distribution
  double  wf_pedshift;               // Amount pedestal shifts (in DC) just
                                     // because we connect inputs to FADCs

  // Stuff related to the peaks
  int     max_num_peaks;
  int     num_peaks;
  int     peaks_found;     // Have we found the peaks yet?
  int     peaks_allocated; // Have we measured the peak properties yet?
  int     peak_plot;       // Show peak positions when plotting?
  int     large_peaks;     // Extrapolate saturated peaks?
  int     *begin_pk;       // First bin included in peak
  int     *end_pk;         // Last bin included in peak
  int     *spikes;         // How smooth is the peak
  int     *peaks;          // Bin values of the actual peak positions
  int     cfde_offset;     // Offset for electronic CFD (in bin values)
  double  cfds_frac;       // Fraction of peak ave for simple CFD 
  double  cfde_frac;       // Fraction of trace for electronic CFD 
  double  *tdcs;           // Something similar to what a TDC would give
  double  *width;
  double  *height;
  double  *charge;

  double  pulsepars[4];      // Pulse parameters
  double  pulsechi2;
  int     ndf;
  float   pulse_start;

  // Converting from FADC data to mV, charge, etc...
  int    acq_ch;
  double dc2mv;
  double offset;
  double bin_ns;
  double impedance;
  double saturated_lo;
  double saturated_hi;

  // Since the trace in the Dig0 bank is stored in integer increments,
  // but we want times in floats, we have to correct for the round off
  // error (which is written in the Dig0 bank).
  double timing_corr;

  // MEMBER FUNCTIONS
  void    InitializePointers();
  void    InitializeVariables(int no_acq);
  void    Message(const char *s);           // Print out messages as needed
  // Stuff related to the peaks
  void    AllocatePeaks();
  double  FindInterpolatedTime(float thresh, int idx, int size);
  double  FindCFDElecTime(float thresh, int idx, int size);
  double  FindCFDSimpTime(int pk_num);
  int     Time2Bin(float t_ns);

};

} // end of namespace GAPS

#endif
