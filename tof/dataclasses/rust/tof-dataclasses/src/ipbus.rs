//! Implementation of the IPBus protocoll for GAPS 
//!
//! Documentation about the IPBus protocoll can be found here.
//! [see docs here](https://ipbus.web.cern.ch/doc/user/html/)
//!
//! We are using only IPBus control packets
//!

use std::fmt;
use std::net::UdpSocket;
use std::error::Error;

use crate::errors::IPBusError;
use crate::serialization::parse_u32_be;

// we have some header and then the board mask (4byte)
// + at max 20*2 byte for the individual LTBs.
// -> guestimate says 128 byte are enough
const MT_MAX_PACKSIZE   : usize = 128;


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

/// Implementation of an IPBus control packet
#[derive(Debug)]
pub struct IPBus {
  pub socket         : UdpSocket,
  pub packet_type    : IPBusPacketType,
  /// IPBus Transaction ID (12bit)
  pub tid            : u16,
  pub expected_tid   : u16,
  pub buffer         : [u8;MT_MAX_PACKSIZE]
}

impl IPBus {
  
  pub fn new(socket : UdpSocket) -> Self {
    Self {
      socket       : socket,
      packet_type  : IPBusPacketType::Read,
      /// actually the transaction id should be the packet id
      tid          : 0,
      expected_tid : 0,
      buffer       : [0;MT_MAX_PACKSIZE]
    }
  }
  
  fn assemble_tid(tid : (u8,u8)) -> u16 {
    let tidu16 = (tid.1 as u16) << 8 | tid.0 as u16;
    return tidu16;
  }

  fn disassemble_tid(tid : u16) -> (u8,u8) {
    let tid0 : u8 = (0x0ff  & tid) as u8;
    let tid1 : u8 = ((0xf00 & tid) >> 8) as u8;
    return (tid0, tid1);
  }


  /// Get the next 12bit transaction ID. 
  /// If we ran out, wrap around and 
  /// start at 0
  fn get_next_tid(&mut self) -> u16 {
    //let tid = Self::disassemble_tid(self.tid);
    let tid = self.tid;
    self.expected_tid = self.tid;
    //// get the next transaction id 
    //self.tid += 1;
    //// wrap around
    //if self.tid > 0xfff {
    //  self.tid = 0;
    //}
    return tid;
  }

  pub fn get_status(&mut self) {
    let mut udp_data = Vec::<u8>::new();
    let mut phead  = self.create_packetheader();
    phead = phead & 0xfffffff0;
    phead = phead | 0x00000001;
    udp_data.extend_from_slice(&phead.to_be_bytes());
    for k in 0..16 {
      udp_data.push(0);
      udp_data.push(0);
      udp_data.push(0);
      udp_data.push(0);
    }
    self.socket.send(udp_data.as_slice()).unwrap();
    println!("[IPBus::get_status => message {:?} sent!", udp_data);
    let (number_of_bytes, _) = self.socket.recv_from(&mut self.buffer).unwrap();
    println!("[IPBus::get_status] => data received!");
    println!("[IPBus::get_status] => buffer {:?}", self. buffer);
  }

  fn create_packetheader(&mut self) -> u32 {
    // we use this to switch the byteorder
    let pid       = self.get_next_tid();
    let pid_bytes = pid.to_be_bytes(); // technically tid is pid here
    //println!("[create_packetheader] => Will use packet ID {pid}");
    let pid_be0   = (pid_bytes[1] as u32) << 16;
    let pid_be1   = (pid_bytes[0] as u32) << 8;
    let header = (0x2 << 28) as u32
               | (0x0 << 24) as u32
               //| (pid_be  << 8) as u32
               | pid_be0
               | pid_be1
               | (0xf << 4) as u32
               | 0x0 as u32; // 0 means control packet, we will 
                             // only use control packets in GAPS
    header
  }

