//! Implementation of the IPBus protocoll for GAPS 
//!
//! Documentation about the IPBus protocoll can be found here.
//! [see docs here](https://ipbus.web.cern.ch/doc/user/html/)
//!
//! We are using only IPBus control packets
//!

use std::fmt;
use std::thread;
use std::io;
use std::net::{
    UdpSocket,
    SocketAddr
};

use std::error::Error;
use std::time::Duration;

use crate::errors::IPBusError;
use crate::serialization::{
    //parse_u32,
    parse_u32_be
};

// we have some header and then the board mask (4byte)
// + at max 20*2 byte for the individual LTBs.
// -> guestimate says 128 byte are enough
pub const MT_MAX_PACKSIZE        : usize = 128;

/// Sleeptime between consequtive UDP queries
/// in microsec
pub const UDP_SOCKET_SLEEP_USEC  : u64 = 100;

/// The IPBus standard encodes several packet types.
///
/// The packet type then will 
/// instruct the receiver to either 
/// write/read/etc. values from its
/// registers.
///
/// Technically, the IPBusPacketType is 
/// only 1 byte!
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum IPBusPacketType {
  Read                 = 0,
  Write                = 1,
  /// For reading multiple words,
  /// this will read the same 
  /// register multiple times
  ReadNonIncrement     = 2,
  WriteNonIncrement    = 3,
  RMW                  = 4,
  /// This is not following IPBus packet
  /// specs
  Unknown              = 99
}

impl IPBusPacketType {

  pub fn to_u8(&self) -> u8 {
    let ret_val : u8;
    match self {
      IPBusPacketType::Read => {
        ret_val = 0;
      }
      IPBusPacketType::Write => {
        ret_val = 1;
      }
      IPBusPacketType::ReadNonIncrement => {
        ret_val = 2;
      }
      IPBusPacketType::WriteNonIncrement => {
        ret_val = 3;
      }
      IPBusPacketType::RMW => {
        ret_val = 4;
      }
      IPBusPacketType::Unknown => {
        ret_val = 99;
      }
    }
    ret_val
  }

  pub fn from_u8(ptype : u8) -> Self {
    let ptype_val : Self;
    match ptype {
      0 => {ptype_val = IPBusPacketType::Read;}
      1 => {ptype_val = IPBusPacketType::Write;}
      2 => {ptype_val = IPBusPacketType::ReadNonIncrement;}
      3 => {ptype_val = IPBusPacketType::WriteNonIncrement;}
      4 => {ptype_val = IPBusPacketType::RMW;}
      _ => {ptype_val = IPBusPacketType::Unknown;}
    }
    return ptype_val;
  }
}

impl fmt::Display for IPBusPacketType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let repr : String;
    match self {
      IPBusPacketType::Read                 => {repr = String::from("Read");} 
      IPBusPacketType::Write                => {repr = String::from("Write");} 
      IPBusPacketType::ReadNonIncrement     => {repr = String::from("ReadNonIncrement");} 
      IPBusPacketType::WriteNonIncrement    => {repr = String::from("WriteNonIncrement");} 
      IPBusPacketType::RMW                  => {repr = String::from("RMW");} 
      IPBusPacketType::Unknown              => {repr = String::from("Unknown");}
    }
    write!(f, "<IPBusPacketType: {}>", repr)
  }
}

#[derive(Debug, Clone)]
pub struct IPBusPacket {
  pub pid   : u16,
  pub ptype : IPBusPacketType,
  pub data  : [u8;MT_MAX_PACKSIZE]
}

/// Implementation of an IPBus control packet
#[derive(Debug)]
pub struct IPBus {
  pub socket         : UdpSocket,
  pub target_address : String,
  pub packet_type    : IPBusPacketType,
  /// IPBus Packet ID 
  pub pid            : u16,
  pub expected_pid   : u16,
  pub last_pid       : u16,
  pub buffer         : [u8;MT_MAX_PACKSIZE]
}

impl IPBus {
  
  pub fn new(target_address : String) 
    -> io::Result<Self> {
    let socket = Self::connect(&target_address)?;
    let mut bus = Self {
      socket         : socket,
      target_address : target_address,
      packet_type    : IPBusPacketType::Read,
      pid            : 0,
      expected_pid   : 0,
      last_pid       : 0,
      buffer         : [0;MT_MAX_PACKSIZE]
    };
    match bus.realign_packet_id() {
      Err(err) => {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Can not realign packet id"));
      },
      Ok(_) => ()
    }
    Ok(bus)
  }

