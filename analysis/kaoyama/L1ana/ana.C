// ver.GAPS tof 
// main analysis file
#include "mydetector.h" // ver.GAPS tof
#include "myfunc.h" // ver.GAPS tof
#include "myheader.h" // ver.GAPS tof

#include "TMinuit.h"
#include "TPolyLine3D.h"

void Reconstrunct(Int_t ievent, Bool_t is_Disp, Bool_t is_tracking);


// measured parameters
vector<int> vhit_paddle_num;
vector<TVector3> vhit_paddle_pos;
vector<double> vhit_paddle_time;
vector<double> vhit_sipma_time;
vector<double> vhit_sipmb_time;
vector<double> vhit_paddle_scale;
vector<double> vhit_sipma_scale;
vector<double> vhit_sipmb_scale;
vector<double> vhit_ch9phase_scale;
Double_t x0_res, y0_res, z0_res, theta_res, phi_res;
Double_t x0_res_err, y0_res_err, z0_res_err, theta_res_err, phi_res_err;

void _GetPhysicsParameters()
{
  vhit_paddle_num.clear();
  vhit_paddle_pos.clear();
  vhit_paddle_time.clear();
  vhit_sipma_time.clear();
  vhit_sipmb_time.clear();
  vhit_paddle_scale.clear();
  vhit_sipma_scale.clear();
  vhit_sipmb_scale.clear();
  
  for(Int_t ipd=0;ipd<npaddle;++ipd){
    // parametes from row level waveform
    // vhit_paddle_num.push_back(Tof->wfs.paddle_id[ipd*2]);
    vhit_paddle_num.push_back(1);
    vhit_sipma_time.push_back(GetMPPCHitTiming(hWaveform_a[ipd]));
    vhit_sipmb_time.push_back(GetMPPCHitTiming(hWaveform_b[ipd]));
    vhit_sipma_scale.push_back(GetMPPCCharge(hWaveform_a[ipd]));
    vhit_sipmb_scale.push_back(GetMPPCCharge(hWaveform_b[ipd]));
    vhit_ch9phase_scale.push_back(Tof->hits.phase[ipd*2]/(2.*TMath::Pi())*50.);

    // parametes calculated by concerning some correction factor
    vhit_paddle_scale.push_back(vhit_sipma_scale.back()+vhit_sipmb_scale.back());
    vhit_paddle_pos.push_back(_GetPaddleHitPosition(vhit_sipma_time.back(), vhit_sipmb_time.back(), vhit_paddle_num.back()));
    // vhit_paddle_time.push_back(_GetPaddleHitTiming(vhit_sipma_time.back()-phase_9->at(ipd), vhit_sipmb_time.back()-phase_9->at(ipd), vhit_paddle_num.back()));
  }
}

