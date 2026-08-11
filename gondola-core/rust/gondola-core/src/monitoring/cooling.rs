// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

// Thermal team cooling data (tracker)
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct CoolingMoniData {
  pub frame_counter   : u32, 
  pub status_1        : u8,  
  pub status_2        : u8,  
  pub rx_byte_num     : u8,  
  pub rx_cmd_num      : u8,  
  pub last_cmd        : u64,
  pub rsv_t           : u16, 
  pub rh_on           : u16, 
  pub rh_off          : u16, 
  pub fpga_board_v_in : u16, 
  pub fpga_board_i_in : u16, 
  pub fpga_board_t    : u16, 
  pub fpga_board_p    : u16, 
  //d::array<u16, 64> : u32, rtd;
  //pub rtd             : Vec<u16>, // 64 temperatures
  // needs to satisfy copy trait
  pub rtd_1           : u16,        
  pub rtd_2           : u16,        
  pub rtd_3           : u16,        
  pub rtd_4           : u16,        
  pub rtd_5           : u16,        
  pub rtd_6           : u16,        
  pub rtd_7           : u16,        
  pub rtd_8           : u16,        
  pub rtd_9           : u16,        
  pub rtd_10          : u16,        
  pub rtd_11          : u16,        
  pub rtd_12          : u16,        
  pub rtd_13          : u16,        
  pub rtd_14          : u16,        
  pub rtd_15          : u16,        
  pub rtd_16          : u16,        
  pub rtd_17          : u16,        
  pub rtd_18          : u16,        
  pub rtd_19          : u16,        
  pub rtd_20          : u16,        
  pub rtd_21          : u16,        
  pub rtd_22          : u16,        
  pub rtd_23          : u16,        
  pub rtd_24          : u16,        
  pub rtd_25          : u16,        
  pub rtd_26          : u16,        
  pub rtd_27          : u16,        
  pub rtd_28          : u16,        
  pub rtd_29          : u16,        
  pub rtd_30          : u16,        
  pub rtd_31          : u16,        
  pub rtd_32          : u16,        
  pub rtd_33          : u16,        
  pub rtd_34          : u16,        
  pub rtd_35          : u16,        
  pub rtd_36          : u16,        
  pub rtd_37          : u16,        
  pub rtd_38          : u16,        
  pub rtd_39          : u16, 
  pub rtd_40          : u16,
  pub rtd_41          : u16,
  pub rtd_42          : u16,
  pub rtd_43          : u16,
  pub rtd_44          : u16,
  pub rtd_45          : u16,
  pub rtd_46          : u16,
  pub rtd_47          : u16,
  pub rtd_48          : u16,
  pub rtd_49          : u16,
  pub rtd_50          : u16,
  pub rtd_51          : u16,
  pub rtd_52          : u16,
  pub rtd_53          : u16,
  pub rtd_54          : u16,
  pub rtd_55          : u16,
  pub rtd_56          : u16,
  pub rtd_57          : u16,
  pub rtd_58          : u16,
  pub rtd_59          : u16,
  pub rtd_60          : u16,
  pub rtd_61          : u16,
  pub rtd_62          : u16,
  pub rtd_63          : u16,
  pub rtd_64          : u16,        
  pub sh_current      : u16, 
  pub rh_current      : u16, 
  pub pw_board1_t     : u16, 
  pub pw_board2_t     : u16, 
  pub sh1_time_left   : u16, 
  pub sh2_time_left   : u16, 
  pub sh3_time_left   : u16, 
  
  // to make it a moni data
  pub timestamp       : u64,
}

