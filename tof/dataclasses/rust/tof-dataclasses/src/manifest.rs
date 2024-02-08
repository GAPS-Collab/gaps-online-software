//! Entities of the tof, that is LocalTriggerBoard, ReadoutBoard, PB, MTB...
//! 
//! This provides structs to hold their metainformation,
//! methods to save/load them from the gaps database as 
//! well as to serialize them locally.

use std::fmt;

cfg_if::cfg_if! {
  if #[cfg(feature = "database")]  {
    use std::path::Path;
    use std::str::FromStr;
    use std::collections::HashMap;
    extern crate sqlite;
  }
}

use regex::Regex;
use glob::glob;
use chrono::{NaiveDateTime, Utc};

use crate::calibrations::RBCalibrations;
use crate::DsiLtbRBMapping;

#[derive(Copy, Clone, Debug)]
pub struct LocalTriggerBoard {
  pub ltb_id        : u8, 
  pub ltb_dsi       : u8, 
  pub ltb_j         : u8,
  // rb id for ltb channel
  pub ltb_ch_rb_id  : [u8;16],
  // rb channel for ltb channel
  pub ltb_ch_rb_ch  : [u8;16]
}

impl LocalTriggerBoard {
  pub fn new() -> Self {
    Self {
      ltb_id         : 0,
      ltb_dsi        : 0,
      ltb_j          : 0,
      ltb_ch_rb_id   : [0;16],
      ltb_ch_rb_ch   : [0;16]
    }
  }
 
  /// Calculate the position in the bitmask from the connectors
  pub fn get_mask_from_dsi_and_j(&self) -> u32 {
    if self.ltb_dsi == 0 || self.ltb_j == 0 { 
      warn!("Invalid dsi/J connection!");
      return 0;
    }   
    let mut mask : u32 = 1;
    mask = mask << ((self.ltb_dsi - 1)*5 + self.ltb_j -1) ;
    mask
  }

  pub fn get_rb_id(&self, chn : u8) -> u8 {
    self.ltb_ch_rb_id[chn as usize -1]
  }

  pub fn set_rb_id(&mut self, chn : u8, rb_id : u8) {
    self.ltb_ch_rb_id[chn as usize -1] = rb_id;
  }
  
  pub fn get_rb_ch(&self, chn : u8) -> u8 {
    self.ltb_ch_rb_ch[chn as usize -1]
  }

  pub fn set_rb_ch(&mut self, chn : u8, rb_ch : u8) {
    self.ltb_ch_rb_ch[chn as usize -1] = rb_ch;
  }
}

impl Default for LocalTriggerBoard {
  fn default() -> Self {
      Self::new()
  }
}

impl fmt::Display for LocalTriggerBoard {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, 
"<LocalTriggerBoard:
   ID  : {}
   DSI : {}
   J   : {}
   CH/RBID MAP {:?}
   CH/RBCH_MAP {:?}>",
           self.ltb_id,
           self.ltb_dsi,
           self.ltb_j,
           self.ltb_ch_rb_id,
           self.ltb_ch_rb_ch)
  }
}

#[cfg(feature = "database")]
pub fn get_rbs_from_sqlite(filename : &Path) -> Vec<ReadoutBoard> {
  let connection = sqlite::open(filename).unwrap();
  let query = "SELECT * FROM tof_db_rb";
  let mut rbs  = Vec::<ReadoutBoard>::new();
  match connection.iterate(query, |pairs| {
    debug!("New rb, has following values...");
    //let mut ltb = LocalTriggerBoard::new();
    let mut rb = ReadoutBoard::new();
    for &(name, value) in pairs.iter() {
      match value {
        None    => {continue;},
        Some(v) => {
          println!("{} = {}", name, v);
          match name {
            "rb_id"      => {rb.rb_id  = u8::from_str(v).unwrap_or(0);},
            "ch1_paddle_id" => {rb.set_paddle_end_id_for_rb_channel(1, u16::from_str(v).unwrap_or(0));},
            "ch2_paddle_id" => {rb.set_paddle_end_id_for_rb_channel(2, u16::from_str(v).unwrap_or(0));},
            "ch3_paddle_id" => {rb.set_paddle_end_id_for_rb_channel(3, u16::from_str(v).unwrap_or(0));},
            "ch4_paddle_id" => {rb.set_paddle_end_id_for_rb_channel(4, u16::from_str(v).unwrap_or(0));},
            "ch5_paddle_id" => {rb.set_paddle_end_id_for_rb_channel(5, u16::from_str(v).unwrap_or(0));},
            "ch6_paddle_id" => {rb.set_paddle_end_id_for_rb_channel(6, u16::from_str(v).unwrap_or(0));},
            "ch7_paddle_id" => {rb.set_paddle_end_id_for_rb_channel(7, u16::from_str(v).unwrap_or(0));},
            "ch8_paddle_id" => {rb.set_paddle_end_id_for_rb_channel(8, u16::from_str(v).unwrap_or(0));},
            _ => {warn!("Found name {}, but not mapping it to self!", name);}                         
          }
        }
      }
    } // end loop over rbs
    rbs.push(rb);
    true
  }) {
    Err(err) => {
      error!("Unable to query DB! Error {err}");
    },
    Ok(_) => {
      debug!("DB query successful!");
    }
  }
  info!("We found {} rbs in the database", rbs.len()); 
  rbs
}


