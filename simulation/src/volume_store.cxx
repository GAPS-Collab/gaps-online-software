#include "G4Box.hh"
#include "G4Tubs.hh"
#include "G4SubtractionSolid.hh"
#include "G4IntersectionSolid.hh"

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "G4VisAttributes.hh"
#include "volume_store.hpp"
#include "materials.hpp"
#include "sili_params.hpp"

using CLHEP::mm;
using CLHEP::cm;
using CLHEP::deg;

namespace go = gondola;

/************************************************************************/

// FIXME - there must be something in geant4 as well
auto go::GetLogicalVolumeByName(const G4String& name) -> go::G4LogicalVolumePtr {
  G4LogicalVolumeStore* lvStore = G4LogicalVolumeStore::GetInstance();
  for (auto* lv : *lvStore) {
    if (lv->GetName() == name) {
      return lv;
    }
  }
  return nullptr; // not found
}

/************************************************************************/

auto go::GetAssemblyFromGdml(const G4String& f_name, bool validate) -> Vec<go::G4VPhysicalVolumePtr> {
  auto placements = Vec<G4VPhysicalVolumePtr>();
  G4GDMLParser parser = G4GDMLParser();
  parser.Clear();
  parser.Read(f_name, validate);
  G4VPhysicalVolumePtr p_world = parser.GetWorldVolume();
  if (p_world == nullptr) {
    spdlog::error("Unable to load logical volume from file {}!", f_name);
    exit(1);
  }
  G4LogicalVolumePtr l_world = p_world->GetLogicalVolume();
  if (l_world == nullptr) {
    spdlog::error("Unable to load logical volume from file {}!", f_name);
    exit(1);
  }
  go::G4VPhysicalVolumePtr pl;
  // log_info("We see the first daughter " << lWorld->GetDaughter(0)->GetName());
  // log_info("We see " << lWorld->GetNoDaughters() << " daughters");
  for (size_t k=0; k<l_world->GetNoDaughters(); k++) {
    pl = l_world->GetDaughter(k);
    placements.push_back(pl);
  }
  // trust me, you want these deletes. I do not know
  // why, but otherwise, it will return the wrong parts
  // I think it might have something to do with how geant4 stores
  // pointers to volumes internally.
  delete p_world;
  delete l_world;
  return placements;
  //if (l_world == nullptr) {
  //  spdlog::error("Unable to load logical volume from file {}!", f_name);
  //  exit(1);
  //}
  //G4VSolidPtr            solid = l_world->GetSolid();
  //if (solid == nullptr) {
  //  spdlog::error("Unable to load solid from file {}!", f_name);
  //  exit(1);
  //} 
  //
  //// this seems to be needed for parts created with mradsim? 
  //if (solid->GetName() == "worldbox") {
  //  spdlog::info("It seems the part is nested in the worldbox. Trying to go down one level");
  //  spdlog::info("We have indeed {} daughters", l_world->GetNoDaughters());
  //  if (l_world->GetDaughter(0)->GetLogicalVolume() == nullptr) {
  //    spdlog::error("... but they don't have volumes");
  //    exit(1);
  //  }
  //  if (l_world->GetDaughter(0)->GetLogicalVolume()->GetSolid() == nullptr) {
  //    spdlog::error("... but they don't have solids.");
  //    exit(1);
  //  }
  //return result;
}

/************************************************************************/