impl CoolingMoniData {
  pub fn new() -> Self {
    Self {
      frame_counter   : u32::MAX, 
      status_1        : u8::MAX,  
      status_2        : u8::MAX,  
      rx_byte_num     : u8::MAX,  
      rx_cmd_num      : u8::MAX,  
      last_cmd        : u64::MAX,
      rsv_t           : u16::MAX, 
      rh_on           : u16::MAX, 
      rh_off          : u16::MAX, 
      fpga_board_v_in : u16::MAX, 
      fpga_board_i_in : u16::MAX, 
      fpga_board_t    : u16::MAX, 
      fpga_board_p    : u16::MAX, 
      // we can't use a vector eree because we 
      // need to obey te Copy trait
      rtd_1           : u16::MAX,        
      rtd_2           : u16::MAX,        
      rtd_3           : u16::MAX,        
      rtd_4           : u16::MAX,        
      rtd_5           : u16::MAX,        
      rtd_6           : u16::MAX,        
      rtd_7           : u16::MAX,        
      rtd_8           : u16::MAX,        
      rtd_9           : u16::MAX,        
      rtd_10          : u16::MAX,        
      rtd_11          : u16::MAX,        
      rtd_12          : u16::MAX,        
      rtd_13          : u16::MAX,        
      rtd_14          : u16::MAX,        
      rtd_15          : u16::MAX,        
      rtd_16          : u16::MAX,        
      rtd_17          : u16::MAX,        
      rtd_18          : u16::MAX,        
      rtd_19          : u16::MAX,        
      rtd_20          : u16::MAX,        
      rtd_21          : u16::MAX,        
      rtd_22          : u16::MAX,        
      rtd_23          : u16::MAX,        
      rtd_24          : u16::MAX,        
      rtd_25          : u16::MAX,        
      rtd_26          : u16::MAX,        
      rtd_27          : u16::MAX,        
      rtd_28          : u16::MAX,        
      rtd_29          : u16::MAX,        
      rtd_30          : u16::MAX,        
      rtd_31          : u16::MAX,        
      rtd_32          : u16::MAX,        
      rtd_33          : u16::MAX,        
      rtd_34          : u16::MAX,        
      rtd_35          : u16::MAX,        
      rtd_36          : u16::MAX,        
      rtd_37          : u16::MAX,        
      rtd_38          : u16::MAX,        
      rtd_39          : u16::MAX, 
      rtd_40          : u16::MAX,
      rtd_41          : u16::MAX,
      rtd_42          : u16::MAX,
      rtd_43          : u16::MAX,
      rtd_44          : u16::MAX,
      rtd_45          : u16::MAX,
      rtd_46          : u16::MAX,
      rtd_47          : u16::MAX,
      rtd_48          : u16::MAX,
      rtd_49          : u16::MAX,
      rtd_50          : u16::MAX,
      rtd_51          : u16::MAX,
      rtd_52          : u16::MAX,
      rtd_53          : u16::MAX,
      rtd_54          : u16::MAX,
      rtd_55          : u16::MAX,
      rtd_56          : u16::MAX,
      rtd_57          : u16::MAX,
      rtd_58          : u16::MAX,
      rtd_59          : u16::MAX,
      rtd_60          : u16::MAX,
      rtd_61          : u16::MAX,
      rtd_62          : u16::MAX,
      rtd_63          : u16::MAX,
      rtd_64          : u16::MAX,        
      sh_current      : u16::MAX, 
      rh_current      : u16::MAX, 
      pw_board1_t     : u16::MAX, 
      pw_board2_t     : u16::MAX, 
      sh1_time_left   : u16::MAX, 
      sh2_time_left   : u16::MAX, 
      sh3_time_left   : u16::MAX,
      timestamp       : u64::MAX
    }
  }

