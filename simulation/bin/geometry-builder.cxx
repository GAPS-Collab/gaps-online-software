#include <iostream>
#include <cstdint>

#include "cxxopts.hpp"

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "G4VisExecutive.hh"
#include "G4UImanager.hh"
//#include "G4UserRunAction.hh"
#include "G4RunManager.hh"
#include "G4PhysListFactory.hh"
#include "G4VUserPrimaryGeneratorAction.hh"
#include "G4PhysicalVolumeStore.hh"

#include "detector.hpp"
#include "gun.hpp"
#include "event_action.hpp"
#include "sim_config.hpp"

// g is taken by CLHEP
namespace go = gondola;

int main(int argc, char *argv[]){
  spdlog::cfg::load_env_levels();
    
  cxxopts::Options options("geometry-builder", "Compile the GAPS geometry with Geant4 and produce a gdml file");
  options.add_options()
  ("h,help", "Print help")
  //("c,calibration", "Folder with binary calibration files for each RB", cxxopts::value<std::string>()->default_value(""))
  //("file", "A Caraspace file", cxxopts::value<std::string>())
  //("directory", "A directory containing .gaps (caraspace) files, e.g. L0 Gaps files", cxxopts::value<std::string>())
  ("o,output", "Output filename", cxxopts::value<std::string>()->default_value("foo.gaps"))
  ("c,config", "Load a .toml config file to configure source and active detectors", cxxopts::value<std::string>()->default_value(""))
  ("visual", "Write a VRML file of the geometry and quit", cxxopts::value<bool>()->default_value("false"))
  ("check-overlap", "Perform Geant4 Overlap check", cxxopts::value<bool>()->default_value("false"))
  ("overlap-check", "Perform Geant4 Overlap check", cxxopts::value<bool>()->default_value("false"))
  ("save-gdml", "Save geometry in output gdml file 'gaps-assembly.gdml'", cxxopts::value<bool>()->default_value("false"))
  ("n,n-events", "Simulate number of events", cxxopts::value<u64>()->default_value("0"))
  ("v,verbose", "Verbose output", cxxopts::value<bool>()->default_value("false"))
  ;
  //options.parse_positional({"file"});
  auto result = options.parse(argc, argv);
  if (result.count("help")) {
    std::cout << options.help() << std::endl;
    exit(EXIT_SUCCESS);
  }

  // these arguments have the exact same function. They are doubled, 
  // because nobody can remember which one it actually was :) 
  auto check_overlap = result["check-overlap"].as<bool>();
  auto overlap_check = result["overlap-check"].as<bool>();
  if (check_overlap || overlap_check) {
    check_overlap = true;
  }
  bool verbose       = result["verbose"].as<bool>();
  bool visual        = result["visual"].as<bool>();
  bool save_gdml     = result["save-gdml"].as<bool>();
  u64  n_events      = result["n-events"].as<u64>();
  std::string output = result["output"].as<std::string>();
  std::string config = result["config"].as<std::string>();

  auto sim_config    = SimConfig::from_file(config);
  // command line overrides config file
  if (n_events == 0) {
    n_events = (u32)sim_config.n_events;
  } 
  //std::cout << sim_config.gun_fixed_pos_x << std::endl;
  //std::cout << sim_config.gun_fixed_pos_y << std::endl;
  //std::cout << sim_config.gun_fixed_pos_z << std::endl;
  //std::cout << sim_config.gun_energy << std::endl;
  auto detector = new go::GapsDetector(sim_config);
  detector->check_overlap = check_overlap;
  if (save_gdml) {
    std::string fname = "gaps-assembly.gdml";
    detector->Construct();
    detector->SaveGeometry(fname);
    exit(0);
  }
  //run_action->SetDetector(detector);
  G4PhysListFactory* physListFactory = new G4PhysListFactory();
  G4VUserPhysicsList * physics       = physListFactory->GetReferencePhysList("FTFP_BERT");
  G4RunManager* run_manager          = new G4RunManager();
  run_manager->SetUserInitialization(detector);
  run_manager->SetUserInitialization(physics);
  G4UserEventAction * event_action   = new go::EventAction(output);
  run_manager->SetUserAction(event_action);
  PrimaryGeneratorAction* gun        = new PrimaryGeneratorAction(sim_config);
  run_manager->SetUserAction(gun);
  run_manager->Initialize();
  if (check_overlap) {
    std::cout << "Checking overlaps from physical volume store" << std::endl;
    G4PhysicalVolumeStore* store = G4PhysicalVolumeStore::GetInstance();
    for (usize k=0; k<store->size(); k++) {
      auto volume = store->at(k);
      if (volume == nullptr) {
        std::cout << "The volume in the store is NULL!" << std::endl;
        std::exit(EXIT_FAILURE);
      }
      //std::cout << volume->GetName() << std::endl;
      //std::cout << "Getting volume at " << k << std::endl;
      //volume->GetLogicalVolume()->GetSolid();
      //std::cout << "Getting mother volume " << std::endl;
      //volume->GetMotherLogical();
      //std::cout << volume->GetMotherLogical()->GetName() << std::endl;
      //std::cout << "Getting mother solid " << std::endl;
      //volume->GetMotherLogical()->GetSolid();
      //std::cout << "good" << std::endl;

      //for (auto volume : *store) {
      //if (volume->CheckOverlaps()) {
      //  G4cerr << "Overlap detected in volume: " << volume->GetName() << G4endl;
      //  std::exit(EXIT_FAILURE);
      //}
    }
    std::cout << "Overlap check completed!" << std::endl;
    std::exit(EXIT_SUCCESS);
  }
  if (visual) {
    G4VisManager* visManager = new G4VisExecutive;
    visManager->Initialize();
    G4UImanager * UImanager = G4UImanager::GetUIpointer();
    //"/vis/scene/add/gun",
    std::vector<std::string> vis_commands = 
      {"/vis/open VRML2FILE",
       "/vis/viewer/set/autoRefresh false",
       "/vis/verbose errors",
       "/vis/drawVolume ",
       "/vis/scene/add/trajectories smooth",
       "/vis/scene/endOfEventAction accumulate",
       "/vis/viewer/zoom 1",
       "/vis/scene/add/axes 0 0 0 1 m",
       //"/run/beamOn 1",
       "/vis/viewer/rebuild",
       "/vis/viewer/flush"
    };
    //G4UserRunAction* run_action = new G4UserRunAction();
    for (auto cmd : vis_commands) { 
      UImanager->ApplyCommand(cmd.c_str());
    }
    std::cout << "WRL file written!" << std::endl;
    exit(EXIT_SUCCESS);
  }
  if (!overlap_check) {
    G4UImanager * UImanager = G4UImanager::GetUIpointer();
    auto cmds = Vec<std::string> {};
    cmds.push_back("/process/had/verbose 0");
    if (verbose) {
      cmds.push_back("/run/printProgress 100");
    }
    auto beam_on = std::format("/run/beamOn {}", n_events);
    std::cout << "Turning on beam!" << std::endl;
    cmds.push_back(beam_on.c_str());
    //G4UserRunAction* run_action = new G4UserRunAction();
    for (auto cmd : cmds) { 
      UImanager->ApplyCommand(cmd.c_str());
    }
    exit(EXIT_SUCCESS);
  }
}
