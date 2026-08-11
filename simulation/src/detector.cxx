#include "G4Box.hh"
#include "G4PVPlacement.hh"
#include "G4Transform3D.hh"
#include "G4GDMLParser.hh"
#include "G4VisAttributes.hh"

#include "detector.hpp"
#include "volume_store.hpp"
#include "sili_params.hpp"

using CLHEP::deg;
using CLHEP::cm;

#include "database.h"
#include "sensitive_detectors.hpp"

// g is taken by CLHEP for the unit of grams 
namespace go = gondola;

go::GapsDetector::GapsDetector(const SimConfig& cfg) {
  sim_config = cfg;
}

auto go::GapsDetector::SaveGeometry(std::string fname) -> void {
  G4GDMLParser parser;
  parser.SetAddPointerToName(false); 
  // mark the active volumes 
  for (auto const &v : active_vols) {
    // FIXME - initialization of this. There is a compiler warning if unit ('mm') and 
    // the nullptr (for the aux list?) is missing
    parser.AddVolumeAuxiliary(G4GDMLAuxStructType("SensDet", "SimpleDet/ActiveDetectorSD","mm", nullptr), v);
    //parser.AddAuxiliary(v, "SensDet", "SimpleDet/ActiveDetectorSD");
  }
  parser.Write(fname.c_str(), world);
}

