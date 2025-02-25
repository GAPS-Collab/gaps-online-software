//! Basic telemetry packets
//!

pub mod magnetometer;

pub use magnetometer::MagnetoMeter;

use std::fmt;
use log::{
  //info,
  debug,
  error
};

use tof_dataclasses::errors::SerializationError;
use tof_dataclasses::serialization::{
  parse_u8,
  parse_u16,
  parse_u32,
  parse_u64,
  Serialization,
  Packable
};

use tof_dataclasses::events::TofEventSummary;
use tof_dataclasses::packets::{
  TofPacket,
  PacketType
};

#[cfg(feature = "pybindings")]
use pyo3::pyclass;

/// Recreate 48bit timestamp from u32 and u16
pub fn make_systime(lower : u32, upper : u16) -> u64 {
  (upper as u64) << 32 | lower as u64
}


#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "pybindings", pyclass)]
#[repr(u8)]
pub enum TelemetryPacketType {
  Unknown            = 0,
  CardHKP            = 30,
  CoolingHK          = 40,
  PDUHK              = 50,
  Tracker            = 80,
  TrackerDAQCntr     = 81,
  GPS                = 82,
  TrkTempLeak        = 83,
  BoringEvent        = 90,
  RBWaveform         = 91,
  AnyTofHK           = 92,
  GcuEvtBldSettings  = 93,
  LabJackHK          = 100,
  MagHK              = 108,
  GcuMon             = 110,
  InterestingEvent   = 190,
  NoGapsTriggerEvent = 191,
  NoTofDataEvent     = 192,
  Ack                = 200,     
  AnyTrackerHK       = 255,
  // unknown/unused stuff
  TmP33              = 33,
  TmP34              = 34,
  TmP37              = 37,
  TmP38              = 38,
  TmP55              = 55,
  TmP64              = 64,
  //TmP92            = 92,
  TmP96              = 96,
  TmP214             = 214,
}

impl From<u8> for TelemetryPacketType {
  fn from(value: u8) -> Self {
    match value {
      0     => TelemetryPacketType::Unknown,
      30    => TelemetryPacketType::CardHKP,
      40    => TelemetryPacketType::CoolingHK,
      50    => TelemetryPacketType::PDUHK,
      80    => TelemetryPacketType::Tracker,
      81    => TelemetryPacketType::TrackerDAQCntr,
      82    => TelemetryPacketType::GPS,
      83    => TelemetryPacketType::TrkTempLeak,
      90    => TelemetryPacketType::BoringEvent,
      91    => TelemetryPacketType::RBWaveform,
      92    => TelemetryPacketType::AnyTofHK,
      93    => TelemetryPacketType::GcuEvtBldSettings,
      100   => TelemetryPacketType::LabJackHK,
      108   => TelemetryPacketType::MagHK,
      110   => TelemetryPacketType::GcuMon,
      190   => TelemetryPacketType::InterestingEvent,
      191   => TelemetryPacketType::NoGapsTriggerEvent,
      192   => TelemetryPacketType::NoTofDataEvent,
      200   => TelemetryPacketType::Ack,
      255   => TelemetryPacketType::AnyTrackerHK,
      _     => TelemetryPacketType::Unknown,
    }
  }
}

impl fmt::Display for TelemetryPacketType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error - Don't understand packet type!"));
    write!(f, "<TelemetryPacketType: {}>", r)
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TelemetryPacket {
  pub header  : TelemetryHeader,
  pub payload : Vec<u8>
}

impl TelemetryPacket {
  pub fn new() -> Self {
    Self {
      header  : TelemetryHeader::new(),
      payload : Vec::<u8>::new()
    }
  }

  pub fn from_bytestream(stream : &Vec<u8>, pos : &mut usize) -> Result<Self, SerializationError> {
    let mut tpacket = TelemetryPacket::new();
    let header  = TelemetryHeader::from_bytestream(stream, pos)?;
    tpacket.header = header;
    //println!("Found header {}", tpacket.header);
    // it seems the payload size is header.size
    // fix - the payload is either sizeof(header) + payload.len 
    tpacket.payload = stream[*pos..*pos + header.length as usize - TelemetryHeader::SIZE].to_vec();
    Ok(tpacket)
  }

  // FIXME - this needs to be a trait
  pub fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::new();
    let mut s_head = self.header.to_bytestream();
    stream.append(&mut s_head);
    stream.extend_from_slice(self.payload.as_slice());
    //stream.append(&mut self.payload);
    stream
  }
}

impl fmt::Display for TelemetryPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TelemetryPacket:");
    repr += &(format!("\n  Header      : {}",self.header));
    repr += &(format!("\n  Payload len : {}>",self.payload.len()));
    write!(f, "{}", repr)
  }
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TelemetryHeader {
  pub sync      : u16,
  pub ptype     : u8,
  pub timestamp : u32,
  pub counter   : u16,
  pub length    : u16,
  pub checksum  : u16
}

//    fn from_bytestream(
//    bytestream: &Vec<u8>,
//    pos: &mut usize
//) -> Result<Self, SerializationError>

