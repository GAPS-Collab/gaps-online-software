//! Packets are a way to send data over the network.
//!
//! Data gets serialized to a bytestream and then 
//! header and tail bytes are added to the front and
//! end of the stream.
//!
//! A TofPacket has the following layout
//! HEAD    : u16 = 0xAAAA
//! TYPE    : u8  = PacketType
//! SIZE    : u32
//! PAYLOAD : [u8;6-SIZE]
//! TAIL    : u16 = 0x5555 
//!
//! The total packet size is thus 13 + SIZE



// re-imports
use std::time::Instant;
use std::fmt;
pub use crate::monitoring::{
    RBMoniData,
    PBMoniData,
    LTBMoniData,
    PAMoniData,
    MtbMoniData,
    CPUMoniData
};

use crate::serialization::{
    Serialization, 
    parse_u8,
    parse_u16,
    parse_u32
};

use std::error::Error;
use crate::errors::{
    SerializationError,
    PacketError
};

use crate::events::{
    RBEventHeader,
    RBEvent,
    MasterTriggerEvent,
    TofEvent,
    RBWaveform,
    TofEventSummary,
};

use crate::commands::{RBCommand, TofCommand, TofResponse};
use crate::calibrations::RBCalibrations;

pub mod packet_type;
pub use packet_type::PacketType;

/// The most basic of all packets
///  
/// A type and a payload. This wraps
/// all other packets.
///
/// Format when in bytestream
/// HEAD : u16

/// PAYLOAD_SIZE : u32
/// PYALOAD : [u8;PAYLOAD_SIZE]
/// TAIL : u16
///
/// => Fixed size is 13
///
#[derive(Debug, Clone)]
pub struct TofPacket {
  pub packet_type        : PacketType,
  pub payload            : Vec<u8>,
  // FUTURE EXTENSION: Be able to send
  /// packets which contain multiple of the same packets
  pub is_multi_packet    : bool,
  // fields which won't get serialized
  /// mark a packet as not eligible to be written to disk
  pub no_write_to_disk   : bool,
  /// mark a packet as not eligible to be sent over network 
  /// FIXME - future extension
  pub no_send_over_nw    : bool,
  /// creation_time for the instance
  pub creation_time    : Instant,
  pub valid            : bool, // will be always valid, unless invalidated
}

impl fmt::Display for TofPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let p_len = self.payload.len();
    if p_len < 4 {
      write!(f, "<TofPacket: type {:?}, payload size {}>", self.packet_type, p_len)
    } else {
      write!(f, "<TofPacket: type {:?}, multi {}, payload [ {} {} {} {} .. {} {} {} {}] of size {} >",
             self.packet_type,
             self.is_multi_packet,
             self.payload[0], self.payload[1], self.payload[2], self.payload[3],
             self.payload[p_len-4], self.payload[p_len-3], self.payload[p_len - 2], self.payload[p_len-1], p_len ) 
    }
  }
}

impl Default for TofPacket {
  fn default() -> Self {
    Self::new()
  }
}

/// Implement because TofPacket saves the creation time, 
/// which never will be the same for 2 different instances
impl PartialEq for TofPacket {
  fn eq(&self, other: &Self) -> bool {
    (self.packet_type == other.packet_type)           &&
    (self.payload == other.payload)                   &&
    (self.is_multi_packet == other.is_multi_packet)   &&
    (self.no_write_to_disk == other.no_write_to_disk) &&
    (self.no_send_over_nw == other.no_send_over_nw)   &&
    (self.valid == other.valid)
  }
}

impl TofPacket {

  pub const PRELUDE_SIZE : usize = 7; 
 
  pub fn new() -> Self {
    let creation_time = Instant::now();
    Self {
      packet_type      : PacketType::Unknown,
      payload          : Vec::<u8>::new(),
      is_multi_packet  : false,
      no_write_to_disk : false,
      no_send_over_nw  : false,
      creation_time    : creation_time,
      valid            : true,
    }
  }

