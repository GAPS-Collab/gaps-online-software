//! Dataio - readino/writing of different types
//! of TOF data products.
//!
//! * TofPacketReader/Writer - sequentially read/write
//!   TofPackets from/to a file on disk.
//!   -> upcoming: Will connect to a network socket
//!
//! * RobinReader: Read (old) RB data files, where 
//!   the file is simply a dump of the internal 
//!   buffers ("RBEventMemoryView").
//!
//! * RBEventMemoryStreamer: Walk over "raw" RBEvents
//!   representations ("RBEventMemoryView") and extract
//!   RBEvents
//!

// change if we switch to a firmware
// where the byteorder of u32 and larger 
// is correct.
const REVERSE_WORDS : bool = true;
const ALGO : crc::Algorithm<u32> = crc::Algorithm {
      width   : 32u8,
      init    : 0xFFFFFFFF,
      //poly    : 0xEDB88320,
      poly    : 0x04C11DB7,
      refin   : true,
      refout  : true,
      xorout  : 0xFFFFFFFF,
      check   : 0,
      residue : 0,
    };

extern crate crc;
use crc::Crc;

use std::path::Path;
use std::fs::{
    self,
    File,
    OpenOptions
};
use std::io;
use std::io::{
    BufReader,
    Seek,
    SeekFrom,
    Read,
    Write,
};
use std::collections::{
    VecDeque,
    HashMap
};

extern crate chrono;
use chrono::{DateTime, Utc};

extern crate indicatif;
use indicatif::{ProgressBar, ProgressStyle};
use crossbeam_channel::Sender;

use crate::events::{
    RBEvent,
    RBEventHeader,
    EventStatus,
};
use crate::packets::{
    TofPacket,
    PacketType,
};
use crate::constants::NWORDS;
use crate::serialization::{
    Serialization,
    SerializationError,
    u8_to_u16_14bit,
    u8_to_u16_err_check,
    search_for_u16,
    parse_u8,
    parse_u16,
    parse_u32,
};

/// Types of files
#[derive(Debug, Copy, Clone)]
pub enum FileType {
  Unknown,
  /// Calibration file for specific RB with id
  CalibrationFile(u8),
  RunFile(u32),
}

/// Get a human readable timestamp
pub fn get_utc_timestamp() -> String {
  let now: DateTime<Utc> = Utc::now();
  // Format the timestamp as "YYYY_MM_DD_HH_MM"
  let timestamp_str = now.format("%Y_%m_%d-%H_%M_%S").to_string();
  timestamp_str
}

/// A standardized name for calibration files saved by 
/// the liftof suite
///
/// # Arguments
///
/// * rb_id   : unique identfier for the 
///             Readoutboard (1-50)
/// * default : if default, just add 
///             "latest" instead of 
///             a timestamp
pub fn get_califilename(rb_id : u8, latest : bool) -> String {
  let ts = get_utc_timestamp();
  if latest {
    format!("RB{rb_id:02}_latest.cali.tof.gaps")
  } else {
    format!("RB{rb_id:02}_{ts}.cali.tof.gaps")
  }
}

/// A standardized name for regular run files saved by
/// the liftof suite
///
/// # Arguments
///
/// * run    : run id (identifier)
/// * subrun : subrun id (identifier of file # within
///            the run
/// * rb_id  : in case this should be used on the rb, 
///            a rb id can be specified as well
pub fn get_runfilename(run : u32, subrun : u64, rb_id : Option<u8>) -> String {
  let ts = get_utc_timestamp();
  let fname : String;
  match rb_id {
    None => {
      fname = format!("Run{run}_{subrun}.{ts}.tof.gaps");
    }
    Some(rbid) => {
      fname = format!("Run{run}_{subrun}.{ts}.RB{rbid:02}.tof.gaps");
    }
  }
  fname
}

//FIXME : this needs to become a trait
fn read_n_bytes(file: &mut BufReader<File>, n: usize) -> io::Result<Vec<u8>> {
  let mut buffer = vec![0u8; n];
  match file.read_exact(&mut buffer) {
    Err(ref err) if err.kind() == io::ErrorKind::UnexpectedEof => {
      // Reached the end of the file
      buffer = Vec::<u8>::new();
      file.read_to_end(&mut buffer)?;
      return Ok(buffer);
    },
    Err(err) => {
      error!("Can not read {n} bytes from file! Error {err}");
      return Err(err);
    },
    Ok(_) => ()
  }
  Ok(buffer)
}

/// Read an entire file into memory
///
/// Represents the contents of a file 
/// as a byte vector
/// 
/// # Arguments:
///
/// * filename : full path to the file to be read
pub fn read_file(filename: &Path) -> io::Result<Vec<u8>> {
  info!("Reading file {}", filename.display());
  let mut f = File::open(&filename)?;
  let metadata = fs::metadata(&filename)?;
  let mut buffer = vec![0; metadata.len() as usize];
  info!("Read {} bytes from {}", buffer.len(), filename.display());
  // read_exact if the amount is not specified
  f.read_exact(&mut buffer)?;
  Ok(buffer)
}


/// Emit RBEvents from a stream of bytes
/// from RBMemory
///
/// The layout of the stream has to have
/// the fpga fw memory layout.
///
/// This provides a next() method to act
/// as a generator for RBEvents
pub struct RBEventMemoryStreamer {
  /// Raw stream read out from the RB buffers.
  pub stream         : Vec<u8>,
  /// Error checking mode - check error bits for 
  /// channels/cells
  pub check_channel_errors : bool,

