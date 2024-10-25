//! A sclerite is the individual (frame) unit which can hold
//! multiple packets
//!
use std::collections::HashMap;
use std::fmt;

#[cfg(feature="pybindings")]
use pyo3::pyclass;

use crate::parsers::*;
use crate::serialization::*;
use crate::errors::*;

#[cfg(feature="random")]
use crate::FromRandom;

#[cfg(feature="random")]
use rand::Rng;



//// Hash function to generate a u16 value based on the name of the variant
//const fn simple_hash(s: &str) -> u16 {
//    let mut hash: u16 = 0;
//    let mut i = 0;
//    while i < s.len() {
//        hash = hash.wrapping_add(s.as_bytes()[i] as u16);
//        i += 1;
//    }
//    hash
//}

// FIXME - this is something for the furutre. Work on a macro system
// The macro system should allow to register any type with the caraspace
// library. Not sure if that is possible.
////// Declare a macro that checks for duplicates in an enum definition
//macro_rules! caraspace_register_types {
//  ( $($variant:ident),* $(,)? ) => {
//      #[cfg_attr(feature = "pybindings", pyclass)]
//      #[derive(Debug, Copy, Clone, PartialEq)]
//      #[repr(u16)]
//      enum CRFrameObject2 {
//        $(
//            $variant = simple_hash(stringify!($variant)),
//        )*
//      }
//  };
//
//  // This helper macro will trigger a compile-time error if the same variant is found twice
//  (@check_duplicate CRFrameObjectType2:ident { $($existing_variant:ident),* }, $duplicate:ident) => {
//    $(
//      compile_error!(concat!("The following type has already been registered with the caraspace system: ", stringify!($duplicate)));
//    )*
//  };
//
//  // Detects duplicates by calling the helper macro to check against previous variants
//  (CRFrameObjectType2:ident { $($existing_variant:ident),*, $duplicate:ident, $($rest:ident),* $(,)? }) => {
//      define_enum!(@check_duplicate CRFrameObjectType2 { $($existing_variant),* }, $duplicate);
//      define_enum!($enum_name { $($existing_variant),*, $duplicate, $($rest),* });
//  };
//}
//
//macro_rules! caraspace_register {
//  ( $(($t:ty, $variant:ident)),* $((,),)? ) => {
//  //($t:ty,  $variant:ident) => {
//    #[cfg_attr(feature = "pybindings", pyclass)]
//    #[derive(Debug, Copy, Clone, PartialEq)]
//    #[repr(u16)]
//    enum CRFrameObjectType2 {
//      $(
//          $variant = simple_hash(stringify!($variant)),
//      )*
//    }
//    pub trait Frameable2 {
//      const CRFRAMEOBJECT_TYPE : CRFrameObjectType2;
//    }
//    $(
//    impl Frameable2 for $t {
//      const CRFRAMEOBJECT_TYPE : CRFrameObjectType2 = CRFrameObjectType2::$variant;
//    }
//    )*
//  };
//}
//
//struct foo {}
//struct bar {}
//
////caraspace_register_types!(FOO, BAR);
//caraspace_register!((foo, Unknown), (bar, BAR));


/// The Caraspace object type determines the 
/// kind of object we are able to put in 
/// a frame and ultimate (de)serialzie
#[cfg_attr(feature = "pybindings", pyclass)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum CRFrameObjectType {
  Unknown          =  0u8,
  TofPacket        = 10u8,
  TelemetryPacket  = 20u8,
}

impl CRFrameObjectType {
  pub fn to_string(&self) -> String {
    match self {
      CRFrameObjectType::Unknown         => {return String::from("Unknown");},
      CRFrameObjectType::TofPacket       => {return String::from("TofPacket");},
      CRFrameObjectType::TelemetryPacket => {return String::from("TelemetryPacket");}
    }
  }