#[cfg(feature = "database")]
pub fn get_ltbs_from_sqlite(filename : &Path) -> Vec<LocalTriggerBoard> {
  let connection = sqlite::open(filename).unwrap();
  let query = "SELECT * FROM tof_db_ltb";
  let mut ltbs  = Vec::<LocalTriggerBoard>::new();
  match connection.iterate(query, |pairs| {
    debug!("New ltb, has following values...");
    let mut ltb = LocalTriggerBoard::new();
    for &(name, value) in pairs.iter() {
      debug!("{} = {}", name, value.unwrap_or(""));
      match value {
        None    => {continue;},
        Some(v) => {
          match name {
            "ltb_id"      => {ltb.ltb_id       = u8::from_str(v).unwrap_or(0);},
            "ltb_dsi"     => {ltb.ltb_dsi      = u8::from_str(v).unwrap_or(0);},
            "ltb_j"       => {ltb.ltb_j        = u8::from_str(v).unwrap_or(0);},
            "ltb_ch1_rb"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_id(1, rb_id);
            },
            "ltb_ch2_rb"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_id(2, rb_id);
            },
            "ltb_ch3_rb"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_id(3, rb_id);
            },
            "ltb_ch4_rb"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_id(4, rb_id);
            },
            "ltb_ch5_rb"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_id(5, rb_id);
            },
            "ltb_ch6_rb"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_id(6, rb_id);
            },
            "ltb_ch7_rb"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_id(7, rb_id);
            },
            "ltb_ch8_rb"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_id(8, rb_id);
            },
            "ltb_ch9_rb"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_id(9, rb_id);
            },
            "ltb_ch10_rb"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_id(10, rb_id);
            },
            "ltb_ch11_rb"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_id(11, rb_id);
            },
            "ltb_ch12_rb"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_id(12, rb_id);
            },
            "ltb_ch13_rb"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_id(13, rb_id);
            },
            "ltb_ch14_rb"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_id(14, rb_id);
            },
            "ltb_ch15_rb"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_id(15, rb_id);
            },
            "ltb_ch16_rb"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_id(16, rb_id);
            },
            "ltb_ch1_rb_ch"  => {
              let rb_ch = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_ch(1, rb_ch);
            },
            "ltb_ch2_rb_ch"  => {
              let rb_ch = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_ch(2, rb_ch);
            },
            "ltb_ch3_rb_ch"  => {
              let rb_ch = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_ch(3, rb_ch);
            },
            "ltb_ch4_rb_ch"  => {
              let rb_ch = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_ch(4, rb_ch);
            },
            "ltb_ch5_rb_ch"  => {
              let rb_ch = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_ch(5, rb_ch);
            },
            "ltb_ch6_rb_ch"  => {
              let rb_ch = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_ch(6, rb_ch);
            },
            "ltb_ch7_rb_ch"  => {
              let rb_ch = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_ch(7, rb_ch);
            },
            "ltb_ch8_rb_ch"  => {
              let rb_ch = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_ch(8, rb_ch);
            },
            "ltb_ch9_rb_ch"  => {
              let rb_ch = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_ch(9, rb_ch);
            },
            "ltb_ch10_rb_ch"  => {
              let rb_id = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_ch(10, rb_id);
            },
            "ltb_ch11_rb_ch"  => {
              let rb_ch = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_ch(11, rb_ch);
            },
            "ltb_ch12_rb_ch"  => {
              let rb_ch = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_ch(12, rb_ch);
            },
            "ltb_ch13_rb_ch"  => {
              let rb_ch = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_ch(13, rb_ch);
            },
            "ltb_ch14_rb_ch"  => {
              let rb_ch = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_ch(14, rb_ch);
            },
            "ltb_ch15_rb_ch"  => {
              let rb_ch = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_ch(15, rb_ch);
            },
            "ltb_ch16_rb_ch"  => {
              let rb_ch = u8::from_str(v).unwrap_or(0);
              ltb.set_rb_ch(16, rb_ch);
            },
            _ => {debug!("Found name {}", name);}                         
          }
        }
      }
    } // end loop over ltbs
    //println!("{}", ltb);
    ltbs.push(ltb);
    true
  }) {
    Err(err) => {
      error!("Unable to query DB! Error {err}");
    },
    Ok(_)    => {
      debug!("DB query successful!");
    }
  }
  info!("We found {} ltbs in the database!", ltbs.len()); 
  ltbs
}