impl TelemetryHeader {

  pub fn new() -> Self {
    Self {
      sync      : 0,
      ptype     : 0,
      timestamp : 0,
      counter   : 0,
      length    : 0,
      checksum  : 0,
    }
  }

  /// A re-implementation of make_packet_stub
  pub fn forge(packet_type : u8) -> Self {
    let mut header = Self::new();
    header.sync  = 0x90EB;
    header.ptype   = packet_type;
    header
  }

//{
//   std::vector<uint8_t> bytes(13,0);
//   *reinterpret_cast<uint16_t*>(&bytes[0]) = 0x90EB;
//   bytes[2] = type;
//   if(timestamp == 0)
//      timestamp = bfsw::timestamp_64ms();
//   *reinterpret_cast<uint32_t*>(&bytes[3]) = timestamp;
//   *reinterpret_cast<uint16_t*>(&bytes[7]) = counter;
//   *reinterpret_cast<uint16_t*>(&bytes[9]) = length;
//   *reinterpret_cast<uint16_t*>(&bytes[11]) = 0;
//   return bytes;
  /// This is a reimplementation of bfsw's timestamp_to_double
  pub fn get_gcutime(&self) -> f64 {
    (self.timestamp as f64) * 0.064 + 1631030675.0
  }
}

impl Serialization for TelemetryHeader {
  
  const HEAD : u16 = 0x90eb;
  const TAIL : u16 = 0x0000; // there is no tail for telemetry packets
  const SIZE : usize = 13; 

  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize)
    -> Result<Self, SerializationError> {
    if stream.len() < *pos + Self::SIZE {
      return Err(SerializationError::StreamTooShort);
    }
    if parse_u16(stream, pos) != 0x90eb {
      error!("The given position {} does not point to a valid header signature of {}", pos, 0x90eb);
      return Err(SerializationError::HeadInvalid {});
    }
    let mut thead = TelemetryHeader::new();
    thead.sync      = 0x90eb;
    thead.ptype     = parse_u8 (stream, pos);
    thead.timestamp = parse_u32(stream, pos);
    thead.counter   = parse_u16(stream, pos);
    thead.length    = parse_u16(stream, pos);
    thead.checksum  = parse_u16(stream, pos);
    Ok(thead)
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::new();
    //let head : u16 = 0x90eb;
    // "SYNC" is the header signature
    stream.extend_from_slice(&self.sync.to_le_bytes());
    stream.extend_from_slice(&self.ptype.to_le_bytes());
    stream.extend_from_slice(&self.timestamp.to_le_bytes());
    stream.extend_from_slice(&self.counter.to_le_bytes());
    stream.extend_from_slice(&self.length.to_le_bytes());
    stream.extend_from_slice(&self.checksum.to_le_bytes());
    stream
  }

}

impl fmt::Display for TelemetryHeader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TelemetryHeader:");
    repr += &(format!("\n  Header      : {}",self.sync));
    repr += &(format!("\n  Packet Type : {}",self.ptype));
    repr += &(format!("\n  Timestamp   : {}",self.timestamp));
    repr += &(format!("\n  Counter     : {}",self.counter));
    repr += &(format!("\n  Length      : {}",self.length));
    repr += &(format!("\n  Checksum    : {}>",self.checksum));
    write!(f, "{}", repr)
  }
}


/// The acknowledgement packet used within the 
/// bfsw code
pub struct AckBfsw {
  pub header    : TelemetryHeader,
  pub ack_type  : u8,
  pub ret_code1 : u8,
  pub ret_code2 : u8,
  pub body      : Vec<u8>
}

impl AckBfsw {
  pub fn new() -> Self {
    //let mut header = TelemetryHeader::new(),

    Self {
      header    : TelemetryHeader::new(),
      ack_type  : 1,
      ret_code1 : 0,
      ret_code2 : 0,
      body      : Vec::<u8>::new()
    }
  }
  
  //pub fn to_bytestream(&self) -> Vec<u8> {
  //  let mut stream = Vec::<u8>::new();
  //  let mut s_head = self.header.to_bytestream();
  //  stream.append(&mut s_head);
  //  stream.extend_from_slice(self.payload.as_slice());
  //  //stream.append(&mut self.payload);
  //  stream
  //}
}

impl Serialization for AckBfsw {
  
  const HEAD : u16 = 0x90eb;
  const TAIL : u16 = 0x0000; // there is no tail for telemetry packets
  const SIZE : usize = 13; 

  fn from_bytestream(stream : &Vec<u8>,
                     pos    : &mut usize)
    -> Result<Self, SerializationError> {
    if stream.len() < *pos + 3 {
      return Err(SerializationError::StreamTooShort);
    }
    let mut ack   = AckBfsw::new();
    ack.ack_type  = parse_u8(stream, pos);
    ack.ret_code1 = parse_u8(stream, pos);
    ack.ret_code2 = parse_u8(stream, pos);
    Ok(ack)
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::new();
    stream.push(self.ack_type);
    stream.push(self.ret_code1);
    stream.push(self.ret_code2);
    stream
  }
}

