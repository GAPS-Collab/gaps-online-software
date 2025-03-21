// ver.GAPS tof
#include <iostream>
#include <fstream>
#include <vector>
#include "TMath.h"
#include "TF1.h"
#include "TH1D.h"
#include "TH3D.h"
#include "TCanvas.h"
#include "TGraphErrors.h"
#include "TGraph2DErrors.h"
#include "TPolyLine3D.h"
#include "TMinuit.h"

// helpful functions
void show(const vector<int> &x) {
    cout << "{ ";
    for (auto v : x) {
        cout << v << " ";
    }
    cout << "} \n";
}

Int_t is_contain(vector<int> *vec, int v)
{
    auto itr=std::find(vec->begin(), vec->end(), v);
    if(itr!=vec->end()){
       return std::distance(vec->begin(), itr);
    }
    else{
        return -1;
    }
}

  
// **** functions to calculate parameters ****
Double_t funcWaveform(Double_t *x, Double_t *par)
{
    Double_t t = x[0]; // time
    Double_t mu = par[0];// t0
    Double_t lam = 1./par[1];// 1/tau
    Double_t sig = par[2]; // sigma
    Double_t A = par[3]; // scale

    Double_t val = A*lam/2.*exp(lam*mu+pow(lam*sig,2)/2.)*exp(-1.*lam*t)*erfc((mu+lam*pow(sig,2)-t)/(sqrt(2.)*sig));
    return val;

}
TF1 *FitWaveform(TH1D *hin)
{
    TF1 *f = new TF1("f",funcWaveform,0, 500, 4);
    hin -> GetXaxis() -> SetRange(30, 900);
    f->SetParameters(hin->GetBinCenter(std::max(hin->GetMaximumBin(), 70)), 15., 2., 800);
    f->SetParLimits(0,20,250);
    f->SetParLimits(1,5,25);
    f->SetParLimits(2,0,10);
    f->SetParLimits(3,1,1e5);
    f->SetNpx(5000);
    f->SetParNames("t0","tau","sigma","Scale");
    hin->Fit("f","Q","0",30., 900.);
    return f;

}


double GetMPPCHitTiming(TH1D *hin)
{
  hin -> GetXaxis() -> SetRange(40, 800);
  Int_t    peakbin = hin -> GetMaximumBin();
  Double_t peak    = hin -> GetBinContent(peakbin);
  Int_t    thrbin  = 1;
  for(int ibin=peakbin-30; ibin<peakbin; ++ibin){
    if(peak/2<hin->GetBinContent(ibin)) break;
    // if(10<hin->GetBinContent(ibin)) break;
    thrbin=ibin;
  }
  return hin->GetBinCenter(thrbin);
}

double GetMPPCHitDT(Double_t ta, Double_t tb)
{
  return tb-ta;
}

double GetMPPCCharge(TH1D *hin)
{
  Int_t peakbin = hin -> GetMaximumBin();
  Int_t mbin = 30;
  Int_t pbin = 300;
  Double_t sum=0;
  for(int ibin=peakbin-mbin; ibin<peakbin+pbin; ++ibin){
    sum+=hin->GetBinContent(ibin);
  }
  return sum;
}

TVector3 _GetPaddleHitPosition(Double_t ta, Double_t tb, Int_t pd)
{
  TVector3 diff = mppc_pos_b[pd-1]-mppc_pos_a[pd-1];
  Double_t L = diff.Mag();
  TVector3 l = diff.Unit()*((v_schinti*(ta-tb)+L)/2.);
  return mppc_pos_a[pd-1] + l;
}

double  _GetPaddleHitTiming(Double_t ta, Double_t tb, Int_t pd)
{
  TVector3 diff = mppc_pos_b[pd-1]-mppc_pos_a[pd-1];
  Double_t L = diff.Mag();
  double t0 = ((ta+tb) - L/v_schinti)/2.;
  return t0;
}
// ***********************************

// tracking functions
void fitFunction(Int_t &npar, Double_t *gin, Double_t &f, Double_t *par, Int_t iflag);
TF1 *fxz;
TF1 *fyz;
TF1 *fxy;
TF1 *fxz_res;
TF1 *fyz_res;
TF1 *fxy_res;
TMinuit *minuit;
TPolyLine3D *line_xyz;
Double_t chi2_res=0.;

TH3D *hDetectorHit;
TH2D *hDetectorHit_d;
TH2D *hDetectorHit_XY;
TH2D *hDetectorHit_XZ;
TH2D *hDetectorHit_YZ;
TH1D *hDetectorHit_time;
TCanvas *cDetectorHit;
TCanvas *cDetectorHitProp;
TGraph2DErrors *gDetectorHit;
TGraphErrors *gDetectorHit_XY;
TGraphErrors *gDetectorHit_XZ;
TGraphErrors *gDetectorHit_YZ;

