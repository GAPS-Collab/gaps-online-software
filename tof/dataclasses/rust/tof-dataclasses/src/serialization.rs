use crate::errors::SerializationError;

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
  pos += 2;

  let mut found = false;
  if (u16::from_le_bytes(two_bytes)) != number {
    // we search for the next packet
    for n in pos..bytestream.len() {
      two_bytes = [bytestream[pos], bytestream[pos + 1]];
      if (u16::from_le_bytes(two_bytes)) == number {
        pos += n;
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

