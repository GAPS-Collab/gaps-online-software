#pragma once 

#include "gondola.hpp"

struct SimConfig {
  std::string parts_root_dir;
  bool        gun_source_above_pid;
  bool        gun_fixed_pos;
  f32         gun_fixed_pos_x;
  f32         gun_fixed_pos_y;
  f32         gun_fixed_pos_z;
  bool        gun_fixed_energy;
  f32         gun_energy;
  std::string gun_particle_type;
  u8          gun_center_around_pid;
  bool        gun_sample_isotropic_box;
  f32         gun_sample_isotropic_box_len;
  f32         gun_min_e_per_n;
  f32         gun_max_e_per_n;
  bool        gun_uniform_energy;
  bool        gun_uniform_beta;
  f64         gun_beta_lim_low;
  f64         gun_beta_lim_up;

  bool        pm_trk_mod_frame;
  bool        pm_outer_frame;
  bool        pm_inner_frame;
  bool        pm_trk_foam_bot;
  bool        pm_trk_foam_layer;
  bool        pm_trk_mod_psv;

  bool        ad_all_sili;
  u8          ad_only_place_pid;
  Vec<u8>     active_paddles;

  bool        digitization;
  bool        use_advanced_digitization  ;
  bool        digi_use_tof_legacy_fb     ;
  // volume id - should be paddle id
  Vec<u32>    digi_tof_excluded_readout  ;
  f64         digi_tof_voltage_width     ; 
  f64         digi_tof_voltage_offset    ; 
  f64         digi_tof_timing_offset     ; 
  f64         digi_tof_timing_width      ; 
  f64         digi_tof_pos_offset        ; 
  f64         digi_tof_pos_width         ;
  std::string digi_tof_pulse_templates   ;
  bool        digi_tof_add_gauss_jitter  ;
  // trigger thresholds, also used for readout 
  f64         digi_tof_hit_threshold_mv  ;
  f64         digi_tof_beta_threshold_mv ; 
  f64         digi_tof_veto_threshold_mv ;
  // tracker parameters for the digitizer
  std::string digi_trk_tf_name           ; 
  std::string digi_trk_mask_name         ; 
  std::string digi_trk_ped_name          ; 
  f64         digi_trk_energy_offset     ; 
  f64         digi_trk_energy_width      ; 
  bool        digi_trk_add_cmn_noise     ;
  
  u64         n_events;
  /// Load a configuration from a .toml file
  static auto from_file(std::string fname) -> SimConfig;
};

