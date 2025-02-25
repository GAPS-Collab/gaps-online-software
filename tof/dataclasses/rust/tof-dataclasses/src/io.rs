//! Dataio - readino/writing of different types
//!
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

use std::fmt;

use crc::Crc;

use std::path::Path;
use std::fs::{
    self,
    File,
    OpenOptions
};

use std::io;
use std::io::{
  ErrorKind,
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
use regex::Regex;

use crate::events::{
    RBEvent,
    RBEventHeader,
    EventStatus,
};
use crate::packets::{
    TofPacket,
    PacketType,
};
use crate::constants::{
    NWORDS,
    HUMAN_TIMESTAMP_FORMAT
};
use crate::serialization::{
    Serialization,
    Packable,
    //SerializationError,
    u8_to_u16_14bit,
    u8_to_u16_err_check,
    search_for_u16,
    parse_u8,
    parse_u16,
    parse_u32,
};

use crate::events::TofEvent;

/// Types of files
#[derive(Debug, Clone)]
pub enum FileType {
  Unknown,
  /// Calibration file for specific RB with id
  CalibrationFile(u8),
  /// A regular run file with TofEvents
  RunFile(u32),
  /// A file created from a file with TofEvents which 
  /// contains only TofEventSummary
  SummaryFile(String),
}

/// Get a human readable timestamp
pub fn get_utc_timestamp() -> String {
  let now: DateTime<Utc> = Utc::now();
  //let timestamp_str = now.format("%Y_%m_%d-%H_%M_%S").to_string();
  let timestamp_str = now.format(HUMAN_TIMESTAMP_FORMAT).to_string();
  timestamp_str
}

/// Create date string in YYMMDD format
pub fn get_utc_date() -> String {
  let now: DateTime<Utc> = Utc::now();
  //let timestamp_str = now.format("%Y_%m_%d-%H_%M_%S").to_string();
  let timestamp_str = now.format("%y%m%d").to_string();
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

/// Take a .tof.gaps file with TofEvents and keep all packets, 
/// but reduce the TofEvents to TofEventSuammry to conserve space.
pub fn summarize_toffile(fname : String) {
  let mut reader    = TofPacketReader::new(fname.clone());
  let outfile       = fname.replace(".tof.", ".tofsum."); 
  let outfile_type  = FileType::SummaryFile(fname.clone());
  let mut writer    = TofPacketWriter::new(outfile,outfile_type); 
  let mut n_errors  = 0u32;
  let npack : usize = reader.get_packet_index().unwrap_or(HashMap::<PacketType,usize>::new()).values().cloned().collect::<Vec<usize>>().iter().sum();
  let bar_template : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.blue/grey} {pos:>7}/{len:7}";
  let bar_style  = ProgressStyle::with_template(bar_template).expect("Unable to set progressbar style!");
  let bar_label  = String::from("Reading events");
  let bar = ProgressBar::new(npack as u64);
  bar.set_position(0);
  bar.set_message (bar_label);
  bar.set_prefix  ("\u{2728}");
  bar.set_style   (bar_style);
  let mut npack = 0u64;
  for pack in reader {
    npack += 1;
    bar.set_position(npack);
    match pack.packet_type {
      PacketType::TofEvent => {
        match pack.unpack::<TofEvent>() {
          Err(err) => {
            debug!("Can't unpack TofEvent! {err}");
            n_errors += 1;
          }
          Ok(te) => {
            let ts = te.get_summary();
            let tp = ts.pack();
            writer.add_tof_packet(&tp); 
          }
        }
      }
      _ => {
        writer.add_tof_packet(&pack);
      }
    }
  }
  bar.finish_with_message("Done!");
  if n_errors > 0 {
    error!("Unpacking TofEvents from {} failed {} times!", n_errors, fname);
  }
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
  pub stream               : Vec<u8>,
  /// Error checking mode - check error bits for 
  /// channels/cells
  pub check_channel_errors : bool,
  /// Ignore channels in this list
  pub mask                 : Vec<u8>,

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
      mask                 : Vec::<u8>::new(),
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
    header.rb_id        = parse_u8(&self.stream, &mut self.pos);
    header.set_channel_mask(parse_u16(&self.stream, &mut self.pos)); 
    match replace_channel_mask {
      None => (),
      Some(mask) => {
        println!("==> Replacing ch mask {} with {}", header.get_channel_mask(), mask);
        header.set_channel_mask(mask); 
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
    let mut any_cell_error = false;
    let mut header_channels = header.get_channels().clone();
    for k in &self.mask {
      header_channels.retain(|x| x != k);
    }

    for ch in header_channels.iter() {
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
                any_cell_error = true;
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
              any_cell_error = true;
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
    if any_cell_error {
      if event_status == EventStatus::ChnSyncErrors {
        event_status = EventStatus::CellAndChnSyncErrors;
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


/// Generics for packet reading (TofPacket, Telemetry packet,...)
/// FIXME - not implemented yet
pub trait PacketReader {
  /// header bytes, e.g. 0xAAAA for TofPackets
  const HEADER0 : u8 = 0;
  const HEADER1 : u8 = 0;

  /// Manage the internal cursor attribute
  fn set_cursor(&mut self, pos : usize);

  /// Rewind the file, so it can be read again from the 
  /// beginning
  fn rewind(&mut self) -> io::Result<()> {
    //self.file_reader.rewind()?;
    self.set_cursor(0);
    Ok(())
  }
}




/// Read serialized TofPackets from an existing file or directory
///
/// This can read the "TOF stream" files, typically suffixed with .tof.gaps
/// These files are typically written by a TofPacketReader instance, e.g. as 
/// on the TOF flight computer
#[derive(Debug)]
pub struct TofPacketReader {
  /// Read from this file
  pub filenames       : Vec<String>,
  file_reader         : BufReader<File>,
  /// Current (byte) position in the file
  cursor              : usize,
  /// Read only packets of type == PacketType
  pub filter          : PacketType,
  /// Number of read packets
  n_packs_read        : usize,
  /// Number of skipped packets
  n_packs_skipped     : usize,
  /// Skip the first n packets
  pub skip_ahead      : usize,
  /// Stop reading after n packets
  pub stop_after      : usize,
  /// The index of the current file in the internal "filenames" vector.
  pub file_index      : usize,
}

impl fmt::Display for TofPacketReader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut range_repr = String::from("");
    if self.skip_ahead > 0 {
      range_repr += &(format!("({}", self.skip_ahead));
    } else {
      range_repr += "(";
    }
    if self.stop_after > 0 {
      range_repr += &(format!("..{})", self.stop_after));
    } else {
      range_repr += "..)";
    }
    let repr = format!("<TofPacketReader :read {} packets, filter {}, range {}\n files {:?}>", self.n_packs_read, self.filter, range_repr, self.filenames);
    write!(f, "{}", repr)
  }
}

impl TofPacketReader {

  fn list_path_contents_sorted(input: &str) -> Result<Vec<String>, io::Error> {
    let path = Path::new(input);
    match fs::metadata(path) {
      Ok(metadata) => {
        if metadata.is_file() {
          //return Ok(vec![path.file_name()
          let fname = String::from(input);
          return Ok(vec![fname]);
          //return Ok(vec![path
          //  .and_then(|name| name.to_str())
          //  .map(String::from)
          //  .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "Invalid filename"))?]);
        } 
        if metadata.is_dir() {
          let re = Regex::new(r"Run\d+_\d+\.(\d{6})_(\d{6})UTC\.tof\.gaps$").unwrap();

          let mut entries: Vec<(u32, u32, String)> = fs::read_dir(path)?
            .filter_map(Result::ok) // Ignore unreadable entries
            .filter_map(|entry| {
              //let filename = String::from(entry.file_name().into_string().ok()?); // Convert to String
              let filename = format!("{}/{}", path.display(), entry.file_name().into_string().ok()?);
              re.captures(&filename.clone()).map(|caps| {
                let date = caps.get(1)?.as_str().parse::<u32>().ok()?;
                let time = caps.get(2)?.as_str().parse::<u32>().ok()?;
                Some((date, time, filename))
              })?
            })
            .collect();

          // Sort by (date, time)
          entries.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
          // Return only filenames
          return Ok(entries.into_iter().map(|(_, _, name)| name).collect());
        } 
        Err(io::Error::new(ErrorKind::Other, "Path exists but is neither a file nor a directory"))
      }
      Err(e) => Err(e),
    }
  }

  /// Setup a new Reader, allowing the argument to be either the name of a single file or 
  /// the name of a directory
  pub fn new(filename_or_directory : String) -> TofPacketReader {
    let firstfile : String;
    match TofPacketReader::list_path_contents_sorted(&filename_or_directory) {
      Err(err) => {
        error!("{} does not seem to be either a valid directory or an existing file! {err}", filename_or_directory);
        panic!("Unable to open files!");
      }
      Ok(files) => {
        firstfile = files[0].clone();
        match OpenOptions::new().create(false).append(false).read(true).open(&firstfile) {
          Err(err) => {
            error!("Unable to open file {firstfile}! {err}");
            panic!("Unable to create reader from {filename_or_directory}!");
          }
          Ok(file) => {
            let packet_reader = Self { 
              filenames       : files,
              file_reader     : BufReader::new(file),
              cursor          : 0,
              filter          : PacketType::Unknown,
              n_packs_read    : 0,
              skip_ahead      : 0,
              stop_after      : 0,
              n_packs_skipped : 0,
              file_index      : 0,
            };
            packet_reader
          }
        }
      }
    } 
  }

  /// The very first TofPacket for a reader
  ///
  ///
  pub fn first_packet(&mut self) -> Option<TofPacket> {
    self.rewind();
    let pack = self.get_next_packet();
    self.rewind();
    return pack;
  }

  /// Te very last TofPacket for a reader
  pub fn last_packet(&mut self) -> Option<TofPacket> { 
    self.file_index = self.filenames.len() - 1;
    let lastfilename = self.filenames[self.file_index].clone();
    let lastfile     = OpenOptions::new().create(false).append(false).read(true).open(lastfilename).expect("Unable to open file {nextfilename}");
    self.file_reader = BufReader::new(lastfile);
    self.cursor      = 0;
    let mut tp = TofPacket::new();
    let mut idx = 0;
    loop {
      match self.get_next_packet() {
        None => {
          self.rewind();
          if idx == 0 {
            return None;
          } else {
            return Some(tp);
          }
        }
        Some(pack) => {
          idx += 1;
          tp = pack;
          continue;
        }
      }
    }
  }


  #[deprecated(since="0.10.0", note="Use public attribute instead!")]
  pub fn set_filter(&mut self, ptype : PacketType) {
    self.filter = ptype;
  }

  /// Get an index of the file - count number of packets
  ///
  /// Returns the number of all PacketTypes in the file
  pub fn get_packet_index(&mut self) -> io::Result<HashMap<PacketType, usize>> {
    let mut index  = HashMap::<PacketType, usize>::new();
    let mut buffer = [0];
    loop {
      match self.file_reader.read_exact(&mut buffer) {
        Err(err) => {
          debug!("Unable to read from file! {err}");
          //return None;
          break;
        }
        Ok(_) => {
          self.cursor += 1;
        }
      }
      if buffer[0] != 0xAA {
        continue;
      } else {
        match self.file_reader.read_exact(&mut buffer) {
          Err(err) => {
            debug!("Unable to read from file! {err}");
            //return None;
            break;
          }
          Ok(_) => {
            self.cursor += 1;
          }
        }

        if buffer[0] != 0xAA { 
          continue;
        } else {
          // the 3rd byte is the packet type
          match self.file_reader.read_exact(&mut buffer) {
             Err(err) => {
              debug!("Unable to read from file! {err}");
              break;
            }
            Ok(_) => {
              self.cursor += 1;
            }
          }
          let ptype    = PacketType::from(buffer[0]);
          // read the the size of the packet
          let mut buffer_psize = [0,0,0,0];
          match self.file_reader.read_exact(&mut buffer_psize) {
            Err(err) => {
              error!("Unable to read from file! {err}");
              break;
            }
            Ok(_) => {
              self.cursor += 4;
            }
          }
          let vec_data = buffer_psize.to_vec();
          let size     = parse_u32(&vec_data, &mut 0);
          match self.file_reader.seek(SeekFrom::Current(size as i64)) {
            Err(err) => {
              debug!("Unable to read more data! {err}");
              break; 
            }
            Ok(_) => {
              self.cursor += size as usize;
              // and then we add the packet type to the 
              // hashmap
              //let ptype_key = ptype as u8;
              if index.contains_key(&ptype) {
                *index.get_mut(&ptype).unwrap() += 1;
              } else {
                index.insert(ptype, 1usize);
              }
            }
          }
        }
      } // if no 0xAA found
    } // end loop
    self.rewind()?;
    Ok(index)
  } // end fn

  pub fn rewind(&mut self) -> io::Result<()> {
    let firstfile = &self.filenames[0];
    match OpenOptions::new().create(false).append(false).read(true).open(&firstfile) {
      Err(err) => {
        error!("Unable to open file {firstfile}! {err}");
        panic!("Unable to create reader from {firstfile}!");
      }
      Ok(file) => {
        self.file_reader  = BufReader::new(file);
      }
    }   
    self.cursor     = 0;
    self.file_index = 0;
    Ok(())
  }

  /// Return the next tofpacket in the stream
  ///
  /// Will return none if the file has been exhausted.
  /// Use ::rewind to start reading from the beginning
  /// again.
  pub fn get_next_packet(&mut self) -> Option<TofPacket> {
    // filter::Unknown corresponds to allowing any

    let mut buffer = [0];
    loop {
      match self.file_reader.read_exact(&mut buffer) {
        Err(err) => {
          debug!("Unable to read from file! {err}");
          if self.file_index == self.filenames.len() -1 {
            return None;
          } else {
            self.file_index += 1;
            let nextfilename = self.filenames[self.file_index].clone();
            let nextfile     = OpenOptions::new().create(false).append(false).read(true).open(nextfilename).expect("Unable to open file {nextfilename}");
            self.file_reader = BufReader::new(nextfile);
            self.cursor      = 0;
            return self.get_next_packet();
          }
        }
        Ok(_) => {
          self.cursor += 1;
        }
      }
      if buffer[0] != 0xAA {
        continue;
      } else {
        match self.file_reader.read_exact(&mut buffer) {
          Err(err) => {
            debug!("Unable to read from file! {err}");
            if self.file_index == self.filenames.len() -1 {
              return None;
            } else {
              self.file_index += 1;
              let nextfilename = self.filenames[self.file_index].clone();
              let nextfile = OpenOptions::new().create(false).append(false).read(true).open(nextfilename).expect("Unable to open file {nextfilename}");
              self.file_reader = BufReader::new(nextfile);
              self.cursor      = 0;
              return self.get_next_packet();
            }
          }
          Ok(_) => {
            self.cursor += 1;
          }
        }

        if buffer[0] != 0xAA { 
          continue;
        } else {
          // the 3rd byte is the packet type
          match self.file_reader.read_exact(&mut buffer) {
             Err(err) => {
              debug!("Unable to read from file! {err}");
              if self.file_index == self.filenames.len() -1 {
                return None;
              } else {
                self.file_index += 1;
                let nextfilename = self.filenames[self.file_index].clone();
                let nextfile = OpenOptions::new().create(false).append(false).read(true).open(nextfilename).expect("Unable to open file {nextfilename}");
                self.cursor      = 0;
                self.file_reader = BufReader::new(nextfile);
                return self.get_next_packet();
              }
            }
            Ok(_) => {
              self.cursor += 1;
            }
          }
          let ptype    = PacketType::from(buffer[0]);
          // read the the size of the packet
          let mut buffer_psize = [0,0,0,0];
          match self.file_reader.read_exact(&mut buffer_psize) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              if self.file_index == self.filenames.len() -1 {
                return None;
              } else {
                self.file_index += 1;
                let nextfilename = self.filenames[self.file_index].clone();
                let nextfile = OpenOptions::new().create(false).append(false).read(true).open(nextfilename).expect("Unable to open file {nextfilename}");
                self.cursor      = 0;
                self.file_reader = BufReader::new(nextfile);
                return self.get_next_packet();
              }
            }
            Ok(_) => {
              self.cursor += 4;
            }
          }
          let vec_data = buffer_psize.to_vec();
          let size     = parse_u32(&vec_data, &mut 0);
          if ptype != self.filter && self.filter != PacketType::Unknown {
            match self.file_reader.seek(SeekFrom::Current(size as i64)) {
              Err(err) => {
                debug!("Unable to read more data! {err}");
                if self.file_index == self.filenames.len() -1 {
                  return None;
                } else {
                  self.file_index += 1;
                  let nextfilename = self.filenames[self.file_index].clone();
                  let nextfile = OpenOptions::new().create(false).append(false).read(true).open(nextfilename).expect("Unable to open file {nextfilename}");
                  self.cursor      = 0;
                  self.file_reader = BufReader::new(nextfile);
                  return self.get_next_packet();
                }
              }
              Ok(_) => {
                self.cursor += size as usize;
              }
            }
            continue; // this is just not the packet we want
          }
          // now at this point, we want the packet!
          // except we skip ahead or stop earlier
          if self.skip_ahead > 0 && self.n_packs_skipped < self.skip_ahead {
            // we don't want it
            match self.file_reader.seek(SeekFrom::Current(size as i64)) {
              Err(err) => {
                debug!("Unable to read more data! {err}");
                if self.file_index == self.filenames.len() -1 {
                  return None;
                } else {
                  self.file_index += 1;
                  let nextfilename = self.filenames[self.file_index].clone();
                  let nextfile = OpenOptions::new().create(false).append(false).read(true).open(nextfilename).expect("Unable to open file {nextfilename}");
                  self.cursor      = 0;
                  self.file_reader = BufReader::new(nextfile);
                  return self.get_next_packet();
                }
              }
              Ok(_) => {
                self.n_packs_skipped += 1;
                self.cursor += size as usize;
              }
            }
            continue; // this is just not the packet we want
          }
          if self.stop_after > 0 && self.n_packs_read >= self.stop_after {
            // we don't want it
            match self.file_reader.seek(SeekFrom::Current(size as i64)) {
              Err(err) => {
                debug!("Unable to read more data! {err}");
                if self.file_index == self.filenames.len() -1 {
                  return None;
                } else {
                  self.file_index += 1;
                  let nextfilename = self.filenames[self.file_index].clone();
                  let nextfile = OpenOptions::new().create(false).append(false).read(true).open(nextfilename).expect("Unable to open file {nextfilename}");
                  self.cursor      = 0;
                  self.file_reader = BufReader::new(nextfile);
                  return self.get_next_packet();
                }
              }
              Ok(_) => {
                self.cursor += size as usize;
              }
            }
            continue; // this is just not the packet we want

          }

          let mut tp = TofPacket::new();
          tp.packet_type = ptype;
          let mut payload = vec![0u8;size as usize];

          match self.file_reader.read_exact(&mut payload) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              if self.file_index == self.filenames.len() -1 {
                return None;
              } else {
                self.file_index += 1;
                let nextfilename = self.filenames[self.file_index].clone();
                let nextfile = OpenOptions::new().create(false).append(false).read(true).open(nextfilename).expect("Unable to open file {nextfilename}");
                self.cursor      = 0;
                self.file_reader = BufReader::new(nextfile);
                return self.get_next_packet();
              }
            }
            Ok(_) => {
              self.cursor += size as usize;
            }
          }
          tp.payload = payload;
          // we don't filter, so we like this packet
          let mut tail = vec![0u8; 2];
          match self.file_reader.read_exact(&mut tail) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              if self.file_index == self.filenames.len() -1 {
                return None;
              } else {
                self.file_index += 1;
                let nextfilename = self.filenames[self.file_index].clone();
                let nextfile = OpenOptions::new().create(false).append(false).read(true).open(nextfilename).expect("Unable to open file {nextfilename}");
                self.cursor      = 0;
                self.file_reader = BufReader::new(nextfile);
                return self.get_next_packet();
              }
            }
            Ok(_) => {
              self.cursor += 2;
            }
          }
          let tail = parse_u16(&tail,&mut 0);
          if tail != TofPacket::TAIL {
            debug!("TofPacket TAIL signature wrong!");
            return None;
          }
          self.n_packs_read += 1;
          return Some(tp);
        }
      } // if no 0xAA found
    } // end loop
  } // end fn
}

