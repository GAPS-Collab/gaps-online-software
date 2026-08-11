// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;
use colored::Colorize;

/// The RBEvent header gets generated once per event
/// per RB. 
/// Contains information about event id, timestamps,
/// etc.
#[cfg_attr(feature="pybindings",pyclass)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RBEventHeader { 
  /// Readoutboard ID - should be in the range 1-50
  /// not consecutive, there are some missing.
  /// In general, we have 40 boards
  pub rb_id                : u8   ,
  /// The event ID as sent from the MTB or self-generated
  /// if not latched to the MTB
  pub event_id             : u32  ,
  /// The DRS stop cell. This is vital information which is
  /// needed for the calibration
  pub stop_cell            : u16  , 
  /// RBPaddleID - component 
  pub pid_ch12             : u8,
  /// RBPaddleID - component
  pub pid_ch34             : u8,
  /// RBPaddleID - component
  pub pid_ch56             : u8,
  /// RBPaddleID - component
  pub pid_ch78             : u8,
  /// RBPaddleID - component
  pub pid_ch_order         : u8,
  /// Reserved
  pub rsvd1                : u8,
  /// Reserved
  pub rsvd2                : u8,
  /// Reserved
  pub rsvd3                : u8,
  /// The adc value for the temperature
  /// of the FPGA
  pub fpga_temp             : u16, 
  /// DRS deadtime as read out from the 
  /// register
  pub drs_deadtime          : u16,
  pub timestamp32           : u32,
  pub timestamp16           : u16,
  /// Store the drs_deadtime instead 
  /// of the fpga temperature
  pub deadtime_instead_temp : bool,
  /// The status byte contains information about lsos of lock
  /// and event fragments and needs to be decoded
  pub status_byte               : u8,
  /// The channel mask is 9bit for the 9 channels.
  /// This leaves 7 bits of space so we actually 
  /// hijack that for the version information 
  /// 
  /// Bit 15 will be set 1 in case we are sending
  /// the DRS_DEADTIME instead of FPGA TEMP
  ///
  /// FIXME - make this proper and use ProtocolVersion 
  ///         instead
  pub channel_mask             : u16, 
}

impl RBEventHeader {

  pub fn new() -> Self {
    Self {
      rb_id                 : 0,  
      status_byte           : 0, 
      event_id              : 0,  
      channel_mask          : 0,  
      stop_cell             : 0,  
      pid_ch12              : 0,
      pid_ch34              : 0,
      pid_ch56              : 0,
      pid_ch78              : 0,
      pid_ch_order          : 0,
      rsvd1                 : 0,
      rsvd2                 : 0,
      rsvd3                 : 0,
      fpga_temp             : 0,  
      drs_deadtime          : 0,
      timestamp32           : 0,
      timestamp16           : 0,
      deadtime_instead_temp : false,
    }
  }

  /// Set the channel mask with the 9bit number
  ///
  /// Set bit 15 to either 1 or 0 depending on
  /// deadtime_instead_temp
  pub fn set_channel_mask(&mut self, channel_mask : u16) {
    if self.deadtime_instead_temp {
      self.channel_mask = 2u16.pow(15) | channel_mask;
    } else {
      self.channel_mask = channel_mask;
    }
  }

  /// Just return the channel mask and strip of 
  /// the part which contains the information about
  /// drs deadtime or fpga temp
  pub fn get_channel_mask(&self) -> u16 {
    self.channel_mask & 0x1ff 
  }

  /// Get the channel mask from a bytestream.
  /// 
  /// This takes into acount that bit 15 is 
  /// used to convey information about if we
  /// stored the drs temperature or deadtime
  pub fn parse_channel_mask(ch_mask : u16) -> (bool, u16) {
    let channel_mask          : u16;
    let deadtime_instead_temp : bool 
      = ch_mask >> 15 == 1;
    channel_mask = ch_mask & 0x1ff;
    (deadtime_instead_temp, channel_mask)
  }

  /// Only get the eventid from a binary stream
  pub fn extract_eventid_from_rbheader(stream :&Vec<u8>) -> u32 {
    // event id is 18 bytes in (including HEAD bytes)
    // event id is 3 bytes in (including HEAD bytes)
    let event_id = parse_u32(stream, &mut 3); // or should it be 5?
    event_id
  }
  
  pub fn is_event_fragment(&self) -> bool {
    self.status_byte & 1 > 0
  }
  
  pub fn drs_lost_trigger(&self) -> bool {
    (self.status_byte >> 1) & 1 > 0
  }

  pub fn lost_lock(&self) -> bool {
    (self.status_byte >> 2) & 1 > 0
  }