  /// Unpack possible content
  pub fn unpack_rbevent(&self) -> Result<RBEvent, Box<dyn Error>> {
    if self.packet_type != PacketType::RBEvent {
      error!("We expeckt packet type {}, but got packet type {} instead!", PacketType::RBEvent, self.packet_type);
      return Err(Box::new(PacketError::WrongPacketType));
    }
    Ok(RBEvent::from_bytestream(&self.payload, &mut 0)?)
  }

  pub fn age(&self) -> u64 {
    self.creation_time.elapsed().as_secs()
  }
 
  pub fn get_n_packets(&self) -> u32 {
    if !self.is_multi_packet {
      return 1; 
    }
    todo!("Can not deal with multipackets right now!");
    #[allow(unreachable_code)] {
      return 1;
    }
  }
}

impl From<&RBWaveform> for TofPacket {
  fn from(rbwave : &RBWaveform) -> Self {
    let mut tp     = Self::new();
    tp.packet_type = PacketType::RBWaveform;
    tp.payload     = rbwave.to_bytestream();
    tp
  }
}

impl From<&TofEventSummary> for TofPacket {
  fn from(tsum : &TofEventSummary) -> Self {
    let mut tp     = Self::new();
    tp.packet_type = PacketType::TofEventSummary;
    tp.payload     = tsum.to_bytestream();
    tp
  }
}

impl From<&TofEvent> for TofPacket {
  fn from(event : &TofEvent) -> Self {
    let mut tp = Self::new();
    tp.packet_type = PacketType::TofEvent;
    tp.payload = event.to_bytestream();
    tp
  }
}

impl From<&mut TofEvent> for TofPacket {
  fn from(event : &mut TofEvent) -> Self {
    let mut tp     = Self::new();
    tp.packet_type = PacketType::TofEvent;
    tp.payload     = event.to_bytestream();
    tp
  }
}

impl From<&TofCommand> for TofPacket {
  fn from(cmd : &TofCommand) -> Self {
    let mut tp = Self::new();
    tp.packet_type = PacketType::TofCommand;
    tp.payload = cmd.to_bytestream();
    tp
  }
}

impl From<&RBCommand> for TofPacket {
  fn from(cmd : &RBCommand) -> Self {
    let mut tp = Self::new();
    tp.packet_type = PacketType::RBCommand;
    tp.payload = cmd.to_bytestream();
    tp
  }
}

impl From<&RBCalibrations> for TofPacket {
  fn from(calib : &RBCalibrations) -> Self {
    let mut tp = Self::new();
    tp.packet_type = PacketType::RBCalibration;
    tp.payload = calib.to_bytestream();
    tp
  }
}

impl From<&CPUMoniData> for TofPacket {
  fn from(moni : &CPUMoniData) -> Self {
    let mut tp = Self::new();
    tp.packet_type = PacketType::CPUMoniData;
    tp.payload     = moni.to_bytestream();
    tp
  }
}

impl From<&mut RBCalibrations> for TofPacket {
  fn from(calib : &mut RBCalibrations) -> Self {
    let mut tp = Self::new();
    tp.packet_type = PacketType::RBCalibration;
    tp.payload = calib.to_bytestream();
    tp
  }
}

impl From<&RBEvent> for TofPacket {
  fn from(event : &RBEvent) -> Self {
    let mut tp = Self::new();
    tp.packet_type = PacketType::RBEvent;
    tp.payload = event.to_bytestream();
    tp
  }
}

impl From<&MasterTriggerEvent> for TofPacket {
  fn from(mt : &MasterTriggerEvent) -> TofPacket {
    let mut tp     = TofPacket::new();
    tp.packet_type = PacketType::MasterTrigger;
    tp.payload     = mt.to_bytestream();
    tp
  }
}


impl From<&RBMoniData> for TofPacket {
  fn from(moni : &RBMoniData) -> Self {
    let mut tp     = Self::new();
    tp.packet_type = PacketType::RBMoni;
    tp.payload     = moni.to_bytestream();
    tp
  }
}

