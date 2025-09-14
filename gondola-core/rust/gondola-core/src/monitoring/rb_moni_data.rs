// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

//use std::fmt;
//
//use crate::monitoring::MoniData;
//
//#[cfg(feature = "random")]
//use crate::random::FromRandom;
//#[cfg(feature = "random")]
//use rand::Rng;
//
//use crate::io::serialization::Serialization;
//
//use crate::packets::TofPackable;
//
//use crate::io::parsers::{
//  parse_u8,
//  parse_u16,
//  parse_f32
//};
//
//use crate::packets::TofPacketType;
//use crate::errors::SerializationError;
//
//#[cfg(feature="pybindings")]
//use pyo3::prelude::*;
//
//#[cfg(feature="pybindings")]
//use pyo3::exceptions::{
//  PyKeyError,
//  PyIOError
//};
//
//#[cfg(feature="pybindings")]
//use crate::packets::TofPacket;
//
//
//use crate::moniseries;
//
//#[cfg(feature="pybindings")]
//use crate::pythonize_packable;
//#[cfg(feature="pybindings")]
//use crate::pythonize_monidata;

#[cfg(feature="tofcontrol")]
use tof_control::helper::rb_type::{
  RBMag,
  RBTemp,
  RBPh,
  RBVcp
};

/// Sensors on the individual RB
///  
/// This includes temperatures, power data,
/// pressure, humidity
/// as well as the magnetic sensors
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct RBMoniData {
  pub board_id           : u8,
  pub rate               : u16,
  pub tmp_drs            : f32,
  pub tmp_clk            : f32,
  pub tmp_adc            : f32,
  pub tmp_zynq           : f32,
  pub tmp_lis3mdltr      : f32,
  pub tmp_bm280          : f32,
  pub pressure           : f32,
  pub humidity           : f32,
  pub mag_x              : f32,
  pub mag_y              : f32,
  pub mag_z              : f32,
  pub drs_dvdd_voltage   : f32, 
  pub drs_dvdd_current   : f32,
  pub drs_dvdd_power     : f32,
  pub p3v3_voltage       : f32,
  pub p3v3_current       : f32,
  pub p3v3_power         : f32,
  pub zynq_voltage       : f32,
  pub zynq_current       : f32,
  pub zynq_power         : f32,
  pub p3v5_voltage       : f32, 
  pub p3v5_current       : f32,
  pub p3v5_power         : f32,
  pub adc_dvdd_voltage   : f32,
  pub adc_dvdd_current   : f32,
  pub adc_dvdd_power     : f32,
  pub adc_avdd_voltage   : f32,
  pub adc_avdd_current   : f32,
  pub adc_avdd_power     : f32,
  pub drs_avdd_voltage   : f32, 
  pub drs_avdd_current   : f32,
  pub drs_avdd_power     : f32,
  pub n1v5_voltage       : f32,
  pub n1v5_current       : f32,
  pub n1v5_power         : f32,
}

impl RBMoniData {
  
  pub fn new() -> Self {
    Self {
      board_id           : 0, 
      rate               : 0,
      tmp_drs            : f32::MAX,
      tmp_clk            : f32::MAX,
      tmp_adc            : f32::MAX,
      tmp_zynq           : f32::MAX,
      tmp_lis3mdltr      : f32::MAX,
      tmp_bm280          : f32::MAX,
      pressure           : f32::MAX,
      humidity           : f32::MAX,
      mag_x              : f32::MAX,
      mag_y              : f32::MAX,
      mag_z              : f32::MAX,
      drs_dvdd_voltage   : f32::MAX, 
      drs_dvdd_current   : f32::MAX,
      drs_dvdd_power     : f32::MAX,
      p3v3_voltage       : f32::MAX,
      p3v3_current       : f32::MAX,
      p3v3_power         : f32::MAX,
      zynq_voltage       : f32::MAX,
      zynq_current       : f32::MAX,
      zynq_power         : f32::MAX,
      p3v5_voltage       : f32::MAX, 
      p3v5_current       : f32::MAX,
      p3v5_power         : f32::MAX,
      adc_dvdd_voltage   : f32::MAX,
      adc_dvdd_current   : f32::MAX,
      adc_dvdd_power     : f32::MAX,
      adc_avdd_voltage   : f32::MAX,
      adc_avdd_current   : f32::MAX,
      adc_avdd_power     : f32::MAX,
      drs_avdd_voltage   : f32::MAX, 
      drs_avdd_current   : f32::MAX,
      drs_avdd_power     : f32::MAX,
      n1v5_voltage       : f32::MAX,
      n1v5_current       : f32::MAX,
      n1v5_power         : f32::MAX,
    }
  }