impl Packable for AckBfsw {
  const PACKET_TYPE : PacketType = PacketType::BfswAckPacket;
}

impl fmt::Display for AckBfsw {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<AckBfsw:");
    //repr += &(format!("\n  Header      : {}" ,self.sync));
    //repr += &(format!("\n  Packet Type : {}" ,self.ptype));
    //repr += &(format!("\n  Timestamp   : {}" ,self.timestamp));
    repr += &(format!("\n  Ack Type    : {}" ,self.ack_type));
    repr += &(format!("\n  Ret Code1   : {}" ,self.ret_code1));
    repr += &(format!("\n  Ret Code2   : {}>",self.ret_code2));
    write!(f, "{}", repr)
  }
}

pub struct MergedEvent {
  
  pub header              : TelemetryHeader,
  pub creation_time       : u64,
  pub event_id            : u32,
  pub tracker_events      : Vec<TrackerEvent>,
  /// in case this is version 2, we don't have
  /// tracker_events, but new-style tracker hits
  /// (TrackerHitV2)
  pub tracker_hitsv2      : Vec<TrackerHitV2>,
  pub tracker_oscillators : Vec<u64>,
  pub tof_data            : Vec<u8>,
  pub raw_data            : Vec<u8>,
  pub flags0              : u8,
  pub flags1              : u8,
  pub version             : u8
}

impl MergedEvent {

  pub fn new() -> Self {
    let mut tracker_oscillators = Vec::<u64>::new();
    for _ in 0..10 {
      tracker_oscillators.push(0);
    }
    Self {
      header              : TelemetryHeader::new(),
      creation_time       : 0,
      event_id            : 0,
      tracker_events      : Vec::<TrackerEvent>::new(),
      tracker_hitsv2      : Vec::<TrackerHitV2>::new(),
      tracker_oscillators : tracker_oscillators,
      tof_data            : Vec::<u8>::new(),
      raw_data            : Vec::<u8>::new(),
      flags0              : 0,
      flags1              : 1,
      version             : 0, 
    }
  }


  pub fn get_tofeventsummary(&self) -> Result<TofEventSummary, SerializationError> {
    match TofPacket::from_bytestream(&self.tof_data, &mut 0) {
      Err(err) => {
        error!("Unable to parse TofPacket! {err}");
        return Err(err);
      }
      Ok(pack) => {
        match pack.unpack::<TofEventSummary>() {
          Err(err) => {
            error!("Unable to parse TofEventSummary! {err}");
            return Err(err);
          }
          Ok(ts)    => {
            return Ok(ts);
          }
        }
      }
    }
  }


  pub fn from_bytestream(stream : &Vec<u8>,
                         pos    : &mut usize)
    -> Result<Self, SerializationError> {
    let mut me        = MergedEvent::new();
    let version      = parse_u8(stream, pos);
    me.version       = version;
    //println!("_version {}", _version);
    me.flags0         = parse_u8(stream, pos);
    // skip a bunch of Alex newly implemented things
    // FIXME
    if version == 0 {
      me.flags1      = parse_u8(stream, pos);
    } else {
      *pos += 8;
    }

    me.event_id       = parse_u32(stream, pos);
    //println!("EVENT ID {}", me.event_id);
    let _tof_delim    = parse_u8(stream, pos);
    //println!("TOF delim : {}", _tof_delim);
    if stream.len() <= *pos + 2 {
      error!("Not able to parse merged event!");
      return Err(SerializationError::StreamTooShort);
    }
    let num_tof_bytes = parse_u16(stream, pos);
    //println!("Num TOF bytes : {}", num_tof_bytes);
    if stream.len() < *pos+num_tof_bytes as usize {
      error!("Not enough bytes for TOF packet! Expected {}, seen {}", *pos+num_tof_bytes as usize, stream.len());
      return Err(SerializationError::StreamTooShort); 
    }
    for _ in *pos..*pos+num_tof_bytes as usize {
      me.tof_data.push(parse_u8(stream, pos));
    }
    let trk_delim    = parse_u8(stream, pos);

    //println!("TRK delim {}", trk_delim);
    if trk_delim != 0xbb {
      return Err(SerializationError::HeadInvalid);
    }
    if version == 1 {
      let num_trk_hits = parse_u16(stream, pos);
      if (*pos + (num_trk_hits as usize)*4 ) > stream.len() {
        return Err(SerializationError::StreamTooShort);
      }
      for _ in 0..num_trk_hits { 
        let mut hit = TrackerHitV2::new();
        let strip_id = parse_u16(stream, pos);
        let adc      = parse_u16(stream, pos);
        hit.channel  = strip_id & 0b11111;
        hit.module   = (strip_id >> 5) & 0b111;
        hit.row      = (strip_id >> 8) & 0b111;
        hit.layer    = (strip_id >> 11) & 0b1111;
        hit.adc      = adc;
        me.tracker_hitsv2.push(hit);
      }
      // oscillators
      let oscillators_delimiter = parse_u8(stream, pos);
      if oscillators_delimiter != 0xcc {
        return Err(SerializationError::HeadInvalid);
      }
      let osc_flags = parse_u8(stream, pos);
      let mut oscillator_idx = Vec::<u8>::new();
      for j in 0..8 {
        if (osc_flags >> j & 0b1) > 0 {
          oscillator_idx.push(j)
        }
      }
      if (*pos + oscillator_idx.len()*6) > stream.len() {
        return Err(SerializationError::StreamTooShort);
      }
      for idx in oscillator_idx.iter() {
        let lower = parse_u32(stream, pos);
        let upper = parse_u16(stream, pos);
        let osc : u64 = (upper as u64) << 32 | (lower as u64);
        me.tracker_oscillators[*idx as usize] = osc;
      }
    } else if version == 0 {
      let num_trk_bytes = parse_u16(stream, pos);
      if (num_trk_bytes as usize + *pos - 2) > stream.len() {
        return Err(SerializationError::StreamTooShort);
      }
      //println!("Num TRK bytes : {}", num_trk_bytes);
      // for now, don't unpack tracker data
      //*pos += num_trk_bytes as usize;
      let max_pos = *pos + num_trk_bytes as usize;
      loop {
         //if *pos > max_pos {
         //  //return Err(SerializationError::StreamTooLong);
         //} 
         if *pos >= max_pos {
           break;
         }
         if *pos >= stream.len() {
           break;
         }
         let mut te = TrackerEvent::from_bytestream(stream, pos)?;
         //println!("{}",te);
         te.event_id = me.event_id;
         me.tracker_events.push(te);
         //if(rc < 0)
         //{
         //   spdlog::info("DEBUG event.unpack rc = {}", rc);
         //   return -9;
         //}
         //else
         //{
         //    tracker_events.push_back(std::move(event));
         //    i += rc;
         //}
      }
    } else {
      error!("Unrecognized version {version}!");
    }
    Ok(me)
  }
}

