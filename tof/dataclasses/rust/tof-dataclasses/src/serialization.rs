pub use crate::errors::SerializationError;

pub trait Serialization {


  ///! Decode a serializable from a bytestream  
  fn from_bytestream(bytestream : &Vec<u8>, 
                     start_pos  : usize)
    -> Result<Self, SerializationError>
    where Self : Sized;

  //pub fn to_bytestream(&self, &

  /////! Add the payload of the serializable to the pre allocated bytestream
  //fn into_bytestream(bytestream : &mut Vec<u8>,
  //                   start_pos  : usize)
  //  -> Result<Self, SerializationError>
  //  where Self : Sized;
}




///! check for a certain number in a bytestream
pub fn search_for_u16(number : u16, bytestream : &Vec<u8>, start_pos : usize) 
  -> Result<usize, SerializationError> {

  if start_pos > bytestream.len() {
    return Err(SerializationError::StreamTooShort);
  }

  let mut pos = start_pos;

  let mut two_bytes : [u8;2]; 
  // will find the next header
  two_bytes = [bytestream[pos], bytestream[pos + 1]];
  if u16::from_be_bytes(two_bytes) == number {
    return Ok(pos);
  }
  // if it is not at start pos, then traverse 
  // the stream
  pos += 2;
  let mut found = false;
  if u16::from_be_bytes(two_bytes) != number {
    // we search for the next packet
    for n in pos..bytestream.len() {
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