impl Default for TofPacketReader {
  fn default() -> Self {
    TofPacketReader::new(String::from(""))
  }
}

impl Iterator for TofPacketReader {
  type Item = TofPacket;
  
  fn next(&mut self) -> Option<Self::Item> {
    self.get_next_packet()
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
  /// The maximum number of (Mega)bytes
  /// per file. After this a new file 
  /// is started
  pub mbytes_per_file : usize,
  /// add timestamps to filenames
  pub file_type       : FileType,
  pub file_name       : String,

  file_id             : usize,
  /// internal packet counter, number of 
  /// packets which went through the writer
  n_packets           : usize,
  /// internal counter for bytes written in 
  /// this file
  file_nbytes_wr      : usize,
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
    let file_name : String;
    if !file_path.ends_with("/") {
      file_path += "/";
    }
    match file_type {
      FileType::Unknown => {
        let filename = file_path.clone() + "Data.tof.gaps";
        let path     = Path::new(&filename); 
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
        file_name = filename;
      }
      FileType::RunFile(runid) => {
        let filename = format!("{}{}", file_path, get_runfilename(runid, 0, None));
        let path     = Path::new(&filename); 
        println!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
        file_name = filename;
      }
      FileType::CalibrationFile(rbid) => {
        let filename = format!("{}{}", file_path, get_califilename(rbid, false));
        //let filename = file_path.clone() + &get_califilename(rbid,false);
        let path     = Path::new(&filename); 
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
        file_name = filename;
      }
      FileType::SummaryFile(ref fname) => {
        let filename = fname.replace(".tof.", ".tofsum.");
        let path     = Path::new(&filename);
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
        file_name = filename;
      }
    }
    Self {
      file,
      file_path        : file_path,
      pkts_per_file    : 0,
      mbytes_per_file  : 420,
      file_nbytes_wr   : 0,    
      file_type        : file_type,
      file_id          : 1,
      n_packets        : 0,
      file_name        : file_name,
    }
  }