  /// Current position in the stream
  pos                      : usize,
  /// The current posion marker points to a header 
  /// signature in the stream.
  pos_at_head              : bool,
  /// An optional crossbeam::channel Sender, which 
  /// will allow to send TofPackets
  pub tp_sender            : Option<Sender<TofPacket>>,
  /// number of extracted events from stream
  /// this manages how we are draining the stream
  n_events_ext             : usize,
  pub is_depleted          : bool,
  /// Calculate the crc32 checksum for the channels 
  /// everytime next() is called
  pub calc_crc32           : bool,
  /// placeholder for checksum calculator
  crc32_sum                : Crc::<u32>,
  pub request_mode         : bool,
  pub request_cache        : VecDeque<(u32,u8)>,
  /// an index for the events in the stream
  /// this links eventid and start position
  /// in the stream together
  pub event_map            : HashMap<u32,(usize,usize)>,
  pub first_evid           : u32,
  pub last_evid            : u32,
  pub last_event_complete  : bool,
  pub last_event_pos       : usize,
  /// When in request mode, number of events the last event in the stream is behind the
  /// first request
  pub is_behind_by         : usize,
  /// When in request mode, number of events the last event in the stream is ahead the
  /// last request
  pub is_ahead_by          : usize,
}

impl RBEventMemoryStreamer {

  pub fn new() -> Self {
    Self {
      stream               : Vec::<u8>::new(),
      check_channel_errors : false,
      pos                  : 0,
      pos_at_head          : false,
      tp_sender            : None,
      n_events_ext         : 0,
      is_depleted          : false,
      calc_crc32           : false,
      crc32_sum            : Crc::<u32>::new(&ALGO),
      request_mode         : false,
      request_cache        : VecDeque::<(u32,u8)>::new(),
      event_map            : HashMap::<u32,(usize,usize)>::new(),
      first_evid           : 0,
      last_evid            : 0,
      last_event_complete  : false,
      last_event_pos       : 0,
      is_behind_by         : 0,
      is_ahead_by          : 0,
    }
  }
 
  /// Create the event index, which is
  /// a map of event ids and position 
  /// + length in the stream
  pub fn create_event_index(&mut self) { //-> Result<Ok, SerializationError> {
    let begin_pos = self.pos;
    let mut event_id = 0u32;
    // we are now at head, 
    // read packet len and event id
    loop {
      let mut result = (0usize, 0usize);
      if !self.seek_next_header(0xaaaa) {
        debug!("Could not find another header...");
        self.pos = begin_pos;
        self.last_evid = event_id;
        if result.0 + result.1 > self.stream.len() - 1 {
          self.last_event_complete = false;
        } else {
          self.last_event_complete = true;
        }
        info!("Indexed {} events from {} to {}", self.event_map.len(), self.first_evid, self.last_evid);
        return;
      }
      result.0 = self.pos;
      self.pos += 4;//header, status
      let packet_len = parse_u16(&self.stream, &mut self.pos) as usize * 2;
      if self.stream.len() < self.pos -6 + packet_len {
        //self.is_depleted = true;
        self.pos = begin_pos;
        self.last_evid = event_id;
        info!("Indexed {} events from {} to {}", self.event_map.len(), self.first_evid, self.last_evid);
        return;
        //return Err(SerializationError::StreamTooShort);
      }
      result.1 = packet_len;
      if packet_len < 6 {
        self.pos = begin_pos;
        self.last_evid = event_id;
        info!("Indexed {} events from {} to {}", self.event_map.len(), self.first_evid, self.last_evid);
        return;
        //return Err(SerializationError::StreamTooShort);
      }
      // rewind
      self.pos -= 6;
      // event id is at pos 22
      self.pos += 22;
      let event_id0    = parse_u16(&self.stream, &mut self.pos);
      let event_id1    = parse_u16(&self.stream, &mut self.pos);
      if REVERSE_WORDS {
        event_id = u32::from(event_id0) << 16 | u32::from(event_id1);
      } else {
        event_id = u32::from(event_id1) << 16 | u32::from(event_id0);
      }
      if self.first_evid == 0 {
        self.first_evid = event_id;
      }
      self.pos += packet_len - 26;
      self.event_map.insert(event_id,result);
    }
  }

  pub fn print_event_map(&self) {
    for k in self.event_map.keys() {
      let pos = self.event_map[&k];
      println!("-- --> {} -> {},{}", k, pos.0, pos.1);
    }
  }

  // EXPERIMENTAL
  pub fn init_sender(&mut self, tp_sender : Sender<TofPacket>) {
    self.tp_sender = Some(tp_sender);
  }

  // EXPERIMENTAL
  pub fn send_all(&mut self) {
    loop {
      match self.next() {
        None => {
          info!("Streamer drained!");
          break;
        },
        Some(ev) => {
          let tp = TofPacket::from(&ev);
          match self.tp_sender.as_ref().expect("Sender needs to be initialized first!").send(tp) {
            Ok(_) => (),
            Err(err) => {
              error!("Unable to send TofPacket! {err}");
            }
          }
        }
      }
    }
  }


  // FIXME - performance. Don't extend it. It would be
  // better if we'd consume the stream without 
  // reallocating memory.
  pub fn add(&mut self, stream : &Vec<u8>, nbytes : usize) {
    //self.stream.extend(stream.iter().copied());
    //println!("self.pos {}", self.pos);
    //println!("Stream before {}",self.stream.len());
    self.is_depleted = false;
    self.stream.extend_from_slice(&stream[0..nbytes]);
    //self.create_event_index();
    //println!("Stream after {}",self.stream.len());
  }

