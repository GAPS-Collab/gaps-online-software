//! Structures for monitoring
//!
//! This is 
//! a) Monitoring the RBs
//! b) Monitoring the tof-computer/main C&C instance
//! c) Monitoring the MTB
//!
//!

use std::fmt;
use crate::serialization::{Serialization,
                           SerializationError,
                           search_for_u16,
                           parse_u8,
                           parse_u16,
                           parse_u32,
                           parse_f32};

/// A collection of monitoring data
/// from the readoutboards. This includes
/// temperatures, power data, pressure, humidity
/// as well as the magnetic sensors
pub struct RBMoniData {

  pub board_id           : u8,
  pub rate               : u16,
  pub tmp_drc            : f32,
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
  // voltages
  pub pow_zynq_v         : f32,
  pub pow_3_3_v          : f32,
  pub pow_3_5_v          : f32,
  pub pow_neg_5_v        : f32,
  pub pow_drs4_dig_2_5_v : f32,
  pub pow_drs4_an_2_5_v  : f32,
  pub pow_adc_dig_2_5_v  : f32,
  pub pow_adc_an_3_0_v   : f32,
  // current
  pub pow_zynq_a         : f32,
  pub pow_3_3_a          : f32,
  pub pow_3_5_a          : f32,
  pub pow_neg_5_a        : f32,
  pub pow_drs4_dig_2_5_a : f32,
  pub pow_drs4_an_2_5_a  : f32,
  pub pow_adc_dig_2_5_a  : f32,
  pub pow_adc_an_3_0_a   : f32,
  // power
  pub pow_zynq_p         : f32,
  pub pow_3_3_p          : f32,
  pub pow_3_5_p          : f32,
  pub pow_neg_5_p        : f32,
  pub pow_drs4_dig_2_5_p : f32,
  pub pow_drs4_an_2_5_p  : f32,
  pub pow_adc_dig_2_5_p  : f32,
  pub pow_adc_an_3_0_p   : f32,
}

impl RBMoniData {

  pub const HEAD : u16 = 0xAAAA;
  pub const TAIL : u16 = 0x5555;
  /// The data size when serialized to a bytestream
  /// This needs to be updated when we change the 
  /// packet layout, e.g. add new members.
  /// HEAD + TAIL + sum(sizeof(m) for m in _all_members_))
  pub const SIZE : usize  = 6;

  pub fn new() -> Self {
    Self {
      board_id           : 0, 
      rate               : 0,
      tmp_drc            : 0.0,
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
      // voltages
      pow_zynq_v         : 0.0,
      pow_3_3_v          : 0.0,
      pow_3_5_v          : 0.0,
      pow_neg_5_v        : 0.0,
      pow_drs4_dig_2_5_v : 0.0,
      pow_drs4_an_2_5_v  : 0.0,
      pow_adc_dig_2_5_v  : 0.0,
      pow_adc_an_3_0_v   : 0.0,
      // current
      pow_zynq_a         : 0.0,
      pow_3_3_a          : 0.0,
      pow_3_5_a          : 0.0,
      pow_neg_5_a        : 0.0,
      pow_drs4_dig_2_5_a : 0.0,
      pow_drs4_an_2_5_a  : 0.0,
      pow_adc_dig_2_5_a  : 0.0,
      pow_adc_an_3_0_a   : 0.0,
      // power
      pow_zynq_p         : 0.0,
      pow_3_3_p          : 0.0,
      pow_3_5_p          : 0.0,
      pow_neg_5_p        : 0.0,
      pow_drs4_dig_2_5_p : 0.0,
      pow_drs4_an_2_5_p  : 0.0,
      pow_adc_dig_2_5_p  : 0.0,
      pow_adc_an_3_0_p   : 0.0,
    }
  }
}

impl Default for RBMoniData {
  fn default() -> Self {
    RBMoniData::new()
  }
}

