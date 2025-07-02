// ver.GAPS merge
// main analysis file

#define HIST_ALLPD 0 // for all paddle histograms
#define TOF_ANA 0 // for TOF analysis
#define SI_ANA 0 // for Si analysis
#define TRACKING 1 // for tracking analysis

#include "myheader.h" // ver.GAPS merge

// test include
#include "TObjString.h"

// ******************************************************************
// *********************** Fill histograms **************************
// ******************************************************************
void _FillTof(Bool_t is_tracking=true)
{
  h_npaddle -> Fill(npaddle);
  for(Int_t ipd=0;ipd<npaddle;++ipd){
    // h_rb_id -> Fill(Tof->wfs.rb_id[ipd*2]);
    h_pd_id -> Fill(vhit_paddle_num[ipd]);
    h_hitdt_ab[vhit_paddle_num[ipd]-1]    -> Fill(vhit_sipmb_time[ipd]-vhit_sipma_time[ipd]);
    h_hittime_ab[vhit_paddle_num[ipd]-1]  -> Fill(vhit_sipma_time[ipd],vhit_sipmb_time[ipd]);
    h_hittime_a[vhit_paddle_num[ipd]-1]   -> Fill(vhit_sipma_time[ipd]);
    h_hittime_b[vhit_paddle_num[ipd]-1]   -> Fill(vhit_sipmb_time[ipd]);
    h_charge_a[vhit_paddle_num[ipd]-1]    -> Fill(vhit_sipma_scale[ipd]);
    h_charge_b[vhit_paddle_num[ipd]-1]    -> Fill(vhit_sipmb_scale[ipd]);
    h_charge_sum[vhit_paddle_num[ipd]-1]  -> Fill(vhit_sipma_scale[ipd]+vhit_sipmb_scale[ipd]);
    h_charge_ab[vhit_paddle_num[ipd]-1]   -> Fill(vhit_sipma_scale[ipd],vhit_sipmb_scale[ipd]);
    h_dt_charge_a[vhit_paddle_num[ipd]-1] -> Fill(vhit_sipmb_time[ipd]-vhit_sipma_time[ipd], vhit_sipma_scale[ipd]);
    h_dt_charge_b[vhit_paddle_num[ipd]-1] -> Fill(vhit_sipmb_time[ipd]-vhit_sipma_time[ipd], vhit_sipmb_scale[ipd]);
    h_dt_charge_sum[vhit_paddle_num[ipd]-1]-> Fill(vhit_sipmb_time[ipd]-vhit_sipma_time[ipd], vhit_sipma_scale[ipd]+vhit_sipmb_scale[ipd]);
    h_phase_ch9[vhit_paddle_num[ipd]-1]   -> Fill(vhit_ch9phase[ipd]);

  }

  #if HIST_ALLPD
  for(Int_t ipd=0;ipd<npaddle;++ipd){
    Int_t pd_num1 = vhit_paddle_num[ipd];
    for(Int_t jpd=ipd+1;jpd<npaddle;++jpd){
      Int_t pd_num2 = vhit_paddle_num[jpd];
      Double_t dt = abs(vhit_paddle_time[jpd]-vhit_paddle_time[ipd]);
      Double_t tof_l = (vhit_paddle_pos[jpd]-vhit_paddle_pos[ipd]).Mag();
    if(pd_num1<pd_num2){
        h_hitdt_pd[pd_num1-1][pd_num2-1] -> Fill(dt);
        h_tof_length[pd_num1-1][pd_num2-1] -> Fill(tof_l);
        h_hitbeta_pd[pd_num1-1][pd_num2-1] -> Fill(tof_l/dt/v_light);
      } else {
        h_hitdt_pd[pd_num2-1][pd_num1-1] -> Fill(dt);
        h_tof_length[pd_num2-1][pd_num1-1] -> Fill(tof_l);
        h_hitbeta_pd[pd_num2-1][pd_num1-1] -> Fill(tof_l/dt/v_light);
      }
    }
  } 
  #endif

  if(is_tracking){
    // tracking
    h_track_theta     -> Fill(theta_res/TMath::Pi()*180.);
    h_track_theta_mod -> Fill(TMath::Cos(theta_res));
    h_track_phi -> Fill(phi_res/TMath::Pi()*180.);
    h_track_chi2 -> Fill(chi2_res);
    h_track_x0 -> Fill(x0_res);
    h_track_y0 -> Fill(y0_res);
    h_track_z0 -> Fill(z0_res);

    if(chi2_res>2.) return;
    for(Int_t ipd=0;ipd<npaddle;++ipd){
      h_charge_sum_theta[vhit_paddle_num[ipd]-1] -> Fill(1./cos(theta_res), vhit_sipma_scale[ipd]+vhit_sipmb_scale[ipd]);
      // h_charge_sum_theta[vhit_paddle_num[ipd]-1] -> Fill(vhit_sipma_scale[0]+vhit_sipmb_scale[0], theta_res/TMath::Pi()*180.);
    }
    h_track_theta_cut1 -> Fill(theta_res/TMath::Pi()*180.);
    h_track_theta_mod_cut1 -> Fill(TMath::Cos(theta_res));
    h_track_phi_cut1 -> Fill(phi_res/TMath::Pi()*180.);
    h_track_chi2_cut1 -> Fill(chi2_res);
    h_track_x0_cut1 -> Fill(x0_res);
    h_track_y0_cut1 -> Fill(y0_res);
    h_track_z0_cut1 -> Fill(z0_res);
  }

}