  /// Transformation of saved u16 values for rtd to 
  /// Temperature in C
  #[allow(non_snake_case)]
  fn rtd_temp(adc : u16) -> f32 { 
    // Keita :) 
    //  u, d, w, G, A, B = 3.3, 1, 1.25, 21.58, 3.9083e-3, -5.775e-7  # constants
    //I = x  # if isinstance(x, int) else int.from_bytes(x, byteorder='little', signed=False)
    //# I = I if I <= 2047 else I - 4096
    //I = np.where(I <= 2047, I, I - 4096)
    //V = I * 10 / 4096 
    //R = u * (w * d + V * (u + d) / G) / (w * u - V * (u + d) / G) 
    //T = (-A + (A**2 - 4 * B * (1 - R)) ** 0.5) / (2 * B) 
    //return T  # degree C
    let mut adc_signed = adc as f32;
    let u : f32 = 3.3;
    let d : f32 = 1.0; 
    let w : f32 = 1.25;
    let G : f32 = 21.58;
    let A : f32 = 3.9083e-3; 
    let B : f32 = -5.775e-7;
    // unsigned -> signed conversion
    if adc > 2047 { 
      adc_signed -= 4086.0;  
    } 
    let V = adc_signed * 10.0 / 4096.0; 
    let R = u * (w * d + V * (u + d) / G) / (w * u - V * (u + d) / G); 
    let T : f32 = (-A + (A.powf(2.0) - 4.0 * B * (1.0 - R)).powf(0.5)) / (2.0 * B); 
    return T;
  }

  // not entirely sure how to map this yet - FIXME
  //fn current(x : u16) -> f32 {
  //  return (5.0 * x as f32 / 4096.0 - 1.25) * 0.1650 / 4.1 / 0.02 
  //}

  //// power board 2 
  //fn current2(x : u16) -> f32 { 
  //  (5.0 * x as f32 / 4096.0 - 1.25) * 0.2050 / 4.1 / 0.05
  //}
  //
  //fn board_temp(x : u16) -> f32 {
  //  (5.0 * x as f32 / 4096.0) * 100.0 - 273.2
  //}
  //
  //fn fpga_v(x : u16) -> f32 {
  //  40.0 * x as f32 / 4096.0
  //}
  //
  //fn fpga_i(x : u16) -> f32 {
  //  3.0 * x as f32 / 4096.0
  //}
  //
  //fn cooling_fpga_p(x : u16) -> f32 {
  //  (5.0 * x as f32 / 4096.0) * 22.878 - 5.253
  //}
}

impl Serialization for CoolingMoniData { 
  
