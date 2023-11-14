//! Input/Output 
//!
//! * Read files into memory
//!   
//!
//!
//!
//!

// change if we switch to a firmware
// where the byteorder of u32 and larger 
// is correct.
const REVERSE_WORDS : bool = true;

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
    Read
};
use std::collections::{
    VecDeque,
    HashMap
};

extern crate indicatif;
use indicatif::{ProgressBar, ProgressStyle};

use crate::errors::SerializationError;
use crate::events::{
    RBEvent,
    RBEventHeader,
    EventStatus,
};
use crate::serialization::{
    u8_to_u16,
    search_for_u16,
    Serialization,
    parse_u8,
    parse_u16,
    parse_u8_deque,
    parse_u16_deque,
};



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
#[derive(Debug, Clone)]
pub struct RBEventMemoryStreamer {
  pub stream      : Vec<u8>,
  pub pos         : usize,
  pub pos_at_head : bool
}

impl RBEventMemoryStreamer {

  pub fn new() -> Self {
    Self {
      stream      : Vec::<u8>::new(),
      pos         : 0,
      pos_at_head : false,
    }
  }

  pub fn add(&mut self, stream : &Vec<u8>, nbytes : usize) {
    //self.stream.extend(stream.iter().copied());
    //println!("self.pos {}", self.pos);
    //println!("Stream before {}",self.stream.len());
    self.stream.extend_from_slice(&stream[0..nbytes]);
    //println!("Stream after {}",self.stream.len());
  }


  /// Headers are expected to be a 2byte signature, 
  /// e.g. 0xaaaa. 
  ///
  /// # Arguments:
  ///   half_header : literally one half of the 2byte 
  ///                 header. E.g. if the header is 
  ///                 expected to be 0xaaaa, this 
  ///                 would be 0xaa
  /// # Returns
  /// 
  ///   * success   : header found
  pub fn seek_next_header(&mut self, half_header : u8) -> bool{
    //let start_pos = self.pos;
    //let mut byte1_found = false;
    //let mut byte1_pos   = 0usize;
    match search_for_u16(0xaaaa, &self.stream, self.pos) {
      Err(err) => {
        return false;
      }
      Ok(head_pos) => {
        self.pos = head_pos;
        self.pos_at_head = true;
        return true;
      }
    }
  }
}

impl Iterator for RBEventMemoryStreamer {
  type Item = RBEvent;