  /// Take in a stream by consuming it, that means moving
  /// This will avoid clones.
  pub fn consume(&mut self, stream : &mut Vec<u8>) {
    self.is_depleted = false;
    // FIXME: append can panic
    // we use it here, since it does not clone
    //println!("[io.rs] consuming {} bytes", stream.len());
    self.stream.append(stream);
    //println!("[io.rs] stream has now {} bytes", self.stream.len());
    //self.create_event_index();
  }

  /// Headers are expected to be a 2byte signature, 
  /// e.g. 0xaaaa. 
  ///
  /// # Arguments:
  ///   header : 2byte header.
  ///
  /// # Returns
  /// 
  ///   * success   : header found
  pub fn seek_next_header(&mut self, header : u16) -> bool{
    match search_for_u16(header, &self.stream, self.pos) {
      Err(_) => {
        return false;
      }
      Ok(head_pos) => {
        self.pos = head_pos;
        self.pos_at_head = true;
        return true;
      }
    }
  }

  pub fn next_tofpacket(&mut self) -> Option<TofPacket> {
    let begin_pos = self.pos; // in case we need
                              // to reset the position
    let foot_pos : usize;
    let head_pos : usize;
    if self.stream.len() == 0 {
      trace!("Stream empty!");
      return None;
    }
    if !self.pos_at_head {
      if !self.seek_next_header(0xaaaa) {
        debug!("Could not find another header...");
        self.pos = begin_pos;
        return None;
      }
    }
    head_pos  = self.pos;
    //let mut foot_pos  = self.pos;
    //head_pos = self.pos;
    if !self.seek_next_header(0x5555) {
      debug!("Could not find another footer...");
      self.pos = begin_pos;
      return None;
    }
    //println!("{} {} {}", self.stream.len(), head_pos, foot_pos);
    foot_pos = self.pos;
    self.n_events_ext += 1;
    let mut tp = TofPacket::new();
    tp.packet_type = PacketType::RBEventMemoryView;
    //let mut payload = Vec::<u8>::with_capacity(18530);
    tp.payload.extend_from_slice(&self.stream[head_pos..foot_pos+2]);
    //tp.payload = payload;
    //self.pos += 2;
    self.pos_at_head = false;
    //self.stream.drain(0..foot_pos);
    //self.pos = 0;
    if self.n_events_ext % 200 == 0 {
      self.stream.drain(0..foot_pos+3);
      self.pos = 0;
    }
    Some(tp)
  }


