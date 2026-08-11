//! The alerts are just clues to trigger an individual to respond to a certain situation
//!
// This file is part of gaps-online-software and published 
// under the GPLv3 license

// kudos Grace!! (who else ^^ ) 
//so these are the alerts relating to the tof:
//-- RB rate = 0 (and if its two we'll know a RAT crashed)
//-- RB temp (from RBMoni)  too cold < -40C
//-- RB temp (from RBMoni) too warm > 80C
//-- RAT labjack temp too cold < -40C
//-- RAT labjack temp too warm > 80C
//-- CAT labjack temp too cold < ?
//-- CAT labjack temp too warm > ?
//-- tofcpu temp too cold (from db?) < -40C
//-- tofcpu temp too warm (from db?) > 95C
//-- tofcpu temp too cold (CPUMoni) < -40C
//-- tofcpu temp too warm (CPUMoni) > 95C
//-- MTB FPGA temp too cold (MTBMoni) < -40C
//-- MTB FPGA temp too warm (MTBMoni) > 80C
//-- tofcpu cpu usage (from db?) > 90%
//-- MTB rate (db) > depends
//-- MTB rate (db) == 0
//-- MTB lost rate (db) > MTB rate

use std::fmt;
use std::collections::HashMap;

use std::time::Instant;
use std::fs::File;
use std::io::{
  Read,
  Write
};

use serde_json::json;

//use crate::serialization::SerializationError;
use crate::prelude::*;

/// helper function to parse output for TofBot
fn remove_from_word(s: String, word: &str) -> String {
  if let Some(index) = s.find(word) {
    // Keep everything up to the found index (not including the word itself)
    s[..index].to_string()
  } else {
    // If the word isn't found, return the original string
    s
  }
}

/// How did whatever went wrong,
/// go bad?
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum OutOfBound {
  TooHigh,
  TooLow,
  TooLowOrTooHigh,
  TooOld,
  Zero,
  Unknown,
}

impl fmt::Display for OutOfBound {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let repr : &str;
    match self {
      OutOfBound::Unknown         => repr = "Unknown",
      OutOfBound::TooHigh         => repr = "TooHigh",
      OutOfBound::TooLowOrTooHigh => repr = "TooLowOrTooHigh",
      OutOfBound::TooLow          => repr = "TooLow",
      OutOfBound::TooOld          => repr = "TooOld",
      OutOfBound::Zero            => repr = "Zero",
    };
    write!(f, "{}", repr)
  }
}

#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Shifters {
  Unknown,
  Grace,
  Kazu,
  Achim,
  TofBot,
}

pub trait Pageable {
  fn page(&self, content : String);

  fn resolve(&self) {
  }
}

impl fmt::Display for Shifters {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let repr : &str;
    match self {
      Shifters::Unknown => repr = "Unknown",
      Shifters::Grace   => repr = "Grace",
      Shifters::Kazu    => repr = "Kazu",
      Shifters::Achim   => repr = "Achim",
      Shifters::TofBot  => repr = "TofBot",
    };
    write!(f, "{}", repr)
  }
}

impl Pageable for Shifters {

  fn page(&self, content : String) {
    match self {
      Shifters::Grace => {
      }
      Shifters::Kazu  => {
      }
      Shifters::Achim => {
      }
      Shifters::TofBot => {
        // currently silence TofBot
        // Achim's channel
        //let url     = "https://hooks.slack.com/services/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";
        // TofBot channel
        //let url     = "https://hooks.slack.com/services/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";
        let message = format!("\u{1F916}\u{1F680}\u{1F388} [LIFTOF (Bot)]\n {}",content);
        let clean_message = remove_from_word(message, "tofbot_webhook");
        let data = json!({
          "text" : clean_message
        });
        match serde_json::to_string(&data) {
          Err(err) => {
            error!("Can not convert .json to string! {err}");
          }
          Ok(data_string) => {
            warn!("Alert system disabled! Not paging TofBot with {}", data_string);
        //    match ureq::post(url)
        //        .set("Content-Type", "application/json")
        //        .send_string(&data_string) {
        //      Err(err) => { 
        //        error!("Unable to send {} to TofBot! {err}", data_string);
        //      }
        //      Ok(response) => {
        //        match response.into_string() {
        //          Err(err) => {
        //            error!("Not able to read response! {err}");
        //          }
        //          Ok(body) => {
        //            info!("TofBot responded with {}", body);
        //          }
        //        }
        //      }
        //    }
          }
        }
      }
      _ => error!("Can't page unknown shifter!"),
    }
  }


  fn resolve(&self) {
    match self {
      Shifters::Grace => {
      }
      Shifters::Kazu  => {
      }
      Shifters::Achim => {
      }
      Shifters::TofBot => {
      }
      _ => error!("Can't page unknown shifter!"),
    }
  }
}

/// How did whatever went wrong,
/// go bad?
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Variable {
  Unknown,
  TriggerRate,
  Telemetry,
  LostTriggerRate,
  FPGATemp,
  CoreTemp,
  LabjackTemp,
  AvailableDiskSpace,
  DataMangling,
  MoniData,
}

impl fmt::Display for Variable {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let repr : &str;
    match self {
      Variable::Unknown            => repr = "Unknown",
      Variable::TriggerRate        => repr = "TriggerRate",
      Variable::Telemetry          => repr = "Telemetry",
      Variable::LostTriggerRate    => repr = "LostTriggerRate",
      Variable::CoreTemp           => repr = "CoreTemp",
      Variable::FPGATemp           => repr = "FPGATemp",
      Variable::LabjackTemp        => repr = "LabjackTemp",
      Variable::AvailableDiskSpace => repr = "AvailableDiskSpace",
      Variable::DataMangling       => repr = "DataMangling",
      Variable::MoniData           => repr = "MoniData",
    };
    write!(f, "{}", repr)
  }
}

/// How did whatever went wrong,
/// go bad?
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Component {
  MTB,
  CPU,
  CAT,
  System,
  RAT(u8),
  RB(u8),
}

impl fmt::Display for Component {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let repr : String;
    match self {
      Component::MTB      => repr = String::from("MTB"),
      Component::CPU      => repr = String::from("CPU"),
      Component::CAT      => repr = String::from("CAT"),
      Component::RAT(rid) => repr = format!("RAT{:02}", rid),
      Component::RB(rid)  => repr = format!("RB{:02}", rid),
      Component::System   => repr = String::from("System"),
    };
    write!(f, "{}", repr)
  }
}



/// Alerts are clues to trigger a 
/// reaction of an individual, so 
/// they are designed for humans, 
/// but should be issued by a 
/// machine
///
/// ISSUES: The design choice of going with 
/// a &str here and carry the livetime around
/// is admittedly a bit questionable, especially
/// since we might want to create similar 
/// alerts automatically (which I did not think 
/// of earlier)
#[derive(Debug, Clone)]
pub struct TofAlert<'a> {
  /// a unique identifier so that we can 
  /// look this up in settings
  pub key          : &'a str,
  /// description of what had happened
  pub descr        : &'a str,
  /// recommended course of action
  pub whattodo     : Vec<&'a str>,
  /// The configurable part of the alerm, 
  /// subscirbers, bounds and is the alarm 
  /// armed
  pub config       : TofAlertConfig,
  /// The variable which is an issue, e.g Rate
  pub variable     : Variable,
  /// The part of the TofSystem which caused the 
  /// issue
  pub component    : Component, 
  /// How did the variable get out-of-bounds? 
  /// E.g. too high, too low?
  pub outofbound   : OutOfBound,
  /// When did this issue occur?
  pub triggered    : Option<Instant>,
  /// How often did this alert page?
  pub n_paged      : u32,
}

