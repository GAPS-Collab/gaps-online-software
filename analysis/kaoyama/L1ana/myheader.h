// ver.GAPS tof
// functions to get ttree parameters
#include <iostream>
#include <fstream>
#include <vector>

#include "TFile.h"
#include "TChain.h"
#include "TTree.h"
#include "TROOT.h"
#include "TStyle.h"

#include "TMath.h"
#include "TRandom3.h"

#include "TF1.h"
#include "TGraphErrors.h"
#include "TGraph2DErrors.h"
#include "TH1D.h"
#include "TH2D.h"
#include "TH3D.h"
#include "TCanvas.h"
#include "TLegend.h"
#include "TVector3.h"
#include "TMarker3DBox.h"

#include "TSystem.h"


#include "/home/kaoyama/software/SimpleDet/build/install/gaps-v2.1.0/include/gaps/CRawHeader.hh"
#include "/home/kaoyama/software/SimpleDet/build/install/gaps-v2.1.0/include/gaps/CRawTof.hh"
#include "/home/kaoyama/software/SimpleDet/build/install/gaps-v2.1.0/include/gaps/CRawTrk.hh"

// ** Load tree **
TString inFileName;
TFile *fin;
TTree *tree;
// Declaration of leaf types
Crane::Calibration::CRawHeader *Header;
Crane::Calibration::CRawTof *Tof;
Crane::Calibration::CRawTrk *Trk;

// **** for Plot()/Next() ****
TH1D *hWaveform_a[100];
TH1D *hWaveform_b[100];
TCanvas *cWaveform;
Int_t npaddle=0;
const Int_t    nbin   = 1024;
const Double_t tw_min = 0., tw_max = 500.;
const Double_t ymin=-10., ymax=80.;

const Int_t nrb=50;
const Int_t nch=8;
const Int_t npd=200;
const Int_t nRebin=1;
const Int_t PED_STARTBIN=1, PED_STOPBIN=50;
const Int_t TOT_STARTBIN=1250, TOT_STOPBIN=1300;
Int_t CurrentEventNumber=0;
Int_t NextEventNumber=0;
// ********************

// **** for Loop() ****
void Loop(Int_t RunNumber, Int_t StartFileNumber, Int_t EndFileNumber, Bool_t flagDisp);
TH1D *h_npaddle;
TH1D *h_rb_id;
TH1D *h_pd_id;
// per paddle
TH1D *h_hitdt_ab[npd];
TH1D *h_hittime_a[npd];
TH1D *h_hittime_b[npd];  
TH1D *h_hittime_ab[npd];  
TH1D *h_hittime_ab_cut[npd]; 
TH1D *h_charge_a[npd];
TH1D *h_charge_b[npd];
TH2D *h_charge_ab[npd];
TH2D *h_dt_charge_a[npd];
TH2D *h_dt_charge_b[npd];
TH1D *h_hitpos_ab[npd];

// conbination between paddles
TH1D *h_hitdt_pd;
TH1D *h_tof_length;
TH2D *h_tof_length_hitdt;
TH1D *h_phase_ch9[npd];

// conbination between paddles
TH1D *h_track_theta;
TH1D *h_track_theta_mod;
TH1D *h_track_phi;
TH1D *h_track_chi2;
TH1D *h_track_x0;
TH1D *h_track_y0;
TH1D *h_track_z0;

TH1D *h_track_theta_cut1;
TH1D *h_track_theta_mod_cut1;
TH1D *h_track_phi_cut1;
TH1D *h_track_chi2_cut1;
TH1D *h_track_x0_cut1;
TH1D *h_track_y0_cut1;
TH1D *h_track_z0_cut1;


