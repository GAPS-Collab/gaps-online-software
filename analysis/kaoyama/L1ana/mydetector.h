// ver.GAPS tof
// functions to get detector informations
#include <iostream>
#include <fstream>
#include "TMarker3DBox.h"
#include "TVector3.h"

// for gaps-online-software
#include "io.hpp"
#include "calibration.h"
#include "database.h"
#include "caraspace.hpp"


// detector information
const Double_t v_schinti = 15.4; // [cm/ns]
const Double_t v_cable = 20.; // [cm/ns]
const Double_t width = 16.; // cm
const Double_t thickness = 0.635; //cm

TVector3 mppc_pos_a[160];
TVector3 mppc_pos_b[160];
Double_t coax_cable_time[160];
Double_t harting_cable_time[160];
void _GetMPPCInformation(){
    auto paddles = Gaps::get_tofpaddles();
    for (auto const &p : paddles) {
        Int_t ipaddle = (unsigned long)p.second.paddle_id-1;
        mppc_pos_a[ipaddle].SetXYZ(p.second.global_pos_x_l0_A,p.second.global_pos_y_l0_A,p.second.global_pos_z_l0_A);
        mppc_pos_b[ipaddle].SetXYZ(p.second.global_pos_x_l0_B,p.second.global_pos_y_l0_B,p.second.global_pos_z_l0_B);
        coax_cable_time[ipaddle] = p.second.coax_cable_time;
        harting_cable_time[ipaddle] = p.second.harting_cable_time;
        std::cout << "************* PADDLE " << ipaddle << "***********" << std::endl;
        std::cout << mppc_pos_a[ipaddle].X() << ", " << mppc_pos_a[ipaddle].Y() << ", " << mppc_pos_a[ipaddle].Z() << ", " << std::endl;
        std::cout << mppc_pos_b[ipaddle].X() << ", " << mppc_pos_b[ipaddle].Y() << ", " << mppc_pos_b[ipaddle].Z() << ", " << std::endl;
        std::cout << coax_cable_time[ipaddle] << " + " << harting_cable_time[ipaddle] << std::endl;
        std::cout << "\n\n" << std::endl; 
    }
}


TMarker3DBox *tbTof[160];
void _GetToFVolume()
{
  for (int i=0;i<160;++i){
    TVector3 center = 0.5 * (mppc_pos_a[i] + mppc_pos_b[i]);
    TVector3 unit = (mppc_pos_a[i] - mppc_pos_b[i]).Unit();
    TVector3 yunit(0.,1.,0.);
    double length = (mppc_pos_a[i] - mppc_pos_b[i]).Mag();

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


TVector3 tof_position_error[160];
void _SetPositionErrors()
{
    for (int i=0;i<160;++i){
        Double_t e_length   = v_schinti*1./2.;    // cm
        Double_t e_width    = width/sqrt(12.);     // cm
        // Double_t e_thickness= thickness/sqrt(12.); // cm
        Double_t e_thickness= sqrt( pow(thickness/sqrt(12.), 2) + pow(2., 2) ); // cm


        TVector3 unit = (mppc_pos_a[i] - mppc_pos_b[i]).Unit();
        TVector3 xyunit(1.,1.,0.);
        if(i<=23||(i>=60&&i<=107)){// cube top pd1-12 & bottom pd13-24 & umbrella pd61-108
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
    _GetMPPCInformation();
    _GetToFVolume();
    _SetPositionErrors();
}