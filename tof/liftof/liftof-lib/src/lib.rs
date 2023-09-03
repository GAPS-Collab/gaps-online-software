use std::error::Error;
use std::time::{Duration, Instant};
//use std::thread;
use std::fmt;
use std::{fs, fs::File, path::Path};
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use std::net::{IpAddr, Ipv4Addr};
use std::io::{Read,
              Write,
              Seek,
              SeekFrom};
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use crossbeam_channel::Receiver;
use zmq;

extern crate json;

use macaddr::MacAddr6;
use netneighbours::get_mac_to_ip_map;
use crossbeam_channel as cbc; 

extern crate indicatif;
use indicatif::{ProgressBar, ProgressStyle};

extern crate pretty_env_logger;
#[macro_use] extern crate log;
#[macro_use] extern crate manifest_dir_macros;

use tof_dataclasses::manifest as mf;
use tof_dataclasses::constants::NWORDS;
use tof_dataclasses::calibrations::ReadoutBoardCalibrations;
use tof_dataclasses::packets::{TofPacket,
                               PacketType,
                               PaddlePacket};
use tof_dataclasses::errors::{SerializationError,
                              AnalysisError};
use tof_dataclasses::serialization::{search_for_u16,
                                     parse_u8,
                                     parse_u32,
                                     Serialization};
use tof_dataclasses::commands::{TofCommand};//, TofResponse};
use tof_dataclasses::events::{MasterTriggerEvent,
                              RBEvent};
use tof_dataclasses::monitoring::MtbMoniData;
use tof_dataclasses::events::master_trigger::{reset_daq,
                                              read_daq,
                                              read_rate,
                                              //read_lost_rate,
                                              read_adc_temp_and_vccint,
                                              read_adc_vccaux_and_vccbram};

use tof_dataclasses::analysis::{calculate_pedestal,
                                integrate,
                                cfd_simple,
                                find_peaks};

pub const MT_MAX_PACKSIZE   : usize = 512;
pub const DATAPORT : u32 = 42000;

//*************************************************
// I/O - read/write (general purpose) files
//
//
pub fn read_value_from_file(file_path: &str) -> io::Result<u32> {
  let mut file = File::open(file_path)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  let value: u32 = contents.trim().parse().map_err(|err| {
    io::Error::new(io::ErrorKind::InvalidData, err)
  })?;
  Ok(value)
}

/// The output is wrapped in a Result to allow matching on errors
/// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}

/// Read a file as a vector of bytes
///
/// This reads the entire file in a 
/// single vector of chars.
///
/// # Arguments 
///
/// * fliename (String) : Name of the file to read in 
#[deprecated(since="0.4.0", note="please use `tof_dataclasses::io::read_file` instead")]
pub fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    return buffer;
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


/// Open a new file and write TofPackets
/// in binary representation
///
/// One packet per line
///
pub struct TofPacketWriter {

  //pub filename : String,
  pub file        : File,
  pub file_prefix : String,
  pkts_per_file   : usize,
  file_id         : usize,
  n_packets       : usize,
}

impl TofPacketWriter {

  /// Instantiate a new PacketWriter 
  ///
  /// # Arguments
  ///
  /// * file_prefix : Prefix file with this string. A continuous number will get 
  ///                 appended to control the file size.
  pub fn new(file_prefix : String) -> Self {
    let filename = file_prefix.clone() + "_0.tof.gaps";
    let path = Path::new(&filename); 
    println!("Writing to file {filename}");
    let file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
    Self {
      file,
      file_prefix   : file_prefix,
      pkts_per_file : 10000,
      file_id : 0,
      n_packets : 0,
    }
  }

  pub fn add_tof_packet(&mut self, packet : &TofPacket) {
    let buffer = packet.to_bytestream();
    match self.file.write_all(buffer.as_slice()) {
      Err(err) => error!("Writing to file with prefix {} failed. Err {}", self.file_prefix, err),
      Ok(_)    => ()
    }
    match self.file.sync_all() {
      Err(err) => error!("File syncing failed! error {err}"),
      Ok(_)    => ()
    }
    self.n_packets += 1;
    if self.n_packets == self.pkts_per_file {
      //drop(self.file);
      let filename = self.file_prefix.clone() + "_" + &self.file_id.to_string() + ".tof.gaps";
      let path  = Path::new(&filename);
      self.file = OpenOptions::new().append(true).open(path).expect("Unable to open file {filename}");
      self.n_packets = 0;
    }
  }
}

impl Default for TofPacketWriter {
  fn default() -> TofPacketWriter {
    TofPacketWriter::new(String::from(""))
  }
}

/**************************************************/

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
    let mut pos = self.cursor + 2;
    let ptype_int  = parse_u8(stream, &mut pos);
    let next_psize = parse_u32(stream, &mut pos);
    let ptype = PacketType::from_u8(ptype_int);
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


/**************************************************/

/// Read RB binary (robin) files. These are also 
/// known as "blob" files
///
/// The robin reader consumes a file. 
///
///
#[derive(Debug)]
pub struct RobinReader {
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
}

impl RobinReader {

  const EVENT_SIZE : usize = 18530;