  pub fn to_u8(&self) -> u8 {
    match self {
      CRFrameObjectType::Unknown         => {return  0;},
      CRFrameObjectType::TofPacket       => {return 10;},
      CRFrameObjectType::TelemetryPacket => {return 20;},
    }
  }
}

impl From<u8> for CRFrameObjectType {
  fn from(value: u8) -> Self {
    match value {
      0   => CRFrameObjectType::Unknown,
      10  => CRFrameObjectType::TofPacket,
      20  => CRFrameObjectType::TelemetryPacket,
       _  => CRFrameObjectType::Unknown
    }
  }
}

impl fmt::Display for CRFrameObjectType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let repr = self.to_string();
    write!(f, "<CRFrameObjectType : {}>", repr)
  }
}

#[cfg(feature = "random")]
impl FromRandom for CRFrameObjectType {

  fn from_random() -> Self {
    let choices = [
      CRFrameObjectType::Unknown,
      CRFrameObjectType::TofPacket,
      CRFrameObjectType::TelemetryPacket,
    ];
    let mut rng  = rand::thread_rng();
    let idx = rng.gen_range(0..choices.len());
    choices[idx]
  }
}


/// A Caraspace object, that can be stored
/// within a frame.
///
/// _For the connaiseur: This is basically a 
/// TofPacket on steroids_
///
///
#[derive(Debug, Clone)]
pub struct CRFrameObject {
  pub version : u8,
  pub ftype   : CRFrameObjectType,
  /// serialized representation of the 
  /// content object
  pub payload : Vec<u8>,
}

impl CRFrameObject {
  pub fn new() -> Self {
    Self {
      version : 0,
      ftype   : CRFrameObjectType::Unknown,
      payload : Vec::<u8>::new(),
    }
  }

  ///// The type of this object, which implicitly 
  ///// defines (de)serialization rules
  //pub fn get_ftype(&self) -> CRFrameObjectType {
  //  self.ftype
  //}

  /// Size of the serialized object, including
  /// header and footer in bytes
  pub fn size(&self) -> usize {
    let size = self.payload.len() + 2 + 4; 
    size
  }

  /// Unpack the TofPacket and return its content
  pub fn extract<T>(&self) -> Result<T, CRSerializationError>
    where T: Frameable + CRSerializeable {
    if T::CRFRAMEOBJECT_TYPE != self.ftype {
      error!("This bytestream is not for a {} packet!", self.ftype);
      return Err(CRSerializationError::IncorrectPacketType);
    }
    let unpacked : T = T::deserialize(&self.payload, &mut 0)?;
    Ok(unpacked)
  }
}

impl CRSerializeable for CRFrameObject {
  
  /// Decode a serializable from a bytestream  
  fn deserialize(stream : &Vec<u8>, 
                     pos    : &mut usize)
    -> Result<Self, CRSerializationError>
    where Self : Sized {
    if stream.len() < 2 {
      return Err(CRSerializationError::HeadInvalid {});
    }
    let head = parse_u16(stream, pos);
    if Self::CRHEAD != head {
      error!("Packet does not start with CRHEAD signature");
      return Err(CRSerializationError::HeadInvalid {});
    }
      let mut f_obj    = CRFrameObject::new();
      f_obj.version    = parse_u8(stream, pos);
      let ftype        = parse_u8(stream, pos);
      f_obj.ftype      = CRFrameObjectType::from(ftype);
      let payload_size = parse_u32(stream, pos);
      *pos += payload_size as usize; 
      let tail = parse_u16(stream, pos);
      if Self::CRTAIL != tail {
        error!("Packet does not end with CRTAIL signature");
        return Err(CRSerializationError::TailInvalid {});
      }
      *pos -= 2; // for tail parsing
      *pos -= payload_size as usize;
      f_obj.payload.extend_from_slice(&stream[*pos..*pos+payload_size as usize]);
      Ok(f_obj)
  }
  