  #[cfg(feature = "tofcontrol")]
  pub fn add_rbvcp(&mut self, rb_vcp   : &RBVcp) {
    self.drs_dvdd_voltage = rb_vcp.drs_dvdd_vcp[0] ;
    self.drs_dvdd_current = rb_vcp.drs_dvdd_vcp[1] ;
    self.drs_dvdd_power   = rb_vcp.drs_dvdd_vcp[2] ;
    self.p3v3_voltage     = rb_vcp.p3v3_vcp[0]  ;
    self.p3v3_current     = rb_vcp.p3v3_vcp[1]  ;
    self.p3v3_power       = rb_vcp.p3v3_vcp[2]  ;
    self.zynq_voltage     = rb_vcp.zynq_vcp[0]  ;
    self.zynq_current     = rb_vcp.zynq_vcp[1]  ;
    self.zynq_power       = rb_vcp.zynq_vcp[2]  ;
    self.p3v5_voltage     = rb_vcp.p3v5_vcp[0]  ;
    self.p3v5_current     = rb_vcp.p3v5_vcp[1]  ;
    self.p3v5_power       = rb_vcp.p3v5_vcp[2]  ;
    self.adc_dvdd_voltage = rb_vcp.adc_dvdd_vcp[0] ;
    self.adc_dvdd_current = rb_vcp.adc_dvdd_vcp[1] ;
    self.adc_dvdd_power   = rb_vcp.adc_dvdd_vcp[2] ;
    self.adc_avdd_voltage = rb_vcp.adc_avdd_vcp[0]  ;
    self.adc_avdd_current = rb_vcp.adc_avdd_vcp[1]  ;
    self.adc_avdd_power   = rb_vcp.adc_avdd_vcp[2]  ;
    self.drs_avdd_voltage = rb_vcp.drs_avdd_vcp[0]  ;
    self.drs_avdd_current = rb_vcp.drs_avdd_vcp[1]  ;
    self.drs_avdd_power   = rb_vcp.drs_avdd_vcp[2]  ;
    self.n1v5_voltage     = rb_vcp.n1v5_vcp[0]      ;
    self.n1v5_current     = rb_vcp.n1v5_vcp[1]      ;
    self.n1v5_power       = rb_vcp.n1v5_vcp[2]      ;
  }
  
  #[cfg(feature = "tofcontrol")] 
  pub fn add_rbph(&mut self, rb_ph   : &RBPh) {
    self.pressure = rb_ph.pressure;
    self.humidity = rb_ph.humidity;
  }
  #[cfg(feature = "tofcontrol")]
  pub fn add_rbtemp(&mut self, rb_temp : &RBTemp) {
    self.tmp_drs         = rb_temp.drs_temp      ; 
    self.tmp_clk         = rb_temp.clk_temp      ; 
    self.tmp_adc         = rb_temp.adc_temp      ; 
    self.tmp_zynq        = rb_temp.zynq_temp     ; 
    self.tmp_lis3mdltr   = rb_temp.lis3mdltr_temp; 
    self.tmp_bm280       = rb_temp.bme280_temp   ; 
  }

  #[cfg(feature = "tofcontrol")] 
  pub fn add_rbmag(&mut self, rb_mag   : &RBMag) {
    self.mag_x   = rb_mag.mag_xyz[0];
    self.mag_y   = rb_mag.mag_xyz[1];
    self.mag_z   = rb_mag.mag_xyz[2];
  }
 
  pub fn get_mag_tot(&self) -> f32 {
    (self.mag_x.powi(2) + self.mag_y.powi(2) + self.mag_z.powi(2)).sqrt()
  }
}