auto go::GetTessSolidFromGdml(const G4String& f_name, bool validate) -> go::G4TessellatedSolidPtr {
   // FIXME - transition to not load solids, but logical volumes instead
   //         from the gdml files.
   //         This will have the big advantages that we can get the 
   //         materials from the gdml file too.
   //         This might require to fix the materials in the gdml file, 
   //         so they pass the validation step.

   G4GDMLParser parser = G4GDMLParser();
   parser.Clear();
   parser.Read(f_name, validate);
   G4VPhysicalVolumePtr p_world = parser.GetWorldVolume();
   G4LogicalVolumePtr   l_world = p_world->GetLogicalVolume();
   if (l_world == nullptr) {
     spdlog::error("Unable to load logical volume from file {}!", f_name);
     exit(1);
   }
   G4VSolidPtr            solid = l_world->GetSolid();
   if (solid == nullptr) {
     spdlog::error("Unable to load solid from file {}!", f_name);
     exit(1);
   } 
   
   // this seems to be needed for parts created with mradsim? 
   if (solid->GetName() == "worldbox") {
     spdlog::info("It seems the part is nested in the worldbox. Trying to go down one level");
     spdlog::info("We have indeed {} daughters", l_world->GetNoDaughters());
     if (l_world->GetDaughter(0)->GetLogicalVolume() == nullptr) {
       spdlog::error("... but they don't have volumes");
       exit(1);
     }
     if (l_world->GetDaughter(0)->GetLogicalVolume()->GetSolid() == nullptr) {
       spdlog::error("... but they don't have solids.");
       exit(1);
     }
     solid = l_world->GetDaughter(0)->GetLogicalVolume()->GetSolid();
     spdlog::info("We got a G4Solid with name {}", solid->GetName());
   }
   // trust me, you want these deletes. I do not know
   // why, but otherwise, it will return the wrong parts
   // I think it might have something to do with how geant4 stores
   // pointers to volumes internally.
   if (p_world != nullptr) {
     delete p_world;
   }
   if (l_world != nullptr) {
     delete l_world;
   }
   G4TessellatedSolidPtr t_solid = dynamic_cast<G4TessellatedSolidPtr>(solid);
   return t_solid;
}

/************************************************************************/

//std::vector<G4VPhysicalVolume*> cs::AssemblyLoader(std::string gdmlFile, bool validate)
//{
//   G4GDMLParser* parser = new G4GDMLParser();
//   parser->Clear();
//   log_info("Will read " << gdmlFile);
//   parser->Read(gdmlFile, validate);
//   G4VPhysicalVolume* gdmlWorld = parser->GetWorldVolume();
//   G4LogicalVolume* lWorld      = gdmlWorld->GetLogicalVolume();
//   std::vector<G4VPhysicalVolume*> placements;
//   placements.clear();
//   G4VPhysicalVolume* pl;
//   log_info("We see the first daughter " << lWorld->GetDaughter(0)->GetName());
//   log_info("We see " << lWorld->GetNoDaughters() << " daughters");
//   for (size_t k=0; k<lWorld->GetNoDaughters(); k++)
//     {
//        pl = lWorld->GetDaughter(k);
//        placements.push_back(pl);
//     }
//   // trust me, you want these deletes. I do not know
//   // why, but otherwise, it will return the wrong parts
//   // I think it might have something to do with how geant4 stores
//   // pointers to volumes internally.
//   delete parser;
//   delete gdmlWorld;
//   delete lWorld;
//   return placements;
//} 