  /// Retrive an RBEvent from a certain position
  pub fn get_event_at_pos_unchecked(&mut self,
                                    replace_channel_mask : Option<u16>)
      -> Option<RBEvent> {
    let mut header       = RBEventHeader::new();
    let mut event        = RBEvent::new();
    let mut event_status = EventStatus::Unknown;
    //let begin_pos = self.pos;
    if self.calc_crc32 && self.check_channel_errors {
      event_status = EventStatus::Perfect;
    }
    if !self.calc_crc32 && !self.check_channel_errors {
      event_status = EventStatus::GoodNoCRCOrErrBitCheck;
    }
    if !self.calc_crc32 && self.check_channel_errors {
      event_status = EventStatus::GoodNoCRCCheck;
    }
    if self.calc_crc32 && !self.check_channel_errors {
      event_status = EventStatus::GoodNoErrBitCheck;
    }
    // start parsing
    //let first_pos = self.pos;
    let head   = parse_u16(&self.stream, &mut self.pos);
    if head != RBEventHeader::HEAD {
      error!("Event does not start with {}", RBEventHeader::HEAD);
      return None;
    }

    let status = parse_u16(&self.stream, &mut self.pos);
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    header.parse_status(status);
    let packet_len = parse_u16(&self.stream, &mut self.pos) as usize * 2;
    let nwords     = parse_u16(&self.stream, &mut self.pos) as usize + 1; // the field will tell you the 
    if self.pos - 8 + packet_len > self.stream.len() { // -1?
      error!("Stream is too short! Stream len is {}, packet len is {}. We are at pos {}", self.stream.len(), packet_len, self.pos);
      self.is_depleted = true;
      self.pos -= 8;
      return None;
    }
    // now we skip the next 10 bytes, 
    // they are dna, rsv, rsv, rsv, fw_hash
    self.pos += 10;
    self.pos += 1; // rb id first byte is rsvd
    header.rb_id        =  parse_u8(&self.stream, &mut self.pos);
    header.channel_mask = parse_u16(&self.stream, &mut self.pos); 
    match replace_channel_mask {
      None => (),
      Some(mask) => {
        println!("==> Replacing ch mask {} with {}", header.channel_mask, mask);
        header.channel_mask    = mask; 
      }
    }
    let event_id0       = parse_u16(&self.stream, &mut self.pos);
    let event_id1       = parse_u16(&self.stream, &mut self.pos);
    let event_id : u32;
    if REVERSE_WORDS {
      event_id = u32::from(event_id0) << 16 | u32::from(event_id1);
    } else {
      event_id = u32::from(event_id1) << 16 | u32::from(event_id0);
    }
    
    header.event_id  = event_id;
    // we are currently not using these
    //let _dtap0       = parse_u16(&self.stream, &mut self.pos);
    //let _drs4_temp   = parse_u16(&self.stream, &mut self.pos);
    self.pos += 4;
    let timestamp0   = parse_u16(&self.stream, &mut self.pos);
    let timestamp1   = parse_u16(&self.stream, &mut self.pos);
    let timestamp2   = parse_u16(&self.stream, &mut self.pos);
    //println!("TIMESTAMPS {} {} {}", timestamp0, timestamp1, timestamp2);
    let timestamp16 : u16;
    let timestamp32 : u32;
    if REVERSE_WORDS {
      timestamp16 = timestamp0;
      timestamp32 = u32::from(timestamp1) << 16 | u32::from(timestamp2);
    } else {
      timestamp16 = timestamp2;
      timestamp32 = u32::from(timestamp0) << 16 | u32::from(timestamp1);
    }
    header.timestamp16 = timestamp16;
    header.timestamp32 = timestamp32;
    // now the payload
    //println!("{}", header);
    //println!("{}", nwords);
    if header.drs_lost_trigger() {
      event.status = EventStatus::IncompleteReadout;
      event.header = header;
      //self.pos_at_head = false;
      return Some(event);
    }
    // make sure we can read them!
    //let expected_packet_size =   header.get_channels().len()*nwords*2 
    //                           + header.get_channels().len()*2 
    //                           + header.get_channels().len()*4;
    for ch in header.get_channels().iter() {
      let ch_id = parse_u16(&self.stream, &mut self.pos);
      if ch_id != *ch as u16 {
        // check where is the next header
        let search_pos = self.pos;
        match search_for_u16(TofPacket::HEAD, &self.stream, search_pos) {
          Err(_) => (),
          Ok(result) => {
            info!("The channel data is corrupt, but we found a header at {} for remaining stream len {}", result, self.stream.len()); 
          }
        }
        let mut stream_view = Vec::<u8>::new();
        let foo_pos = self.pos;
        for k in foo_pos -3..foo_pos + 3 {
          stream_view.push(self.stream[k]);
        }
        error!("We got {ch_id} but expected {ch} for event {}. The parsed ch id is not in the channel mask! We will fill this channel with u16::MAX .... Stream view +- 3 around the ch id {:?}", header.event_id, stream_view);
        event_status = EventStatus::ChannelIDWrong;
        // we fill the channel with MAX values:
        event.adc[*ch as usize] = vec![u16::MAX;NWORDS];
        self.pos += 2*nwords + 4;
        continue;
      } else {
      //if ch_id == *ch as u16 {
        //println!("Got ch id {}", ch_id);
        //let header = parse_u16(&self.stream, &mut self.pos);
        // noice!!
        //let data : Vec<u8> = self.stream.iter().skip(self.pos).take(2*nwords).map(|&x| x).collect();
         
        let mut dig = self.crc32_sum.digest();
        if self.calc_crc32 {
          let mut this_ch_adc = Vec::<u16>::with_capacity(nwords);
          for _ in 0..nwords {
            let this_field = parse_u16(&self.stream, &mut self.pos);
            dig.update(&this_field.to_le_bytes());
            if self.check_channel_errors {
              if ((0x8000 & this_field) >> 15) == 0x1 {
                error!("Ch error bit set for ch {}!", ch);
                event_status = EventStatus::ChnSyncErrors;
              }
              if ((0x4000 & this_field) >> 14) == 0x1 {
                error!("Cell error bit set for ch {}!", ch);
                event_status = EventStatus::CellSyncErrors;
              }
            }
            this_ch_adc.push(0x3fff & this_field)
          }
          event.adc[*ch as usize] = this_ch_adc;
        } else {
          if self.check_channel_errors {
            let adc_w_errs = u8_to_u16_err_check(&self.stream[self.pos..self.pos + 2*nwords]);
            if adc_w_errs.1 {
              error!("Ch error bit set for ch {}!", ch);
              event_status = EventStatus::ChnSyncErrors;
            } else if adc_w_errs.2 {
              error!("Cell error bit set for ch {}!", ch);
              event_status = EventStatus::CellSyncErrors;
            }
            event.adc[*ch as usize] = adc_w_errs.0;
          } else {
            event.adc[*ch as usize] = u8_to_u16_14bit(&self.stream[self.pos..self.pos + 2*nwords]);
          }
          self.pos += 2*nwords;
        } 
        //let data = &self.stream[self.pos..self.pos+2*nwords];
        //self.pos += 2*nwords;
        let crc320 = parse_u16(&self.stream, &mut self.pos);
        let crc321 = parse_u16(&self.stream, &mut self.pos);
        //let checksum = self.crc32_sum.clone().finalize();
        if self.calc_crc32 {
          let crc32 : u32;
          if REVERSE_WORDS {
            crc32 = u32::from(crc320) << 16 | u32::from(crc321);
          } else {
            crc32 = u32::from(crc321) << 16 | u32::from(crc320);
          }
          let checksum = dig.finalize();
          if checksum != crc32 {
            event_status = EventStatus::CRC32Wrong;
          }
          println!("== ==> Checksum {}, channel checksum {}!", checksum, crc32); 
        }
      }
    }
    
    if !header.drs_lost_trigger() {
      header.stop_cell = parse_u16(&self.stream, &mut self.pos);
    }
    // CRC32 checksum - next 4 bytes
    // FIXME
    // skip crc32 checksum
    self.pos += 4;

    // in principle there is a checksum for the whole event, whcih
    // we are currently not using (it is easy to spot wrong bytes
    // in the header)
    //let crc320         = parse_u16(&self.stream, &mut self.pos);
    //let crc321         = parse_u16(&self.stream, &mut self.pos);
    //if self.calc_crc32 {
    //  let crc32 : u32;
    //  if REVERSE_WORDS {
    //    crc32 = u32::from(crc320) << 16 | u32::from(crc321);
    //  } else {
    //    crc32 = u32::from(crc321) << 16 | u32::from(crc320);
    //  }
    //  warn!("Checksum test for the whole event is not yet implemented!");
    //  //if event.header.crc32 != crc32 {
    //  //  trace!("Checksum test for the whole event is not yet implemented!");
    //  //}
    //}
    
    let tail         = parse_u16(&self.stream, &mut self.pos);
    if tail != 0x5555 {
      error!("Tail signature {} for event {} is invalid!", tail, header.event_id);
      event_status = EventStatus::TailWrong;
    } 
    //self.stream.drain(0..self.pos);
    self.pos_at_head = false;
    event.header = header;
    event.status = event_status;
    if event_status == EventStatus::TailWrong {
      info!("{}", event);
    }
    Some(event)
  }