impl Default for RBMoniData {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for RBMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<RBMoniData:
  BOARD ID       {}
  RATE           {}    [Hz] 
  ** Temperatures **
  DRS TMP        {:.3} [\u{00B0}C]
  CLK TMP        {:.3} [\u{00B0}C]
  ADC TMP        {:.3} [\u{00B0}C]
  ZYNQ TMP       {:.3} [\u{00B0}C]
  LIS3MDLTR TMP  {:.3} [\u{00B0}C]  
  BM280 TMP      {:.3} [\u{00B0}C]
  ** Ambience **
  PRESSURE       {:.3} [hPa]
  HUMIDITY       {:.3} [%]
  MAG_X , MAG_Y, MAG_Z, MAG_TOT:
   |->  {:.3} [G] | {:.3} [G] | {:.3} [G] | {:.3} [G]
  ** Power **
  ZYNQ 3.3V         Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
  3.3V              Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
  3.5V              Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
  -1.5V             Power: {:.3}  [V] | {:.3} [A] | {:.3} [W]
  DRS4 Digital 2.5V Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
  DRS4 Analog 2.5V  Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
  ADC Digital 2.5V  Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
  ADC Analog 3.0V   Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]>",
           self.board_id        , 
           self.rate            ,
           self.tmp_drs         ,
           self.tmp_clk         ,
           self.tmp_adc         ,
           self.tmp_zynq        ,
           self.tmp_lis3mdltr   ,
           self.tmp_bm280       ,
           self.pressure        ,
           self.humidity        ,
           self.mag_x           ,
           self.mag_y           ,
           self.mag_z           ,
           self.get_mag_tot()   ,
           self.zynq_voltage    ,
           self.zynq_current    ,
           self.zynq_power      ,
           self.p3v3_voltage    ,
           self.p3v3_current    ,
           self.p3v3_power      ,
           self.p3v5_voltage    , 
           self.p3v5_current    ,
           self.p3v5_power      ,
           self.n1v5_voltage    ,
           self.n1v5_current    ,
           self.n1v5_power      ,
           self.drs_dvdd_voltage, 
           self.drs_dvdd_current,
           self.drs_dvdd_power  ,
           self.drs_avdd_voltage, 
           self.drs_avdd_current,
           self.drs_avdd_power  ,
           self.adc_dvdd_voltage,
           self.adc_dvdd_current,
           self.adc_dvdd_power  ,
           self.adc_avdd_voltage,
           self.adc_avdd_current,
           self.adc_avdd_power  )
  }
}

impl MoniData for RBMoniData {

  fn get_board_id(&self) -> u8 {
    self.board_id
  }
  
  /// Access the (data) members by name 
  fn get(&self, varname : &str) -> Option<f32> {
    match varname {
      "board_id"         => Some(self.board_id as   f32),
      "rate"             => Some(self.rate as f32      ), 
      "tmp_drs"          => Some(self.tmp_drs          ), 
      "tmp_clk"          => Some(self.tmp_clk          ), 
      "tmp_adc"          => Some(self.tmp_adc          ), 
      "tmp_zynq"         => Some(self.tmp_zynq         ), 
      "tmp_lis3mdltr"    => Some(self.tmp_lis3mdltr    ), 
      "tmp_bm280"        => Some(self.tmp_bm280        ), 
      "pressure"         => Some(self.pressure         ), 
      "humidity"         => Some(self.humidity         ), 
      "mag_x"            => Some(self.mag_x            ), 
      "mag_y"            => Some(self.mag_y            ), 
      "mag_z"            => Some(self.mag_z            ), 
      "mag_tot"          => Some(self.get_mag_tot()    ),
      "drs_dvdd_voltage" => Some(self.drs_dvdd_voltage ), 
      "drs_dvdd_current" => Some(self.drs_dvdd_current ), 
      "drs_dvdd_power"   => Some(self.drs_dvdd_power   ), 
      "p3v3_voltage"     => Some(self.p3v3_voltage     ), 
      "p3v3_current"     => Some(self.p3v3_current     ), 
      "p3v3_power"       => Some(self.p3v3_power       ), 
      "zynq_voltage"     => Some(self.zynq_voltage     ), 
      "zynq_current"     => Some(self.zynq_current     ), 
      "zynq_power"       => Some(self.zynq_power       ), 
      "p3v5_voltage"     => Some(self.p3v5_voltage     ), 
      "p3v5_current"     => Some(self.p3v5_current     ), 
      "p3v5_power"       => Some(self.p3v5_power       ), 
      "adc_dvdd_voltage" => Some(self.adc_dvdd_voltage ), 
      "adc_dvdd_current" => Some(self.adc_dvdd_current ), 
      "adc_dvdd_power"   => Some(self.adc_dvdd_power   ), 
      "adc_avdd_voltage" => Some(self.adc_avdd_voltage ), 
      "adc_avdd_current" => Some(self.adc_avdd_current ), 
      "adc_avdd_power"   => Some(self.adc_avdd_power   ), 
      "drs_avdd_voltage" => Some(self.drs_avdd_voltage ), 
      "drs_avdd_current" => Some(self.drs_avdd_current ), 
      "drs_avdd_power"   => Some(self.drs_avdd_power   ), 
      "n1v5_voltage"     => Some(self.n1v5_voltage     ), 
      "n1v5_current"     => Some(self.n1v5_current     ), 
      "n1v5_power"       => Some(self.n1v5_power       ), 
      _             => None
    }
  }