  pub fn new(filename : String) -> Self {
    let filename_c = filename.clone();
    let mut robin_reader = Self { 
      filename      : String::from(""),
      file_reader   : None,
      board_id      : 0,
      cache         : HashMap::<u32,RBEvent>::new(),
      index         : HashMap::<u32,usize>::new(),
      eof_reached   : false,
      n_events_read : 0,
      n_bytes_read  : 0
    };
    robin_reader.open(filename_c);
    robin_reader.init();
    robin_reader
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
    println!("Index [reversed]:");
    println!("\t pos -> event id");
    //println!("{:?}", reverse_index);
    println!("{:?}", self.index);
    let mut sorted_keys: Vec<&usize> = reverse_index.keys().collect();
    sorted_keys.sort();
    //let mut n = 0u32;
    for k in sorted_keys {
      println!("{k} -> {}", reverse_index[&k]);
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
    let event_size = 18532usize;
    let chunk      = 1;
    let packet : RBEvent;
    match read_n_bytes(self.file_reader.as_mut().unwrap(), event_size*chunk) { 
    //match self.file_reader.as_mut().expect("No file available!").read_until(b'\n', &mut line) {
      Ok(buffer) => {
        if buffer.len() > 0 {
          trace!("Read {} bytes", buffer.len());
          self.n_bytes_read += buffer.len();
          for _ in 0..chunk {
            match RBEvent::extract_from_rbeventmemoryview(&buffer, &mut 0) {
              Ok(pack) => {
                packet = pack;
                self.n_events_read += 1;     
                return Some(packet);
              }
              Err(err) => { 
                error!("Error getting packet from file {err}");
                //return None;
                continue;
              }
            }
          }
        } else {
          warn!("End of file reached!");
          self.eof_reached = true;
          return None;
        }
      },
      Err(err) => {
        self.eof_reached = true;
        error!("Error reading from file {} error: {}", self.filename, err);
      }
    }
  None
  }
}

/**************************************************/

/// Broadcast commands over the tof-computer network
/// socket via zmq::PUB to the rb network.
/// Currently, the only participants in the rb network
/// are the readoutboards.
///
/// After the reception of a TofCommand, this will currently be 
/// broadcasted to all readout boards.
///
/// ISSUE/FIXME  : Send commands only to specific addresses.
///
/// # Arguments 
///
/// * cmd        : a [crossbeam] receiver, to receive 
///                TofCommands.
pub fn readoutboard_commander(cmd : Receiver<TofCommand>){
             
  info!("Initialiized");
  let ctx = zmq::Context::new();
  //let mut sockets = Vec::<zmq::Socket>::new();

  let mut address_ip = String::from("tcp://");
  //let this_board_ip = local_ip().expect("Unable to obtainl local board IP. Something is messed up!");
  let data_port    = DATAPORT;
  let this_board_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 1, 1));

  match this_board_ip {
    IpAddr::V4(ip) => address_ip += &ip.to_string(),
    IpAddr::V6(_) => panic!("Currently, we do not support IPV6!")
  }
  let data_address : String = address_ip.clone() + ":" + &data_port.to_string();
  let data_socket = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  data_socket.bind(&data_address).expect("Unable to bind to data (PUB) socket {data_adress}");
  println!("0MQ PUB socket bound to address {data_address}");
  //let init_run = TofCommand::DataRunStart(100000);
  //let mut payload_cmd  = init_run.to_bytestream();
  //let mut payload  = String::from("BRCT").into_bytes();
  //payload.append(&mut payload_cmd);

  println!("Starting cmd receiver loop!");
  loop {
    // check if we get a command from the main 
    // thread
    match cmd.try_recv() {
      Err(err) => trace!("Did not receive a new command, error {err}"),
      Ok(new_command) => {
        info!("Received new command!");
        let mut payload  = String::from("BRCT").into_bytes();
        let mut payload_cmd = new_command.to_bytestream();
        payload.append(&mut payload_cmd);
        println!("{:?}", payload);
        match data_socket.send(&payload,0) {
          Err(err) => error!("Can send command! Error {err}"),
          Ok(_)    => info!("BRCT command sent!")
        }
      }
    }
  }
}

//**********************************************
//
// Analysis
//


/// Basically Rene's analysis engine. This creates 
/// the PaddlePackets with reduced waveform 
/// information from a version of the ReadoutBoard
/// event. 
///
/// ISSUES:
/// In the future, we might want to refactor this 
/// a little bit and it might operate on a 
/// RBEvent instaad and fill it paddle packet 
/// members
///
/// FIXME - I think this should take a HashMap with 
/// calibration options, which we can load from a 
/// json file
pub fn waveform_analysis(event         : &RBEvent,
                         readoutboard  : &mf::ReadoutBoard,
                         calibration   : &ReadoutBoardCalibrations)
                         // settiungs will be future extension
                         //settings      : &HashMap<String, f32>) 