  pub fn get_event_at_id(&mut self, event_id : u32, replace_channel_mask : Option<u16>) -> Option<RBEvent> {
    let begin_pos = self.pos; // in case we need
                              // to reset the position
    //println!("--> Requested {}", event_id);
    //if self.event_map.contains_key(&event_id) {
    //  //println!("-- We have it!");
    //} else {
    //  //println!("-- We DON'T have it, event_map len {}", self.event_map.len());
    //  //self.print_event_map();
    //  //println!("-- last event id {}", self.last_evid);
    //  //println!("-- first event id {}", self.first_evid);
    //}
    let pos = self.event_map.remove(&event_id)?;
    if self.stream.len() < pos.0 + pos.1 {
      trace!("Stream is too short!");
      self.is_depleted = true;
      self.pos = begin_pos;
      return None;
    }
    self.pos = pos.0;
    self.get_event_at_pos_unchecked(replace_channel_mask)
  }
}

impl Iterator for RBEventMemoryStreamer {
  type Item = RBEvent;

  fn next(&mut self) -> Option<Self::Item> {
    // FIXME - we should init this only once
    // event id from stream
    //let event_id  = 0u32;
    let begin_pos : usize; // in case we need
                           // to rewind
     
    self.pos_at_head = false;
    begin_pos = self.pos; // in case we need
                                // to reset the position
    if self.stream.len() == 0 {
      trace!("Stream empty!");
      self.is_depleted = true;
      self.pos = 0;
      return None;
    }
    if !self.pos_at_head {
      if !self.seek_next_header(0xaaaa) {
        debug!("Could not find another header...");
        self.pos = begin_pos;
        self.is_depleted = true;
        return None;
      }
    }
    
    let event          = self.get_event_at_pos_unchecked(None)?;
    self.n_events_ext += 1;
    self.stream.drain(0..self.pos);
    self.pos           = 0;
    self.pos_at_head   = false;
    Some(event)
  }
}

/// Read serialized TofPackets from an existing file
///
/// This is mainly to read stream files previously
/// written by TofPacketWriter
#[derive(Debug)]
pub struct TofPacketReader {

  pub filename    : String,
  file_reader     : Option<BufReader<File>>,
  cursor          : usize
}

impl TofPacketReader {

  pub fn new(filename : String) -> TofPacketReader {
    let filename_c = filename.clone();
    let mut packet_reader = TofPacketReader { 
      filename       : filename,
      file_reader    : None,
      cursor : 0,
    };
    packet_reader.open(filename_c);
    packet_reader
  }
 
  pub fn get_next_packet_size(&self, stream : &Vec<u8>) -> u32 {
    // cursor needs at HEAD position and then we have to 
    // add one byte for the packet type
    let mut pos    = self.cursor + 2;
    let ptype_int  = parse_u8(stream, &mut pos);
    let next_psize = parse_u32(stream, &mut pos);
    let ptype      = ptype_int as u8;
    debug!("We anticpate a TofPacket of type {:?} and size {} (bytes)",ptype, next_psize);
    next_psize
  }

  pub fn open(&mut self, filename : String) {
    if self.filename != "" {
      warn!("Overiding previously set filename {}", self.filename);
    }
    let self_filename = filename.clone();
    self.filename     = self_filename;
    if filename != "" {
      let path = Path::new(&filename); 
      info!("Reading from {}", &self.filename);
      let file = OpenOptions::new().create(false).append(false).read(true).open(path).expect("Unable to open file {filename}");
      self.file_reader    = Some(BufReader::new(file));
      self.init();
    }
  }

  fn init(&mut self) {
    match self.search_start() {
      Err(err) => {
        error!("Can not find any header signature (typically 0xAAAA) in file! Err {err}");
        panic!("This is most likely a useless endeavour! Hence, I panic!");
      }
      Ok(start_pos) => {
        self.cursor = start_pos;
      }
    }
  }

  fn search_start(&mut self) -> Result<usize, SerializationError> {
    let mut pos       = 0u64;
    let mut start_pos = 0usize; 
    //let mut stream  = Vec::<u8>::new();
    let max_bytes   = self.get_file_nbytes();
    info!("Using file with {max_bytes} bytes!");
    let chunk       = 10usize;
    while pos < max_bytes {
      match read_n_bytes(self.file_reader.as_mut().unwrap(), chunk) {
        Err(err) => {
          error!("Can not read from file, error {err}");
          error!("Most likely, the file/stream is too short!");
          return Err(SerializationError::StreamTooShort);
        }
        Ok(stream) => {
          debug!("Got stream {:?}", stream);
          match search_for_u16(TofPacket::HEAD, &stream, 0) {
            Err(_) => {
              pos += chunk as u64;
              continue;
            }
            Ok(result) => {
              start_pos = result + pos as usize;           
              // make sure the current chunk is accounted 
              // for before the break
              pos += chunk as u64;
              break;
            }
          }
        } // end Ok
      } // end match 
    } // end while
    let mut rewind : i64 = pos.try_into().unwrap();
    rewind = -1*rewind + start_pos as i64;
    debug!("Rewinding {rewind} bytes");
    match self.file_reader.as_mut().unwrap().seek(SeekFrom::Current(rewind)) {
      Err(err) => {
        error!("Can not rewind file buffer! Error {err}");
      }
      Ok(_) => ()
    }
    Ok(start_pos)
  }
  
