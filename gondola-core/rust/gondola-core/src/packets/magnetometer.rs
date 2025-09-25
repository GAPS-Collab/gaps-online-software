// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

#[cfg_attr(feature="pybindings", pyclass)]
pub struct MagnetoMeter {
  pub telemetry_header : TelemetryPacketHeader,
  pub temp             : u16, 
  pub mag_x            : u16, 
  pub mag_y            : u16, 
  pub mag_z            : u16, 
  pub acc_x            : u16, 
  pub acc_y            : u16, 
  pub acc_z            : u16, 
  pub roll             : u16, 
  pub pitch            : u16, 
  pub yaw              : u16, 
  pub mag_roll         : u16, 
  pub mag_field        : u16, 
  pub grav_field       : u16, 
  pub expected_size    : u64, // technically usize
  pub end_byte         : u16, 
  pub zero             : u8, 
  pub ndata            : u8, 
}

impl MagnetoMeter {
  pub fn new() -> Self {
    Self {
     telemetry_header  : TelemetryPacketHeader::new(),
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
    repr += &(format!("\n {}", self.telemetry_header));
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
    mag.telemetry_header  = TelemetryPacketHeader::from_bytestream(stream, pos)?;
    // we do have to deal with a bunch of empty bytes
    *pos += 1;
    let mut n_data = parse_u8(stream, pos);
    if n_data != 16 {
      error!("Decoding of magnetometer packet faILed! We expected 16 data bytes, but got {} instead!", n_data);
      return Err(SerializationError::WrongByteSize);
    }
    //*pos += n_empty as usize;
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
    *pos += 1; // empty bytes that we do not care about from the first magnetometer packet
    n_data     = parse_u8(stream, pos);
    if n_data != 16 {
      error!("The second magnetometer data chunk seems to have the wrong size! ({} instead of 16)", n_data);
      return Err(SerializationError::WrongByteSize);
    };
    mag.roll        = parse_u16_be(stream, pos); 
    mag.mag_roll    = parse_u16_be(stream, pos); 
    mag.pitch       = parse_u16_be(stream, pos); 
    mag.mag_field   = parse_u16_be(stream, pos); 
    mag.yaw         = parse_u16_be(stream, pos); 
    mag.grav_field  = parse_u16_be(stream, pos); 
    *pos += 4; // ALEX - "more temp data we are not reading out"
    mag.zero = parse_u8(stream, pos);
    if mag.zero != 0 {
      // FIXME - better error type
      error!("Decoding of magnetometer packet failed! Byte whcih should be zero is not zero!");
      return Err(SerializationError::WrongByteSize);
    }
    *pos += 1; // ALEX  - "another checksum (from second packet) we are not checking"
    mag.end_byte = parse_u16_be(stream, pos);
    if mag.end_byte != 32767 {
      error!("Decoding of magnetormeter packet faailed! Tail incorrect!");
      return Err(SerializationError::TailInvalid);
    }
    Ok(mag)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl MagnetoMeter {

  #[getter]
  fn get_temp         (&self) -> u16 { 
    self.temp
  }
  
  #[getter]
  fn get_mag_x        (&self) -> u16 { 
    self.mag_x
  }
  
  #[getter]
  fn get_mag_y        (&self) -> u16 { 
    self.mag_y
  }
  
  #[getter]
  fn get_mag_z        (&self) -> u16 { 
    self.mag_z
  }
  
  //#[getter]
  //fn mag_tot        (&self) -> u16 { 
  //  self.mag.mag_z
  //}
  
  #[getter]
  fn get_acc_x        (&self) -> u16 { 
    self.acc_x
  }
  
  #[getter]
  fn get_acc_y        (&self) -> u16 { 
    self.acc_y
  }
  
  #[getter]
  fn get_acc_z        (&self) -> u16 { 
    self.acc_z
  }
  
  #[getter]
  fn get_roll         (&self) -> u16 { 
    self.roll
  }
  
  #[getter]
  fn get_pitch        (&self) -> u16 { 
    self.pitch
  }
  
  #[getter]
  fn get_yaw          (&self) -> u16 { 
    self.yaw
  }
  
  #[getter]
  fn get_mag_roll     (&self) -> u16 { 
    self.mag_roll
  }
  
  #[getter]
  fn get_mag_field    (&self) -> u16 { 
    self.mag_field
  }
  
  #[getter]
  fn get_grav_field   (&self) -> u16 { 
    self.grav_field
  }
 
 //fn expected_size(&self) -> u64 { 
 //fn end_byte     (&self) -> u16 { 
 //fn zero         (&self) -> u8  { 
 //fn ndata        (&self) -> u8  { 
}

#[cfg(feature="pybindings")]
pythonize_telemetry!(MagnetoMeter);