void _FillPhysicsParameters(Bool_t is_tracking=true)
{
  for(Int_t ipd=0;ipd<npaddle;++ipd){

    if(vhit_sipma_scale.at(ipd)<500. || vhit_sipmb_scale.at(ipd)<500.) continue;
    if(vhit_sipma_scale.at(ipd)>4000. || vhit_sipmb_scale.at(ipd)>4000.) continue;
    if(vhit_sipma_time.at(ipd)<10. || vhit_sipmb_time.at(ipd)<10.) continue;


    // h_rb_id -> Fill(Tof->wfs.rb_id[ipd*2]);
    h_pd_id -> Fill(vhit_paddle_num.at(ipd));
    h_hitdt_ab[vhit_paddle_num.at(ipd)-1]    -> Fill(vhit_sipmb_time.at(ipd)-vhit_sipma_time.at(ipd));
    h_hittime_ab[vhit_paddle_num.at(ipd)-1]  -> Fill(vhit_sipma_time.at(ipd),vhit_sipmb_time.at(ipd));
    h_hittime_a[vhit_paddle_num.at(ipd)-1]   -> Fill(vhit_sipma_time.at(ipd));
    h_hittime_b[vhit_paddle_num.at(ipd)-1]   -> Fill(vhit_sipmb_time.at(ipd));
    h_charge_a[vhit_paddle_num.at(ipd)-1]    -> Fill(vhit_sipma_scale.at(ipd));
    h_charge_b[vhit_paddle_num.at(ipd)-1]    -> Fill(vhit_sipmb_scale.at(ipd));
    h_charge_ab[vhit_paddle_num.at(ipd)-1]   -> Fill(vhit_sipma_scale.at(ipd),vhit_sipmb_scale.at(ipd));
    h_dt_charge_a[vhit_paddle_num.at(ipd)-1] -> Fill(vhit_sipmb_time.at(ipd)-vhit_sipma_time.at(ipd), vhit_sipma_scale.at(ipd));
    h_dt_charge_b[vhit_paddle_num.at(ipd)-1] -> Fill(vhit_sipmb_time.at(ipd)-vhit_sipma_time.at(ipd), vhit_sipmb_scale.at(ipd));
    h_phase_ch9[vhit_paddle_num.at(ipd)-1]   -> Fill(vhit_ch9phase_scale.at(ipd));

  }

  cout << "ok" << endl;
  Int_t contains1 = is_contain(&vhit_paddle_num, 68);
  Int_t contains2 = is_contain(&vhit_paddle_num, 9);
  if(contains1!=-1&&contains2!=-1){
    if(vhit_sipma_scale.at(contains1)<500. || vhit_sipmb_scale.at(contains1)<500.) return;
    if(vhit_sipma_scale.at(contains2)<500. || vhit_sipmb_scale.at(contains2)<500.) return;

    Double_t tof_l = (vhit_paddle_pos[contains2]-vhit_paddle_pos[contains1]).Mag();

    // if(tof_l<95||tof_l>100) return;
    h_tof_length -> Fill(tof_l);
    h_hitdt_pd -> Fill(vhit_paddle_time[contains2]-vhit_paddle_time[contains1]);
    h_tof_length_hitdt -> Fill(tof_l, vhit_paddle_time[contains2]-vhit_paddle_time[contains1]);
  }

  cout << "ok" << endl;

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
    h_track_theta_cut1 -> Fill(theta_res/TMath::Pi()*180.);
    h_track_theta_mod_cut1 -> Fill(TMath::Cos(theta_res));
    h_track_phi_cut1 -> Fill(phi_res/TMath::Pi()*180.);
    h_track_chi2_cut1 -> Fill(chi2_res);
    h_track_x0_cut1 -> Fill(x0_res);
    h_track_y0_cut1 -> Fill(y0_res);
    h_track_z0_cut1 -> Fill(z0_res);
  }

}