impl TofAlert<'_> {
  
  pub fn acknowledge(&mut self) {
    self.triggered = None;
  }

  pub fn has_triggered(&self) -> bool {
    self.triggered.is_some()
  }

  //pub fn annoy(&self) {
  //  if self.triggered.is_some() && self.armed {
  //    for person in &self.config.subscribers {
  //      person.page();
  //    }
  //  }
  //}
  pub fn format_page(&self) -> String {
    let mut page_text = format!("<< Alert {} triggered!\n", self.key);
    page_text += &self.get_text();
    page_text += ">>";
    page_text
  }

  pub fn get_text(&self) -> String {
    let mut repr = format!("< {} | {} | {}", self.component, self.variable, self.outofbound);
    //repr    += &(format!(" -- descr : {}", self.descr));
    //repr    += &(format!(" -- action: {}", self.whattodo));
    //repr    += &(format!(" -- pages : "));
    //for shifter in &self.subscribers {
    //  repr += &(format!(" {} ", shifter)); 
    //}
    repr    += ">";
    repr
  }
  //pub fn new() -> Self {
  //  Self {
  //    message      : String::from(""),
  //    variable     :
  //    outofbound   : OutOfBound::Unknown,
  //    acknowledged : false
  //  }
  //}

  // FIXME find a solution for the unwraps
  pub fn trigger(&mut self, val : f32) {
    let mut triggered = false;
    match self.outofbound {
      OutOfBound::TooHigh => {
        match self.config.max_allowed {
          None => {
            error!("Set condition for {} as 'TooHigh', but 'max_allowed' is not specified", self.key);
          }
          Some(max_allowed) => {
            if val > max_allowed {
              warn!("Alarm {} triggered at {}!", self.key, max_allowed);
              triggered = true;
            }
          }
        }
      }
      OutOfBound::TooLow  => {
        match self.config.min_allowed {
          None => {
            error!("Set condition for {} as 'TooLow', but 'min_allowed' is not specified", self.key);
          }
          Some(min_allowed) => {
            if val < min_allowed {
              warn!("Alarm {} triggered at {}!", self.key, min_allowed);
              triggered = true;
            }
          }
        }
      }
      OutOfBound::TooLowOrTooHigh => {
        if val < self.config.min_allowed.unwrap() 
        || val > self.config.max_allowed.unwrap() {
          //warn!("Alarm {} triggered at {}!", self.key, max_allowed);
          triggered = true;
        }
      }
      OutOfBound::TooOld  => {
        match self.config.max_allowed {
          None => {
            error!("Set condition for {} as 'TooOld', but 'max_allowed' is not specified", self.key);
          }
          Some(max_allowed) => {
            if val > max_allowed {
              warn!("Alarm {} triggered at {}!", self.key, max_allowed);
              triggered = true;
            }
          }
        }
      }
      OutOfBound::Zero    => {
        if val == 0.0 {
          triggered = true;
        }
      }
      OutOfBound::Unknown => (),  
    }
    if triggered {
      self.triggered = Some(Instant::now());
    }
  }
 
  pub fn page(&mut self) {
    let content = self.format_page();
    self.n_paged += 1;
    for person in &self.config.subscribers {
      match person {
        Shifters::TofBot => {
          // always page TofBot
          person.page(content.clone());
        }
        _ => {
          if self.config.armed {
            // person who might need sleep, only 
            // page them when the alert is armed
            person.page(content.clone());
          }
        }
      }
    }
  }


  pub fn configure_from_manifest(&mut self, manifest : &TofAlertManifest)  {
    match manifest.get(self.key) {
      Some(cfg) => {
        self.config = cfg;
      }
      None => {
        error!("No entry for {} found in the alert manifest!", self.key);
      }
    }
  }
}

impl PartialEq for TofAlert<'_> {
  fn eq(&self, other: &Self) -> bool {
    self.variable      == other.variable
    && self.component  == other.component
    && self.outofbound == other.outofbound
  }
}

impl fmt::Display for TofAlert<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let repr = self.get_text();
    write!(f, "{}", repr)
  }
}

///// Create the temperature and rate alerts for a RB
//pub fn alert_factory_rb<'a>(rb_id      : u8) -> (TofAlert<'a>, TofAlert<'a>) {
//  let rate_alert = TofAlert {
//    key          : &(format!("RB{:02}_rate_zero", rb_id)),
//    descr        : &(format!("RB{:02} is not triggering!", rb_id)),
//    whattodo     : vec!["1) Run restart", "2) Restart liftof-rb", "3) Soft reboot RB", "4) Soft reboot MTB", "5) Hard reboot RAT",  "6) Hard reboot MTB"],
//    config       : TofAlertConfig::new(),
//    variable     : Variable::TriggerRate,
//    outofbound   : OutOfBound::Zero,
//    component    : Component::RB(rb_id),
//    triggered    : None,
//  };
//  
//  let temp_alert = TofAlert {
//    key          : format!("RB{:02}_temp", rb_id).as_str(),
//    descr        : format!("RB{:02} (FPGA) temperature is out of bounds!", rb_id).as_str(),
//    whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
//    config       : TofAlertConfig::new(),
//    variable     : Variable::TriggerRate,
//    outofbound   : OutOfBound::Zero,
//    component    : Component::RB(rb_id),
//    triggered    : None,
//  };
//  (rate_alert, temp_alert)
//}



//}
//
///// Create an alert for mtb temperature out of range
//pub fn alert_factory_rb_temp<'a>(rb_id       : u8,
//                                 subscribers : Vec<Shifters>,
//                                 outofbound  : OutOfBound) -> TofAlert<'a> {
//  TofAlert {
//    key          : "rb_temp",
//    descr        : "RB (FPGA) Temperature out of bounds!!",
//    whattodo     : vec!["This is critical! Check with everyone and then shut off this RB!"],
//    config       : TofAlertConfig::new(),
//    variable     : Variable::FPGATemp,
//    outofbound   : outofbound,
//    component    : Component::RB(rb_id),
//    acknowledged : false,
//    created      : Instant::now(),
//  }
//}
//
///// Create an alert for the case where the MTB is not triggering
//pub fn alert_factory_mtb_rate_zero<'a>(subscribers : Vec<Shifters>) -> TofAlert<'a> {
//  TofAlert {
//    key          : "mtb_rate_zero",
//    descr        : "MTB is not triggering!",
//    whattodo     : vec!["1) Check about TIU status", "2) Run restart", "3) If SSH available, debug with pybindings", "4) Soft reboot MTB", "5) Hard reboot MTB"],
//    config       : TofAlertConfig::new(),
//    variable     : Variable::TriggerRate,
//    outofbound   : OutOfBound::Zero,
//    component    : Component::MTB,
//    acknowledged : false,
//    created      : Instant::now(),
//  }
//}
//
///// Create an alert for mtb temperature out of range
//pub fn alert_factory_mtb_temp<'a>(subscribers : Vec<Shifters>,
//                                  outofbound  : OutOfBound) -> TofAlert<'a> {
//  TofAlert {
//    key          : "mtb_fpga_temp",
//    descr        : "MTB Temperature out of bounds!!",
//    whattodo     : vec!["This is critical! Check with everyone and then shut off the MTB!"],
//    config       : TofAlertConfig::new(),
//    variable     : Variable::FPGATemp,
//    outofbound   : outofbound,
//    component    : Component::MTB,
//    acknowledged : false,
//    created      : Instant::now(),
//  }
//}
//
///// Create an alert for mtb temperature out of range
//pub fn alert_factory_mtb_lost_rate<'a>(subscribers : Vec<Shifters>,
//                                       outofbound  : OutOfBound) -> TofAlert<'a> {
//  TofAlert {
//    key          : "mtb_lost_rate",
//    descr        : "MTB Lost rate is whacky!!",
//    whattodo     : vec!["Unclear course of action. Check with tracker, maybe nothing can be done!"],
//    config       : TofAlertConfig::new(),
//    variable     : Variable::LostTriggerRate,
//    outofbound   : outofbound,
//    component    : Component::MTB,
//    acknowledged : false,
//    created      : Instant::now(),
//  }
//}

/// Configure alerts from a .toml file
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TofAlertConfig {
  pub subscribers : Vec<Shifters>,
  pub armed       : bool,
  pub min_allowed : Option<f32>,
  pub max_allowed : Option<f32>,
  pub non_zero    : Option<bool>,
}

impl TofAlertConfig {
  pub fn new() -> Self {
    Self {
      subscribers : vec![Shifters::Grace,
                         Shifters::Kazu,
                         Shifters::Achim,
                         Shifters::TofBot],
      armed       : false,
      min_allowed : Some(-40.0),
      max_allowed : Some(80.0),
      non_zero    : Some(false),
    }
  }
}



