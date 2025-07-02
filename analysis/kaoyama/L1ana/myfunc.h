// ver.GAPS merge

// helpful functions
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

  
Double_t func_gaus_exp(Double_t *x, Double_t *par)
{
    Double_t t = x[0]; // time
    Double_t mu = par[0];// t0
    Double_t lam = 1./par[1];// 1/tau
    Double_t sig = par[2]; // sigma
    Double_t A = par[3]; // scale

    Double_t val = A*lam/2.*exp(lam*mu+pow(lam*sig,2)*0.5)*exp(-1.*lam*t)*erfc((mu+lam*pow(sig,2)-t)/(sqrt(2.)*sig));
    return val;

}

// Double_t func_gaus_landau(Double_t *x, Double_t *par)
// {
//   double xx = x[0];
//   double landauMPV = par[0];  // MPV of Landau
//   double landauSigma = par[1];  // Landau width
//   double gausSigma = par[2];  // Gaussian sigma
//   double norm = par[3];  // Normalization factor

//   return norm * ROOT::Math::landau_gau_pdf(xx, landauMPV, landauSigma, gausSigma);
// }

TF1 *FitWaveform(TH1D *hin)
{
    TF1 *f = new TF1("f",func_gaus_exp,0, 500, 4);
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

// **** functions to calculate parameters ****
double _GetMPPCHitTiming(TH1D *hin)
{
  hin -> GetXaxis() -> SetRange(40, 800);
  Int_t    peakbin = hin -> GetMaximumBin();
  Double_t thr    = hin -> GetBinContent(peakbin)*0.5;
  Int_t    thrbin  = 1;
  for(int ibin=peakbin-30; ibin<peakbin; ++ibin){
    if(thr<hin->GetBinContent(ibin)) break;
    thrbin=ibin;
  }
  return hin->GetBinCenter(thrbin);
}

double _GetMPPCHitDT(Double_t ta, Double_t tb)
{
  return tb-ta;
}

double _GetMPPCCharge(TH1D *hin)
{
  Int_t peakbin = hin -> GetMaximumBin();
  constexpr Int_t mbin = 30;
  constexpr Int_t pbin = 300;
  Double_t sum=0.;
  for(int ibin=peakbin-mbin; ibin<peakbin+pbin; ++ibin){
    sum+=hin->GetBinContent(ibin);
  }
  return sum;
}

TVector3 _GetPaddleHitPosition(Double_t ta, Double_t tb, TVector3 pos_a, TVector3 pos_b)
{
  TVector3 diff = pos_b-pos_a;
  Double_t L = diff.Mag();
  TVector3 l = diff.Unit()*((v_schinti*(ta-tb)+L)/2.);
  return pos_a + l;
}

double  _GetPaddleHitTiming(Double_t ta, Double_t tb, TVector3 pos_a, TVector3 pos_b, Double_t tcor)
{
  TVector3 diff = pos_b-pos_a;
  Double_t L = diff.Mag();
  double t0 = ((ta+tb) - L/v_schinti)/2.;
  return t0 + tcor;
}
// ***********************************

// tracking functions
void fitFunction(Int_t &npar, Double_t *gin, Double_t &f, Double_t *par, Int_t iflag);
void _chi2_linear(Int_t &npar, Double_t *gin, Double_t &f, Double_t *par, Int_t iflag);
TF1 *fxz;
TF1 *fyz;
TF1 *fxy;
TF1 *fxz_res;
TF1 *fyz_res;
TF1 *fxy_res;
TMinuit *minuit;
TPolyLine3D *line_xyz;
Double_t chi2_res=0.;





