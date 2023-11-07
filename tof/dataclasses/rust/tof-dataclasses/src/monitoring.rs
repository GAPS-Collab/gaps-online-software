//! Structures for monitoring
//!
//! This is 
//! a) Monitoring the RBs
//! b) Monitoring the tof-computer/main C&C instance
//! c) Monitoring the MTB
//!
//!

use std::fmt;

// Takeru's tof-control code
#[cfg(feature = "tof-control")]
use tof_control::rb_control::rb_temp::RBtemp;
#[cfg(feature = "tof-control")]
use tof_control::rb_control::rb_mag::RBmag;
#[cfg(feature = "tof-control")]
use tof_control::rb_control::rb_vcp::RBvcp;
#[cfg(feature = "tof-control")]
use tof_control::rb_control::rb_ph::RBph;

#[cfg(feature = "random")]
use crate::FromRandom;
#[cfg(feature = "random")]
extern crate rand;
#[cfg(feature = "random")]
use rand::Rng;


use crate::serialization::{Serialization,
                           SerializationError,
                           search_for_u16,
                           parse_u8,
                           parse_u16,
                           parse_f32};

/// A collection of monitoring data
/// from the readoutboards. This includes
/// temperatures, power data, pressure, humidity
/// as well as the magnetic sensors
#[derive(Debug, Copy, Clone, PartialEq)]
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
  pub mag_tot            : f32,
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

  #[cfg(feature = "tof-control")]
  pub fn add_rbtemp(&mut self, rb_temp : &RBtemp) {
    self.tmp_drs         = rb_temp.drs_temp      ; 
    self.tmp_clk         = rb_temp.clk_temp      ; 
    self.tmp_adc         = rb_temp.adc_temp      ; 
    self.tmp_zynq        = rb_temp.zynq_temp     ; 
    self.tmp_lis3mdltr   = rb_temp.lis3mdltr_temp; 
    self.tmp_bm280       = rb_temp.bme280_temp   ; 
  }

  #[cfg(feature = "tof-control")] 
  pub fn add_rbmag(&mut self, rb_mag   : &RBmag) {
    self.mag_x   = rb_mag.magnetic_x;
    self.mag_y   = rb_mag.magnetic_y;
    self.mag_z   = rb_mag.magnetic_z;
    self.mag_tot = rb_mag.magnetic_t;
  }
  
  #[cfg(feature = "tof-control")]
  pub fn add_rbvcp(&mut self, rb_vcp   : &RBvcp) {
    self.drs_dvdd_voltage = rb_vcp.drs_dvdd_voltage ;
    self.drs_dvdd_current = rb_vcp.drs_dvdd_current ;
    self.drs_dvdd_power   = rb_vcp.drs_dvdd_power   ;
    self.p3v3_voltage     = rb_vcp.p3v3_voltage     ;
    self.p3v3_current     = rb_vcp.p3v3_current     ;
    self.p3v3_power       = rb_vcp.p3v3_power       ;
    self.zynq_voltage     = rb_vcp.zynq_voltage     ;
    self.zynq_current     = rb_vcp.zynq_current     ;
    self.zynq_power       = rb_vcp.zynq_power       ;
    self.p3v5_voltage     = rb_vcp.p3v5_voltage     ;
    self.p3v5_current     = rb_vcp.p3v5_current     ;
    self.p3v5_power       = rb_vcp.p3v5_power       ;
    self.adc_dvdd_voltage = rb_vcp.adc_dvdd_voltage ;
    self.adc_dvdd_current = rb_vcp.adc_dvdd_current ;
    self.adc_dvdd_power   = rb_vcp.adc_dvdd_power   ;
    self.adc_avdd_voltage = rb_vcp.adc_avdd_voltage ;
    self.adc_avdd_current = rb_vcp.adc_avdd_current ;
    self.adc_avdd_power   = rb_vcp.adc_avdd_power   ;
    self.drs_avdd_voltage = rb_vcp.drs_avdd_voltage ;
    self.drs_avdd_current = rb_vcp.drs_avdd_current ;
    self.drs_avdd_power   = rb_vcp.drs_avdd_power   ;
    self.n1v5_voltage     = rb_vcp.n1v5_voltage     ;
    self.n1v5_current     = rb_vcp.n1v5_current     ;
    self.n1v5_power       = rb_vcp.n1v5_power       ;
  }
  
  #[cfg(feature = "tof-control")] 
  pub fn add_rbph(&mut self, rb_ph   : &RBph) {
    self.pressure = rb_ph.pressure;
    self.humidity = rb_ph.humidity;
  }

  pub fn new() -> Self {
    Self {
      board_id           : 0, 
      rate               : 0,
      tmp_drs            : 0.0,
      tmp_clk            : 0.0,
      tmp_adc            : 0.0,
      tmp_zynq           : 0.0,
      tmp_lis3mdltr      : 0.0,
      tmp_bm280          : 0.0,
      pressure           : 0.0,
      humidity           : 0.0,
      mag_x              : 0.0,
      mag_y              : 0.0,
      mag_z              : 0.0,
      mag_tot            : 0.0,
      drs_dvdd_voltage   : 0.0, 
      drs_dvdd_current   : 0.0,
      drs_dvdd_power     : 0.0,
      p3v3_voltage       : 0.0,
      p3v3_current       : 0.0,
      p3v3_power         : 0.0,
      zynq_voltage       : 0.0,
      zynq_current       : 0.0,
      zynq_power         : 0.0,
      p3v5_voltage       : 0.0, 
      p3v5_current       : 0.0,
      p3v5_power         : 0.0,
      adc_dvdd_voltage   : 0.0,
      adc_dvdd_current   : 0.0,
      adc_dvdd_power     : 0.0,
      adc_avdd_voltage   : 0.0,
      adc_avdd_current   : 0.0,
      adc_avdd_power     : 0.0,
      drs_avdd_voltage   : 0.0, 
      drs_avdd_current   : 0.0,
      drs_avdd_power     : 0.0,
      n1v5_voltage       : 0.0,
      n1v5_current       : 0.0,
      n1v5_power         : 0.0,
    }
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
           \t BOARD ID {}
           \t RATE     {}    [Hz] 
           \t DRS TMP  {:.3} [C]
           \t CLK TMP  {:.3} [C]
           \t ADC TMP  {:.3} [C]
           \t ZYNQ TMP {:.3} [C]
           \t LIS3MDLTR TMP  {:.3} [C]  
           \t BM280 TMP      {:.3} [C]
           \t PRESSURE      {:.3} [hPa]
           \t HUMIDITY       {:.3} [%]
           \t MAG_X , MAG_Y, MAG_Z, MAG_TOT: {:.3} [G] | {:.3} [G] | {:.3} [G] | {:.3} [G]
           \t ZYNQ 3.3V         Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
           \t 3.3V              Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
           \t 3.5V              Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
           \t -1.5V             Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
           \t DRS4 Digital 2.5V Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
           \t DRS4 Analog 2.5V  Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
           \t ADC Digital 2.5V  Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]
           \t ADC Analog 3.0V   Power:  {:.3}  [V] | {:.3} [A] | {:.3} [W]>",
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
           self.mag_tot         ,
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