void _DefineHistLoop1()
{
  h_npaddle = new TH1D("h_npaddle","h_npaddle; #paddle, #event",101,-0.5,100.5);
  h_rb_id = new TH1D("h_rb_id","h_rb_id; rb id; #event",51,-0.5,50.5);
  h_pd_id = new TH1D("h_pd_id","h_pd_id; paddle id; #event",201,-0.5,200.5);
  for(Int_t ipd=0;ipd<npd;++ipd){
    h_hitdt_ab[ipd]    = new TH1D(Form("h_hitdt_ab_pd%d", ipd),Form("h_hitdt_ab_pd%d; time difference tb-ta (ns); #event", ipd), 100, (-50.-0.5)*500./1024., (50-0.5)*500./1024.);
    h_hittime_a[ipd]   = new TH1D(Form("h_hittime_a_pd%d", ipd),Form("h_hittime_a_pd%d; MPPC hit timing (ns); #event", ipd), 1024, 0, 500);
    h_hittime_b[ipd]   = new TH1D(Form("h_hittime_b_pd%d", ipd),Form("h_hittime_b_pd%d; MPPC hit timing (ns); #event", ipd), 1024, 0, 500);
    h_hittime_ab[ipd]   = new TH1D(Form("h_hittime_ab_pd%d", ipd),Form("h_hittime_ab_pd%d; MPPC hit timing (ns); #event", ipd), 1024, 0, 500);
    h_charge_a[ipd]    = new TH1D(Form("h_charge_a_pd%d", ipd),Form("h_charge_a_pd%d; integrated charge (voltage?); #event", ipd), 5000, 0, 50000);
    h_charge_b[ipd]    = new TH1D(Form("h_charge_b_pd%d", ipd),Form("h_charge_b_pd%d; integrated charge (voltage?); #event", ipd), 5000, 0, 50000);
    h_charge_ab[ipd]   = new TH2D(Form("h_charge_ab_pd%d", ipd),Form("h_charge_ab_pd%d; integrated charge (voltage?); integrated charge (voltage?);", ipd), 500, 0, 50000 ,500, 0, 50000);    
    h_dt_charge_a[ipd] = new TH2D(Form("h_dt_charge_a_pd%d", ipd),Form("h_dt_charge_a_pd%d; time difference tb-ta (ns); integrated charge (voltage?)", ipd), 100, (-50.-0.5)*500./1024., (50-0.5)*500./1024., 500, 0, 50000);
    h_dt_charge_b[ipd] = new TH2D(Form("h_dt_charge_b_pd%d", ipd),Form("h_dt_charge_b_pd%d; time difference tb-ta (ns); integrated charge (voltage?)", ipd), 100, (-50.-0.5)*500./1024., (50-0.5)*500./1024., 500, 0, 50000);
    h_hitpos_ab[ipd]   = new TH1D(Form("h_hitpos_ab_pd%d", ipd),Form("h_hitpos_ab_pd%d", ipd), 50, -2., 2.);

    h_hittime_ab_cut[ipd]   = new TH1D(Form("h_hittime_ab_cut_pd%d", ipd),Form("h_hittime_ab_cut_pd%d; MPPC hit timing (ns); #event", ipd), 1024, 0, 500);
    h_phase_ch9[ipd] = new TH1D(Form("h_phase_ch9_pd%d", ipd),Form("h_phase_ch9_pd%d; phase ch9 (ns); #event", ipd), 2000, -1000, 1000);

  }


  h_hitdt_pd = new TH1D("h_hitdt_pd","h_hitdt_pd; dt between paddle; #event",401,-100.25,100.25);
  // h_hitdt_pd = new TH1D("h_hitdt_pd","h_hitdt_pd; dt between paddle; #event",1000,-100,100);
  h_tof_length = new TH1D("h_tof_length","h_tof_length; dt between paddle; #event",100,0,1000);
  h_tof_length_hitdt = new TH2D("h_tof_length_hitdt","h_tof_length_hitdt; dt between paddle; #event",100,0,500, 401,-100.25,100.25);



  // tracking
  h_track_theta = new TH1D("h_track_theta","h_track_theta; theta; #event",100,0,180);
  h_track_theta_mod = new TH1D("h_track_theta_mod","h_track_theta_mod; cos(#theta); #event",100,0.,1.);
  h_track_phi = new TH1D("h_track_phi","h_track_phi; phi; #event",100,-180,180);
  h_track_chi2 = new TH1D("h_track_chi2","h_track_chi2; chi2; #event",1000,0,100);
  h_track_x0 = new TH1D("h_track_x0","h_track_x0; x0; #event",1000,-2000,2000);
  h_track_y0 = new TH1D("h_track_y0","h_track_y0; y0; #event",1000,-2000,2000);
  h_track_z0 = new TH1D("h_track_z0","h_track_z0; z0; #event",100,-50,250);
  
  h_track_theta_cut1 = new TH1D("h_track_theta_cut1","h_track_theta_cut1; theta; #event",100,0,180);
  h_track_theta_mod_cut1 = new TH1D("h_track_theta_mod_cut1","h_track_theta_mod_cut1; cos(#theta); #event/sin(theta)",100,0,1.);
  h_track_phi_cut1 = new TH1D("h_track_phi_cut1","h_track_phi_cut1; phi; #event",100,-180,180);
  h_track_chi2_cut1 = new TH1D("h_track_chi2_cut1","h_track_chi2_cut1; chi2; #event",1000,0,100);
  h_track_x0_cut1 = new TH1D("h_track_x0_cut1","h_track_x0_cut1; x0; #event",1000,-2000,2000);
  h_track_y0_cut1 = new TH1D("h_track_y0_cut1","h_track_y0_cut1; y0; #event",1000,-2000,2000);
  h_track_z0_cut1 = new TH1D("h_track_z0_cut1","h_track_z0_cut1; z0; #event",100,-50,250);
  h_track_theta_cut1 -> SetLineColor(2);
  h_track_theta_mod_cut1 -> SetLineColor(2);
  h_track_phi_cut1 -> SetLineColor(2);
  h_track_chi2_cut1 -> SetLineColor(2);
  h_track_x0_cut1 -> SetLineColor(2);
  h_track_y0_cut1 -> SetLineColor(2);
  h_track_z0_cut1 -> SetLineColor(2);

}
// ********************


