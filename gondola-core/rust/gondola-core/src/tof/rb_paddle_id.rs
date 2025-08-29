// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Squeze the rb channel - paddle mapping into 5 bytes
/// for a single RB
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct RBPaddleID {
  /// Paddle connected to RB channel 1/2
  pub paddle_12     : u8,
  /// Paddle connected to RB channel 3/4
  pub paddle_34     : u8,
  /// Paddle connected to RB channel 5/6
  pub paddle_56     : u8,
  /// Paddle connected to RB channel 7/8
  pub paddle_78     : u8,
  /// Order - 1 if the smaller channel is the 
  /// A side, 2, if the smaller channel is the 
  /// B side
  pub channel_order : u8
}

impl Default for RBPaddleID {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for RBPaddleID {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<RBPaddleID:");
    for k in 1..9 {
      let pid = self.get_paddle_id(k);
      let ord = self.get_order_str(k);
      repr += &(format!("\n  {k} -> {} ({ord})", pid.0)) 
    }
    repr += ">";
    write!(f, "{}", repr)
  }
}

impl RBPaddleID {
  pub fn new() -> Self {
    RBPaddleID {
      paddle_12     : 0,
      paddle_34     : 0,
      paddle_56     : 0,
      paddle_78     : 0,
      channel_order : 0
    }
  }

  pub fn to_u64(&self) -> u64 {
    let val : u64 = (self.channel_order as u64) << 32 | (self.paddle_78 as u64) << 24 | (self.paddle_56 as u64) << 16 | (self.paddle_34 as u64) << 8 |  self.paddle_12 as u64;
    val
  }

  /// Typically, the A-side will be connected to a lower channel id
  ///
  /// If the order is flipped, the lower channel will be connected to 
  /// the B-side
  ///
  /// # Arguments
  /// * channel : RB channel (1-8)
  pub fn get_order_flipped(&self, channel : u8) -> bool {
    match channel {
      1 | 2 => {
        return (self.channel_order & 1) == 1;
      }
      3 | 4 => {
        return (self.channel_order & 2) == 2;
      }
      5 | 6 => {
        return (self.channel_order & 4) == 4;
      }
      7 | 8 => {
        return (self.channel_order & 8) == 8;
      }
      _ => {
        error!("{} is not a valid RB channel!", channel);
        return false;
      }
    }
  }

  pub fn get_order_str(&self, channel : u8) -> String {
    if self.get_order_flipped(channel) {
      return String::from("BA");
    } else {
      return String::from("AB");
    }
  }

  pub fn is_a(&self, channel : u8) -> bool {
    match channel {
      1 => {
        if self.get_order_flipped(channel) {
          return false;
        } else {
          return true
        }
      }
      2 => {
        if self.get_order_flipped(channel) {
          return true;
        } else {
          return false
        }
      }
      3 => {
        if self.get_order_flipped(channel) {
          return false;
        } else {
          return true
        }
      }
      4 => {
        if self.get_order_flipped(channel) {
          return true;
        } else {
          return false
        }
      }
      5 => {
        if self.get_order_flipped(channel) {
          return false;
        } else {
          return true
        }
      }
      6 => {
        if self.get_order_flipped(channel) {
          return true;
        } else {
          return false
        }
      }
      7 => {
        if self.get_order_flipped(channel) {
          return false;
        } else {
          return true
        }
      }
      8 => {
        if self.get_order_flipped(channel) {
          return true;
        } else {
          return false
        }
      }
      _ => {
        error!("{} is not a valid RB channel!", channel);
        return false;
      }
    }
  }

  pub fn from_u64(val : u64) -> Self {
    let paddle_12     : u8 = ((val & 0xFF)) as u8;
    let paddle_34     : u8 = ((val & 0xFF00) >> 8) as u8;
    let paddle_56     : u8 = ((val & 0xFF0000) >> 16) as u8;
    let paddle_78     : u8 = ((val & 0xFF000000) >> 24) as u8; 
    let channel_order : u8 = ((val & 0xFF00000000) >> 32) as u8;
    Self {
      paddle_12,
      paddle_34,
      paddle_56,
      paddle_78,
      channel_order,
    }
  }


  /// Get the paddle id together with the information 
  /// if this is the A side
  ///
  /// channel in rb channels (starts at 1)
  pub fn get_paddle_id(&self, channel : u8) -> (u8, bool) {
    let flipped = self.get_order_flipped(channel);
    match channel {
      1 | 2 => {
        return (self.paddle_12, flipped); 
      }
      3 | 4 => {
        return (self.paddle_34, flipped); 
      }
      5 | 6 => {
        return (self.paddle_56, flipped); 
      }
      7 | 8 => {
        return (self.paddle_78, flipped); 
      }
      _ => {
        error!("{} is not a valid RB channel!", channel);
        return (0,false);
      }
    }
  }
}


#[cfg(feature="pybindings")]
#[pymethods]
impl RBPaddleID {
  #[new]
  fn new_py() -> Self {
    Self::new()
  }

  #[pyo3(name="get_paddle_id")]
  fn get_paddle_id_py(&self, channel : u8) -> (u8, bool) {
    self.get_paddle_id(channel)
  } 
}

#[cfg(feature="database")]
impl RBPaddleID {

  pub fn from_rb( rb : &ReadoutBoard) -> Self {
    let mut rb_pid = RBPaddleID::new();
    rb_pid.paddle_12 = rb.paddle12.paddle_id as u8;    
    rb_pid.paddle_34 = rb.paddle34.paddle_id as u8;    
    rb_pid.paddle_56 = rb.paddle56.paddle_id as u8;    
    rb_pid.paddle_78 = rb.paddle78.paddle_id as u8;    
    let mut flipped  = 0u8 ;
    if rb.get_paddle12_chA() != 1 {
      flipped |= 1;
    }
    if rb.get_paddle34_chA() != 3 {
      flipped |= 2;
    }
    if rb.get_paddle56_chA() != 5 {
      flipped |= 4;
    }
    if rb.get_paddle78_chA() != 7 {
      flipped |= 8;
    }
    rb_pid.channel_order = flipped;
    rb_pid
  }
}

#[cfg(feature = "random")]
impl FromRandom for RBPaddleID {
    
  fn from_random() -> Self {
    let mut rb_pid  = Self::new();
    let mut rng = rand::rng();
    rb_pid.paddle_12   = rng.random::<u8>();
    rb_pid.paddle_34   = rng.random::<u8>();
    rb_pid.paddle_56   = rng.random::<u8>();
    rb_pid.paddle_78   = rng.random::<u8>();
    rb_pid.channel_order = rng.random::<u8>();
    rb_pid
  }
}