/// Describes a .toml file with alert. 
/// settings
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TofAlertManifest {
  pub data_mangling   : TofAlertConfig,
  pub miss_tofevid    : TofAlertConfig,
  pub tofev_vs_merge  : TofAlertConfig,
  pub notof_vs_merge  : TofAlertConfig,
  pub frac_interest   : TofAlertConfig,
  pub mtb_lost_rate   : TofAlertConfig,
  pub mtb_fpga_temp   : TofAlertConfig,
  pub mtb_rate_zero   : TofAlertConfig,
  pub mtb_hk_too_old  : TofAlertConfig,
  pub cpu_core0_temp  : TofAlertConfig,
  pub cpu_core1_temp  : TofAlertConfig,
  pub cpu_hk_too_old  : TofAlertConfig,
  pub cpu_disk        : TofAlertConfig,
  pub rb01_temp       : TofAlertConfig,
  pub rb02_temp       : TofAlertConfig,  
  pub rb03_temp       : TofAlertConfig,  
  pub rb04_temp       : TofAlertConfig, 
  pub rb05_temp       : TofAlertConfig, 
  pub rb06_temp       : TofAlertConfig, 
  pub rb07_temp       : TofAlertConfig,
  pub rb08_temp       : TofAlertConfig, 
  pub rb09_temp       : TofAlertConfig, 
  pub rb11_temp       : TofAlertConfig, 
  pub rb13_temp       : TofAlertConfig,
  pub rb14_temp       : TofAlertConfig, 
  pub rb15_temp       : TofAlertConfig, 
  pub rb16_temp       : TofAlertConfig, 
  pub rb17_temp       : TofAlertConfig, 
  pub rb18_temp       : TofAlertConfig, 
  pub rb19_temp       : TofAlertConfig, 
  pub rb20_temp       : TofAlertConfig, 
  pub rb21_temp       : TofAlertConfig, 
  pub rb22_temp       : TofAlertConfig, 
  pub rb23_temp       : TofAlertConfig, 
  pub rb24_temp       : TofAlertConfig, 
  pub rb25_temp       : TofAlertConfig, 
  pub rb26_temp       : TofAlertConfig, 
  pub rb27_temp       : TofAlertConfig, 
  pub rb28_temp       : TofAlertConfig, 
  pub rb29_temp       : TofAlertConfig, 
  pub rb30_temp       : TofAlertConfig, 
  pub rb31_temp       : TofAlertConfig, 
  pub rb32_temp       : TofAlertConfig, 
  pub rb33_temp       : TofAlertConfig, 
  pub rb34_temp       : TofAlertConfig, 
  pub rb35_temp       : TofAlertConfig, 
  pub rb36_temp       : TofAlertConfig, 
  pub rb39_temp       : TofAlertConfig, 
  pub rb40_temp       : TofAlertConfig, 
  pub rb41_temp       : TofAlertConfig, 
  pub rb42_temp       : TofAlertConfig, 
  pub rb44_temp       : TofAlertConfig, 
  pub rb46_temp       : TofAlertConfig, 
  pub rb01_rate_zero  : TofAlertConfig, 
  pub rb02_rate_zero  : TofAlertConfig,  
  pub rb03_rate_zero  : TofAlertConfig,  
  pub rb04_rate_zero  : TofAlertConfig, 
  pub rb05_rate_zero  : TofAlertConfig, 
  pub rb06_rate_zero  : TofAlertConfig, 
  pub rb07_rate_zero  : TofAlertConfig, 
  pub rb08_rate_zero  : TofAlertConfig, 
  pub rb09_rate_zero  : TofAlertConfig, 
  pub rb11_rate_zero  : TofAlertConfig, 
  pub rb13_rate_zero  : TofAlertConfig, 
  pub rb14_rate_zero  : TofAlertConfig, 
  pub rb15_rate_zero  : TofAlertConfig, 
  pub rb16_rate_zero  : TofAlertConfig, 
  pub rb17_rate_zero  : TofAlertConfig, 
  pub rb18_rate_zero  : TofAlertConfig, 
  pub rb19_rate_zero  : TofAlertConfig, 
  pub rb20_rate_zero  : TofAlertConfig, 
  pub rb21_rate_zero  : TofAlertConfig, 
  pub rb22_rate_zero  : TofAlertConfig, 
  pub rb23_rate_zero  : TofAlertConfig, 
  pub rb24_rate_zero  : TofAlertConfig, 
  pub rb25_rate_zero  : TofAlertConfig, 
  pub rb26_rate_zero  : TofAlertConfig, 
  pub rb27_rate_zero  : TofAlertConfig, 
  pub rb28_rate_zero  : TofAlertConfig, 
  pub rb29_rate_zero  : TofAlertConfig, 
  pub rb30_rate_zero  : TofAlertConfig, 
  pub rb31_rate_zero  : TofAlertConfig, 
  pub rb32_rate_zero  : TofAlertConfig, 
  pub rb33_rate_zero  : TofAlertConfig, 
  pub rb34_rate_zero  : TofAlertConfig, 
  pub rb35_rate_zero  : TofAlertConfig, 
  pub rb36_rate_zero  : TofAlertConfig, 
  pub rb39_rate_zero  : TofAlertConfig, 
  pub rb40_rate_zero  : TofAlertConfig, 
  pub rb41_rate_zero  : TofAlertConfig, 
  pub rb42_rate_zero  : TofAlertConfig, 
  pub rb44_rate_zero  : TofAlertConfig, 
  pub rb46_rate_zero  : TofAlertConfig, 
  pub rb01_hk_too_old : TofAlertConfig, 
  pub rb02_hk_too_old : TofAlertConfig,  
  pub rb03_hk_too_old : TofAlertConfig,  
  pub rb04_hk_too_old : TofAlertConfig, 
  pub rb05_hk_too_old : TofAlertConfig, 
  pub rb06_hk_too_old : TofAlertConfig, 
  pub rb07_hk_too_old : TofAlertConfig, 
  pub rb08_hk_too_old : TofAlertConfig, 
  pub rb09_hk_too_old : TofAlertConfig, 
  pub rb11_hk_too_old : TofAlertConfig, 
  pub rb13_hk_too_old : TofAlertConfig, 
  pub rb14_hk_too_old : TofAlertConfig, 
  pub rb15_hk_too_old : TofAlertConfig, 
  pub rb16_hk_too_old : TofAlertConfig, 
  pub rb17_hk_too_old : TofAlertConfig, 
  pub rb18_hk_too_old : TofAlertConfig, 
  pub rb19_hk_too_old : TofAlertConfig, 
  pub rb20_hk_too_old : TofAlertConfig, 
  pub rb21_hk_too_old : TofAlertConfig, 
  pub rb22_hk_too_old : TofAlertConfig, 
  pub rb23_hk_too_old : TofAlertConfig, 
  pub rb24_hk_too_old : TofAlertConfig, 
  pub rb25_hk_too_old : TofAlertConfig, 
  pub rb26_hk_too_old : TofAlertConfig, 
  pub rb27_hk_too_old : TofAlertConfig, 
  pub rb28_hk_too_old : TofAlertConfig, 
  pub rb29_hk_too_old : TofAlertConfig, 
  pub rb30_hk_too_old : TofAlertConfig, 
  pub rb31_hk_too_old : TofAlertConfig, 
  pub rb32_hk_too_old : TofAlertConfig, 
  pub rb33_hk_too_old : TofAlertConfig, 
  pub rb34_hk_too_old : TofAlertConfig, 
  pub rb35_hk_too_old : TofAlertConfig, 
  pub rb36_hk_too_old : TofAlertConfig, 
  pub rb39_hk_too_old : TofAlertConfig, 
  pub rb40_hk_too_old : TofAlertConfig, 
  pub rb41_hk_too_old : TofAlertConfig, 
  pub rb42_hk_too_old : TofAlertConfig, 
  pub rb44_hk_too_old : TofAlertConfig, 
  pub rb46_hk_too_old : TofAlertConfig, 
}