impl Serialization for RBMoniData {
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(RBMoniData::SIZE);
    stream.extend_from_slice(&RBMoniData::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.board_id          .to_le_bytes()); 
    stream.extend_from_slice(&self.rate              .to_le_bytes()); 
    stream.extend_from_slice(&self.tmp_drc           .to_le_bytes()); 
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
    stream.extend_from_slice(&self.pow_zynq_v        .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_3_3_v         .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_3_5_v         .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_neg_5_v       .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_drs4_dig_2_5_v.to_le_bytes()); 
    stream.extend_from_slice(&self.pow_drs4_an_2_5_v .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_adc_dig_2_5_v .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_adc_an_3_0_v  .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_zynq_a        .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_3_3_a         .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_3_5_a         .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_neg_5_a       .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_drs4_dig_2_5_a.to_le_bytes()); 
    stream.extend_from_slice(&self.pow_drs4_an_2_5_a .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_adc_dig_2_5_a .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_adc_an_3_0_a  .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_zynq_p        .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_3_3_p         .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_3_5_p         .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_neg_5_p       .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_drs4_dig_2_5_p.to_le_bytes()); 
    stream.extend_from_slice(&self.pow_drs4_an_2_5_p .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_adc_dig_2_5_p .to_le_bytes()); 
    stream.extend_from_slice(&self.pow_adc_an_3_0_p  .to_le_bytes()); 
    stream.extend_from_slice(&RBMoniData::TAIL.to_le_bytes());
    stream
  }

  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<RBMoniData, SerializationError>{
    let mut moni_data = RBMoniData::new();
    let head_pos = search_for_u16(RBMoniData::HEAD, stream, *pos)?; 
    let tail_pos = search_for_u16(RBMoniData::TAIL, stream, head_pos + RBMoniData::SIZE-2)?;
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    if tail_pos + 2 - head_pos != RBMoniData::SIZE {
      error!("RBMoniData. Seing {} bytes, but expecting {}", tail_pos + 2 - head_pos, RBMoniData::SIZE);
      *pos = head_pos + 2; 
      return Err(SerializationError::WrongByteSize);
    }

    moni_data.board_id           = parse_u8(&stream, pos); 
    moni_data.rate               = parse_u16(&stream, pos); 
    moni_data.tmp_drc            = parse_f32(&stream, pos); 
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
    moni_data.pow_zynq_v         = parse_f32(&stream, pos); 
    moni_data.pow_3_3_v          = parse_f32(&stream, pos); 
    moni_data.pow_3_5_v          = parse_f32(&stream, pos); 
    moni_data.pow_neg_5_v        = parse_f32(&stream, pos); 
    moni_data.pow_drs4_dig_2_5_v = parse_f32(&stream, pos); 
    moni_data.pow_drs4_an_2_5_v  = parse_f32(&stream, pos); 
    moni_data.pow_adc_dig_2_5_v  = parse_f32(&stream, pos); 
    moni_data.pow_adc_an_3_0_v   = parse_f32(&stream, pos); 
    moni_data.pow_zynq_a         = parse_f32(&stream, pos); 
    moni_data.pow_3_3_a          = parse_f32(&stream, pos); 
    moni_data.pow_3_5_a          = parse_f32(&stream, pos); 
    moni_data.pow_neg_5_a        = parse_f32(&stream, pos); 
    moni_data.pow_drs4_dig_2_5_a = parse_f32(&stream, pos); 
    moni_data.pow_drs4_an_2_5_a  = parse_f32(&stream, pos); 
    moni_data.pow_adc_dig_2_5_a  = parse_f32(&stream, pos); 
    moni_data.pow_adc_an_3_0_a   = parse_f32(&stream, pos); 
    moni_data.pow_zynq_p         = parse_f32(&stream, pos); 
    moni_data.pow_3_3_p          = parse_f32(&stream, pos); 
    moni_data.pow_3_5_p          = parse_f32(&stream, pos); 
    moni_data.pow_neg_5_p        = parse_f32(&stream, pos); 
    moni_data.pow_drs4_dig_2_5_p = parse_f32(&stream, pos); 
    moni_data.pow_drs4_an_2_5_p  = parse_f32(&stream, pos); 
    moni_data.pow_adc_dig_2_5_p  = parse_f32(&stream, pos); 
    moni_data.pow_adc_an_3_0_p   = parse_f32(&stream, pos); 
    *pos += 2;
    Ok(moni_data) 
  }
}

/// Monitoring the main tof computer
pub struct TofCmpMoniData {
  pub core1_tmp : u8,
  pub core2_tmp : u8,
  pub pch_tmp   : u8
}

impl TofCmpMoniData {
  const SIZE : usize = 7;
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;
  
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
           \t CORE1 TMP [C] {}
           \t CORE2 TMP [C] {}
           \t PCH TMP   [C] {}>",
           self.core1_tmp, self.core2_tmp, self.pch_tmp)
  }
}

impl Serialization for TofCmpMoniData {

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
pub struct MtbMoniData {
  pub fpga_temp    : f32,
  pub fpga_vccint  : f32,
  pub fpga_vccaux  : f32,
  pub fpga_vccbram : f32,
  pub rate         : u16,
  pub lost_rate    : u16
}

impl MtbMoniData {
  const SIZE : usize = 24;
  const HEAD : u16   = 0xAAAA;
  const TAIL : u16   = 0x5555;
  
  pub fn new() -> MtbMoniData {
    MtbMoniData {
      fpga_temp    : -4242.42,
      fpga_vccint  : -4242.42,
      fpga_vccaux  : -4242.42,
      fpga_vccbram : -4242.42,
      rate         : u16::MAX,
      lost_rate    : u16::MAX
    }
  }
}

impl Default for MtbMoniData {
  fn default() -> MtbMoniData {
    MtbMoniData::new()
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

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(MtbMoniData::SIZE);
    stream.extend_from_slice(&MtbMoniData::HEAD.to_le_bytes());
    stream.extend_from_slice(&self.fpga_temp   .to_le_bytes());
    stream.extend_from_slice(&self.fpga_vccint .to_le_bytes());
    stream.extend_from_slice(&self.fpga_vccaux .to_le_bytes());
    stream.extend_from_slice(&self.fpga_vccbram.to_le_bytes());
    stream.extend_from_slice(&self.rate        .to_le_bytes());
    stream.extend_from_slice(&self.lost_rate   .to_le_bytes());
    stream.extend_from_slice(&MtbMoniData::TAIL.to_le_bytes());
    stream
  }

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
    -> Result<MtbMoniData, SerializationError> {
    let mut moni_data = MtbMoniData::new();
    let head_pos = search_for_u16(MtbMoniData::HEAD, stream, *pos)?; 
    let tail_pos = search_for_u16(MtbMoniData::TAIL, stream, head_pos + MtbMoniData::SIZE-2)?;
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    if tail_pos + 2 - head_pos != MtbMoniData::SIZE {
      error!("MtbMoniData incomplete. Seing {} bytes, but expecting {}", tail_pos + 2 - head_pos, TofCmpMoniData::SIZE);
      //error!("{:?}", &stream[head_pos + 18526..head_pos + 18540]);
      *pos = head_pos + 2; //start_pos += RBBinaryDump::SIZE;
      return Err(SerializationError::WrongByteSize);
    }
    *pos = head_pos + 2; 
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