///////////////////////////////////////////////////////////
#[cfg(feature = "database")]
pub fn get_dsi_from_sqlite(filename : &Path) -> Vec<DSICard> {
  let connection    = sqlite::open(filename).expect("Unable to open DB file!");
  let query         = "SELECT * FROM tof_db_dsicard";
  let mut dsi_cards = Vec::<DSICard>::new();
  match connection.iterate(query, |pairs| {
    let mut dsi  = DSICard::new();
    for &(name, value) in pairs.iter() {
      match value {
        None    => {continue;},
        Some(v) => {
          //println!("{} = {}", name, v);
          match name {
            "dsi_id"         => {dsi.dsi_id    = u8::from_str(v).unwrap_or(0);},
            "j1_rat_id"      => {dsi.j1_rat_id = u8::from_str(v).unwrap_or(0);},
            "j2_rat_id"      => {dsi.j2_rat_id = u8::from_str(v).unwrap_or(0);},
            "j3_rat_id"      => {dsi.j3_rat_id = u8::from_str(v).unwrap_or(0);},
            "j4_rat_id"      => {dsi.j4_rat_id = u8::from_str(v).unwrap_or(0);},
            "j5_rat_id"      => {dsi.j5_rat_id = u8::from_str(v).unwrap_or(0);},
            _                => () 
          }
        }
      }
      dsi_cards.push(dsi);
    }
    true
  }) {
    Err(err) => {
      error!("Unable to query DB for DSILtbRBMap! {err}");
    },
    Ok(_) => {
      debug!("DB query successful!");
    }
  }
  dsi_cards
}

/// Dsi -> J -> (RBID,RBCH)
#[cfg(feature = "database")]
pub fn get_dsi_j_ltbch_vs_rbch_map(filename : &Path) -> DsiLtbRBMapping {
  let mut map = DsiLtbRBMapping::new();
  let ltbs    = get_ltbs_from_sqlite(filename);
  for dsi in 1..6 {
    let mut jmap = HashMap::<u8, HashMap<u8, (u8, u8)>>::new();
    for j in 1..6 {
      let mut rbidch_map : HashMap<u8, (u8,u8)> = HashMap::new();
      for ch in 1..17 {
        let rbidch = (0,0);
        rbidch_map.insert(ch,rbidch);
        //map[dsi] = 
      }
      jmap.insert(j,rbidch_map);
    }
    map.insert(dsi,jmap);
  }
  for ltb in ltbs {
    for ch in 1..17 {
      let rb_id = ltb.get_rb_id(ch as u8);
      let rb_ch = ltb.get_rb_ch(ch as u8);
      *map.get_mut(&ltb.ltb_dsi).unwrap().get_mut(&ltb.ltb_j).unwrap().get_mut(&ch).unwrap() = (rb_id, rb_ch);
      //map[&ltb.ltb_dsi][&ltb.ltb_j].insert((rb_id, rb_ch);
    }
  }
  map
}
//---------------------------------------------------------

