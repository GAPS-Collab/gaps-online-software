// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Monitoring the MTB
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct MtbMoniData {
  pub tiu_busy_len : u32,
  /// tiu_status\[0\] = emu_mode
  /// tiu_status\[1\] = use_aux_link
  /// tiu_status\[2\] = tiu_bad
  /// tiu_status\[3\] = bsy_stuck
  /// tiu_status\[4\] = ignore_bsy
  pub tiu_status   : u8,
  ///// Prescale factor in per cent
  ///// (might not be accurate)
  //pub prescale     : f16,
  //pub rsvd         : u8,
  pub daq_queue_len: u16,
  //pub vccpaux      : u16, 
  //pub vccoddr      : u16, 
  pub temp         : u16, 
  /// Unfortunatly at this point we only have
  /// a single byte left
  pub rb_lost_rate : u8,
  pub rate         : u16, 
  pub lost_rate    : u16, 
  pub vccint       : u16, 
  pub vccbram      : u16, 
  pub vccaux       : u16, 
}

impl MtbMoniData {
  
  pub fn new() -> Self {
    Self {
      tiu_busy_len  : u32::MAX,
      tiu_status    : u8::MAX,
      //rsvd          : u8::MAX,
      //prescale      : f16::MAX,
      daq_queue_len : u16::MAX,
      temp          : u16::MAX,
      vccint        : u16::MAX,
      vccaux        : u16::MAX,
      vccbram       : u16::MAX,
      rate          : u16::MAX,
      lost_rate     : u16::MAX,
      rb_lost_rate  : u8::MAX
    }
  }

  pub fn get_tiu_emulation_mode(&self) -> bool {
    self.tiu_status & 0x1 > 0
  }
  
  pub fn get_tiu_use_aux_link(&self) -> bool {
    self.tiu_status & 0x2 > 0
  }

  pub fn get_tiu_bad(&self) -> bool { 
    self.tiu_status & 0x4 > 0
  }

  pub fn get_tiu_busy_stuck(&self) -> bool {
    self.tiu_status & 0x8 > 0
  }

  pub fn get_tiu_ignore_busy(&self) -> bool {
    self.tiu_status & 0x10 > 0
  }


  /// Convert ADC temp from adc values to Celsius
  pub fn get_fpga_temp(&self) -> f32 {
    self.temp as f32 * 503.975 / 4096.0 - 273.15
  }
  
  /// Convert ADC VCCINT from adc values to Voltage
  pub fn adc_vcc_conversion(data : u16) -> f32 {
    3.0 * data as f32 / (2_u32.pow(12-1)) as f32
  }

  //pub fn set_prescale(&mut self, prescale : f32) {
  //  self.prescale = f16::from_f32(prescale);
  //}

  //pub fn get_prescale(&self) -> f32 {
  //  self.prescale.to_f32()  
  //}
}

impl Default for MtbMoniData {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for MtbMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<MtbMoniData:
  MTB  EVT RATE  [Hz] {}
  LOST EVT RATE  [Hz] {}
  LOST RB EVT R  [Hz] {}
  TIU BUSY CNT  [CLK] {}
  DAQ QUEUE LEN       {}
  --- TIU STATUS ---
    EMU MODE          {}
    USE AUX LINK      {}
    TIU BAD           {}
    BUSY STUCK        {}
    IGNORE BUSY       {}
  --- --- --- --- --
  FPGA TEMP      [\u{00B0}C] {:.2}
  VCCINT          [V] {:.3}
  VCCAUX          [V] {:.3},
  VCCBRAM         [V] {:.3}>",
           self.rate,
           self.lost_rate,
           self.rb_lost_rate,
           self.tiu_busy_len,
           self.daq_queue_len,
           //self.get_prescale(),
           self.get_tiu_emulation_mode(),
           self.get_tiu_use_aux_link(),
           self.get_tiu_bad(),
           self.get_tiu_busy_stuck(),
           self.get_tiu_ignore_busy(),
           self.get_fpga_temp(),
           Self::adc_vcc_conversion(self.vccint    ),
           Self::adc_vcc_conversion(self.vccaux    ),
           Self::adc_vcc_conversion(self.vccbram   ),
           )
  }
}

impl TofPackable for MtbMoniData {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::MtbMoniData;
}

impl Serialization for MtbMoniData {
  
  const SIZE : usize = 24;
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.tiu_busy_len.to_le_bytes());
    stream.extend_from_slice(&self.tiu_status .to_le_bytes());
    //stream.extend_from_slice(&self.rsvd.to_le_bytes());
    stream.extend_from_slice(&self.rb_lost_rate.to_le_bytes());
    stream.extend_from_slice(&self.daq_queue_len.to_le_bytes());
    stream.extend_from_slice(&self.temp       .to_le_bytes());
    stream.extend_from_slice(&self.vccint     .to_le_bytes()); 
    stream.extend_from_slice(&self.vccaux     .to_le_bytes()); 
    stream.extend_from_slice(&self.vccbram    .to_le_bytes()); 
    //stream.extend_from_slice(&self.prescale   .to_le_bytes());
    //stream.extend_from_slice(&self.rb_lost_rate.to_le_bytes());
    stream.extend_from_slice(&self.rate       .to_le_bytes()); 
    stream.extend_from_slice(&self.lost_rate  .to_le_bytes());
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<Self, SerializationError> {
    let mut moni_data      = Self::new();
    Self::verify_fixed(stream, pos)?;
    moni_data.tiu_busy_len  = parse_u32(&stream, pos);
    moni_data.tiu_status    = parse_u8(&stream, pos);
    //moni_data.rsvd          = parse_u8(&stream, pos);
    moni_data.rb_lost_rate  = parse_u8(&stream, pos);
    moni_data.daq_queue_len = parse_u16(&stream, pos);
    moni_data.temp          = parse_u16(&stream, pos);
    moni_data.vccint        = parse_u16(&stream, pos);
    //moni_data.prescale      = parse_f16(&stream, pos);
    moni_data.vccaux        = parse_u16(&stream, pos);
    moni_data.vccbram       = parse_u16(&stream, pos);
    //moni_data.rb_lost_rate  = parse_u16(&stream, pos);
    moni_data.rate          = parse_u16(&stream, pos);
    moni_data.lost_rate     = parse_u16(&stream, pos);
    *pos += 2; // since we deserialized the tail earlier and 
              // didn't account for it
    Ok(moni_data)
  }
}