  pub fn lost_lock_last_sec(&self) -> bool {
    (self.status_byte >> 3) & 1 > 0
  }

  pub fn is_locked(&self) -> bool {
    !self.lost_lock()
  }
  
  pub fn is_locked_last_sec(&self) -> bool {
    !self.lost_lock_last_sec()
  }
  
  /// extract lock, drs busy and fpga temp from status field
  pub fn parse_status(&mut self, status_bytes : u16) {
    // status byte is only 4bit really
    self.status_byte        = (status_bytes & 0xf) as u8;
    self.fpga_temp = status_bytes >> 4;
  }

  /// Get the temperature value (Celsius) from the fpga_temp adc.
  pub fn get_fpga_temp(&self) -> f32 {
    let zynq_temp = (((self.fpga_temp & 4095) as f32 * 503.975) / 4096.0) - 273.15;
    zynq_temp
  }

  /// Check if the channel 9 is present in the 
  /// channel mask
  pub fn has_ch9(&self) -> bool {
    self.channel_mask & 256 > 0
  }

  pub fn get_rbpaddleid(&self) -> RBPaddleID {
    let mut pid = RBPaddleID::new();
    pid.paddle_12     = self.pid_ch12;
    pid.paddle_34     = self.pid_ch34;
    pid.paddle_56     = self.pid_ch56;
    pid.paddle_78     = self.pid_ch78;
    pid.channel_order = self.pid_ch_order;
    pid                    
  }
  
  pub fn set_rbpaddleid(&mut self, pid : &RBPaddleID) {
    self.pid_ch12     = pid.paddle_12;
    self.pid_ch34     = pid.paddle_34;
    self.pid_ch56     = pid.paddle_56;
    self.pid_ch78     = pid.paddle_78;
    self.pid_ch_order = pid.channel_order;
  }

  /// Decode the channel mask into channel ids.
  ///
  /// The channel ids inside the memory representation
  /// of the RB Event data ("blob") are from 0-7
  ///
  /// We keep ch9 seperate.
  pub fn get_channels(&self) -> Vec<u8> {
    let mut channels = Vec::<u8>::with_capacity(8);
    for k in 0..9 {
      if self.channel_mask & (1 << k) > 0 {
        channels.push(k);
      }
    }
    channels
  }

  /// Get the active paddles
  pub fn get_active_paddles(&self) -> Vec<(u8,bool)> {
    // FIXME - help. Make this nicer. My brain is fried 
    // at this point. Please. I'll be thankful.
    let mut active_paddles = Vec::<(u8,bool)>::new();
    let active_channels = self.get_channels();
    let pid             = self.get_rbpaddleid();
    let mut ch0_pair_done = false;
    let mut ch2_pair_done = false;
    let mut ch4_pair_done = false;
    let mut ch6_pair_done = false;
    for ch in active_channels {
      if (ch == 0 || ch == 1) && !ch0_pair_done {
        active_paddles.push(pid.get_paddle_id(ch));
        ch0_pair_done = true;
      }
      if (ch == 2 || ch == 3) && !ch2_pair_done {
        active_paddles.push(pid.get_paddle_id(ch));
        ch2_pair_done = true;
      }
      if (ch == 4 || ch == 5) && !ch4_pair_done {
        active_paddles.push(pid.get_paddle_id(ch));
        ch4_pair_done = true;
      }
      if (ch == 6 || ch == 7) && !ch6_pair_done {
        active_paddles.push(pid.get_paddle_id(ch));
        ch6_pair_done = true;
      }
    }
    active_paddles
  }

  /// Get the number of data channels + 1 for ch9
  pub fn get_nchan(&self) -> usize {
    self.get_channels().len()
  }
  
  pub fn get_timestamp48(&self) -> u64 {
    ((self.timestamp16 as u64) << 32) | self.timestamp32 as u64
  }
}

impl Default for RBEventHeader {
  fn default() -> Self {
    Self::new()
  }
}


