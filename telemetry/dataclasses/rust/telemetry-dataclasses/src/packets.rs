//! Basic telemetry packets
//!

use std::fmt;
use log::error;

use tof_dataclasses::errors::SerializationError;
use tof_dataclasses::serialization::{
  parse_u8,
  parse_u16,
  parse_u32,
  parse_u64,
  Serialization,
};

use tof_dataclasses::events::TofEventSummary;
use tof_dataclasses::packets::TofPacket;

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum TelemetryPacketType {
  Unknown          = 0,
  RBWaveform  = 91,
  Tracker     = 80,
  MergedEvent = 90,
  AnyTofHK    = 92,
  InterestingEvent = 190,
  Command          = 200,       
  CardHKP          = 30,
  TmP33       = 33,
  TmP34       = 34,
  TmP37       = 37,
  TmP38       = 38,
  TmP40       = 40,
  TmP50       = 50,
  TmP55       = 55,
  TmP64       = 64,
  Tmp81       = 81,
  TmP83       = 83,
  //TmP92       = 92,
  TmP96       = 96,
  TmP214      = 214,
  TmP255      = 255
}

impl From<u8> for TelemetryPacketType {
  fn from(value: u8) -> Self {
    match value {
      0u8   => TelemetryPacketType::Unknown,
      80u8  => TelemetryPacketType::Tracker,
      90u8  => TelemetryPacketType::MergedEvent,
      91u8  => TelemetryPacketType::RBWaveform,
      92u8  => TelemetryPacketType::AnyTofHK,
      190u8 => TelemetryPacketType::InterestingEvent,
      200u8 => TelemetryPacketType::Command,
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

pub struct MergedEvent {
  
  pub header         : TelemetryHeader,
  pub creation_time  : u64,
  pub event_id       : u32,
  pub tracker_events : Vec<TrackerEvent>,
  pub tof_data       : Vec<u8>,
  pub raw_data       : Vec<u8>,
  pub flags0         : u8,
  pub flags1         : u8,
}

impl MergedEvent {

  pub fn new() -> Self {
    Self {
      header         : TelemetryHeader::new(),
      creation_time  : 0,
      event_id       : 0,
      tracker_events : Vec::<TrackerEvent>::new(),
      tof_data       : Vec::<u8>::new(),
      raw_data       : Vec::<u8>::new(),
      flags0         : 0,
      flags1         : 1,
    
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
    let _version      = parse_u8(stream, pos);
    //println!("_version {}", _version);
    me.flags0         = parse_u8(stream, pos);
    // skip a bunch of Alex newly implemented things
    // FIXME
    *pos += 8;

    me.event_id       = parse_u32(stream, pos);
    //println!("EVENT ID {}", me.event_id);
    let _tof_delim    = parse_u8(stream, pos);
    //println!("TOF delim : {}", _tof_delim);
    let num_tof_bytes = parse_u16(stream, pos);
    //println!("Num TOF bytes : {}", num_tof_bytes);
    if stream.len() < *pos+num_tof_bytes as usize {
      println!("Not enough bytes for TOF packet! Expected {}, seen {}", *pos+num_tof_bytes as usize, stream.len());
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
    for ev in &self.tracker_events {
      evids.push(ev.event_id);
      for h in &ev.hits { 
        if h.adc != 0 {
          good_hits += 1;
        }
      }
    }
    evids.sort();
    evids.dedup();
    repr += &(format!("  {}", self.header));
    repr += "\n  ** ** ** MERGED  ** ** **";
    repr += &(format!("\n  event ID        {}", self.event_id));  
    repr += &(format!("\n  -- TOF          {}", tof_evid));
    repr += &(format!("\n  -- TRK          {:?}", evids));
    repr += "\n  ** ** ** TRACKER ** ** **";
    repr += &(format!("\n  N Trk events    {}", self.tracker_events.len()));
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
    h.sys_time    = parse_u64(stream, pos);
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
    //tp.telemetry_header = TelemetryHeader::from_bytestream(stream, pos)?;
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
        println!(">170 events in this packet!");
        return Err(SerializationError::StreamTooLong);
      }
    }
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