void _FillSi(Bool_t is_tracking=true)
{
  for(Int_t ist=0;ist<nstrip;++ist){
    Int_t layer = (Int_t)Trk->layer[ist];
    Int_t row = (Int_t)Trk->row[ist];
    Int_t module = (Int_t)Trk->module[ist];
    Int_t channel = (Int_t)Trk->channel[ist];
    h_si_energy[layer][row][module][channel] -> Fill(StripE[ist]);
    h_si_energy_per_module[layer][row][module] -> Fill(StripE[ist]);
    h_hit_module[layer] -> Fill(row, module);

    if(is_tracking){
      // if(ModuleNum[ist].X() != 1) continue;
      h_si_energy_per_module_theta[8][0][0] -> Fill(1./cos(theta_res), StripE[ist]);
      h_si_energy_per_module_theta[layer][row][module] -> Fill(1./cos(theta_res), StripE[ist]);
    }
  }
}


void _FillSipos_track()
{
  // get fit result
  double x1 = sin(theta_res) * cos(phi_res);
  double y1 = sin(theta_res) * sin(phi_res);
  double z1 = cos(theta_res);

  for(Int_t ist=0;ist<nstrip;++ist){
    // get xy position from fit result
    Int_t layer = (Int_t)ModuleNum[ist].X();
    Int_t row = (Int_t)ModuleNum[ist].Y();
    Int_t module = (Int_t)ModuleNum[ist].Z();
    Int_t channel = (Int_t)StripCh[ist];
    Double_t z = strip_pos[layer][row][module][channel].Z();
    Double_t t = ( (z - z0_res) / z1 );
    Double_t x = x0_res + x1 * t;
    Double_t y = y0_res + y1 * t;
    Double_t dx = strip_pos[layer][row][module][channel].X() - x;
    Double_t dy = strip_pos[layer][row][module][channel].Y() - y;
    h_diff_track_strip_xy_per_module[layer][row][module] -> Fill(dx, dy);
    h_diff_track_strip_xy[layer][row][module][channel] -> Fill(dx, dy);
  }

}

void _Fill()
{
  #if TOF_ANA
  _FillTof();
  #endif
  #if SI_ANA
  _FillSi();
  #endif

  _FillSipos_track();
}

