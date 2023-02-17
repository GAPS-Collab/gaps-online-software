//! Serialization/Deserialization helpers
//!
//!


pub use crate::errors::SerializationError;


/// Get u32 from a bytestream and move on the position marker
///
/// # Arguments 
///
/// * bs
/// * pos 
//pub fn u32_from_bs(bs : &Vec::<u8>, mut pos : usize) -> u32 {
//  let value = u32::from_le_bytes([bs[pos], bs[pos+1], bs[pos+2], bs[pos+3]]);
//  pos += 4;
//  value
//}


/// En/Decode to a bytestream, that is `Vec<u8>`
pub trait Serialization {


  ///! Decode a serializable from a bytestream  
  fn from_bytestream(bytestream : &Vec<u8>, 
                     start_pos  : usize)
    -> Result<Self, SerializationError>
    where Self : Sized;

  fn from_slice(slice     : &[u8],
                start_pos : usize)
    -> Result<Self, SerializationError>
    where Self : Sized {
    println!("There can't be a default implementation for this trait!");
    todo!();
    }


  /// Construct byte slice out of self.
  ///
  /// Can not fail.
  fn to_slice(&self) 
    -> &[u8]
    where Self : Sized {
    println!("There can't be a default implementation for this trait!");
    todo!();
    }
  
  //pub fn to_bytestream(&self, &

  /////! Add the payload of the serializable to the pre allocated bytestream
  //fn into_bytestream(bytestream : &mut Vec<u8>,
  //                   start_pos  : usize)
  //  -> Result<Self, SerializationError>
  //  where Self : Sized;
}




/// Search for a certain number of type `u16` in a bytestream
pub fn search_for_u16(number : u16, bytestream : &Vec<u8>, start_pos : usize) 
  -> Result<usize, SerializationError> {

  if start_pos > bytestream.len() - 1 {
    return Err(SerializationError::StreamTooShort);
  }

  let mut pos = start_pos;

  let mut two_bytes : [u8;2]; 
  // will find the next header
  two_bytes = [bytestream[pos], bytestream[pos + 1]];
  // FIXME - this should be little endian?
  if u16::from_be_bytes(two_bytes) == number {
    return Ok(pos);
  }
  // if it is not at start pos, then traverse 
  // the stream
  pos += 2;
  let mut found = false;
  if u16::from_be_bytes(two_bytes) != number {
    // we search for the next packet
    for n in pos..bytestream.len() - 1 {
      two_bytes = [bytestream[n], bytestream[n + 1]];
      if (u16::from_be_bytes(two_bytes)) == number {
        pos = n;
        found = true;
        break;
      }
    }
    if !found {
      return Err(SerializationError::ValueNotFound);
    }
  }
  trace!("Found {number} at {pos}");
  Ok(pos)
}

#[cfg(test)]
mod test_serialization {
  use crate::serialization::search_for_u16;

  #[test]
  fn test_search_for_2_bytemarker()
  {
    // just test it two times - FIXME - use a better method
    let mut bytestream = vec![1,2,3,0xAA, 0xAA, 5, 7];
    let mut pos = search_for_u16(0xAAAA, &bytestream, 0).unwrap();
    assert_eq!(pos, 3);
    
    bytestream = vec![1,2,3,244, 16, 32, 0xff, 0xAA, 5, 7];
    pos = search_for_u16(65450, &bytestream, 1).unwrap();
    assert_eq!(pos, 6);
    
    bytestream = vec![0xaa,0xaa,3,244, 16, 32, 0xff, 0xAA, 5, 7];
    pos = search_for_u16(0xaaaa, &bytestream, 0).unwrap();
    assert_eq!(pos, 0);
  }
}
