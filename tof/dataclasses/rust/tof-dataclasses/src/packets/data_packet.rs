use crate::packets::generic_packet::GenericPacket;
use crate::serialization::Serialization;
use crate::errors::SerializationError;
///! A packet with a single value and a 
///  label
///
///  The package layout in binary is like this
///  HEAD       : u16 = 0xAAAA
///  LABEL_SIZE : u8
///  LABEL      : [u8;LABEL_SIZE]
///  DATA       : [u8;LABEL_SIZE + sizeof( u8 // .. // u64)]
///  TAIL       : u16 = 0x5555
///
///  Total size of the packet : 5 + LABELSIZE + sizeof( u8 // .. // u64)
///
#[derive(Debug)]
pub struct DataPacket<T> {
  pub data  : T,
  pub label : String
}

//pub type CommandPacket = DataPacket::<u32>;

impl DataPacket::<u8> {
  pub fn from_gp(packet : &GenericPacket)
    -> Option<DataPacket<u8>>{
    let label = &packet.label;
    match packet.payload_size {
      1 => Some(DataPacket::<u8> {
                label :label.to_string(),
                data  : packet.payload[0]
           }),
      _ => None
    }
  }
}

impl DataPacket::<u16> {
  pub fn from_gp(packet : &GenericPacket)
    -> Option<DataPacket<u16>>{
    let label = &packet.label;
    match packet.payload_size {
      2 => Some(DataPacket::<u16> {
                label :label.to_string(),
                data  : u16::from_le_bytes([packet.payload[0],
                                            packet.payload[1]])
           }),
      _ => None
    }
  }
}

impl Serialization for DataPacket::<u32> {
  fn from_bytestream(stream : &Vec<u8>, 
                     start_pos : usize) 
    -> Result<DataPacket<u32>, SerializationError>{
  
    let mut pos      = start_pos; 
    let mut two_bytes : [u8;2];
    let four_bytes    : [u8;4];
    two_bytes = [stream[pos],
                 stream[pos+1]];
    pos += 2;
    if DataPacket::<u32>::HEAD != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not start with HEAD signature");
      return Err(SerializationError::HeadInvalid {});
    }
   
    let label_size   = stream[pos];
    pos += 1;
    let label        = String::from_utf8((&stream[pos..label_size as usize + pos]).to_vec()).unwrap();
    pos += label_size as usize;
    // Here we know that the packet is containing a single u32
    // Otherwise, it is a different packet
    if stream.len() < 5 - label_size as usize {
      trace!("We have a payload size of {}", stream.len());
      return Err(SerializationError::StreamTooShort);
    }
    let payload_size = stream.len() - 5 - label_size as usize;
    //let payload_size = bytestream[3+label_size as usize];
    if payload_size != 4 {
      trace!("We have a payload size of {}", payload_size);
      return Err(SerializationError::WrongByteSize);
    }
    four_bytes = [stream[5 + label_size as usize],
                  stream[5 + label_size as usize + 1],
                  stream[5 + label_size as usize + 2],
                  stream[5 + label_size as usize + 3]];
    pos += 4;
    let data = u32::from_le_bytes(four_bytes);
    two_bytes = [stream[pos],
                 stream[pos+1]];
    if DataPacket::<u32>::TAIL != u16::from_le_bytes(two_bytes) {
      warn!("Packet does not end with TAIL signature");
      return Err(SerializationError::TailInvalid {});
    }
    Ok(DataPacket::<u32>{
        label, 
        data
    })   
  }
}

impl DataPacket::<u32> {

  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;

  //pub fn from_gp(packet : &GenericPacket)
  //  -> Option<DataPacket<u32>>{
  //  let label = &packet.label;
  //  match packet.payload_size {
  //    4 => Some(DataPacket::<u32> {
  //              label :label.to_string(),
  //              data  : u32::from_le_bytes([packet.payload[0],
  //                                          packet.payload[1],
  //                                          packet.payload[2],
  //                                          packet.payload[3]])

  //         }),
  //    _ => None
  //  }
  //}
}

impl DataPacket::<u64> {
  pub fn from_gp(packet : &GenericPacket)
    -> Option<DataPacket<u64>>{
    let label = &packet.label;
    match packet.payload_size {
      8 => Some(DataPacket::<u64> {
                label :label.to_string(),
                data  : u64::from_le_bytes([packet.payload[0],
                                            packet.payload[1],
                                            packet.payload[2],
                                            packet.payload[3],
                                            packet.payload[4],
                                            packet.payload[5],
                                            packet.payload[6],
                                            packet.payload[7]])

           }),
      _ => None
    }
  }
}