impl MoniData for MtbMoniData {

  fn get_board_id(&self) -> u8 {
    return 0;
  }
  
  fn get(&self, varname : &str) -> Option<f32> {
    match varname {
      "board_id"     => Some(0.0f32),
      "tiu_busy_len" => Some(self.tiu_busy_len as f32), 
      "tiu_status"   => Some(self.tiu_status as f32), 
      "daq_queue_len"  => Some(self.daq_queue_len as f32), 
      //"prescale"     => Some(self.get_prescale()),
      "temp"         => Some(self.get_fpga_temp()), 
      "vccint"       => Some(Self::adc_vcc_conversion(self.vccint)), 
      "vccaux"       => Some(Self::adc_vcc_conversion(self.vccaux)), 
      "vccbram"      => Some(Self::adc_vcc_conversion(self.vccbram)), 
      "rate"         => Some(self.rate as f32), 
      "lost_rate"    => Some(self.lost_rate as f32), 
      "rb_lost_rate" => Some(self.rb_lost_rate as f32), 
      _              => None
    }
  }
  
  fn keys() -> Vec<&'static str> {
    vec![
      "board_id"      ,  
      "tiu_busy_len"  , 
      "tiu_status"    , 
      "daq_queue_len" , 
      "temp"          , 
      "vccint"        , 
      "vccaux"        , 
      "vccbram"       , 
      "rb_lost_rate"  ,
      "rate"          , 
      "lost_rate"] 
  }
}

#[cfg(feature = "random")]
impl FromRandom for MtbMoniData {
  fn from_random() -> Self {
    let mut moni      = Self::new();
    let mut rng       = rand::rng();
    moni.tiu_busy_len = rng.random::<u32>();
    moni.tiu_status   = rng.random::<u8>();
    //moni.prescale     = f16::from_f32(rng.random::<f32>());
    moni.daq_queue_len= rng.random::<u16>();
    moni.temp         = rng.random::<u16>();
    moni.vccint       = rng.random::<u16>();
    moni.vccaux       = rng.random::<u16>();
    moni.vccbram      = rng.random::<u16>();
    moni.rb_lost_rate = rng.random::<u8>();
    moni.rate         = rng.random::<u16>();
    moni.lost_rate    = rng.random::<u16>();
    moni
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl MtbMoniData {
  
  #[getter]
  fn get_vccint(&self) -> f32 {
    Self::adc_vcc_conversion(self.vccint)
  }

  #[getter]
  fn get_vccbram(&self) -> f32 {
    Self::adc_vcc_conversion(self.vccbram)
  }

  #[getter]
  fn get_vccaux(&self) -> f32 {
    Self::adc_vcc_conversion(self.vccaux)
  }

  #[getter]
  pub fn get_rate(&self) -> u16 {
    self.rate
  }
  
  #[getter]
  pub fn get_lost_rate(&self) -> u16 {
    self.lost_rate
  }

  #[getter]
  /// Length of the received BUSY signal from 
  /// the TIU in nanoseconds
  pub fn get_tiu_busy_len(&self) -> u32 {
    self.tiu_busy_len * 10
  }

  #[getter]
  pub fn get_daq_queue_len(&self) -> u16 {
    self.daq_queue_len
  }

  #[getter]
  #[pyo3(name="tiu_emulation_mode")]
  pub fn get_tiu_emulation_mode_py(&self) -> bool {
    self.get_tiu_emulation_mode()
  }
  
  #[getter]
  #[pyo3(name="tiu_use_aux_link")]
  pub fn get_tiu_use_aux_link_py(&self) -> bool {
    self.get_tiu_use_aux_link()
  }

  #[getter]
  #[pyo3(name="tiu_bad")]
  pub fn get_tiu_bad_py(&self) -> bool { 
    self.get_tiu_bad()
  }

  #[getter]
  #[pyo3(name="tiu_busy_stuck")]
  pub fn get_tiu_busy_stuck_py(&self) -> bool {
    self.get_tiu_busy_stuck()
  }

  #[getter]
  #[pyo3(name="tiu_ignore_busy")]
  pub fn get_tiu_ignore_busy_py(&self) -> bool {
    self.get_tiu_ignore_busy()
  }


  #[getter]
  #[pyo3(name="fpga_temp")]
  pub fn get_fpga_temp_py(&self) -> f32 {
    self.get_fpga_temp()
  }
}

//----------------------------------------

// make it available as a monidata series
moniseries!(MtbMoniDataSeries, MtbMoniData);

#[cfg(feature="pybindings")]
pythonize_packable!(MtbMoniData);

#[cfg(feature="pybindings")]
pythonize_monidata!(MtbMoniData);

//----------------------------------------

#[test]
#[cfg(feature = "random")]
fn pack_mtbmonidata() {
  for _ in 0..100 {
    let data = MtbMoniData::from_random();
    let test : MtbMoniData = data.pack().unpack().unwrap();
    assert_eq!(data, test);
  }
}

