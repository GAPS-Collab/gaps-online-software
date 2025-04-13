// ver.GAPS merge
// functions to get detector informations

// detector constants
// tof
constexpr Double_t v_light = 29.9792458; // [cm/ns]
constexpr Double_t v_schinti = 15.4; // [cm/ns]
constexpr Double_t v_cable = 20.; // [cm/ns]
constexpr Double_t width = 16.; // cm
constexpr Double_t thickness = 0.635; //cm
constexpr Int_t nrb=50;
constexpr Int_t nch=8;
constexpr Int_t npd=160;
// si
constexpr Int_t nlayer = 10;
constexpr Int_t nrow = 6;
constexpr Int_t nmodule = 6;
constexpr Int_t nchannel = 32;

// detector positions
TVector3 mppc_pos_a[160];
TVector3 mppc_pos_b[160];
Double_t coax_cable_time[160];
Double_t harting_cable_time[160];
TVector3 module_pos[10][6][6][32];
TVector3 strip_pos[10][6][6][32];

// detector volumes
TMarker3DBox *tbTof[160];
TPolyLine3D *tbSi[10][6][6][32];


void _GetMPPCInformation()
{
    auto paddles = Gaps::get_tofpaddles();
    for (auto const &[_,p] : paddles) {
        Int_t ipd = (unsigned long)p.paddle_id-1;
        mppc_pos_a[ipd].SetXYZ(p.global_pos_x_l0_A,p.global_pos_y_l0_A,p.global_pos_z_l0_A);
        mppc_pos_b[ipd].SetXYZ(p.global_pos_x_l0_B,p.global_pos_y_l0_B,p.global_pos_z_l0_B);
        coax_cable_time[ipd] = p.coax_cable_time;
        harting_cable_time[ipd] = p.harting_cable_time;
    }
}


void _GetSiInformation()
{
    auto strips = Gaps::get_trackerstrips();
    constexpr double dw[8] = {-3.72, -2.365, -1.37, -0.45, 0.45, 1.37, 2.365, 3.72}; // cm
    for (auto const &[_,s] : strips) {
        Int_t ilayer = s.layer;
        Int_t irow = s.row;
        Int_t imodule=s.module;
        Int_t ichannel = s.channel;
        Double_t dx=0., dy=0., dz=-20.;
        // if(ilayer%2==0){
        //     if(ichannel/8==0) dx = dw[ichannel%8];
        //     else if(ichannel/8==1) dx = -dw[ichannel%8];
        //     else if(ichannel/8==2) dx = -dw[ichannel%8];
        //     else if(ichannel/8==3) dx = dw[ichannel%8];
        //     else cout << "error: channel number is wrong" << endl;
        // } else {
        //     if(ichannel/8==0) dy = -dw[ichannel%8];
        //     else if(ichannel/8==1) dy = dw[ichannel%8];
        //     else if(ichannel/8==2) dy = dw[ichannel%8];
        //     else if(ichannel/8==3) dy = -dw[ichannel%8];
        //     else cout << "error: channel number is wrong" << endl;
        // }
        strip_pos[ilayer][irow][imodule][ichannel].SetXYZ(s.global_pos_x_det_l0+dx, s.global_pos_y_det_l0+dy, s.global_pos_z_det_l0+dz);
        module_pos[ilayer][irow][imodule][ichannel].SetXYZ(s.global_pos_x_l0, s.global_pos_y_l0, s.global_pos_z_l0+dz);
    }
}