impl fmt::Display for MergedEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr     = String::from("<MergedEvent:");
    let mut tof_str  = String::from("- UNABLE TO PARSE TOF DATA!");
    let mut tof_evid = 0u32;
    match TofPacket::from_bytestream(&self.tof_data, &mut 0) {
      Err(err) => error!("Unable to parse TofPacket! {err}"),
      Ok(pack) => {
        match pack.unpack::<TofEventSummary>() {
          Err(err) => error!("Unable to parse TofEventSummary! {err}"),
          Ok(ts)    => {
            tof_str  = format!("\n  {}", ts);
            tof_evid = ts.event_id;
          }
        }
      }
    }
    let mut good_hits = 0;
    let mut evids = Vec::<u32>::new();
    if self.version == 0 {
      for ev in &self.tracker_events {
        evids.push(ev.event_id);
        for h in &ev.hits { 
          if h.adc != 0 {
            good_hits += 1;
          }
        }
      evids.sort();
      evids.dedup();
      }
    } else if self.version == 1 {
      for _ in &self.tracker_hitsv2 {
        good_hits += 1;
      }
    }
    repr += &(format!("  {}", self.header));
    repr += "\n  ** ** ** MERGED  ** ** **";
    repr += &(format!("\n  version         {}", self.version));
    repr += &(format!("\n  event ID        {}", self.event_id));  
    if self.version == 0 {
      repr += &(format!("\n  -- TOF          {}", tof_evid));
      repr += &(format!("\n  -- TRK          {:?}", evids));
    }
    repr += "\n  ** ** ** TRACKER ** ** **";
    if self.version == 0 {
      repr += &(format!("\n  N Trk events    {}", self.tracker_events.len()));
    } else if self.version == 1 {
      repr += &(format!("\n  Trk oscillators {:?}", self.tracker_oscillators)); 
    }
    repr += &(format!("\n  N Good Trk Hits {}", good_hits));
    repr += &tof_str;
    write!(f,"{}", repr)
  }
}

#[derive(Debug, Clone)]
pub struct TrackerEvent {

  pub layer      : u8,
  pub flags1     : u8,
  pub event_id   : u32,
  pub event_time : u64,
  pub hits       : Vec<TrackerHit>,
}

impl TrackerEvent {

  pub fn new() -> Self {
    Self { 
      layer      : 0,
      flags1     : 0,
      event_id   : 0,
      event_time : 0,
      hits       : Vec::<TrackerHit>::new(),
    }
  }

  /// Loop over the filtered hits, returning only those satisfying a condition
  ///
  /// # Arguments:
  /// 
  /// * filter : filter function - take input hit and decide if it should be 
  ///            returned
  pub fn filter_hits(&self, filter : fn(&TrackerHit) -> bool) -> Vec<TrackerHit> {
    let mut filtered_hits = Vec::<TrackerHit>::new();
    for h in &self.hits {
      if filter(h) {
        filtered_hits.push(*h);
      }
    }
    filtered_hits
  }