  const SIZE : usize  = 105;

  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize)
    -> Result<Self, SerializationError> {
    let mut cln = Self::new();
    if stream.len() - *pos < Self::SIZE {
      return Err(SerializationError::StreamTooShort);
    }
    let start_byte = parse_u8(stream, pos);
    if start_byte != 0x1e {
      error!("Start byte is {} and not 0x1e!", start_byte);
      return Err(SerializationError::InvalidByte);
    } 
    //cln.rtd.fill(0xffff);
    cln.frame_counter   = 0xffffff & parse_u32(stream, pos);
    *pos -= 1;
    cln.status_1        = parse_u8(stream, pos);  
    cln.status_2        = parse_u8(stream, pos);  
    cln.rx_byte_num     = parse_u8(stream, pos);  
    cln.rx_cmd_num      = parse_u8(stream, pos);  
    cln.last_cmd        = 0xffffffffffffff & parse_u64(stream, pos); 
    *pos -= 1;
    cln.rsv_t           = parse_u16(stream, pos);
    cln.rh_on           = parse_u16(stream, pos);
    cln.rh_off          = parse_u16(stream, pos);
    cln.fpga_board_v_in = parse_u16(stream, pos);
    cln.fpga_board_i_in = parse_u16(stream, pos);
    *pos += 2;
    cln.fpga_board_t    = parse_u16(stream, pos);
    cln.fpga_board_p    = parse_u16(stream, pos);
    *pos += 6;
    cln.rtd_1           = parse_u16(stream, pos);        
    cln.rtd_2           = parse_u16(stream, pos);        
    cln.rtd_3           = parse_u16(stream, pos);        
    cln.rtd_4           = parse_u16(stream, pos);        
    cln.rtd_5           = parse_u16(stream, pos);        
    cln.rtd_6           = parse_u16(stream, pos);        
    cln.rtd_7           = parse_u16(stream, pos);        
    cln.rtd_8           = parse_u16(stream, pos);        
    cln.rtd_9           = parse_u16(stream, pos);        
    cln.rtd_10          = parse_u16(stream, pos);        
    cln.rtd_11          = parse_u16(stream, pos);        
    cln.rtd_12          = parse_u16(stream, pos);        
    cln.rtd_13          = parse_u16(stream, pos);        
    cln.rtd_14          = parse_u16(stream, pos);        
    cln.rtd_15          = parse_u16(stream, pos);        
    cln.rtd_16          = parse_u16(stream, pos);        
    cln.rtd_17          = parse_u16(stream, pos);        
    cln.rtd_18          = parse_u16(stream, pos);        
    cln.rtd_19          = parse_u16(stream, pos);        
    cln.rtd_20          = parse_u16(stream, pos);        
    cln.rtd_21          = parse_u16(stream, pos);        
    cln.rtd_22          = parse_u16(stream, pos);        
    cln.rtd_23          = parse_u16(stream, pos);        
    cln.rtd_24          = parse_u16(stream, pos);        
    cln.rtd_25          = parse_u16(stream, pos);        
    cln.rtd_26          = parse_u16(stream, pos);        
    cln.rtd_27          = parse_u16(stream, pos);        
    cln.rtd_28          = parse_u16(stream, pos);        
    cln.rtd_29          = parse_u16(stream, pos);        
    cln.rtd_30          = parse_u16(stream, pos);        
    cln.rtd_31          = parse_u16(stream, pos);        
    cln.rtd_32          = parse_u16(stream, pos);        
    cln.rtd_33          = parse_u16(stream, pos);        
    cln.rtd_34          = parse_u16(stream, pos);        
    cln.rtd_35          = parse_u16(stream, pos);        
    cln.rtd_36          = parse_u16(stream, pos);        
    cln.rtd_37          = parse_u16(stream, pos);        
    cln.rtd_38          = parse_u16(stream, pos);        
    cln.rtd_39          = parse_u16(stream, pos); 
    cln.rtd_40          = parse_u16(stream, pos);
    cln.rtd_41          = parse_u16(stream, pos);
    cln.rtd_42          = parse_u16(stream, pos);
    cln.rtd_43          = parse_u16(stream, pos);
    cln.rtd_44          = parse_u16(stream, pos);
    cln.rtd_45          = parse_u16(stream, pos);
    cln.rtd_46          = parse_u16(stream, pos);
    cln.rtd_47          = parse_u16(stream, pos);
    cln.rtd_48          = parse_u16(stream, pos);
    cln.rtd_49          = parse_u16(stream, pos);
    cln.rtd_50          = parse_u16(stream, pos);
    cln.rtd_51          = parse_u16(stream, pos);
    cln.rtd_52          = parse_u16(stream, pos);
    cln.rtd_53          = parse_u16(stream, pos);
    cln.rtd_54          = parse_u16(stream, pos);
    cln.rtd_55          = parse_u16(stream, pos);
    cln.rtd_56          = parse_u16(stream, pos);
    cln.rtd_57          = parse_u16(stream, pos);
    cln.rtd_58          = parse_u16(stream, pos);
    cln.rtd_59          = parse_u16(stream, pos);
    cln.rtd_60          = parse_u16(stream, pos);
    cln.rtd_61          = parse_u16(stream, pos);
    cln.rtd_62          = parse_u16(stream, pos);
    cln.rtd_63          = parse_u16(stream, pos);
    cln.rtd_64          = parse_u16(stream, pos);        
    cln.sh_current    = parse_u16(stream, pos);
    cln.rh_current    = parse_u16(stream, pos);
    cln.pw_board1_t   = parse_u16(stream, pos);
    cln.pw_board2_t   = parse_u16(stream, pos);
    cln.sh1_time_left = parse_u16(stream, pos);
    cln.sh2_time_left = parse_u16(stream, pos);
    cln.sh3_time_left = parse_u16(stream, pos);
    *pos += 2; //spare
    //auto stop_byte = parse_u8(stream,pos);
    //if (stop_byte != 0x0a) {
    //  std::string message = std::format("Stop byte for cooling packet incorrect! Got {} instead of 0x0a", stop_byte);
    //  auto err = g::IOError(g::IOError::ErrorKind::WrongDelimiter, message);
    //  return Err(err);
    //} 
    //return Ok(cln);
    Ok(cln)
  }
}


