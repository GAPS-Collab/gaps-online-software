//! Parse magnetometer data

use std::fmt;
use log::error;

use tof_dataclasses::serialization::{
  parse_u8,
  parse_u16,
  parse_u16_be,
  //parse_u32,
  parse_u64,
  Serialization,
  SerializationError,
  //Packable
};

use crate::packets::TelemetryHeader;

pub struct MagnetoMeter {
 pub header        : TelemetryHeader,
 pub temp          : u16, 
 pub mag_x         : u16, 
 pub mag_y         : u16, 
 pub mag_z         : u16, 
 pub acc_x         : u16, 
 pub acc_y         : u16, 
 pub acc_z         : u16, 
 pub roll          : u16, 
 pub pitch         : u16, 
 pub yaw           : u16, 
 pub mag_roll      : u16, 
 pub mag_field     : u16, 
 pub grav_field    : u16, 
 pub expected_size : u64, // technically usize
 pub end_byte      : u16, 
 pub zero          : u8, 
 pub ndata         : u8, 
}

impl MagnetoMeter {
  pub fn new() -> Self {
    Self {
     header            : TelemetryHeader::new(),
     temp              : 0, 
     mag_x             : 0, 
     mag_y             : 0, 
     mag_z             : 0, 
     acc_x             : 0, 
     acc_y             : 0, 
     acc_z             : 0, 
     roll              : 0, 
     pitch             : 0, 
     yaw               : 0, 
     mag_roll          : 0, 
     mag_field         : 0, 
     grav_field        : 0, 
     expected_size     : 0, // technically usize
     end_byte          : 0, 
     zero              : 0, 
     ndata             : 0, 
    }
  }
}

impl Default for MagnetoMeter {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for MagnetoMeter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<MagnetoMeter: ");
    repr += &(format!("\n {}", self.header));
    repr += &(format!("\n temp          : {}", self.temp            ));   
    repr += &(format!("\n mag_x         : {}", self.mag_x           ));   
    repr += &(format!("\n mag_y         : {}", self.mag_y           ));   
    repr += &(format!("\n mag_z         : {}", self.mag_z           ));   
    repr += &(format!("\n acc_x         : {}", self.acc_x           ));   
    repr += &(format!("\n acc_y         : {}", self.acc_y           ));   
    repr += &(format!("\n acc_z         : {}", self.acc_z           ));   
    repr += &(format!("\n roll          : {}", self.roll            ));   
    repr += &(format!("\n pitch         : {}", self.pitch           ));   
    repr += &(format!("\n yaw           : {}", self.yaw             ));   
    repr += &(format!("\n mag_roll      : {}", self.mag_roll        ));   
    repr += &(format!("\n mag_field     : {}", self.mag_field       ));   
    repr += &(format!("\n grav_field    : {}", self.grav_field      ));   
    repr += &(format!("\n expected_size : {}", self.expected_size   ));   
    repr += &(format!("\n end_byte      : {}", self.end_byte        ));   
    repr += &(format!("\n zero          : {}", self.zero            ));   
    repr += &(format!("\n ndata         : {}", self.ndata           ));   
    write!(f, "{}", repr)
  }
}

impl Serialization for MagnetoMeter {
  
  const HEAD : u16   = 0x90eb;
  const TAIL : u16   = 0x0000; // there is no tail for telemetry packets
  const SIZE : usize = 57; 
  
  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize)
    -> Result<Self, SerializationError> {
    let mut mag = Self::new();
    if stream.len() < Self::SIZE {
      error!("We got {} bytes, but need {}!", stream.len(),Self::SIZE);
      return Err(SerializationError::StreamTooShort);
    }
    mag.header  = TelemetryHeader::from_bytestream(stream, pos)?;
    // we do have to deal with a bunch of empty bytes
    let n_empty = parse_u8(stream, pos);
    if n_empty != 16 {
      error!("Decoding of magnetometer packet faILed! We expected 16 empty bytes, but got {} instead!", n_empty);
      return Err(SerializationError::WrongByteSize);
    }
    *pos += n_empty as usize;
    mag.mag_x = parse_u16_be(stream, pos);
    mag.acc_x = parse_u16_be(stream, pos);
    mag.mag_y = parse_u16_be(stream, pos);
    mag.acc_y = parse_u16_be(stream, pos);
    mag.mag_z = parse_u16_be(stream, pos);
    mag.acc_z = parse_u16_be(stream, pos);
    mag.temp  = parse_u16(stream, pos);
    //i += from_bytes(&bytes[i],temp);
    //i +=2; // the other temp we do not understand
    *pos += 2; // ALEX - "the other temp we do not understand"
    //i += from_bytes(&bytes[i],zero);
    mag.zero  = parse_u8(stream, pos);
    if mag.zero != 0 {
      // FIXME - better error type
      error!("Decoding of magnetometer packet failed! Byte whcih should be zero is not zero!");
      return Err(SerializationError::WrongByteSize);
    }
    *pos += 1; // ALEX - "the checksum we are not checking"
    mag.end_byte = parse_u16_be(stream, pos);
    if mag.end_byte != 32767 {
      error!("Decoding of magnetormeter packet faailed! Tail incorrect!");
      return Err(SerializationError::TailInvalid);
    }
    Ok(mag)
  }
}
