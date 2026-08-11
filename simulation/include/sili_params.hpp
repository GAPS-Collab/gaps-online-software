#ifndef SILI_PARAMS_H_INCLUDED
#define SILI_PARAMS_H_INCLUDED

#include <numbers>
#include <algorithm>
#include "G4SystemOfUnits.hh"

#include "tof_typedefs.h"

namespace gondola {

  /// Relevant parameters for SiLi detector wafers
  /// Density & mass calculations from F.Rogers, 2018
  struct SiLiParams {
    f32 mass_pi           = 0.036  *CLHEP::g;
    f32 thickness_n_layer = 0.01   *CLHEP::cm;
    f32 thickness_p_layer = 0.01   *CLHEP::cm;
    f32 thickness_ni      = 0.00002*CLHEP::mm;
    f32 thickness_au      = 0.0001 *CLHEP::mm;
    f32 radius_wafer      = 5      *CLHEP::cm   ;// - 3*mm;
    f32 radius_guardring  = 4.81   *CLHEP::cm;// - 3*mm;
    f32 radius_active     = 4.48   *CLHEP::cm;
    f32 thickness_det     = 0.249  *CLHEP::cm;
    f32 depth_guardring   = 0.17   *CLHEP::cm;
    f32 groove_fraction   = 0.99; 
    auto strip_area() -> f32 {
      return std::numbers::pi_v<float>*radius_active*radius_active/2/8;
    }

    auto strip_widths() -> Vec<f32> {
      Vec<f32> widths = 
        {16.34907456*CLHEP::mm, 10.32502464*CLHEP::mm,
          9.23299776*CLHEP::mm,  8.84362752*CLHEP::mm,
          8.84362752*CLHEP::mm,  9.23299776*CLHEP::mm,
         10.32502464*CLHEP::mm, 16.34907456*CLHEP::mm};
       return widths;
    }

    /// We assume the HV connector is to the left,
    /// see ../resources/mapping_asic_strips.jpg also
    /// Then we label the strips from left to right
    /// when looking at them from the top. The 
    /// leftmost strip is A
    auto get_strip_label(u16 ch) -> std::string {
      // 0,8,16,24
      std::string lbl = "X";
      if (ch == 7 || ch == 8  || ch == 31  || ch == 16) {
        lbl = "A";
      }  
      if (ch == 6 || ch == 9  || ch == 30  || ch == 17)  {
        lbl = "B";
      }  
      if (ch == 5 || ch == 10 || ch == 29 || ch == 18)  {
        lbl = "C";
      }  
      if (ch == 4 || ch == 11 || ch == 28 || ch == 19)  {
        lbl = "D";
      }  
      if (ch == 3 || ch == 12 || ch == 27 || ch == 20)  {
        lbl = "E";
      }  
      if (ch == 2 || ch == 13 || ch == 26 || ch == 21)  {
        lbl = "F";
      }  
      if (ch == 1 || ch == 14 || ch == 25 || ch == 22)  {
        lbl = "G";
      }  
      if (ch == 0 || ch == 15 || ch == 24 || ch == 23)  {
        lbl = "H";
      }  
      return lbl;
    }

    /// The distance a strip center is away from 
    /// the center of the disk
    /// Distances to the left are negative
    auto det_center_distance() -> Vec<f32> {
      //positive distance from cent
      Vec<f32> dfc = {};
      //dist_from_edge.push_back(-strip_widths[0]/2 - strip_widths[1]
      //                         -strip_widths[2]   - strip_widths[3]);
      //dist_from_edge.push_back(strip_widths[1]/2 - strip_widths[2]
      //                         -strip_widths[3]);
      //dist_from_edge.push_back(strip_widths[2]/2 
      //                         -strip_widths[3]);
      //dist_from_edge.push_back(strip_widths[3]/2);
      auto sw = strip_widths();
      dfc.push_back( sw[4]/2);
      dfc.push_back( sw[4] 
                    +sw[5]/2);
      dfc.push_back( sw[4] 
                    +sw[5]
                    +sw[6]/2);
      dfc.push_back(sw[4] 
                   +sw[5]
                   +sw[6]
                   +sw[7]/2);
      Vec<f32> dist_from_cent = {-dfc[3], -dfc[2], -dfc[1], -dfc[0],
                                  dfc[0], dfc[1],   dfc[2], dfc[3]};
      return dist_from_cent;
    }

    auto top_surface_area() -> f32 {
      f32 area = 
        mass_pi*radius_guardring*radius_guardring-0.1
        *cm*(2*radius_active*mass_pi+2*radius_active
        +2*8.75*cm+2*8.2*cm+2*6.9*cm);
      return area;
    }
  
    auto density_toplayer() -> f32 {
      f32 density = 
           (mass_pi+top_surface_area()*
           (2.336*thickness_n_layer+
            58.6934*thickness_ni+
            96.97*thickness_au)*g/cm3)
          /(mass_pi*pow(radius_guardring,2)
              *thickness_n_layer);
      return density;
    }
    
    auto mass_fraction_si_top() -> f32 {
      f32 frac = 
        2.336*thickness_n_layer*top_surface_area()*(g/cm3)
        /(mass_pi+top_surface_area()*
        (96.97*thickness_au+58.6934*thickness_ni+2.336*thickness_n_layer)*g/cm3);//mass au / total m
      return frac;
    }
  
    auto mass_fraction_ni_top() -> f32 {
      f32 frac = 
        58.6934*thickness_ni*top_surface_area()*(g/cm3)/
        (mass_pi+top_surface_area()*
        (96.97*thickness_au+58.6934*thickness_ni+2.336*thickness_n_layer)*g/cm3); // mass gold /total m
      return frac;
    }
  
    auto mass_fraction_au_top() -> f32 {
      f32 frac = 
        thickness_au*96.97*top_surface_area()*(g/cm3)/
        (mass_pi+top_surface_area()*
        (96.97*thickness_au+58.6934*thickness_ni+2.336*thickness_n_layer)*g/cm3); // mass gold / total m
      return frac;
    }
    
    auto mass_fraction_pi() -> f32 {
      f32 frac = 
        mass_pi/
          (mass_pi+top_surface_area()*
          (96.97*thickness_au+58.6934*thickness_ni+2.336*thickness_n_layer)*g/cm3); 
      return frac;
    }
 
    auto density_botlayer() -> f32 {
      f32 dens = 
        (2.336*thickness_p_layer+58.6934*thickness_ni
        +96.97*thickness_au)/thickness_p_layer*g/cm3;
      return dens;
    }

    auto mass_fraction_si_bot() -> f32 {
      f32 frac = 
          2.336*thickness_p_layer/
          (2.336*thickness_p_layer+
          58.6934*thickness_ni+96.97*thickness_au);
      return frac;  
    }
    
    auto mass_fraction_ni_bot() -> f32 {
      f32 frac = 
        58.6934*thickness_ni/
        (2.336*thickness_p_layer+
        58.6934*thickness_ni+96.97*thickness_au);
      return frac; 
    }

    auto mass_fraction_au_bot() -> f32 {
      f32 frac = 
        96.97*thickness_au/
        (2.336*thickness_p_layer+58.6934*thickness_ni+96.97*thickness_au); // density of gold * thickness of gold layer / total density*depth
      return frac;
    }
  };
}
#endif
