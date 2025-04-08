// ver.GAPS merge
// functions to get ttree parameters
#include <iostream>
#include <fstream>
#include <vector>
#include <time.h>

// for ROOT objects
#include "TCanvas.h"
#include "TH1D.h"
#include "TH2D.h"
#include "TH3D.h"
#include "TGraphErrors.h"
#include "TGraph2DErrors.h"
#include "TLegend.h"
#include "TPaveText.h"
#include "TPolyLine3D.h"
#include "TMarker3DBox.h"

#include "TFile.h"
#include "TTree.h"
#include "TROOT.h"
#include "TStyle.h"
#include "TSystem.h"

#include "TF1.h"
#include "TF1Convolution.h"
#include "TMinuit.h"
#include "TMath.h"
#include "TRandom3.h"
#include "TVector3.h"

// for gaps-online-software
#include "io.hpp"
#include "calibration.h"
#include "database.h"
#include "caraspace.hpp"

// for SimpleDet functions
#include "/home/kaoyama/software/SimpleDet/build/install/gaps-v2.1.0/include/gaps/CRawHeader.hh"
#include "/home/kaoyama/software/SimpleDet/build/install/gaps-v2.1.0/include/gaps/CRawTof.hh"
#include "/home/kaoyama/software/SimpleDet/build/install/gaps-v2.1.0/include/gaps/CRawTrk.hh"

// original headers
#include "mydetector.h" // ver.GAPS merge
#include "myfunc.h" // ver.GAPS merge
#include "myobjects.h" // ver.GAPS merge


TString data_dir = "/home/kaoyama/data/Antarctic_2024/processed/L1/";



// **** Load tree ****
TString inFileName;
TFile *fin;
TTree *tree;
Crane::Calibration::CRawHeader *Header;
Crane::Calibration::CRawTof *Tof;
Crane::Calibration::CRawTrk *Trk;

// **** Plot()/Next() ****
constexpr Int_t max_tofhit = 15;
constexpr Int_t max_sihit = 20;
TH1D *hWaveform_a[max_tofhit];
TH1D *hWaveform_b[max_tofhit];
TVector3 ModuleNum[max_sihit];
Int_t StripCh[max_sihit];
Double_t StripE[max_sihit];
Int_t npaddle=0;
Int_t nstrip=0;

constexpr Int_t    nbin   = 1024;
constexpr Double_t tw_min = 0., tw_max = 500.;
constexpr Double_t ymin=-10., ymax=80.;
constexpr Int_t nRebin=1;
constexpr Int_t PED_STARTBIN=1, PED_STOPBIN=50;
constexpr Int_t TOT_STARTBIN=1250, TOT_STOPBIN=1300;
Int_t CurrentEventNumber=0;
Int_t NextEventNumber=0;

TCanvas *cEvent;
TPaveText *ptStrip;

// **** Plot3D() ****
TH3D *hDetectorHit;
TH2D *hDetectorHit_d;
TH2D *hDetectorHit_XY;
TH2D *hDetectorHit_XZ;
TH2D *hDetectorHit_YZ;
TH2D *hDetectorHit_TZ;
TH1D *hDetectorHit_T;
TCanvas *cDetectorHit;
TCanvas *cDetectorHitProp;
TGraph2DErrors *gDetectorHit;
TGraphErrors *gDetectorHit_XY;
TGraphErrors *gDetectorHit_XZ;
TGraphErrors *gDetectorHit_YZ;
TGraphErrors *gDetectorHit_TZ;
TGraph2DErrors *gDetectorHit_tof;
TGraphErrors *gDetectorHit_tof_XY;
TGraphErrors *gDetectorHit_tof_XZ;
TGraphErrors *gDetectorHit_tof_YZ;
TGraphErrors *gDetectorHit_tof_TZ;
TGraph2DErrors *gDetectorHit_si;
TGraphErrors *gDetectorHit_si_XY;
TGraphErrors *gDetectorHit_si_XZ;
TGraphErrors *gDetectorHit_si_YZ;
TGraphErrors *gDetectorHit_si_TZ;