  fn next(&mut self) -> Option<Self::Item> {
    let begin_pos = self.pos; // in case we need
                              // to reset the position
    if self.stream.len() == 0 {
      error!("Stream empty!");
      return None;
    }
    if !self.pos_at_head {
      if !self.seek_next_header(0xaa) {
        debug!("Could not find another header...");
        self.pos = begin_pos;
        return None;
      }
    }
    // now we need to check for the minimum size of 
    // the expected RBEvent
    // the fixed size of header + footer is 42 bytes
    if !(self.stream.len() > self.pos + 42) {
      debug!("Less than 42 bytes reamin in stream after pos {}. This is not enough to extract status, len and roi, rb id and ch mask. The event might be incomplete and we will need more bytes to digest", self.pos);
      return None;
    }
    //for k in self.pos..self.pos + 42 {
    //  println!("word {}", self.stream[k]);
    //}
    let mut header       = RBEventHeader::new();
    let mut event        = RBEvent::new();
    let mut event_status = EventStatus::Perfect;
    // start parsing
    let head   = parse_u16(&self.stream, &mut self.pos);
    let status = parse_u16(&self.stream, &mut self.pos);
    //let head_pos   = search_for_u16(RBEvent::HEAD, &self.stream, self.pos); 
    // At this state, this can be a header or a full event. Check here and
    // proceed depending on the options
    header.parse_status(status);
    let packet_len = parse_u16(&self.stream, &mut self.pos) as usize * 2;
    let nwords     = parse_u16(&self.stream, &mut self.pos) as usize + 1; // the field will tell you the 
    // now we skip the next 10 bytes, 
    // they are dna, rsv, rsv, rsv, fw_hash
    self.pos += 10;
    self.pos += 1; // rb id first byte is rsvd
    header.rb_id     =  parse_u8(&self.stream, &mut self.pos);
    let channel_mask = parse_u16(&self.stream, &mut self.pos); 
    header.channel_mask = channel_mask; 
    //header.parse_channel_mask(channel_mask);
    //println!("Header channels {:?}", header.channels);
    let event_id0    = parse_u16(&self.stream, &mut self.pos);
    let event_id1    = parse_u16(&self.stream, &mut self.pos);
    let mut event_id : u32;
    if REVERSE_WORDS {
      event_id = u32::from(event_id0) << 16 | u32::from(event_id1);
    } else {
      event_id = u32::from(event_id1) << 16 | u32::from(event_id0);
    }
    header.event_id  = event_id;
    header.dtap0     = parse_u16(&self.stream, &mut self.pos);
    header.drs4_temp = parse_u16(&self.stream, &mut self.pos);
    let timestamp0   = parse_u16(&self.stream, &mut self.pos);
    let timestamp1   = parse_u16(&self.stream, &mut self.pos);
    let timestamp2   = parse_u16(&self.stream, &mut self.pos);
    //println!("TIMESTAMPS {} {} {}", timestamp0, timestamp1, timestamp2);
    let mut timestamp16 : u16;
    let mut timestamp32 : u32;
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
    let mut read_channels = true;
    if header.drs_lost_trigger() {
      read_channels = false;
    }
    if read_channels {
      // make sure we can read them!
      let expected_packet_size =   header.get_channels().len()*nwords*2 
                                 + header.get_channels().len()*2 
                                 + header.get_channels().len()*4;
      if self.stream.len() < self.pos + expected_packet_size + 2 + 4 + 2 {
        debug!("Stream ends prematurely, let's not return this event and rewind instead!");
        debug!("{} bytes missing!", self.pos + expected_packet_size + 2 + 4 + 2 - self.stream.len());
        self.pos = begin_pos;
        self.pos_at_head = false;
        return None;
      }
      for ch in header.get_channels().iter() {
        let ch_id = parse_u16(&self.stream, &mut self.pos);
        if ch_id == *ch as u16 {
          //println!("Got ch id {}", ch_id);
          //let header = parse_u16(&self.stream, &mut self.pos);
          // noice!!
          //let data : Vec<u8> = self.stream.iter().skip(self.pos).take(2*nwords).map(|&x| x).collect();
          let mut this_ch_adc = Vec::<u16>::with_capacity(nwords);
          for j in 0..nwords {
            this_ch_adc.push(0x3fff & parse_u16(&self.stream, &mut self.pos))
          }
          event.adc[*ch as usize] = this_ch_adc;
          //let data = &self.stream[self.pos..self.pos+2*nwords];
          //event.adc.push(u8_to_u16(&data));
          //self.pos += 2*nwords;
          let crc320 = parse_u16(&self.stream, &mut self.pos);
          let crc321 = parse_u16(&self.stream, &mut self.pos);
          let mut crc32 : u32;
          if REVERSE_WORDS {
            crc32 = u32::from(crc321) << 16 | u32::from(crc320);
          } else {
            crc32 = u32::from(crc320) << 16 | u32::from(crc321);
          }
        } else {
          error!("We saw a ch id of {} in the data, but this is not accounted for in the channel mask in the header!", ch_id);
          error!("We will skip this channel data, but that might cause corrupted event data!");
          self.pos += 2 + 2*nwords + 4;
        }
      }
    }
    if !header.drs_lost_trigger() {
      header.stop_cell = parse_u16(&self.stream, &mut self.pos);
    }
    let crc320       = parse_u16(&self.stream, &mut self.pos);
    let crc321       = parse_u16(&self.stream, &mut self.pos);
    let mut crc32 : u32;
    if REVERSE_WORDS {
      crc32 = u32::from(crc320) << 16 | u32::from(crc321);
    } else {
      crc32 = u32::from(crc321) << 16 | u32::from(crc320);
    }
    
    let tail         = parse_u16(&self.stream, &mut self.pos);
    if tail != 0x5555 {
      error!("Tail signature is wrong! Got {}", tail);
      for k in self.pos - 10..self.pos+10 {
        //println!("broken tail word {}", self.stream[k]);
        event_status = EventStatus::TailWrong;
      }
      //header.broken = true;
      //return Some(event);
      //panic!("uff der titanic!");
    }
    self.stream.drain(0..self.pos);
    //self.seek_next_header(0xaa);
    //println!("{} {}", self.pos, self.stream.len());
    self.pos = 0;
    self.pos_at_head = false;
    event.header = header;
    event.status = event_status;
    Some(event)
  }
}

/// Read RB binary (robin) files. These are also 
/// known as "blob" files
///
/// The robin reader consumes a file. 
///
///
#[derive(Debug)]
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
    self.generate_index();
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
    for k in sorted_keys {
      //println!("{k} -> {}", reverse_index[&k]);
      //n += 1;
      //if n == 8000 {break;}
    }
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
      let file = OpenOptions::new().create(false).append(false).read(true).open(path).expect("Unable to open file {filename}");
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
        const chunksize : usize  = 200000;
        let mut buffer      = [0u8;chunksize];
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
                  Err(err) => {},
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