// ******************************************************************
// *********************** linear fitting ***************************
// ******************************************************************
void _LinearFitDraw()
{
  cout << endl << "**** minimization result ****" << endl;
  cout << "- status: " << minuit->fCstatu << endl;
  cout << "- chi2  : " << chi2_res  << endl;
  cout << "- x0    : " << x0_res    << "\t +- " << x0_res_err    << endl;
  cout << "- y0    : " << y0_res    << "\t +- " << y0_res_err    << endl;
  cout << "- z0    : " << z0_res    << "\t +- " << z0_res_err    << endl;
  cout << "- theta : " << theta_res << "\t +- " << theta_res_err << endl;
  cout << "- phi   : " << phi_res   << "\t +- " << phi_res_err   << endl;
  cout << "*****************************" << endl << endl;

  double x1 = sin(theta_res) * cos(phi_res);
  double y1 = sin(theta_res) * sin(phi_res);
  double z1 = cos(theta_res);

  line_xyz = new TPolyLine3D(100);
  for (int i = 0; i < 100; i++) {
      double t = i * 4 - 200;
      double x_fit = x0_res + x1 * t;
      double y_fit = y0_res + y1 * t;
      double z_fit = z0_res + z1 * t;
      line_xyz->SetPoint(i, x_fit, y_fit, z_fit);
  }
  line_xyz->SetLineColor(kRed);
  cDetectorHit -> cd(1);
  line_xyz->Draw("same");

  delete fxz_res;
  fxz_res = new TF1("fxz_res", "pol1", -200., 200.);
  delete fyz_res;
  fyz_res = new TF1("fyz_res", "pol1", -200., 200.);
  delete fxy_res;
  fxy_res = new TF1("fxy_res", "pol1", -200., 200.);

  cDetectorHit -> cd(2);
  fxy_res->SetParameters(y0_res-x0_res*tan(phi_res), tan(phi_res));
  fxy_res->SetLineColor(kRed);
  fxy_res->SetNpx(1000);
  fxy_res->Draw("same");

  cDetectorHit -> cd(3);
  fxz_res->SetParameters(z0_res-x0_res*cos(theta_res)/(sin(theta_res)*cos(phi_res)), cos(theta_res)/(sin(theta_res)*cos(phi_res)));
  fxz_res->SetLineColor(kRed);
  fxz_res->SetNpx(1000);
  fxz_res->Draw("same");

  cDetectorHit -> cd(4);
  fyz_res->SetParameters(z0_res-y0_res*cos(theta_res)/(sin(theta_res)*sin(phi_res)), cos(theta_res)/(sin(theta_res)*sin(phi_res)));
  fyz_res->SetLineColor(kRed);
  fyz_res->SetNpx(1000);
  fyz_res->Draw("same");


}


void _chi2_linear(Int_t &npar, Double_t *gin, Double_t &f, Double_t *par, Int_t iflag){
  double x0    = par[0];
  double y0    = par[1];
  double z0    = par[2];
  double theta = par[3];
  double phi   = par[4];

  double x1 = sin(theta) * cos(phi);
  double y1 = sin(theta) * sin(phi);
  double z1 = cos(theta);

  double chi2 = 0.0;

  for (int i = 0; i < gDetectorHit->GetN(); i++){
      double x = gDetectorHit->GetX()[i];
      double y = gDetectorHit->GetY()[i];
      double z = gDetectorHit->GetZ()[i];
      double sigma_x = gDetectorHit->GetEX()[i];
      double sigma_y = gDetectorHit->GetEY()[i];
      double sigma_z = gDetectorHit->GetEZ()[i];

      double t = ( (x - x0) * x1 + (y - y0) * y1 + (z - z0) * z1 );

      double x_fit = x0 + x1 * t;
      double y_fit = y0 + y1 * t;
      double z_fit = z0 + z1 * t;
      
      double dx = (x - x_fit) / sigma_x;
      double dy = (y - y_fit) / sigma_y;
      double dz = (z - z_fit) / sigma_z;
      
      chi2 += dx * dx + dy * dy + dz * dz;
  }
  if(gDetectorHit->GetN()>1) chi2_res = chi2/(Double_t)(gDetectorHit->GetN()*3-5);
  f = chi2_res;
}