  /// A list of the variables in this MoniData
  fn keys() -> Vec<&'static str> {
    vec![
      "board_id"        , 
      "rate"            , 
      "tmp_drs"         , 
      "tmp_clk"         , 
      "tmp_adc"         , 
      "tmp_zynq"        , 
      "tmp_lis3mdltr"   , 
      "tmp_bm280"       , 
      "pressure"        , 
      "humidity"        , 
      "mag_x"           , 
      "mag_y"           , 
      "mag_z"           , 
      "drs_dvdd_voltage", 
      "drs_dvdd_current", 
      "drs_dvdd_power"  , 
      "p3v3_voltage"    , 
      "p3v3_current"    , 
      "p3v3_power"      , 
      "zynq_voltage"    , 
      "zynq_current"    , 
      "zynq_power"      , 
      "p3v5_voltage"    , 
      "p3v5_current"    , 
      "p3v5_power"      , 
      "adc_dvdd_voltage", 
      "adc_dvdd_current", 
      "adc_dvdd_power"  , 
      "adc_avdd_voltage", 
      "adc_avdd_current", 
      "adc_avdd_power"  , 
      "drs_avdd_voltage", 
      "drs_avdd_current", 
      "drs_avdd_power"  , 
      "n1v5_voltage"    , 
      "n1v5_current"    , 
      "n1v5_power"      ] 
  }
}

impl TofPackable for RBMoniData {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::RBMoniData;
}

