//! Caraapace I/O system
//!
// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// The Caraspace object type determines the 
/// kind of object we are able to put in 
/// a frame and ultimate (de)serialize
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
#[derive(Debug, Copy, Clone, PartialEq,FromRepr, AsRefStr, EnumIter)]
#[repr(u8)]
pub enum CRFrameObjectType {
  Unknown          =  0u8,
  TofPacket        = 10u8,
  TelemetryPacket  = 20u8,
}

expand_and_test_enum!(CRFrameObjectType, test_crframeobjecttype_repr);

//---------------------------------------------------

/// A Caraspace object, that can be stored
/// within a frame.
///
/// _For the connaiseur_: This is basically a 
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

  /// Size of the serialized object, including
  /// header and footer in bytes
  pub fn size(&self) -> usize {
    let size = self.payload.len() + 2 + 4; 
    size
  }

  /// Unpack the TofPacket and return its content
  pub fn extract<T>(&self) -> Result<T, SerializationError>
    where T: Frameable + Serialization {
    if T::CRFRAMEOBJECT_TYPE != self.ftype {
      error!("This bytestream is not for a {} packet!", self.ftype);
      return Err(SerializationError::IncorrectPacketType);
    }
    let unpacked : T = T::from_bytestream(&self.payload, &mut 0)?;
    Ok(unpacked)
  }
}

impl Serialization for CRFrameObject {
  
  /// Decode a serializable from a bytestream  
  fn from_bytestream(stream : &Vec<u8>, 
                     pos    : &mut usize)
    -> Result<Self, SerializationError>
    where Self : Sized {
    if stream.len() < 2 {
      return Err(SerializationError::HeadInvalid {});
    }
    let head = parse_u16(stream, pos);
    if Self::HEAD != head {
      error!("Packet does not start with CRHEAD signature");
      return Err(SerializationError::HeadInvalid {});
    }
      let mut f_obj    = CRFrameObject::new();
      f_obj.version    = parse_u8(stream, pos);
      let ftype        = parse_u8(stream, pos);
      f_obj.ftype      = CRFrameObjectType::from(ftype);
      let payload_size = parse_u32(stream, pos);
      *pos += payload_size as usize; 
      let tail = parse_u16(stream, pos);
      if Self::TAIL != tail {
        error!("Packet does not end with CRTAIL signature");
        return Err(SerializationError::TailInvalid {});
      }
      *pos -= 2; // for tail parsing
      *pos -= payload_size as usize;
      f_obj.payload.extend_from_slice(&stream[*pos..*pos+payload_size as usize]);
      Ok(f_obj)
  }
  
