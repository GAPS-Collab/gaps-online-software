//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

use crate::prelude::*;

pub use crate::io::caraspace::{
  Frameable
};

/// Encode/decode structs to `Vec::<u8>` to write to a file or
/// send over the network
pub trait Serialization {

  /// Byte marker to mark beginning of payload
  const HEAD: u16 = 0xAAAA;
  /// Byte marker to mark end of payload
  const TAIL: u16 = 0x5555;
  /// The SIZE is the size of the serialized 
  /// bytestream INCLUDING 4 bytes for head
  /// and tail bytes. In case the struct does 
  /// NOT HAVE a fixed size, SIZE will be 0
  /// (so default value of the trait
  const SIZE: usize = 0;

  /// Verify that the serialized representation of the struct has the 
  /// correct size, including header + footer.
  ///
  /// Will panic for variable sized structs.
  fn verify_fixed(stream : &Vec<u8>, 
                  pos    : &mut usize) -> Result<(), SerializationError> {
    if !Self::SIZE == 0 {
      // we can panic here, since this is a conceptional logic error. If we
      // don't panic, monsters will arise downstream.
      panic!("Self::verify_fixed can be only used for structs with a fixed size! In case you are convinced, that your struct has indeed a fixed size, please implement trait Serialization::SIZE with the serialized size in bytes including 4 bytes for header and footer!");
    }
    let head_pos = seek_marker(stream, Self::HEAD, *pos)?; 
    let tail_pos = seek_marker(stream, Self::TAIL, head_pos + Self::SIZE-2)?;
    if tail_pos + 2 - head_pos != Self::SIZE {
      error!("Seing {} bytes, but expecting {}", tail_pos + 2 - head_pos, Self::SIZE);
      *pos = head_pos + 2; 
      return Err(SerializationError::WrongByteSize);
    }
    *pos = head_pos + 2;
    Ok(())
  } 

  /// Decode a serializable from a bytestream  
  ///
  /// # Arguments:
  ///   * bytestream : bytes including the ones which should 
  ///                  be decoded
  ///   * pos        : first byte in the bytestream which is 
  ///                  part of the expected payload
  //fn from_bytestream<T: AsRef<[u8]>>(bytestream : T, 
  fn from_bytestream(bytestream : &Vec<u8>,
                     pos        : &mut usize)
    -> Result<Self, SerializationError>
    where Self : Sized;
  
  /// Decode a serializable from a bytestream. This provides 
  /// an alternative method to get the packet. If not implemented,
  /// it will be the same as from_bytestream.
  ///
  /// # Arguments:
  ///   * bytestream : bytes including the ones which should 
  ///                  be decoded
  ///   * pos        : first byte in the bytestream which is 
  ///                  part of the expected payload
  fn from_bytestream_alt(bytestream : &Vec<u8>, 
                         pos        : &mut usize)
    -> Result<Self, SerializationError>
    where Self : Sized {
    Self::from_bytestream(bytestream, pos)
  }

  /// Encode a serializable to a bytestream  
  /// 
  /// This shall return a representation of the struct
  /// in such a way that to_bytestream and from_bytestream
  /// are inverse operations.
  fn to_bytestream(&self) -> Vec<u8> {
    error!("No default implementation for trait!");
    return Vec::<u8>::new();
  }
}

//---------------------------------------------------

/// Search for a u16 bytemarker in a stream.
///
/// E.g. This can be an 0xAAAA indicator as a packet delimiter
///
/// # Arguments:
///  
///  * marker     : The marker to search for. Currently, only 
///                 16bit markers are supported
///  * bytestream : The stream to search the number in
///  * start_pos  : Start searching from this position in 
///                 the stream
pub fn seek_marker<T: AsRef<[u8]>>(stream : &T, marker : u16, start_pos :usize) 
  -> Result<usize, SerializationError> {
  // -2 bc later on we are looking for 2 bytes!
  let bytestream = stream.as_ref();
  if bytestream.len() == 0 {
    error!("Stream empty!");
    return Err(SerializationError::StreamTooShort);
  }
  if start_pos  > bytestream.len() - 2 {
    error!("Start position {} beyond stream capacity {}!", start_pos, bytestream.len() -2);
    return Err(SerializationError::StreamTooShort);
  }
  let mut pos = start_pos;
  let mut two_bytes : [u8;2]; 
  // will find the next header
  two_bytes = [bytestream[pos], bytestream[pos + 1]];
  // FIXME - this should be little endian?
  if u16::from_le_bytes(two_bytes) == marker {
    return Ok(pos);
  }
  // if it is not at start pos, then traverse 
  // the stream
  pos += 1;
  let mut found = false;
  // we search for the next packet
  for n in pos..bytestream.len() - 1 {
    two_bytes = [bytestream[n], bytestream[n + 1]];
    if (u16::from_le_bytes(two_bytes)) == marker {
      pos = n;
      found = true;
      break;
    }
  }
  if !found {
    let delta = bytestream.len() - start_pos;
    warn!("Can not find {} in bytestream [-{}:{}]!", marker, delta ,bytestream.len());
    return Err(SerializationError::ValueNotFound);
  }
  trace!("Found {marker} at {pos}");
  Ok(pos)
}

#[test]
fn test_seek_marker() {
  // just test it two times - FIXME - use a better method
  let mut bytestream = vec![1,2,3,0xAA, 0xAA, 5, 7];
  let mut pos = seek_marker(&bytestream, 0xaaaa, 0).unwrap();
  assert_eq!(pos, 3);
  
  bytestream = vec![1,2,3,244, 16, 32, 0xaa, 0xff, 5, 7];
  // remember byte order in vectors
  pos = seek_marker(&bytestream, 0xffaa, 1).unwrap();
  assert_eq!(pos, 6);
  
  bytestream = vec![0xaa,0xaa,3,244, 16, 32, 0xAA, 0xFF, 5, 7];
  pos = seek_marker(&bytestream, 0xaaaa, 0).unwrap();
  assert_eq!(pos, 0);
}