void Loop(Int_t RunNumber, Int_t StartFileNumber, Int_t EndFileNumber, Bool_t is_Disp=false)
{
  _DefineHistLoop1();
  GetDetectorInformation();

  for(Int_t ifile=StartFileNumber;ifile<=EndFileNumber;++ifile){
    LoadRootFile(RunNumber, ifile);
    Int_t nevent=tree->GetEntries();
    cout << "Start process for Run" << RunNumber << "-file" << ifile << ", " << nevent << "event" << endl;

    for(Int_t ievent=0;ievent<1000;++ievent){
    // for(Int_t ievent=0;ievent<nevent;++ievent){
      Plot(ievent, is_Disp, true);
      if(npaddle<=1) continue; 
      else if(npaddle>8) continue;
      // _GetPhysicsParameters();
      Reconstrunct(ievent, is_Disp, true);
      _FillPhysicsParameters();
    }
  }

  // ** temporary canvas  
  Int_t paddlenum=68;
  TCanvas *cSummary = new TCanvas("cSummary", "cSummary", 1000,800);
  cSummary -> Divide(4,4);
  cSummary -> cd(1);
  h_npaddle -> Draw();
  cSummary -> cd(2);
  h_rb_id -> Draw();
  cSummary -> cd(3);
  h_pd_id -> Draw();
  cSummary -> cd(4);
  h_hittime_a[paddlenum] -> Draw();
  h_hittime_b[paddlenum] ->SetLineColor(2);
  h_hittime_b[paddlenum] -> Draw("same");
  cSummary -> cd(5);
  h_hitdt_ab[paddlenum] -> Draw();
  cSummary -> cd(6);
  h_charge_a[paddlenum] -> Draw();
  h_charge_b[paddlenum] -> SetLineColor(2);
  h_charge_b[paddlenum] -> Draw("same");
  cSummary -> cd(7);
  h_charge_ab[paddlenum] -> Draw("colz");
  cSummary -> cd(8);
  h_dt_charge_a[paddlenum] -> Draw("colz");
  cSummary -> cd(9);
  h_dt_charge_b[paddlenum] -> Draw("colz");
  cSummary -> cd(10);
  h_hitpos_ab[paddlenum] -> Draw();
  cSummary -> cd(11);
  h_hittime_ab[paddlenum] -> Draw();
  h_hittime_ab_cut[paddlenum] -> SetLineColor(2);
  h_hittime_ab_cut[paddlenum] -> Draw("same");

  cSummary -> cd(12);
  h_hitdt_pd -> Draw();
  cSummary -> cd(13);
  h_tof_length -> Draw();
  cSummary -> cd(14);
  h_tof_length_hitdt -> Draw();

  TCanvas *cTracking = new TCanvas("cTracking", "cTracking", 1000,800);
  cTracking -> Divide(3,2);
  cTracking -> cd(1);
  h_track_theta -> Draw();
  h_track_theta_cut1 -> Draw("same");
  cTracking -> cd(2);
  h_track_phi -> Draw();
  h_track_phi_cut1 -> Draw("same");
  cTracking -> cd(3);
  h_track_chi2 -> Draw();
  h_track_chi2_cut1 -> Draw("same");
  cTracking -> cd(4);
  h_track_x0 -> Draw();
  h_track_x0_cut1 -> Draw("same");
  cTracking -> cd(5);
  h_track_y0 -> Draw();
  h_track_y0_cut1 -> Draw("same");
  cTracking -> cd(6);
  h_track_theta_mod -> Draw();
  h_track_theta_mod_cut1 -> Draw("same");


  cout << "finish event loop" << endl;

}