impl From<&PBMoniData> for TofPacket {
  fn from(moni : &PBMoniData) -> Self {
    let mut tp     = Self::new();
    tp.packet_type = PacketType::PBMoniData;
    tp.payload     = moni.to_bytestream();
    tp
  }
}
impl From<&LTBMoniData> for TofPacket {
  fn from(moni : &LTBMoniData) -> Self {
    let mut tp     = Self::new();
    tp.packet_type = PacketType::LTBMoniData;
    tp.payload     = moni.to_bytestream();
    tp
  }
}

impl From<&PAMoniData> for TofPacket {
  fn from(moni : &PAMoniData) -> Self {
    let mut tp     = Self::new();
    tp.packet_type = PacketType::PAMoniData;
    tp.payload     = moni.to_bytestream();
    tp
  }
}

impl From<&MtbMoniData> for TofPacket {
  fn from(moni : &MtbMoniData) -> TofPacket {
    let mut tp = TofPacket::new();
    tp.packet_type = PacketType::MonitorMtb;
    tp.payload = moni.to_bytestream();
    tp
  }
}

impl From<&RBEventHeader> for TofPacket {
  fn from(ev_header : &RBEventHeader) -> TofPacket {
    let mut tp     = TofPacket::new();
    tp.packet_type = PacketType::RBEventHeader;
    tp.payload     = ev_header.to_bytestream();
    tp
  }
}

impl From<&TofResponse> for TofPacket {
  fn from(ev_header : &TofResponse) -> TofPacket {
    let mut tp     = TofPacket::new();
    tp.packet_type = PacketType::TofResponse;
    tp.payload     = ev_header.to_bytestream();
    tp
  }
}

impl Serialization for TofPacket {
  const HEAD : u16 = 0xaaaa;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 0; // FIXME - size/prelude_size 

  fn from_bytestream(stream : &Vec<u8>, pos : &mut usize)
  -> Result<Self, SerializationError> {
    let head = parse_u16(stream, pos);
    if Self::HEAD != head {
      error!("Packet does not start with HEAD signature");
      return Err(SerializationError::HeadInvalid {});
    }
    let packet_type : PacketType;
    let packet_type_enc = parse_u8(stream, pos);
    match PacketType::try_from(packet_type_enc) {
      Ok(pt) => packet_type = pt,
      Err(_) => {
        error!("Can not decode packet with packet type {}", packet_type_enc);
        return Err(SerializationError::UnknownPayload);}
    }
    let payload_size = parse_u32(stream, pos);
    *pos += payload_size as usize; 
    let tail = parse_u16(stream, pos);
    if Self::TAIL != tail {
      error!("Packet does not end with TAIL signature");
      return Err(SerializationError::TailInvalid {});
    }
    *pos -= 2; // for tail parsing
    *pos -= payload_size as usize;

    let mut tp = TofPacket::new();
    tp.packet_type = packet_type;
    tp.payload.extend_from_slice(&stream[*pos..*pos+payload_size as usize]);
    Ok(tp) 
  }
  
  fn to_bytestream(&self) 
    -> Vec<u8> {
    if self.is_multi_packet {
      todo!("Can not deal with multipackets right now!");
    }
    let mut bytestream = Vec::<u8>::with_capacity(6 + self.payload.len());
    bytestream.extend_from_slice(&TofPacket::HEAD.to_le_bytes());
    let p_type = self.packet_type as u8;
    bytestream.push(p_type);
    // payload size of 32 bit accomodates up to 4 GB packet
    // a 16 bit size would only hold 65k, which might be not
    // good enough if we sent multiple events in a batch in 
    // the same TofPacket (in case we do that)
    let payload_len = self.payload.len() as u32;
    //let foo = &payload_len.to_le_bytes();
    //debug!("TofPacket binary payload: {foo:?}");
    bytestream.extend_from_slice(&payload_len.to_le_bytes());
    bytestream.extend_from_slice(self.payload.as_slice());
    bytestream.extend_from_slice(&TofPacket::TAIL.to_le_bytes());
    bytestream
  }
}