  pub fn from_bytestream(stream : &Vec<u8>,
                         pos    : &mut usize)
    -> Result<Self, SerializationError> {
  if *pos + 8 > stream.len() {
    return Err(SerializationError::StreamTooShort);
  }
  let mut te = TrackerEvent::new();
  // first timestamp
  let ts32   = parse_u32(stream, pos);
  let ts16   = parse_u16(stream, pos);
  let ts64   = ((ts16 as u64) << 16) | ts32 as u64;
  te.event_time = ts64;
  
  te.layer   = parse_u8(stream, pos);
  let nhits  = parse_u8(stream, pos); 
  //println!("See layer {} and nhits {}, expected size {}", te.layer, nhits, nhits as usize * TrackerHit::SIZE); 
  //panic!("uff der titantic! (na wo sonst wohl?)");
  for _ in 0..nhits {
    te.hits.push(TrackerHit::from_bytestream(stream, pos)?);
  }
  Ok(te)
  }
}

impl fmt::Display for TrackerEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerEvent:");
    repr += &(format!("\n  EventTime     : {}" ,self.event_time));
    repr += &(format!("\n  EventID       : {}" ,self.event_id));
    repr += &(format!("\n  Layer         : {}" ,self.layer));
    repr += &(format!("\n  Flags1        : {}" ,self.flags1));
    repr += &(format!("\n**** HITS {} ****", self.hits.len()));
    for h in &self.hits {
      repr += &(format!("\n {}", h));
    }
    write!(f, "{}", repr)
  }
}

#[derive(Debug, Copy, Clone)]
pub struct TrackerHitV2 {
  pub layer           : u16,
  pub row             : u16,
  pub module          : u16,
  pub channel         : u16,
  pub adc             : u16,
  pub oscillator      : u64
}

impl TrackerHitV2 {
  //const SIZE : usize = 18;
  
  pub fn new() -> Self {
    Self {
      layer           : 0,
      row             : 0,
      module          : 0,
      channel         : 0,
      adc             : 0,
      oscillator      : 0,
    }
  }
}

impl fmt::Display for TrackerHitV2 {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerHitV2:");
    repr += &(format!("\n  Layer         : {}" ,self.layer));
    repr += &(format!("\n  Row           : {}" ,self.row));
    repr += &(format!("\n  Module        : {}" ,self.module));
    repr += &(format!("\n  Channel       : {}" ,self.channel));
    repr += &(format!("\n  ADC           : {}" ,self.adc));
    repr += &(format!("\n  Oscillator    : {}>",self.oscillator));
    write!(f, "{}", repr)
  }
}

#[derive(Debug, Copy, Clone)]
pub struct TrackerHit {
  pub row             : u8,
  pub module          : u8,
  pub channel         : u8,
  pub adc             : u16,
  pub asic_event_code : u8,
}

impl TrackerHit {
  const SIZE : usize = 6;

  pub fn new() -> Self {
    Self {
      row             : 0,
      module          : 0,
      channel         : 0,
      adc             : 0,
      asic_event_code : 0,
    }
  }

  pub fn from_bytestream(stream : &Vec<u8>,
                         pos    : &mut usize)
    -> Result<Self, SerializationError> {
    if *pos + Self::SIZE > stream.len() {
      return Err(SerializationError::StreamTooShort);
    }

    let mut th         = TrackerHit::new();
    th.row             = parse_u8(stream, pos);
    th.module          = parse_u8(stream, pos);
    th.channel         = parse_u8(stream, pos);
    th.adc             = parse_u16(stream, pos);
    th.asic_event_code = parse_u8(stream, pos);
    Ok(th)
  } 
}

impl fmt::Display for TrackerHit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerHit:");
    repr += &(format!("\n  Row           : {}" ,self.row));
    repr += &(format!("\n  Module        : {}" ,self.module));
    repr += &(format!("\n  Channel       : {}" ,self.channel));
    repr += &(format!("\n  ADC           : {}" ,self.adc));
    repr += &(format!("\n  ASIC Ev. Code : {}>",self.asic_event_code));
    write!(f, "{}", repr)
  }
}

#[derive(Clone)]
pub struct TrackerHeader {
  pub sync        : u16,
  pub crc         : u16,
  pub sys_id      : u8,
  pub packet_id   : u8,
  pub length      : u16,
  pub daq_count   : u16,
  pub sys_time    : u64,
  pub version     : u8,
} 

impl TrackerHeader {
  
  pub const SIZE : usize = 17;

  pub fn new() -> Self {
    Self {
      sync        : 0,
      crc         : 0,
      sys_id      : 0,
      packet_id   : 0,
      length      : 0,
      daq_count   : 0,
      sys_time    : 0,
      version     : 0,
    }
  }