  /// Connect to MTB Utp socket
  ///
  /// This will try a number of options to bind 
  /// to the local port.
  /// 
  /// # Arguments 
  ///
  /// * target_address  : IP/port of the target 
  ///                     probably some kind of
  ///                     FPGA
  pub fn connect(target_address : &String) 
    ->io::Result<UdpSocket> {
    // provide a number of local ports to try
    let local_addrs = [
      SocketAddr::from(([0, 0, 0, 0], 50100)),
      SocketAddr::from(([0, 0, 0, 0], 50101)),
      SocketAddr::from(([0, 0, 0, 0], 50102)),
      SocketAddr::from(([0, 0, 0, 0], 50103)),
      SocketAddr::from(([0, 0, 0, 0], 50104)),
    ];
    let local_socket = UdpSocket::bind(&local_addrs[..]);
    let socket : UdpSocket;
    match local_socket {
      Err(err)   => {
        error!("Can not create local UDP socket for master trigger connection!, err {}", err);
        return Err(err);
      }
      Ok(value)  => {
        info!("Successfully bound UDP socket for master trigger communcations to {:?}", value);
        socket = value;
        // this is not strrictly necessary, but 
        // it is nice to limit communications
        match socket.set_read_timeout(Some(Duration::from_millis(1))) {
          Err(err) => error!("Can not set read timeout for Udp socket! Error {err}"),
          Ok(_)    => ()
        }
        match socket.connect(&target_address) {
          Err(err) => {
            error!("Can not connect to IPBus socket to target address {}! {}", target_address, err);
            return Err(err);
          }
          Ok(_)    => info!("Successfully connected IPBus to target address {}!", target_address)
        }
        return Ok(socket);
      }
    } // end match
  }  

  /// Reconnect to the same address after timeout
  pub fn reconnect(&mut self) 
    -> io::Result<()> {
    self.socket = Self::connect(&self.target_address)?;
    Ok(())
  }


  /// Get the next 12bit transaction ID. 
  /// If we ran out, wrap around and 
  /// start at 0
  fn get_next_pid(&mut self) -> u16 {
    let pid = self.pid;
    self.expected_pid = self.pid;
    //// get the next transaction id 
    self.pid += 1;
    // wrap around
    if self.pid > u16::MAX {
      self.pid = 0;
      return 0;
    }
    return pid;
  }

  /// Receive number_of_bytes from UdpSocket and sleep after
  /// to avoid too many queries
  pub fn receive(&mut self) -> Result<usize, Box<dyn Error>> {
    let (number_of_bytes, _) = self.socket.recv_from(&mut self.buffer)?;
    //thread::sleep(Duration::from_micros(UDP_SOCKET_SLEEP_USEC));
    Ok(number_of_bytes)
  }
  
  /// Receive number_of_bytes from UdpSocket and sleep after
  /// to avoid too many queries
  pub fn send(&mut self, data : &Vec<u8>) -> Result<(), Box<dyn Error>> {
    self.socket.send(data.as_slice())?;
    thread::sleep(Duration::from_micros(UDP_SOCKET_SLEEP_USEC));
    Ok(())
  }

  pub fn get_status(&mut self) 
    -> Result<(), Box<dyn Error>> {
    let mut udp_data = Vec::<u8>::new();
    let mut phead  = self.create_packetheader(true);
    phead = phead & 0xfffffff0;
    phead = phead | 0x00000001;
    udp_data.extend_from_slice(&phead.to_be_bytes());
    for _ in 0..15 {
      udp_data.push(0);
      udp_data.push(0);
      udp_data.push(0);
      udp_data.push(0);
    }
    //self.socket.send(udp_data.as_slice())?;
    match self.send(&udp_data) {
      Err(err) => error!("Unable to send udp data!"),
      Ok(_)    => ()
    }
    trace!("[IPBus::get_status => message {:?} sent!", udp_data);
    //let (number_of_bytes, _) = self.socket.recv_from(&mut self.buffer)?;
    //let (number_of_bytes, _) = self.socket.recv_from(&mut self.buffer)?;
    //let number_of_bytes = self.receive()?;
    let number_of_bytes : usize;
    match self.receive() {
      Err(err) => {
        error!("Can not receive from Udp Socket");
        return Err(Box::new(IPBusError::NotAStatusPacket));
      },
      Ok(_number_of_bytes)    => {
        number_of_bytes = _number_of_bytes;
      }
    }
    // check if this is really a status packet
    let status_byte = self.buffer[3];
    if status_byte & 0x1 != 1 {
      // not a status packet
      return Err(Box::new(IPBusError::NotAStatusPacket));
    }
    trace!("[IPBus::get_status] => {} bytes received!", number_of_bytes);
    //println!("[IPBus::get_status] => buffer {:?}", self. buffer);
    for word in 0..16 {
      trace!("[IPBus::get_status] => WORD {word} : [{},{},{},{}]", self.buffer[word*4], self.buffer[word*4 + 1], self.buffer[word*4+2], self.buffer[word*4+3]);
    }
    Ok(())
  }

