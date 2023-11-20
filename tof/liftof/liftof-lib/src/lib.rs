pub mod master_trigger;
use constants::{DEFAULT_CALIB_VOLTAGE,
                DEFAULT_CALIB_EXTRA,
                DEFAULT_RB_ID,
                DEFAULT_PB_ID,
                DEFAULT_LTB_ID,
                DEFAULT_PREAMP_ID,
                DEFAULT_PREAMP_BIAS,
                DEFAULT_POWER_STATUS,
                DEFAULT_RUN_TYPE,
                DEFAULT_RUN_EVENT_NO,
                DEFAULT_RUN_TIME};
pub use master_trigger::{connect_to_mtb,
                         master_trigger};
pub mod constants;

use std::error::Error;
use std::fmt;
use std::{fs::File, path::Path};
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::fs::read_to_string;
use std::io::{self, BufReader};
use std::io::{Read,
              Write,
              Seek,
              SeekFrom};
use std::collections::HashMap;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use crossbeam_channel::Receiver;
use zmq;
use colored::{Colorize, ColoredString};

use serde_json::Value;

use log::Level;
use macaddr::MacAddr6;
use netneighbours::get_mac_to_ip_map;

extern crate indicatif;
use indicatif::{ProgressBar, ProgressStyle};

//extern crate pretty_env_logger;
#[macro_use] extern crate log;

use tof_dataclasses::manifest as mf;
use tof_dataclasses::DsiLtbRBMapping;
use tof_dataclasses::constants::NWORDS;
use tof_dataclasses::calibrations::RBCalibrations;
use tof_dataclasses::packets::{TofPacket,
                               PacketType};
use tof_dataclasses::errors::{SerializationError,
                              AnalysisError};
use tof_dataclasses::serialization::{search_for_u16,
                                     parse_u8,
                                     parse_u32,
                                     Serialization};
use tof_dataclasses::commands::RBCommand;
use tof_dataclasses::events::{RBEvent,
                              TofHit};

use tof_dataclasses::analysis::{calculate_pedestal,
                                integrate,
                                cfd_simple,
                                find_peaks};

use clap::{arg,
  //value_parser,
  //ArgAction,
  //Command,
  Parser,
  Args,
  Subcommand};

pub const MT_MAX_PACKSIZE   : usize = 512;
pub const DATAPORT : u32 = 42000;