  fn get_file_nbytes(&self) -> u64 {
    let metadata  = self.file_reader.as_ref().unwrap().get_ref().metadata().unwrap();
    let file_size = metadata.len();
    file_size
  }
}

impl Default for TofPacketReader {
  fn default() -> Self {
    TofPacketReader::new(String::from(""))
  }
}

impl Iterator for TofPacketReader {
  type Item = TofPacket;
  //type Item = io::Result<TofPacket>;

  fn next(&mut self) -> Option<Self::Item> {
    let packet : TofPacket;
    match read_n_bytes(self.file_reader.as_mut().unwrap(), 7) { 
    //match self.file_reader.as_mut().expect("No file available!").read_until(b'\n', &mut line) {
      Err(err) => {
        error!("Error reading from file {} error: {}", self.filename, err);
      },
      Ok(chunk) => {
        if chunk.len() < 7 {
          error!("The stream is too short!");
          return None;
        }
        else {
          trace!("Read {} bytes", chunk.len());
          let expected_payload_size = self.get_next_packet_size(&chunk) as usize; 
          match read_n_bytes(self.file_reader.as_mut().unwrap(), expected_payload_size + 2) {
            Err(err) => {
              error!("Unable to read {} requested bytes to decode tof packet! Err {err}", expected_payload_size - 7 );
              return None;
            },
            Ok(data) => {
              let mut stream = Vec::<u8>::with_capacity(expected_payload_size + 9);
              stream.extend_from_slice(&chunk);
              stream.extend_from_slice(&data);
              //println!("{:?}", stream);
              match TofPacket::from_bytestream(&stream, &mut 0) {
                Ok(pack) => {
                  packet = pack;
                  return Some(packet);
                }
                Err(err) => { 
                  error!("Error getting packet from file {err}");
                  return None;
                }
              }
            }
          }
        }
      }, // end outer OK
    }
  None
  }
}

/// Write TofPackets to disk.
///
/// Operates sequentially, packets can 
/// be added one at a time, then will
/// be synced to disk.
pub struct TofPacketWriter {

  pub file            : File,
  /// location to store the file
  pub file_path       : String,
  /// The maximum number of packets 
  /// for a single file. Ater this 
  /// number is reached, a new 
  /// file is started.
  pub pkts_per_file   : usize,
  /// add timestamps to filenames
  pub file_type       : FileType,

  file_id             : usize,
  /// internal packet counter, number of 
  /// packets which went through the writer
  n_packets           : usize,
}

impl TofPacketWriter {

  /// Instantiate a new PacketWriter 
  ///
  /// # Arguments
  ///
  /// * file_prefix     : Prefix file with this string. A continuous number will get 
  ///                     appended to control the file size.
  /// * file_type       : control the behaviour of how the filename is
  ///                     assigned.
  pub fn new(mut file_path : String, file_type : FileType) -> Self {
    //let filename = file_prefix.clone() + "_0.tof.gaps";
    let file : File;
    if !file_path.ends_with("/") {
      file_path += "/";
    }
    match file_type {
      FileType::Unknown => {
        let filename = file_path.clone() + "Data.tof.gaps";
        let path     = Path::new(&filename); 
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
      }
      FileType::RunFile(runid) => {
        let filename = file_path.clone() + &get_runfilename(runid,1, None);
        let path     = Path::new(&filename); 
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
      }
      FileType::CalibrationFile(rbid) => {
        let filename = file_path.clone() + &get_califilename(rbid,false);
        let path     = Path::new(&filename); 
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
      }
    }
    Self {
      file,
      file_path       : file_path,
      pkts_per_file   : 3000,
      file_type       : file_type,
      file_id         : 1,
      n_packets       : 0,
    }
  }

  pub fn get_file(&self) -> File { 
    let file : File;
    match self.file_type {
      FileType::Unknown => {
        let filename = self.file_path.clone() + "Data.tof.gaps";
        let path     = Path::new(&filename); 
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
      }
      FileType::RunFile(runid) => {
        let filename = self.file_path.clone() + &get_runfilename(runid,self.file_id as u64, None);
        let path     = Path::new(&filename); 
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
      }
      FileType::CalibrationFile(rbid) => {
        let filename = self.file_path.clone() + &get_califilename(rbid,false);
        let path     = Path::new(&filename); 
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
      }
    }
    file
  }

  /// Induce serialization to disk for a TofPacket
  ///
  ///
  pub fn add_tof_packet(&mut self, packet : &TofPacket) {
    let buffer = packet.to_bytestream();
    match self.file.write_all(buffer.as_slice()) {
      Err(err) => error!("Writing to file to path {} failed! {}", self.file_path, err),
      Ok(_)    => ()
    }
    self.n_packets += 1;
    if self.n_packets == self.pkts_per_file {
      //let filename = self.file_prefix.clone() + "_" + &self.file_id.to_string() + ".tof.gaps";
      match self.file.sync_all() {
        Err(err) => {
          error!("Unable to sync file to disc! {err}");
        },
        Ok(_) => ()
      }
      self.file = self.get_file();
      self.n_packets = 0;
      self.file_id += 1;
      //let path  = Path::new(&filename);
      //println!("==> [TOFPACKETWRITER] Will start a new file {}", path.display());
      //self.file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
      //self.n_packets = 0;
      //self.file_id += 1;
    }
  debug!("TofPacket written!");
  }
}