  fn create_packetheader(&mut self, status : bool) -> u32 {
    // we use this to switch the byteorder
    let pid : u16;
    if status {
      pid = 0;
    } else {
      pid = self.get_next_pid();
    }
    let pid_bytes = pid.to_be_bytes(); 
    let pid_be0   = (pid_bytes[0] as u32) << 16;
    let pid_be1   = (pid_bytes[1] as u32) << 8;
    let header = (0x2 << 28) as u32
               | (0x0 << 24) as u32
               | pid_be0
               | pid_be1
               | (0xf << 4) as u32
               | 0x0 as u32; // 0 means control packet, we will 
                             // only use control packets in GAPS
    trace!("[IPBus::create_packetheader] => Will use packet ID {pid}");
    trace!("[IPBus::create_packetheader] => Generated header {:?}", header.to_be_bytes());
    header
  }

  fn create_transactionheader(&self, nwords : u8) -> u32 {
    let header = (0x2 << 28) as u32
               | (0x0 << 24) as u32
               | (0x0 << 20) as u32
               | (0x0 << 16) as u32
               | (nwords as u32) << 8
               | ((self.packet_type.to_u8() & 0xf) << 4) as u32
               | 0xf as u32; // 0xf is for outbound request 
    header
  }

  /// Encode register addresses and values in IPBus packet
  ///
  /// # Arguments:
  ///
  /// * addr        : register addresss
  /// * packet_type : read/write register?
  /// * data        : the data value at the specific
  ///                 register.
  ///                 In case packet type is Write/Read
  ///                 len of data has to be 1
  ///
  fn encode_payload(&mut self,
                    addr        : u32,
                    data        : &Vec<u32>) -> Vec<u8> {
    let mut udp_data = Vec::<u8>::new();
    let pheader = self.create_packetheader(false);
    let nwords  = data.len() as u8;
    trace!("[IPBus::encode_payload] => Encoding payload for packet type {}!", self.packet_type);
    let theader = self.create_transactionheader(nwords);
    udp_data.extend_from_slice(&pheader.to_be_bytes());
    udp_data.extend_from_slice(&theader.to_be_bytes());
    udp_data.extend_from_slice(&addr.to_be_bytes());
    if self.packet_type    == IPBusPacketType::Write
     || self.packet_type == IPBusPacketType::WriteNonIncrement { 
      for i in data {
        udp_data.extend_from_slice(&i.to_be_bytes());
      }
    }
    trace!("[IPBus::encode_payload] => payload {:?}", udp_data);
    udp_data
  }
  