// **** GetPhysicsParameer() ****
vector<int> vhit_paddle_num;
vector<TVector3> vhit_paddle_pos;
vector<double> vhit_paddle_time;
vector<double> vhit_sipma_time;
vector<double> vhit_sipmb_time;
vector<double> vhit_paddle_scale;
vector<double> vhit_sipma_scale;
vector<double> vhit_sipmb_scale;
vector<double> vhit_ch9phase;
vector<double> vhit_si_dx;
vector<double> vhit_si_dy;
Double_t x0_res, y0_res, z0_res, theta_res, phi_res;
Double_t x0_res_err, y0_res_err, z0_res_err, theta_res_err, phi_res_err;

// ********************


void LoadRootFile(TString fileName)
{
  inFileName = fileName;
  fin = TFile::Open(inFileName);
  delete tree;
  tree = (TTree*)fin->Get("TreeRaw");
  tree -> SetBranchAddress("Header", &Header);
  tree -> SetBranchAddress("Tof", &Tof);
  tree -> SetBranchAddress("Trk", &Trk);
  tree -> GetEntry(0);
  cout << "Load waveform file: " << inFileName << endl;
}


void LoadRootFile(Int_t RunNumber, Long_t itime)
{
  TString timeStr = Form("%ld", itime);
  TString datePart = timeStr(0, 6);
  TString timePart = timeStr(6, 6);
  TString cmd = Form("ls /home/kaoyama/data/Antarctic_2024/processed/L1/%d/Run%d.gse5_%s_%sUTC_rec.root", RunNumber, RunNumber, datePart.Data(), timePart.Data());
  TString fileName = gSystem->GetFromPipe(cmd.Data());
  fileName = fileName.Strip(TString::kBoth);
  LoadRootFile(fileName);
}



Bool_t _GetTof()
{
  if(!gROOT->FindObject("hWaveform_a_pd0")){
    for(Int_t ipd=0;ipd<max_tofhit;++ipd){
      delete hWaveform_a[ipd];
      hWaveform_a[ipd] = new TH1D(Form("hWaveform_a_pd%d",ipd),"hWaveform_a_pd;Time (ns);ADC counts", nbin, tw_min, tw_max);
      delete hWaveform_b[ipd];
      hWaveform_b[ipd] = new TH1D(Form("hWaveform_b_pd%d",ipd),"hWaveform_b_pd;Time (ns);ADC counts", nbin, tw_min, tw_max);  
    }
  }
  else{
    for(Int_t ipd=0;ipd<npaddle;++ipd){
      hWaveform_a[ipd] -> Reset();
      hWaveform_b[ipd] -> Reset();
    }
  }

  // retun false if incomplete data
  if(Tof->wfs.voltages.empty()) {npaddle = 0; return false;}
  npaddle = Tof->wfs.voltages.size();
  if(npaddle%2){ npaddle = 0; return false;}
  npaddle /= 2;
  if(npaddle>max_tofhit){ npaddle = 0; return false;}


  for(Int_t ipd=0;ipd<npaddle;++ipd){
    Int_t paddle_index_a = ipd*2 + (Tof->wfs.paddle_id[ipd*2] > 0);
    Int_t paddle_index_b = ipd*2 + (Tof->wfs.paddle_id[ipd*2] < 0);
    hWaveform_a[ipd] -> SetTitle(Form("hWaveform_a_pd%d",Tof->wfs.paddle_id[paddle_index_a]));
    hWaveform_b[ipd] -> SetTitle(Form("hWaveform_b_pd%d",Tof->wfs.paddle_id[paddle_index_b]));
  
    for(Int_t ibin=0;ibin<nbin;++ibin){
      hWaveform_a[ipd] -> SetBinContent(ibin+1,Tof->wfs.voltages[paddle_index_a][ibin]);
      hWaveform_a[ipd] -> SetBinError(ibin+1,1.);
      hWaveform_b[ipd] -> SetBinContent(ibin+1,Tof->wfs.voltages[paddle_index_b][ibin]);
      hWaveform_b[ipd] -> SetBinError(ibin+1,1.);
    }
  }

  return true;
}