#[cfg(feature = "random")]
impl FromRandom for RBMoniData {
    
  fn from_random() -> RBMoniData {
    let mut moni = RBMoniData::new();
    let mut rng = rand::thread_rng();
    moni.board_id           = rng.gen::<u8>(); 
    moni.rate               = rng.gen::<u16>();
    moni.tmp_drs            = rng.gen::<f32>();
    moni.tmp_clk            = rng.gen::<f32>();
    moni.tmp_adc            = rng.gen::<f32>();
    moni.tmp_zynq           = rng.gen::<f32>();
    moni.tmp_lis3mdltr      = rng.gen::<f32>();
    moni.tmp_bm280          = rng.gen::<f32>();
    moni.pressure           = rng.gen::<f32>();
    moni.humidity           = rng.gen::<f32>();
    moni.mag_x              = rng.gen::<f32>();
    moni.mag_y              = rng.gen::<f32>();
    moni.mag_z              = rng.gen::<f32>();
    moni.mag_tot            = rng.gen::<f32>();
    moni.drs_dvdd_voltage   = rng.gen::<f32>(); 
    moni.drs_dvdd_current   = rng.gen::<f32>();
    moni.drs_dvdd_power     = rng.gen::<f32>();
    moni.p3v3_voltage       = rng.gen::<f32>();
    moni.p3v3_current       = rng.gen::<f32>();
    moni.p3v3_power         = rng.gen::<f32>();
    moni.zynq_voltage       = rng.gen::<f32>();
    moni.zynq_current       = rng.gen::<f32>();
    moni.zynq_power         = rng.gen::<f32>();
    moni.p3v5_voltage       = rng.gen::<f32>(); 
    moni.p3v5_current       = rng.gen::<f32>();
    moni.p3v5_power         = rng.gen::<f32>();
    moni.adc_dvdd_voltage   = rng.gen::<f32>();
    moni.adc_dvdd_current   = rng.gen::<f32>();
    moni.adc_dvdd_power     = rng.gen::<f32>();
    moni.adc_avdd_voltage   = rng.gen::<f32>();
    moni.adc_avdd_current   = rng.gen::<f32>();
    moni.adc_avdd_power     = rng.gen::<f32>();
    moni.drs_avdd_voltage   = rng.gen::<f32>(); 
    moni.drs_avdd_current   = rng.gen::<f32>();
    moni.drs_avdd_power     = rng.gen::<f32>();
    moni.n1v5_voltage       = rng.gen::<f32>();
    moni.n1v5_current       = rng.gen::<f32>();
    moni.n1v5_power         = rng.gen::<f32>();
    moni
  }
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
    let mut stream = Vec::<u8>::with_capacity(RBMoniData::SIZE);
    stream.extend_from_slice(&RBMoniData::HEAD.to_le_bytes());
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
    stream.extend_from_slice(&self.mag_tot           .to_le_bytes()); 
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
    stream.extend_from_slice(&RBMoniData::TAIL.to_le_bytes());
    stream
  }

  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<RBMoniData, SerializationError>{
    let mut moni_data = Self::new();
    //println!("{:?}", stream);
    let head_pos = search_for_u16(Self::HEAD, stream, *pos)?; 
    let tail_pos = search_for_u16(Self::TAIL, stream, head_pos + RBMoniData::SIZE-2)?;
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    if tail_pos + 2 - head_pos != Self::SIZE {
      error!("RBMoniData. Seing {} bytes, but expecting {}", tail_pos + 2 - head_pos, Self::SIZE);
      *pos = head_pos + 2; 
      return Err(SerializationError::WrongByteSize);
    }
    *pos = head_pos + 2;
    moni_data.board_id           = parse_u8(&stream, pos); 
    moni_data.rate               = parse_u16(&stream, pos); 
    moni_data.tmp_drs            = parse_f32(&stream, pos); 
    moni_data.tmp_clk            = parse_f32(&stream, pos); 
    moni_data.tmp_adc            = parse_f32(&stream, pos); 
    moni_data.tmp_zynq           = parse_f32(&stream, pos); 
    moni_data.tmp_lis3mdltr      = parse_f32(&stream, pos); 
    moni_data.tmp_bm280          = parse_f32(&stream, pos); 
    moni_data.pressure           = parse_f32(&stream, pos); 
    moni_data.humidity           = parse_f32(&stream, pos); 
    moni_data.mag_x              = parse_f32(&stream, pos); 
    moni_data.mag_y              = parse_f32(&stream, pos); 
    moni_data.mag_z              = parse_f32(&stream, pos); 
    moni_data.mag_tot            = parse_f32(&stream, pos); 
    moni_data.drs_dvdd_voltage   = parse_f32(&stream, pos); 
    moni_data.drs_dvdd_current   = parse_f32(&stream, pos); 
    moni_data.drs_dvdd_power     = parse_f32(&stream, pos); 
    moni_data.p3v3_voltage       = parse_f32(&stream, pos); 
    moni_data.p3v3_current       = parse_f32(&stream, pos); 
    moni_data.p3v3_power         = parse_f32(&stream, pos); 
    moni_data.zynq_voltage       = parse_f32(&stream, pos); 
    moni_data.zynq_current       = parse_f32(&stream, pos); 
    moni_data.zynq_power         = parse_f32(&stream, pos); 
    moni_data.p3v5_voltage       = parse_f32(&stream, pos); 
    moni_data.p3v5_current       = parse_f32(&stream, pos); 
    moni_data.p3v5_power         = parse_f32(&stream, pos); 
    moni_data.adc_dvdd_voltage   = parse_f32(&stream, pos); 
    moni_data.adc_dvdd_current   = parse_f32(&stream, pos); 
    moni_data.adc_dvdd_power     = parse_f32(&stream, pos); 
    moni_data.adc_avdd_voltage   = parse_f32(&stream, pos); 
    moni_data.adc_avdd_current   = parse_f32(&stream, pos); 
    moni_data.adc_avdd_power     = parse_f32(&stream, pos); 
    moni_data.drs_avdd_voltage   = parse_f32(&stream, pos); 
    moni_data.drs_avdd_current   = parse_f32(&stream, pos); 
    moni_data.drs_avdd_power     = parse_f32(&stream, pos); 
    moni_data.n1v5_voltage       = parse_f32(&stream, pos); 
    moni_data.n1v5_current       = parse_f32(&stream, pos); 
    moni_data.n1v5_power         = parse_f32(&stream, pos); 
    *pos += 2;
    Ok(moni_data) 
  }
}