  pub fn from_bytestream(stream: &Vec<u8>,
                         pos: &mut usize)
    -> Result<Self, SerializationError> {
    if stream.len() <= Self::SIZE {
      error!("Unable to decode TrackerHeader!"); 
      return Err(SerializationError::StreamTooShort);
    }
    let mut h     = TrackerHeader::new();
    h.sync        = parse_u16(stream, pos);
    h.crc         = parse_u16(stream, pos); 
    h.sys_id      = parse_u8 (stream, pos);
    h.packet_id   = parse_u8 (stream, pos);
    h.length      = parse_u16(stream, pos);
    h.daq_count   = parse_u16(stream, pos);
    let lower     = parse_u32(stream, pos);
    let upper     = parse_u16(stream, pos);
    h.sys_time    = make_systime(lower, upper);
    h.version     = parse_u8 (stream, pos);
    Ok(h)
  }
}

impl fmt::Display for TrackerHeader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerHeader");
    repr    += &(format!("\n  Sync     : {}", self.sync));
    repr    += &(format!("\n  Crc      : {}", self.crc));
    repr    += &(format!("\n  PacketID : {}", self.packet_id));
    repr    += &(format!("\n  Length   : {}", self.length));
    repr    += &(format!("\n  DAQ Cnt  : {}", self.daq_count));
    repr    += &(format!("\n  Sys Time : {}", self.sys_time));
    repr    += &(format!("\n  Version  : {}>", self.version));
    write!(f, "{}", repr)
  }
}


pub struct GPSPacket {
  pub telemetry_header : TelemetryHeader,
  pub tracker_header   : TrackerHeader,
  pub utc_time         : u32,
  pub gps_info         : u8
}

impl GPSPacket {
  pub fn new() -> Self {
    Self {
      telemetry_header : TelemetryHeader::new(),
      tracker_header   : TrackerHeader::new(),
      utc_time         : 0,
      gps_info         : 0,
    }
  }
  
  pub fn from_bytestream(stream: &Vec<u8>,
                         pos: &mut usize)
    -> Result<Self, SerializationError> {
    let mut gps_p       = GPSPacket::new();
    gps_p.tracker_header   = TrackerHeader::from_bytestream(stream, pos)?;
    if stream.len() == *pos as usize {
      error!("Packet contains only header!");
      return Ok(gps_p);
    }
    gps_p.utc_time = parse_u32(stream, pos);
    gps_p.gps_info = parse_u8(stream, pos);
    Ok(gps_p)
  }
}

impl fmt::Display for GPSPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<GPSPacket");
    repr    += &(format!("\n {}", self.telemetry_header));
    repr    += &(format!("\n {}", self.tracker_header));
    repr    += "\n*** GPS TIME ***";
    repr    += &(format!("\n UTC TIME (32bit) {}", self.utc_time));
    repr    += &(format!("\n GSP INFO (8bit)  {}", self.gps_info));
    repr    += ">";
    write!(f, "{}", repr)
  }
}

/// Re-implementation of Alex' tracker packet
pub struct TrackerPacket {
  pub telemetry_header : TelemetryHeader,
  pub tracker_header   : TrackerHeader,
  pub events           : Vec<TrackerEvent>,
}

impl fmt::Display for TrackerPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerPacket");
    repr    += &(format!("\n {}", self.telemetry_header));
    repr    += &(format!("\n {}", self.tracker_header));
    repr    += "\n*** Events ***";
    for ev in &self.events {
      repr  += &(format!("\n {}", ev));
    }
    repr    += ">";
    write!(f, "{}", repr)
  }
}

impl TrackerPacket {
  pub fn new() -> Self {
    Self {
      telemetry_header : TelemetryHeader::new(),
      tracker_header   : TrackerHeader::new(),
      events           : Vec::<TrackerEvent>::new(),
    }
  }
  
  pub fn from_bytestream(stream: &Vec<u8>,
                         pos: &mut usize)
    -> Result<Self, SerializationError> {
    let mut tp          = TrackerPacket::new();
    tp.tracker_header   = TrackerHeader::from_bytestream(stream, pos)?;
    if stream.len() == *pos as usize {
      error!("Packet contains only header!");
      return Ok(tp);
    }
    let _settings       = parse_u8(stream, pos);

    loop {    
      let mut event    = TrackerEvent::new();
      event.layer      = tp.tracker_header.sys_id;
      if *pos + 12 > stream.len() {
        error!("Unable to decode header part for tracker event!");
        return Err(SerializationError::StreamTooShort);
      }
      let num_hits     = parse_u8(stream, pos);
      event.flags1     = parse_u8(stream, pos);
      event.event_id   = parse_u32(stream, pos);
      let ts32         = parse_u32(stream, pos);
      let ts16         = parse_u16(stream, pos);
      event.event_time = (ts16 as u64) << 32 | ts32 as u64;
      if num_hits > 192 {
        //isn't a real event, looking at filler bytes.
        //once event packets stop having filler,
        //logic here will need to change
        break; 
      }
      if *pos + 3*(num_hits as usize) > stream.len() {
        error!("We expect {} hits, but the stream is not long enough!", num_hits);
        return Err(SerializationError::StreamTooShort);
      }
      for _ in 0..num_hits {
        let h0 = parse_u8(stream, pos);
        let h1 = parse_u8(stream, pos);
        let h2 = parse_u8(stream, pos);
        let asic_event_code : u8 = h2 >> 6;
        let channel : u8  = h0 & 0x1f;
        let module  : u8  = h0 >> 5;
        let row     : u8  = h1 & 0x7;
        let adc     : u16 = (((h2 as u16) & 0x3f) << 5) | (h1 as u16) >> 3; 
        event.hits.push(TrackerHit { row, module, channel, adc, asic_event_code});
      }
      tp.events.push(event);
      if tp.events.len() > 170 {
        error!(">170 events in this packet!");
        return Err(SerializationError::StreamTooLong);
      }
    }
    Ok(tp)
  }
}
      