/// This represents an entire TOF panel
/// (an assembly of paddles)
pub struct Panel {
    panel_id                : u8, 
    smallest_paddle_id      : u8, 
    n_paddles               : u8, 
}

impl Panel {
  pub fn new() -> Self {
    Self {
      panel_id           : 0,
      smallest_paddle_id : 0,
      n_paddles          : 0
    }
  }
}

impl Default for Panel {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for Panel {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<PANEL: ID {}; SMALLEST PADDLE ID {}; N PADDLES {}>", self.panel_id, self.smallest_paddle_id, self.n_paddles)
  }
}

//---------------------------------------------------------

/// A represnetation of a TOF paddle
pub struct Paddle {
  pub paddle_id    : u8 ,
  pub volume_id    : u32,
  pub height       : f32,
  pub width        : f32,
  pub length       : f32,
  pub global_pos_x : f32,
  pub global_pos_y : f32,
  pub global_pos_z : f32,
}

impl Paddle {

  pub fn new() -> Self {
    Self {
      paddle_id      : 0,
      volume_id      : 0,
      height         : 0.0,
      width          : 0.0,
      length         : 0.0,
      global_pos_x   : 0.0,
      global_pos_y   : 0.0,
      global_pos_z   : 0.0,
    }
  }
}

impl Default for Paddle {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for Paddle {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<PADDLE: ID {}; HEIGHT {}; WIDTH {} LENGTH {}>", self.paddle_id, self.height, self.width, self.length)
  }
}

//---------------------------------------------------------

#[derive(Debug, Copy, Clone)]
pub enum PaddleEndIdentifier {
  A,
  B
}

//impl fmt::Display for PaddleEndIdentifier {
//  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//    match self {
//      PaddleEndIdentifier::A => {write!(f, "A")}
//      PaddleEndIdentifier::B => {write!(f, "B")}
//  }
//}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum PaddleEndLocation {
  PositiveX,
  NegativeX,
  PositiveY,
  NegativeY,
  PositiveZ,
  NegativeZ
}

pub struct PaddleEnd {
  pub paddle_id     : u8,
  pub paddle_end_id : u16,

  pub end           : PaddleEndIdentifier, 
  pub end_location  : PaddleEndLocation, 
  pub panel_id      : u8, 
  pub cable_length  : f32, 
  pub rat           : u8, 
  pub ltb_id        : u8,
  pub rb_id         : u8,
  pub pb_id         : u8,
  pub ltb_ch        : u8,
  pub pb_ch         : u8,
  pub rb_ch         : u8,
  pub dsi           : u8,
  pub rb_harting_j  : u8,
  pub ltb_harting_j : u8,
}

impl PaddleEnd {
  pub fn new(paddle_id : u8, end : PaddleEndIdentifier, loc : PaddleEndLocation) 
    -> Self {
    let mut pe = Self {
      paddle_id     : paddle_id,
      paddle_end_id : 0,
      end           : end, 
      end_location  : loc,
      panel_id      : 0, 
      cable_length  : 0.0, 
      rat           : 0, 
      ltb_id        : 0,
      rb_id         : 0,
      pb_id         : 0,
      ltb_ch        : 0,
      pb_ch         : 0,
      rb_ch         : 0,
      dsi           : 0,
      rb_harting_j  : 0,
      ltb_harting_j : 0, 
    };
    pe.construct_paddle_id();
    pe
  }

  pub fn construct_paddle_id(&mut self) {
    match self.end {
      PaddleEndIdentifier::A => {
        self.paddle_end_id = 1000 + self.paddle_id as u16;
      }
      PaddleEndIdentifier::B => {
        self.paddle_end_id = 2000 + self.paddle_id as u16;
      }
    }
  }
}

impl Default for PaddleEnd {
  fn default() -> Self {
    Self::new(0,PaddleEndIdentifier::A, PaddleEndLocation::PositiveX)
  }
}

impl fmt::Display for PaddleEnd {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    
    write!(f, "<PaddleEnd: PADDLE ID {}, END {:?}>", self.paddle_id, self.end)
  }
}

//---------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DSICard {
  pub dsi_id     : u8, 
  pub j1_rat_id  : u8, 
  pub j2_rat_id  : u8, 
  pub j3_rat_id  : u8, 
  pub j4_rat_id  : u8, 
  pub j5_rat_id  : u8, 
}