impl Serialization for RBMoniData {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  /// The data size when serialized to a bytestream
  /// This needs to be updated when we change the 
  /// packet layout, e.g. add new members.
  /// HEAD + TAIL + sum(sizeof(m) for m in _all_members_))
  const SIZE : usize  = 7 + (36*4) ;
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.board_id          .to_le_bytes()); 
    stream.extend_from_slice(&self.rate              .to_le_bytes()); 
    stream.extend_from_slice(&self.tmp_drs           .to_le_bytes()); 
    stream.extend_from_slice(&self.tmp_clk           .to_le_bytes()); 
    stream.extend_from_slice(&self.tmp_adc           .to_le_bytes()); 
    stream.extend_from_slice(&self.tmp_zynq          .to_le_bytes()); 
    stream.extend_from_slice(&self.tmp_lis3mdltr     .to_le_bytes()); 
    stream.extend_from_slice(&self.tmp_bm280         .to_le_bytes()); 
    stream.extend_from_slice(&self.pressure          .to_le_bytes()); 
    stream.extend_from_slice(&self.humidity          .to_le_bytes()); 
    stream.extend_from_slice(&self.mag_x             .to_le_bytes()); 
    stream.extend_from_slice(&self.mag_y             .to_le_bytes()); 
    stream.extend_from_slice(&self.mag_z             .to_le_bytes());
    // padding - just for compatibility
    stream.extend_from_slice(&0.0_f32                 .to_le_bytes());
    stream.extend_from_slice(&self.drs_dvdd_voltage   .to_le_bytes()); 
    stream.extend_from_slice(&self.drs_dvdd_current   .to_le_bytes()); 
    stream.extend_from_slice(&self.drs_dvdd_power     .to_le_bytes()); 
    stream.extend_from_slice(&self.p3v3_voltage       .to_le_bytes()); 
    stream.extend_from_slice(&self.p3v3_current       .to_le_bytes()); 
    stream.extend_from_slice(&self.p3v3_power         .to_le_bytes()); 
    stream.extend_from_slice(&self.zynq_voltage       .to_le_bytes()); 
    stream.extend_from_slice(&self.zynq_current       .to_le_bytes()); 
    stream.extend_from_slice(&self.zynq_power         .to_le_bytes()); 
    stream.extend_from_slice(&self.p3v5_voltage       .to_le_bytes()); 
    stream.extend_from_slice(&self.p3v5_current       .to_le_bytes()); 
    stream.extend_from_slice(&self.p3v5_power         .to_le_bytes()); 
    stream.extend_from_slice(&self.adc_dvdd_voltage   .to_le_bytes()); 
    stream.extend_from_slice(&self.adc_dvdd_current   .to_le_bytes()); 
    stream.extend_from_slice(&self.adc_dvdd_power     .to_le_bytes()); 
    stream.extend_from_slice(&self.adc_avdd_voltage   .to_le_bytes()); 
    stream.extend_from_slice(&self.adc_avdd_current   .to_le_bytes()); 
    stream.extend_from_slice(&self.adc_avdd_power     .to_le_bytes()); 
    stream.extend_from_slice(&self.drs_avdd_voltage   .to_le_bytes()); 
    stream.extend_from_slice(&self.drs_avdd_current   .to_le_bytes()); 
    stream.extend_from_slice(&self.drs_avdd_power     .to_le_bytes()); 
    stream.extend_from_slice(&self.n1v5_voltage       .to_le_bytes()); 
    stream.extend_from_slice(&self.n1v5_current       .to_le_bytes()); 
    stream.extend_from_slice(&self.n1v5_power         .to_le_bytes()); 
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }

  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<RBMoniData, SerializationError>{
    let mut moni_data = Self::new();
    Self::verify_fixed(stream, pos)?;
    moni_data.board_id           = parse_u8( stream, pos); 
    moni_data.rate               = parse_u16(stream, pos); 
    moni_data.tmp_drs            = parse_f32(stream, pos); 
    moni_data.tmp_clk            = parse_f32(stream, pos); 
    moni_data.tmp_adc            = parse_f32(stream, pos); 
    moni_data.tmp_zynq           = parse_f32(stream, pos); 
    moni_data.tmp_lis3mdltr      = parse_f32(stream, pos); 
    moni_data.tmp_bm280          = parse_f32(stream, pos); 
    moni_data.pressure           = parse_f32(stream, pos); 
    moni_data.humidity           = parse_f32(stream, pos); 
    moni_data.mag_x              = parse_f32(stream, pos); 
    moni_data.mag_y              = parse_f32(stream, pos); 
    moni_data.mag_z              = parse_f32(stream, pos); 
    // compatibility, no mag_tot anymore
    *pos += 4;
    moni_data.drs_dvdd_voltage   = parse_f32(stream, pos); 
    moni_data.drs_dvdd_current   = parse_f32(stream, pos); 
    moni_data.drs_dvdd_power     = parse_f32(stream, pos); 
    moni_data.p3v3_voltage       = parse_f32(stream, pos); 
    moni_data.p3v3_current       = parse_f32(stream, pos); 
    moni_data.p3v3_power         = parse_f32(stream, pos); 
    moni_data.zynq_voltage       = parse_f32(stream, pos); 
    moni_data.zynq_current       = parse_f32(stream, pos); 
    moni_data.zynq_power         = parse_f32(stream, pos); 
    moni_data.p3v5_voltage       = parse_f32(stream, pos); 
    moni_data.p3v5_current       = parse_f32(stream, pos); 
    moni_data.p3v5_power         = parse_f32(stream, pos); 
    moni_data.adc_dvdd_voltage   = parse_f32(stream, pos); 
    moni_data.adc_dvdd_current   = parse_f32(stream, pos); 
    moni_data.adc_dvdd_power     = parse_f32(stream, pos); 
    moni_data.adc_avdd_voltage   = parse_f32(stream, pos); 
    moni_data.adc_avdd_current   = parse_f32(stream, pos); 
    moni_data.adc_avdd_power     = parse_f32(stream, pos); 
    moni_data.drs_avdd_voltage   = parse_f32(stream, pos); 
    moni_data.drs_avdd_current   = parse_f32(stream, pos); 
    moni_data.drs_avdd_power     = parse_f32(stream, pos); 
    moni_data.n1v5_voltage       = parse_f32(stream, pos); 
    moni_data.n1v5_current       = parse_f32(stream, pos); 
    moni_data.n1v5_power         = parse_f32(stream, pos); 
    *pos += 2; // for tail
    Ok(moni_data) 
  }
}