auto go::GapsDetector::Construct() -> G4VPhysicalVolume* {
  // some switches for development
  bool tof_paddles    = true;
  // Geant4 manages all the boxes and logical volumes, don't delete
  u64 psv_vid         = 300000000; // smallest passive Volume id
  f32 world_extent    = 4400;
  // make all logical volumes globally available through 
  // Geant4 managers, together with the needed materials
  go::InitLVolumes();
  auto air      = G4Material::GetMaterial("Air"); 
  auto worldbox = new G4Box("WorldVolId0", world_extent, world_extent, world_extent);
  auto worldvol = new G4LogicalVolume(worldbox,air, "WorldVolId0"); 
  auto worldvis = new G4VisAttributes();
  worldvis->SetVisibility(false);
  worldvol->SetVisAttributes(worldvis);
  world         = new G4PVPlacement(nullptr, G4ThreeVector(), worldvol, "WordlVolId0",
                                    nullptr, false, 0, check_overlap); // last argument is overlap check
  
  auto strips = gondola::get_trackerstrips();
  auto sp     = go::SiLiParams();
  u64 ndetectors = 0;
  
  auto foam_lvol   = GetLogicalVolumeByName("PassiveTrackerFoam");
  auto foam_width  = 242; // 245 
  auto foam_height = 95; 
  auto foam_pos    = G4ThreeVector(-840,0,-0);

  // detector foam
  f32 r = 1.0;
  f32 g = 105.0/255.0;
  f32 b = 180.0/255.0;
  G4VisAttributes* foam_vis   = new G4VisAttributes(G4Colour(r,g,b,0.1));
  foam_vis->SetVisibility(true);
  foam_vis->SetForceSolid(true);          // Set to false if you want wireframe
  foam_vis->SetForceAuxEdgeVisible(true); // Makes edges visible in wireframe

  // tracker foam
  auto general_foam_trafo = G4ThreeVector(0,46.5, -3.1);
  auto tfoam_odd  = go::GetAssemblyFromGdml("/srv/gaps/gaps-detector-parts/gdml/tracker/foam/layer_odd.assembly.gdml", false);
  if (sim_config.pm_trk_foam_bot) {
    auto tfoam_bot  = go::GetAssemblyFromGdml("/srv/gaps/gaps-detector-parts/trk-foam-bot.gdml", false);
    for (auto const &k : tfoam_bot) {
      auto trala = k->GetTranslation();
      // slight adjustment, reason unknonwn
      //trala += G4ThreeVector(0,0,-2.1); // -12
      //trala += G4ThreeVector(0,46.5,0); // bottom foam in the 
      //trala += G4ThreeVector(0,-2,0);                                  // cad file is slightly 
                                        // off, not sure why
      trala += general_foam_trafo;
      auto traro = k->GetRotation();
      if (traro != nullptr) {
        traro->operator*=( CLHEP::HepRotationZ(90*deg));
      } else {
        traro = new CLHEP::HepRotation(CLHEP::HepRotationX(270*deg));
      }
      auto k_vol = k->GetLogicalVolume();
      k_vol->SetVisAttributes(foam_vis);
      new G4PVPlacement(traro, trala,
                        k->GetLogicalVolume(),
                        "PassiveTrackerFoamBot",
                        worldvol,
                        false,
                        psv_vid,
                        check_overlap);
      ++psv_vid;
    }
  }
  if (sim_config.pm_trk_foam_layer) {
    //auto foam_layer_pos   = G4ThreeVector(0,0,-799.5);
    auto foam_layer_pos   = G4ThreeVector(0,0,-799.6);
    for (auto const _ : {9}) {
    //for (auto const layer : {9,8,7,6,5,4,3,2,1}) {
      for (auto const &k : tfoam_odd) {
        //G4Transform3D foam_layer_trafo = G4Translate3D(foam_layer_pos);
        auto trala = k->GetTranslation();
        trala += general_foam_trafo;
        //trala += G4ThreeVector(2.0,-46.2, 0);
        trala += G4ThreeVector(-0.89,-47.3, 0);
        trala += foam_layer_pos;
        auto traro = k->GetRotation();
        if (traro != nullptr) {
          traro->operator*=( CLHEP::HepRotationZ(90*deg));
        } else {
          traro = new CLHEP::HepRotation(CLHEP::HepRotationX(270*deg));
        }
        //if (layer % 2 == 0) {
        //  //continue;
        //  foam_layer_trafo = foam_layer_trafo*G4RotateZ3D(90*deg);
        //}
        //foam_layer_trafo = foam_layer_trafo*G4RotateX3D(90*deg);
        
        //auto trala = k->GetTranslation();
        //// slight adjustment, reason unknonwn
        //trala += G4ThreeVector(0,0,-10);
        //auto traro = k->GetRotation();
        //if (traro != nullptr) {
        //  traro->operator*=( CLHEP::HepRotationZ(90*deg));
        //} else {
        //  traro = new CLHEP::HepRotation(CLHEP::HepRotationX(270*deg));
        //}
        auto k_vol = k->GetLogicalVolume();
        k_vol->SetVisAttributes(foam_vis);
        new G4PVPlacement(traro, trala,
                          k->GetLogicalVolume(),
                          "PassiveTrackerFoamLayersOdd",
                          worldvol,
                          false,
                          psv_vid,
                          check_overlap);
        ++psv_vid;
      }
      foam_layer_pos += G4ThreeVector(0,0,101.6);
    }
  } 
  //for (auto const &k : tfoam_even) {
  //  auto trala = k->GetTranslation();
  //  // slight adjustment, reason unknonwn
  //  trala += G4ThreeVector(0,0,-200);
  //  auto traro = k->GetRotation();
  //  if (traro != nullptr) {
  //    traro->operator*=( CLHEP::HepRotationZ(90*deg));
  //  } else {
  //    traro = new CLHEP::HepRotation(CLHEP::HepRotationX(270*deg));
  //  }
  //  auto k_vol = k->GetLogicalVolume();
  //  k_vol->SetVisAttributes(foam_vis);
  //  new G4PVPlacement(traro, trala,
  //                    k->GetLogicalVolume(),
  //                    "PassiveTrackerFoamLayersEven",
  //                    worldvol,
  //                    false,
  //                    psv_vid,
  //                    check_overlap);
  //  ++psv_vid;
  //}

  //auto foam_lvol_bot = GetLogicalVolumeByName("PassiveTrackerFoamBot");
  //G4Transform3D foam_transform_bot = G4Translate3D(foam_pos);
  //new G4PVPlacement(foam_transform_bot,
  //                  foam_lvol_bot,
  //                  "PassiveTrackerFoamBot",
  //                  worldvol,
  //                  false,
  //                  psv_vid,
  //                  check_overlap);
  ++psv_vid;
   
  // module top & bottom
  auto module_frame = go::GetLogicalVolumeByName("PassiveTrackerModuleFrame");
  auto module_top   = go::GetLogicalVolumeByName("PassiveTrackerTopWindow");
  auto module_bot   = go::GetLogicalVolumeByName("PassiveTrackerBotWindow");
  //for (u8 const layer : {9,8}) {
  for (u8 const layer : {0,1,2,3,4,5,6,7,8,9}) {
    for (u8 const row : {0,1,2,3,4,5}) {
    //for (u8 const row : {0}) {
      // we place a bit of foam
      //foam_pos += G4ThreeVector(foam_width,0,0);
      //G4Transform3D foam_transform = G4Translate3D(foam_pos);
      ////std::cout << foam_pos.x() << " " << foam_pos.y() << " " << foam_pos.z() << std::endl;
      //foam_transform = foam_transform*G4RotateX3D(90*deg);
      //if (trk_foam) {
      //  new G4PVPlacement(foam_transform,
      //                    foam_lvol,
      //                    "PassiveTrackerFoam",
      //                    worldvol,
      //                    false,
      //                    psv_vid,
      //                    check_overlap);
      //  ++psv_vid;
      //}
      //for (u8 const mod : {0}) { 
      for (u8 const mod : {0,1,2,3,4,5}) {
        if (sim_config.pm_trk_mod_psv) {
          auto mod_name = std::format("L{}R{}M{}", layer, row, mod);
          // for each module, we place the top and bottom
          auto mod_pos = gondola::get_module_position(layer, row, mod, strips);
          //std::cout << std::format("X {} Y {} Z {}", mod_pos[0], mod_pos[1], mod_pos[2]) << std::endl;
          auto mod_psv_zshift = 3.375;
          G4Transform3D mod_frame_trafo = G4Translate3D(G4ThreeVector(10*mod_pos[0],
                                                                      10*mod_pos[1],
                                                                      10*mod_pos[2]
                                                                      + mod_psv_zshift));
          G4Transform3D mod_topwd_trafo = G4Translate3D(G4ThreeVector(10*mod_pos[0],
                                                                      10*mod_pos[1],
                                                                      10*mod_pos[2]
                                                                      + mod_psv_zshift));
          G4Transform3D mod_botwd_trafo = G4Translate3D(G4ThreeVector(10*mod_pos[0],
                                                                      10*mod_pos[1],
                                                                      10*mod_pos[2]
                                                                      + mod_psv_zshift));
          if (layer % 2 == 0) {
            mod_botwd_trafo = mod_botwd_trafo*G4RotateX3D(90*deg);
            mod_frame_trafo = mod_frame_trafo*G4RotateX3D(90*deg);
          } else {
            mod_botwd_trafo = mod_botwd_trafo*G4RotateZ3D(-90*deg)*G4RotateX3D(90*deg);
            mod_frame_trafo = mod_frame_trafo*G4RotateZ3D(-90*deg)*G4RotateX3D(90*deg);
          }
                                                                      //10*mod_pos[2] + 4 + 2.1));
          mod_topwd_trafo = mod_topwd_trafo*G4RotateX3D(90*deg);
         
          if (sim_config.pm_trk_mod_frame) {  
            new G4PVPlacement(mod_frame_trafo,
                              module_frame,
                              std::format("PassiveModuleFrame{}", mod_name),
                              worldvol,
                              false,
                              psv_vid,
                              check_overlap);
            ++psv_vid;
          }
          new G4PVPlacement(mod_topwd_trafo,
                            module_top,
                            "PassiveModuleTopWindow",
                            worldvol,
                            false,
                            psv_vid,
                            check_overlap);
          ++psv_vid;
          new G4PVPlacement(mod_botwd_trafo,
                            module_bot,
                            std::format("PassiveTrackerBotWindow{}", mod_name),
                            worldvol,
                            false,
                            psv_vid,
                            check_overlap);
          ++psv_vid;
        }
        for (u8 const ch : {0,8,16,24}) {
    //std::cout << std::format("layer {} , row {} , mod {} , ch {}", layer, row, mod, ch) << std::endl;
          auto stripid = gondola::TrackerStrip::create_id(layer, row, mod, ch);
          ////std::cout << stripid << std::endl;
          auto strip   = strips[stripid];
          G4ThreeVector  detpos = {strip.global_pos_x_det_l0*10, 
                                   strip.global_pos_y_det_l0*10,
                                   strip.global_pos_z_det_l0*10
                                    - 0.5*sp.thickness_n_layer
                                    + 0.5*sp.thickness_p_layer};
          //std::cout << strip << std::endl;
          G4Transform3D transform = G4Translate3D(detpos);
         
          ++ndetectors;
          auto guard_ring = GetLogicalVolumeByName("PassiveSiLiGuardRing");
          if (sim_config.ad_all_sili) {
            new G4PVPlacement(transform,
                              guard_ring,
                              "PassiveSiLiGuardRing",
                              worldvol,
                              false,
                              psv_vid,
                              check_overlap);
            ++psv_vid;
          }
          //-----------------------------------------------------           
          auto nlayer = GetLogicalVolumeByName("PassiveDiskNRing");
          auto nlayer_pos = G4ThreeVector(detpos.x(),
                                          detpos.y(),
                                          detpos.z()
                                          + 0.5*sp.thickness_det
                                          + 0.5*sp.thickness_n_layer);
                                          // in the original code, there 
                                          // is a -0.5*sp.thickness_n_layer
                                          // however, that makes us fail the 
                                          // overlap check
          G4Transform3D transform_nlayer = G4Translate3D(nlayer_pos);
          if (sim_config.ad_all_sili) {
            new G4PVPlacement(transform_nlayer,
                              nlayer,
                              "PassiveDiskNRing",
                              worldvol,
                              false,
                              psv_vid,
                              check_overlap);
            ++psv_vid;
          }
          //-----------------------------------------------------           
          auto player = GetLogicalVolumeByName("PassiveDiskPRing");
          auto player_pos = G4ThreeVector(detpos.x(),
                                          detpos.y(),
                                          detpos.z()
                                          - 0.5*sp.thickness_det-0.5*sp.thickness_p_layer);
          transform = G4Translate3D(player_pos);
          if (sim_config.ad_all_sili) {
            new G4PVPlacement(transform,
                              player,
                              "PassiveDiskPRing",
                              worldvol,
                              false,
                              psv_vid,
                              check_overlap);
            ++psv_vid;
          }
        }
      }
    }
    foam_pos += G4ThreeVector(-foam_width*6,0,foam_height);
    //break;
  }
  std::cout << "We placed " << ndetectors << " Si(Li) wafers!" << std::endl;
  for (const auto &[stripid,strip] : strips) {
    G4ThreeVector  strippos = {strip.global_pos_x_l0*10, 
                               strip.global_pos_y_l0*10,
                               strip.global_pos_z_l0*10
                               - 0.5*sp.thickness_n_layer
                               + 0.5*sp.thickness_p_layer};
    G4ThreeVector  detpos = {strip.global_pos_x_det_l0*10, 
                             strip.global_pos_y_det_l0*10,
                             strip.global_pos_z_det_l0*10
                            - 0.5*sp.thickness_n_layer
                            + 0.5*sp.thickness_p_layer};
    //G4Transform3D transform = G4Translate3D(strippos);
    G4Transform3D transform = G4Translate3D(detpos);
    if (strip.layer % 2 == 0) {
      //continue;
    } else {
      // this transformation seems correct
      transform = transform*G4RotateZ3D(90*deg);
      //continue;
    }
    auto s_label = sp.get_strip_label(strip.channel);
    //std::cout << s_label << std::endl;
    //if (s_label != "C") {
    //  continue;
    //}
    std::string strip_name = std::format("LVOLForIntersec{}", s_label);
    auto strip_vol = GetLogicalVolumeByName(strip_name);
    auto strip_plc_name = std::format("ActiveSiLiDetCylStripVolId{}",strip.volume_id);
    if (sim_config.ad_all_sili) {
      new G4PVPlacement(transform,
                        strip_vol,
                        strip_plc_name,
                        worldvol,
                        false,
                        (u64)strip.volume_id,
                        check_overlap);
    }
  }
  // for each strip we are placing, we add also the 
  // reference to our own active volumes, so that we 
  // can save that information to the .gdml file 
  // this needs to be done per logical volume
  for (auto k : {"A","B","C","D","E","F","G","H"}) {
    std::string strip_name = std::format("LVOLForIntersec{}", k);
    auto strip_vol = GetLogicalVolumeByName(strip_name);
    active_vols.push_back(strip_vol);
  }
  
  // place active TOF paddles 
  auto paddles = gondola::get_tofpaddles(); 
  usize npaddles = 0;
  std::cout << "[detector.cxx] Found " << paddles.size() << " paddles in db!" << std::endl;
  if (tof_paddles) {
    for (auto const &pdl : paddles) {
      auto it = std::find(sim_config.active_paddles.begin(), sim_config.active_paddles.end(), (u8)pdl.first);
      //for (auto const &h : sim_config.active_paddles) {
      //  std::cout << (int)h << std::endl;
      //}
      if (it == sim_config.active_paddles.end()) {
        // has not been found 
        continue;
      }
      ++npaddles;
      auto pdl_name = std::format("ActiveTofPaddleScinti{}mm", pdl.second.length*10);
      if (pdl.first == 32 || pdl.first == 48) {
        pdl_name = "ActiveTofPaddleScintiPID32OR48";
      }
      if (pdl.first == 80 || pdl.first == 98) {
        pdl_name = "ActiveTofPaddleScintiPID80OR98";
      }

      auto pdl_vol  = GetLogicalVolumeByName(pdl_name);
      if (!pdl_vol) {
        std::cout << "Can not find " << pdl_name << std::endl;
        exit(1);
      }
      G4Transform3D transform;
      G4ThreeVector pos = G4ThreeVector(pdl.second.global_pos_x_l0*10,
                                        pdl.second.global_pos_y_l0*10,
                                        pdl.second.global_pos_z_l0*10);
      if (pdl.second.panel_id == 1) {
        transform = G4Translate3D(pos)
         * G4RotateZ3D(90*deg);
      } 
      if (pdl.second.panel_id == 2) {
        transform = G4Translate3D(pos)
         * G4RotateZ3D(90*deg);
      }
      if (pdl.second.panel_id == 3) {
        transform = G4Translate3D(pos)
        * G4RotateZ3D(90*deg) 
        * G4RotateX3D(90*deg);
        //continue;
      }
      if (pdl.second.panel_id == 4) {
        transform = G4Translate3D(pos)
         * G4RotateX3D(90*deg);
        //continue;
      }
      if (pdl.second.panel_id == 5) {
        transform = G4Translate3D(pos)
         * G4RotateZ3D(90*deg) 
         * G4RotateX3D(90*deg);
        //continue;
      }
      if (pdl.second.panel_id == 6) {
        transform = G4Translate3D(pos)
         * G4RotateX3D(90*deg);
        //continue;
      }
      if (pdl.second.panel_id == 7 
          || pdl.second.panel_id == 10 
          || pdl.second.panel_id == 11 
          || pdl.second.panel_id == 8 
          || pdl.second.panel_id == 13) {
        transform = //G4RotateZ3D(90*deg)
                                      G4Translate3D(pos)
                                     * G4RotateZ3D(90*deg);
      }
      if (pdl.second.panel_id == 9) {
        transform = G4Translate3D(pos);
                     //* G4RotateZ3D(90*deg);
      }
      if (pdl.second.panel_id == 12) {
        transform = G4Translate3D(pos);
                     //* G4RotateZ3D(90*deg);
      }
      // cortina 10pp
      if (pdl.second.panel_id == 14) {
        transform = G4Translate3D(pos)
         * G4RotateZ3D(90*deg) 
         * G4RotateX3D(90*deg);
      }

      if (pdl.second.panel_id == 15) {
        transform = G4Translate3D(pos)
         * G4RotateX3D(90*deg);
        //continue;
      }
      if (pdl.second.panel_id == 16) {
        transform = G4Translate3D(pos)
         * G4RotateZ3D(90*deg) 
         * G4RotateX3D(90*deg);
        //continue;
      }
      if (pdl.second.panel_id == 17) {
        transform = G4Translate3D(pos)
         * G4RotateX3D(90*deg);
        //continue;
      }
      // 3pp
      if (pdl.second.panel_id == 18) {
        transform = G4Translate3D(pos)
         * G4RotateZ3D(-45*deg)
         * G4RotateY3D(90*deg)
         * G4RotateX3D(90*deg);
      }
      if (pdl.second.panel_id == 19) {
        transform = G4Translate3D(pos)
         * G4RotateZ3D(45*deg)
         * G4RotateY3D(90*deg)
         * G4RotateX3D(90*deg);
      }
      if (pdl.second.panel_id == 20) {
        transform = G4Translate3D(pos)
         * G4RotateZ3D(-45*deg)
         * G4RotateY3D(90*deg)
         * G4RotateX3D(90*deg);
      }
      if (pdl.second.panel_id == 21) {
        transform = G4Translate3D(pos)
         * G4RotateZ3D(45*deg)
         * G4RotateY3D(90*deg)
         * G4RotateX3D(90*deg);
      }
      // cbe edge paddles
      if (pdl.second.paddle_id == 57) {
        transform = G4Translate3D(pos)
         * G4RotateZ3D(-45*deg)
         * G4RotateY3D(90*deg)
         * G4RotateX3D(90*deg);
      }
      if (pdl.second.paddle_id == 58) {
        transform = G4Translate3D(pos)
         * G4RotateZ3D(45*deg)
         * G4RotateY3D(90*deg)
         * G4RotateX3D(90*deg);
      }
      if (pdl.second.paddle_id == 59) {
        transform = G4Translate3D(pos)
         * G4RotateZ3D(-45*deg)
         * G4RotateY3D(90*deg)
         * G4RotateX3D(90*deg);
      }
      if (pdl.second.paddle_id == 60) {
        transform = G4Translate3D(pos)
         * G4RotateZ3D(45*deg)
         * G4RotateY3D(90*deg)
         * G4RotateX3D(90*deg);
      }
      auto pdl_pl_name = std::format("ActiveTofPaddleBoxScintiVolId{}", pdl.second.volume_id);
      new G4PVPlacement(transform,
                        pdl_vol,
                        pdl_pl_name,
                        worldvol,
                        false,
                        (u64)pdl.second.volume_id,
                        check_overlap);
    } 
    std::cout << "[detector.cxx] Placed " << npaddles << " TOF paddles!" << std::endl;
    for (auto k : {"ActiveTofPaddleScinti1800mm",
                   "ActiveTofPaddleScinti1720mm",
                   "ActiveTofPaddleScinti1560mm",
                   "ActiveTofPaddleScinti1510mm",
                   "ActiveTofPaddleScinti1082mm",
                   "ActiveTofPaddleScintiPID32OR48",
                   "ActiveTofPaddleScintiPID80OR98"}) {
      auto strip_vol = GetLogicalVolumeByName(k);
      active_vols.push_back(strip_vol);
    }
  }
  world->SetCopyNo(0); //explitly set vid
  return world;
}

auto go::GapsDetector::ConstructSDandField() -> void {
  // make the active volumes active 
  auto sd = new go::SensitiveDetector("foo");
  auto* sd_man = G4SDManager::GetSDMpointer();
  sd_man->AddNewDetector(sd);
  
  for (auto v : active_vols) {
    auto this_sd = dynamic_cast<G4VSensitiveDetector*>(sd);
    v-> SetSensitiveDetector(this_sd);  	
  } 
}