impl DSICard {
  pub fn new() -> DSICard {
    DSICard {
      dsi_id     : 0, 
      j1_rat_id  : 0, 
      j2_rat_id  : 0, 
      j3_rat_id  : 0, 
      j4_rat_id  : 0, 
      j5_rat_id  : 0, 
    }
  }

  pub fn get_rat_id_for_j(&self, j : u8) -> u8 {
    let rat_id : u8;
    match j {
      1 => {rat_id = self.j1_rat_id;},
      2 => {rat_id = self.j2_rat_id;},
      3 => {rat_id = self.j3_rat_id;},
      4 => {rat_id = self.j4_rat_id;},
      5 => {rat_id = self.j5_rat_id;},
      _ => { 
        error!("No j > 5! Returning 0");
        rat_id = 0;
      }
    }
    rat_id
  }
}

impl Default for DSICard {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for DSICard {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
    let mut repr = String::from("<DSICard:");
    repr += &(format!("\n  DSI ID  : {}", self.dsi_id)   );
    repr += &(format!("\n  J1 RAT  : {}", self.j1_rat_id)); 
    repr += &(format!("\n  J2 RAT  : {}", self.j2_rat_id)); 
    repr += &(format!("\n  J3 RAT  : {}", self.j3_rat_id)); 
    repr += &(format!("\n  J4 RAT  : {}", self.j4_rat_id)); 
    repr += &(format!("\n  J5 RAT  : {}", self.j5_rat_id)); 
    repr += &String::from(">");
    write!(f, "{}", repr)
  }
}

//---------------------------------------------------------

pub struct RAT {
  pub rat_id                    : u8,
  pub pb_id                     : u8,
  pub rb1_id                    : u8,
  pub rb2_id                    : u8,
  pub ltb_id                    : u8,
  pub ltb_harting_cable_length  : u8,
}

impl RAT {
  pub fn new() -> Self {
    Self {  
      rat_id                    : 0,
      pb_id                     : 0,
      rb1_id                    : 0,
      rb2_id                    : 0,
      ltb_id                    : 0,
      ltb_harting_cable_length  : 0,
    }
  }
}

impl Default for RAT {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for RAT {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
    write!(f, "<RAT: ID {}>", self.rat_id)
  }
}

#[derive(Clone, Debug)]
pub struct ReadoutBoard {
  pub rb_id                : u8,  
  pub dna                  : u64, 
  channel_to_paddle_end_id : [u16;8],
  pub calib_file_path      : String,
  pub calibration          : RBCalibrations,       
  pub trig_ch_mask         : [bool;8],
}

impl ReadoutBoard {
  pub fn new() -> Self {
    Self {
      rb_id                    : 0,  
      dna                      : 0, 
      channel_to_paddle_end_id : [0;8],
      calib_file_path          : String::from(""),
      calibration              : RBCalibrations::new(0),
      trig_ch_mask             : [false;8],
    }
  }

  /// Load the newest calibration from the calibration file path
  pub fn load_latest_calibration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    //  files look like RB20_2024_01_26-08_15_54.cali.tof.gaps
    let re = Regex::new(r"(\d{4}_\d{2}_\d{2}-\d{2}_\d{2}_\d{2})")?;
    // Define your file pattern (e.g., "logs/*.log" for all .log files in the logs directory)
    let pattern = format!("{}/RB{:02}_*", self.calib_file_path, self.rb_id); // Adjust this pattern to your files' naming convention
    let mut newest_file = (String::from(""), NaiveDateTime::from_timestamp_opt(0, 0).unwrap());
    
    // Iterate over files that match the pattern
    let mut filename : String;
    for entry in glob(&pattern)? {
      if let Ok(path) = entry {
        // Get the filename as a string
        //let cpath = path.clone();
        match path.file_name() {
          None => continue,
          Some(fname) => {
              // the expect might be ok, since this is something done during initialization
              filename = fname.to_os_string().into_string().expect("Unwrapping filename failed!");
          }
        }
        if let Some(caps) = re.captures(&filename) {
          if let Some(timestamp_str) = caps.get(0).map(|m| m.as_str()) {
            println!("{}",timestamp_str);
            let timestamp = NaiveDateTime::parse_from_str(timestamp_str, "%Y_%m_%d-%H_%M_%S")?;
            if timestamp > newest_file.1 {
              newest_file.1 = timestamp;
              newest_file.0 = filename.clone();
            }
          }
        }
      }
    }
    