void _LinearFit_morethan_2hits(Bool_t is_Disp=true)
{
  delete minuit;
  minuit = new TMinuit(5);
  minuit->SetPrintLevel(-1);
  minuit->SetFCN(_chi2_linear);
  
  TVector3 line = vhit_paddle_pos[1]-vhit_paddle_pos[0];
  Double_t x0_init = vhit_paddle_pos[0].X();
  Double_t y0_init = vhit_paddle_pos[0].Y();
  Double_t z0_init = vhit_paddle_pos[0].Z();
  Double_t theta_init = line.Theta();
  Double_t phi_init = line.Phi();
  minuit->DefineParameter(0, "x0", x0_init, 0.1, -200., 200.);
  minuit->DefineParameter(1, "y0", y0_init, 0.1, -200., 200.);
  minuit->DefineParameter(2, "z0", z0_init, 0.5, -50., 250.);
  minuit->DefineParameter(3, "theta", theta_init, 0.1, 0., TMath::Pi());
  minuit->DefineParameter(4, "phi", phi_init, 0.1, -1.*TMath::Pi(), TMath::Pi());
  
  minuit->Migrad();

  minuit->GetParameter(0, x0_res, x0_res_err);
  minuit->GetParameter(1, y0_res, y0_res_err);
  minuit->GetParameter(2, z0_res, z0_res_err);
  minuit->GetParameter(3, theta_res, theta_res_err);
  minuit->GetParameter(4, phi_res, phi_res_err);
  if(theta_res>TMath::Pi()/2.) theta_res = TMath::Pi()-theta_res;


  if(is_Disp) _LinearFitDraw();
}


void _LinearFit_is_2hits(Bool_t is_Disp=true)
{
  TVector3 line = vhit_paddle_pos[1]-vhit_paddle_pos[0];
  x0_res = vhit_paddle_pos[0].X();
  y0_res = vhit_paddle_pos[0].Y();
  z0_res = vhit_paddle_pos[0].Z();
  theta_res = line.Theta();
  phi_res = line.Phi();
  if(theta_res>TMath::Pi()/2.) theta_res = TMath::Pi()-theta_res;
  if(is_Disp) _LinearFitDraw();
}


void _LinearFit(Bool_t is_Disp=true)
{
  Int_t n_hit = gDetectorHit->GetN();
  if(n_hit > 2) _LinearFit_morethan_2hits(is_Disp);
  else if(n_hit == 2) _LinearFit_is_2hits(is_Disp);
  else {
    x0_res = y0_res = z0_res = theta_res = phi_res = -999.;
    x0_res_err = y0_res_err = z0_res_err = theta_res_err = phi_res_err = -999.;
    chi2_res = 999.;
  }

  
}

// ******************************************************************
// *********************** analyze histograms ***********************
// ******************************************************************
void _hists_ana_tofresolution()
{
  TF1 *f_gaus = new TF1("f_gaus","gaus",0.,20.);
  TH1D *h_timing_resolution = new TH1D("h_timing_resolution","h_timing_resolution; #sigma; Number of event", 100., 0., 5.);
  TH2D *h_timing_center_resolution = new TH2D("h_timing_center_resolution","h_timing_center_resolution; center; #sigma", 100, 0., 20., 100, 0., 5.);
  for(Int_t ipd=0;ipd<npd;++ipd)
  for(Int_t jpd=ipd+1;jpd<npd;++jpd){
    f_gaus -> SetParameters(h_hitdt_pd[ipd][jpd]->GetEntries(), h_hitdt_pd[ipd][jpd]->GetMean(), h_hitdt_pd[ipd][jpd]->GetRMS());
    h_hitdt_pd[ipd][jpd]->Fit("f_gaus","Q0","I");
    h_timing_resolution -> Fill(f_gaus->GetParameter(2));
    h_timing_center_resolution -> Fill(f_gaus->GetParameter(1), f_gaus->GetParameter(2));
  }

  TCanvas *c_timing_resolution = new TCanvas("c_timing_resolution","c_timing_resolution",800,600);
  c_timing_resolution -> Divide(2,1);
  c_timing_resolution -> cd(1);
  h_timing_resolution -> Draw();
  c_timing_resolution -> cd(2);
  h_timing_center_resolution -> Draw("colz");
}