Bool_t _GetTrk()
{
  for(Int_t ist=0;ist<nstrip;++ist){
    ModuleNum[ist].SetXYZ(0,0,0);
    StripCh[ist] = 0;
    StripE[ist] = 0;
  }

  if(Trk->layer.empty()){ nstrip = 0; return true;}
  nstrip = Trk->layer.size();

  if(nstrip>max_sihit){
    cout << "too many strip hits, nstrip = " << nstrip << endl;
    nstrip = 0;
    return false;
  } else if (nstrip!=(Int_t)Trk->row.size()){
    cout << "number of data in layer and row is different" << endl;
    nstrip = 0;
    return false;
  } else if (nstrip!=(Int_t)Trk->module.size()){
    cout << "number of data in layer and module is different" << endl;
    nstrip = 0;
    return false;
  } else if (nstrip!=(Int_t)Trk->channel.size()){
    cout << "number of data in layer and channel is different" << endl;
    nstrip = 0;
    return false;
  } else if (nstrip!=(Int_t)Trk->adcdata.size()){
    cout << "number of data in layer and adcdata is different" << endl;
    nstrip = 0;
    return false;
  }

  for(Int_t ist=0;ist<nstrip;++ist){
    ModuleNum[ist].SetXYZ((Double_t)Trk->layer[ist], (Double_t)Trk->row[ist], (Double_t)Trk->module[ist]);
    StripCh[ist] = (Int_t)Trk->channel[ist];
    StripE[ist] = (Double_t)Trk->adcdata[ist];
  }

  return true;
}


void _CalculateTofHits()
{
  vhit_paddle_num.resize(npaddle);
  vhit_paddle_pos.resize(npaddle);
  vhit_paddle_time.resize(npaddle);
  vhit_sipma_time.resize(npaddle);
  vhit_sipmb_time.resize(npaddle);
  vhit_paddle_scale.resize(npaddle);
  vhit_sipma_scale.resize(npaddle);
  vhit_sipmb_scale.resize(npaddle);
  vhit_ch9phase.resize(npaddle);
  
  for(Int_t ipd=0;ipd<npaddle;++ipd){
    Int_t pd_num  = abs(Tof->wfs.paddle_id[ipd*2]);
    Double_t t9 = (Tof->hits.phase[ipd]+TMath::Pi())/(2.*TMath::Pi())*50.;// ns
    Double_t tcor = harting_cable_time[pd_num-1] - coax_cable_time[pd_num-1] + t9;
    vhit_paddle_num[ipd] = pd_num;

    // parametes from row level waveform
    vhit_sipma_time[ipd]  = _GetMPPCHitTiming(hWaveform_a[ipd]);
    vhit_sipmb_time[ipd]  = _GetMPPCHitTiming(hWaveform_b[ipd]);
    vhit_sipma_scale[ipd] = _GetMPPCCharge(hWaveform_a[ipd]);
    vhit_sipmb_scale[ipd] = _GetMPPCCharge(hWaveform_b[ipd]);
    vhit_ch9phase[ipd]    = t9;// ns

    // parametes calculated by concerning some correction factor
    vhit_paddle_scale[ipd] = vhit_sipma_scale[ipd]+vhit_sipmb_scale[ipd];
    vhit_paddle_pos[ipd]   = _GetPaddleHitPosition(vhit_sipma_time[ipd], vhit_sipmb_time[ipd], mppc_pos_a[pd_num-1], mppc_pos_b[pd_num-1]);
    vhit_paddle_time[ipd]  = _GetPaddleHitTiming(  vhit_sipma_time[ipd], vhit_sipmb_time[ipd], mppc_pos_a[pd_num-1], mppc_pos_b[pd_num-1], tcor);
  }
}


void _CalculateSiHits()
{


}