/// Make sure that the loglevel is in color, even though not using pretty_env logger
pub fn color_log(level : &Level) -> ColoredString {
  match level {
    Level::Error    => String::from(" ERROR!").red(),
    Level::Warn     => String::from(" WARN  ").yellow(),
    Level::Info     => String::from(" Info  ").green(),
    Level::Debug    => String::from(" debug ").blue(),
    Level::Trace    => String::from(" trace ").cyan(),
  }
}


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
    //let filename = file_prefix.clone() + "_0.tof.gaps";
    let filename = file_prefix.clone() + ".tof.gaps";
    let path = Path::new(&filename); 
    info!("Writing to file {filename}");
    let file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
    Self {
      file,
      file_prefix   : file_prefix,
      pkts_per_file : 3000,
      file_id       : 1,
      n_packets     : 0,
    }
  }

  pub fn add_tof_packet(&mut self, packet : &TofPacket) {
    let buffer = packet.to_bytestream();
    match self.file.write_all(buffer.as_slice()) {
      Err(err) => error!("Writing to file with prefix {} failed. Err {}", self.file_prefix, err),
      Ok(_)    => ()
    }
    // FIXME - this must go into the drop method
    match self.file.sync_all() {
      Err(err) => error!("File syncing failed! error {err}"),
      Ok(_)    => ()
    }
    self.n_packets += 1;
    if self.n_packets == self.pkts_per_file {
      //drop(self.file);
      let filename = self.file_prefix.clone() + "_" + &self.file_id.to_string() + ".tof.gaps";
      let path  = Path::new(&filename);
      println!("==> [TOFPACKETWRITER] Will start a new file {}", path.display());
      self.file.sync_all();
      self.file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
      self.n_packets = 0;
      self.file_id += 1;
    }
  debug!("TofPacket written!");
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
    let ptype = ptype_int as u8;
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

/// Helper function to generate a proper tcp string starting
/// from the ip one.
pub fn build_tcp_from_ip(ip: String, port: String) -> String {
  String::from("tcp://") + &ip + ":" + &port
}

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
pub fn readoutboard_commander(cmd : &Receiver<TofPacket>){
  debug!(".. started!");
  let ctx = zmq::Context::new();
  //let this_board_ip = local_ip().expect("Unable to obtainl local board IP. Something is messed up!");
  let this_board_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 1, 1));

  let address_ip;
  match this_board_ip {
    IpAddr::V4(ip) => address_ip = ip.to_string().clone(),
    IpAddr::V6(_) => panic!("Currently, we do not support IPV6!")
  }
  let data_address : String = build_tcp_from_ip(address_ip,DATAPORT.to_string());
  let data_socket = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  data_socket.bind(&data_address).expect("Unable to bind to data (PUB) socket {data_adress}");
  println!("==> 0MQ PUB socket bound to address {data_address}");
  loop {
    // check if we get a command from the main 
    // thread
    match cmd.try_recv() {
      Err(err) => trace!("Did not receive a new command, error {err}"),
      Ok(packet) => {
        // now we have several options
        match packet.packet_type {
          PacketType::TofCommand => {
            info!("Received TofCommand! Broadcasting to all TOF entities who are listening!");
            let mut payload  = String::from("BRCT").into_bytes();
            payload.append(&mut packet.to_bytestream());
            match data_socket.send(&payload,0) {
              Err(err) => error!("Unable to send command! Error {err}"),
              Ok(_)    => info!("BRCT command sent!")
            }
          },
          PacketType::RBCommand => {
            debug!("Received RBCommand!");
            let mut payload_str  = String::from("RB");
            match RBCommand::from_bytestream(&packet.payload, &mut 0) {
              Ok(rb_cmd) => {
                let to_rb_id = rb_cmd.rb_id;
                if rb_cmd.rb_id < 10 {
                  payload_str += &String::from("0");
                  payload_str += &to_rb_id.to_string();
                } else {
                  payload_str += &to_rb_id.to_string();
                }

                let mut payload = payload_str.into_bytes();
                payload.append(&mut packet.to_bytestream());
                match data_socket.send(&payload,0) {
                  Err(err) => error!("Unable to send command {}! Error {err}", rb_cmd),
                  Ok(_)    => debug!("Making event request! {}", rb_cmd)
                }
              }
              Err(err) => {
                error!("Can not construct RBCommand, error {err}");
              }
            }
          },
          _ => {
            error!("Received garbage package! {}", packet);
          }
        }// end match
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
/// algorithm settings, which we can load from a 
/// json file
/// FIXME - add the paddle packets to the RBEvent instead 
/// of returning 
pub fn waveform_analysis(event         : &mut RBEvent,
                         readoutboard  : &mf::ReadoutBoard,
                         calibration   : &RBCalibrations)
-> Result<(), AnalysisError> {
  //if event.status != EventStatus::Perfect {
  //if event.header.broken {
  //  // just return the analysis error, there 
  //  // is probably nothing else we can do?
  //  return Err(AnalysisError::InputBroken);
  //}
  let pids = readoutboard.get_all_pids();
  let mut paddles = HashMap::<u8, TofHit>::new();
  // just paranoid
  if pids.len() != 4 {
    error!("RB {} seems to have a strange number of paddles ({}) connected!",
           readoutboard.rb_id, pids.len());
  }
  for k in pids.iter() {
    // fill the general information of 
    // the paddles already
    let mut hit   = TofHit::new();
    hit.paddle_id = *k;
    match paddles.insert(*k, hit) {
      None => (),
      Some(_) => {
        error!("We have seen paddle id {k} already!");
      }
    };
  }
  // do the calibration
  //let mut active_channels = event.header.get_active_data_channels();
  //active_channels.push(9); // always do ch9 callibration
  //let mut active_channels = event.header.decode_channel_mask();
  let mut active_channels = event.header.get_channels();
  //if event.header.has_ch9 {
  //  active_channels.push(8); 
  //}
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
    let ch  = *active_ch as usize;
    match event.get_channel_by_id(ch) {
      Ok(adc) => { 
        calibration.voltages(ch,
                             event.header.stop_cell as usize,
                             &adc,
                             &mut all_voltages[ch]);
      },
      Err(err) => {
        error!("Can not get channel {ch}. Err {err}");  
        return Err(AnalysisError::MissingChannel);
      }
    }
    calibration.nanoseconds(ch,
                            event.header.stop_cell as usize,
                            &mut all_times[ch]);
  }
  match RBCalibrations::spike_cleaning(&mut all_voltages,
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
    //FIXME : is it ok to panic here?
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
  event.hits = result;
  Ok(())
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum ReadoutBoardError {
  NoConnectionInfo,
  NoResponse,
}

impl fmt::Display for ReadoutBoardError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this ReadoutBoardError"));
    write!(f, "<ReadoutBoardError: {}>", r)
  }
}

impl Error for ReadoutBoardError {
}



/// A generic representation of a LocalTriggerBoard
///
/// This is important to make the mapping between 
/// trigger information and readoutboard.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
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
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this LTB"));
    write!(f, "<LTB: {}>", r)
  }
}

