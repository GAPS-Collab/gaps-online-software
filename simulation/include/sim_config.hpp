#pragma once 

#include "gondola.hpp"

struct SimConfig {
  bool gun_source_above_pid;
  bool gun_fixed_pos;
  f32  gun_fixed_pos_x;
  f32  gun_fixed_pos_y;
  f32  gun_fixed_pos_z;
  bool gun_fixed_energy;
  f32  gun_energy;
  std::string gun_particle_type;
  u8   gun_center_around_pid;
  bool gun_sample_isotropic_box;
  f32  gun_sample_isotropic_box_len;
  f32  gun_min_e_per_n;
  f32  gun_max_e_per_n;
  bool gun_uniform_energy;

  bool pm_trk_mod_frame;
  bool pm_outer_frame;
  bool pm_inner_frame;
  bool pm_trk_foam_bot;
  bool pm_trk_foam_layer;
  bool pm_trk_mod_psv;

  bool ad_all_sili;
  u8   ad_only_place_pid;
  Vec<u8> active_paddles;
  
  u64  n_events;
  /// Load a configuration from a .toml file
  static auto from_file(std::string fname) -> SimConfig;
};