void _Plot3D(){
  // tof hit
  // delete gDetectorHit_tof;
  // gDetectorHit_tof = new TGraph2DErrors();
  delete gDetectorHit_tof_XY; gDetectorHit_tof_XY = new TGraphErrors();
  delete gDetectorHit_tof_XZ; gDetectorHit_tof_XZ = new TGraphErrors();
  delete gDetectorHit_tof_YZ; gDetectorHit_tof_YZ = new TGraphErrors();

  // si hit
  delete gDetectorHit_si_XY; gDetectorHit_si_XY = new TGraphErrors();
  delete gDetectorHit_si_XZ; gDetectorHit_si_XZ = new TGraphErrors();
  delete gDetectorHit_si_YZ; gDetectorHit_si_YZ = new TGraphErrors();

  // all hit
  delete gDetectorHit   ; gDetectorHit = new TGraph2DErrors();
  delete gDetectorHit_XY; gDetectorHit_XY = new TGraphErrors();
  delete gDetectorHit_XZ; gDetectorHit_XZ = new TGraphErrors();
  delete gDetectorHit_YZ; gDetectorHit_YZ = new TGraphErrors();
  delete gDetectorHit_TZ; gDetectorHit_TZ = new TGraphErrors();
  delete hDetectorHit_T ; hDetectorHit_T = new TH1D("hDetecotr_T","hDetector_T; Hit timing; Number of event", 1024, -50.-0.5/1024., 450.-0.5/1024);


  Int_t ipoint = 0;
  // tof hit
  for(Int_t ipd=0;ipd<npaddle;++ipd){
    // gDetectorHit_tof    -> SetPoint(ipd, vhit_paddle_pos[ipd].X(), vhit_paddle_pos[ipd].Y(), vhit_paddle_pos[ipd].Z());
    Int_t pd = vhit_paddle_num[ipd]-1;
    Double_t x = vhit_paddle_pos[ipd].X();
    Double_t y = vhit_paddle_pos[ipd].Y();
    Double_t z = vhit_paddle_pos[ipd].Z();
    Double_t t = vhit_paddle_time[ipd];
    Double_t ex = tof_position_error[pd].X();
    Double_t ey = tof_position_error[pd].Y();
    Double_t ez = tof_position_error[pd].Z();
    Double_t et = 0.5;

    gDetectorHit_tof_XY -> SetPoint(ipd, x, y);
    gDetectorHit_tof_XZ -> SetPoint(ipd, x, z);
    gDetectorHit_tof_YZ -> SetPoint(ipd, y, z);
    gDetectorHit_tof_XY -> SetPointError(ipd, ex, ey);
    gDetectorHit_tof_XZ -> SetPointError(ipd, ex, ez);
    gDetectorHit_tof_YZ -> SetPointError(ipd, ey, ez);
    hDetectorHit_T -> Fill(vhit_paddle_time[ipd]);

    // hit selection
    if(vhit_sipma_scale[ipd]<500. || vhit_sipmb_scale[ipd]<500.) continue;
    if(vhit_sipma_scale[ipd]>10000. || vhit_sipmb_scale[ipd]>10000.) continue;
    gDetectorHit    -> SetPoint(ipoint, x, y, z);
    gDetectorHit_XY -> SetPoint(ipoint, x, y);
    gDetectorHit_XZ -> SetPoint(ipoint, x, z);
    gDetectorHit_YZ -> SetPoint(ipoint, y, z);
    gDetectorHit_TZ -> SetPoint(ipoint, t, z);
    gDetectorHit    -> SetPointError(ipoint, ex, ey, ez);
    gDetectorHit_XY -> SetPointError(ipoint, ex, ey);
    gDetectorHit_XZ -> SetPointError(ipoint, ex, ez);
    gDetectorHit_YZ -> SetPointError(ipoint, ey, ez);
    gDetectorHit_TZ -> SetPointError(ipoint, et, ez);

    ++ipoint;
  }

  // si hit
  for(Int_t ist=0;ist<nstrip;++ist){
    Int_t layer = (Int_t)Trk->layer[ist];
    Int_t row = (Int_t)Trk->row[ist];
    Int_t module = (Int_t)Trk->module[ist];
    Int_t channel = (Int_t)Trk->channel[ist];
    Double_t x = strip_pos[layer][row][module][channel].X();
    Double_t y = strip_pos[layer][row][module][channel].Y();
    Double_t z = strip_pos[layer][row][module][channel].Z();
    // Double_t ex = si_position_error[vhit_paddle_num[ipd]-1].X();
    // Double_t ey = si_position_error[vhit_paddle_num[ipd]-1].Y();
    // Double_t ez = si_position_error[vhit_paddle_num[ipd]-1].Z();

    gDetectorHit_si_XY -> SetPoint(ist, x, y);
    gDetectorHit_si_XZ -> SetPoint(ist, x, z);
    gDetectorHit_si_YZ -> SetPoint(ist, y, z);
    // gDetectorHit_si    -> SetPointError(ipd, ex, ey, ez);
    // gDetectorHit_si_XY -> SetPointError(ipd, ex, ey);
    // gDetectorHit_si_XZ -> SetPointError(ipd, ex, ez);
    // gDetectorHit_si_YZ -> SetPointError(ipd, ey, ez);
  }
}