-> Result<Vec<PaddlePacket>, AnalysisError> {
  if event.header.broken {
    // just return the analysis error, there 
    // is probably nothing else we can do?
    return Err(AnalysisError::InputBroken);
  }
  let pids = readoutboard.get_all_pids();
  let mut paddles = HashMap::<u8, PaddlePacket>::new();
  // just paranoid
  if pids.len() != 4 {
    error!("RB {} seems to have a strange number of paddles ({}) connected!",
           readoutboard.rb_id, pids.len());
  }
  for k in pids.iter() {
    // fill the general information of 
    // the paddles already
    let mut pp   = PaddlePacket::new();
    pp.paddle_id = *k;
    pp.event_id  = event.header.event_id;
    // FIXME - think better about timestamps!
    //pp.timestamp_32 = blob_data.timestamp_32;
    //pp.timestamp_16 = blob_data.timestamp_16;
    match paddles.insert(*k, pp) {
      None => (),
      Some(_) => {
        error!("We have seen paddle id {k} already!");
      }
    };
  }
  // do the calibration
  let mut active_channels = event.header.get_active_data_channels();
  active_channels.push(9); // always do ch9 callibration
  // allocate memory for voltages
  // this allocates more memory than needed
  // (needed is active_channels.len()), however,
  // in flight operations all channels should
  // be active anyway.
  let mut all_voltages = Vec::<Vec::<f32>>::new();
  let mut all_times    = Vec::<Vec::<f32>>::new();
  for _ in 0..9 {
    let ch_voltages : Vec<f32>= vec![0.0; NWORDS];
    all_voltages.push(ch_voltages);
    let ch_times : Vec<f32>= vec![0.0; NWORDS];
    all_times.push(ch_times);
  }

  for active_ch in &active_channels {
    let ch = *active_ch as usize;
    let adc          = event.get_adc_ch(*active_ch);
    calibration.voltages(ch,
                         event.header.stop_cell as usize,
                         &adc,
                         &mut all_voltages[ch]);
    calibration.nanoseconds(ch,
                            event.header.stop_cell as usize,
                            &mut all_times[ch]);
  }
  match ReadoutBoardCalibrations::spike_cleaning(&mut all_voltages,
                                                 event.header.stop_cell) {
    Err(err) => {
      error!("Spike cleaning failed! Err {err}");
    }
    Ok(_)    => ()
  }

  // analysis
  for active_ch in &active_channels {
    let ch = *active_ch as usize;
    let (ped, ped_err) = calculate_pedestal(&all_voltages[ch],
                                            10.0, 10, 50);
    debug!("Got pedestal of {} +- {}", ped, ped_err);
    for n in 0..all_voltages[ch].len() {
      all_voltages[ch][n] -= ped;
    }
    let mut charge : f32 = 0.0;
    warn!("Check impedance value!");
    match integrate(&all_voltages[ch],
                    &all_times[ch],
                    270.0, 70.0, 50.0) {
      Err(err) => {
        error!("Integration failed! Err {err}");
      }
      Ok(chrg)   => {
        charge = chrg;
      }
    }
    let peaks : Vec::<(usize, usize)>;
    let mut cfd_times = Vec::<f32>::new();
    match find_peaks(&all_voltages[ch]    ,
                     &all_times[ch]      ,
                     270.0, 
                     70.0 ,
                     3    ,
                     10.0 ,
                     5      ) {
      Err(err) => {
        error!("Unable to find peaks for ch {ch}! Ignoring this channel!");
        error!("We won't be able to calculate timing information for this channel! Err {err}");
      }
      Ok(pks)  => {
        peaks = pks;
        for pk in peaks.iter() {
          match cfd_simple(&all_voltages[ch],
                           &all_times[ch],
                           0.2,pk.0, pk.1) {
            Err(err) => {
              error!("Unable to calculate cfd for peak {} {}! Err {}", pk.0, pk.1, err);
            }
            Ok(cfd) => {
              cfd_times.push(cfd);
            }
          }
        }
      }
    }

    let ch_pid = readoutboard.get_pid_for_ch(ch);
    let end    = readoutboard.get_paddle_end(ch); 
    match end {
      mf::PaddleEndIdentifier::A => {
        paddles.get_mut(&ch_pid).expect("Bad paddlemap!").set_charge_a(charge);
        paddles.get_mut(&ch_pid).expect("Bad paddlemap!").set_time_a(cfd_times[0]);
      }
      mf::PaddleEndIdentifier::B => {
        paddles.get_mut(&ch_pid).expect("Bad paddlemap!").set_charge_b(charge);
        paddles.get_mut(&ch_pid).expect("Bad paddlemap!").set_time_b(cfd_times[0]);
      }
    }
  }
  let result = paddles.into_values().collect();
  Ok(result)
}

//**********************************************
//
// Subsystem communication
//


/// construct a request string which can be broadcast over 0MQ to all the boards
/// ///
/// /// Boards will only send paddle information when this request string is received
pub fn construct_event_request(rb_id : u8) -> String {
  let mut request = String::from("RB");
  if rb_id < 10 {
    request += "0";
  }
  request += &rb_id.to_string();
  request
}