  /// Unpack a binary representation of an IPBusPacket
  ///
  /// # Arguments:
  ///
  /// * message : The binary representation following 
  ///             the specs of IPBus protocoll
  /// * verbose : print information for debugging.
  ///
  /// FIXME - currently this is always successful.
  /// Should we check for garbage?
  fn decode_payload(&mut self,
                    debug_pid : &mut u16,
                    verbose : bool)
    -> Result<Vec<u32>, IPBusError> {
    let mut pos  : usize = 0;
    let mut data = Vec::<u32>::new();
    let buffer   = self.buffer.to_vec();
    // check if this is a status packet
    let is_status = buffer[3] & 0x1 == 1;
    trace!("[IPBus::decode_payload] => buffer (vec) {:?}", buffer); 
    let pheader  = parse_u32_be(&buffer, &mut pos);
    let theader  = parse_u32_be(&buffer, &mut pos);
    trace!("[IPBus::decode_payload] => pheader {pheader}"); 
    trace!("[IPBus::decode_payload] => theader {theader}"); 
    let pid      = ((0x00ffff00 & pheader) >> 8) as u16;
    let size     = ((0x0000ff00 & theader) >> 8) as u16;
    let ptype    = ((0x000000f0 & theader) >> 4) as u8;
    let packet_type = IPBusPacketType::from_u8(ptype);
    trace!("[IPBus::decode_payload] => PID, SIZE, PTYPE : {} {} {}", pid, size, packet_type);
    *debug_pid = pid;
    if pid != self.expected_pid {
      if !is_status {
        error!("Invalid packet ID. Expected {}, received {}", self.expected_pid, pid);
        // we do know that the next expected packet id should be the latest one + 1
        //if pid == u16::MAX {
        //  self.expected_pid = 0; 
        //} else {
        //  self.expected_pid = pid + 1;
        //}
        return Err(IPBusError::InvalidPacketID);
      }
    }
    match packet_type {
      IPBusPacketType::Unknown => {
        return Err(IPBusError::DecodingFailed);
      }
      IPBusPacketType::Read |
      IPBusPacketType::ReadNonIncrement => {
        for i in 0..size as usize {
          data.push(  ((self.buffer[8 + i * 4]  as u32) << 24) 
                    | ((self.buffer[9 + i * 4]  as u32) << 16) 
                    | ((self.buffer[10 + i * 4] as u32) << 8)  
                    |   self.buffer[11 + i * 4]  as u32)
        }
      },
      IPBusPacketType::Write => {
        data.push(0);
      },
      IPBusPacketType::WriteNonIncrement => {
        error!("Decoding of WriteNonIncrement packet not supported!");
      },
      IPBusPacketType::RMW => {
        error!("Decoding of RMW packet not supported!!");
      }
    }
    if verbose { 
      println!("[IPBus::decode_payload] ==> Decoding IPBus Packet:");
      println!(" >> Msg            : {:?}", self.buffer);
      //println!(" >> IPBus version  : {}", ipbus_version);
      //println!(" >> Transaction ID : {}", tid);
      //println!(" >> ID             : {}", id);
      //println!(" >> Size           : {}", size);
      //println!(" >> Type           : {:?}", packet_type);
      //println!(" >> Info           : {}", info_code);
      println!(" >> data           : {:?}", data);
    }
    Ok(data)
  }

  /// Set the packet id to that what is expected from the targetr
  pub fn realign_packet_id(&mut self) 
    -> Result<(), Box<dyn Error>> {
    trace!("[IPBus::realign_packet_id] - aligning...");
    match self.get_target_next_expected_packet_id() {
      Ok(pid) => {
        self.pid = pid;
      }
      Err(err) => {
        error!("Can not get next expected packet id from target, will use 0");
        self.pid = 0;
      }
    }
    self.expected_pid = self.pid;
    trace!("[IPBus::realign_packet_id] - aligned {}", self.pid);
    Ok(())
  }

  pub fn buffer_is_status(&self) -> bool {
    self.buffer[3] & 0x1 == 1
  }

  /// Get the packet id which is expected by the target
  pub fn get_target_next_expected_packet_id(&mut self)
    -> Result<u16, Box<dyn Error>> {
    self.get_status()?;
    // the expected packet id is in WORD 3
    let word = 3usize;
    trace!("[IPBus::get_status] => WORD {word} : [{},{},{},{}]", self.buffer[word*4], self.buffer[word*4 + 1], self.buffer[word*4+2], self.buffer[word*4+3]);
    let word3 = [self.buffer[word*4], self.buffer[word*4 + 1], self.buffer[word*4 + 2], self.buffer[word*4 + 3]];
    let target_exp_pid = u16::from_be_bytes([word3[1], word3[2]]);
    trace!("[IPBus::target_next_pid] => Get expected packet id {target_exp_pid}");
    Ok(target_exp_pid)
  }

