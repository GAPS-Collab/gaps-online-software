//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license
//!
//! Implementation of the IPBus protocoll for GAPS 
//!
//! Documentation about the IPBus protocoll can be found here.
//! [see docs here](https://ipbus.web.cern.ch/doc/user/html/)
//!
//! We are using only IPBus control packets
//!

use crate::prelude::*;


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
#[derive(Debug, PartialEq, Clone, Copy, FromRepr, AsRefStr, EnumIter)]
#[repr(u8)]
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

expand_and_test_enum!(IPBusPacketType, test_ipbuspackettype_repr);


///// Representation of a bytestream send via UDP
///// The IPBus protocoll implements different 
///// kinds of packets as well as packet counters
///// and is thus able to check for missing packets
//#[derive(Debug, Clone)]
//pub struct IPBusPacket {
//  /// packet identifier (counter)
//  pub pid   : u16,
//  pub ptype : IPBusPacketType,
//  /// payload 
//  pub data  : [u8;MT_MAX_PACKSIZE]
//}

/// Implementation of an IPBus control packet
#[derive(Debug)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct IPBus {
  pub socket         : UdpSocket,
  //pub target_address : String,
  pub packet_type    : IPBusPacketType,
  /// IPBus Packet ID - this is then NEXT
  /// pid which will be sent
  pub pid            : u16,
  pub expected_pid   : u16,
  pub last_pid       : u16,
  pub buffer         : [u8;MT_MAX_PACKSIZE]
}

impl IPBus {
  
  pub fn new(target_address : &str) 
    -> io::Result<Self> {
    let socket = Self::connect(target_address)?;
    let mut bus = Self {
      socket         : socket,
      //target_address : target_address,
      packet_type    : IPBusPacketType::Read,
      pid            : 0,
      expected_pid   : 0,
      last_pid       : 0,
      buffer         : [0;MT_MAX_PACKSIZE]
    };
    match bus.realign_packet_id() {
      Err(err) => {
        error!("Packet ID realign failed! {}", err); 
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
  pub fn connect(target_address : &str) 
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
          Err(err) => error!("Can not set read timeout for Udp socket! {err}"),
          Ok(_)    => ()
        }
        match socket.set_write_timeout(Some(Duration::from_millis(1))) {
          Err(err) => error!("Can not set write timeout for Udp socket! {err}"),
          Ok(_)    => ()
        }
        match socket.connect(target_address) {
          Err(err) => {
            error!("Can not connect to IPBus socket to target address {}! {}", target_address, err);
            return Err(err);
          }
          Ok(_)    => info!("Successfully connected IPBus to target address {}!", target_address)
        }
        match socket.set_nonblocking(false) {
          Err(err) => {
            error!("Can not set socket to blocking mode! {err}");
          },
          Ok(_) => ()
        }
        return Ok(socket);
      }
    } // end match
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
  pub fn receive(&mut self) -> io::Result<usize> {
    let number_of_bytes = self.socket.recv(&mut self.buffer)?;
    //thread::sleep(Duration::from_micros(UDP_SOCKET_SLEEP_USEC));
    Ok(number_of_bytes)
  }
 

  /// Receive number_of_bytes from UdpSocket and sleep after
  /// to avoid too many queries
  pub fn send(&mut self, data : &Vec<u8>) -> io::Result<()> {
    self.socket.send(data.as_slice())?;
    thread::sleep(Duration::from_micros(UDP_SOCKET_SLEEP_USEC));
    Ok(())
  }
  


