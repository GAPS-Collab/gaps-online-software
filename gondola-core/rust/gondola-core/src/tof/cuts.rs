// This file is part of gaps-online-software and published 
// under the GPLv3 license

use std::ops::AddAssign; 
use std::ops::Add;
use std::cmp::Ordering;
use crate::prelude::*;

/// A large number for the lightspeed cut.
/// This can be used for the error, which is 
/// in % of the lightspeed so everything 
/// > 1 is non-sensical
const NO_LIGHTSPEED_CUTS : f64 = 42e9;

/// Sets of cuts which can be imposed on 
/// TofEvents 
///
///
//FIXME - it is addmitedly a bit of a mess, since
//        it will perform hit cleanings 
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TofCuts {
  /// the number of events which the cut instance
  /// has seen. This is all events, both rejected
  /// and passed.
  #[serde(skip_serializing)]
  pub nevents             : u64 ,
  /// require at least N COR hits
  pub min_hit_cor         : u8  ,
  /// require at least N CBE hits
  pub min_hit_cbe         : u8  ,
  /// require at least N UMB hits
  pub min_hit_umb         : u8  ,
  pub max_hit_cor         : u8  ,
  pub max_hit_cbe         : u8  ,
  pub max_hit_umb         : u8  ,
  pub min_hit_all         : u8  ,
  pub max_hit_all         : u8  ,
  pub min_cos_theta       : f32 ,
  pub max_cos_theta       : f32 ,
  pub only_causal_hits    : bool,
  #[serde(skip_serializing)]
  pub hit_cbe_acc         : u64 ,
  #[serde(skip_serializing)]
  pub hit_umb_acc         : u64 ,
  #[serde(skip_serializing)]
  pub hit_cor_acc         : u64 ,
  #[serde(skip_serializing)]
  pub hit_all_acc         : u64 ,
  #[serde(skip_serializing)]
  pub fh_umb_acc          : u64 ,
  #[serde(skip_serializing)]
  pub cos_theta_acc       : u64 ,
  #[serde(skip_serializing)]
  pub hits_total          : u64 ,
  #[serde(skip_serializing)]
  pub hits_rmvd_csl       : u64 ,
  #[serde(skip_serializing)]
  pub hits_rmvd_ls        : u64 ,
  pub fh_must_be_umb      : bool,
  #[serde(skip_serializing)]
  pub ls_cleaning_t_err   : f64 ,
  pub thru_going          : bool,
  #[serde(skip_serializing)]
  pub thru_going_acc      : u64 ,
  pub fhi_not_bot         : bool,
  #[serde(skip_serializing)]
  pub fhi_not_bot_acc     : u64 ,
  pub fho_must_panel7     : bool,
  #[serde(skip_serializing)]
  pub fho_must_panel7_acc : u64 ,
  pub lh_must_panel2      : bool, 
  #[serde(skip_serializing)]
  pub lh_must_panel2_acc  : u64 ,
  pub hit_high_edep       : bool,
  #[serde(skip_serializing)]
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
      ls_cleaning_t_err   : NO_LIGHTSPEED_CUTS ,
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
  
  /// Write the settings to a toml file
  pub fn to_toml(&self, mut filename : String) {
    if !filename.ends_with(".toml") {
      filename += ".toml";
    }
    info!("Will write to file {}!", filename);
    match File::create(&filename) {
      Err(err) => {
        error!("Unable to open file {}! {}", filename, err);
      }
      Ok(mut file) => {
        match toml::to_string_pretty(&self) {
          Err(err) => {
            error!("Unable to serialize toml! {err}");
          }
          Ok(toml_string) => {
            match file.write_all(toml_string.as_bytes()) {
              Err(err) => error!("Unable to write to file {}! {}", filename, err),
              Ok(_)    => debug!("Wrote settings to {}!", filename)
            }
          }
        }
      }
    }
  }
  
  pub fn from_toml(filename : &str) -> Result<Self, SerializationError> {
    match File::open(filename) {
      Err(err) => {
        error!("Unable to open {}! {}", filename, err);
        return Err(SerializationError::TomlDecodingError);
      }
      Ok(mut file) => {
        let mut toml_string = String::from("");
        match file.read_to_string(&mut toml_string) {
          Err(err) => {
            error!("Unable to read {}! {}", filename, err);
            return Err(SerializationError::TomlDecodingError);
          }
          Ok(_) => {
            match toml::from_str(&toml_string) {
              Err(err) => {
                error!("Can't interpret toml! {}", err);
                return Err(SerializationError::TomlDecodingError);
              }
              Ok(cuts) => {
                return Ok(cuts);
              }
            }
          }
        }
      }
    }
  }

  /// Can two cut instances be added? 
  ///
  /// Void cuts will automatically be comptaible
  pub fn is_compatible(&self, other : &TofCuts) -> bool {
    if self.only_causal_hits != other.only_causal_hits {
      return false;
    }
    if self.min_hit_cor  != other.min_hit_cor {
      return false;
    }
    if self.min_hit_cbe  != other.min_hit_cbe {
      return false;
    }
    if self.min_hit_umb  != other.min_hit_umb {
      return false;
    }
    if self.max_hit_cor  != other.max_hit_cor {
      return false;
    }
    if self.max_hit_cbe  != other.max_hit_cbe {
      return false;
    }
    if self.max_hit_umb  != other.max_hit_umb {
      return false;
    }
    if self.min_hit_all  != other.min_hit_all {
      return false;
    }
    if self.max_hit_all  != other.max_hit_all {
      return false;
    }
    if self.ls_cleaning_t_err != other.ls_cleaning_t_err {
      return false;
    }
    if self.fh_must_be_umb != other.fh_must_be_umb {
      return false;
    } 
    if self.thru_going != other.thru_going {
      return false;
    }
    if self.fhi_not_bot != other.fhi_not_bot {
      return false;
    }
    if self.min_cos_theta != other.min_cos_theta {
      return false; 
    }
    if self.max_cos_theta != other.max_cos_theta {
      return false; 
    }
    if self.fho_must_panel7 != other.fho_must_panel7 {
      return false; 
    }
    if self.lh_must_panel2 != other.lh_must_panel2 {
      return false; 
    }
    if self.hit_high_edep != other.hit_high_edep { 
      return false;
    }
    true
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
    
  pub fn is_void(&self) -> bool {
    if self.min_hit_cor      != 0 {
      return false;
    }
    if self.min_hit_cbe      != 0 {
      return false;
    }
    if self.min_hit_umb      != 0 {
      return false;
    }
    if self.max_hit_cor      != 161 {
      return false;
    }
    if self.max_hit_cbe      != 161 {
      return false;
    }
    if self.max_hit_umb      != 161 {
      return false;  
    }
    if self.min_hit_all      != 0 {
      return false;
    }
    if self.max_hit_all      != 161 {
      return false;
    }
    if self.only_causal_hits {
      return false;
    }
    if self.ls_cleaning_t_err != NO_LIGHTSPEED_CUTS {
      return false;
    }
    if self.fh_must_be_umb != false {
      return false;
    }
    if self.thru_going != false {
      return false;
    }
    if self.fhi_not_bot != false {
      return false;
    }
    if self.min_cos_theta != 0.0 {
      return false; 
    }
    if self.max_cos_theta != 1.0 {
      return false; 
    }
    if self.fho_must_panel7 {
      return false; 
    }
    if self.lh_must_panel2 {
      return false; 
    }
    if self.hit_high_edep {
      return false;
    }
    return true;
  }

  pub fn get_acc_frac_hit_umb(&self) -> f64 {
    if self.nevents == 0 {
      return 0.0;
    }
    self.hit_umb_acc as f64/(self.nevents as f64)
  }
  
  pub fn get_acc_frac_hit_cbe(&self) -> f64 {
    if self.nevents == 0 {
      return 0.0;
    }
    self.hit_cbe_acc as f64/(self.nevents as f64)
  }
  
  pub fn get_acc_frac_hit_cor(&self) -> f64 {
    if self.nevents == 0 {
      return 0.0;
    }
    self.hit_cor_acc as f64/(self.nevents as f64)
  }
  
  pub fn get_acc_frac_hit_all(&self) -> f64 {
    if self.nevents == 0 {
      return 0.0;
    }
    self.hit_all_acc as f64/(self.nevents as f64)
  }
  
  pub fn get_acc_frac_cos_theta(&self) -> f64 {
    if self.nevents == 0 {
      return 0.0;
    }
    self.cos_theta_acc as f64/(self.nevents as f64)
  }
  
  pub fn get_acc_frac_fh_must_be_umb(&self)  -> f64 {
    if self.nevents == 0 {
      return 0.0;
    }
    self.fh_umb_acc as f64/(self.nevents as f64)
  }

  pub fn get_acc_frac_thru_going(&self)  -> f64 {
    if self.nevents == 0 {
      return 0.0;
    }
    self.thru_going_acc as f64/(self.nevents as f64)
  }

  pub fn get_acc_frac_fhi_not_bot(&self) -> f64 {
    if self.nevents == 0 {
      return 0.0;
    }
    self.fhi_not_bot_acc as f64/(self.nevents as f64) 
  }

  pub fn get_acc_frac_fho_must_panel7(&self) -> f64 {
    if self.nevents == 0 {
      return 0.0;
    }
    self.fho_must_panel7_acc as f64/(self.nevents as f64)
  }
  
  pub fn get_acc_frac_lh_must_panel2(&self) -> f64 {
    if self.nevents == 0 {
      return 0.0;
    }
    self.lh_must_panel2_acc as f64/(self.nevents as f64)
  }
  
  pub fn get_acc_frac_hit_high_edep(&self) -> f64 {
    if self.nevents == 0 {
      return 0.0;
    }
    self.hit_high_edep_acc as f64/(self.nevents as f64)
  }

  
  /// Check if an event passes the selection
  /// and update the counters. 
  /// If cleanings are enabled, this will 
  /// change the event in-place!
  pub fn accept(&mut self, ev : &mut TofEvent) -> bool {
    if self.is_void() {
      return true;
    }
    // The order of events is important. Hit cleaning 
    // comes before the application of cuts.
    let nhits        = ev.get_nhits() as u64;
    self.hits_total += nhits;
    self.nevents    += 1;
    // we need to make sure the times are 
    // calculated.properly. If they are, 
    // this won't do anything
    ev.normalize_hit_times();
    if self.only_causal_hits {
      let rm_pids = ev.remove_non_causal_hits();
      self.hits_rmvd_csl  += rm_pids.len() as u64;
    }
    if self.ls_cleaning_t_err != NO_LIGHTSPEED_CUTS {
      // FIXME - change type of ls_cleaning_t_err to f32
      let rm_pids_ls       = ev.lightspeed_cleaning(self.ls_cleaning_t_err as f32);
      self.hits_rmvd_ls   += rm_pids_ls.0.len() as u64;
    }
    // get number of cbe/umb/cor hits - only for valid hits
    let nhits_cbe   = ev.get_nhits_cbe() as u64;
    let nhits_umb   = ev.get_nhits_umb() as u64;
    let nhits_cor   = ev.get_nhits_cor() as u64;
    let clean_nhits = nhits_cbe + nhits_umb + nhits_cor;
    // check for min/max hits on cbe, umb, cor
    // these cuts are combined with AND
    if !((self.min_hit_all as u64 <= clean_nhits) && (clean_nhits <= self.max_hit_all as u64)) {
      return false;
    } else { 
      self.hit_all_acc += 1;
    }
    if !((self.min_hit_cbe as u64<= nhits_cbe) && (nhits_cbe <= self.max_hit_cbe as u64)) {
      return false;
    } else {
      self.hit_cbe_acc += 1;
    }
    if !((self.min_hit_umb as u64 <= nhits_umb) && (nhits_umb <= self.max_hit_umb as u64)) {
      return false;
    } else {
      self.hit_umb_acc += 1;
    }
    if !((self.min_hit_cor as u64 <= nhits_cor) && (nhits_cor <= self.max_hit_cor as u64)) {
      return false;
    } else {
      self.hit_cor_acc += 1;
    }
    //# at this point, it can still be that we don't have any TOF hits at all
    //# the following set of cuts can only be calculated if there are hits
    //#no_cos_possible = False 
    if self.fh_must_be_umb 
      || self.thru_going 
      || self.fhi_not_bot 
      || (self.min_cos_theta != 0.0) 
      || (self.max_cos_theta != 1.0) 
      || self.fho_must_panel7 
      || self.lh_must_panel2 
      || self.hit_high_edep {
      // in this casese, we need inner and outer hits and have them sorted 
          ev.hits.sort_by(|a,b| a.event_t0.partial_cmp(&b.event_t0).unwrap_or(Ordering::Greater));
          let hits_sorted = &ev.hits;
          if hits_sorted.len() == 0 {
            //if we don't have hits, we also don't fulfill any of these conditions. simple.
            return false;
          }
          let first_pid  = hits_sorted[0].paddle_id; 
          let last_pid   = hits_sorted.last().expect("No HITS!").paddle_id;
          let hits_inner: Vec<&TofHit> = hits_sorted.iter()
                                         .filter(|k| k.paddle_id < 61)
                                         .collect();
          let hits_outer: Vec<&TofHit> = hits_sorted.iter()
                                         .filter(|k| k.paddle_id > 60)
                                         .collect();
      //# now we are sure that there are hits
      if self.fh_must_be_umb {
        if  (first_pid < 61) || (first_pid > 108) {
          return false;
        } else {
          self.fh_umb_acc += 1;
        }
      } else {
        self.fh_umb_acc += 1;
      }
      if self.thru_going {
        //if  (last_pid in range(13,25) or 108 < last_pid):
        if (last_pid >= 13 && last_pid < 25) || 108 < last_pid {
          self.thru_going_acc += 1;
        } else {
          return false;
        }
      } else {
        self.thru_going_acc += 1;
      } 
      if self.fhi_not_bot {
        if hits_inner.len() == 0 {
            self.fhi_not_bot_acc += 1;
        } else if (12 < hits_inner[0].paddle_id) && (hits_inner[0].paddle_id < 25) {
          return false;
        } else {
          self.fhi_not_bot_acc += 1;
        }
      } else {
        self.fhi_not_bot_acc += 1;
      } 
      if self.min_cos_theta != 0.0 || self.max_cos_theta != 1.0 {
        let dist = hits_inner[0].distance(hits_outer[0])/1000.0;
        let cos_theta = f32::abs(hits_inner[0].z - hits_outer[0].z)/(1000.0*dist);  
        if !((self.min_cos_theta <= cos_theta) && (cos_theta <= self.max_cos_theta)) {
          return false;
        } else {
          self.cos_theta_acc += 1;
        }
        self.cos_theta_acc += 1;
      }
      if self.fho_must_panel7 {
        //if first_pid not in range(61, 73):
        if first_pid < 61 || first_pid >= 72 {
          return false; 
        } else {
          self.fho_must_panel7_acc += 1;
        }
      }
      if self.lh_must_panel2 {
        if last_pid < 13 || last_pid > 24 {
          return false; 
        } else {
          self.lh_must_panel2_acc += 1;
        }
      }
      if self.hit_high_edep {
        let mut found = false; 
        for h in hits_sorted {
          if h.get_edep() > 20.0 {
            self.lh_must_panel2_acc += 1;
            found = true;
            break;
          }
        }
        if !found {
          return false;
        }
      }
    }
    // if we arrive here, we passed everything
    true
  }

  /// Print out nicely formatted efficiencies
  pub fn pretty_print_efficiency(&self) -> String {
    let mut repr =  String::from("-- -- -- -- -- -- -- -- -- -- --");
    repr += &(format!("\n  TOTAL EVENTS : {}", self.nevents));
    repr += &(format!("\n    {} <= NHit(UMB) <= {} : {:.2} %", self.min_hit_umb, self.max_hit_umb, 100.0*self.get_acc_frac_hit_umb())); 
    repr += &(format!("\n    {} <= NHit(CBE) <= {} : {:.2} %", self.min_hit_cbe, self.max_hit_cbe, 100.0*self.get_acc_frac_hit_cbe())); 
    repr += &(format!("\n    {} <= NHit(COR) <= {} : {:.2} %", self.min_hit_cor, self.max_hit_cor, 100.0*self.get_acc_frac_hit_cor())); 
    repr += &(format!("\n    {} <= NHit(TOF) <= {} : {:.2} %", self.min_hit_all, self.max_hit_all, 100.0*self.get_acc_frac_hit_all())); 
    repr += &(format!("\n    {} <= COS(THET) <= {} : {:.2} %", self.min_cos_theta, self.max_cos_theta, self.get_acc_frac_cos_theta()));  
    if self.only_causal_hits {
      if self.hits_total > 0 {
        repr += &(format!("\n Removed {:.2} % of hits due to causality cut!", 100.0*self.hits_rmvd_csl as f64/self.hits_total as f64));
      }
    }
    if self.ls_cleaning_t_err != NO_LIGHTSPEED_CUTS {
      if self.hits_total > 0 {
        repr += &(format!("\n Removed {:.2} % of hits due to lightspeed cut!", 100.0*(self.hits_rmvd_ls as f64)/self.hits_total as f64));
      }
    }
    if self.fh_must_be_umb {
      repr += "\n First hit must be on UMB!";
      repr += &(format!("\n   -- Accepted {:.2} %", 100.0*self.get_acc_frac_fh_must_be_umb()));
    }
    if self.thru_going {
      repr += "\n Require through-going track!";
      repr += &(format!("\n   -- Accepted {:.2} %", 100.0*self.get_acc_frac_thru_going()));
    }
    if self.fhi_not_bot {
      repr += "\n Require first hit on the inner TOF can not be on the Bottom 12PP";
      repr += &(format!("\n   -- Accepted {:.2} %", 100.0*self.get_acc_frac_fhi_not_bot()));
    }
    if self.fho_must_panel7 {
      repr += "\n Require first hit on the outer TOF must be on panel7";
      repr += &(format!("\n   -- Accepted {:.2} %", 100.0*self.get_acc_frac_fho_must_panel7()));
    }
    if self.lh_must_panel2 {
      repr += "\n Require last hit must be on the bottom CBE panel";
      repr += &(format!("\n   -- Accepted {:.2} %", 100.0*self.get_acc_frac_lh_must_panel2()));
    }
    if self.hit_high_edep {
      repr += "\n Require that one hit has an edep > 20MeV";
      repr += &(format!("\n   -- Accepted {:.2} %", 100.0*self.get_acc_frac_hit_high_edep()));
    }
    repr +=  "\n-- -- -- -- -- -- -- -- -- -- --";
    //println!("{}",repr);
    repr
  }
}

impl Default for TofCuts {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for TofCuts {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TofCuts:");
    if self.is_void() {
      repr += " (void)>";
    } else {
      if self.only_causal_hits {
        repr += &(format!("\n -- removes non-causal hits!"));
      }
      // FIXME - 1e9 is a magic value
      if self.ls_cleaning_t_err != NO_LIGHTSPEED_CUTS {
        repr += "\n -- removes hits which are not correlated with the first hit!";
        repr += &(format!("\n --   assumed timing error {}", self.ls_cleaning_t_err));
      }
      if self.fh_must_be_umb {
        repr += "\n -- first hit must be on UMB";
      }
      if self.thru_going {
        repr += "\n -- require last hit on CBE BOT or COR (thru-going tracks)";
      }
      if self.fhi_not_bot {
        repr += "\n -- require that the first hit on the inner TOF is not on CBE BOT";
      }
      if self.fho_must_panel7 {
        repr += "\n -- require that the first hit on the outer TOF is on panel7";
      }
      if self.lh_must_panel2 {
        repr += "\n -- require that the last hit on the inner TOF is on CBE BOT";
      }
      if self.hit_high_edep {
        repr += "\n -- require that at least one hit has an edep of > 29MeV";
      }
      repr += &(format!("\n  {} <= NHit(UMB) <= {}", self.min_hit_umb, self.max_hit_umb)); 
      repr += &(format!("\n  {} <= NHit(CBE) <= {}", self.min_hit_cbe, self.max_hit_cbe)); 
      repr += &(format!("\n  {} <= NHit(COR) <= {}", self.min_hit_cor, self.max_hit_cor)); 
      repr += &(format!("\n  {} <= NHit(TOF) <= {}", self.min_hit_all, self.max_hit_all)); 
      repr += &(format!("\n  {} <= COS(THET) <= {}", self.min_cos_theta, self.max_cos_theta)); 
      repr += ">";
    }
    write!(f, "{}", repr)
  }
}

impl AddAssign for TofCuts {
  fn add_assign(&mut self, other : Self) {
    if !self.is_compatible(&other) {
      // not sure if that should raise panic?
      panic!("Cuts are not compatible!");
    }
    self.nevents             += other.nevents;
    self.hit_cbe_acc         += other.hit_cbe_acc; 
    self.hit_umb_acc         += other.hit_umb_acc; 
    self.hit_cor_acc         += other.hit_cor_acc;
    self.hit_all_acc         += other.hit_all_acc;
    self.cos_theta_acc       += other.cos_theta_acc; 
    self.hits_total          += other.hits_total;
    self.hits_rmvd_csl       += other.hits_rmvd_csl;
    self.hits_rmvd_ls        += other.hits_rmvd_ls;
    self.fh_umb_acc          += other.fh_umb_acc;
    self.thru_going_acc      += other.thru_going_acc;
    self.fhi_not_bot_acc     += other.fhi_not_bot_acc;
    self.fho_must_panel7_acc += other.fho_must_panel7_acc; 
    self.lh_must_panel2_acc  += other.lh_must_panel2_acc;
    self.hit_high_edep_acc   += other.hit_high_edep_acc;
  }
}

impl Add for TofCuts {

  type Output = TofCuts;

  fn add(self, other: Self) -> Self::Output {
    let mut output = self.clone();
    output += other;
    return output;
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TofCuts {
 
  /// Return a literal full deep copy 
  /// of the instance
  fn copy(&self) -> Self {
    self.clone()
  }

  #[pyo3(name="is_compatible")]
  fn is_compatible_py(&self, other : &Self) -> bool {
    self.is_compatible(other)
  }

  #[pyo3(name = "clear_stats")]
  fn clear_stats_py(&mut self) {
    self.clear_stats();
  }

  #[pyo3(name="accept")]
  fn accept_py(&mut self, event : &mut TofEvent) -> bool {
    self.accept(event)
  }

  #[getter]
  fn get_min_hit_cor        (&self) -> u8   {
    self.min_hit_cor
  }

  #[setter]
  fn set_min_hit_cor(&mut self, value : u8) -> PyResult<()> {
    self.min_hit_cor = value;
    Ok(())
  }

  fn __iadd__(&mut self, other : &TofCuts) {
    self.add_assign(*other);
  }

  fn __add__(&self, other : &TofCuts) -> TofCuts {
    let other_c = other.clone();
    self.add(other_c)
  }
 
  #[pyo3(name="to_toml")]
  fn to_toml_py(&self, filename : String) {
    self.to_toml(filename); 
  }
  
  #[staticmethod]
  #[pyo3(name="from_toml")]
  fn from_toml_py(filename : String) -> PyResult<Self> {
    match Self::from_toml(&filename) {
      Err(err)       => {
        return Err(PyValueError::new_err(err.to_string()));
      }
      Ok(cuts_)  => {
        return Ok(cuts_);
      }
    }
  }

  #[getter]
  fn void(&self) -> bool {
    self.is_void()
  }

  /// Return a prettily formated string with 
  /// the efficiency information for all the 
  /// individual cuts
  #[pyo3(name="pretty_print_efficiency")]
  fn pretty_print_efficiency_py(&self) -> String {
    self.pretty_print_efficiency()
  }

  #[getter]
  fn get_min_hit_cbe        (&self) -> u8   {
    self.min_hit_cbe
  }
  
  #[setter]
  fn set_min_hit_cbe(&mut self, value : u8) -> PyResult<()> {
    self.min_hit_cbe = value;
    Ok(())
  }

  #[getter]
  fn get_min_hit_umb        (&self) -> u8   {
    self.min_hit_umb
  }
  
  #[setter]
  fn set_min_hit_umb(&mut self, value : u8) -> PyResult<()> {
    self.min_hit_umb = value;
    Ok(())
  }

  #[getter]
  fn get_max_hit_cor        (&self) -> u8   {
    self.max_hit_cor
  }
  
  #[setter]
  fn set_max_hit_cor(&mut self, value : u8) -> PyResult<()> {
    self.max_hit_cor = value;
    Ok(())
  }

  #[getter]
  fn get_max_hit_cbe        (&self) -> u8   {
    self.max_hit_cbe
  }
  
  #[setter]
  fn set_max_hit_cbe(&mut self, value : u8) -> PyResult<()> {
    self.max_hit_cbe = value;
    Ok(())
  }

  #[getter]
  fn get_max_hit_umb        (&self) -> u8   {
    self.max_hit_umb
  }
  
  #[setter]
  fn set_max_hit_umb(&mut self, value : u8) -> PyResult<()> {
    self.max_hit_umb = value;
    Ok(())
  }

  #[getter]
  fn get_min_hit_all        (&self) -> u8   {
    self.min_hit_all
  }
  
  #[setter]
  fn set_min_hit_all(&mut self, value : u8) -> PyResult<()> {
    self.min_hit_all = value;
    Ok(())
  }

  #[getter]
  fn get_max_hit_all        (&self) -> u8   {
    self.max_hit_all
  }
  
  #[setter]
  fn set_max_hit_all(&mut self, value : u8) -> PyResult<()> {
    self.max_hit_all = value;
    Ok(())
  }

  #[getter]
  fn get_min_cos_theta      (&self) -> f32  {
    self.min_cos_theta
  }
  
  #[setter]
  fn set_min_cos_theta(&mut self, value : f32) -> PyResult<()> {
    self.min_cos_theta = value;
    Ok(())
  }

  #[getter]
  fn get_max_cos_theta      (&self) -> f32  {
    self.max_cos_theta
  }
  
  #[setter]
  fn set_max_cos_theta(&mut self, value : f32) -> PyResult<()> {
    self.max_cos_theta = value;
    Ok(())
  }

  #[getter]
  fn get_only_causal_hits   (&self) -> bool {
    self.only_causal_hits
  }
  
  #[setter]
  fn set_only_causal_hits(&mut self, value : bool) -> PyResult<()> {
    self.only_causal_hits = value;
    Ok(())
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
  
  #[setter]
  fn set_fh_must_be_umb(&mut self, value : bool) -> PyResult<()> {
    self.fh_must_be_umb = value;
    Ok(())
  }

  #[getter]
  fn get_fh_umb_acc         (&self) -> u64  {
    self.fh_umb_acc
  }

  #[getter]
  fn get_ls_cleaning_t_err  (&self) -> f64  {
    self.ls_cleaning_t_err
  }
  
  #[setter]
  fn set_ls_cleaning_t_err(&mut self, value : f64) -> PyResult<()> {
    self.ls_cleaning_t_err = value;
    Ok(())
  }

  #[getter]
  fn get_thru_going         (&self) -> bool {
    self.thru_going
  }
  
  #[setter]
  fn set_thru_going(&mut self, value : bool) -> PyResult<()> {
    self.thru_going = value;
    Ok(())
  }

  #[getter]
  fn get_thru_going_acc     (&self) -> u64  {
    self.thru_going_acc
  }

  #[getter]
  fn get_fhi_not_bot        (&self) -> bool {
    self.fhi_not_bot
  }
  
  #[setter]
  fn set_fhi_not_bot(&mut self, value : bool) -> PyResult<()> {
    self.fhi_not_bot = value;
    Ok(())
  }

  #[getter]
  fn get_fhi_not_bot_acc    (&self) -> u64  {
    self.fhi_not_bot_acc
  }

  #[getter]
  fn get_fho_must_panel7    (&self) -> bool {
    self.fho_must_panel7
  }
  
  #[setter]
  fn set_fho_must_panel7(&mut self, value : bool) -> PyResult<()> {
    self.fho_must_panel7 = value;
    Ok(())
  }

  #[getter]
  fn get_fho_must_panel7_acc(&self) -> u64  {
    self.fho_must_panel7_acc
  }

  #[getter]
  fn get_lh_must_panel2     (&self) -> bool {
    self.lh_must_panel2
  }
  
  #[setter]
  fn set_lh_must_panel2(&mut self, value : bool) -> PyResult<()> {
    self.lh_must_panel2 = value;
    Ok(())
  }

  #[getter]
  fn get_lh_must_panel2_acc (&self) -> u64  {
    self.lh_must_panel2_acc
  }

  #[getter]
  fn get_hit_high_edep      (&self) -> bool {
    self.hit_high_edep 
  }
  
  #[setter]
  fn set_hit_high_edep(&mut self, value : bool) -> PyResult<()> {
    self.hit_high_edep = value;
    Ok(())
  }

  #[getter]
  #[pyo3(name="acc_frac_hit_umb")]
  fn get_acc_frac_hit_umb_py(&self) -> f64 {
    self.get_acc_frac_hit_umb()
  } 
  
  #[getter]
  #[pyo3(name="acc_frac_hit_cbe")]
  fn get_acc_frac_hit_cbe_py(&self) -> f64 {
    self.get_acc_frac_hit_cbe()
  }
  
  #[getter]
  #[pyo3(name="acc_frac_hit_cor")]
  fn get_acc_frac_hit_cor_py(&self) -> f64 {
    self.get_acc_frac_hit_cor()
  }
  
  #[getter]
  #[pyo3(name="acc_frac_hit_all")]
  fn get_acc_frac_hit_all_py(&self) -> f64 {
    self.get_acc_frac_hit_all()
  }
  
  #[getter]
  #[pyo3(name="acc_frac_cos_theta")]
  fn get_acc_frac_cos_theta_py(&self) -> f64 {
    self.get_acc_frac_cos_theta()
  }
  
  #[getter]
  #[pyo3(name="acc_frac_fh_must_be_umb")]
  fn get_acc_frac_fh_must_be_umb_py(&self)  -> f64 {
    self.get_acc_frac_fh_must_be_umb()
  }

  #[getter]
  #[pyo3(name="get_frac_thru_going")]
  fn get_acc_frac_thru_going_py(&self)  -> f64 {
    self.get_acc_frac_thru_going()
  }

  #[getter]
  #[pyo3(name="acc_frac_fhi_not_bot")]
  fn get_acc_frac_fhi_not_bot_py(&self) -> f64 {
    self.get_acc_frac_fhi_not_bot()
  }

  #[getter]
  #[pyo3(name="acc_frac_fho_must_panel7")]
  fn get_acc_frac_fho_must_panel7_py(&self) -> f64 {
    self.get_acc_frac_fho_must_panel7()
  }
  
  #[getter]
  #[pyo3(name="acc_frac_lh_must_panel2")]
  fn get_acc_frac_lh_must_panel2_py(&self) -> f64 {
    self.get_acc_frac_lh_must_panel2()
  }

  #[getter]
  fn get_hit_high_edep_acc  (&self) -> u64  {
    self.hit_high_edep_acc
  }
}

#[cfg(feature="pybindings")]
pythonize!(TofCuts);