  pub fn read(&mut self, addr   : u32, verify_tid : bool) 
    -> Result<u32, Box<dyn Error>> {
    let send_data = Vec::<u32>::from([0]);
    self.packet_type = IPBusPacketType::Read;
    let message   = self.encode_payload(addr, &send_data);
    trace!("[IPBus::read => sending message {:?}", message);
    //self.socket.send(message.as_slice())?;
    self.send(&message)?;
    //println!("[IPBus::read => message sent!");
    let mut data = Vec::<u32>::new();
    let mut debug_pid = 0u16;
    let mut notify_success = false;
    let mut ntries    = 4usize;
    loop {
      //let (number_of_bytes, _) = self.socket.recv_from(&mut self.buffer)?;
      let number_of_bytes = self.receive()?;
      trace!("[IPBus::read] => Received {} bytes from master trigger! Message {:?}", number_of_bytes, self.buffer);
      if self.buffer_is_status() {
        continue;
      }
      // this one can actually succeed, but return an emtpy vector
      match self.decode_payload(&mut debug_pid,false) { 
        Err(err) => {
          ntries -= 1;
          if ntries == 0 {
            // FIXME - this might not be the best error to return
            return Err(Box::new(IPBusError::DecodingFailed));
          }
          if err == IPBusError::InvalidPacketID {
            debug!("--> invalid packet id, trying again");
            if verify_tid {
              if debug_pid < self.expected_pid {
                if self.expected_pid - debug_pid == 1 {
                  // in this case, we simply try again
                  error!("[IPBus::read] Packet ID is 1 behind.. retry!");
                  notify_success = true;
                  continue;
                }
              } else {
                match self.realign_packet_id() {
                  Err(err) => {
                    error!("Unable to realign packet id! {err}");
                    return Err(err);
                  }
                  Ok(_) => ()
                }
              }
              notify_success = true;
              continue;
            } else {
              break;
            }
          } else {
            error!("[IPBus::read] Received error {err}");
            break;
          }
        }
        Ok(_data) => {
          data = _data;
          if notify_success {
            println!("[IPBus::read] Packet ID has been restored, data acquired..");
          }
          break;
        }
      } 
    }
    if data.len() == 0 {
      error!("[IPBus::read] Data has size 0");
      return Err(Box::new(IPBusError::DecodingFailed));
    }
    Ok(data[0])
  }

  pub fn read_multiple(&mut self,
                       addr           : u32,
                       nwords         : usize,
                       increment_addr : bool,
                       verify_tid     : bool) 
    -> Result<Vec<u32>, Box<dyn Error>> {
    let send_data = vec![0u32;nwords];
    let mut data = Vec::<u32>::new();
    let mut debug_pid = 0u16;
    if increment_addr {
      self.packet_type = IPBusPacketType::Read;
    } else {
      self.packet_type = IPBusPacketType::ReadNonIncrement;
    }
    // FIXME - we assume nwords != 1
    let mut message = self.encode_payload(addr, &send_data);
    self.send(&message)?;
    let mut n_send_failures = 0usize;
    loop {
      //let (number_of_bytes, _) = self.socket.recv_from(&mut self.buffer)?;
      let number_of_bytes : usize;
      match self.receive() {
        Err(err) => {
          n_send_failures += 1;
          if n_send_failures == 4 {
            error!("allowed send failures exceeded!");
            return Err(err);
          }
          if self.buffer_is_status() {
            continue;
          }
          // the only error this can throw is a timeout
          // This most likely means the pid is wrong
          let which_pid_next = self.get_target_next_expected_packet_id()?;
          error!("self.receive threw {err}. pid {} exp. pid.{} target exp. pid.{}", self.pid, self.expected_pid, which_pid_next);
          self.realign_packet_id()?;
          message = self.encode_payload(addr, &send_data);
          self.send(&message)?;
          continue;
        }
        Ok(_number_of_bytes) => {
          number_of_bytes = _number_of_bytes;
          if self.buffer_is_status() {
            continue;
          }

        }
      }
      trace!("[IPBus::read_multiple] Received {} bytes from master trigger. Buffer {:?}", number_of_bytes, self.buffer);
      // this one can actually succeed, but return an emtpy vector
      match self.decode_payload(&mut debug_pid,false) { 
        Err(err) => {
          debug!("Got error {err}!");
          if err == IPBusError::InvalidPacketID {
            debug!("--> invalid packet id, trying again");
            if verify_tid {
              self.realign_packet_id()?;
              continue;
            } else {
              break;
            }
          }
        }
        Ok(_data) => {
          data = _data;
          break;
        }
      } 
    }
    if data.len() == 0 {
      error!("Received empty data!");
      return Err(Box::new(IPBusError::DecodingFailed));
    }
    Ok(data)
  }
  
  pub fn write(&mut self,
               addr   : u32,
               data   : u32) 
    -> Result<(), Box<dyn Error>> {
    let send_data = Vec::<u32>::from([data]);
    self.packet_type = IPBusPacketType::Write;
    let message = self.encode_payload(addr, &send_data);
    //self.socket.send(message.as_slice())?;
    self.send(&message)?;
    //let (number_of_bytes, _) = self.socket.recv_from(&mut self.buffer)?;
    let number_of_bytes = self.receive()?;
    trace!("Received {} bytes from master trigger", number_of_bytes);
    Ok(())
  }

}

impl fmt::Display for IPBus {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr  = String::from("<IPBus:");
    repr         += &(format!("  pid : {}>", self.pid)); 
    write!(f, "{}", repr)
  }
}