/// A generic representation of a Readout board
///
///
///
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
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
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this ReadoutBoard"));
    write!(f, "<ReadoutBoard: {}>", r)
  }
}

impl Default for ReadoutBoard {
  fn default() -> ReadoutBoard {
    ReadoutBoard::new()
  }
}

pub fn get_ltb_dsi_j_ch_mapping(mapping_file : PathBuf) -> DsiLtbRBMapping {
  let mut mapping = HashMap::<u8,HashMap::<u8,HashMap::<u8,(u8,u8)>>>::new();
  for dsi in 1..6 {
    mapping.insert(dsi, HashMap::<u8,HashMap::<u8, (u8, u8)>>::new());
    for j in 1..6 {
      mapping.get_mut(&dsi).unwrap().insert(j, HashMap::<u8,(u8, u8)>::new());
      for ch in 1..17 {
        mapping.get_mut(&dsi).unwrap().get_mut(&j).unwrap().insert(ch, (0,0));
      }
    }
  }
  let json_content : String;
  match read_to_string(&mapping_file) {
    Ok(_json_content) => {
      json_content = _json_content;
    },
    Err(err) => { 
      error!("Unable to parse json file {}. Error {err}", mapping_file.display());
      return mapping;
    }      
  }
  let json : Value;
  match serde_json::from_str(&json_content) {
    Ok(_json) => {
      json = _json;
    },
    Err(err) => { 
      error!("Unable to parse json file {}. Error {err}", mapping_file.display());
      return mapping;
    }
  }
  for dsi in 1..6 { 
    for j in 1..6 {
      for ch in 1..17 {
        let val = mapping.get_mut(&dsi).unwrap().get_mut(&j).unwrap().get_mut(&ch).unwrap();
        //println!("Checking {} {} {}", dsi, j, ch);
        let tmp_val = &json[dsi.to_string()][j.to_string()][ch.to_string()];
        *val = (tmp_val[0].to_string().parse::<u8>().unwrap_or(0), tmp_val[1].to_string().parse::<u8>().unwrap_or(0));
      }
    }
  }
  debug!("Mapping {:?}", mapping);
  mapping
}

/// Convert an int value to the board ID string.
pub fn to_board_id_string(rb_id: u32) -> String {
  String::from("RB") + &format!("{:02}", rb_id)
}

/**********************************************************/
/// Command Enums and stucts
#[derive(Debug, Parser, PartialEq)]
pub enum Command {
  /// Power control of TOF sub-systems.
  #[command(subcommand)]
  Power(PowerCmd),
  /// Remotely trigger the readoutboards to run the calibration routines (tcal, vcal).
  #[command(subcommand)]
  Calibration(CalibrationCmd),
  /// Start/stop data taking run.
  #[command(subcommand)]
  Run(RunCmd)
}

/// Calibration cmds ====================================================
#[derive(Debug, Subcommand, PartialEq)]
pub enum CalibrationCmd {
  /// Default calibration run, meaning 2 voltage calibrations and one timing calibration on all RBs with the default values.
  Default(DefaultOpts),
  /// No input data taking run. All RB are targeted are default ones if nothing else is specified.
  Noi(NoiOpts),
  /// Voltage data taking run. All RB are targeted and voltage are default ones if nothing else is specified.
  Voltage(VoltageOpts),
  /// Timing data taking run. All RB are targeted and voltage are default ones if nothing else is specified.
  Timing(TimingOpts)
}

#[derive(Debug, Args, PartialEq)]
pub struct DefaultOpts {
  /// Voltage level to be set in voltage calibration run.
  #[arg(short, long, default_value_t = DEFAULT_CALIB_VOLTAGE)]
  pub voltage_level: u16,
  /// RB to target in voltage calibration run.
  #[arg(short, long, default_value_t = DEFAULT_RB_ID)]
  pub rb_id: u8,
  /// Extra arguments in voltage calibration run (not implemented).
  #[arg(short, long, default_value_t = DEFAULT_CALIB_EXTRA)]
  pub extra: u8,
}

impl DefaultOpts {
  pub fn new(voltage_level: u16, rb_id: u8, extra: u8) -> Self {
    Self { 
      voltage_level,
      rb_id,
      extra
    }
  }
}