impl Default for TofPacketWriter {
  fn default() -> TofPacketWriter {
    TofPacketWriter::new(String::from(""), FileType::Unknown)
  }
}

/// Read RB binary (robin) files. These are also 
/// known as "blob" files
///
/// The robin reader consumes a file. 
///
///
pub struct RobinReader {
  pub streamer    : RBEventMemoryStreamer,
  pub filename    : String,
  file_reader     : Option<BufReader<File>>,
  pub board_id    : u8,
  // cache events
  cache           : HashMap<u32, RBEvent>, 
  // event id position of in stream
  index           : HashMap<u32, usize>,
  /// number of events we have successfully parsed from the file
  n_events_read   : usize,
  n_bytes_read    : usize,
  pub eof_reached : bool,
  pub extra_filenames : Vec<String>,
}

impl RobinReader {

  /// The "old" Robin files have a fixed 
  /// bytesize by design
  const EVENT_SIZE : usize = 18530;

  pub fn new(filename : String) -> Self {
    let filename_c = filename.clone();
    let mut robin_reader = Self { 
      streamer        : RBEventMemoryStreamer::new(),
      filename        : String::from(""),
      file_reader     : None,
      board_id        : 0,
      cache           : HashMap::<u32,RBEvent>::new(),
      index           : HashMap::<u32,usize>::new(),
      eof_reached     : false,
      n_events_read   : 0,
      n_bytes_read    : 0,
      extra_filenames : Vec::<String>::new(),
    };
    robin_reader.open(filename_c);
    robin_reader.init();
    robin_reader
  }
 
  pub fn add_file(&mut self, filename : String) {
    self.extra_filenames.push(filename);
  }

  fn init(&mut self) {
    //match self.search_start() {
    //  Err(err) => {
    //    error!("Can not find any header signature (typically 0xAAAA) in file! Err {err}");
    //    panic!("This is most likely a useless endeavour! Hence, I panic!");
    //  }
    //  Ok(start_pos) => {
    //    self.cursor = start_pos;
    //  }
    //}
    // get the first event to infer board id, then rewind
    if let Some(ev) = self.next() {
      self.board_id = ev.header.rb_id;  
      let rewind : i64 = RobinReader::EVENT_SIZE.try_into().expect("That needs to fit!");
      match self.file_reader.as_mut().unwrap().seek(SeekFrom::Current(rewind)) {
        Err(err) => {
          error!("Read first event, but can not rewind stream! Err {}", err);
          panic!("I don't understand, panicking...");
        }
        Ok(_) => {
          self.n_bytes_read  = 0;
          self.n_events_read = 0;
        }
      }
    } else {
      panic!("I can not find a single event in this file! Panicking!");
    }
    //self.generate_index();
  }

  pub fn get_from_cache(&mut self, event_id : &u32) -> Option<RBEvent> {
    self.cache.remove(event_id)
  }

  pub fn cache_all_events(&mut self) {
    self.rewind();
    while !self.eof_reached {
      match self.next() {
        None => {
          break;
        }
        Some(ev) => {
          //println!("{}", ev.header.event_id); 
          self.cache.insert(ev.header.event_id, ev);
        }
      }
    }
    info!("Cached {} events!", self.cache.len());
  }

