//! Entities of the tof, that is LocalTriggerBoard, ReadoutBoard, PB, MTB...
//! 
//! This provides structs to hold their metainformation,
//! methods to save/load them from the gaps database as 
//! well as to serialize them locally.

use std::fmt;
use std::str::FromStr;
use std::net::Ipv4Addr;
use std::path::Path;

#[cfg(feature = "random")]
extern crate sqlite;

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
  pub fn new() -> LocalTriggerBoard {
    LocalTriggerBoard {
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
  fn default() -> LocalTriggerBoard {
      LocalTriggerBoard::new()
  }
}

impl fmt::Display for LocalTriggerBoard {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<LocalTriggerBoard: ID {}; DSI {}; J {}>", self.ltb_id, self.ltb_dsi, self.ltb_j)
  }
}

#[cfg(feature = "random")]
pub fn get_rbs_from_sqlite(filename : &Path) -> Vec<ReadoutBoard> {
  let connection = sqlite::open(filename).unwrap();
  let query = "SELECT * FROM tof_db_rb";
  let mut rbs  = Vec::<ReadoutBoard>::new();
  connection
    .iterate(query, |pairs| {
    println!("New rb, has following values...");
    //let mut ltb = LocalTriggerBoard::new();
    let mut rb = ReadoutBoard::new();
    for &(name, value) in pairs.iter() {
      match value {
        None    => {continue;},
        Some(v) => {
          println!("{} = {}", name, v);
          match name {
            "rb_id"      => {rb.rb_id  = u8::from_str(v).unwrap_or(0);},
            "port"       => {rb.port   = u16::from_str(v).unwrap_or(0);},
            //"ip_address" => {10.0.1.116
            "ch1_paddle_id" => {rb.set_paddle_end_id_for_rb_channel(1, u16::from_str(v).unwrap_or(0));},
            "ch2_paddle_id" => {rb.set_paddle_end_id_for_rb_channel(2, u16::from_str(v).unwrap_or(0));},
            "ch3_paddle_id" => {rb.set_paddle_end_id_for_rb_channel(3, u16::from_str(v).unwrap_or(0));},
            "ch4_paddle_id" => {rb.set_paddle_end_id_for_rb_channel(4, u16::from_str(v).unwrap_or(0));},
            "ch5_paddle_id" => {rb.set_paddle_end_id_for_rb_channel(5, u16::from_str(v).unwrap_or(0));},
            "ch6_paddle_id" => {rb.set_paddle_end_id_for_rb_channel(6, u16::from_str(v).unwrap_or(0));},
            "ch7_paddle_id" => {rb.set_paddle_end_id_for_rb_channel(7, u16::from_str(v).unwrap_or(0));},
            "ch8_paddle_id" => {rb.set_paddle_end_id_for_rb_channel(8, u16::from_str(v).unwrap_or(0));},
            _ => {println!("Found name {}", name);}                         
          }
        }
      }
    } // end loop over rbs
    rbs.push(rb);
    true
  });
  println!("We found {} rbs", rbs.len()); 
  rbs
}


#[cfg(feature = "random")]
pub fn get_ltbs_from_sqlite(filename : &Path) -> Vec<LocalTriggerBoard> {
  let connection = sqlite::open(filename).unwrap();
  let query = "SELECT * FROM tof_db_ltb";
  let mut ltbs  = Vec::<LocalTriggerBoard>::new();
  connection
    .iterate(query, |pairs| {
    println!("New ltb, has following values...");
    let mut ltb = LocalTriggerBoard::new();
    for &(name, value) in pairs.iter() {
      match value {
        None    => {continue;},
        Some(v) => {
          match name {
            //println!("{} = {}", name, v);
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
            _ => {println!("FOund name {}", name);}                         
          }
        }
      }
    } // end loop over ltbs
    println!("{}", ltb);
    ltbs.push(ltb);
    true
  });
  println!("We found {} ltbs", ltbs.len()); 
  ltbs
}


//---------------------------------------------------------

pub struct Panel {
    panel_id                : u8, 
    smallest_paddle_id      : u8, 
    n_paddles               : u8, 
}

impl Panel {
  pub fn new() -> Panel {
    Panel {
      panel_id           : 0,
      smallest_paddle_id : 0,
      n_paddles          : 0
    }
  }
}

impl Default for Panel {
  fn default() -> Panel {
    Panel::new()
  }
}

impl fmt::Display for Panel {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<PANEL: ID {}; SMALLEST PADDLE ID {}; N PADDLES {}>", self.panel_id, self.smallest_paddle_id, self.n_paddles)
  }
}

//---------------------------------------------------------

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

  pub fn new() -> Paddle {
    Paddle {
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
  fn default() -> Paddle {
    Paddle::new()
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
    -> PaddleEnd {
    let mut pe = PaddleEnd {
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
  fn default() -> PaddleEnd {
    PaddleEnd::new(0,PaddleEndIdentifier::A, PaddleEndLocation::PositiveX)
  }
}

impl fmt::Display for PaddleEnd {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    
    write!(f, "<PaddleEnd: PADDLE ID {}, END {:?}>", self.paddle_id, self.end)
  }
}

//---------------------------------------------------------

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
}

impl Default for DSICard {
  fn default() -> DSICard {
    DSICard::new()
  }
}

impl fmt::Display for DSICard {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
    write!(f, "<DSICard: DSI ID {}>", self.dsi_id)
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
  pub fn new() -> RAT {
    RAT {  
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
  fn default() -> RAT {
    RAT::new()
  }
}

impl fmt::Display for RAT {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
    write!(f, "<RAT: ID {}>", self.rat_id)
  }
}

#[derive(Clone, Debug)]
pub struct ReadoutBoard {
  pub rb_id         : u8,  
  pub dna           : u64, 
  pub port          : u16, 
  pub ip_address    : Ipv4Addr,
  //pub ip_address       = models.GenericIPAddressField(unique=True)
  //pub mac_address      = models.CharField(max_length=11, unique=True, null=True)
  channel_to_paddle_end_id : [u16;8],
  pub calib_file    : String,
  pub trig_ch_mask  : [bool;8],
}

impl ReadoutBoard {
  pub fn new() -> ReadoutBoard {
    ReadoutBoard {
      rb_id         : 0,  
      dna           : 0, 
      port          : 0, 
      ip_address    : Ipv4Addr::new(0,0,0,0),
      //b ip_address  0  = models.GenericIPAddressField(unique=True)
      //b mac_address 0  = models.CharField(max_length=11, unique=True, null=True)
      channel_to_paddle_end_id : [0;8],
      calib_file    : String::from(""),
      trig_ch_mask  : [false;8],
    }
  }

  pub fn infer_ip_address(&mut self) {
    let address = 100 + self.rb_id;
    self.ip_address = Ipv4Addr::new(10,0,1,address);
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
    return self.calib_file.clone();
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
  fn default() -> ReadoutBoard {
    ReadoutBoard::new()
  }
}

impl fmt::Display for ReadoutBoard {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
    write!(f, "<ReadoutBoard: ID {}>", self.rb_id)
  }
}

//#[test]
//fn test_get_rbs_sqlite() {
//  get_ltbs_from_sqlite();
//  get_rbs_from_sqlite();
//}