impl fmt::Display for RBEventHeader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<RBEventHeader:");
    repr += &(format!("\n  RB ID            {}",self.rb_id               )); 
    repr += &(format!("\n  event id         {}",self.event_id            ));  
    repr += &(format!("\n  ch mask          {}",self.channel_mask        ));  
    repr += &(format!("\n  has ch9          {}",self.has_ch9()           )); 
    repr += &(format!("\n  ch mapping       {}",self.get_rbpaddleid()    ));
    if self.deadtime_instead_temp {
      repr += &(format!("\n  DRS deadtime          : {:.2}", self.drs_deadtime));
    } else {
      repr += &(format!("\n  FPGA T [\u{00B0}C]    : {:.2}", self.get_fpga_temp()));
    }
    repr += &(format!("\n  timestamp32      {}", self.timestamp32            )); 
    repr += &(format!("\n  timestamp16      {}", self.timestamp16            )); 
    repr += &(format!("\n   |-> timestamp48 {}", self.get_timestamp48()      )); 
    repr += &(format!("\n  stop cell        {}", self.stop_cell              )); 
    let mut perfect = true;
    if self.drs_lost_trigger() {
      repr += &"\n  !! DRS4 REPORTS LOST TRIGGER!".red().bold();
      perfect = false;
    }
    if self.is_event_fragment() {
      repr += &"\n  !! EVENT FRAGMENT!".red().bold();
      perfect = false;
    }
    if self.lost_lock() {
      repr += &"\n  !! RB CLOCK IS NOT LOCKED!".yellow().bold();
      perfect = false;
    }
    if self.lost_lock_last_sec() {
      repr += &"\n  !! RB CLOCK HAS LOST ITS LOCK WITHIN THE LAST SECOND!".yellow().bold();
      perfect = false;
    }
    if perfect {
      repr += &"\n  -- locked: YES, locked last second; YES, no event fragemnet, and no lost trigger!".green();
    }
    repr += ">";
    write!(f, "{}", repr)
  }
}

impl Serialization for RBEventHeader {
  
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;
  const SIZE : usize = 30; // size in bytes with HEAD and TAIL

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<Self, SerializationError> {
    let mut header  = Self::new();
    Self::verify_fixed(stream, pos)?;
    header.rb_id                 = parse_u8 (stream, pos);  
    header.event_id              = parse_u32(stream, pos);  
    let ch_mask                  = parse_u16(stream, pos);
    let (deadtime_instead_temp, channel_mask)  
      = Self::parse_channel_mask(ch_mask);
    header.deadtime_instead_temp = deadtime_instead_temp;
    header.set_channel_mask(channel_mask);
    header.status_byte         = parse_u8 (stream, pos);
    header.stop_cell             = parse_u16(stream, pos);  
    header.pid_ch12              = parse_u8(stream, pos);
    header.pid_ch34              = parse_u8(stream, pos);
    header.pid_ch56              = parse_u8(stream, pos);
    header.pid_ch78              = parse_u8(stream, pos);
    header.pid_ch_order          = parse_u8(stream, pos);
    header.rsvd1                 = parse_u8(stream, pos);
    header.rsvd2                 = parse_u8(stream, pos);
    header.rsvd3                 = parse_u8(stream, pos);
    if deadtime_instead_temp {
      header.drs_deadtime        = parse_u16(stream, pos);
    } else {
      header.fpga_temp           = parse_u16(stream, pos);
    }
    header.timestamp32           = parse_u32(stream, pos);
    header.timestamp16           = parse_u16(stream, pos);
    *pos += 2; // account for tail earlier 
    Ok(header) 
  }
  

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.rb_id             .to_le_bytes());
    stream.extend_from_slice(&self.event_id          .to_le_bytes());
    let ch_mask = ((self.deadtime_instead_temp as u16) << 15) | self.get_channel_mask();
    stream.extend_from_slice(&ch_mask                .to_le_bytes());
    stream.extend_from_slice(&self.status_byte       .to_le_bytes());
    stream.extend_from_slice(&self.stop_cell         .to_le_bytes());
    stream.push(self.pid_ch12    );
    stream.push(self.pid_ch34    );
    stream.push(self.pid_ch56    );
    stream.push(self.pid_ch78    );
    stream.push(self.pid_ch_order);
    stream.push(self.rsvd1       );
    stream.push(self.rsvd2       );
    stream.push(self.rsvd3       );
    if self.deadtime_instead_temp {
      stream.extend_from_slice(&self.drs_deadtime    .to_le_bytes());
    } else {
      stream.extend_from_slice(&self.fpga_temp       .to_le_bytes());
    }
    stream.extend_from_slice(&self.timestamp32       .to_le_bytes());
    stream.extend_from_slice(&self.timestamp16       .to_le_bytes());
    stream.extend_from_slice(&RBEventHeader::TAIL.to_le_bytes());
    stream
  }
}

#[cfg(feature = "random")]
impl FromRandom for RBEventHeader {
    