pub struct TrackerTempLeakPacket {
  pub telemetry_header : TelemetryHeader,
  pub tracker_header   : TrackerHeader,
  pub row_offset       : u8,
  pub templeak         : [[u32;6];6],
  pub seu              : [[u32;6];6]
}

impl fmt::Display for TrackerTempLeakPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerTempLeakPacket");
    repr    += &(format!("\n {}", self.telemetry_header));
    repr    += &(format!("\n {}", self.tracker_header));
    repr    += &(format!("\n ROW OFFSET {}", self.row_offset));
    repr    += "\n*** TEMPLEAK ***";
    for k in 0..6 {
      repr  += &(format!("\n {:?}", self.templeak[k]));
    }
    repr    += "\n*** SEU ***";
    for k in 0..6 {
      repr  += &(format!("\n {:?}", self.seu[k]));
    }
    repr    += ">";
    write!(f, "{}", repr)
  }
}

impl TrackerTempLeakPacket {
  pub fn new() -> Self {
    Self {
      telemetry_header : TelemetryHeader::new(),
      tracker_header   : TrackerHeader::new(),
      row_offset       : 0,
      templeak         : [[0;6];6],
      seu              : [[0;6];6]
    }
  }
  
  pub fn from_bytestream(stream: &Vec<u8>,
                         pos: &mut usize)
    -> Result<Self, SerializationError> {
    let mut tp          = TrackerTempLeakPacket::new();
    tp.tracker_header   = TrackerHeader::from_bytestream(stream, pos)?;
    if stream.len() == *pos as usize {
      error!("Packet contains only header!");
      return Ok(tp);
    }
    if stream.len() - *pos < (36*3 + 1) {
      return Err(SerializationError::StreamTooShort);
    }
    let row_info = parse_u8(stream, pos);
    tp.row_offset = row_info & 0x7;
    for row in 0..6 {
      for module in 0..6 {
        let b0 = parse_u8(stream, pos) as u32;
        let b1 = parse_u8(stream, pos) as u32;
        let b2 = parse_u8(stream, pos) as u32;
        let seu_ : u32 = b2 >> 1;
        let mut templeak_ : u32 = (b2 << 10) | (b1 << 2)  | (b0 >> 6);
        templeak_ &= 0x7ff;
        tp.templeak[row][module] = templeak_;
        tp.seu[row][module] = seu_;
      }
    }
    Ok(tp)
  }
}

pub struct TrackerDAQTempPacket {
  pub telemetry_header : TelemetryHeader,
  pub tracker_header   : TrackerHeader,
  pub rom_id           : [u64;256],
  pub temp             : [u16;256]
}

impl fmt::Display for TrackerDAQTempPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerDAQTempPacket");
    repr    += &(format!("\n {}", self.telemetry_header));
    repr    += &(format!("\n {}", self.tracker_header));
    repr    += "\n*** ROM ID ***";
    repr  += &(format!("\n {:?}", self.rom_id));
    repr    += "\n*** TEMP ***";
    repr  += &(format!("\n {:?}>", self.temp));
    write!(f, "{}", repr)
  }
}

impl TrackerDAQTempPacket {
  pub fn new() -> Self {
    Self {
      telemetry_header : TelemetryHeader::new(),
      tracker_header   : TrackerHeader::new(),
      rom_id           : [0;256],
      temp             : [0;256]
    }
  }
  
  pub fn from_bytestream(stream: &Vec<u8>,
                         pos: &mut usize)
    -> Result<Self, SerializationError> {
    let mut tp          = TrackerDAQTempPacket::new();
    tp.tracker_header   = TrackerHeader::from_bytestream(stream, pos)?;
    if tp.tracker_header.packet_id != 0x09 {
      error!("This is not a TrackerDAQTempPacket, but has packet_id {} instead!", tp.tracker_header.packet_id);
      return Err(SerializationError::IncorrectPacketType);
    }
    debug!("tracker header {}", tp.tracker_header);
    if stream.len() == *pos as usize {
      error!("Packet contains only header!");
      return Ok(tp);
    }
    //if stream.len() - *pos < (36*3 + 1) {
    //  return Err(SerializationError::StreamTooShort);
    //}
    // this is hack, since the TreckerHeader in this packet does not have a 
    // version (-> Alex) 
    *pos -= 1;
    let dummy64 = 0u64;
    let dummy16 = 0u16;
    error!("{}", tp.tracker_header);
    error!("Expected of the packet {}", (tp.tracker_header.length as usize)/2);
    for k in 0..256usize {
      if k < (tp.tracker_header.length as usize)/2 {
        tp.rom_id[k] = parse_u64(stream, pos);
        tp.temp[k]   = parse_u16(stream, pos);
      } else {
        tp.rom_id[k] = dummy64;
        tp.temp[k]   = dummy16;
      }
    }
    Ok(tp)
  }
}