impl TofAlertManifest {
  pub fn new() -> Self {
    Self {
      data_mangling   : TofAlertConfig::new(),
      miss_tofevid    : TofAlertConfig::new(),
      tofev_vs_merge  : TofAlertConfig::new(),
      notof_vs_merge  : TofAlertConfig::new(),
      frac_interest   : TofAlertConfig::new(),
      mtb_lost_rate   : TofAlertConfig::new(),
      mtb_fpga_temp   : TofAlertConfig::new(),
      mtb_rate_zero   : TofAlertConfig::new(),
      mtb_hk_too_old  : TofAlertConfig::new(),
      cpu_core0_temp  : TofAlertConfig::new(),
      cpu_core1_temp  : TofAlertConfig::new(),
      cpu_hk_too_old  : TofAlertConfig::new(),
      cpu_disk        : TofAlertConfig::new(),
      rb01_temp       : TofAlertConfig::new(),
      rb02_temp       : TofAlertConfig::new(),  
      rb03_temp       : TofAlertConfig::new(),  
      rb04_temp       : TofAlertConfig::new(), 
      rb05_temp       : TofAlertConfig::new(), 
      rb06_temp       : TofAlertConfig::new(), 
      rb07_temp       : TofAlertConfig::new(),
      rb08_temp       : TofAlertConfig::new(), 
      rb09_temp       : TofAlertConfig::new(), 
      rb11_temp       : TofAlertConfig::new(), 
      rb13_temp       : TofAlertConfig::new(),
      rb14_temp       : TofAlertConfig::new(), 
      rb15_temp       : TofAlertConfig::new(), 
      rb16_temp       : TofAlertConfig::new(), 
      rb17_temp       : TofAlertConfig::new(), 
      rb18_temp       : TofAlertConfig::new(), 
      rb19_temp       : TofAlertConfig::new(), 
      rb20_temp       : TofAlertConfig::new(), 
      rb21_temp       : TofAlertConfig::new(), 
      rb22_temp       : TofAlertConfig::new(), 
      rb23_temp       : TofAlertConfig::new(), 
      rb24_temp       : TofAlertConfig::new(), 
      rb25_temp       : TofAlertConfig::new(), 
      rb26_temp       : TofAlertConfig::new(), 
      rb27_temp       : TofAlertConfig::new(), 
      rb28_temp       : TofAlertConfig::new(), 
      rb29_temp       : TofAlertConfig::new(), 
      rb30_temp       : TofAlertConfig::new(), 
      rb31_temp       : TofAlertConfig::new(), 
      rb32_temp       : TofAlertConfig::new(), 
      rb33_temp       : TofAlertConfig::new(), 
      rb34_temp       : TofAlertConfig::new(), 
      rb35_temp       : TofAlertConfig::new(), 
      rb36_temp       : TofAlertConfig::new(), 
      rb39_temp       : TofAlertConfig::new(), 
      rb40_temp       : TofAlertConfig::new(), 
      rb41_temp       : TofAlertConfig::new(), 
      rb42_temp       : TofAlertConfig::new(), 
      rb44_temp       : TofAlertConfig::new(), 
      rb46_temp       : TofAlertConfig::new(), 
      rb01_rate_zero  : TofAlertConfig::new(), 
      rb02_rate_zero  : TofAlertConfig::new(),  
      rb03_rate_zero  : TofAlertConfig::new(),  
      rb04_rate_zero  : TofAlertConfig::new(), 
      rb05_rate_zero  : TofAlertConfig::new(), 
      rb06_rate_zero  : TofAlertConfig::new(), 
      rb07_rate_zero  : TofAlertConfig::new(), 
      rb08_rate_zero  : TofAlertConfig::new(), 
      rb09_rate_zero  : TofAlertConfig::new(), 
      rb11_rate_zero  : TofAlertConfig::new(), 
      rb13_rate_zero  : TofAlertConfig::new(), 
      rb14_rate_zero  : TofAlertConfig::new(), 
      rb15_rate_zero  : TofAlertConfig::new(), 
      rb16_rate_zero  : TofAlertConfig::new(), 
      rb17_rate_zero  : TofAlertConfig::new(), 
      rb18_rate_zero  : TofAlertConfig::new(), 
      rb19_rate_zero  : TofAlertConfig::new(), 
      rb20_rate_zero  : TofAlertConfig::new(), 
      rb21_rate_zero  : TofAlertConfig::new(), 
      rb22_rate_zero  : TofAlertConfig::new(), 
      rb23_rate_zero  : TofAlertConfig::new(), 
      rb24_rate_zero  : TofAlertConfig::new(), 
      rb25_rate_zero  : TofAlertConfig::new(), 
      rb26_rate_zero  : TofAlertConfig::new(), 
      rb27_rate_zero  : TofAlertConfig::new(), 
      rb28_rate_zero  : TofAlertConfig::new(), 
      rb29_rate_zero  : TofAlertConfig::new(), 
      rb30_rate_zero  : TofAlertConfig::new(), 
      rb31_rate_zero  : TofAlertConfig::new(), 
      rb32_rate_zero  : TofAlertConfig::new(), 
      rb33_rate_zero  : TofAlertConfig::new(), 
      rb34_rate_zero  : TofAlertConfig::new(), 
      rb35_rate_zero  : TofAlertConfig::new(), 
      rb36_rate_zero  : TofAlertConfig::new(), 
      rb39_rate_zero  : TofAlertConfig::new(), 
      rb40_rate_zero  : TofAlertConfig::new(), 
      rb41_rate_zero  : TofAlertConfig::new(), 
      rb42_rate_zero  : TofAlertConfig::new(), 
      rb44_rate_zero  : TofAlertConfig::new(), 
      rb46_rate_zero  : TofAlertConfig::new(), 
      rb01_hk_too_old : TofAlertConfig::new(), 
      rb02_hk_too_old : TofAlertConfig::new(),  
      rb03_hk_too_old : TofAlertConfig::new(),  
      rb04_hk_too_old : TofAlertConfig::new(), 
      rb05_hk_too_old : TofAlertConfig::new(), 
      rb06_hk_too_old : TofAlertConfig::new(), 
      rb07_hk_too_old : TofAlertConfig::new(), 
      rb08_hk_too_old : TofAlertConfig::new(), 
      rb09_hk_too_old : TofAlertConfig::new(), 
      rb11_hk_too_old : TofAlertConfig::new(), 
      rb13_hk_too_old : TofAlertConfig::new(), 
      rb14_hk_too_old : TofAlertConfig::new(), 
      rb15_hk_too_old : TofAlertConfig::new(), 
      rb16_hk_too_old : TofAlertConfig::new(), 
      rb17_hk_too_old : TofAlertConfig::new(), 
      rb18_hk_too_old : TofAlertConfig::new(), 
      rb19_hk_too_old : TofAlertConfig::new(), 
      rb20_hk_too_old : TofAlertConfig::new(), 
      rb21_hk_too_old : TofAlertConfig::new(), 
      rb22_hk_too_old : TofAlertConfig::new(), 
      rb23_hk_too_old : TofAlertConfig::new(), 
      rb24_hk_too_old : TofAlertConfig::new(), 
      rb25_hk_too_old : TofAlertConfig::new(), 
      rb26_hk_too_old : TofAlertConfig::new(), 
      rb27_hk_too_old : TofAlertConfig::new(), 
      rb28_hk_too_old : TofAlertConfig::new(), 
      rb29_hk_too_old : TofAlertConfig::new(), 
      rb30_hk_too_old : TofAlertConfig::new(), 
      rb31_hk_too_old : TofAlertConfig::new(), 
      rb32_hk_too_old : TofAlertConfig::new(), 
      rb33_hk_too_old : TofAlertConfig::new(), 
      rb34_hk_too_old : TofAlertConfig::new(), 
      rb35_hk_too_old : TofAlertConfig::new(), 
      rb36_hk_too_old : TofAlertConfig::new(), 
      rb39_hk_too_old : TofAlertConfig::new(), 
      rb40_hk_too_old : TofAlertConfig::new(), 
      rb41_hk_too_old : TofAlertConfig::new(), 
      rb42_hk_too_old : TofAlertConfig::new(), 
      rb44_hk_too_old : TofAlertConfig::new(), 
      rb46_hk_too_old : TofAlertConfig::new(), 
    }
  }