  fn create_transactionheader(&self, nwords : u8) -> u32 {
    /// FIXME - for now, let's have transaction
    /// id always to be 0
    let header = (0x2 << 28) as u32
               | (0x0 << 24) as u32
               | (0x0 << 20) as u32
               | (0x0 << 16) as u32
               | (nwords as u32) << 8
               | (self.packet_type.to_u8() & 0xf << 4) as u32
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
    // get a new transaction id
    //let tid = self.get_next_tid();
    let mut udp_data = Vec::<u8>::new();

    let pheader = self.create_packetheader();
    let nwords  = data.len() as u8;
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
    //// this will silently overflow, but 
    //// if the message is that long, then 
    //// most likely there will be a 
    //// problem elsewhere, so we 
    //// don't care
    //let size      = data.len() as u8;
    //let packet_id = 0u8;
    //// we go byte-by-byte here
    //let mut udp_data = Vec::<u8>::from([
    //  // Transaction Header
    //  0x20 as u8, // Protocol version & RSVD
    //  //0x00 as u8, // Transaction ID (0 or bug)
    //  //0x00 as u8, // Transaction ID (0 or bug)
    //  tid.1, // Transaction ID (0 or bug)
    //  tid.0, // Transaction ID (0 or bug)
    //  0xf0 as u8, // Packet order & packet_type
    //  // Packet Header
    //  //
    //  // FIXME - in the original python script, 
    //  // the 0xf0 is a 0xf00, but this does not
    //  // make any sense in my eyes...
    //  (0x20 as u8 | ((packet_id & 0xf0 as u8) as u32 >> 8) as u8), // Protocol version & Packet ID MSB
    //  (packet_id & 0xff as u8), // Packet ID LSB,
    //  size, // Words
    //  (((self.packet_type as u8 & 0xf as u8) << 4) | 0xf as u8), // Packet_Type & Info code
    //  // Address
    //  ((addr & 0xff000000 as u32) >> 24) as u8,
  
    //  ((addr & 0x00ff0000 as u32) >> 16) as u8,
    //  ((addr & 0x0000ff00 as u32) >> 8)  as u8,
    //  (addr  & 0x000000ff as u32) as u8]);
  
    //if self.packet_type    == IPBusPacketType::Write
    //   || self.packet_type == IPBusPacketType::WriteNonIncrement {
    //  for i in 0..size as usize {
    //    udp_data.push (((data[i] & 0xff000000 as u32) >> 24) as u8);
    //    udp_data.push (((data[i] & 0x00ff0000 as u32) >> 16) as u8);
    //    udp_data.push (((data[i] & 0x0000ff00 as u32) >> 8)  as u8);
    //    udp_data.push ( (data[i] & 0x000000ff as u32)        as u8);
    //  }
    //}
    // fill up with 0s
    //while udp_data.len() != MT_MAX_PACKSIZE {
    //  udp_data.push(0);
    //}
    //println!("[encode_payload] UDPDATA {:?}", udp_data);
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
  fn decode_payload(&self,
                    verbose : bool)
    -> Result<Vec<u32>, IPBusError> {
    let mut pos  : usize = 0;
    let mut data = Vec::<u32>::new();
    let buffer   = self.buffer.to_vec();
//    let mut ipbus_version = message[0] >> 4;
//    let id            = (((message[4] & 0xf as u8) as u16) << 8) as u8 | message[5];
//    let size          = message[6];
//    let pt_val        = (message[7] & 0xf0 as u8) >> 4;
//    let info_code     = message[7] & 0xf as u8;
//    let mut data      = Vec::<u32>::new(); //[None]*size
//
    
    let pheader  = parse_u32_be(&buffer, &mut pos);
    let theader  = parse_u32_be(&buffer, &mut pos);
    let pid      = ((0x00ff0000 & pheader) >> 16) as u16;
    let size     = ((0x0000ff00 & theader) >> 8) as u16;
    let ptype    = ((0x000000f0 & theader) >> 4) as u8;
    if pid != self.expected_tid {
      error!("Invalid transaction ID. Expected {}, received {}", self.expected_tid, pid);
      return Err(IPBusError::InvalidTransactionID);
    }
    let packet_type = IPBusPacketType::from_u8(ptype);
    //println!("PID, SIZE, PTYPE : {} {} {}", pid, size, ptype);
    //println!("[decode_payload] BUFFER {:?}", buffer);
    //// Response
    //let ipbus_version = self.buffer[0] >> 4;
    //// re-assemble the transaction ID
    //let tid1          = self.buffer[1];
    //let tid0          = self.buffer[2];
    //let tid           = Self::assemble_tid((tid0,tid1));
    //let id            = (((self.buffer[4] & 0xf as u8) as u16) << 8) as u8 | self.buffer[5];
    //let size          = self.buffer[6];
    //let pt_val        = (self.buffer[7] & 0xf0 as u8) >> 4;
    //let info_code     = self.buffer[7] & 0xf as u8;
    //let mut data      = Vec::<u32>::new(); //[None]*size
  
    //let packet_type = IPBusPacketType::from_u8(pt_val);
    //if packet_type == IPBusPacketType::Unknown {
    //  return Err(IPBusError::DecodingFailed);
    //}
    //// Read
    //println!("[decode_payload] => PacketType {}", packet_type); 
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
    //if verbose { 
    //  println!("Decoding IPBus Packet:");
    //  println!(" >> Msg            : {:?}", self.buffer);
    //  println!(" >> IPBus version  : {}", ipbus_version);
    //  println!(" >> Transaction ID : {}", tid);
    //  println!(" >> ID             : {}", id);
    //  println!(" >> Size           : {}", size);
    //  println!(" >> Type           : {:?}", packet_type);
    //  println!(" >> Info           : {}", info_code);
    //  println!(" >> data           : {:?}", data);
    //}
    Ok(data)
  }

  pub fn read(&mut self, addr   : u32, verify_tid : bool) 
    -> Result<u32, Box<dyn Error>> {
    let send_data = Vec::<u32>::from([0]);
    let message   = self.encode_payload(addr, &send_data);
    //println!("[IPBus::read => messasge {:?}", message);
    self.socket.send(message.as_slice())?;
    //println!("[IPBus::read => message sent!");
    let mut data = Vec::<u32>::new();
    loop {
      let (number_of_bytes, _) = self.socket.recv_from(&mut self.buffer)?;
      //println!("[IPBus::read] => Received {} bytes from master trigger! Message {:?}", number_of_bytes, self.buffer);
      // this one can actually succeed, but return an emtpy vector
      match self.decode_payload(false) { 
        Err(err) => {
          if err == IPBusError::InvalidTransactionID {
            println!("--> invalid transaction id, trying again");
            if verify_tid {
              continue;
            } else {
              break;
            }
          } else {
            println!("[IPBus::read] Received error {err}");
          }
        }
        Ok(_data) => {
          data = _data;
          break;
        }
      } 
    }
    if data.len() == 0 {
      println!("[IPBus::read] Data has size 0");
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
    let message : Vec<u8>;
    if increment_addr {
      self.packet_type = IPBusPacketType::Read;
    } else {
      self.packet_type = IPBusPacketType::ReadNonIncrement;
    }
    // FIXME - we assume nwords != 1
    message = self.encode_payload(addr, &send_data);
    //println!("Sending message ...");
    self.socket.send(message.as_slice())?;
    //println!("... done");
    //let (number_of_bytes, _) = self.socket.recv_from(&mut self.buffer)?;
    //trace!("Received {} bytes from master trigger", number_of_bytes);
    let mut data = Vec::<u32>::new();
    loop {
      let (number_of_bytes, _) = self.socket.recv_from(&mut self.buffer)?;
      //println!("[read_multiple] Received {} bytes from master trigger. Buffer {:?}", number_of_bytes, self.buffer);
      // this one can actually succeed, but return an emtpy vector
      match self.decode_payload(false) { 
        Err(err) => {
          error!("Got error {err}!");
          if err == IPBusError::InvalidTransactionID {
            println!("--> invalid transaction id, trying again");
            if verify_tid {
              continue;
            } else {
              break;
            }
          }
        
        }
        Ok(_data) => {
          data = _data;
          //println!("... break... ");
          break;
        }
      } 
    }
    if data.len() == 0 {
      error!("Received empty data!");
      return Err(Box::new(IPBusError::DecodingFailed));
    }
    //if data.len() < nwords {
    //  error!("Received data of size {}, but was expecting {}!", data.len(), nwords);
    //  return Err(Box::new(IPBusError::DecodingFailed));
    //}
    //println!("returning");
    Ok(data)
  }
  
  pub fn write(&mut self,
               addr   : u32,
               data   : u32) 
    -> Result<(), Box<dyn Error>> {
    let send_data = Vec::<u32>::from([data]);
    self.packet_type = IPBusPacketType::Write;
    //println!("1tid {}", self.tid);
    let message = self.encode_payload(addr, &send_data);
    //println!("mess {:?}", message);
    //println!("2tid {}", self.tid);
    //println!("Sending...");
    self.socket.send(message.as_slice())?;
    //println!("tid {}", self.tid);
    //println!("..done...receiving..");
    let (number_of_bytes, _) = self.socket.recv_from(&mut self.buffer)?;
    //println!("Received {} bytes from master trigger", number_of_bytes);
    Ok(())
  }

}

impl fmt::Display for IPBus {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr  = String::from("<IPBus:");
    repr         += &(format!("  tid : {}>", self.tid)); 
    write!(f, "{}", repr)
  }
}