  /// Encode a serializable to a bytestream  
  fn serialize(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::new();
    stream.extend_from_slice(&Self::CRHEAD.to_le_bytes());
    stream.push(self.version);
    stream.push(self.ftype.to_u8());
    let size = self.payload.len() as u32;
    stream.extend_from_slice(&size.to_le_bytes());
    stream.extend_from_slice(&self.payload.as_slice());
    stream.extend_from_slice(&Self::CRTAIL.to_le_bytes());
    stream
  }
}

impl fmt::Display for CRFrameObject {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let p_len = self.payload.len();
    write!(f, "<CRFrameObject: type {:?}, payload [ {} {} {} {} .. {} {} {} {}] of size {} >",
           self.ftype,
           self.payload[0], self.payload[1], self.payload[2], self.payload[3],
           self.payload[p_len-4], self.payload[p_len-3], self.payload[p_len - 2], self.payload[p_len-1], p_len ) 
  }
}



/// Allows to pack a certain structure within 
/// a CRFrameObject
pub trait Frameable {
  const CRFRAMEOBJECT_TYPE : CRFrameObjectType;

  /// Wrap myself in a TofPacket
  fn pack(&self) -> CRFrameObject 
    where Self: CRSerializeable {
    let mut cr     = CRFrameObject::new();
    cr.payload     = self.serialize();
    cr.ftype       = Self::CRFRAMEOBJECT_TYPE;
    //cr.size        = cr.payload.len();
    cr
  }
}

/// The central data container of the 
/// caraspace suite. 
///
/// A CRFrame can hold multiple CRFrameObjects
/// and is basically a little sclerite of 
/// the entire skeleton.
#[derive(Debug, Clone)]
pub struct CRFrame {
  // the index holds name, position in frame as well as the type of 
  // object stored in the frame
  pub index       : HashMap<String, (u64, CRFrameObjectType)>,
  pub bytestorage : Vec<u8>,
}

impl CRFrame {
  
  pub fn new() -> Self {
    Self {
      index       : HashMap::<String, (u64, CRFrameObjectType)>::new(),
      bytestorage : Vec::<u8>::new(),
    }
  }

  pub fn serialize_index(&self) -> Vec<u8> {
    let mut s_index  = Vec::<u8>::new();
    // more than 255 frame items are not supported
    let idx_size = self.index.len() as u8;
    s_index.push(idx_size);
    for k in &self.index {
      let mut s_name  = Self::string_to_bytes(k.0.clone());
      let s_pos   = k.1.0.to_le_bytes();
      s_index.append(&mut s_name);
      s_index.extend_from_slice(&s_pos);
      s_index.push(k.1.1.to_u8());
    }
    s_index
  }

  fn string_to_bytes(value : String) -> Vec<u8> {
    let mut stream  = Vec::<u8>::new();
    let mut payload = value.into_bytes();
    let string_size = payload.len() as u16; // limit size
    stream.extend_from_slice(&string_size.to_le_bytes());
    stream.append(&mut payload);
    stream
  }


  pub fn parse_index(stream : &Vec<u8>, pos : &mut usize) -> HashMap<String, (u64, CRFrameObjectType)> {
    let idx_size = parse_u8(stream, pos);
    //println!("Found index of size {idx_size}");
    let mut index    = HashMap::<String, (u64, CRFrameObjectType)>::new();
    for _ in 0..idx_size as usize {
      let name    = parse_string(stream, pos);
      let obj_pos = parse_u64(stream, pos);
      let obj_t   = CRFrameObjectType::from(stream[*pos]);
      *pos += 1;
      //println!("-- {} {} {}", name, obj_pos, obj_t);
      index.insert(name, (obj_pos, obj_t));
    }
    index
  }

  /// Store any eligible object in the frame
  ///
  /// Eligible object must implement the "Frameable" trait
  pub fn put<T: CRSerializeable + Frameable>(&mut self, object : T, name : String) {
    let f_object = object.pack();
    self.put_fobject(f_object, name);
  }