  pub fn keys(&self) -> Vec<&'static str> {
    let keys = vec!["data_mangling",  
                    "miss_tofevid",   
                    "tofev_vs_merge", 
                    "notof_vs_merge", 
                    "frac_interest",  
                    "mtb_lost_rate",
                    "mtb_fpga_temp",
                    "mtb_rate_zero",
                    "mtb_hk_too_old",
                    "cpu_core0_temp",
                    "cpu_core1_temp",
                    "cpu_hk_too_old",
                    "cpu_disk",
                    "rb01_temp",
                    "rb02_temp",  
                    "rb03_temp",  
                    "rb04_temp", 
                    "rb05_temp", 
                    "rb06_temp", 
                    "rb07_temp",
                    "rb08_temp", 
                    "rb09_temp", 
                    "rb11_temp", 
                    "rb13_temp",
                    "rb14_temp", 
                    "rb15_temp", 
                    "rb16_temp", 
                    "rb17_temp", 
                    "rb18_temp", 
                    "rb19_temp", 
                    "rb20_temp", 
                    "rb21_temp", 
                    "rb22_temp", 
                    "rb23_temp", 
                    "rb24_temp", 
                    "rb25_temp", 
                    "rb26_temp", 
                    "rb27_temp", 
                    "rb28_temp", 
                    "rb29_temp", 
                    "rb30_temp", 
                    "rb31_temp", 
                    "rb32_temp", 
                    "rb33_temp", 
                    "rb34_temp", 
                    "rb35_temp", 
                    "rb36_temp", 
                    "rb39_temp", 
                    "rb40_temp", 
                    "rb41_temp", 
                    "rb42_temp", 
                    "rb44_temp", 
                    "rb46_temp", 
                    "rb01_rate_zero", 
                    "rb02_rate_zero",  
                    "rb03_rate_zero",  
                    "rb04_rate_zero", 
                    "rb05_rate_zero", 
                    "rb06_rate_zero", 
                    "rb07_rate_zero", 
                    "rb08_rate_zero", 
                    "rb09_rate_zero", 
                    "rb11_rate_zero", 
                    "rb13_rate_zero", 
                    "rb14_rate_zero", 
                    "rb15_rate_zero", 
                    "rb16_rate_zero", 
                    "rb17_rate_zero", 
                    "rb18_rate_zero", 
                    "rb19_rate_zero", 
                    "rb20_rate_zero", 
                    "rb21_rate_zero", 
                    "rb22_rate_zero", 
                    "rb23_rate_zero", 
                    "rb24_rate_zero", 
                    "rb25_rate_zero", 
                    "rb26_rate_zero", 
                    "rb27_rate_zero", 
                    "rb28_rate_zero", 
                    "rb29_rate_zero", 
                    "rb30_rate_zero", 
                    "rb31_rate_zero", 
                    "rb32_rate_zero", 
                    "rb33_rate_zero", 
                    "rb34_rate_zero", 
                    "rb35_rate_zero", 
                    "rb36_rate_zero", 
                    "rb39_rate_zero", 
                    "rb40_rate_zero", 
                    "rb41_rate_zero", 
                    "rb42_rate_zero", 
                    "rb44_rate_zero", 
                    "rb46_rate_zero", 
                    "rb01_hk_too_old", 
                    "rb02_hk_too_old",  
                    "rb03_hk_too_old",  
                    "rb04_hk_too_old", 
                    "rb05_hk_too_old", 
                    "rb06_hk_too_old", 
                    "rb07_hk_too_old", 
                    "rb08_hk_too_old", 
                    "rb09_hk_too_old", 
                    "rb11_hk_too_old", 
                    "rb13_hk_too_old", 
                    "rb14_hk_too_old", 
                    "rb15_hk_too_old", 
                    "rb16_hk_too_old", 
                    "rb17_hk_too_old", 
                    "rb18_hk_too_old", 
                    "rb19_hk_too_old", 
                    "rb20_hk_too_old", 
                    "rb21_hk_too_old", 
                    "rb22_hk_too_old", 
                    "rb23_hk_too_old", 
                    "rb24_hk_too_old", 
                    "rb25_hk_too_old", 
                    "rb26_hk_too_old", 
                    "rb27_hk_too_old", 
                    "rb28_hk_too_old", 
                    "rb29_hk_too_old", 
                    "rb30_hk_too_old", 
                    "rb31_hk_too_old", 
                    "rb32_hk_too_old", 
                    "rb33_hk_too_old", 
                    "rb34_hk_too_old", 
                    "rb35_hk_too_old", 
                    "rb36_hk_too_old", 
                    "rb39_hk_too_old", 
                    "rb40_hk_too_old", 
                    "rb41_hk_too_old", 
                    "rb42_hk_too_old", 
                    "rb44_hk_too_old", 
                    "rb46_hk_too_old", 
                    ];
    return keys;
  }

  pub fn get(&self, key :  &str) -> Option<TofAlertConfig> {
    match key {
      "data_mangling"   => Some(self.data_mangling.clone()),  
      "miss_tofevid"    => Some(self.miss_tofevid.clone()),   
      "tofev_vs_merge"  => Some(self.tofev_vs_merge.clone()), 
      "notof_vs_merge"  => Some(self.notof_vs_merge.clone()), 
      "frac_interest"   => Some(self.frac_interest.clone()),  
      "mtb_lost_rate"   => Some(self.mtb_lost_rate.clone()),
      "mtb_fpga_temp"   => Some(self.mtb_fpga_temp.clone()),
      "mtb_rate_zero"   => Some(self.mtb_rate_zero.clone()),
      "mtb_hk_too_old"  => Some(self.mtb_hk_too_old.clone()),
      "cpu_core0_temp"  => Some(self.cpu_core0_temp.clone()),
      "cpu_core1_temp"  => Some(self.cpu_core1_temp.clone()),
      "cpu_hk_too_old"  => Some(self.cpu_hk_too_old.clone()),
      "cpu_disk"        => Some(self.cpu_disk.clone()),
      "rb01_temp"       => Some(self.rb01_temp.clone()),
      "rb02_temp"       => Some(self.rb02_temp.clone()),  
      "rb03_temp"       => Some(self.rb03_temp.clone()),  
      "rb04_temp"       => Some(self.rb04_temp.clone()), 
      "rb05_temp"       => Some(self.rb05_temp.clone()), 
      "rb06_temp"       => Some(self.rb06_temp.clone()), 
      "rb07_temp"       => Some(self.rb07_temp.clone()),
      "rb08_temp"       => Some(self.rb08_temp.clone()), 
      "rb09_temp"       => Some(self.rb09_temp.clone()), 
      "rb11_temp"       => Some(self.rb11_temp.clone()), 
      "rb13_temp"       => Some(self.rb13_temp.clone()),
      "rb14_temp"       => Some(self.rb14_temp.clone()), 
      "rb15_temp"       => Some(self.rb15_temp.clone()), 
      "rb16_temp"       => Some(self.rb16_temp.clone()), 
      "rb17_temp"       => Some(self.rb17_temp.clone()), 
      "rb18_temp"       => Some(self.rb18_temp.clone()), 
      "rb19_temp"       => Some(self.rb19_temp.clone()), 
      "rb20_temp"       => Some(self.rb20_temp.clone()), 
      "rb21_temp"       => Some(self.rb21_temp.clone()), 
      "rb22_temp"       => Some(self.rb22_temp.clone()), 
      "rb23_temp"       => Some(self.rb23_temp.clone()), 
      "rb24_temp"       => Some(self.rb24_temp.clone()), 
      "rb25_temp"       => Some(self.rb25_temp.clone()), 
      "rb26_temp"       => Some(self.rb26_temp.clone()), 
      "rb27_temp"       => Some(self.rb27_temp.clone()), 
      "rb28_temp"       => Some(self.rb28_temp.clone()), 
      "rb29_temp"       => Some(self.rb29_temp.clone()), 
      "rb30_temp"       => Some(self.rb30_temp.clone()), 
      "rb31_temp"       => Some(self.rb31_temp.clone()), 
      "rb32_temp"       => Some(self.rb32_temp.clone()), 
      "rb33_temp"       => Some(self.rb33_temp.clone()), 
      "rb34_temp"       => Some(self.rb34_temp.clone()), 
      "rb35_temp"       => Some(self.rb35_temp.clone()), 
      "rb36_temp"       => Some(self.rb36_temp.clone()), 
      "rb39_temp"       => Some(self.rb39_temp.clone()), 
      "rb40_temp"       => Some(self.rb40_temp.clone()), 
      "rb41_temp"       => Some(self.rb41_temp.clone()), 
      "rb42_temp"       => Some(self.rb42_temp.clone()), 
      "rb44_temp"       => Some(self.rb44_temp.clone()), 
      "rb46_temp"       => Some(self.rb46_temp.clone()), 
      "rb01_rate_zero"  => Some(self.rb01_rate_zero.clone()), 
      "rb02_rate_zero"  => Some(self.rb02_rate_zero.clone()),  
      "rb03_rate_zero"  => Some(self.rb03_rate_zero.clone()),  
      "rb04_rate_zero"  => Some(self.rb04_rate_zero.clone()), 
      "rb05_rate_zero"  => Some(self.rb05_rate_zero.clone()), 
      "rb06_rate_zero"  => Some(self.rb06_rate_zero.clone()), 
      "rb07_rate_zero"  => Some(self.rb07_rate_zero.clone()), 
      "rb08_rate_zero"  => Some(self.rb08_rate_zero.clone()), 
      "rb09_rate_zero"  => Some(self.rb09_rate_zero.clone()), 
      "rb11_rate_zero"  => Some(self.rb11_rate_zero.clone()), 
      "rb13_rate_zero"  => Some(self.rb13_rate_zero.clone()), 
      "rb14_rate_zero"  => Some(self.rb14_rate_zero.clone()), 
      "rb15_rate_zero"  => Some(self.rb15_rate_zero.clone()), 
      "rb16_rate_zero"  => Some(self.rb16_rate_zero.clone()), 
      "rb17_rate_zero"  => Some(self.rb17_rate_zero.clone()), 
      "rb18_rate_zero"  => Some(self.rb18_rate_zero.clone()), 
      "rb19_rate_zero"  => Some(self.rb19_rate_zero.clone()), 
      "rb20_rate_zero"  => Some(self.rb20_rate_zero.clone()), 
      "rb21_rate_zero"  => Some(self.rb21_rate_zero.clone()), 
      "rb22_rate_zero"  => Some(self.rb22_rate_zero.clone()), 
      "rb23_rate_zero"  => Some(self.rb23_rate_zero.clone()), 
      "rb24_rate_zero"  => Some(self.rb24_rate_zero.clone()), 
      "rb25_rate_zero"  => Some(self.rb25_rate_zero.clone()), 
      "rb26_rate_zero"  => Some(self.rb26_rate_zero.clone()), 
      "rb27_rate_zero"  => Some(self.rb27_rate_zero.clone()), 
      "rb28_rate_zero"  => Some(self.rb28_rate_zero.clone()), 
      "rb29_rate_zero"  => Some(self.rb29_rate_zero.clone()), 
      "rb30_rate_zero"  => Some(self.rb30_rate_zero.clone()), 
      "rb31_rate_zero"  => Some(self.rb31_rate_zero.clone()), 
      "rb32_rate_zero"  => Some(self.rb32_rate_zero.clone()), 
      "rb33_rate_zero"  => Some(self.rb33_rate_zero.clone()), 
      "rb34_rate_zero"  => Some(self.rb34_rate_zero.clone()), 
      "rb35_rate_zero"  => Some(self.rb35_rate_zero.clone()), 
      "rb36_rate_zero"  => Some(self.rb36_rate_zero.clone()), 
      "rb39_rate_zero"  => Some(self.rb39_rate_zero.clone()), 
      "rb40_rate_zero"  => Some(self.rb40_rate_zero.clone()), 
      "rb41_rate_zero"  => Some(self.rb41_rate_zero.clone()), 
      "rb42_rate_zero"  => Some(self.rb42_rate_zero.clone()), 
      "rb44_rate_zero"  => Some(self.rb44_rate_zero.clone()), 
      "rb46_rate_zero"  => Some(self.rb46_rate_zero.clone()), 
      "rb01_hk_too_old" => Some(self.rb01_hk_too_old.clone()), 
      "rb02_hk_too_old" => Some(self.rb02_hk_too_old.clone()),  
      "rb03_hk_too_old" => Some(self.rb03_hk_too_old.clone()),  
      "rb04_hk_too_old" => Some(self.rb04_hk_too_old.clone()), 
      "rb05_hk_too_old" => Some(self.rb05_hk_too_old.clone()), 
      "rb06_hk_too_old" => Some(self.rb06_hk_too_old.clone()), 
      "rb07_hk_too_old" => Some(self.rb07_hk_too_old.clone()), 
      "rb08_hk_too_old" => Some(self.rb08_hk_too_old.clone()), 
      "rb09_hk_too_old" => Some(self.rb09_hk_too_old.clone()), 
      "rb11_hk_too_old" => Some(self.rb11_hk_too_old.clone()), 
      "rb13_hk_too_old" => Some(self.rb13_hk_too_old.clone()), 
      "rb14_hk_too_old" => Some(self.rb14_hk_too_old.clone()), 
      "rb15_hk_too_old" => Some(self.rb15_hk_too_old.clone()), 
      "rb16_hk_too_old" => Some(self.rb16_hk_too_old.clone()), 
      "rb17_hk_too_old" => Some(self.rb17_hk_too_old.clone()), 
      "rb18_hk_too_old" => Some(self.rb18_hk_too_old.clone()), 
      "rb19_hk_too_old" => Some(self.rb19_hk_too_old.clone()), 
      "rb20_hk_too_old" => Some(self.rb20_hk_too_old.clone()), 
      "rb21_hk_too_old" => Some(self.rb21_hk_too_old.clone()), 
      "rb22_hk_too_old" => Some(self.rb22_hk_too_old.clone()), 
      "rb23_hk_too_old" => Some(self.rb23_hk_too_old.clone()), 
      "rb24_hk_too_old" => Some(self.rb24_hk_too_old.clone()), 
      "rb25_hk_too_old" => Some(self.rb25_hk_too_old.clone()), 
      "rb26_hk_too_old" => Some(self.rb26_hk_too_old.clone()), 
      "rb27_hk_too_old" => Some(self.rb27_hk_too_old.clone()), 
      "rb28_hk_too_old" => Some(self.rb28_hk_too_old.clone()), 
      "rb29_hk_too_old" => Some(self.rb29_hk_too_old.clone()), 
      "rb30_hk_too_old" => Some(self.rb30_hk_too_old.clone()), 
      "rb31_hk_too_old" => Some(self.rb31_hk_too_old.clone()), 
      "rb32_hk_too_old" => Some(self.rb32_hk_too_old.clone()), 
      "rb33_hk_too_old" => Some(self.rb33_hk_too_old.clone()), 
      "rb34_hk_too_old" => Some(self.rb34_hk_too_old.clone()), 
      "rb35_hk_too_old" => Some(self.rb35_hk_too_old.clone()), 
      "rb36_hk_too_old" => Some(self.rb36_hk_too_old.clone()), 
      "rb39_hk_too_old" => Some(self.rb39_hk_too_old.clone()), 
      "rb40_hk_too_old" => Some(self.rb40_hk_too_old.clone()), 
      "rb41_hk_too_old" => Some(self.rb41_hk_too_old.clone()), 
      "rb42_hk_too_old" => Some(self.rb42_hk_too_old.clone()), 
      "rb44_hk_too_old" => Some(self.rb44_hk_too_old.clone()), 
      "rb46_hk_too_old" => Some(self.rb46_hk_too_old.clone()), 
      _                => None
    }
  }
  
  pub fn from_toml(filename : &str) -> Result<Self, SerializationError> {
    match File::open(filename) {
      Err(err) => {
        println!("Unable to open {}! {}", filename, err);
        return Err(SerializationError::TomlDecodingError);
      }
      Ok(mut file) => {
        let mut toml_string = String::from("");
        match file.read_to_string(&mut toml_string) {
          Err(err) => {
            println!("Unable to read {}! {}", filename, err);
            return Err(SerializationError::TomlDecodingError);
          }
          Ok(_) => {
            match toml::from_str::<TofAlertManifest>(&toml_string) {
              Err(err) => {
                println!("Can't interpret toml! {}", err);
                return Err(SerializationError::TomlDecodingError);
              }
              Ok(manifest) => {
                Ok(manifest)
              }
            }
          }
        }
      }
    }
  }
  
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
  
}