/// Monitoring the main tof computer
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TofCmpMoniData {
  pub core1_tmp : u8,
  pub core2_tmp : u8,
  pub pch_tmp   : u8
}

impl TofCmpMoniData {
  
  pub fn new() -> TofCmpMoniData {
    TofCmpMoniData {
      core1_tmp : 0,
      core2_tmp : 0,
      pch_tmp   : 0
    }
  }
}

impl Default for TofCmpMoniData {
  fn default() -> TofCmpMoniData {
    TofCmpMoniData::new()
  }
}

impl fmt::Display for TofCmpMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<TofCmpMoniData:
           \t CORE1 TMP {}: [C]
           \t CORE2 TMP {}: [C]
           \t PCH   TMP {}: [C]>",
           self.core1_tmp, self.core2_tmp, self.pch_tmp)
  }
}

impl Serialization for TofCmpMoniData {
  
  const SIZE : usize = 7;
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(TofCmpMoniData::SIZE);
    stream.extend_from_slice(&TofCmpMoniData::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.core1_tmp  .to_le_bytes());
    stream.extend_from_slice(&self.core2_tmp  .to_le_bytes());
    stream.extend_from_slice(&self.pch_tmp    .to_le_bytes());
    stream.extend_from_slice(&TofCmpMoniData::TAIL.to_le_bytes());
    stream
  }

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<TofCmpMoniData, SerializationError> {
    let mut moni_data = TofCmpMoniData::new();
    let head_pos = search_for_u16(TofCmpMoniData::HEAD, stream, *pos)?; 
    let tail_pos = search_for_u16(TofCmpMoniData::TAIL, stream, head_pos + TofCmpMoniData::SIZE-2)?;
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    if tail_pos + 2 - head_pos != TofCmpMoniData::SIZE {
      error!("TofCmpMoniData incomplete. Seing {} bytes, but expecting {}", tail_pos + 2 - head_pos, TofCmpMoniData::SIZE);
      //error!("{:?}", &stream[head_pos + 18526..head_pos + 18540]);
      *pos = head_pos + 2; //start_pos += RBBinaryDump::SIZE;
      return Err(SerializationError::WrongByteSize);
    }
    *pos = head_pos + 2; 
    moni_data.core1_tmp  = parse_u8(&stream, pos);
    moni_data.core2_tmp  = parse_u8(&stream, pos);
    moni_data.pch_tmp    = parse_u8(&stream, pos);
    *pos += 2; // since we deserialized the tail earlier and 
              // didn't account for it
    Ok(moni_data)
  }
}