  /// Send a ipbus status packet and receive the response
  ///
  /// Inspect self.buffer after the call if interested in 
  /// the result.
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
    let mut send_again = true;
    let mut number_of_bytes : usize;
    loop {
      if send_again {
        match self.send(&udp_data) {
          Err(err) => error!("Unable to send udp data! {err}"),
          Ok(_)    => ()
        }
      }
      trace!("[IPBus::get_status => message {:?} sent!", udp_data);
      match self.receive() {
        Err(err) => {
          error!("Can not receive status packet from Udp Socket! {err}");
          return Err(Box::new(IPBusError::UdpReceiveFailed));
        },
        Ok(_number_of_bytes)    => {
          number_of_bytes = _number_of_bytes;
        }
      }
      // check if this is really a status packet
      let status_byte = self.buffer[3];
      if status_byte & 0x1 != 1 {
        // not a status packet
        //return Err(Box::new(IPBusError::NotAStatusPacket));
        send_again = false;
        continue;
      } else {
        break;
      }
    }
    trace!("[IPBus::get_status] => {} bytes received!", number_of_bytes);
    //println!("[IPBus::get_status] => buffer {:?}", self. buffer);
    for word in 0..16 {
      trace!("[IPBus::get_status] => WORD {word} : [{},{},{},{}]", self.buffer[word*4], self.buffer[word*4 + 1], self.buffer[word*4+2], self.buffer[word*4+3]);
    }
    Ok(())
  }


  /// Assemble the 32bit packet header 
  ///
  /// This will include the (presume) next
  /// packet id
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
               | ((self.packet_type as u8 & 0xf) << 4) as u32
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
 
  pub fn get_pid_from_current_buffer(&self) -> u16 {
    let buffer   = self.buffer.to_vec();
    let pheader  = parse_u32_be(&buffer, &mut 0);
    let pid      = ((0x00ffff00 & pheader) >> 8) as u16;
    pid
  }

  /// Unpack a binary representation of an IPBusPacket
  ///
  /// # Arguments:
  ///
  /// * message : The binary representation following 
  ///             the specs of IPBus protocoll
  /// * verbose : print information for debugging.
  ///
  fn decode_payload(&mut self,
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
    let packet_type = IPBusPacketType::from(ptype);
    trace!("[IPBus::decode_payload] => PID, SIZE, PTYPE : {} {} {}", pid, size, packet_type);
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
        if (((size as usize) * 4) + 11) < MT_MAX_PACKSIZE { 
          for i in 0..size as usize {
            data.push(  ((self.buffer[8 + i * 4]  as u32) << 24) 
                      | ((self.buffer[9 + i * 4]  as u32) << 16) 
                      | ((self.buffer[10 + i * 4] as u32) << 8)  
                      |   self.buffer[11 + i * 4]  as u32)
          }
        } else {
          error!("Size {} larger than bufffer len {}", size, data.len());
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
    let pid = self.get_target_next_expected_packet_id()?;
    self.pid = pid;
    self.expected_pid = pid;
    ////match self.get_target_next_expected_packet_id() {
    //  Ok(pid) => {
    //    self.pid = pid;
    //  }
    //  Err(err) => {
    //    error!("Can not get next expected packet id from target, will use 0! {err}");
    //    self.pid = 0;
    //  }
    //}
    //self.expected_pid = self.pid;
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


  /// Multiple read operations with a single UDP request
  ///
  /// Read either the same register multiple times 
  /// or read from  incrementing register addresses 
  ///
  /// # Arguments:
  ///
  /// * addr           : register addresss to read 
  ///                    from
  /// * nwords         : number of read operations
  /// * increment_addr : if true, increment the 
  ///                    register address after
  ///                    each read operation
  pub fn read_multiple(&mut self,
                       addr           : u32,
                       nwords         : usize,
                       increment_addr : bool) 
    -> Result<Vec<u32>, Box<dyn Error>> {
    let send_data = vec![0u32;nwords];
    if increment_addr {
      self.packet_type = IPBusPacketType::Read;
    } else {
      self.packet_type = IPBusPacketType::ReadNonIncrement;
    }
    let mut message = self.encode_payload(addr, &send_data);
    let mut send_again = true;
    let timeout = Instant::now();
    loop {
      if send_again {
        self.send(&message)?;
      }
      match self.receive() {
        Err(_err) => {
          // In case the address is correct, this
          // MUST be some kind of timeout/udp issue. 
          // We assume the packet is lost, realign
          // the packet id and try again
          //
          //// this will be the last pid we have received
          //error!("Can not receive from socket! {err}. self.pid {}, self.expected_pid {}, buffer pid {}", self.pid, self.expected_pid, pid_from_buffer);
          self.realign_packet_id()?;
          // we need to rewrite the message with the 
          // new packet id
          message    = self.encode_payload(addr, &send_data);
          send_again = true;
          if timeout.elapsed().as_millis() == 10 {
            return Err(Box::new(IPBusError::UdpReceiveFailed));
          }
          continue;
        }
        Ok(number_of_bytes) => {
          
          if self.buffer_is_status() {
            // if we received a stray status message, let's just try again
            continue;
          }
          
          let pid_from_buffer = self.get_pid_from_current_buffer();
          if pid_from_buffer != self.expected_pid {
            error!("We got a packet, but the PacketID is not as expected!");
            error!("-- self.pid {}, self.expected_pid {}, buffer pid {}", self.pid, self.expected_pid, pid_from_buffer);
            // we try to fix it. If it is behind, we can just call receive again
            if self.expected_pid > pid_from_buffer {
              send_again = false;
              continue;
            } else {
              // we have missed out on messages and need to send our original message
              // again
              self.realign_packet_id()?;
              message    = self.encode_payload(addr, &send_data);
              send_again = true;
              continue;
            }
          }
          trace!("[IPBus::read] => Received {} bytes from master trigger! Message {:?}", number_of_bytes, self.buffer);
          break;
        }
      } // end match
    } // here we must have either gotten the message or 
    let data = self.decode_payload(false)?;  
    if data.len() == 0 {
      error!("Received empty data!");
      return Err(Box::new(IPBusError::DecodingFailed));
    }
    Ok(data)
  }
  
  /// Read a single value from a register
  ///
  /// # Arguments:
  ///
  /// * addr : register address to be read from
  pub fn read(&mut self, 
              addr : u32) 
    -> Result<u32, Box<dyn Error>> {
  
    let data  = self.read_multiple(addr,1,false)?;
    Ok(data[0])
  }
  
  /// Write a single value to a register
  ///
  /// # Arguments
  ///
  /// * addr        : target register address
  /// * data        : word to write in register
  pub fn write(&mut self,
               addr   : u32,
               data   : u32)
      -> Result<(), Box<dyn Error>> {
    // we don't expect any issues with sending the data
    let send_data = Vec::<u32>::from([data]);
    self.packet_type = IPBusPacketType::Write;
    let message = self.encode_payload(addr, &send_data);
    match self.send(&message) {
      Err(err) => {
        error!("Sending Udp message failed! {err}");
        return Err(Box::new(IPBusError::UdpSendFailed));
      },
      Ok(_) => ()
    }
    match self.receive() {
      // this can have two failure modes
      // 1) we receive nothing (timeout)
      Err(err) => {
        //if err.kind = std::io::ErrorKind  
        error!("Unable to receive data! i/o error : {}", err.kind());
        let target_exp_pid = self.get_target_next_expected_packet_id()?;
        let buffer_pid = self.get_pid_from_current_buffer();
        error!("self.pid {}, self.expected.pid {}, target expects pid {:?}, buffer pid {} ", self.pid, self.expected_pid, target_exp_pid, buffer_pid);
        return Err(Box::new(IPBusError::UdpReceiveFailed));
        //if err == IPBusError::InvalidPacketID {
      }
      Ok(_data) => {
        // _data is tne number of bytes received
        trace!("[ipbus::write] => Got buffer {:?}", self.buffer);
        return Ok(());  
      }
    }
  } 
}

impl fmt::Display for IPBus {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr  = String::from("<IPBus:");
    repr         += &(format!("  pid : {}>", self.pid)); 
    write!(f, "{}", repr)
  }
}


