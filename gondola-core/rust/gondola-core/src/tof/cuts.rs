// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

#[derive(Copy, Clone)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TofCuts {
  pub min_hit_cor         : u8  ,
  pub min_hit_cbe         : u8  ,
  pub min_hit_umb         : u8  ,
  pub max_hit_cor         : u8  ,
  pub max_hit_cbe         : u8  ,
  pub max_hit_umb         : u8  ,
  pub min_hit_all         : u8  ,
  pub max_hit_all         : u8  ,
  pub min_cos_theta       : f32 ,
  pub max_cos_theta       : f32 ,
  pub only_causal_hits    : bool,
  pub hit_cbe_acc         : u64 ,
  pub hit_umb_acc         : u64 ,
  pub hit_cor_acc         : u64 ,
  pub hit_all_acc         : u64 ,
  pub cos_theta_acc       : u64 ,
  pub nevents             : u64 ,
  pub hits_total          : u64 ,
  pub hits_rmvd_csl       : u64 ,
  pub hits_rmvd_ls        : u64 ,
  pub fh_must_be_umb      : bool,
  pub fh_umb_acc          : u64 ,
  pub ls_cleaning_t_err   : f64 ,
  pub thru_going          : bool,
  pub thru_going_acc      : u64 ,
  pub fhi_not_bot         : bool,
  pub fhi_not_bot_acc     : u64 ,
  pub fho_must_panel7     : bool,
  pub fho_must_panel7_acc : u64 ,
  pub lh_must_panel2      : bool, 
  pub lh_must_panel2_acc  : u64 ,
  pub hit_high_edep       : bool,
  pub hit_high_edep_acc   : u64 , 
}

impl TofCuts {

  pub fn new() -> Self {
    Self {
      min_hit_cor         : 0  ,
      min_hit_cbe         : 0  ,
      min_hit_umb         : 0  ,
      max_hit_cor         : 161,
      max_hit_cbe         : 161,
      max_hit_umb         : 161,
      min_hit_all         : 0  ,
      max_hit_all         : 161,
      min_cos_theta       : 0.0,
      max_cos_theta       : 1.0,
      only_causal_hits    : false,
      hit_cbe_acc         : 0,
      hit_umb_acc         : 0,
      hit_cor_acc         : 0,
      hit_all_acc         : 0,
      cos_theta_acc       : 0,
      nevents             : 0,
      hits_total          : 0,
      hits_rmvd_csl       : 0,
      hits_rmvd_ls        : 0,
      fh_must_be_umb      : false,
      fh_umb_acc          : 0 ,
      ls_cleaning_t_err   : 1e9 , // should be inf
      thru_going          : false,
      thru_going_acc      : 0 ,
      fhi_not_bot         : false,
      fhi_not_bot_acc     : 0 ,
      fho_must_panel7     : false,
      fho_must_panel7_acc : 0 ,
      lh_must_panel2      : false, 
      lh_must_panel2_acc  : 0 ,
      hit_high_edep       : false,
      hit_high_edep_acc   : 0 , 
    }
  }
   
  /// Zero out the event counter variables
  pub fn clear_stats(&mut self) {
    self.hit_cbe_acc         = 0; 
    self.hit_umb_acc         = 0; 
    self.hit_cor_acc         = 0;
    self.hit_all_acc         = 0; 
    self.cos_theta_acc       = 0;
    self.nevents             = 0;
    self.hits_total          = 0;
    self.hits_rmvd_csl       = 0;
    self.hits_rmvd_ls        = 0;
    self.fh_umb_acc          = 0;
    self.thru_going_acc      = 0;
    self.fhi_not_bot_acc     = 0;
    self.fho_must_panel7_acc = 0; 
    self.lh_must_panel2_acc  = 0; 
    self.hit_high_edep_acc   = 0;
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TofCuts {
  #[getter]
  fn get_min_hit_cor        (&self) -> u8   {
    self.min_hit_cor
  }

  #[getter]
  fn get_min_hit_cbe        (&self) -> u8   {
    self.min_hit_cbe
  }

  #[getter]
  fn get_min_hit_umb        (&self) -> u8   {
    self.min_hit_umb
  }

  #[getter]
  fn get_max_hit_cor        (&self) -> u8   {
    self.max_hit_cor
  }

  #[getter]
  fn get_max_hit_cbe        (&self) -> u8   {
    self.max_hit_cbe
  }

  #[getter]
  fn get_max_hit_umb        (&self) -> u8   {
    self.max_hit_umb
  }

  #[getter]
  fn get_min_hit_all        (&self) -> u8   {
    self.min_hit_all
  }

  #[getter]
  fn get_max_hit_all        (&self) -> u8   {
    self.max_hit_all
  }

  #[getter]
  fn get_min_cos_theta      (&self) -> f32  {
    self.min_cos_theta
  }

  #[getter]
  fn get_max_cos_theta      (&self) -> f32  {
    self.max_cos_theta
  }

  #[getter]
  fn get_only_causal_hits   (&self) -> bool {
    self.only_causal_hits
  }

  #[getter]
  fn get_hit_cbe_acc        (&self) -> u64  {
    self.hit_cbe_acc
  }

  #[getter]
  fn get_hit_umb_acc        (&self) -> u64  {
    self.hit_umb_acc
  }

  #[getter]
  fn get_hit_cor_acc        (&self) -> u64  {
    self.hit_cor_acc
  }

  #[getter]
  fn get_hit_all_acc        (&self) -> u64  {
    self.hit_all_acc
  }

  #[getter]
  fn get_cos_theta_acc      (&self) -> u64  {
    self.cos_theta_acc
  }

  #[getter]
  fn get_nevents            (&self) -> u64  {
    self.nevents
  }

  #[getter]
  fn get_hits_total         (&self) -> u64  {
    self.hits_total
  }

  #[getter]
  fn get_hits_rmvd_csl      (&self) -> u64  {
    self.hits_rmvd_csl
  }

  #[getter]
  fn get_hits_rmvd_ls       (&self) -> u64  {
    self.hits_rmvd_ls 
  }

  #[getter]
  fn get_fh_must_be_umb     (&self) -> bool {
    self.fh_must_be_umb
  }

  #[getter]
  fn get_fh_umb_acc         (&self) -> u64  {
    self.fh_umb_acc
  }

  #[getter]
  fn get_ls_cleaning_t_err  (&self) -> f64  {
    self.ls_cleaning_t_err
  }

  #[getter]
  fn get_thru_going         (&self) -> bool {
    self.thru_going
  }

  #[getter]
  fn get_thru_going_acc     (&self) -> u64  {
    self.thru_going_acc
  }

  #[getter]
  fn get_fhi_not_bot        (&self) -> bool {
    self.fhi_not_bot
  }

  #[getter]
  fn get_fhi_not_bot_acc    (&self) -> u64  {
    self.fhi_not_bot_acc
  }

  #[getter]
  fn get_fho_must_panel7    (&self) -> bool {
    self.fho_must_panel7
  }

  #[getter]
  fn get_fho_must_panel7_acc(&self) -> u64  {
    self.fho_must_panel7_acc
  }

  #[getter]
  fn get_lh_must_panel2     (&self) -> bool {
    self.lh_must_panel2
  }

  #[getter]
  fn get_lh_must_panel2_acc (&self) -> u64  {
    self.lh_must_panel2_acc
  }

  #[getter]
  fn get_hit_high_edep      (&self) -> bool {
    self.hit_high_edep 
  }

  #[getter]
  fn get_hit_high_edep_acc  (&self) -> u64  {
    self.hit_high_edep_acc
  }
}