pub struct TrackerDAQHSKPacket {
  pub telemetry_header : TelemetryHeader,
  pub tracker_header   : TrackerHeader,
  pub temp             : [u16;12],
}

impl fmt::Display for TrackerDAQHSKPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerDAQHSKPacket");
    repr    += &(format!("\n {}", self.telemetry_header));
    repr    += &(format!("\n {}", self.tracker_header));
    repr    += "\n*** TEMP ***";
    repr    += &(format!("\n {:?}>", self.temp));
    write!(f, "{}", repr)
  }
}

impl TrackerDAQHSKPacket {
  pub fn new() -> Self {
    Self {
      telemetry_header : TelemetryHeader::new(),
      tracker_header   : TrackerHeader::new(),
      temp             : [0;12]
    }
  }
  
  pub fn from_bytestream(stream: &Vec<u8>,
                         pos: &mut usize)
    -> Result<Self, SerializationError> {
    let mut tp          = TrackerDAQHSKPacket::new();
    tp.tracker_header   = TrackerHeader::from_bytestream(stream, pos)?;
    if tp.tracker_header.packet_id != 0xff {
      error!("This is not a TrackerDAQHSKPacket, but has packet_id {} instead!", tp.tracker_header.packet_id);
      return Err(SerializationError::IncorrectPacketType);
    }
    if stream.len() == *pos as usize {
      error!("Packet contains only header!");
      return Ok(tp);
    }
    //if stream.len() - *pos < (36*3 + 1) {
    //  return Err(SerializationError::StreamTooShort);
    //}
    // this is hack, since the TreckerHeader in this packet does not have a 
    // version (-> Alex) 
    *pos += 193; // skip a bunch of other stuff right now (Alex)
    for k in 0..12usize {
      tp.temp[k]   = parse_u16(stream, pos);
    }
    Ok(tp)
  }
}

pub struct TrackerEventIDEchoPacket {
  pub telemetry_header : TelemetryHeader,
  pub tracker_header   : TrackerHeader,
  pub temp             : [u16;12],
  pub event_id         : u32,
  pub event_id_errors  : u16,
}

impl fmt::Display for TrackerEventIDEchoPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerEventIDEchoPacket");
    repr    += &(format!("\n {}", self.telemetry_header));
    repr    += &(format!("\n {}", self.tracker_header));
    repr    += "\n*** TEMP ***";
    repr    += &(format!("\n {:?}>", self.temp));
    write!(f, "{}", repr)
  }
}

impl TrackerEventIDEchoPacket {
  pub fn new() -> Self {
    Self {
      telemetry_header : TelemetryHeader::new(),
      tracker_header   : TrackerHeader::new(),
      temp             : [0;12],
      event_id         : 0,
      event_id_errors  : 0,
    }
  }
  
  pub fn from_bytestream(stream: &Vec<u8>,
                         pos: &mut usize)
    -> Result<Self, SerializationError> {
    let mut tp          = TrackerEventIDEchoPacket::new();
    tp.tracker_header   = TrackerHeader::from_bytestream(stream, pos)?;
    if tp.tracker_header.packet_id != 0x03 {
      error!("This is not a TrackerEventIDEchoPacket, but has packet_id {} instead!", tp.tracker_header.packet_id);
      return Err(SerializationError::IncorrectPacketType);
    }
    if stream.len() == *pos as usize {
      error!("Packet contains only header!");
      return Ok(tp);
    }
    //if stream.len() - *pos < (36*3 + 1) {
    //  return Err(SerializationError::StreamTooShort);
    //}
    tp.event_id        = parse_u32(stream, pos);
    tp.event_id_errors = parse_u16(stream, pos);
    Ok(tp)
  }
}


/// This is mine :) Not telemetry
pub struct GapsTracker {

}

pub struct GapsEvent {
  pub tof     : TofEventSummary,
  pub tracker : Vec<TrackerEvent>
}

impl GapsEvent {
  pub fn new() -> Self {
    Self {
      tof     : TofEventSummary::new(),
      tracker : Vec::<TrackerEvent>::new(),
    }
  }
}

impl fmt::Display for GapsEvent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<GapsEvent");
    repr    += "\n  *** TOF ***";
    repr    += &(format!("\n  {}", self.tof));
    repr    += "*** TRACKER ***";
    for ev in &self.tracker {
      repr    += &(format!("\n  -- {}", ev));
    }
    repr    += ">";
    write!(f, "{}", repr)
  }
}