/// Connect to MTB Utp socket
///
/// This will try a number of options to bind 
/// to the local port.
/// 
/// # Arguments 
///
/// * mtb_ip    : IP Adress of the MTB
/// * mtb_port  : Port of the MTB
///
pub fn connect_to_mtb(mt_address : &String) 
  ->io::Result<UdpSocket> {
  let local_port = "0.0.0.0:50100";
  let local_addrs = [
    SocketAddr::from(([0, 0, 0, 0], 50100)),
    SocketAddr::from(([0, 0, 0, 0], 50101)),
    SocketAddr::from(([0, 0, 0, 0], 50102)),
  ];
  //let local_socket = UdpSocket::bind(local_port);
  let local_socket = UdpSocket::bind(&local_addrs[..]);
  let socket : UdpSocket;
  match local_socket {
    Err(err)   => {
      error!("Can not create local UDP port for master trigger connection at {}!, err {}", local_port, err);
      return Err(err);
    }
    Ok(value)  => {
      info!("Successfully bound UDP socket for master trigger communcations to {}", local_port);
      socket = value;
      // this is not strrictly necessary, but 
      // it is nice to limit communications
      match socket.set_read_timeout(Some(Duration::from_millis(1))) {
        Err(err) => error!("Can not set read timeout for Udp socket! Error {err}"),
        Ok(_)    => ()
      }
      match socket.connect(&mt_address) {
        Err(err) => {
          error!("Can not connect to master trigger at {}, err {}", mt_address, err);
          return Err(err);
        }
        Ok(_)    => info!("Successfully connected to the master trigger at {}", mt_address)
      }
      return Ok(socket);
    }
  } // end match
}  