void _hists_ana_chargeMPV()
{
  TCanvas *c_charge_MPV = new TCanvas("c_charge_MPV","c_charge_MPV",800,600);
  c_charge_MPV -> Divide(3,2);
  c_charge_MPV -> cd(1);
  constexpr Int_t nset = 5;
  TH1D *gh_charge_MPV[nset];
  TGraphErrors *g_charge_MPV_a[nset], *g_charge_MPV_b[nset];
  TString name_set[nset] = {"cube_top", "cube_bottom", "cube_side", "umbrella", "cortina"};
  Int_t paddle_set[nset+1] = {0, 12, 24, 60, 108, 160};
  TF1Convolution *conv = new TF1Convolution("gaus","landau",0.,20000.,true);
  conv -> SetNofPointsFFT(1000);
  TF1 *f_langau = new TF1("f_langau", *conv, 0, 20000, conv->GetNpar());
  f_langau -> SetParNames("Gaus_Norm","Gaus_Mean", "Gaus_Sigma", "Landau_MPV", "Landau_Sigma");
  f_langau -> SetParLimits(2, 0., 100.);
  f_langau -> SetParLimits(3, 0., 5000.);
  f_langau -> SetParLimits(4, 100., 1000.);
  f_langau -> FixParameter(1, 0.);

  for(Int_t iset=0;iset<nset;++iset){
    gh_charge_MPV[iset] = new TH1D(Form("gh_charge_MPV_%s", name_set[iset].Data()), Form("gh_charge_MPV_%s; Paddle id; charge MPV", name_set[iset].Data()), 161, -0.5, 160.5);
    gh_charge_MPV[iset] -> GetYaxis() -> SetRangeUser(0., 4000.);
    gh_charge_MPV[iset] -> GetXaxis() -> SetRangeUser(paddle_set[iset], paddle_set[iset+1]);
    g_charge_MPV_a[iset] = new TGraphErrors();
    g_charge_MPV_b[iset] = new TGraphErrors();
    Int_t ipoint = 0;
    for(Int_t ipd=paddle_set[iset];ipd<paddle_set[iset+1];++ipd){
      if (h_charge_a[ipd] && h_charge_a[ipd]->GetEntries() > 0) {
        f_langau -> SetParameters(h_charge_a[ipd]->GetEntries(), 0., 10., h_charge_a[ipd]->GetBinCenter(h_charge_a[ipd]->GetMaximumBin()), 150.);
        h_charge_a[ipd]->Fit("f_langau","Q","");
        g_charge_MPV_a[iset] -> SetPoint(ipoint, ipd, f_langau->GetParameter(3));
        g_charge_MPV_a[iset] -> SetPointError(ipoint, 0., f_langau->GetParError(3));
      }

      if (h_charge_b[ipd] && h_charge_b[ipd]->GetEntries() > 0) {
        f_langau -> SetParameters(h_charge_b[ipd]->GetEntries(), 0., 10., h_charge_b[ipd]->GetBinCenter(h_charge_b[ipd]->GetMaximumBin()), 150.);
        h_charge_b[ipd]->Fit("f_langau","Q","");
        g_charge_MPV_b[iset] -> SetPoint(ipoint, ipd, f_langau->GetParameter(3));
        g_charge_MPV_b[iset] -> SetPointError(ipoint, 0., f_langau->GetParError(3));
      }
      ++ipoint;
    }
  }

  for(Int_t iset=0;iset<nset;++iset){
    c_charge_MPV -> cd(iset+1);
    gh_charge_MPV[iset] -> Draw();
    g_charge_MPV_a[iset] -> SetLineColor(1);
    g_charge_MPV_a[iset] -> SetMarkerColor(1);
    g_charge_MPV_a[iset] -> Draw("same p");
    g_charge_MPV_b[iset] -> SetLineColor(2);
    g_charge_MPV_b[iset] -> SetMarkerColor(2);
    g_charge_MPV_b[iset] -> Draw("same p");
  }
  

}