auto go::InitLVolumes(const SimConfig& cfg) -> void {

  // rgb colors
  f32 r     = 3.0/255.0;
  f32 g     = 57.0/255.0;
  f32 b     = 108.0/255.0;
  f32 alpha = 0.7;
  G4VisAttributes* paddle_vis = new G4VisAttributes(G4Colour(r, g, b, alpha));
  paddle_vis->SetVisibility(true);
  paddle_vis->SetForceSolid(true);          // Set to false if you want wireframe
  paddle_vis->SetForceAuxEdgeVisible(true); // Makes edges visible in wireframe
  r = 103.0/255.0;
  g = 2.0/255.0;
  b = 61.0/255.0; 
  G4VisAttributes* sili_vis   = new G4VisAttributes(G4Colour(r,g,b,alpha));
  sili_vis->SetVisibility(true);
  sili_vis->SetForceSolid(true);          // Set to false if you want wireframe
  sili_vis->SetForceAuxEdgeVisible(true); // Makes edges visible in wireframe
  r = 215.0/255.0;
  g = 215.0/255.0;
  b = 215.0/255.0; 
  G4VisAttributes* passive_vis   = new G4VisAttributes(G4Colour(r,g,b,0.1));
  passive_vis->SetVisibility(true);
  passive_vis->SetForceSolid(true);          // Set to false if you want wireframe
  passive_vis->SetForceAuxEdgeVisible(true); // Makes edges visible in wireframe
  // make foam hotpink
  r = 1.0;
  g = 105.0/255.0;
  b = 180.0/255.0;
  G4VisAttributes* foam_vis   = new G4VisAttributes(G4Colour(r,g,b,0.1));
  foam_vis->SetVisibility(true);
  foam_vis->SetForceSolid(true);          // Set to false if you want wireframe
  foam_vis->SetForceAuxEdgeVisible(true); // Makes edges visible in wireframe
  r = 102.0/255.0;
  g = 153.0/255.0;
  b = 204.0/255.0;  
  G4VisAttributes* frame_vis   = new G4VisAttributes(G4Colour(r,g,b,0.1));
  frame_vis->SetVisibility(true);
  frame_vis->SetForceSolid(true);          // Set to false if you want wireframe
  frame_vis->SetForceAuxEdgeVisible(true); // Makes edges visible in wireframe
  
  // let's start simple by creating some tof paddle scintis
  // the length of the box is the half length of the thing
  go::InitMaterials(); 
  f32 default_width  = 160*mm; 
  f32 default_height = 6.35*mm;
  // 12pp
  std::string scinti_name = "ActiveTofPaddleScinti1800mm";
  auto scinti_1800 = new G4Box (scinti_name + "Solid",
                                1800*0.5*mm,
                                default_width*0.5,
                                default_height*0.5);

  auto scinti_1800_lvol = new G4LogicalVolume (scinti_1800,
                                               G4Material::GetMaterial("PVT"),
                                               scinti_name );
  scinti_1800_lvol->SetVisAttributes(paddle_vis);
  // 0pp
  scinti_name = "ActiveTofPaddleScinti1720mm";
  auto scinti_1720 = new G4Box (scinti_name + "Solid",
                                1720*0.5*mm,
                                default_width*0.5,
                                default_height*0.5);
  auto scinti_1720_lvol = new G4LogicalVolume (scinti_1720,
                                               G4Material::GetMaterial("PVT"),
                                               scinti_name );
  scinti_1720_lvol->SetVisAttributes(paddle_vis);
  // 8pp
  scinti_name = "ActiveTofPaddleScinti1560mm";
  auto scinti_1560 = new G4Box (scinti_name + "Solid",
                                1560*0.5*mm,
                                default_width*0.5,
                                default_height*0.5);
  auto scinti_1560_lvol = new G4LogicalVolume (scinti_1560,
                                               G4Material::GetMaterial("PVT"),
                                               scinti_name );
  scinti_1560_lvol->SetVisAttributes(paddle_vis);
  // 3pp 
  scinti_name = "ActiveTofPaddleScinti1510mm";
  auto scinti_1510 = new G4Box (scinti_name + "Solid",
                                1510*0.5*mm,
                                default_width*0.5,
                                default_height*0.5); // are these really thinner?
  auto scinti_1510_lvol = new G4LogicalVolume (scinti_1510,
                                               G4Material::GetMaterial("PVT"),
                                               scinti_name );
  scinti_1510_lvol->SetVisAttributes(paddle_vis);
  // edgepaddles
  scinti_name = "ActiveTofPaddleScinti1082mm";
  auto scinti_1082 = new G4Box (scinti_name + "Solid",
                                1082*0.5*mm,
                                100*0.5*mm,
                                default_height*0.5);
  auto scinti_1082_lvol = new G4LogicalVolume (scinti_1082,
                                               G4Material::GetMaterial("PVT"),
                                               scinti_name );
  scinti_1082_lvol->SetVisAttributes(paddle_vis);
  // paddle 32 and 49 are special CBE side paddles
  scinti_name = "ActiveTofPaddleScintiPID32OR48";
  auto scinti_pid32_48 = new G4Box (scinti_name + "Solid",
                                    1510*0.5*mm,
                                    145*0.5*mm,
                                    default_height*0.5);
  auto scinti_pid32_48_lvol = new G4LogicalVolume (scinti_pid32_48,
                                                   G4Material::GetMaterial("PVT"),
                                                   scinti_name );
  scinti_pid32_48_lvol->SetVisAttributes(paddle_vis);
  // there is also 2 special UMB paddles
  scinti_name = "ActiveTofPaddleScintiPID80OR98";
  auto scinti_pid80_98 = new G4Box (scinti_name + "Solid",
                                    1490*0.5*mm,
                                    default_width*0.5,
                                    default_height*0.5);
  auto scinti_pid80_98_lvol = new G4LogicalVolume (scinti_pid80_98,
                                                   G4Material::GetMaterial("PVT"),
                                                   scinti_name );
  scinti_pid80_98_lvol->SetVisAttributes(paddle_vis);

  // gaps detector part dir 
  auto parts_root = cfg.parts_root_dir;

  // tracker volumes
  auto t_mod_frame      = go::GetTessSolidFromGdml(parts_root + "/gdml/tracker/module/shrink/frame.shrink0.999.fix.gdml", false);
  auto t_mod_frame_lvol = new G4LogicalVolume (t_mod_frame,
                                               go::GetMaterial("Al"),
                                               "PassiveTrackerModuleFrame");
  t_mod_frame_lvol->SetVisAttributes(frame_vis);
  // top_window
  auto t_mod_top = go::GetTessSolidFromGdml(parts_root + "/gdml/tracker/module/top_window.fix.gdml", false);
  //auto t_mod_top      = go::GetTessSolidFromGdml("/srv/gaps/gaps-detector-parts/gdml/tracker/module/top_window.fix.gdml", false);
  auto t_mod_top_lvol = new G4LogicalVolume (t_mod_top,
                                             go::GetMaterial("Al"),
                                             "PassiveTrackerTopWindow");
  t_mod_top_lvol->SetVisAttributes(frame_vis);
  auto t_mod_bot      = go::GetTessSolidFromGdml(parts_root + "/gdml/tracker/module/bot_window.gdml", false);
  auto t_mod_bot_lvol = new G4LogicalVolume (t_mod_bot,
                                             go::GetMaterial("Al"),
                                             "PassiveTrackerBotWindow");
  t_mod_bot_lvol->SetVisAttributes(frame_vis);
  
  // frame, inner and outer
  auto frame_outer = go::GetTessSolidFromGdml(parts_root + "/frame-outer.gdml", false);
  auto frame_outer_lvol = new G4LogicalVolume (frame_outer,
                                               G4Material::GetMaterial("Al"),
                                               "PassiveOuterFrame");
  frame_outer_lvol->SetVisAttributes(frame_vis);

  //auto frame_inner = go::GetTessSolidFromGdml(parts_root + "/frame-outer.gdml", false);
  //auto frame_inner_lvol = new G4LogicalVolume(frame_inner,
  //                                            G4Material::GetMaterial("Al"),
  //                                            "PassiveInnerFrame");
  //frame_inner_lvol->SetVisAttributes(frame_vis);
  
  // foam
  //auto tfoam = go::GetTessSolidFromGdml("/srv/gaps/gaps-detector-parts/tracker-foam-piece.fix.gdml", false);
  //auto tfoam_lvol = new G4LogicalVolume (tfoam,
  //                                       G4Material::GetMaterial("Ethafoam"),
  //                                       "PassiveTrackerFoam");
  //tfoam_lvol->SetVisAttributes(foam_vis);

  //auto tfoam_bot      = go::GetAssemblyFromGdml("/srv/gaps/gaps-detector-parts/trk-foam-bot.gdml", false);
  //u32 pc = 0;
  //for (auto const &k : tfoam_bot) {
  //  auto tfoam_bot_piece_name = std::format("PassiveTrackerFoamBotPc{}", pc); 
  //  auto tfoam_bot_lvol = new G4LogicalVolume (tfoam_bot,
  //                                             G4Material::GetMaterial("Ethafoam"),
  //                                             tfoam_bot_piece_name);
  //  tfoam_bot_lvol->SetVisAttributes(foam_vis);
  //  ++pc;
  //}
  //std::cout << "We added  " << pc << " pieces of bottom foam!" << std::endl;
  //exit(EXIT_FAILURE);
  auto sp = go::SiLiParams();
  std::string active_sili_name = "ActiveSiLiDetectorCyl";
  auto active_sili = new G4Tubs(active_sili_name + "Solid",
                                0,
                                sp.radius_active,
                                0.5*sp.thickness_det, 0, 360*deg);
  // name strips A-H from "left" to "right"
  auto strip_widths       = sp.strip_widths();
  Vec<f32> dfc            = sp.det_center_distance();
  Vec<std::string> labels = {"A", "B", "C", "D", "E", "F", "G", "H"};
  for (auto const &s_idx : {0,1,2,3,4,5,6,7} ) {
    auto sw    = strip_widths[s_idx];
    auto label = labels[s_idx];
    std::cout << std::format("{} : {}", label, sw) << std::endl;
    auto strip_placement   = G4ThreeVector(0,dfc[s_idx], 0);
    std::cout << strip_placement << std::endl;
    auto active_sili_strip = new G4Box("BoxForIntersection",
                                       sp.radius_active,
                                       sw*0.5*sp.groove_fraction,
                                       sp.thickness_det*0.5); 
    auto active_sili_strip_sec = new G4IntersectionSolid(active_sili_name + "StripSecSolid",
            active_sili, active_sili_strip, 0, strip_placement);
    auto active_sili_lvol = new G4LogicalVolume (active_sili_strip_sec,
                                                 //active_sili_strip,
                                                 go::GetMaterial("G4_Si"),
                                                 "LVOLForIntersec" + label);
                                                 //active_sili_name + "Strip" + label );
    active_sili_lvol->SetVisAttributes(sili_vis);
    //break;
  }
  //auto active_sili_lvol = new G4LogicalVolume (active_sili,
  //                                             go::GetMaterial("G4_Si"),
  //                                             active_sili_name );
  
  //--------------------------------------------------
  std::string guardring_name = "PassiveSiLiGuardRing";
  auto guardring_sili = new G4Tubs(guardring_name + "Solid", sp.radius_guardring, sp.radius_wafer, 
                                   0.5*(sp.thickness_det - sp.thickness_n_layer - sp.thickness_p_layer), 0, 360*deg);
  guardring_name = "PassiveRingGuardRing";
  auto guardring_tophat = new G4Tubs(guardring_name + "Solid", sp.radius_guardring, sp.radius_wafer,
                                     0.5*(sp.depth_guardring - sp.thickness_n_layer), 0, 360*deg);
  auto guardring = new G4SubtractionSolid(guardring_name, guardring_sili, guardring_tophat,
                                          0, G4ThreeVector(0, 0, 0.5*(sp.thickness_det - sp.thickness_n_layer-sp.depth_guardring)));
  auto guardring_lvol = new G4LogicalVolume (guardring,
                                             go::GetMaterial("G4_Si"),
                                             "PassiveSiLiGuardRing" );

  guardring_lvol->SetVisAttributes(passive_vis);
  //--------------------------------------------------
  auto nlayer = new G4Tubs("PassiveDiskNRingSolid", 0.0, sp.radius_guardring, 0.5*sp.thickness_n_layer, 0, 360*deg);
  auto nlayer_lvol = new G4LogicalVolume(nlayer,
                                         go::GetMaterial("SiLiTop"),
                                         "PassiveDiskNRing");
  nlayer_lvol->SetVisAttributes(passive_vis);
  //--------------------------------------------------
  auto player = new G4Tubs("PassiveDiskPRingSolid", 0.0, sp.radius_wafer, 0.5*sp.thickness_p_layer, 0, 360*deg);
  auto player_lvol = new G4LogicalVolume(player,
                                         go::GetMaterial("SiLiBottom"),
                                         "PassiveDiskPRing");
  player_lvol->SetVisAttributes(passive_vis);
  //--------------------------------------------------
  std::cout << "Volumes prepared!" << std::endl;
}