/// Monitoring the MTB
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MtbMoniData {
  pub fpga_temp    : f32,
  pub fpga_vccint  : f32,
  pub fpga_vccaux  : f32,
  pub fpga_vccbram : f32,
  pub rate         : u16,
  pub lost_rate    : u16
}

impl MtbMoniData {
  
  pub fn new() -> Self {
    Self {
      fpga_temp    : f32::MAX,
      fpga_vccint  : f32::MAX,
      fpga_vccaux  : f32::MAX,
      fpga_vccbram : f32::MAX,
      rate         : u16::MAX,
      lost_rate    : u16::MAX
    }
  }
}

impl Default for MtbMoniData {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for MtbMoniData {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<MtbMoniData:
           \t MTB  EVT RATE [Hz] {}
           \t LOST EVT RATE [Hz] {}
           \t FPGA TMP      [C]  {}
           \t FPGA VCCINT   [V]  {}
           \t FPGA VCCAUX   [V]  {}
           \t FPGA VCCBRAM  [V]  {}>",
           self.rate,
           self.lost_rate,
           self.fpga_vccint,
           self.fpga_vccaux,
           self.fpga_vccbram,
           self.fpga_temp)
  }
}

impl Serialization for MtbMoniData {
  
  const SIZE : usize = 24;
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(Self::SIZE);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.fpga_temp   .to_le_bytes());
    stream.extend_from_slice(&self.fpga_vccint .to_le_bytes());
    stream.extend_from_slice(&self.fpga_vccaux .to_le_bytes());
    stream.extend_from_slice(&self.fpga_vccbram.to_le_bytes());
    stream.extend_from_slice(&self.rate        .to_le_bytes());
    stream.extend_from_slice(&self.lost_rate   .to_le_bytes());
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<Self, SerializationError> {
    let mut moni_data      = Self::new();
    Self::verify_fixed(stream, pos)?;
    moni_data.fpga_temp    = parse_f32(&stream, pos);
    moni_data.fpga_vccint  = parse_f32(&stream, pos);
    moni_data.fpga_vccaux  = parse_f32(&stream, pos);
    moni_data.fpga_vccbram = parse_f32(&stream, pos);
    moni_data.rate         = parse_u16(&stream, pos);
    moni_data.lost_rate    = parse_u16(&stream, pos);
    *pos += 2; // since we deserialized the tail earlier and 
              // didn't account for it
    Ok(moni_data)
  }
}

#[cfg(feature = "random")]
impl FromRandom for MtbMoniData {
  fn from_random() -> Self {
    let mut moni      = Self::new();
    let mut rng       = rand::thread_rng();
    moni.fpga_temp    = rng.gen::<f32>();
    moni.fpga_vccint  = rng.gen::<f32>();
    moni.fpga_vccaux  = rng.gen::<f32>();
    moni.fpga_vccbram = rng.gen::<f32>();
    moni.rate         = rng.gen::<u16>();
    moni.lost_rate    = rng.gen::<u16>();
    moni
  }
}

#[cfg(all(test,feature = "random"))]
mod test_monitoring {
  use crate::serialization::Serialization;
  use crate::FromRandom;
  use crate::monitoring::RBMoniData;
  use crate::monitoring::MtbMoniData;

  #[test]
  fn serialization_mtbmonidata() {
    let data = MtbMoniData::from_random();
    let test = MtbMoniData::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
    assert_eq!(data, test);
  }

  #[test]
  fn serialization_rbmonidata() {
    let data = RBMoniData::from_random();
    let test = RBMoniData::from_bytestream(&data.to_bytestream(), &mut 0).unwrap();
    assert_eq!(data, test);
  }
}

