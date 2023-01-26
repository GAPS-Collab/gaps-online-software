//pub mod misc;

use port_scanner::scan_ports_addrs;

use std::error::Error;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr};
// FIXME - remove this crate
//use mac_address::MacAddress;
use zmq;

extern crate json;

use macaddr::MacAddr6;
use netneighbours::get_mac_to_ip_map;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use tof_dataclasses::commands::{TofCommand, TofResponse};

extern crate pretty_env_logger;
#[macro_use] extern crate log;

#[macro_use] extern crate manifest_dir_macros;

//extern crate libarp;

//use libarp::{arp::ArpMessage, client::ArpClient, interfaces::Interface, interfaces::MacAddr};


///// Stolen from the arp-toolkit example
//fn resolve_simple(mac_addr: MacAddress, ip_addr: Ipv4Addr) {
//    let mut client = ArpClient::new().unwrap();
//
//
//    println!("Simple: IP for MAC {} is {}", mac_addr, result.unwrap());
//
//    let result = client.ip_to_mac(ip_addr, None);
//    println!("Simple: MAC for IP {} is {}", ip_addr, result.unwrap());
//}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
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
            IpAddr::V6(a) => panic!("IPV6 not suppported!"),
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
pub fn discover_boards() -> Vec<ReadoutBoard> {
  let board_list = Vec::<ReadoutBoard>::new();
  board_list
}


/// A generic representation of a Readout board
///
///
///
#[derive(Debug, Copy, Clone)]
pub struct ReadoutBoard {
  pub id           : Option<u8>,
  pub mac_address  : Option<MacAddr6>,
  pub ip_address   : Option<Ipv4Addr>, 
  pub data_port    : Option<u16>,
  pub cmd_port     : Option<u16>,
  pub is_connected : bool,
  pub uptime       : u32,
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
      first_up     : 0
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
    write!(f, "<ReadoutBoard: \n ID \t\t: {} \n MAC addr \t: {} \n IP addr \t: {} \n 0MQ PUB \t: {} \n 0MQ REP \t: {} \n connected \t: {} \n uptime \t: {} >", 
            self.id.unwrap_or(0).to_string()           ,      
            self.mac_address.unwrap_or(default_mac).to_string()  ,
            self.ip_address.unwrap_or(default_ip).to_string()   ,
            self.data_port.unwrap_or(0).to_string()    ,
            self.cmd_port.unwrap_or(0).to_string()     , 
            self.is_connected.to_string() , 
            self.uptime.to_string()       ,
    )
  }
}

impl Default for ReadoutBoard {
  fn default() -> ReadoutBoard {
    ReadoutBoard::new()
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
