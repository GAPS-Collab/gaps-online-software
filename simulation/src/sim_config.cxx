#include "sim_config.hpp"

#include <toml++/toml.hpp>
#include <iostream>

using namespace std::literals;

auto SimConfig::from_file(std::string fname) -> SimConfig {
  auto sim    = SimConfig();
  auto config = toml::parse_file(fname);
  std::cout << "======SIMULATION CONFIGURATION========" << std::endl;
  std::cout << config << std::endl;
  std::cout << "======================================" << std::endl;
  //exit(1);
  // get key-value pairs
  sim.gun_fixed_pos                = config["gun"]["fixed_pos"].value_or(false);
  sim.gun_fixed_pos_x              = config["gun"]["fixed_pos_x"].value_or(0.0);
  sim.gun_fixed_pos_y              = config["gun"]["fixed_pos_y"].value_or(0.0);
  sim.gun_fixed_pos_z              = config["gun"]["fixed_pos_z"].value_or(0.0);
  sim.gun_fixed_energy             = config["gun"]["fixed_energy"].value_or(false);
  sim.gun_energy                   = config["gun"]["energy"].value_or(0.0);
  sim.gun_particle_type            = config["gun"]["particle_type"].value_or("proton"); 
  sim.gun_center_around_pid        = (u8)config["gun"]["center_around_pid"].value_or(0);
  sim.gun_sample_isotropic_box     = config["gun"]["sample_isotropic_box"].value_or(false);
  sim.gun_sample_isotropic_box_len = config["gun"]["sample_isotropic_box_len"].value_or(2200);
  sim.gun_uniform_energy           = config["gun"]["uniform_energy"].value_or(false);
  //std::cout << "GUE :" << sim.gun_uniform_energy << std::endl;
  //exit(1);
  if (sim.gun_uniform_energy) {
    sim.gun_min_e_per_n              = (f32)config["gun"]["min_e_per_n"].value_or(0.0);
    sim.gun_max_e_per_n              = (f32)config["gun"]["max_e_per_n"].value_or(0.0);
    if (sim.gun_max_e_per_n == 0) {
      std::cout << "Upper energy bound as set in the config file is zero! However, a uniform energy between min/max was requested. Unable to comply!" << std::endl;
      exit(1);
    }
  } 
  //std::cout << sim.gun_min_e_per_n << std::endl;
  //std::cout << sim.gun_max_e_per_n << std::endl;
  //exit(1);
  sim.pm_trk_mod_frame  = config["passive_materials"]["trk_mod_frame"].value_or(true); 
  sim.pm_trk_mod_psv    = config["passive_materials"]["trk_mod_psv"].value_or(true);
  sim.pm_outer_frame    = config["passive_materials"]["outer_frame"].value_or(true);
  sim.pm_inner_frame    = config["passive_materials"]["inner_frame"].value_or(true);
  sim.pm_trk_foam_bot   = config["passive_materials"]["trk_foam_bot"].value_or(true); 
  sim.pm_trk_foam_layer = config["passive_materials"]["trk_foam_layer"].value_or(true);
  
  sim.ad_only_place_pid  = config["active_detectors"]["only_place_pid"].value_or(0);  
  sim.ad_all_sili        = config["active_detectors"]["all_sili"].value_or(true);  
  if (sim.ad_only_place_pid > 0) {
    sim.active_paddles = {sim.ad_only_place_pid};
  } else {
    auto active_pids       = Vec<u8>(); 
    for (auto&& el : *config["active_detectors"]["tof_panel_1"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_2"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_3"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_4"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_5"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_6"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["edge_paddles"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_7"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_8"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    sim.active_paddles = active_pids;
  } 
  //std::cout << "LOWER ENERGY BOUND " << sim.gun_min_e_per_n << std::endl;
  //std::cout << "UPPER ENERGY BOUND " << sim.gun_max_e_per_n << std::endl;
  //exit(1);
  sim.n_events = config["general"]["n_events"].value_or(0); 
  return sim;
}