#[cfg(feature="random")]
#[test] 
fn tofpacket_from_rbevent() {
  let data = RBEvent::new();
  let pk   = TofPacket::from(&data);
  let test = TofPacket::from_bytestream(&pk.to_bytestream(),&mut 0).unwrap();
  assert_eq!(pk.packet_type, PacketType::RBEvent);
  assert_eq!(pk, test);
  let data_test = RBEvent::from_bytestream(&pk.payload, &mut 0).unwrap();
  assert_eq!(data, data_test);
}

#[cfg(feature="random")]
#[test] 
fn tofpacket_from_tofevent() {
  let data = TofEvent::new();
  let pk   = TofPacket::from(&data);
  let test = TofPacket::from_bytestream(&pk.to_bytestream(),&mut 0).unwrap();
  assert_eq!(pk.packet_type, PacketType::TofEvent);
  assert_eq!(pk, test);
  warn!("PartialEq missing for TofEvent!");
  //let data_test = TofEvent::from_bytestream(&pk.payload, &mut 0).unwrap();
  //assert_eq!(data, data_test);
}

#[cfg(feature="random")]
#[test] 
fn tofpacket_from_mtevent() {
  let data = MasterTriggerEvent::new(0,0);
  let pk   = TofPacket::from(&data);
  let test = TofPacket::from_bytestream(&pk.to_bytestream(),&mut 0).unwrap();
  assert_eq!(pk.packet_type, PacketType::MasterTrigger);
  assert_eq!(pk, test);
  let data_test = MasterTriggerEvent::from_bytestream(&pk.payload, &mut 0).unwrap();
  assert_eq!(data, data_test);
}

#[cfg(feature="random")]
#[test] 
fn tofpacket_from_rbmonidata() {
  let data = RBMoniData::new();
  let pk   = TofPacket::from(&data);
  let test = TofPacket::from_bytestream(&pk.to_bytestream(),&mut 0).unwrap();
  assert_eq!(pk.packet_type, PacketType::RBMoni);
  assert_eq!(pk, test);
  let data_test = RBMoniData::from_bytestream(&pk.payload, &mut 0).unwrap();
  assert_eq!(data, data_test);
}

#[cfg(feature="random")]
#[test] 
fn tofpacket_from_ltbmonidata() {
  let data = LTBMoniData::new();
  let pk   = TofPacket::from(&data);
  let test = TofPacket::from_bytestream(&pk.to_bytestream(),&mut 0).unwrap();
  assert_eq!(pk.packet_type, PacketType::LTBMoniData);
  assert_eq!(pk, test);
  let data_test = LTBMoniData::from_bytestream(&pk.payload, &mut 0).unwrap();
  assert_eq!(data, data_test);
}

#[cfg(feature="random")]
#[test] 
fn tofpacket_from_pbmonidata() {
  let data = PBMoniData::new();
  let pk   = TofPacket::from(&data);
  let test = TofPacket::from_bytestream(&pk.to_bytestream(),&mut 0).unwrap();
  assert_eq!(pk.packet_type, PacketType::PBMoniData);
  assert_eq!(pk, test);
  let data_test = PBMoniData::from_bytestream(&pk.payload, &mut 0).unwrap();
  assert_eq!(data, data_test);
}

#[cfg(feature="random")]
#[test] 
fn tofpacket_from_pamonidata() {
  let data = PAMoniData::new();
  let pk   = TofPacket::from(&data);
  let test = TofPacket::from_bytestream(&pk.to_bytestream(),&mut 0).unwrap();
  assert_eq!(pk.packet_type, PacketType::PAMoniData);
  assert_eq!(pk, test);
  let data_test = PAMoniData::from_bytestream(&pk.payload, &mut 0).unwrap();
  assert_eq!(data, data_test);
}

#[cfg(feature="random")]
#[test] 
fn tofpacket_from_mtbmonidata() {
  let data = MtbMoniData::new();
  let pk   = TofPacket::from(&data);
  let test = TofPacket::from_bytestream(&pk.to_bytestream(),&mut 0).unwrap();
  assert_eq!(pk.packet_type, PacketType::MonitorMtb);
  assert_eq!(pk, test);
  let data_test = MtbMoniData::from_bytestream(&pk.payload, &mut 0).unwrap();
  assert_eq!(data, data_test);
}