    if newest_file.0.is_empty() {
      error!("No matching calibration available for board {}!", self.rb_id);
    } else {
      let file_to_load = self.calib_file_path.clone() + "/" + &newest_file.0;
      println!("==> Loading calibration from file: {}", file_to_load);
      self.calibration = RBCalibrations::from_file(file_to_load)?;
      println!("==> Loaded calibration {}", self.calibration);
    }
    Ok(())
  }

  #[deprecated(note="Won't work with tmiestamped cali files!")]
  pub fn guess_calibration_filename(&self) -> String {
    let mut cali = String::from("");
    cali += "rb_";
    if self.rb_id < 10 {
      cali += "0";
    }
    cali += &self.rb_id.to_string();
    cali += ".cali.tof.gaps";
    cali
  }

  /// The address the RB is publishing packets on 
  ///
  /// There is NO GUARANTEE that this is the correct
  /// address!
  pub fn guess_address(&self) -> String {
    format!("tcp://10.0.1.1{:02}:42000", self.rb_id)
  }


  pub fn get_triggered_pids(&self) -> Vec<u8> {
    let mut pids = Vec::<u8>::new();
    for k in 0..8 {
      let pid = self.get_pid_for_ch(k);
      if self.trig_ch_mask[k] {
        if pids.contains(&pid) {
          continue;
        }
        pids.push(self.get_pid_for_ch(k));
      }
    }
    pids
  }

  pub fn get_calibration(&self) -> String {
    return self.calib_file_path.clone();
  }

  pub fn get_paddle_end(&self, ch : usize) 
    -> PaddleEndIdentifier {
    let end_id = self.get_paddle_end_id_for_rb_channel(ch);
    if end_id >= 2000 {
      return PaddleEndIdentifier::B;
    } else {
      return PaddleEndIdentifier::A;
    }
  }

  pub fn get_all_pids(&self) -> Vec<u8> {
    let mut pids = Vec::<u8>::new();
    for k in 1..9 {
      let pid = self.get_pid_for_ch(k);
      if pids.contains(&pid) {
        continue;
      }
      pids.push(self.get_pid_for_ch(k));
    }
    pids
  }

  /// Get the paddle id for the connected paddle on channel
  ///
  /// Arguments:
  ///
  /// * channel 1-8
  pub fn get_pid_for_ch(&self, channel : usize) -> u8 {
    if channel > 9 || channel == 0 {
      error!("Got invalid channel value! Returning rubbish");
      return 0;
    }
    let p_end_id = self.channel_to_paddle_end_id[channel -1];
    if p_end_id % 2000 > 0 {
      return (p_end_id - 2000) as u8;
    } else {
      return (p_end_id - 1000) as u8;
    }
  }

  pub fn get_p_end_for_ch(&self, channel : usize) -> PaddleEndIdentifier {
    let p_end_id = self.channel_to_paddle_end_id[channel -1];
    if p_end_id % 2000 > 0 {
      return PaddleEndIdentifier::B;
    } else {
      return PaddleEndIdentifier::A;
    } 
  }

  pub fn set_paddle_end_id_for_rb_channel(&mut self,channel : usize, paddle_end_id : u16) {
    self.channel_to_paddle_end_id[channel -1] = paddle_end_id;
  }


  pub fn get_paddle_end_id_for_rb_channel(&self,channel : usize) -> u16 {
    if channel > 9 || channel == 0 {
      error!("Got invalid channel value! Returning rubbish");
      return 0;
    } 
    return self.channel_to_paddle_end_id[channel -1]
  }
}

impl Default for ReadoutBoard {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for ReadoutBoard {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
    write!(f,
"<ReadoutBoard:
    ID                :  {}
    DNA               :  {} 
    calibration path  :  {}
    calibration       :  {}
    CHANNEL/PADDLE END:  {:?}
    TRIG_CH_MASK      :  {:?}>",
      self.rb_id,
      self.dna,
      self.calib_file_path,
      self.calibration,
      self.channel_to_paddle_end_id,
      self.trig_ch_mask)
  }
}