#[cfg(feature = "random")]
impl FromRandom for RBMoniData {
    
  fn from_random() -> RBMoniData {
    let mut moni = RBMoniData::new();
    let mut rng = rand::rng();
    moni.board_id           = rng.random::<u8>(); 
    moni.rate               = rng.random::<u16>();
    moni.tmp_drs            = rng.random::<f32>();
    moni.tmp_clk            = rng.random::<f32>();
    moni.tmp_adc            = rng.random::<f32>();
    moni.tmp_zynq           = rng.random::<f32>();
    moni.tmp_lis3mdltr      = rng.random::<f32>();
    moni.tmp_bm280          = rng.random::<f32>();
    moni.pressure           = rng.random::<f32>();
    moni.humidity           = rng.random::<f32>();
    moni.mag_x              = rng.random::<f32>();
    moni.mag_y              = rng.random::<f32>();
    moni.mag_z              = rng.random::<f32>();
    moni.drs_dvdd_voltage   = rng.random::<f32>(); 
    moni.drs_dvdd_current   = rng.random::<f32>();
    moni.drs_dvdd_power     = rng.random::<f32>();
    moni.p3v3_voltage       = rng.random::<f32>();
    moni.p3v3_current       = rng.random::<f32>();
    moni.p3v3_power         = rng.random::<f32>();
    moni.zynq_voltage       = rng.random::<f32>();
    moni.zynq_current       = rng.random::<f32>();
    moni.zynq_power         = rng.random::<f32>();
    moni.p3v5_voltage       = rng.random::<f32>(); 
    moni.p3v5_current       = rng.random::<f32>();
    moni.p3v5_power         = rng.random::<f32>();
    moni.adc_dvdd_voltage   = rng.random::<f32>();
    moni.adc_dvdd_current   = rng.random::<f32>();
    moni.adc_dvdd_power     = rng.random::<f32>();
    moni.adc_avdd_voltage   = rng.random::<f32>();
    moni.adc_avdd_current   = rng.random::<f32>();
    moni.adc_avdd_power     = rng.random::<f32>();
    moni.drs_avdd_voltage   = rng.random::<f32>(); 
    moni.drs_avdd_current   = rng.random::<f32>();
    moni.drs_avdd_power     = rng.random::<f32>();
    moni.n1v5_voltage       = rng.random::<f32>();
    moni.n1v5_current       = rng.random::<f32>();
    moni.n1v5_power         = rng.random::<f32>();
    moni
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl RBMoniData {
  
  #[getter]
  #[pyo3(name="mag_tot")]
  fn get_mag_tot_py(&self) -> f32 {
    self.get_mag_tot()
  }

  #[getter]
  fn get_rate             (&self) -> u16 {
    self.rate
  }
  
  #[getter]
  fn get_tmp_drs          (&self) -> f32 {
    self.tmp_drs
  }
  
  #[getter]
  fn get_tmp_clk          (&self) -> f32 {
    self.tmp_clk
  }
  

  #[getter]
  fn get_tmp_adc          (&self) -> f32 {
    self.tmp_adc
  }

  #[getter]
  fn get_tmp_zynq         (&self) -> f32 {
    self.tmp_zynq
  }

  #[getter]
  fn get_tmp_lis3mdltr    (&self) -> f32 {
    self.tmp_lis3mdltr
  }

  #[getter]
  fn get_tmp_bm280        (&self) -> f32 {
    self.tmp_bm280
  }
  
  #[getter]
  fn get_pressure         (&self) -> f32 {
    self.pressure
  }

  #[getter]
  fn get_humidity         (&self) -> f32 {
    self.humidity
  }

  #[getter]
  fn get_mag_x            (&self) -> f32 {
    self.mag_x
  }
  
  #[getter]
  fn get_mag_y            (&self) -> f32 {
    self.mag_y
  }
  
  #[getter]
  fn get_mag_z            (&self) -> f32 {
    self.mag_z
  }
  
  #[getter]
  fn get_drs_dvdd_voltage (&self) -> f32 { 
    self.drs_dvdd_voltage
  }

  #[getter]
  fn get_drs_dvdd_current (&self) -> f32 {
    self.drs_dvdd_current
  }

  #[getter]
  fn get_drs_dvdd_power   (&self) -> f32 {
    self.drs_dvdd_power
  }

  #[getter]
  fn get_p3v3_voltage     (&self) -> f32 {
    self.p3v3_voltage
  }

  #[getter]
  fn get_p3v3_current     (&self) -> f32 {
    self.p3v3_current
  }

  #[getter]
  fn get_p3v3_power       (&self) -> f32 {
    self.p3v3_power
  }

  #[getter]
  fn get_zynq_voltage     (&self) -> f32 {
    self.zynq_voltage
  }

  #[getter]
  fn get_zynq_current     (&self) -> f32 {
    self.zynq_current
  }

  #[getter]
  fn get_zynq_power       (&self) -> f32 {
    self.zynq_power
  }

  #[getter]
  fn get_p3v5_voltage     (&self) -> f32 { 
    self.p3v5_voltage
  }

  #[getter]
  fn get_p3v5_current     (&self) -> f32 {
    self.p3v5_current
  }

  #[getter]
  fn get_p3v5_power       (&self) -> f32 {
    self.p3v5_power
  }

  #[getter]
  fn get_adc_dvdd_voltage (&self) -> f32 {
    self.adc_dvdd_voltage
  }
  
  #[getter]
  fn get_adc_dvdd_current (&self) -> f32 {
    self.adc_dvdd_current
  }
  
  #[getter]
  fn get_adc_dvdd_power   (&self) -> f32 {
    self.adc_dvdd_power
  }
  
  #[getter]
  fn get_adc_avdd_voltage (&self) -> f32 {
    self.adc_avdd_voltage
  }
  
  #[getter]
  fn get_adc_avdd_current (&self) -> f32 {
    self.adc_avdd_current
  }
  
  #[getter]
  fn get_adc_avdd_power   (&self) -> f32 {
    self.adc_avdd_power
  }
  
  #[getter]
  fn get_drs_avdd_voltage (&self) -> f32 { 
    self.drs_avdd_voltage
  }
  
  #[getter]
  fn get_drs_avdd_current (&self) -> f32 {
    self.drs_avdd_current
  }
  
  #[getter]
  fn get_drs_avdd_power   (&self) -> f32 {
    self.drs_avdd_power
  }
  
  #[getter]
  fn get_n1v5_voltage     (&self) -> f32 {
    self.n1v5_voltage
  }
  
  #[getter]
  fn get_n1v5_current     (&self) -> f32 {
    self.n1v5_current
  }
  
  #[getter]
  fn get_n1v5_power       (&self) -> f32 {
    self.n1v5_power
  }
}

//----------------------------------------

moniseries!(RBMoniDataSeries, RBMoniData);

#[cfg(feature="pybindings")]
pythonize_packable!(RBMoniData);

#[cfg(feature="pybindings")]
pythonize_monidata!(RBMoniData);

//-----------------------------------

#[test]
#[cfg(feature = "random")]
fn monidata_rbmonidata() {
  let data = RBMoniData::from_random();
  for k in RBMoniData::keys() {
    assert!(data.get(k).is_some());
  }
  assert_eq!(data.get_board_id(), data.board_id);
}