/// Communications with the master trigger over Udp
///
/// The master trigger can send packets over the network.
/// These packets contain timestamps as well as the 
/// eventid and a hitmaks to identify which LTBs have
/// participated in the trigger.
/// The packet format is described
/// [here](https://gitlab.com/ucla-gaps-tof/firmware/-/tree/develop/)
///
/// # Arguments
///
/// * mt_ip       : ip address of the master trigger, most likely 
///                 something like 10.0.1.10
/// * mt_port     : 
///
/// * sender_rate : 
/// 
/// * 
///
/// * verbose     : Print "heartbeat" output 
///
pub fn master_trigger(mt_ip          : &str, 
                      mt_port        : usize,
                      sender_rate    : &cbc::Sender<u32>,
                      evid_sender    : &cbc::Sender<MasterTriggerEvent>,
                      verbose        : bool) {

  let mt_address = mt_ip.to_owned() + ":" + &mt_port.to_string();
 
  let mut socket = connect_to_mtb(&mt_address).expect("Can not create local UDP socket for MTB connection!"); 
  //socket.set_nonblocking(true).unwrap();
  // we only allocate the buffer once
  // and reuse it for all operations
  let mut buffer = [0u8;MT_MAX_PACKSIZE];  
  
  //let mut event_cnt      = 0u32;
  let mut last_event_cnt = 0u32;
  let mut missing_evids  = 0usize;
  //let mut event_missing  = false;
  let mut n_events       = 0usize;
  // these are the number of expected events
  // (missing included)
  let mut n_events_expected = 0usize;
  //let mut n_paddles_expected : u32;
  let mut rate : f64;
  // for rate measurement
  let start = Instant::now();

  let mut next_beat = true;
  
  // FIXME - this is a good idea
  // limit polling rate to a maximum
  //let max_rate = 200.0; // hz
    
  // reset the master trigger before acquisiton
  info!("Resetting master trigger");
  match reset_daq(&socket, &mt_address) {
    Err(err) => error!("Can not reset DAQ, error {err}"),
    Ok(_)    => ()
  }
  // the event counter has to be reset before 
  // we connect to the readoutboards
  //reset_event_cnt(&socket, &mt_address); 
  let mut ev : MasterTriggerEvent;// = read_daq(&socket, &mt_address, &mut buffer);
  let mut timeout = Instant::now();
  //let timeout = Duration::from_secs(5);
  info!("Starting MT event loop at {:?}", timeout);
  let mut timer = Instant::now();


  loop {
    // a heartbeat every 10 s
    let elapsed = start.elapsed().as_secs();
    if (elapsed % 10 == 0) && next_beat {
      rate = n_events as f64 / elapsed as f64;
      let expected_rate = n_events_expected as f64 / elapsed as f64; 
      if verbose {
        println!("== == == == == == == == MT HEARTBEAT! {} seconds passed!", elapsed);
        println!("==> {} events recorded, trigger rate: {:.3} Hz", n_events, rate);
        println!("==> -- expected rate {:.3} Hz", expected_rate);   
        println!("== == == == == == == == END HEARTBEAT!");
      }
      next_beat = false;
    } else if elapsed % 10 != 0 {
      next_beat = true;
    }
    if timeout.elapsed().as_secs() > 10 {
      drop(socket);
      socket = connect_to_mtb(&mt_address).expect("Can not create local UDP socket for MTB connection!"); 
      timeout = Instant::now();
    }
    if timer.elapsed().as_secs() > 10 {
      match read_rate(&socket, &mt_address, &mut buffer) {
        Err(err) => {
          error!("Unable to obtain MT rate information! error {err}");
          continue;
        }
        Ok(rate) => {
          info!("Got rate from MTB {rate}");
          match sender_rate.try_send(rate) {
            Err(err) => error!("Can't send rate, error {err}"),
            Ok(_)    => ()
          }
        }
      }
      timer = Instant::now();
    }

    //info!("Next iter...");
    // limit the max polling rate
    
    //let milli_sleep = Duration::from_millis((1000.0/max_rate) as u64);
    //thread::sleep(milli_sleep);
    

    //info!("Done sleeping..."); 
    //match socket.connect(&mt_address) {
    //  Err(err) => panic!("Can not connect to master trigger at {}, err {}", mt_address, err),
    //  Ok(_)    => info!("Successfully connected to the master trigger at {}", mt_address)
    //}
    //  let received = socket.recv_from(&mut buffer);

    //  match received {
    //    Ok((size, addr)) => println!("Received {} bytes from address {}", size, addr),
    //    Err(err)         => {
    //      println!("Received nothing! err {}", err);
    //      continue;
    //    }
    //  } // end match
    
    // daq queue states
    // 0 - full
    // 1 - something
    // 2 - empty
    //if 0 != (read_register(&socket, &mt_address, 0x12, &mut buffer) & 0x2) {
    //if read_register(&socket, &mt_address, 0x12, &mut buffer) == 2 {
    //  trace!("No new information from DAQ");
    //  //reset_daq(&socket, &mt_address);  
    //  continue;
    //}
    
    //event_cnt = read_event_cnt(&socket, &mt_address, &mut buffer);
    //println!("Will read daq");
    //mt_event = read_daq(&socket, &mt_address, &mut buffer);
    //println!("Got event");
    match read_daq(&socket, &mt_address, &mut buffer) {
      Err(err) => {
        trace!("Did not get new event, Err {err}");
        continue;
      }
      Ok(new_event) => {
        ev = new_event; 
      }
    }
    if ev.event_id == last_event_cnt {
      trace!("Same event!");
      continue;
    }

    // sometimes, the counter will just read 0
    // throw these away. 
    // FIXME - there is actually an event with ctr 0
    // but not sure how to address that yet
    if ev.event_id == 0 {
      trace!("event 0 encountered! Continuing...");
      //continue;
    }

    // FIXME
    if ev.event_id == 2863311530 {
      warn!("Magic event number! continuing! 2863311530");
      //continue;
    }

    // we have a new event
    //println!("** ** evid: {}",event_cnt);
    
    // if I am correct, there won't be a counter
    // overflow for a 32bit counter in 99 days 
    // for a rate of 500Hz
    if ev.event_id < last_event_cnt {
      error!("Event counter id overflow! this cntr: {} last cntr: {last_event_cnt}!", ev.event_id);
      last_event_cnt = 0;
      continue;
    }
    
    if ev.event_id - last_event_cnt > 1 {
      let missing = ev.event_id - last_event_cnt;
      error!("We missed {missing} eventids"); 
      // FIXME
      if missing < 200 {
        missing_evids += missing as usize;
      } else {
        warn!("We missed too many event ids from the master trigger!");
        //missing = 0;
      }
      //error!("We missed {} events!", missing);
      //event_missing = true;
    }
    
    trace!("Got new event id from master trigger {}",ev.event_id);
    match evid_sender.send(ev) {
      Err(err) => trace!("Can not send event, err {err}"),
      Ok(_)    => ()
    }
    last_event_cnt = ev.event_id;
    n_events += 1;
    n_events_expected = n_events + missing_evids;

    //if n_events % 1000 == 0 {
      //let pk = TofPacket::new();
      //error!("Sending of mastertrigger packets down the global data sink not supported yet!");
    //}

    let elapsed = start.elapsed().as_secs();
    // measure rate every 100 events
    if n_events % 1000 == 0 {
      rate = n_events as f64 / elapsed as f64;
      if verbose {
        println!("==> [MASTERTRIGGER] {} events recorded, trigger rate: {:.3} Hz", n_events, rate);
      }
      rate = n_events_expected as f64 / elapsed as f64;
      if verbose {
        println!("==> -- expected rate {:.3} Hz", rate);   
      }
    } 
    // end new event
  } // end loop
}

/// Obtain monitoring data from the MTB.
///
/// # Arguments"
///
/// * mtb_address   : ip + port of the master trigger
/// * moni          : preallocated struct to hold monitoring 
///                   data
pub fn monitor_mtb(mtb_address : &String,
                   mtb_moni    : &mut MtbMoniData) {
  let socket = connect_to_mtb(&mtb_address); 
  let mut buffer = [0u8;MT_MAX_PACKSIZE];  
  match socket {
    Err(err) => {error!("Can not connect to MTB, error {err}")},
    Ok(sock) => {
      match read_rate(&sock, &mtb_address, &mut buffer) {
        Err(err) => {
          error!("Unable to obtain MT rate information! error {err}");
        }
        Ok(rate) => {
          info!("Got MTB rate of {rate}");
          mtb_moni.rate = rate as u16;
        }
      } // end match
      match read_adc_vccaux_and_vccbram(&sock, &mtb_address, &mut buffer) {
        Err(err) => {
          error!("Unable to obtain MT VCCAUX and VCCBRAM! error {err}");
        }
        Ok(values) => {
          mtb_moni.fpga_vccaux  = values.0;
          mtb_moni.fpga_vccbram = values.1; 
        }
      }
      match read_adc_temp_and_vccint(&sock, &mtb_address, &mut buffer) {
        Err(err) => {
          error!("Unable to obtain MT VCCAUX and VCCBRAM! error {err}");
        }
        Ok(values) => {
          mtb_moni.fpga_temp    = values.0;
          mtb_moni.fpga_vccint  = values.1; 
        }
      }
    } // end OK
  } // end match
}