#[derive(Debug, Args, PartialEq)]
pub struct NoiOpts {
  /// RB to target in timing calibration run.
  #[arg(short, long, default_value_t = DEFAULT_RB_ID)]
  pub rb_id: u8,
  /// Extra arguments in timing calibration run (not implemented).
  #[arg(short, long, default_value_t = DEFAULT_CALIB_EXTRA)]
  pub extra: u8,
}

impl NoiOpts {
  pub fn new(rb_id: u8, extra: u8) -> Self {
    Self { 
      rb_id,
      extra
    }
  }
}

#[derive(Debug, Args, PartialEq)]
pub struct VoltageOpts {
  /// Voltage level to be set in voltage calibration run.
  #[arg(short, long, default_value_t = DEFAULT_CALIB_VOLTAGE)]
  pub voltage_level: u16,
  /// RB to target in voltage calibration run.
  #[arg(short, long, default_value_t = DEFAULT_RB_ID)]
  pub rb_id: u8,
  /// Extra arguments in voltage calibration run (not implemented).
  #[arg(short, long, default_value_t = DEFAULT_CALIB_EXTRA)]
  pub extra: u8,
}

impl VoltageOpts {
  pub fn new(voltage_level: u16, rb_id: u8, extra: u8) -> Self {
    Self { 
      voltage_level,
      rb_id,
      extra
    }
  }
}

#[derive(Debug, Args, PartialEq)]
pub struct TimingOpts {
  /// Voltage level to be set in voltage calibration run.
  #[arg(short, long, default_value_t = DEFAULT_CALIB_VOLTAGE)]
  pub voltage_level: u16,
  /// RB to target in voltage calibration run.
  #[arg(short, long, default_value_t = DEFAULT_RB_ID)]
  pub rb_id: u8,
  /// Extra arguments in voltage calibration run (not implemented).
  #[arg(short, long, default_value_t = DEFAULT_CALIB_EXTRA)]
  pub extra: u8,
}

impl TimingOpts {
  pub fn new(voltage_level: u16, rb_id: u8, extra: u8) -> Self {
    Self { 
      voltage_level,
      rb_id,
      extra
    }
  }
}
/// END Calibration cmds ================================================

/// Power cmds ====================================================
#[derive(Debug, Subcommand, PartialEq)]
pub enum PowerCmd {
  /// Power up everything (PB + RB + LTB + preamps + MT)
  All(PowerStatus),
  /// Power up MT alone
  MT(PowerStatus),
  /// Power up everything but MT (PB + RB + LTB + preamps)
  AllButMT(PowerStatus),
  /// Power up all or specific PBs
  PB(PBPowerOpts),
  /// Power up all or specific RBs
  RB(RBPowerOpts),
  /// Power up all or specific LTBs
  LTB(LTBPowerOpts),
  /// Power up all or specific preamp
  Preamp(PreampPowerOpts)
}

#[derive(Debug, Args, PartialEq)]
pub struct PowerStatus {
  /// Which power status one wants to achieve
  pub power_status: PowerStatusEnum
}

impl PowerStatus {
  pub fn new(power_status: PowerStatusEnum) -> Self {
    Self { 
      power_status
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum TofComponent {
  /// everything (PB + RB + LTB + preamps + MT)
  All       = 0u8,
  /// MT alone
  MT        = 10u8,
  /// everything but MT (PB + RB + LTB + preamps)
  AllButMT  = 20u8,
  /// all or specific PBs
  PB        = 30u8,
  /// all or specific RBs
  RB        = 40u8,
  /// all or specific LTBs
  LTB       = 50u8,
  /// all or specific preamp
  Preamp    = 60u8
}

impl fmt::Display for TofComponent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this TofComponent"));
    write!(f, "<TofComponent: {}>", r)
  }
}

impl TryFrom<u8> for TofComponent {
  type Error = &'static str;

  // I am not sure about this hard coding, but the code
  //  looks nicer - Paolo
  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0u8  => Ok(TofComponent::All),
      10u8 => Ok(TofComponent::MT),
      20u8 => Ok(TofComponent::AllButMT),
      30u8 => Ok(TofComponent::PB),
      40u8 => Ok(TofComponent::RB),
      50u8 => Ok(TofComponent::LTB),
      60u8 => Ok(TofComponent::Preamp),
      _    => Err("I am not sure how to convert this value!")
    }
  }
}

// repr is u16 in order to leave room for preamp bias
#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize, clap::ValueEnum)]
#[repr(u16)]
pub enum PowerStatusEnum {
  OFF       = 0u16,
  ON        = 10u16,
  Cycle     = 20u16,
}