impl FromRandom for CoolingMoniData {
    
  fn from_random() -> Self {
    let mut moni         = Self::new();
    let mut rng          = rand::rng();
    moni.frame_counter   = rng.random::<u32>();
    moni.status_1        = rng.random::<u8>(); 
    moni.status_2        = rng.random::<u8>(); 
    moni.rx_byte_num     = rng.random::<u8>(); 
    moni.rx_cmd_num      = rng.random::<u8>(); 
    moni.last_cmd        = rng.random::<u64>();
    moni.rsv_t           = rng.random::<u16>(); 
    moni.rh_on           = rng.random::<u16>(); 
    moni.rh_off          = rng.random::<u16>(); 
    moni.fpga_board_v_in = rng.random::<u16>(); 
    moni.fpga_board_i_in = rng.random::<u16>(); 
    moni.fpga_board_t    = rng.random::<u16>(); 
    moni.fpga_board_p    = rng.random::<u16>(); 
    moni.rtd_1           = rng.random::<u16>();        
    moni.rtd_2           = rng.random::<u16>();        
    moni.rtd_3           = rng.random::<u16>();        
    moni.rtd_4           = rng.random::<u16>();        
    moni.rtd_5           = rng.random::<u16>();        
    moni.rtd_6           = rng.random::<u16>();        
    moni.rtd_7           = rng.random::<u16>();        
    moni.rtd_8           = rng.random::<u16>();        
    moni.rtd_9           = rng.random::<u16>();        
    moni.rtd_10          = rng.random::<u16>();        
    moni.rtd_11          = rng.random::<u16>();        
    moni.rtd_12          = rng.random::<u16>();        
    moni.rtd_13          = rng.random::<u16>();        
    moni.rtd_14          = rng.random::<u16>();        
    moni.rtd_15          = rng.random::<u16>();        
    moni.rtd_16          = rng.random::<u16>();        
    moni.rtd_17          = rng.random::<u16>();        
    moni.rtd_18          = rng.random::<u16>();        
    moni.rtd_19          = rng.random::<u16>();        
    moni.rtd_20          = rng.random::<u16>();        
    moni.rtd_21          = rng.random::<u16>();        
    moni.rtd_22          = rng.random::<u16>();        
    moni.rtd_23          = rng.random::<u16>();        
    moni.rtd_24          = rng.random::<u16>();        
    moni.rtd_25          = rng.random::<u16>();        
    moni.rtd_26          = rng.random::<u16>();        
    moni.rtd_27          = rng.random::<u16>();        
    moni.rtd_28          = rng.random::<u16>();        
    moni.rtd_29          = rng.random::<u16>();        
    moni.rtd_30          = rng.random::<u16>();        
    moni.rtd_31          = rng.random::<u16>();        
    moni.rtd_32          = rng.random::<u16>();        
    moni.rtd_33          = rng.random::<u16>();        
    moni.rtd_34          = rng.random::<u16>();        
    moni.rtd_35          = rng.random::<u16>();        
    moni.rtd_36          = rng.random::<u16>();        
    moni.rtd_37          = rng.random::<u16>();        
    moni.rtd_38          = rng.random::<u16>();        
    moni.rtd_39          = rng.random::<u16>(); 
    moni.rtd_40          = rng.random::<u16>();
    moni.rtd_41          = rng.random::<u16>();
    moni.rtd_42          = rng.random::<u16>();
    moni.rtd_43          = rng.random::<u16>();
    moni.rtd_44          = rng.random::<u16>();
    moni.rtd_45          = rng.random::<u16>();
    moni.rtd_46          = rng.random::<u16>();
    moni.rtd_47          = rng.random::<u16>();
    moni.rtd_48          = rng.random::<u16>();
    moni.rtd_49          = rng.random::<u16>();
    moni.rtd_50          = rng.random::<u16>();
    moni.rtd_51          = rng.random::<u16>();
    moni.rtd_52          = rng.random::<u16>();
    moni.rtd_53          = rng.random::<u16>();
    moni.rtd_54          = rng.random::<u16>();
    moni.rtd_55          = rng.random::<u16>();
    moni.rtd_56          = rng.random::<u16>();
    moni.rtd_57          = rng.random::<u16>();
    moni.rtd_58          = rng.random::<u16>();
    moni.rtd_59          = rng.random::<u16>();
    moni.rtd_60          = rng.random::<u16>();
    moni.rtd_61          = rng.random::<u16>();
    moni.rtd_62          = rng.random::<u16>();
    moni.rtd_63          = rng.random::<u16>();
    moni.rtd_64          = rng.random::<u16>();        
    moni.sh_current      = rng.random::<u16>(); 
    moni.rh_current      = rng.random::<u16>(); 
    moni.pw_board1_t     = rng.random::<u16>(); 
    moni.pw_board2_t     = rng.random::<u16>(); 
    moni.sh1_time_left   = rng.random::<u16>(); 
    moni.sh2_time_left   = rng.random::<u16>(); 
    moni.sh3_time_left   = rng.random::<u16>(); 
    moni
  }
}