/// Get the tof channel/paddle mapping and involved components
///
/// This reads the configuration from a json file and panics 
/// if there are any problems.
///
#[deprecated(since="0.4.0", note="please use the database methods from tof_dataclasses instead")]
pub fn get_tof_manifest(json_config : PathBuf) -> (Vec::<LocalTriggerBoard>, Vec::<ReadoutBoard>) {
  let mut ltbs = Vec::<LocalTriggerBoard>::new();
  let mut rbs  = Vec::<ReadoutBoard>::new();
  let js_file = json_config.as_path();
   if !js_file.exists() {
     panic!("The file {} does not exist!", js_file.display());
   }
   info!("Found config file {}", js_file.display());
   let json_content = std::fs::read_to_string(js_file).expect("Unable to read file!");
   let config = json::parse(&json_content).expect("Unable to parse json!");
   for n in 0..config["ltbs"].len() {
     ltbs.push(LocalTriggerBoard::from(&config["ltbs"][n]));
   }
   for n in 0..config["rbs"].len() {
     rbs.push(ReadoutBoard::from(&config["rbs"][n]));
   }
  (ltbs, rbs)
}


#[deprecated(since="0.1.0", note="please use `get_tof_manifest` instead")]
pub fn get_rb_manifest() -> Vec<ReadoutBoard> {
  let rb_manifest_path = path!("assets/rb.manifest");
  let mut connected_boards = Vec::<ReadoutBoard>::new();
  let mac_table = get_mac_to_ip_map();
  if let Ok(lines) = read_lines(rb_manifest_path) {
    // Consumes the iterator, returns an (Optional) String
    for line in lines {
      if let Ok(ip) = line {
        if ip.starts_with("#") {
          continue;
        }
        if ip.len() == 0 {
          continue;
        }
        let identifier: Vec<&str> = ip.split(";").collect();
        debug!("{:?}", identifier);
        let mut rb = ReadoutBoard::new();
        let mc_address = identifier[1].replace(" ","");
        let mc_address : Vec<&str> = mc_address.split(":").collect();
        println!("{:?}", mc_address);
        let mc_address : Vec<u8>   = mc_address.iter().map(|&x| {u8::from_str_radix(x,16).unwrap()} ).collect();
        assert!(mc_address.len() == 6);
        let mac = MacAddr6::new(mc_address[0],
                                mc_address[1],
                                mc_address[2],
                                mc_address[3],
                                mc_address[4],
                                mc_address[5]);

        rb.id          = Some(identifier[0].parse::<u8>().expect("Invalid RB ID!"));
        rb.mac_address = Some(mac);
        let rb_ip = mac_table.get(&mac);
        println!("Found ip address {:?}", rb_ip);
        match rb_ip {
          None => println!("Can not resolve RBBoard with MAC address {:?}, it is not in the system's ARP tables", mac),
          Some(ip)   => match ip[0] {
            IpAddr::V6(a) => panic!("IPV6 {a} not suppported!"),
            IpAddr::V4(a) => {
              rb.ip_address = Some(a);
              rb.data_port  = Some(42000);
              connected_boards.push(rb);
              // now we will try and check if the ports are open
              //let mut all_data_ports = Vec::<String>::new();//scan_ports_range(30000..39999);
              //let mut all_cmd_ports  = Vec::<String>::new();//scan_ports_range(40000..49999);
              //// FIXME - the ranges here are somewhat arbitrary
              //for n in 30000..39999 {
              //  all_data_ports.push(rb.ip_address.unwrap().to_string() + ":" + &n.to_string());
              //  //scan_ports_addrs(
              //}
              //for n in 40000..49999 {
              //  all_cmd_ports.push(rb.ip_address.unwrap().to_string() + ":" + &n.to_string());
              //}
              //let open_data_ports = scan_ports_addrs(all_data_ports);
              //let open_cmd_ports  = scan_ports_addrs(all_cmd_ports);
              //assert!(open_cmd_ports.len() < 2);
              //assert!(open_data_ports.len() < 2);
              //if open_cmd_ports.len() == 1 {
              //  rb.cmd_port = Some(open_cmd_ports[0].port());
              //  match rb.ping() {
              //    Ok(_)    => println!("... connected!"),
              //    Err(err) => println!("Can't connect to RB, err {err}"),
              //  }
              //} else {
              //  rb.cmd_port = None;
              //}
              //

              //println!("Found open data ports {:?}", open_data_ports);
              //if open_data_ports.len() == 1 {
              //  rb.data_port = Some(open_data_ports[0].port());
              //} else {
              //  rb.data_port = None;
              //}
              //if rb.is_connected {
              //  connected_boards.push(rb);
              //}
            }
          }
        }

        
        println!("{:?}", connected_boards);
      }
    }
  }
  return connected_boards;
}



