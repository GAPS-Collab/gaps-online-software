//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

use crate::errors::SerializationError;

/// Encode/decode structs to `Vec::<u8>` to write to a file or
/// send over the network
pub trait Serialization {

  /// Byte marker to mark beginning of payload
  const HEAD: u16;
  /// Byte marker to mark end of payload
  const TAIL: u16;
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
    let head_pos = search_for_u16(Self::HEAD, stream, *pos)?; 
    let tail_pos = search_for_u16(Self::TAIL, stream, head_pos + Self::SIZE-2)?;
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

  ///// Decode a serializable directly from a TofPacket
  //fn from_tofpacket(packet : &TofPacket)
  //  -> Result<Self, SerializationError>
  //  where Self: Sized {
  //  let unpacked = Self::from_bytestream(&packet.payload, &mut 0)?;
  //  Ok(unpacked)
  //}

  /// Encode a serializable to a bytestream  
  /// 
  /// This shall return a representation of the struct
  /// in such a way that to_bytestream and from_bytestream
  /// are inverse operations.
  fn to_bytestream(&self) -> Vec<u8>;

  //fn from_slice(_slice     : &[u8],
  //              _start_pos : usize)
  //  -> Result<Self, SerializationError>
  //  where Self : Sized {
  //  println!("There can't be a default implementation for this trait!");
  //  todo!();
  //  }

  ///// Construct byte slice out of self.
  /////
  ///// Can not fail.
  //fn to_slice(&self) 
  //  -> &[u8]
  //  where Self : Sized {
  //  println!("There can't be a default implementation for this trait!");
  //  todo!();
  //}
}



/// Search for a certain number of type `u16` in a bytestream
///
/// # Arguments:
///   * number     : The number to search for
///   * bytestream : The data to search the number in  
///   * start_pos  : Skip "start_pos" bytes from the beginning when searchinig
///
/// # Returns:
///   * position in bytestream where number is found. If the number is not found, 
///     return SerializationError::ValueNotFound
pub fn search_for_u16(number : u16, bytestream : &Vec<u8>, start_pos : usize) 
  -> Result<usize, SerializationError> {
  // -2 bc later on we are looking for 2 bytes!
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
  two_bytes = [bytestream[pos], bytestream[pos + 1]];
  if u16::from_le_bytes(two_bytes) == number {
    return Ok(pos);
  }
  // if it is not at start pos, then traverse 
  // the stream
  pos += 1;
  let mut found = false;
  for n in pos..bytestream.len() - 1 {
    two_bytes = [bytestream[n], bytestream[n + 1]];
    if (u16::from_le_bytes(two_bytes)) == number {
      pos = n;
      found = true;
      break;
    }
  }
  if !found {
    let delta = bytestream.len() - start_pos;
    warn!("Can not find {} in bytestream [-{}:{}]!", number, delta ,bytestream.len());
    return Err(SerializationError::ValueNotFound);
  }
  trace!("Found {number} at {pos}");
  Ok(pos)
}

#[test]
fn test_search_for_u16() {
  // just test it two times - FIXME - use a better method
  let mut bytestream = vec![1,2,3,0xAA, 0xAA, 5, 7];
  let mut pos = search_for_u16(0xAAAA, &bytestream, 0).unwrap();
  assert_eq!(pos, 3);
  
  bytestream = vec![1,2,3,244, 16, 32, 0xaa, 0xff, 5, 7];
  pos = search_for_u16(65450, &bytestream, 1).unwrap();
  assert_eq!(pos, 6);
  
  bytestream = vec![0xaa,0xaa,3,244, 16, 32, 0xAA, 0xFF, 5, 7];
  pos = search_for_u16(0xaaaa, &bytestream, 0).unwrap();
  assert_eq!(pos, 0);
}