impl MoniData for CoolingMoniData {
  
  fn get_board_id(&self) -> u8 {
    return 0;
  }

  fn get_timestamp(&self) -> u64 {
    self.timestamp 
  }

  fn set_timestamp(&mut self, ts: u64) {
    self.timestamp = ts;
  }

  fn keys() -> Vec<&'static str> {
    vec!["board_id",
         "frame_counter",   
         "status_1",        
         "status_2",        
         "rx_byte_num",     
         "rx_cmd_num",      
         "last_cmd",        
         "rsv_t",           
         "rh_on",           
         "rh_off",          
         "fpga_board_v_in", 
         "fpga_board_i_in", 
         "fpga_board_t",    
         "fpga_board_p",    
         "rtd_1",        
         "rtd_2",        
         "rtd_3",        
         "rtd_4",        
         "rtd_5",        
         "rtd_6",        
         "rtd_7",        
         "rtd_8",        
         "rtd_9",        
         "rtd_10",        
         "rtd_11",        
         "rtd_12",        
         "rtd_13",        
         "rtd_14",        
         "rtd_15",        
         "rtd_16",        
         "rtd_17",        
         "rtd_18",        
         "rtd_19",        
         "rtd_20",        
         "rtd_21",        
         "rtd_22",        
         "rtd_23",        
         "rtd_24",        
         "rtd_25",        
         "rtd_26",        
         "rtd_27",        
         "rtd_28",        
         "rtd_29",        
         "rtd_30",        
         "rtd_31",        
         "rtd_32",        
         "rtd_33",        
         "rtd_34",        
         "rtd_35",        
         "rtd_36",        
         "rtd_37",        
         "rtd_38",        
         "rtd_39", 
         "rtd_40",
         "rtd_41",
         "rtd_42",
         "rtd_43",
         "rtd_44",
         "rtd_45",
         "rtd_46",
         "rtd_47",
         "rtd_48",
         "rtd_49",
         "rtd_50",
         "rtd_51",
         "rtd_52",
         "rtd_53",
         "rtd_54",
         "rtd_55",
         "rtd_56",
         "rtd_57",
         "rtd_58",
         "rtd_59",
         "rtd_60",
         "rtd_61",
         "rtd_62",
         "rtd_63",
         "rtd_64",        
         "sh_current",      
         "rh_current",      
         "pw_board1_t",     
         "pw_board2_t",     
         "sh1_time_left",   
         "sh2_time_left",   
         "sh3_time_left",   
         "timestamp"]
  }

  fn get(&self, varname : &str) -> Option<f32> {
    match varname {
      "board_id"        => Some(0.0),
      "frame_counter"   => Some(self.frame_counter   as f32),   
      "status_1"        => Some(self.status_1        as f32),        
      "status_2"        => Some(self.status_2        as f32),        
      "rx_byte_num"     => Some(self.rx_byte_num     as f32),     
      "rx_cmd_num"      => Some(self.rx_cmd_num      as f32),      
      "last_cmd"        => Some(self.last_cmd        as f32),        
      "rsv_t"           => Some(self.rsv_t           as f32),           
      "rh_on"           => Some(self.rh_on           as f32),           
      "rh_off"          => Some(self.rh_off          as f32),          
      "fpga_board_v_in" => Some(self.fpga_board_v_in as f32), 
      "fpga_board_i_in" => Some(self.fpga_board_i_in as f32), 
      "fpga_board_t"    => Some(self.fpga_board_t   as f32),    
      "fpga_board_p"    => Some(self.fpga_board_p   as f32),    
      "rtd_1"           => Some(Self::rtd_temp(self.rtd_1 )),         
      "rtd_2"           => Some(Self::rtd_temp(self.rtd_2 )),        
      "rtd_3"           => Some(Self::rtd_temp(self.rtd_3 )),        
      "rtd_4"           => Some(Self::rtd_temp(self.rtd_4 )),        
      "rtd_5"           => Some(Self::rtd_temp(self.rtd_5 )),        
      "rtd_6"           => Some(Self::rtd_temp(self.rtd_6 )),        
      "rtd_7"           => Some(Self::rtd_temp(self.rtd_7 )),        
      "rtd_8"           => Some(Self::rtd_temp(self.rtd_8 )),        
      "rtd_9"           => Some(Self::rtd_temp(self.rtd_9 )),        
      "rtd_10"          => Some(Self::rtd_temp(self.rtd_10)),        
      "rtd_11"          => Some(Self::rtd_temp(self.rtd_11)),        
      "rtd_12"          => Some(Self::rtd_temp(self.rtd_12)),        
      "rtd_13"          => Some(Self::rtd_temp(self.rtd_13)),        
      "rtd_14"          => Some(Self::rtd_temp(self.rtd_14)),        
      "rtd_15"          => Some(Self::rtd_temp(self.rtd_15)),        
      "rtd_16"          => Some(Self::rtd_temp(self.rtd_16)),        
      "rtd_17"          => Some(Self::rtd_temp(self.rtd_17)),        
      "rtd_18"          => Some(Self::rtd_temp(self.rtd_18)),        
      "rtd_19"          => Some(Self::rtd_temp(self.rtd_19)),        
      "rtd_20"          => Some(Self::rtd_temp(self.rtd_20)),        
      "rtd_21"          => Some(Self::rtd_temp(self.rtd_21)),        
      "rtd_22"          => Some(Self::rtd_temp(self.rtd_22)),        
      "rtd_23"          => Some(Self::rtd_temp(self.rtd_23)),        
      "rtd_24"          => Some(Self::rtd_temp(self.rtd_24)),        
      "rtd_25"          => Some(Self::rtd_temp(self.rtd_25)),        
      "rtd_26"          => Some(Self::rtd_temp(self.rtd_26)),        
      "rtd_27"          => Some(Self::rtd_temp(self.rtd_27)),        
      "rtd_28"          => Some(Self::rtd_temp(self.rtd_28)),        
      "rtd_29"          => Some(Self::rtd_temp(self.rtd_29)),        
      "rtd_30"          => Some(Self::rtd_temp(self.rtd_30)),        
      "rtd_31"          => Some(Self::rtd_temp(self.rtd_31)),        
      "rtd_32"          => Some(Self::rtd_temp(self.rtd_32)),        
      "rtd_33"          => Some(Self::rtd_temp(self.rtd_33)),        
      "rtd_34"          => Some(Self::rtd_temp(self.rtd_34)),        
      "rtd_35"          => Some(Self::rtd_temp(self.rtd_35)),        
      "rtd_36"          => Some(Self::rtd_temp(self.rtd_36)),        
      "rtd_37"          => Some(Self::rtd_temp(self.rtd_37)),        
      "rtd_38"          => Some(Self::rtd_temp(self.rtd_38)),        
      "rtd_39"          => Some(Self::rtd_temp(self.rtd_39)), 
      "rtd_40"          => Some(Self::rtd_temp(self.rtd_40)),
      "rtd_41"          => Some(Self::rtd_temp(self.rtd_41)),
      "rtd_42"          => Some(Self::rtd_temp(self.rtd_42)),
      "rtd_43"          => Some(Self::rtd_temp(self.rtd_43)),
      "rtd_44"          => Some(Self::rtd_temp(self.rtd_44)),
      "rtd_45"          => Some(Self::rtd_temp(self.rtd_45)),
      "rtd_46"          => Some(Self::rtd_temp(self.rtd_46)),
      "rtd_47"          => Some(Self::rtd_temp(self.rtd_47)),
      "rtd_48"          => Some(Self::rtd_temp(self.rtd_48)),
      "rtd_49"          => Some(Self::rtd_temp(self.rtd_49)),
      "rtd_50"          => Some(Self::rtd_temp(self.rtd_50)),
      "rtd_51"          => Some(Self::rtd_temp(self.rtd_51)),
      "rtd_52"          => Some(Self::rtd_temp(self.rtd_52)),
      "rtd_53"          => Some(Self::rtd_temp(self.rtd_53)),
      "rtd_54"          => Some(Self::rtd_temp(self.rtd_54)),
      "rtd_55"          => Some(Self::rtd_temp(self.rtd_55)),
      "rtd_56"          => Some(Self::rtd_temp(self.rtd_56)),
      "rtd_57"          => Some(Self::rtd_temp(self.rtd_57)),
      "rtd_58"          => Some(Self::rtd_temp(self.rtd_58)),
      "rtd_59"          => Some(Self::rtd_temp(self.rtd_59)),
      "rtd_60"          => Some(Self::rtd_temp(self.rtd_60)),
      "rtd_61"          => Some(Self::rtd_temp(self.rtd_61)),
      "rtd_62"          => Some(Self::rtd_temp(self.rtd_62)),
      "rtd_63"          => Some(Self::rtd_temp(self.rtd_63)),
      "rtd_64"          => Some(Self::rtd_temp(self.rtd_64)),        
      "sh_current"      => Some(self.sh_current     as f32),      
      "rh_current"      => Some(self.rh_current     as f32),      
      "pw_board1_t"     => Some(self.pw_board1_t    as f32),     
      "pw_board2_t"     => Some(self.pw_board2_t    as f32),     
      "sh1_time_left"   => Some(self.sh1_time_left  as f32),   
      "sh2_time_left"   => Some(self.sh2_time_left  as f32),   
      "sh3_time_left"   => Some(self.sh3_time_left  as f32),   
      "timestamp" => Some(self.timestamp as f32),
      _           => None
    }
  }  
}

impl TelemetryPackable for CoolingMoniData {
  const TEL_PACKET_TYPE : TelemetryPacketType = TelemetryPacketType::CoolingHK;
}

moniseries_telemetry!(CoolingMoniDataSeries, CoolingMoniData);

#[cfg(feature="pybindings")]
pythonize_monidata!(CoolingMoniData);

