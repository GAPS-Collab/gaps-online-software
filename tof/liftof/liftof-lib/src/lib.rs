//pub mod misc;

use port_scanner::scan_ports_addrs;

use std::error::Error;
use std::time::{Duration, Instant};
use std::fmt;
use std::path::PathBuf;
use std::net::{IpAddr, Ipv4Addr};
use std::io::Write;
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};

use tof_dataclasses::events::master_trigger::{read_daq, read_rate, reset_daq};

// FIXME - remove this crate
//use mac_address::MacAddress;
use zmq;

extern crate json;

use macaddr::MacAddr6;
use netneighbours::get_mac_to_ip_map;
use crossbeam_channel as cbc; 

use std::fs::File;
use std::fs::OpenOptions;

use std::io::{self, BufRead};
use std::path::Path;

use tof_dataclasses::commands::{TofCommand, TofResponse};
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::events::MasterTriggerEvent;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

#[macro_use] extern crate manifest_dir_macros;

const MT_MAX_PACKSIZE   : usize = 512;

/// The output is wrapped in a Result to allow matching on errors
/// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
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
  pub fn new(file_prefix : String) -> TofPacketWriter {
    let filename = file_prefix.clone() + "_0.tof.gaps";
    let path = Path::new(&filename); 
    let file = OpenOptions::new().append(true).open(path).expect("Unable to open file {filename}");
    TofPacketWriter {
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
      Err(err) => warn!("Writing to file with prefix {} failed. Err {}", self.file_prefix, err),
      Ok(_)     => ()
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

/// Connect to MTB Utp socket
pub fn connect_to_mtb(mt_ip   : &str, 
                      mt_port : &usize) 
  ->io::Result<UdpSocket> {
  let mt_address = mt_ip.to_owned() + ":" + &mt_port.to_string();
  let local_port = "0.0.0.0:50100";
  let local_addrs = [
    SocketAddr::from(([0, 0, 0, 0], 50100)),
    SocketAddr::from(([0, 0, 0, 0], 50101)),
  ];
  //let local_socket = UdpSocket::bind(local_port);
  let local_socket = UdpSocket::bind(&local_addrs[..]);
  let mut socket : UdpSocket;
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
      
      let ro = socket.set_read_timeout(Some(Duration::from_millis(1)));

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

/// Communications with the master trigger
///
///
pub fn master_trigger(mt_ip          : &str, 
                      mt_port        : usize,
                      glob_data_sink : &cbc::Sender<TofPacket>,
                      sender_rate    : &cbc::Sender<u32>,
                      evid_sender    : &cbc::Sender<MasterTriggerEvent>,
                      verbose        : bool) {

  let mt_address = mt_ip.to_owned() + ":" + &mt_port.to_string();
 
  let mut socket = connect_to_mtb(&mt_ip, &mt_port).expect("Can not create local UDP socket for MTB connection!"); 
  //socket.set_nonblocking(true).unwrap();
  
  // we only allocate the buffer once
  // and reuse it for all operations
  let mut buffer = [0u8;MT_MAX_PACKSIZE];  
  
  //let mut event_cnt      = 0u32;
  let mut last_event_cnt = 0u32;
  let mut missing_evids  = 0usize;
  let mut event_missing  = false;
  let mut n_events       = 0usize;
  // these are the number of expected events
  // (missing included)
  let mut n_events_expected = 0usize;
  let mut n_paddles_expected : u32;
  let mut rate = 0f64;
  // for rate measurement
  let start = Instant::now();

  let mut next_beat = true;
  // limit polling rate to a maximum
  let max_rate = 200.0; // hz
    
  // reset the master trigger before acquisiton
  info!("Resetting master trigger");
  reset_daq(&socket, &mt_address);  
  // the event counter has to be reset before 
  // we connect to the readoutboards
  //reset_event_cnt(&socket, &mt_address); 
  let mut mt_event = read_daq(&socket, &mt_address, &mut buffer);
  let mut timeout = Instant::now();
  //let timeout = Duration::from_secs(5);
  info!("Starting MT event loop at {:?}", timeout);

  let rate_query_rate = Duration::from_secs(5);
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
      socket = connect_to_mtb(&mt_ip, &mt_port).expect("Can not create local UDP socket for MTB connection!"); 
      timeout = Instant::now();
    }
    if timer.elapsed().as_secs() > 10 {
      match read_rate(&socket, &mt_address, &mut buffer) {
        Err(err) => {
          error!("Unable to obtain MT rate information!");
          continue;
        }
        Ok(rate) => {
          info!("Got rate from MTB {rate}");
          match sender_rate.try_send(rate) {
            Err(err) => error!("Can't send rate"),
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
    mt_event = read_daq(&socket, &mt_address, &mut buffer);
    //println!("Got event");
    match mt_event {
      Err(err) => {
        trace!("Did not get new event, Err {err}");
        continue;
      }
      Ok(_)    => ()
    }
    let ev = mt_event.unwrap();
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
      let mut missing = ev.event_id - last_event_cnt;
      error!("We missed {missing} eventids"); 
      // FIXME
      if missing < 200 {
        missing_evids += missing as usize;
      } else {
        warn!("We missed too many event ids from the master trigger!");
        missing = 0;
      }
      //error!("We missed {} events!", missing);
      event_missing = true;
    }
    
    trace!("Got new event id from master trigger {}",ev.event_id);
    match evid_sender.send(ev) {
      Err(err) => trace!("Can not send event, err {err}"),
      Ok(_)    => ()
    }
    last_event_cnt = ev.event_id;
    n_events += 1;
    n_events_expected = n_events + missing_evids;

    if n_events % 1000 == 0 {
      let pk = TofPacket::new();
      
    }

    let elapsed = start.elapsed().as_secs();
    // measure rate every 100 events
    if n_events % 1000 == 0 {
      rate = n_events as f64 / elapsed as f64;
      println!("==> [MASTERTRIGGER] {} events recorded, trigger rate: {:.3} Hz", n_events, rate);
      rate = n_events_expected as f64 / elapsed as f64;
      println!("==> -- expected rate {:.3} Hz", rate);   
    } 
    // end new event
  } // end loop
}





/// Get a list of ReadoutBoards from a json file
pub fn rb_manifest_from_json(config : json::JsonValue) -> Vec<ReadoutBoard> {
  let mut boards = Vec::<ReadoutBoard>::new();

  let nboards = config["readout_boards"].len();
  info!("Found configuration for {} readout boards!", nboards);
  for n in 0..nboards {
    let board_config   = &config["readout_boards"][n];
    let mut address_ip = String::from("tcp://");
    //let rb_comm_socket = ctx.socket(zmq::REP).unwrap();
    let rb_id = board_config["id"].as_usize().unwrap();
    address_ip += board_config["ip_address"].as_str().unwrap();
    let port        = board_config["port"].as_usize().unwrap();
    let address = address_ip.to_owned() + ":" + &port.to_string();
    let mut rb = ReadoutBoard::new();
    rb.id = Some(rb_id as u8);//           : Option<u8>,
    //mac_address  : Option<MacAddr6>,
    //rb.ip_address = Some(  : Option<Ipv4Addr>, 
    //rb.ip_address = Some(Ipv4Addr::from_str(address_ip).expect("Wrong format for ip!"));
    rb.data_port  = Some(port as u16);
    //cmd_port     : Option<u16>,
    //is_connected : bool,
    //uptime       : u32,
    boards.push (rb);

  }
  todo!();
  boards
}

/// Get the tof channel/paddle mapping and involved components
///
/// This reads the configuration from a json file and panics 
/// if there are any problems.
///
pub fn get_tof_manifest(json_config : std::path::PathBuf) -> (Vec::<LocalTriggerBoard>, Vec::<ReadoutBoard>) {
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
              // now we will try and check if the ports are open
              let mut all_data_ports = Vec::<String>::new();//scan_ports_range(30000..39999);
              let mut all_cmd_ports  = Vec::<String>::new();//scan_ports_range(40000..49999);
              // FIXME - the ranges here are somewhat arbitrary
              for n in 30000..39999 {
                all_data_ports.push(rb.ip_address.unwrap().to_string() + ":" + &n.to_string());
                //scan_ports_addrs(
              }
              for n in 40000..49999 {
                all_cmd_ports.push(rb.ip_address.unwrap().to_string() + ":" + &n.to_string());
              }
              let open_data_ports = scan_ports_addrs(all_data_ports);
              let open_cmd_ports  = scan_ports_addrs(all_cmd_ports);
              assert!(open_cmd_ports.len() < 2);
              assert!(open_data_ports.len() < 2);
              if open_cmd_ports.len() == 1 {
                rb.cmd_port = Some(open_cmd_ports[0].port());
                match rb.ping() {
                  Ok(_)    => println!("... connected!"),
                  Err(err) => println!("Can't connect to RB, err {err}"),
                }
              } else {
                rb.cmd_port = None;
              }
              

              println!("Found open data ports {:?}", open_data_ports);
              if open_data_ports.len() == 1 {
                rb.data_port = Some(open_data_ports[0].port());
              } else {
                rb.data_port = None;
              }
              if rb.is_connected {
                connected_boards.push(rb);
              }
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
    let mask = LocalTriggerBoard::get_mask_from_dsi_and_j(dsi, j);
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
  first_up         : u32,
}

impl ReadoutBoard {

  pub fn new() -> ReadoutBoard {
    ReadoutBoard {
      id           : None,
      mac_address  : None,
      ip_address   : None,
      data_port    : None,
      cmd_port     : None,
      is_connected : false,
      uptime       : 0,
      ch_to_pid    : [0;8],
      sorted_pids  : [0;4], 
      calib_file   : String::from(""),
      first_up     : 0
    }
  }

  /// Get the readoutboard ip address from 
  /// the ARP tables
  fn get_ip(&mut self) {
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
    
  /// Ping it  
  pub fn ping(&mut self) -> Result<(), Box<dyn Error>> { 
    // connect to the command port and send a ping
    // message
    let ctx =  zmq::Context::new();
    if matches!(self.ip_address, None) || matches!(self.cmd_port, None) {
      self.is_connected = false;
      return Err(Box::new(ReadoutBoardError::NoConnectionInfo));
    }
    let address = "tcp://".to_owned() + &self.ip_address.unwrap().to_string() + ":" + &self.cmd_port.unwrap().to_string(); 
    let socket  = ctx.socket(zmq::REQ)?;
    socket.connect(&address)?;
    info!("Have connected to adress {address}");
    // if the readoutboard is there, it should send *something* back
    let p = TofCommand::Ping(1);

    socket.send(p.to_bytestream(), 0)?;
    info!("Sent ping signal, waiting for response!");
    let data = socket.recv_bytes(0)?;
    if data.len() != 0 {
      self.is_connected = true;
      return Ok(());
    }
    self.is_connected = false;
    return Err(Box::new(ReadoutBoardError::NoResponse));
  }
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
  //pub mac_address  : Option<MacAddr6>,
  //pub ip_address   : Option<Ipv4Addr>, 
  //pub data_port    : Option<u16>,
  //pub cmd_port     : Option<u16>,
  //pub is_connected : bool,
  //pub uptime       : u32,
  //pub ch_to_pid    : [u8;8],
  //first_up         : u32,
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
    for ch in 0..ch_to_pid.len() {
      board.ch_to_pid[ch] = json["ch_to_pid"][&ch.to_string()].as_u8().unwrap();
    }
    let mut paddle_ids : [u8;4] = [0,0,0,0];
    let mut counter = 0;
    for ch in board.ch_to_pid.iter().step_by(2) {
      paddle_ids[counter] = *ch;
      counter += 1;
    }
    board.sorted_pids = paddle_ids;
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