void LoadRootFile(Int_t RunNumber, Int_t FileNumber)
{
  TString cmd = Form("ls /home/kaoyama/data/Antarctic_2024/processed/L1/%d/Run%d_%d.*_*.root", RunNumber, RunNumber, FileNumber);
  TString fileName = gSystem->GetFromPipe(cmd.Data());
  inFileName = fileName.Strip(TString::kBoth); // 前後の空白や改行を削除

  fin = TFile::Open(inFileName);
  delete tree;
  tree = (TTree*)fin->Get("TreeRaw");
  tree -> SetBranchAddress("Header", &Header);
  tree -> SetBranchAddress("Tof", &Tof);
  tree -> SetBranchAddress("Trk", &Trk);
  cout << "Load waveform file: " << inFileName << endl;

}

Int_t Plot(Int_t ievent, Bool_t is_Disp, Bool_t is_3D);
void Next()
{
  NextEventNumber = CurrentEventNumber+1;
  cout << "plot event" << NextEventNumber << endl;
  Plot(NextEventNumber,true,false);
}

Int_t Plot(Int_t ievent, Bool_t is_Disp=true, Bool_t is_3D=true)// need to modify pedestal errors
{
  // Make Waveform
  CurrentEventNumber = ievent;
  tree->GetEntry(ievent);

  if(Tof->wfs.voltages.empty()){
    // cout << "no waveform data" << endl;
    return -1;
  }
  npaddle = Tof->wfs.voltages.size();
  if(npaddle>50){
    cout << "too many waveforms, npaddle = " << npaddle << endl;
    return -1;
  }
  else if(npaddle%2==1){
    cout << "number of paddle is odd" << npaddle << endl;
  }
  npaddle = npaddle/2;

  for(Int_t ipaddle=0;ipaddle<npaddle;ipaddle+=1){
    delete hWaveform_a[ipaddle];
    hWaveform_a[ipaddle] = new TH1D(Form("hWaveform_a_pd%d",ipaddle),Form("hWaveform_a_pd%d;Time (ns);ADC counts",Tof->wfs.paddle_id[ipaddle*2]),nbin,tw_min, tw_max);
    delete hWaveform_b[ipaddle];
    hWaveform_b[ipaddle] = new TH1D(Form("hWaveform_b_pd%d",ipaddle),Form("hWaveform_b_pd%d;Time (ns);ADC counts",Tof->wfs.paddle_id[ipaddle*2+1]),nbin,tw_min, tw_max);
    
    for(Int_t ibin=0;ibin<nbin;++ibin){
        hWaveform_a[ipaddle] -> SetBinContent(ibin+1,Tof->wfs.voltages[ipaddle*2][ibin]);
        hWaveform_b[ipaddle] -> SetBinContent(ibin+1,Tof->wfs.voltages[ipaddle*2+1][ibin]);
        hWaveform_a[ipaddle] -> SetBinError(ibin+1,1.);
        hWaveform_b[ipaddle] -> SetBinError(ibin+1,1.);
    }
  }

  if(is_Disp){
    if(!gROOT->FindObject("cWaveform")){
      delete cWaveform;
      cWaveform = new TCanvas("cWaveform", "cWaveform", 800, 600);
      cWaveform -> Divide(3,3);
    }
    for(Int_t ipaddle=0;ipaddle<npaddle;++ipaddle){
      cWaveform -> cd(ipaddle+1);
      hWaveform_a[ipaddle] -> SetLineColor(1);
      hWaveform_b[ipaddle] -> SetLineColor(2);
      hWaveform_a[ipaddle] -> GetYaxis() -> SetRangeUser(ymin, ymax);
      hWaveform_a[ipaddle] -> Draw("hist");
      hWaveform_b[ipaddle] -> Draw("same hist");
    }
  }

  return 1;
}