  /// Encode a serializable to a bytestream  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::new();
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.push(self.version);
    stream.push(self.ftype as u8);
    let size = self.payload.len() as u32;
    stream.extend_from_slice(&size.to_le_bytes());
    stream.extend_from_slice(&self.payload.as_slice());
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
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

//---------------------------------------------------

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
  // FIXME - this needs to be HashMap<&str, (u64, CRFrameObjectType)>
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
      s_index.push(k.1.1 as u8);
    }
    s_index
  }

  /// Get the timestamp from the actual telemetry packet in the frame
  pub fn get_timestamp(&self) -> u64 {
    todo!("Needs to be implemented!");
    return 0
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

  /// Delete a CRFrameObject by this name from the frame
  ///
  /// To delete multiple objects, delete calls can be 
  /// chained
  /// 
  /// # Arguments:
  ///   * name : The name of the FrameObject to delte 
  ///            (must be in index)
  ///
  /// # Returns:
  ///   A complete copy of self, without the given object.
  pub fn delete(&self, name : &str) -> Result<CRFrame, SerializationError> {
    if !self.has(name) {
      error!("There is no object with name {} in this frame!", name);
      return Err(SerializationError::ObjectNotFound);
    }
    let mut new_frame = CRFrame::new();
    for objname in self.index.keys() {
      if objname == name {
        continue;
      }
      let obj = self.get_fobject(&objname)?;
      new_frame.put_fobject(obj, objname.clone());
    }
    Ok(new_frame)
  }


  /// Store any eligible object in the frame
  ///
  /// Eligible object must implement the "Frameable" trait
  pub fn put<T: Serialization + Frameable>(&mut self, object : T, name : String) {
    let f_object = object.pack();
    self.put_fobject(f_object, name);
  }

  fn put_fobject(&mut self, object : CRFrameObject, name : String) {
    let pos    = self.bytestorage.len() as u64;
    self.index.insert(name, (pos, object.ftype));
    let mut stream = object.to_bytestream();
    //self.put_stream(&mut stream, name);
    //let pos    = self.bytestorage.len();
    //self.index.insert(name, pos);
    self.bytestorage.append(&mut stream);
  }

  /// Check if the frame contains an object with the given name
  ///
  /// # Arguments:
  ///   * name : The name of the object as it appears in the index
  pub fn has(&self, name : &str) -> bool {
    self.index.contains_key(name)
  }
  //pub fn put_stream(&mut self, stream : &mut Vec<u8>, name : String) {
  //  let pos    = self.bytestorage.len();
  //  self.index.insert(name, pos);
  //  self.bytestorage.append(stream);
  //}

  pub fn get_fobject(&self, name : &str) -> Result<CRFrameObject, SerializationError> {
    let mut pos    : usize;
    match self.index.get(name) {
      None => {
        error!("There is no object with name {} in this frame!", name);
        return Err(SerializationError::ObjectNotFound);
      }
      Some(meta)  => {
        //lookup = meta;
        pos   = meta.0 as usize;
      }
    }
    let cr_object = CRFrameObject::from_bytestream(&self.bytestorage, &mut pos)?;
    Ok(cr_object)
  }

  pub fn get<T : Serialization + Frameable>(&self, name : String) -> Result<T, SerializationError> {
    
    //let mut lookup : (usize, CRFrameObjectType);
    let mut pos    : usize;
    match self.index.get(&name) {
      None => {
        return Err(SerializationError::ValueNotFound);
      }
      Some(meta)  => {
        //lookup = meta;
        pos   = meta.0 as usize;
      }
    }
    let cr_object = CRFrameObject::from_bytestream(&self.bytestorage, &mut pos)?;
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

impl Default for CRFrame {
  fn default() -> Self {
    Self::new()
  }
}

impl Serialization for CRFrame {
  /// Decode a serializable from a bytestream  
  fn from_bytestream(stream : &Vec<u8>, 
                     pos    : &mut usize)
    -> Result<Self, SerializationError> {
    if stream.len() < 2 {
      return Err(SerializationError::HeadInvalid {});
    }
    let head = parse_u16(stream, pos);
    if Self::HEAD != head {
      error!("FrameObject does not start with HEAD signature");
      return Err(SerializationError::HeadInvalid {});
    }
    let fr_size   = parse_u64(stream, pos) as usize; 
    *pos += fr_size as usize;
    let tail = parse_u16(stream, pos);
    if Self::TAIL != tail {
      error!("FrameObject does not end with TAIL signature");
      return Err(SerializationError::TailInvalid {});
    }
    *pos -= fr_size - 2; // wind back
    let mut frame = CRFrame::new();
    let size    = parse_u64(stream, pos) as usize;
    frame.index = Self::parse_index(stream, pos);
    frame.bytestorage = stream[*pos..*pos + size].to_vec();
    Ok(frame)
  }
  
  /// Encode a serializable to a bytestream  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream  = Vec::<u8>::new();
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    let mut s_index = self.serialize_index();
    //let idx_size    = s_index.len() as u64;
    let size = self.bytestorage.len() as u64 + s_index.len() as u64;
    //println!("Will store frame with {size} bytes!");
    stream.extend_from_slice(&size.to_le_bytes());
    stream.append(&mut s_index);
    stream.extend_from_slice(&self.bytestorage.as_slice());
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
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