void _hists_ana_sipos_track()
{

  TH1D *h_dx_track_strip[nlayer][nrow][nmodule][nchannel];
  TH1D *h_dy_track_strip[nlayer][nrow][nmodule][nchannel];
  for(Int_t ilayer=0;ilayer<nlayer;++ilayer)
  for(Int_t irow=0;irow<nrow;++irow)
  for(Int_t imodule=0;imodule<nmodule;++imodule)
  for(Int_t ichannel=0;ichannel<nchannel;++ichannel){
    h_dx_track_strip[ilayer][irow][imodule][ichannel] = h_diff_track_strip_xy[ilayer][irow][imodule][ichannel] -> ProjectionX(Form("h_dx_track_strip_%d_%d_%d_%d", ilayer, irow, imodule, ichannel));
    h_dy_track_strip[ilayer][irow][imodule][ichannel] = h_diff_track_strip_xy[ilayer][irow][imodule][ichannel] -> ProjectionY(Form("h_dy_track_strip_%d_%d_%d_%d", ilayer, irow, imodule, ichannel));
    h_dx_track_strip[ilayer][irow][imodule][ichannel] -> SetTitle(Form("h_dx_track_strip_%d_%d_%d_%d;dx;Number of event", ilayer, irow, imodule, ichannel));
    h_dy_track_strip[ilayer][irow][imodule][ichannel] -> SetTitle(Form("h_dy_track_strip_%d_%d_%d_%d;dy;Number of event", ilayer, irow, imodule, ichannel));
    h_dx_track_strip[ilayer][irow][imodule][ichannel] -> SetLineColor(51+int(ichannel%8)*6);
    h_dy_track_strip[ilayer][irow][imodule][ichannel] -> SetLineColor(51+int(ichannel%8)*6);    
  }

  TCanvas *c_dx_track_strip = new TCanvas("c_dx_track_strip","c_dx_track_strip",800,600);
  c_dx_track_strip -> Divide(2,2);
  TCanvas *c_dy_track_strip = new TCanvas("c_dy_track_strip","c_dy_track_strip",800,600);
  c_dy_track_strip -> Divide(2,2);
  Int_t ilayer = 1;
  Int_t irow = 4;
  Int_t imodule = 3;
  for(Int_t ichannel=0;ichannel<nchannel;++ichannel){
    c_dx_track_strip -> cd(ichannel/8+1);
    h_dx_track_strip[ilayer][irow][imodule][ichannel] -> Draw("same hist");
    c_dy_track_strip -> cd(ichannel/8+1);
    h_dy_track_strip[ilayer][irow][imodule][ichannel] -> Draw("same hist");
  }


}


void _hists_ana()
{
  #if HIST_ALLPD
  _hists_ana_tofresolution();
  #endif

  #if TOF_ANA
  _hists_ana_chargeMPV();
  #endif

  #if TRACKING
  _hists_ana_sipos_track();
  #endif
}

// ******************************************************************
// ************************ main function ***************************
// ******************************************************************
vector<TString> GetFileList(Int_t RunNumber){
  TString cmd = Form("ls /home/kaoyama/data/Antarctic_2024/processed/L1/%d/Run%d.gse5_*_*UTC_rec.root > ./filelist.dat", RunNumber, RunNumber);
  gSystem->GetFromPipe(cmd.Data());

  
  vector<TString> filelist;
  ifstream infile("./filelist.dat");
  TString line;
  while (infile.good()) {
    line.ReadLine(infile);
    if (!line.IsNull()) {
      filelist.push_back(line);
    }
  }
  infile.close();

  return filelist;  

}

void Loop(Int_t RunNumber, Int_t StartFile, Int_t EndFile, Bool_t is_Disp=false, Bool_t is_tracking=true)
{
  gStyle->SetOptStat(0);
  gStyle->SetOptFit(0);
  DefineHistLoop1();
  GetDetectorInformation();

  // fout = new TFile(Form("Run%d.root", RunNumber), "RECREATE");

  vector<TString> filelist=GetFileList(RunNumber);
  if(filelist.size() == 0){
    cout << "No file found" << endl;
    return;
  }

  for(Int_t ifile=StartFile;ifile<=EndFile;++ifile){
     LoadRootFile(filelist[ifile]);
    Int_t nevent=tree->GetEntries();
    cout << "Start process for: " << filelist[ifile] << ", " << nevent << "event" << endl;

    clock_t start = clock();
    for(Int_t ievent=0;ievent<nevent;++ievent){
      Bool_t is_plot = Plot(ievent, is_Disp, true);
      if(is_tracking) _LinearFit(is_Disp);
      if(chi2_res>2.) continue;
      _Fill();
    }
    clock_t end = clock();
    cout << (end-start)/1000. << "ms" << endl;
  }

  DrawLoop1();
  _hists_ana();
}