  fn put_fobject(&mut self, object : CRFrameObject, name : String) {
    let pos    = self.bytestorage.len() as u64;
    self.index.insert(name, (pos, object.ftype));
    let mut stream = object.serialize();
    //self.put_stream(&mut stream, name);
    //let pos    = self.bytestorage.len();
    //self.index.insert(name, pos);
    self.bytestorage.append(&mut stream);
  }

  //pub fn put_stream(&mut self, stream : &mut Vec<u8>, name : String) {
  //  let pos    = self.bytestorage.len();
  //  self.index.insert(name, pos);
  //  self.bytestorage.append(stream);
  //}

  pub fn get<T : CRSerializeable + Frameable>(&self, name : String) -> Result<T, CRSerializationError> {
    
    //let mut lookup : (usize, CRFrameObjectType);
    let mut pos    : usize;
    match self.index.get(&name) {
      None => {
        return Err(CRSerializationError::ValueNotFound);
      }
      Some(meta)  => {
        //lookup = meta;
        pos   = meta.0 as usize;
      }
    }
    let cr_object = CRFrameObject::deserialize(&self.bytestorage, &mut pos)?;
    let result    = cr_object.extract::<T>()?;
    Ok(result)
  }

  /// A verbose display of the frame content
  pub fn show_frame(&self) -> String {
    let mut repr = String::from("");
    for k in &self.index {
      repr += &(format!("\n -- {}@{}:{} --", k.0, k.1.0, k.1.1));
      //match k.1.1 {
      //  CRFrameObjectType::TelemetryPacket => {
      //    repr += &(format!("\n -- -- {}", self.get<TelemetryPacket>
      //  }
      //  CRFrameObjectType::TofPacket => {
      //  }
      //}
    }
    repr 
  }
}

impl CRSerializeable for CRFrame {
  /// Decode a serializable from a bytestream  
  fn deserialize(stream : &Vec<u8>, 
                 pos    : &mut usize)
    -> Result<Self, CRSerializationError> {
    if stream.len() < 2 {
      return Err(CRSerializationError::HeadInvalid {});
    }
    let head = parse_u16(stream, pos);
    if Self::CRHEAD != head {
      error!("FrameObject does not start with HEAD signature");
      return Err(CRSerializationError::HeadInvalid {});
    }
    let fr_size   = parse_u64(stream, pos) as usize; 
    *pos += fr_size as usize;
    let tail = parse_u16(stream, pos);
    if Self::CRTAIL != tail {
      error!("FrameObject does not end with TAIL signature");
      return Err(CRSerializationError::TailInvalid {});
    }
    *pos -= fr_size - 2; // wind back
    let mut frame = CRFrame::new();
    let size    = parse_u64(stream, pos) as usize;
    frame.index = Self::parse_index(stream, pos);
    frame.bytestorage = stream[*pos..*pos + size].to_vec();
    Ok(frame)
  }
  
  /// Encode a serializable to a bytestream  
  fn serialize(&self) -> Vec<u8> {
    let mut stream  = Vec::<u8>::new();
    stream.extend_from_slice(&Self::CRHEAD.to_le_bytes());
    let mut s_index = self.serialize_index();
    //let idx_size    = s_index.len() as u64;
    let size = self.bytestorage.len() as u64 + s_index.len() as u64;
    //println!("Will store frame with {size} bytes!");
    stream.extend_from_slice(&size.to_le_bytes());
    stream.append(&mut s_index);
    stream.extend_from_slice(&self.bytestorage.as_slice());
    stream.extend_from_slice(&Self::CRTAIL.to_le_bytes());
    stream
  }
}

impl fmt::Display for CRFrame {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<CRFrame : ");
    repr += &self.show_frame();
    repr += "\n>";
    write!(f, "{}", repr)
  }
}