#[derive(Debug)]
pub enum ReadoutBoardError {
  NoConnectionInfo,
  NoResponse,
}


impl fmt::Display for ReadoutBoardError{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp : String;
    match self {
      ReadoutBoardError::NoConnectionInfo => {disp = String::from("NoConnectionInfo");},
      ReadoutBoardError::NoResponse       => {disp = String::from("NoResponse");},
    } 
    write!(f, "<ReadoutBoardError : {}>", disp)
  }
}

impl Error for ReadoutBoardError {
}

/// Find boards in the network
///
///
///
//pub fn discover_boards() -> Vec<ReadoutBoard> {
//  let board_list = Vec::<ReadoutBoard>::new();
//  board_list
//}


/// A generic representation of a LocalTriggerBoard
///
/// This is important to make the mapping between 
/// trigger information and readoutboard.
#[derive(Debug, Clone)]
pub struct LocalTriggerBoard {
  pub id : u8,
  /// The LTB has 16 channels, 
  /// which are connected to the RBs
  /// Each channel corresponds to a 
  /// specific RB channel, represented
  /// by the tuple (RBID, CHANNELID)
  pub ch_to_rb : [(u8,u8);16],
  /// the MTB bit in the MTEvent this 
  /// LTB should reply to
  pub mt_bitmask : u32,
}

impl LocalTriggerBoard {
  pub fn new() -> LocalTriggerBoard {
    LocalTriggerBoard {
      id : 0,
      ch_to_rb : [(0,0);16],
      mt_bitmask : 0
    }
  }

  /// Calculate the position in the bitmask from the connectors
  pub fn get_mask_from_dsi_and_j(dsi : u8, j : u8) -> u32 {
    if dsi == 0 || j == 0 {
      warn!("Invalid dsi/J connection!");
      return 0;
    }
    let mut mask : u32 = 1;
    mask = mask << (dsi*5 + j -1) ;
    mask
  }
}

impl fmt::Display for LocalTriggerBoard {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<LTB: \n ID \t\t: {} \n bitmask \t\t: {} \n channels \t: {:?} >", 
            self.id.to_string() ,
            self.mt_bitmask.to_string(),
            self.ch_to_rb
    )
  }
}

impl From<&json::JsonValue> for LocalTriggerBoard {
  fn from(json : &json::JsonValue) -> Self {
    let id  = json["id"].as_u8().expect("id value json problem");
    let dsi = json["DSI"].as_u8().expect("DSI value json problem");
    let j   = json["J"].as_u8().expect("J value json problem");
    //let mask = LocalTriggerBoard::get_mask_from_dsi_and_j(dsi, j);
    let channels = &json["ch_to_rb"];//.members();
    let mut rb_channels = [(0, 0);16];
    for ch in 0..channels.len() {
      if channels.has_key(&ch.to_string()) {
        rb_channels[ch] = (channels[&ch.to_string()][0].as_u8().unwrap(),
                           channels[&ch.to_string()][1].as_u8().unwrap());  
      }
    }
    let bitmask = LocalTriggerBoard::get_mask_from_dsi_and_j(dsi, j);
    LocalTriggerBoard {
      id : id,
      ch_to_rb : rb_channels,
      mt_bitmask : bitmask
    }
  }
}

/// A generic representation of a Readout board
///
///
///
#[derive(Debug, Clone)]
pub struct ReadoutBoard {
  pub id           : Option<u8>,
  pub mac_address  : Option<MacAddr6>,
  pub ip_address   : Option<Ipv4Addr>, 
  pub data_port    : Option<u16>,
  pub cmd_port     : Option<u16>,
  pub is_connected : bool,
  pub uptime       : u32,
  pub ch_to_pid    : [u8;8],
  pub sorted_pids  : [u8;4],
  pub calib_file   : String,
  pub configured   : bool,
}

impl ReadoutBoard {

  pub fn new() -> ReadoutBoard {
    ReadoutBoard {
      id            : None,
      mac_address   : None,
      ip_address    : None,
      data_port     : None,
      cmd_port      : None,
      is_connected  : false,
      uptime        : 0,
      ch_to_pid     : [0;8],
      sorted_pids   : [0;4], 
      calib_file    : String::from(""),
      configured    : false
    }
  }

  pub fn get_connection_string(&mut self) -> String {
    if !self.configured {
      panic!("Can not get connection string. This board has not been configured. Get the information from corresponding json tof manifest");
    }

    self.get_ip();
    let mut address_ip = String::from("tcp://");
    match self.ip_address {
      None => panic!("This board does not have an ip address. Unable to obtain connection information"),
      Some(ip) => {
        address_ip = address_ip + &ip.to_string();
      }
    }
    match self.data_port {
      None => panic!("This board does not have a known data port. Typically, this should be 42000. Please check your tof-manifest.jsdon"),
      Some(port) => {
        address_ip += &":".to_owned();
        address_ip += &port.to_string();
      }
    }
    address_ip
  }