void _GetToFVolume()
{
    TVector3 yunit(0.,1.,0.);
    for (int i=0;i<160;++i){
        TVector3 center = 0.5 * (mppc_pos_a[i] + mppc_pos_b[i]);
        TVector3 unit = (mppc_pos_a[i] - mppc_pos_b[i]).Unit();
        Double_t length = (mppc_pos_a[i] - mppc_pos_b[i]).Mag();

        if(i<=23){// cube top pd1-12 & bottom pd13-24
            tbTof[i] = new TMarker3DBox(center.X(), center.Y(), center.Z(), width/2., length/2., thickness/2., 0., unit.Angle(yunit)/TMath::Pi()*180.);
            tbTof[i]->SetLineColor(kGray);
        }
        else if(i<=55){// cube side pd25-56
            tbTof[i] = new TMarker3DBox(center.X(), center.Y(), center.Z(), width/2., length/2., thickness/2., 90., unit.Angle(yunit)/TMath::Pi()*180.);
            tbTof[i]->SetLineColor(kGray);
        }
        else if(i<=59){// corner pd57-60
            tbTof[i] = new TMarker3DBox(center.X(), center.Y(), center.Z(), length/2., width/2., thickness/2., 90., 45.*pow(-1, i));
            tbTof[i]->SetLineColor(kGray);
        }
        else if(i<=107){// umbrella pd61-108
            tbTof[i] = new TMarker3DBox(center.X(), center.Y(), center.Z(), width/2., length/2., thickness/2., 0., unit.Angle(yunit)/TMath::Pi()*180.);
            tbTof[i]->SetLineColor(kCyan);
        }
        else if(i <= 147){// cortina pd109-149
            tbTof[i] = new TMarker3DBox(center.X(), center.Y(), center.Z(), width/2., length/2., thickness/2., 90., unit.Angle(yunit)/TMath::Pi()*180.);
            tbTof[i]->SetLineColor(kGreen);
        }
        else {// cortina corner pd152-160
            tbTof[i] = new TMarker3DBox(center.X(), center.Y(), center.Z(), length/2., width/2., thickness/2., 90., 45.*pow(-1,int((i+2)/3)));
            tbTof[i]->SetLineColor(kGreen);
        }
        tbTof[i]->SetLineWidth(1);
    }
}


void _GetSiVolume()
{
    constexpr int n = 6;
    constexpr double r = 5.0;
    for (int ilayer = 0; ilayer < nlayer; ++ilayer)
    for (int irow = 0; irow < nrow; ++irow)
    for (int imodule = 0; imodule < nmodule; ++imodule)
    for (int ichannel = 0; ichannel < 32; ++ichannel) {
        double x[n], y[n], z[n];
        // auto& pos = module_pos[ilayer][irow][imodule][ichannel];
        auto& pos = strip_pos[ilayer][irow][imodule][ichannel];
        for (int i = 0; i < n; i++) {
            double theta = TMath::TwoPi() * i / n;
            x[i] = pos.X() + r*cos(theta);
            y[i] = pos.Y() + r*sin(theta);
            z[i] = pos.Z();
        }

        auto& tb = tbSi[ilayer][irow][imodule][ichannel];
        tb = new TPolyLine3D(n, x, y, z);
        tb -> SetLineColor(kOrange);
        tb -> SetLineWidth(1);
        tb -> SetBit(kCanDelete, false);
    }
}


TVector3 tof_position_error[160];
void _SetToFPositionErrors()
{
    Double_t e_length   = v_schinti*0.6/2.;    // cm
    Double_t e_width    = width/sqrt(12.);     // cm
    Double_t e_thickness= sqrt( pow(thickness/sqrt(12.), 2) + pow(1., 2) ); // cm
    // Double_t e_width    = width;     // cm
    // Double_t e_thickness= thickness; // cm
    for (int i=0;i<160;++i){
        
        TVector3 unit = (mppc_pos_a[i] - mppc_pos_b[i]).Unit();
        unit.SetXYZ(unit.X()*unit.X(), unit.Y()*unit.Y(), unit.Z()*unit.Z());
        TVector3 xyunit(1.,1.,0.);
        if(i<=23||(i>=60&&i<=107)){// cube top pd1-12 & cube bottom pd13-24 & umbrella pd61-108
            tof_position_error[i] = unit* e_length\
                                    + (xyunit-unit)* e_width\
                                    + TVector3(0, 0., e_thickness);
        }
        else if((i>=24&&i<=55)||(i>=108&&i<=148)){// cube side pd25-56 & // cortina pd109-149
            tof_position_error[i] = unit* e_length\
                                    + (xyunit-unit)* e_thickness\
                                    + TVector3(0, 0., e_width);
        }
        else {// corner pd57-60 & cortina corner pd152-160
            tof_position_error[i].SetXYZ(sqrt(pow(e_width,2)+pow(e_thickness,2)), sqrt(pow(e_width,2)+pow(e_thickness,2)), e_length);
        }
    }
}



void GetDetectorInformation()
{
    // for tof paddles
    _GetMPPCInformation();
    _GetToFVolume();
    _SetToFPositionErrors();

    // for tracker strips
    _GetSiInformation();
    _GetSiVolume();
}