  pub fn get_file(&self) -> File { 
    let file : File;
    match &self.file_type {
      FileType::Unknown => {
        let filename = self.file_path.clone() + "Data.tof.gaps";
        let path     = Path::new(&filename); 
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
      }
      FileType::RunFile(runid) => {
        let filename = format!("{}{}", self.file_path, get_runfilename(*runid, self.file_id as u64, None));
        //let filename = self.file_path.clone() + &get_runfilename(runid,self.file_id as u64, None);
        let path     = Path::new(&filename); 
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
      }
      FileType::CalibrationFile(rbid) => {
        //let filename = self.file_path.clone() + &get_califilename(rbid,false);
        let filename = format!("{}{}", self.file_path, get_califilename(*rbid, false));
        let path     = Path::new(&filename); 
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
      }
      FileType::SummaryFile(fname) => {
        let filename = fname.replace(".tof.", ".tofsum.");
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
    self.file_nbytes_wr += buffer.len();
    match self.file.write_all(buffer.as_slice()) {
      Err(err) => error!("Writing to file to path {} failed! {}", self.file_path, err),
      Ok(_)    => ()
    }
    self.n_packets += 1;
    let mut newfile = false;
    if self.pkts_per_file != 0 {
      if self.n_packets == self.pkts_per_file {
        newfile = true;
        self.n_packets = 0;
      }
    } else if self.mbytes_per_file != 0 {
      // multiply by mebibyte
      if self.file_nbytes_wr >= self.mbytes_per_file * 1_048_576 {
        newfile = true;
        self.file_nbytes_wr = 0;
      }
    }
    if newfile {
        //let filename = self.file_prefix.clone() + "_" + &self.file_id.to_string() + ".tof.gaps";
        match self.file.sync_all() {
          Err(err) => {
            error!("Unable to sync file to disc! {err}");
          },
          Ok(_) => ()
        }
        self.file = self.get_file();
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
//#[deprecated(since="0.10.0", note="There are no robin files anymore. RBs will write data with RBEvents wrapped in TofPackets!")]
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