impl fmt::Display for PowerStatusEnum {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this PowerStatusEnum"));
    write!(f, "<PowerStatusEnum: {}>", r)
  }
}

impl TryFrom<u16> for PowerStatusEnum {
  type Error = &'static str;

  // I am not sure about this hard coding, but the code
  //  looks nicer - Paolo
  fn try_from(value: u16) -> Result<Self, Self::Error> {
    match value {
      0u16  => Ok(PowerStatusEnum::OFF),
      10u16 => Ok(PowerStatusEnum::ON),
      20u16 => Ok(PowerStatusEnum::Cycle),
      _    => Err("I am not sure how to convert this value!")
    }
  }
}

#[derive(Debug, Args, PartialEq)]
pub struct PBPowerOpts {
  /// Which power status one wants to achieve
  pub power_status: PowerStatusEnum,
  /// ID of the PB to be powered up
  #[arg(short, long, default_value_t = DEFAULT_PB_ID)]
  pub pb_id: u8
}

impl PBPowerOpts {
  pub fn new(power_status: PowerStatusEnum, pb_id: u8) -> Self {
    Self { 
      power_status,
      pb_id
    }
  }
}

#[derive(Debug, Args, PartialEq)]
pub struct RBPowerOpts {
  /// Which power status one wants to achieve
  pub power_status: PowerStatusEnum,
  /// ID of the RB to be powered up
  #[arg(short, long, default_value_t = DEFAULT_RB_ID)]
  pub rb_id: u8
}

impl RBPowerOpts {
  pub fn new(power_status: PowerStatusEnum, rb_id: u8) -> Self {
    Self {
      power_status,
      rb_id
    }
  }
}

#[derive(Debug, Args, PartialEq)]
pub struct LTBPowerOpts {
  /// Which power status one wants to achieve
  pub power_status: PowerStatusEnum,
  /// ID of the LTB to be powered up
  #[arg(short, long, default_value_t = DEFAULT_LTB_ID)]
  pub ltb_id: u8
}

impl LTBPowerOpts {
  pub fn new(power_status: PowerStatusEnum, ltb_id: u8) -> Self {
    Self {
      power_status,
      ltb_id
    }
  }
}

#[derive(Debug, Args, PartialEq)]
pub struct PreampPowerOpts {
  /// Which power status one wants to achieve
  pub power_status: PowerStatusEnum,
  /// ID of the preamp to be powered up
  #[arg(short, long, default_value_t = DEFAULT_PREAMP_ID)]
  pub preamp_id: u8,
  /// Turn on bias of the preamp specified
  #[arg(short, long, default_value_t = DEFAULT_PREAMP_BIAS)]
  pub preamp_bias: u16
}

impl PreampPowerOpts {
  pub fn new(power_status: PowerStatusEnum, preamp_id: u8, preamp_bias: u16) -> Self {
    Self {
      power_status,
      preamp_id,
      preamp_bias
    }
  }
}
/// END Power cmds ================================================

/// Run cmds ======================================================
#[derive(Debug, Subcommand, PartialEq)]
pub enum RunCmd {
  /// Start data taking
  Start(StartRunOpts),
  /// Stop data taking
  Stop(StopRunOpts)
}

#[derive(Debug, Args, PartialEq)]
pub struct StartRunOpts {
  /// Which kind of run is to be launched
  #[arg(long, default_value_t = DEFAULT_RUN_TYPE)]
  pub run_type: u8,
  /// ID of the RB where to run data taking
  #[arg(long, default_value_t = DEFAULT_RB_ID)]
  pub rb_id: u8,
  /// Number of events to be generated
  #[arg(short, long, default_value_t = DEFAULT_RUN_EVENT_NO)]
  pub event_no: u8,
  /// Time the run is expected to go on for
  #[arg(short, long, default_value_t = DEFAULT_RUN_TIME)]
  pub time: u8
}

impl StartRunOpts {
  pub fn new(run_type: u8, rb_id: u8, event_no: u8, time: u8) -> Self {
    Self {
      run_type,
      rb_id,
      event_no,
      time
    }
  }
}

#[derive(Debug, Args, PartialEq)]
pub struct StopRunOpts {
  /// ID of the RB where to run data taking
  #[arg(short, long, default_value_t = DEFAULT_RB_ID)]
  pub rb_id: u8
}

impl StopRunOpts {
  pub fn new(rb_id: u8) -> Self {
    Self {
      rb_id
    }
  }
}
/// END Run cmds ==================================================

#[test]
fn test_display() {
  let rb = ReadoutBoard::default();
  println!("Readout board {}", rb);
  assert_eq!(1,1);
}