  /// Get the readoutboard ip address from 
  /// the ARP tables
  pub fn get_ip(&mut self) {
    let mac_table = get_mac_to_ip_map();
    let rb_ip = mac_table.get(&self.mac_address.unwrap());
    info!("Found ip address {:?} for RB {}", rb_ip, self.id.unwrap_or(0));
    match rb_ip {
      None => panic!("Can not resolve RBBoard with MAC address {:?}, it is not in the system's ARP tables", &self.mac_address),
      Some(ip)   => match ip[0] {
        IpAddr::V6(a) => panic!("IPV6 {a} not suppported!"),
        IpAddr::V4(a) => {
          self.ip_address = Some(a); 
        }
      }
    }
  }
    
  ///// Ping it  
  //pub fn ping(&mut self) -> Result<(), Box<dyn Error>> { 
  //  // connect to the command port and send a ping
  //  // message
  //  let ctx =  zmq::Context::new();
  //  if matches!(self.ip_address, None) || matches!(self.cmd_port, None) {
  //    self.is_connected = false;
  //    return Err(Box::new(ReadoutBoardError::NoConnectionInfo));
  //  }
  //  let address = "tcp://".to_owned() + &self.ip_address.unwrap().to_string() + ":" + &self.cmd_port.unwrap().to_string(); 
  //  let socket  = ctx.socket(zmq::REQ)?;
  //  socket.connect(&address)?;
  //  info!("Have connected to adress {address}");
  //  // if the readoutboard is there, it should send *something* back
  //  let p = TofCommand::Ping(1);

  //  socket.send(p.to_bytestream(), 0)?;
  //  info!("Sent ping signal, waiting for response!");
  //  let data = socket.recv_bytes(0)?;
  //  if data.len() != 0 {
  //    self.is_connected = true;
  //    return Ok(());
  //  }
  //  self.is_connected = false;
  //  return Err(Box::new(ReadoutBoardError::NoResponse));
  //}
}

impl fmt::Display for ReadoutBoard {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let default_ip  = Ipv4Addr::new(0,0,0,0);
    let default_mac = MacAddr6::default();
    write!(f, "<ReadoutBoard: \n ID \t\t: {} \n MAC addr \t: {} \n IP addr \t: {} \n 0MQ PUB \t: {} \n 0MQ REP \t: {} \n connected \t: {}\n calib file \t: {} \n uptime \t: {} >", 
            self.id.unwrap_or(0).to_string()           ,      
            self.mac_address.unwrap_or(default_mac).to_string()  ,
            self.ip_address.unwrap_or(default_ip).to_string()   ,
            self.data_port.unwrap_or(0).to_string()    ,
            self.cmd_port.unwrap_or(0)     , 
            self.is_connected.to_string() , 
            "?",
            //&self.calib_file.unwrap_or(String::from("")),
            self.uptime.to_string()       ,
    )
  }
}

impl Default for ReadoutBoard {
  fn default() -> ReadoutBoard {
    ReadoutBoard::new()
  }
}

impl From<&json::JsonValue> for ReadoutBoard {
  fn from(json : &json::JsonValue) -> Self {
    let mut board =  ReadoutBoard::new();
    board.id = Some(json["id"].as_u8().unwrap());
    //let identifier: Vec<&str> = ip.split(";").collect();
    let identifier = json["mac_address"].as_str().unwrap();
    let mc_address = identifier.replace(" ","");
    let mc_address : Vec<&str> = mc_address.split(":").collect();
    println!("{:?}", mc_address);
    let mc_address : Vec<u8>   = mc_address.iter().map(|&x| {u8::from_str_radix(x,16).unwrap()} ).collect();
    assert!(mc_address.len() == 6);
    let mac = MacAddr6::new(mc_address[0],
                            mc_address[1],
                            mc_address[2],
                            mc_address[3],
                            mc_address[4],
                            mc_address[5]);
    let data_port = Some(json["port"].as_u16().unwrap());
    let calib_file = json["calibration_file"].as_str().unwrap();
    board.mac_address = Some(mac);
    board.data_port   = data_port;
    board.calib_file  = calib_file.to_string();
    board.get_ip();
    let ch_to_pid = &json["ch_to_pid"];
    let mut ch_true : usize;
    for ch in 0..ch_to_pid.len() {
      ch_true = ch + 1;
      //println!("{ch}");
      //println!("{:?}", json["ch_to_pid"]);
      match json["ch_to_pid"][&ch_true.to_string()].as_u8() {
        Some(foo) => {board.ch_to_pid[ch] = foo;}
        None => {
          error!("Can not get data for ch {ch}");
          board.ch_to_pid[ch] = 0;
        }
      }
      //board.ch_to_pid[ch] = json["ch_to_pid"][&ch_true.to_string()].as_u8().unwrap();
    }
    let mut paddle_ids : [u8;4] = [0,0,0,0];
    let mut counter = 0;
    for ch in board.ch_to_pid.iter().step_by(2) {
      paddle_ids[counter] = *ch;
      counter += 1;
    }
    board.sorted_pids = paddle_ids;
    board.configured  = true;
    board
  }
}

#[test]
fn test_display() {
  let rb = ReadoutBoard::default();
  println!("Readout board {}", rb);
  assert_eq!(1,1);
}


#[test]
fn show_manifest() {
  get_rb_manifest();
  assert_eq!(1,1);
}
