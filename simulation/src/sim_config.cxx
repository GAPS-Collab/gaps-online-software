#include "sim_config.hpp"

#include <toml++/toml.hpp>
#include <iostream>

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

using namespace std::literals;

auto SimConfig::from_file(std::string fname) -> SimConfig {
  auto sim    = SimConfig();
  auto config = toml::parse_file(fname);
  std::cout << "======SIMULATION CONFIGURATION========" << std::endl;
  std::cout << config << std::endl;
  std::cout << "======================================" << std::endl;
  //exit(1);
  // get key-value pairs
  sim.parts_root_dir               = config["general"]["parts_root_dir"].value_or("");
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
  sim.gun_uniform_beta  = config["gun"]["uniform_beta"].value_or(false); 
  sim.gun_beta_lim_low  = (f64)config["gun"]["beta_lim_low"].value_or(0.0);
  sim.gun_beta_lim_up   = (f64)config["gun"]["beta_lim_up"].value_or(0.0); 
  // perform checks if the user's input for the gun are sane 
  if (((u8)sim.gun_uniform_beta + (u8)sim.gun_uniform_energy + (u8)sim.gun_fixed_energy) > 1) { 
    std::string message("Input can only be uniform beta, uniform energy or fixed energy, but not a combination of these!");
    spdlog::error(message);
    throw message;
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
    for (auto&& el : *config["active_detectors"]["tof_panel_9"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_10"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_11"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_12"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_13"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_14"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_15"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_16"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_17"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_18"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_19"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_20"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    for (auto&& el : *config["active_detectors"]["tof_panel_21"].as_array()) {
      active_pids.push_back(el.value_or(0)); 
    } 
    sim.active_paddles = active_pids;
  } 

  sim.digitization               = config["digitization"]["use_digitization"].value_or(false); 
  sim.use_advanced_digitization  = config["digitization"]["use_advanced_features"].value_or(false);
  sim.digi_use_tof_legacy_fb     = config["digitization"]["digi_use_tof_legacy_fallback"].value_or(false);
  for (auto&& el : *config["digitization"]["digi_tof_paddles_exclude_from_readout"].as_array()) {
    sim.digi_tof_excluded_readout.push_back((u32)el.value_or(0)); 
  } 
  sim.digi_tof_voltage_width     = (f32)config["digitization"]["digi_tof_voltage_width"].value_or(0.0);
  sim.digi_tof_voltage_offset    = (f32)config["digitization"]["digi_tof_voltage_offset"].value_or(0);
  sim.digi_tof_timing_offset     = (f32)config["digitization"]["digi_tof_timing_offset"].value_or(0);
  sim.digi_tof_timing_width      = (f32)config["digitization"]["digi_tof_timing_width"].value_or(0);
  sim.digi_tof_pos_offset        = (f32)config["digitization"]["digi_tof_pos_offset"].value_or(0);
  sim.digi_tof_pos_width         = (f32)config["digitization"]["digi_tof_pos_width"].value_or(0);
  sim.digi_tof_pulse_templates   = config["digitization"]["digi_tof_pulse_template_file"].value_or("");
  sim.digi_tof_add_gauss_jitter  = config["digitization"]["digi_tof_add_gauss_jitter"].value_or(false);
  sim.digi_tof_hit_threshold_mv  = (f32)config["digitization"]["digi_tof_hit_threshold_mv"].value_or(0.0);
  sim.digi_tof_beta_threshold_mv = (f32)config["digitization"]["digi_tof_beta_threshold_mv"].value_or(0.0);
  sim.digi_tof_veto_threshold_mv = (f32)config["digitization"]["digi_tof_veta_threshold_mv"].value_or(0.0);
  // tracker digitization part 
  sim.digi_trk_tf_name           = config["digitization"]["digi_trk_tf_name_in_db"].value_or("");
  sim.digi_trk_mask_name         = config["digitization"]["digi_trk_mask_name_in_db"].value_or(""); 
  sim.digi_trk_ped_name          = config["digitization"]["digi_trk_ped_name_in_db"].value_or(""); 
  
  sim.digi_trk_energy_offset     = (f32)config["digitization"]["digi_trk_energy_offset"].value_or(0);
  sim.digi_trk_energy_width      = (f32)config["digitization"]["digi_trk_energy_width"].value_or(04/2.355); // from SimpleDet 
  sim.digi_trk_add_cmn_noise     = config["digitization"]["digi_trk_add_cmn_noise"].value_or(false); // from SimpleDet 
  //std::cout << "LOWER ENERGY BOUND " << sim.gun_min_e_per_n << std::endl;
  //std::cout << "UPPER ENERGY BOUND " << sim.gun_max_e_per_n << std::endl;
  //exit(1);
  sim.n_events = config["general"]["n_events"].value_or(0); 
  return sim;
}