void _PlotDraw()
{
  if(!gROOT->FindObject("cEvent")){
    cEvent = new TCanvas("cEvent", "cEvent", 800, 600);
    cDetectorHit = new TCanvas("cDetectorHit","cDetectorHit", 800, 800);
    cDetectorHitProp = new TCanvas("cDetectorHitProp","cDetectorHitProp", 800, 800);

    // 3d plot
    hDetectorHit = new TH3D("hDetectorHit","hDetectorHit;x;y;z",100,-200.,200.,100,-200.,200.,100,-50.,250.);
    hDetectorHit -> SetStats(0);
    // 2d plot
    hDetectorHit_d = new TH2D("hDetectorHit_d","hDetectorHit_d;x;y",100,-200.,200.,100,-200.,200.);
    hDetectorHit_d -> SetStats(0);
    hDetectorHit_XY = new TH2D("hDetectorHit_XY","hDetectorHit_XY;x;y",100,-200.,200.,100,-200.,200.);
    hDetectorHit_XY -> SetStats(0);
    hDetectorHit_XZ = new TH2D("hDetectorHit_XZ","hDetectorHit_XZ;x;z",100,-200.,200.,100,-50.,250.);
    hDetectorHit_XZ -> SetStats(0);
    hDetectorHit_YZ = new TH2D("hDetectorHit_YZ","hDetectorHit_YZ;y;z",100,-200.,200.,100,-50.,250.);
    hDetectorHit_YZ -> SetStats(0);
    hDetectorHit_TZ = new TH2D("hDetecotr_TZ","hDetector_TZ; Hit timing; Z", 1024, -50.-0.5/1024., 450.-0.5/1024, 300, -50., 250.);
    hDetectorHit_TZ -> SetStats(0);

  }
  cEvent -> Clear();
  cEvent -> Divide(3,3);
  cDetectorHit -> Clear();
  cDetectorHit -> Divide(2,2);
  cDetectorHitProp -> Clear();
  cDetectorHitProp -> Divide(2,2);

  cEvent -> cd(1);
  delete ptStrip;
  ptStrip = new TPaveText(0.01, 0.1, 0.99, 0.9, "NDC");
  for(Int_t ist=0;ist<nstrip;++ist){
    ptStrip->AddText(TString::Format("Hit %d: Layer %d, Row %d, Module %d, Ch %d -> Energy = %.1f",
             ist, (Int_t)ModuleNum[ist].X(), (Int_t)ModuleNum[ist].Y(), (Int_t)ModuleNum[ist].Z(), (Int_t)StripCh[ist], StripE[ist]));
  }
  ptStrip -> SetTextAlign(11);
  ptStrip -> Draw();
  for(Int_t ipd=0;ipd<npaddle;++ipd){
    cEvent -> cd(ipd+2);
    hWaveform_a[ipd] -> SetLineColor(1);
    hWaveform_b[ipd] -> SetLineColor(2);
    // hWaveform_a[ipd] -> GetYaxis() -> SetRangeUser(ymin, ymax);
    hWaveform_a[ipd] -> Draw("hist");
    hWaveform_b[ipd] -> Draw("same hist");
  }
  cEvent -> Update();


  cDetectorHit -> cd(1);
  hDetectorHit -> Reset();
  for(Int_t ipd=0;ipd<npaddle;++ipd){
    cout << vhit_paddle_num[ipd] << ": " << vhit_paddle_pos[ipd].X() << ", " << vhit_paddle_pos[ipd].Y() << ", " << vhit_paddle_pos[ipd].Z() << endl;
    hDetectorHit -> Fill(vhit_paddle_pos[ipd].X(), vhit_paddle_pos[ipd].Y(), vhit_paddle_pos[ipd].Z());
  }
  hDetectorHit -> Draw("box");
  for (int i = 0; i < 160; ++i){
      tbTof[i] -> Draw("same");
  }
  for(int ilayer=0; ilayer<nlayer;++ilayer){
    for(int irow=0; irow<nrow;++irow){
      for(int imodule=0;imodule<nmodule;++imodule){
        for(int ichannel=0;ichannel<32;++ichannel){
          tbSi[ilayer][irow][imodule][ichannel] -> Draw("same");
        }
      }
    }
  }
  cDetectorHit -> cd(2);
  hDetectorHit_XY -> Draw("");
  gDetectorHit_XY -> Draw("same p");
  gDetectorHit_tof_XY -> SetLineColor(kBlue);
  gDetectorHit_tof_XY -> SetMarkerColor(kBlue);
  gDetectorHit_tof_XY -> Draw("same p");
  gDetectorHit_si_XY -> SetLineColor(kGreen);
  gDetectorHit_si_XY -> SetMarkerColor(kGreen);
  gDetectorHit_si_XY -> Draw("same p");
  cDetectorHit -> cd(3);
  hDetectorHit_XZ -> Draw("");
  gDetectorHit_XZ -> Draw("same p");
  gDetectorHit_tof_XZ -> SetLineColor(kBlue);
  gDetectorHit_tof_XZ -> SetMarkerColor(kBlue);
  gDetectorHit_tof_XZ -> Draw("same p");
  gDetectorHit_si_XZ -> SetLineColor(kGreen);
  gDetectorHit_si_XZ -> SetMarkerColor(kGreen);
  gDetectorHit_si_XZ -> Draw("same p");
  cDetectorHit -> cd(4);
  hDetectorHit_YZ -> Draw("");
  gDetectorHit_YZ -> Draw("same p");
  gDetectorHit_tof_YZ -> SetLineColor(kBlue);
  gDetectorHit_tof_YZ -> SetMarkerColor(kBlue);
  gDetectorHit_tof_YZ -> Draw("same p");
  gDetectorHit_si_YZ -> SetLineColor(kGreen);
  gDetectorHit_si_YZ -> SetMarkerColor(kGreen);
  gDetectorHit_si_YZ -> Draw("same p");
  cDetectorHit -> Update();

  cDetectorHitProp -> cd(1);
  hDetectorHit_T -> Draw();
  cDetectorHitProp -> cd(3);
  hDetectorHit_TZ -> Draw("");
  gDetectorHit_TZ -> Draw("same p");
  cDetectorHitProp -> Update();

}


Bool_t Plot(Int_t ievent, Bool_t is_Disp=true, Bool_t is_3D=true)// need to modify pedestal errors
{
  // Get tree event
  CurrentEventNumber = ievent;
  tree->GetEntry(ievent);

  Bool_t is_tof = _GetTof();
  Bool_t is_trk = _GetTrk();

  if(!is_tof) return false;
  // if(!is_trk) return -1;

  _CalculateTofHits();
  _CalculateSiHits();

  if(is_3D) _Plot3D();
  if(is_Disp) _PlotDraw();


  return true;
}


void Next()
{
  NextEventNumber = CurrentEventNumber+1;
  cout << "plot event" << NextEventNumber << endl;
  Plot(NextEventNumber,true,true);
}