void Reconstrunct(Int_t ievent, Bool_t is_Disp=true, Bool_t is_tracking=true)
{

  Plot(ievent, false, true);
  if(npaddle<=1){
    cout << "no waveform" << endl;
    return;
  } 
  else if(npaddle>10){
    cout << "too many waveform" << endl;
    return;
  }
  _GetPhysicsParameters();

  delete gDetectorHit;
  gDetectorHit = new TGraph2DErrors();
  delete gDetectorHit_XY;
  gDetectorHit_XY = new TGraphErrors();
  delete gDetectorHit_XZ;
  gDetectorHit_XZ = new TGraphErrors();
  delete gDetectorHit_YZ;
  gDetectorHit_YZ = new TGraphErrors();
  for(Int_t ipd=0;ipd<npaddle;++ipd){
    gDetectorHit    -> SetPoint(ipd, vhit_paddle_pos[ipd].X(), vhit_paddle_pos[ipd].Y(), vhit_paddle_pos[ipd].Z());
    gDetectorHit_XY -> SetPoint(ipd, vhit_paddle_pos[ipd].X(), vhit_paddle_pos[ipd].Y());
    gDetectorHit_XZ -> SetPoint(ipd, vhit_paddle_pos[ipd].X(), vhit_paddle_pos[ipd].Z());
    gDetectorHit_YZ -> SetPoint(ipd, vhit_paddle_pos[ipd].Y(), vhit_paddle_pos[ipd].Z());

    Double_t ex = tof_position_error[vhit_paddle_num[ipd]-1].X();
    Double_t ey = tof_position_error[vhit_paddle_num[ipd]-1].Y();
    Double_t ez = tof_position_error[vhit_paddle_num[ipd]-1].Z();
    gDetectorHit    -> SetPointError(ipd, ex, ey, ez);
    gDetectorHit_XY -> SetPointError(ipd, ex, ey);
    gDetectorHit_XZ -> SetPointError(ipd, ex, ez);
    gDetectorHit_YZ -> SetPointError(ipd, ey, ez);
  }

  // linear fitting
  if(is_tracking){
    delete fxz;
    fxz = new TF1("fxz","pol1", -1.e5, 1.e5);
    delete fyz;
    fyz = new TF1("fyz","pol1", -1.e5, 1.e5);
    delete fxy;
    fxy = new TF1("fxy","pol1", -200., 200.);
    fxy->SetLineColor(kBlue);
    fxz->SetLineColor(kBlue);
    fyz->SetLineColor(kBlue);

    gDetectorHit_XZ -> Fit("fxz","Q", "",-200,200);
    gDetectorHit_YZ -> Fit("fyz","Q", "",-200,200);
    // gDetectorHit_XY -> Fit("fxy","Q","0");
    {
      Double_t z0 = fxz->Eval(0);
      // Double_t y0 = fyz->GetX(z0, -200., 200.);
      Double_t y0 = fyz->GetX(z0);
      Double_t a_xz_init = fxz->GetParameter(1);
      Double_t a_yz_init = fyz->GetParameter(1);
      TVector3 dir(1./a_xz_init, 1./a_yz_init, 1.);
      dir = dir.Unit();
      fxy -> SetParameters(y0, dir.Y()/dir.X());
    }

    delete minuit;
    minuit = new TMinuit(5);
    minuit->SetPrintLevel(-1);
    minuit->SetFCN(fitFunction);
    
    Double_t min_horizontal_orig_point = -1e9;
    Double_t max_horizontal_orig_point = 1e9;
    Double_t fix_cubetop_orig_point = 110.39;
    Double_t x0_init = fxz->GetX(fix_cubetop_orig_point, min_horizontal_orig_point, max_horizontal_orig_point);
    Double_t y0_init = fyz->GetX(fix_cubetop_orig_point, min_horizontal_orig_point, max_horizontal_orig_point);
    Double_t z0_init = fix_cubetop_orig_point;
    Double_t a_xz_init = fxz->GetParameter(1);
    Double_t a_yz_init = fyz->GetParameter(1);
    Double_t a_xy_init = fxy->GetParameter(1);
    TVector3 dir(1, a_xy_init, a_xz_init);
    dir = dir.Unit();
    Double_t theta_init = TMath::ACos(dir.Z());
    if (theta_init > TMath::Pi()/2){
      theta_init = TMath::Pi() - theta_init;
    }
    Double_t phi_init = TMath::ATan2(dir.Y(), dir.X());
    minuit->DefineParameter(0, "x0", x0_init, 0.1, min_horizontal_orig_point, max_horizontal_orig_point);
    minuit->DefineParameter(1, "y0", y0_init, 0.1, min_horizontal_orig_point, max_horizontal_orig_point);
    minuit->DefineParameter(2, "z0", z0_init, 0.5, -50., 250.);
    minuit->DefineParameter(3, "theta", theta_init, 0.1, 0, TMath::Pi()/2.);
    minuit->DefineParameter(4, "phi", phi_init, 0.1, -1.*TMath::Pi(), TMath::Pi());
    
    minuit->FixParameter(2);

    minuit->Migrad();

    minuit->GetParameter(0, x0_res, x0_res_err);
    minuit->GetParameter(1, y0_res, y0_res_err);
    minuit->GetParameter(2, z0_res, z0_res_err);
    minuit->GetParameter(3, theta_res, theta_res_err);
    minuit->GetParameter(4, phi_res, phi_res_err);

  }


  if(is_Disp){

    cout << endl << "**** minimization result ****" << endl;
    cout << "- status: " << minuit->fCstatu << endl;
    cout << "- chi2  : " << chi2_res  << endl;
    cout << "- x0    : " << x0_res    << "\t +- " << x0_res_err    << endl;
    cout << "- y0    : " << y0_res    << "\t +- " << y0_res_err    << endl;
    cout << "- z0    : " << z0_res    << "\t +- " << z0_res_err    << endl;
    cout << "- theta : " << theta_res << "\t +- " << theta_res_err << endl;
    cout << "- phi   : " << phi_res   << "\t +- " << phi_res_err   << endl;
    cout << "*****************************" << endl << endl;

    if(!gROOT->FindObject("cDetectorHit")){
      delete cDetectorHit;
      cDetectorHit = new TCanvas("cDetectorHit","cDetectorHit", 800, 800);
      cDetectorHit -> Divide(2,2);
      cDetectorHitProp = new TCanvas("cDetectorHitProp","cDetectorHitProp", 500, 500);

      // 3d plot
      delete hDetectorHit;
      hDetectorHit = new TH3D("hDetectorHit","hDetectorHit;x;y;z",100,-200.,200.,100,-200.,200.,100,-50.,250.);
      hDetectorHit -> SetStats(0);
      // 2d plot
      delete hDetectorHit_d;
      hDetectorHit_d = new TH2D("hDetectorHit_d","hDetectorHit_d;x;y",100,-200.,200.,100,-200.,200.);
      hDetectorHit_d -> SetStats(0);
      delete hDetectorHit_XY;
      hDetectorHit_XY = new TH2D("hDetectorHit_XY","hDetectorHit_XY;x;y",100,-200.,200.,100,-200.,200.);
      hDetectorHit_XY -> SetStats(0);
      delete hDetectorHit_XZ;
      hDetectorHit_XZ = new TH2D("hDetectorHit_XZ","hDetectorHit_XZ;x;z",100,-200.,200.,100,-50.,250.);
      hDetectorHit_XZ -> SetStats(0);
      delete hDetectorHit_YZ;
      hDetectorHit_YZ = new TH2D("hDetectorHit_YZ","hDetectorHit_YZ;y;z",100,-200.,200.,100,-50.,250.);
      hDetectorHit_YZ -> SetStats(0);
    }
    cDetectorHit -> cd(1);
    hDetectorHit -> Reset();
    for(Int_t ipd=0;ipd<npaddle;++ipd){
      cout << ipd << ": " << vhit_paddle_pos[ipd].X() << ", " << vhit_paddle_pos[ipd].Y() << ", " << vhit_paddle_pos[ipd].Z() << endl;
      hDetectorHit -> Fill(vhit_paddle_pos[ipd].X(), vhit_paddle_pos[ipd].Y(), vhit_paddle_pos[ipd].Z());
    }
    hDetectorHit -> Draw("box");
    for (int i = 0; i < 160; ++i){
      tbTof[i] -> Draw("same");
    }
    cDetectorHit -> cd(2);
    hDetectorHit_XY -> Draw("");
    gDetectorHit_XY -> Draw("same p");
    cDetectorHit -> cd(3);
    hDetectorHit_XZ -> Draw("");
    gDetectorHit_XZ -> Draw("same p");
    cDetectorHit -> cd(4);
    hDetectorHit_YZ -> Draw("");
    gDetectorHit_YZ -> Draw("same p");

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
    fxy -> SetLineStyle(9);
    fxy -> Draw("same");
    fxy_res->Draw("same");

    cDetectorHit -> cd(3);
    fxz_res->SetParameters(z0_res-x0_res*cos(theta_res)/(sin(theta_res)*cos(phi_res)), cos(theta_res)/(sin(theta_res)*cos(phi_res)));
    fxz_res->SetLineColor(kRed);
    fxz -> SetLineStyle(9);
    fxz->Draw("same");
    fxz_res->Draw("same");

    cDetectorHit -> cd(4);
    fyz_res->SetParameters(z0_res-y0_res*cos(theta_res)/(sin(theta_res)*sin(phi_res)), cos(theta_res)/(sin(theta_res)*sin(phi_res)));
    fyz_res->SetLineColor(kRed);
    fyz -> SetLineStyle(9);
    fyz->Draw("same");
    fyz_res->Draw("same");


  }

}


// tracking functions
void fitFunction(Int_t &npar, Double_t *gin, Double_t &f, Double_t *par, Int_t iflag){
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

      double t = ( (x - x0) * x1 + (y - y0) * y1 + (z - z0) * z1 );

      double sigma_x = gDetectorHit->GetEX()[i];
      double sigma_y = gDetectorHit->GetEY()[i];
      double sigma_z = gDetectorHit->GetEZ()[i];

      double x_fit = x0 + x1 * t;
      double y_fit = y0 + y1 * t;
      double z_fit = z0 + z1 * t;
      
      double dx = (x - x_fit) / sigma_x;
      double dy = (y - y_fit) / sigma_y;
      double dz = (z - z_fit) / sigma_z;
      
      chi2 += dx * dx + dy * dy + dz * dz;
  }
  chi2_res = chi2/(gDetectorHit->GetN());
  f = chi2_res;
}