/// Load all alerts from the manifest
pub fn load_alerts<'a>(manifest : TofAlertManifest) -> HashMap<&'a str, TofAlert<'a>> {
  let mut alerts = HashMap::from([("mtb_rate_zero", TofAlert {
                                                        key          : "mtb_rate_zero",
                                                        descr        : "MTB is not triggering!",
                                                        whattodo     : vec!["1) Check about TIU status", "2) Run restart", "3) If SSH available, debug with pybindings", "4) Soft reboot MTB", "5) Hard reboot MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::MTB,
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                   ("mtb_lost_rate", TofAlert { 
                                                        key          : "mtb_lost_rate",
                                                        descr        : "MTB Lost rate is whacky!!",
                                                        whattodo     : vec!["Unclear course of action. Check with tracker, maybe nothing can be done!"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::LostTriggerRate,
                                                        outofbound   : OutOfBound::TooHigh,
                                                        component    : Component::MTB,
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("mtb_fpga_temp", TofAlert {
                                                        key          : "mtb_fpga_temp",
                                                        descr        : "MTB Temperature out of bounds!!",
                                                        whattodo     : vec!["This is critical! Check with everyone and then shut off the MTB!"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::MTB,
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("mtb_hk_too_old" , TofAlert {
                                                        key          : "mtb_hk_too_old", 
                                                        descr        : "MTBMoniData is out-of-date!",
                                                        whattodo     : vec!["Critical! This most likely means we are not triggering!" ,  "1) Check with tracker", "2) Run restart", "3) Soft reboot MTB", "4) Powercycle CAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(40),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("cpu_core0_temp", TofAlert {
                                                        key          : "cpu_core0_temp",
                                                        descr        : "CPU Core0 temperatrue is out of bounds!",
                                                        whattodo     : vec!["This is critical! Check with everyone and then stop the run and likely turn off the CAT!"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::CoreTemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::CPU,
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("cpu_core1_temp", TofAlert {
                                                        key          : "cpu_core1_temp",
                                                        descr        : "CPU Core1 temperatrue is out of bounds!",
                                                        whattodo     : vec!["This is critical! Check with everyone and then stop the run and likely turn off the CAT!"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::CoreTemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::CPU,
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("cpu_disk", TofAlert {
                                                        key          : "cpu_disk",
                                                        descr        : "The disks on the TOF CPU are getting full.",
                                                        whattodo     : vec!["This is a general issue and need to be discussed in ops","No immediate action necessary"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::AvailableDiskSpace,
                                                        outofbound   : OutOfBound::TooHigh,
                                                        component    : Component::CPU,
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("cpu_hk_too_old" , TofAlert {
                                                        key          : "cpu_hk_too_old", 
                                                        descr        : "CPUMoniData is out-of-date!",
                                                        whattodo     : vec!["If everything else is fine, this is non-critical", "Typicallly, if this is an issue, there might be other, more severe issues!"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(40),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("data_mangling", TofAlert {
                                                        key          : "data_mangling",
                                                        descr        : "Data mangling (intermix of RB channels on one or several RBs) is excessive!",
                                                        whattodo     : vec!["This is a general issue!","No immediate action necessary","Options have to be discussed"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::DataMangling,
                                                        outofbound   : OutOfBound::TooHigh,
                                                        component    : Component::System,
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("miss_tofevid", TofAlert {
                                                        key          : "miss_tofevid",
                                                        descr        : "The Tof system is missing event ids in an excessive amount!",
                                                        whattodo     : vec!["This is nist likely related to ","No immediate action necessary","Options have to be discussed"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::DataMangling,
                                                        outofbound   : OutOfBound::TooHigh,
                                                        component    : Component::System,
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("tofev_vs_merge", TofAlert {
                                                        key          : "tofev_vs_merge",
                                                        descr        : "The rate of Tof event is larger thatn the rate of merged events!",
                                                        whattodo     : vec!["If you are getting the tof events from telemetry, this simply can't happen ","If this happens, something is utterly broken, most likely with our monitoring","In case we don't get TofEventSummary from telemetry but directlly from the TofCPU, we are on ground and this simply needs to be debugged"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::Telemetry,
                                                        outofbound   : OutOfBound::TooHigh,
                                                        component    : Component::System,
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("notof_vs_merge", TofAlert {
                                                        key          : "notof_vs_merge",
                                                        descr        : "The rate of merged events without TofEvents is excessive!",
                                                        whattodo     : vec!["The stream of TofEvents to the flight computer might have stopped","1) Run restart recommended","2) Soft Reboot MTB", "3) Hard reboot CAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::Telemetry,
                                                        outofbound   : OutOfBound::TooHigh,
                                                        component    : Component::System,
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("frac_interest", TofAlert {
                                                        key          : "frac_interest",
                                                        descr        : "The rate of interesting events is too low!",
                                                        whattodo     : vec!["The stream of TofEvents to the flight computer might have stopped","1) Run restart recommended","2) Soft Reboot MTB", "3) Hard reboot CAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::Telemetry,
                                                        outofbound   : OutOfBound::TooHigh,
                                                        component    : Component::System,
                                                        n_paged      : 0,
                                                        triggered    : None}),

                                    // 40x 3 RB alerts. I am only doing this to celebrate me being
                                    // miserable on basically ny's eve. May the gods have mercy and 
                                    // strike me down.
                                    ("rb01_rate_zero" , TofAlert {
                                                        key          : "rb01_rate_zero", 
                                                        descr        : "RB01 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(1),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb01_temp" , TofAlert {
                                                        key          : "rb01_temp", 
                                                        descr        : "RB01 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(1),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb01_hk_too_old" , TofAlert {
                                                        key          : "rb01_hk_too_old", 
                                                        descr        : "RB01 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(1),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    /////////////////////////////////////////////////////////
                                    ("rb02_rate_zero" , TofAlert {
                                                        key          : "rb02_rate_zero", 
                                                        descr        : "RB02 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(2),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb02_temp" , TofAlert {
                                                        key          : "rb02_temp", 
                                                        descr        : "RB02 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(2),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb02_hk_too_old" , TofAlert {
                                                        key          : "rb02_hk_too_old", 
                                                        descr        : "RB02 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        n_paged      : 0,
                                                        component    : Component::RB(2),
                                                        triggered    : None}),
                                    ("rb03_rate_zero" , TofAlert {
                                                        key          : "rb03_rate_zero", 
                                                        descr        : "RB03 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        n_paged      : 0,
                                                        component    : Component::RB(3),
                                                        triggered    : None}),
                                    ("rb03_temp" , TofAlert {
                                                        key          : "rb03_temp", 
                                                        descr        : "RB03 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(3),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb03_hk_too_old" , TofAlert {
                                                        key          : "rb03_hk_too_old", 
                                                        descr        : "RB03 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(3),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb04_rate_zero" , TofAlert {
                                                        key          : "rb04_rate_zero", 
                                                        descr        : "RB04 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(4),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb04_temp" , TofAlert {
                                                        key          : "rb04_temp", 
                                                        descr        : "RB04 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(4),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb04_hk_too_old" , TofAlert {
                                                        key          : "rb05_hk_too_old", 
                                                        descr        : "RB05 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(4),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb05_rate_zero" , TofAlert {
                                                        key          : "rb05_rate_zero", 
                                                        descr        : "RB05 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        n_paged      : 0,
                                                        component    : Component::RB(5),
                                                        triggered    : None}),
                                    ("rb05_temp" , TofAlert {
                                                        key          : "rb05_temp", 
                                                        descr        : "RB01 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(5),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb05_hk_too_old" , TofAlert {
                                                        key          : "rb05_hk_too_old", 
                                                        descr        : "RB01 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(5),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb06_rate_zero" , TofAlert {
                                                        key          : "rb06_rate_zero", 
                                                        descr        : "RB06 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(6),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb06_temp" , TofAlert {
                                                        key          : "rb06_temp", 
                                                        descr        : "RB06 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(6),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb06_hk_too_old" , TofAlert {
                                                        key          : "rb06_hk_too_old", 
                                                        descr        : "RB06 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(6),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb07_rate_zero" , TofAlert {
                                                        key          : "rb07_rate_zero", 
                                                        descr        : "RB07 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(7),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb07_temp" , TofAlert {
                                                        key          : "rb07_temp", 
                                                        descr        : "RB07 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(7),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb07_hk_too_old" , TofAlert {
                                                        key          : "rb07_hk_too_old", 
                                                        descr        : "RB07 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(7),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb08_rate_zero" , TofAlert {
                                                        key          : "rb08_rate_zero", 
                                                        descr        : "RB08 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(8),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb08_temp" , TofAlert {
                                                        key          : "rb08_temp", 
                                                        descr        : "RB08 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(8),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb08_hk_too_old" , TofAlert {
                                                        key          : "rb08_hk_too_old", 
                                                        descr        : "RB08 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(8),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb09_rate_zero" , TofAlert {
                                                        key          : "rb09_rate_zero", 
                                                        descr        : "RB09 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(9),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb09_temp" , TofAlert {
                                                        key          : "rb09_temp", 
                                                        descr        : "RB09 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(9),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb09_hk_too_old" , TofAlert {
                                                        key          : "rb09_hk_too_old", 
                                                        descr        : "RB09 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(9),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb11_rate_zero" , TofAlert {
                                                        key          : "rb11_rate_zero", 
                                                        descr        : "RB11 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(11),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb11_temp" , TofAlert {
                                                        key          : "rb11_temp", 
                                                        descr        : "RB11 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(11),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb11_hk_too_old" , TofAlert {
                                                        key          : "rb11_hk_too_old", 
                                                        descr        : "RB11 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(11),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    /////////////////////////////////////////////////////////
                                    ("rb13_rate_zero" , TofAlert {
                                                        key          : "rb13_rate_zero", 
                                                        descr        : "RB13 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(13),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb13_temp" , TofAlert {
                                                        key          : "rb13_temp", 
                                                        descr        : "RB13 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(13),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb13_hk_too_old" , TofAlert {
                                                        key          : "rb13_hk_too_old", 
                                                        descr        : "RB13 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(13),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb14_rate_zero" , TofAlert {
                                                        key          : "rb14_rate_zero", 
                                                        descr        : "RB14 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(14),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb14_temp" , TofAlert {
                                                        key          : "rb14_temp", 
                                                        descr        : "RB14 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(14),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb14_hk_too_old" , TofAlert {
                                                        key          : "rb14_hk_too_old", 
                                                        descr        : "RB14 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(14),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb15_rate_zero" , TofAlert {
                                                        key          : "rb15_rate_zero", 
                                                        descr        : "RB15 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(15),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb15_temp" , TofAlert {
                                                        key          : "rb15_temp", 
                                                        descr        : "RB15 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(15),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb15_hk_too_old" , TofAlert {
                                                        key          : "rb15_hk_too_old", 
                                                        descr        : "RB15 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(15),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb16_rate_zero" , TofAlert {
                                                        key          : "rb16_rate_zero", 
                                                        descr        : "RB16 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(16),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb16_temp" , TofAlert {
                                                        key          : "rb16_temp", 
                                                        descr        : "RB16 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(16),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb16_hk_too_old" , TofAlert {
                                                        key          : "rb16_hk_too_old", 
                                                        descr        : "RB16 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(16),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb17_rate_zero" , TofAlert {
                                                        key          : "rb17_rate_zero", 
                                                        descr        : "RB17 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(17),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb17_temp" , TofAlert {
                                                        key          : "rb17_temp", 
                                                        descr        : "RB17 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(17),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb17_hk_too_old" , TofAlert {
                                                        key          : "rb17_hk_too_old", 
                                                        descr        : "RB17 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(17),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb18_rate_zero" , TofAlert {
                                                        key          : "rb18_rate_zero", 
                                                        descr        : "RB18 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(18),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb18_temp" , TofAlert {
                                                        key          : "rb18_temp", 
                                                        descr        : "RB18 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(18),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb18_hk_too_old" , TofAlert {
                                                        key          : "rb18_hk_too_old", 
                                                        descr        : "RB18 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(18),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb19_rate_zero" , TofAlert {
                                                        key          : "rb19_rate_zero", 
                                                        descr        : "RB19 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(19),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb19_temp" , TofAlert {
                                                        key          : "rb19_temp", 
                                                        descr        : "RB19 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(19),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb19_hk_too_old" , TofAlert {
                                                        key          : "rb19_hk_too_old", 
                                                        descr        : "RB19 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(19),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb20_rate_zero" , TofAlert {
                                                        key          : "rb20_rate_zero", 
                                                        descr        : "RB20 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(20),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb20_temp" , TofAlert {
                                                        key          : "rb20_temp", 
                                                        descr        : "RB20 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(20),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb20_hk_too_old" , TofAlert {
                                                        key          : "rb20_hk_too_old", 
                                                        descr        : "RB20 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(20),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb21_rate_zero" , TofAlert {
                                                        key          : "rb21_rate_zero", 
                                                        descr        : "RB21 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(21),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb21_temp" , TofAlert {
                                                        key          : "rb21_temp", 
                                                        descr        : "RB21 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(21),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb21_hk_too_old" , TofAlert {
                                                        key          : "rb21_hk_too_old", 
                                                        descr        : "RB21 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(21),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ////////////////////////////////////////////////////////
                                    ("rb22_rate_zero" , TofAlert {
                                                        key          : "rb22_rate_zero", 
                                                        descr        : "RB22 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(22),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb22_temp" , TofAlert {
                                                        key          : "rb22_temp", 
                                                        descr        : "RB22 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(22),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb22_hk_too_old" , TofAlert {
                                                        key          : "rb22_hk_too_old", 
                                                        descr        : "RB22 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(22),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb23_rate_zero" , TofAlert {
                                                        key          : "rb23_rate_zero", 
                                                        descr        : "RB23 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(23),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb23_temp" , TofAlert {
                                                        key          : "rb23_temp", 
                                                        descr        : "RB23 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(23),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb23_hk_too_old" , TofAlert {
                                                        key          : "rb23_hk_too_old", 
                                                        descr        : "RB23 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(23),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb24_rate_zero" , TofAlert {
                                                        key          : "rb24_rate_zero", 
                                                        descr        : "RB24 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(24),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb24_temp" , TofAlert {
                                                        key          : "rb24_temp", 
                                                        descr        : "RB24 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(24),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb24_hk_too_old" , TofAlert {
                                                        key          : "rb24_hk_too_old", 
                                                        descr        : "RB24 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(24),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb25_rate_zero" , TofAlert {
                                                        key          : "rb25_rate_zero", 
                                                        descr        : "RB25 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(25),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb25_temp" , TofAlert {
                                                        key          : "rb25_temp", 
                                                        descr        : "RB25 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(25),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb25_hk_too_old" , TofAlert {
                                                        key          : "rb25_hk_too_old", 
                                                        descr        : "RB25 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(25),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb26_rate_zero" , TofAlert {
                                                        key          : "rb26_rate_zero", 
                                                        descr        : "RB26 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(26),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb26_temp" , TofAlert {
                                                        key          : "rb26_temp", 
                                                        descr        : "RB26 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(26),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb26_hk_too_old" , TofAlert {
                                                        key          : "rb26_hk_too_old", 
                                                        descr        : "RB26 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(26),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb27_rate_zero" , TofAlert {
                                                        key          : "rb27_rate_zero", 
                                                        descr        : "RB27 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(27),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb27_temp" , TofAlert {
                                                        key          : "rb27_temp", 
                                                        descr        : "RB27 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(27),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb27_hk_too_old" , TofAlert {
                                                        key          : "rb27_hk_too_old", 
                                                        descr        : "RB27 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(27),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb28_rate_zero" , TofAlert {
                                                        key          : "rb28_rate_zero", 
                                                        descr        : "RB28 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(28),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb28_temp" , TofAlert {
                                                        key          : "rb28_temp", 
                                                        descr        : "RB28 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(28),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb28_hk_too_old" , TofAlert {
                                                        key          : "rb28_hk_too_old", 
                                                        descr        : "RB28 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(28),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb29_rate_zero" , TofAlert {
                                                        key          : "rb29_rate_zero", 
                                                        descr        : "RB29 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(29),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb29_temp" , TofAlert {
                                                        key          : "rb29_temp", 
                                                        descr        : "RB29 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(29),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb29_hk_too_old" , TofAlert {
                                                        key          : "rb29_hk_too_old", 
                                                        descr        : "RB29 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(29),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ////////////////////////////////////////////////////////
                                    ("rb30_rate_zero" , TofAlert {
                                                        key          : "rb30_rate_zero", 
                                                        descr        : "RB30 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(30),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb30_temp" , TofAlert {
                                                        key          : "rb30_temp", 
                                                        descr        : "RB30 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(30),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb30_hk_too_old" , TofAlert {
                                                        key          : "rb30_hk_too_old", 
                                                        descr        : "RB30 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(30),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb31_rate_zero" , TofAlert {
                                                        key          : "rb31_rate_zero", 
                                                        descr        : "RB31 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(31),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb31_temp" , TofAlert {
                                                        key          : "rb31_temp", 
                                                        descr        : "RB31 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(31),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb31_hk_too_old" , TofAlert {
                                                        key          : "rb31_hk_too_old", 
                                                        descr        : "RB31 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(31),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb32_rate_zero" , TofAlert {
                                                        key          : "rb32_rate_zero", 
                                                        descr        : "RB32 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(32),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb32_temp" , TofAlert {
                                                        key          : "rb32_temp", 
                                                        descr        : "RB32 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(32),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb32_hk_too_old" , TofAlert {
                                                        key          : "rb32_hk_too_old", 
                                                        descr        : "RB30 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(32),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb33_rate_zero" , TofAlert {
                                                        key          : "rb33_rate_zero", 
                                                        descr        : "RB33 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(33),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb33_temp" , TofAlert {
                                                        key          : "rb33_temp", 
                                                        descr        : "RB33 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(33),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb33_hk_too_old" , TofAlert {
                                                        key          : "rb33_hk_too_old", 
                                                        descr        : "RB33 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(33),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb34_rate_zero" , TofAlert {
                                                        key          : "rb34_rate_zero", 
                                                        descr        : "RB34 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(34),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb34_temp" , TofAlert {
                                                        key          : "rb34_temp", 
                                                        descr        : "RB34 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(34),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb34_hk_too_old" , TofAlert {
                                                        key          : "rb34_hk_too_old", 
                                                        descr        : "RB34 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(34),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb35_rate_zero" , TofAlert {
                                                        key          : "rb35_rate_zero", 
                                                        descr        : "RB35 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(35),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb35_temp" , TofAlert {
                                                        key          : "rb35_temp", 
                                                        descr        : "RB35 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(35),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb35_hk_too_old" , TofAlert {
                                                        key          : "rb35_hk_too_old", 
                                                        descr        : "RB35 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(35),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb36_rate_zero" , TofAlert {
                                                        key          : "rb36_rate_zero", 
                                                        descr        : "RB36 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(36),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb36_temp" , TofAlert {
                                                        key          : "rb36_temp", 
                                                        descr        : "RB36 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(36),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb36_hk_too_old" , TofAlert {
                                                        key          : "rb36_hk_too_old", 
                                                        descr        : "RB36 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(36),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb39_rate_zero" , TofAlert {
                                                        key          : "rb39_rate_zero", 
                                                        descr        : "RB39 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(39),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb39_temp" , TofAlert {
                                                        key          : "rb39_temp", 
                                                        descr        : "RB39 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(39),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb39_hk_too_old" , TofAlert {
                                                        key          : "rb39_hk_too_old", 
                                                        descr        : "RB39 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(39),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                ///////////////////////////////////////////////////////////
                                    ("rb40_rate_zero" , TofAlert {
                                                        key          : "rb40_rate_zero", 
                                                        descr        : "RB40 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(40),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb40_temp" , TofAlert {
                                                        key          : "rb40_temp", 
                                                        descr        : "RB40 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(40),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb40_hk_too_old" , TofAlert {
                                                        key          : "rb40_hk_too_old", 
                                                        descr        : "RB40 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(40),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb41_rate_zero" , TofAlert {
                                                        key          : "rb41_rate_zero", 
                                                        descr        : "RB41 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(41),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb41_temp" , TofAlert {
                                                        key          : "rb41_temp", 
                                                        descr        : "RB41 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(41),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb41_hk_too_old" , TofAlert {
                                                        key          : "rb41_hk_too_old", 
                                                        descr        : "RB41 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(41),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb42_rate_zero" , TofAlert {
                                                        key          : "rb42_rate_zero", 
                                                        descr        : "RB42 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(42),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb42_temp" , TofAlert {
                                                        key          : "rb42_temp", 
                                                        descr        : "RB42 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(42),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb42_hk_too_old" , TofAlert {
                                                        key          : "rb42_hk_too_old", 
                                                        descr        : "RB42 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(42),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb44_rate_zero" , TofAlert {
                                                        key          : "rb44_rate_zero", 
                                                        descr        : "RB44 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(44),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb44_temp" , TofAlert {
                                                        key          : "rb44_temp", 
                                                        descr        : "RB44 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(44),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb44_hk_too_old" , TofAlert {
                                                        key          : "rb44_hk_too_old", 
                                                        descr        : "RB44 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(44),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb46_rate_zero" , TofAlert {
                                                        key          : "rb46_rate_zero", 
                                                        descr        : "RB46 is not triggering!", 
                                                        whattodo     : vec!["If we are getting HK data, but the RAT is not triggering, this indicated an issue with either the RAT (the other RB might be down as well or the MTB", "If the other RB in the RAT is working, try soft rebooting the MTB", "Hard reboot the MTB"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::TriggerRate,
                                                        outofbound   : OutOfBound::Zero,
                                                        component    : Component::RB(46),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb46_temp" , TofAlert {
                                                        key          : "rb46_temp", 
                                                        descr        : "RB46 (FPGA) temperature is out of bounds!",
                                                        whattodo     : vec!["This is critical!" ,  "Check with everybody and then likeliy switch off the RAT"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::FPGATemp,
                                                        outofbound   : OutOfBound::TooLowOrTooHigh,
                                                        component    : Component::RB(46),
                                                        n_paged      : 0,
                                                        triggered    : None}),
                                    ("rb46_hk_too_old" , TofAlert {
                                                        key          : "rb46_hk_too_old", 
                                                        descr        : "RB46 RBMoniData is out-of-date!",
                                                        whattodo     : vec!["Maybe the RB is down?" ,  "Telemetry issues?", "Have an eye on it and take notes"],
                                                        config       : TofAlertConfig::new(),
                                                        variable     : Variable::MoniData,
                                                        outofbound   : OutOfBound::TooOld,
                                                        component    : Component::RB(46),
                                                        n_paged      : 0,
                                                        triggered    : None}),

  ]);
   // rb ids
   // 1,2,3,4,5,6,7,8,9,11,13,14,15,16,17,18,19,
   // 20,21,22,23,24,25,26,27,28,29,
   // 30,31,32,33,34,35,36,
   // 39,40,41,44,46

//  TofAlert {
//    key          : "rb_rate_zero",
//    descr        : "RB1 is not triggering!",
//    whattodo     : vec!["1) Run restart", "2) Restart liftof-rb", "3) Soft reboot RB", "4) Soft reboot MTB", "5) Hard reboot RAT",  "6) Hard reboot MTB"],
//    config       : TofAlertConfig::new(),
//    variable     : Variable::TriggerRate,
//    outofbound   : OutOfBound::Zero,
//    component    : Component::RB(rb_id),
//    acknowledged : false,
//    created      : None,
//  }
  //let mut alerts = HashMap::<&str, TofAlert>::new();
  for k in manifest.keys() {
    println!("{}",k);
    alerts.get_mut(k).unwrap().config = manifest.get(k).unwrap().clone();
  }
  alerts

}

#[test]
fn write_alert_manifest() {
  let settings = TofAlertManifest::new();
  //println!("{}", settings);
  settings.to_toml(String::from("alert-manifest-test.toml"));
}

#[test]
fn read_alert_manifest() {
  let _settings = TofAlertManifest::from_toml("alert-manifest-test.toml");
}