  /// Loop over the whole file and create a mapping event_id -> position
  ///
  /// This will allow to use the ::seek method
  ///
  pub fn generate_index(&mut self) {
    if self.n_events_read > 0 {
      error!("Can not generate index when events have already been read! Use ::rewind() first!");
      return;
    }
    self.n_events_read  = 0;
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} Generating eventid index...").unwrap());
    let mut seen_before  = 0usize;
    let mut total_events = 0usize;
    while !self.eof_reached { 
      if let Some(ev) = self.next() {
        if self.index.contains_key(&ev.header.event_id) {
          debug!("We have seen this event id {} before!", ev.header.event_id);
          seen_before += 1;
        }
        self.index.insert(ev.header.event_id,self.n_events_read);
        self.n_events_read += 1;
        total_events += 1;
      }
      pb.tick();
    }
    if seen_before > 0 {
      error!("There have been duplicate event ids! In total, we discard {}/{}", seen_before, total_events);
    }
    info!("Generated index by reading {} events!", self.n_events_read);
    self.rewind();
    info!("Generated index for {} events!", self.index.len());
  }

  pub fn get_cache_size(&self) -> usize {
    self.cache.len()
  }

  pub fn print_index(&self) {
    let mut reverse_index = HashMap::<usize, u32>::new();
    for k in self.index.keys() {
      reverse_index.insert(self.index[k], *k);
    }
    debug!("Generated reversed index of size {}", reverse_index.len());
    //println!("Index [reversed]:");
    //println!("\t pos -> event id");
    //println!("{:?}", reverse_index);
    //println!("{:?}", self.index);
    let mut sorted_keys: Vec<&usize> = reverse_index.keys().collect();
    sorted_keys.sort();
    //let mut n = 0u32;
    //for k in sorted_keys {
      //println!("{k} -> {}", reverse_index[&k]);
      //n += 1;
      //if n == 8000 {break;}
    //}
  }

  pub fn is_indexed(&self, event_id : &u32) -> bool {
    self.index.contains_key(event_id)
  }


  /// Get RBEvents from the file in ascending order of event ID
  ///
  /// In case the event_id jumps, this function is not suitable
  pub fn get_in_order(&mut self, event_id : &u32) -> Option<RBEvent> {
    if !self.is_indexed(event_id) {
      error!("Can not get event {} since it is not in the index!", event_id);
      return None;
    }
    let event_idx = self.index.remove(event_id).unwrap();
    if self.n_events_read > event_idx {
      error!("Can not get event {} since we have already read it. You can use ::rewind() and try again!", event_id);
      return None;
    } else {
      let delta = event_idx - self.n_events_read;
      let mut n_read = 0usize;
      //let mut ev = RBEvent::new();
      loop {
        match self.next() {
          Some(ev) => {
            n_read += 1;
            if n_read == delta {
              return Some(ev);
            }
          },
          None => {
            break;
          }
        }    
      }
    }
    None
  }
  
  /// Rewind the underlying file back to the beginning
  pub fn rewind(&mut self) {
    warn!("Rewinding {}", self.filename);
    let mut rewind : i64 = self.n_bytes_read.try_into().unwrap();
    rewind = -1*rewind;
    debug!("Attempting to rewind {rewind} bytes");
    match self.file_reader.as_mut().unwrap().seek(SeekFrom::Current(rewind)) {
      Err(err) => {
        error!("Can not rewind file buffer! Error {err}");
      }
      Ok(_) => {
        info!("File rewound by {rewind} bytes!");
        self.n_events_read = 0;
        self.n_bytes_read  = 0;
      }
    }
    self.eof_reached = false;
  }

  pub fn open(&mut self, filename : String) {
    if self.filename != "" {
      warn!("Overiding previously set filename {}", self.filename);
    }
    let self_filename = filename.clone();
    self.filename     = self_filename;
    if filename != "" {
      let path = Path::new(&filename); 
      info!("Reading from {}", &self.filename);
      let file = OpenOptions::new().create(false).append(true).read(true).open(path).expect("Unable to open file {filename}");
      self.file_reader = Some(BufReader::new(file));
    }
  }

  pub fn precache_events(&mut self, n_events : usize) {
    self.cache.clear();
    let mut n_ev = 0usize;
    if self.eof_reached {
      return;
    }
    for _ in 0..n_events {
      let event = self.next();
      n_ev += 1;
      if let Some(ev) = event {
        self.cache.insert(ev.header.event_id, ev);
      } else {
        error!("Can not cache {}th event!", n_ev);
        self.eof_reached = true;
        break
      }
    }
  }

  pub fn max_cached_event_id(&self) -> Option<u32> {
    let keys : Vec<u32> = self.cache.keys().cloned().collect();
    keys.iter().max().copied()
  }
  
  pub fn min_cached_event_id(&self) -> Option<u32> {
    let keys : Vec<u32> = self.cache.keys().cloned().collect();
    keys.iter().min().copied()
  }

  pub fn is_cached(&self, event_id : &u32) -> bool {
    let keys : Vec<&u32> = self.cache.keys().collect();
    keys.contains(&event_id)
  }

  pub fn get_event_by_id(&mut self, event_id : &u32) -> Option<RBEvent> {
    self.cache.remove(event_id)
  }

  pub fn is_expired(&self) -> bool {
    self.eof_reached && self.cache.len() == 0
  }

  pub fn event_ids_in_cache(&self) -> Vec<u32> {
    trace!("We have {} elements in the cache!", self.cache.len());
    let mut keys : Vec<u32> = self.cache.keys().cloned().collect();
    trace!("We have {} elements in the cache!", keys.len());
    keys.sort();
    keys
  }

  pub fn get_events(&self) -> Vec<RBEvent> {
    self.cache.values().cloned().collect()
  }

  pub fn count_packets(&self) -> u64 {
    let metadata  = self.file_reader.as_ref().unwrap().get_ref().metadata().unwrap();
    let file_size = metadata.len();
    let n_packets =  file_size/RobinReader::EVENT_SIZE as u64; 
    info!("The file {} contains likely ~{} event packets!", self.filename, n_packets);
    n_packets
  }
}

impl Default for RobinReader {

  fn default() -> Self {
    RobinReader::new(String::from(""))
  }
}

impl Iterator for RobinReader {
  type Item = RBEvent;

  fn next(&mut self) -> Option<Self::Item> {
    match self.streamer.next() {
      Some(event) => {
        return Some(event);
      },
      None => {
        // check if we can feed more data to the 
        // streamer
        const CHUNKSIZE : usize  = 200000;
        let mut buffer      = [0u8;CHUNKSIZE];
        match self.file_reader.as_mut().unwrap().read(&mut buffer) {
          Err(err) => {
            error!("Unable to read any bytes from file {}, {}", self.filename, err);
            return None;
          },
          Ok(_nbytes) => {
            self.n_bytes_read += _nbytes;
            if _nbytes == 0 {
              self.eof_reached = true;
              if self.extra_filenames.len() > 0 {
                let next_filename = self.extra_filenames.pop().unwrap();
                self.open(next_filename);
                self.eof_reached = false;
                match self.file_reader.as_mut().unwrap().read(&mut buffer) {
                  Err(err) => {
                    error!("Failed reading bytes from buffer! {}", err);
                  },
                  Ok(_nbytes2) => {}
                }
              }
              return None;
            }
            self.streamer.add(&buffer.to_vec(), _nbytes);
            match self.streamer.next() {
              None => {
                //println!("none..");
                return None;
              },
              Some(event) => {
                return Some(event);
                //println!("{}", event);
              } 
            }
          }
        }
      }
    }
  }
}

#[test]
fn crc32() {
  let crc32_sum = Crc::<u32>::new(&ALGO);
  let mut dig   = crc32_sum.digest();
  dig.update(&0u16.to_le_bytes());
  let result = dig.finalize();
  //assert_eq!(stream.len(), RBEventHeader::SIZE);
  assert_eq!(1104745215,result);
}