  fn from_random() -> Self {
    let mut header = RBEventHeader::new();
    let mut rng = rand::rng();

    header.rb_id                 = rng.random::<u8>();    
    header.event_id              = rng.random::<u32>();   
    header.status_byte           = rng.random::<u8>();    
    header.stop_cell             = rng.random::<u16>();   
    header.pid_ch12              = rng.random::<u8>();
    header.pid_ch34              = rng.random::<u8>();
    header.pid_ch56              = rng.random::<u8>();
    header.pid_ch78              = rng.random::<u8>();
    header.pid_ch_order          = rng.random::<u8>();
    header.rsvd1                 = rng.random::<u8>();
    header.rsvd2                 = rng.random::<u8>();
    header.rsvd3                 = rng.random::<u8>();
    header.deadtime_instead_temp = rng.random::<bool>();
    if header.deadtime_instead_temp {
      header.drs_deadtime          = rng.random::<u16>();
    } else {
      header.fpga_temp             = rng.random::<u16>();  
    }
    // make sure the generated channel mask is valid!
    let ch_mask                  = rng.random::<u16>() & 0x1ff;
    header.set_channel_mask(ch_mask);
    header.timestamp32           = rng.random::<u32>();
    header.timestamp16           = rng.random::<u16>();
    header
  }
}

//------------------------------------------

#[cfg(feature="pybindings")]
#[pymethods]
impl RBEventHeader {
  
  #[getter]
  #[pyo3(name="rb_id")]
  fn rb_id_py(&self) -> u8 {
    self.rb_id
  }
  
  #[getter]
  #[pyo3(name="event_id")]
  fn event_id_py(&self) -> u32 {
    self.event_id
  }
  
  //#[getter]
  //fn status_byte(&self) -> u8 {
  //  self.status_byte
  //}
  
  #[getter]
  #[pyo3(name="channel_mask")]
  fn channel_mask_py(&self) -> u16 {
    self.get_channel_mask()
  }
  
  #[getter]
  #[pyo3(name="stop_cell")]
  fn stop_cell_py(&self) -> u16 {
    self.stop_cell
  }
  
  #[getter]
  #[pyo3(name="fpga_temp")]
  fn fpga_temp_py(&self) -> f32 {
    self.get_fpga_temp()
  }
  
  #[getter]
  #[pyo3(name="drs_deadtime")]
  fn drs_deadtime_py(&self) -> u16 {
    self.drs_deadtime 
  }

  #[getter]
  #[pyo3(name="timestamp32")]
  fn timestamp32_py(&self) -> u32 {
    self.timestamp32
  }
  
  #[getter]
  #[pyo3(name="timestamp16")]
  fn timestamp16_py(&self) -> u16 {
    self.timestamp16
  }

  //  pub ch9_amp: u16,
  //  pub ch9_freq: u16,
  //  pub ch9_phase: u32,
  #[pyo3(name="get_channels")]
  fn get_channels_py(&self) -> Vec<u8> {
    self.get_channels()
  }

  #[getter]
  #[pyo3(name="is_event_fragment")]
  pub fn is_event_fragment_py(&self) -> bool {
    self.is_event_fragment()
  }

  #[getter]
  #[pyo3(name="drs_lost_trigger")]
  pub fn drs_lost_trigger_py(&self) -> bool {
    self.drs_lost_trigger()
  }

  #[getter]
  #[pyo3(name="lost_lock")]
  fn lost_lock_py(&self) -> bool {
    self.lost_lock()
  }

  #[getter]
  #[pyo3(name="lost_lock_last_sec")]
  fn lost_lock_last_sec_py(&self) -> bool {
    self.lost_lock_last_sec()
  }

  #[getter]
  #[pyo3(name="is_locked")]
  fn is_locked_py(&self) -> bool {
    self.is_locked()
  }

  #[getter]
  #[pyo3(name="is_locked_last_sec")]
  fn is_locked_last_sec_py(&self) -> bool {
    self.is_locked_last_sec()
  }
}

#[cfg(feature="pybindings")]
pythonize!(RBEventHeader);

//------------------------------------------

#[test]
fn serialization_rbeventheader() {
  for _ in 0..100 {
    let mut pos = 0usize;
    let head = RBEventHeader::from_random();
    //println!("{}",  head);
    let stream = head.to_bytestream();
    assert_eq!(stream.len(), RBEventHeader::SIZE);
    let test = RBEventHeader::from_bytestream(&stream, &mut pos).unwrap();
    println!("{}", test);
    assert_eq!(pos, RBEventHeader::SIZE);
    assert_eq!(head, test);
    assert_eq!(head.lost_lock()         , test.lost_lock());
    assert_eq!(head.lost_lock_last_sec(), test.lost_lock_last_sec());
    assert_eq!(head.drs_lost_trigger()  , test.drs_lost_trigger());
    assert_eq!(head, test);
  }
}