#[cfg(feature="pybindings")]
#[pymethods]
impl IPBus {
  #[new]
  fn new_py(target_address : &str) -> Self {
    IPBus::new(target_address).expect("Unable to connect to {target_address}")
  }

  /// Make a IPBus status query
  #[getter]
  #[pyo3(name="status")]
  pub fn get_status_py(&mut self) -> PyResult<()> {
    match self.get_status() {
      Ok(_) => {
        return Ok(());
      },
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  #[getter]
  #[pyo3(name="buffer")]
  pub fn get_buffer(&self) -> [u8;MT_MAX_PACKSIZE] {
    return self.buffer.clone();
  }

  #[pyo3(name="set_packet_id")]
  pub fn set_packet_id_py(&mut self, pid : u16) {
    self.pid = pid;
  }
 
  #[pyo3(name="get_packet_id")]
  pub fn get_packet_id_py(&self) -> u16 {
    self.pid
  }

  #[getter]
  #[pyo3(name="expected_pid")]
  pub fn get_expected_packet_id_py(&self) -> u16 {
    self.expected_pid
  }

  /// Set the packet id to that what is expected from the targetr
  #[pyo3(name="realign_packet_id")]
  pub fn realign_packet_id_py(&mut self) -> PyResult<()> {
    match self.realign_packet_id() {
      Ok(_) => {
        return Ok(());
      },
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  /// Get the next packet id, which is expected by the target
  #[getter]
  #[pyo3(name="target_next_expected_pid")]
  pub fn get_target_next_expected_packet_id_py(&mut self) 
    -> PyResult<u16> {
    match self.get_target_next_expected_packet_id() {
      Ok(result) => {
        return Ok(result);
      },
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  #[pyo3(name="read_multiple")]
  pub fn read_multiple_py(&mut self,
                          addr           : u32,
                          nwords         : usize,
                          increment_addr : bool) 
    -> PyResult<Vec<u32>> {
  
    match self.read_multiple(addr,
                             nwords,
                             increment_addr) {
      Ok(result) => {
        return Ok(result);
      },
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  #[pyo3(name="write")]
  pub fn write_py(&mut self,
               addr   : u32,
               data   : u32) 
    -> PyResult<()> {
    
    match self.write(addr, data) {
      Ok(_) => Ok(()),
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
 

  #[pyo3(name="read")]
  pub fn read_py(&mut self, addr   : u32) 
    -> PyResult<u32> {
    match self.read(addr) {
      Ok(result) => {
        return Ok(result);
      },
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}
