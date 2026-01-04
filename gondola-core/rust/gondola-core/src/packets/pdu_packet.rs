// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct PduChannel {
  vpower_acc  : i64,
  vbus        : i32,
  vsense      : i32,
  vbus_avg    : i32,
  vsense_avg  : i32,
  vpower      : i32
}

impl PduChannel {
  pub fn new() -> Self {
    Self {
      vpower_acc  : 0,
      vbus        : 0,
      vsense      : 0,
      vbus_avg    : 0,
      vsense_avg  : 0,
      vpower      : 0
    }
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl PduChannel {

  #[getter]
  fn get_vpower_acc(&self) -> i64  {
    self.vpower_acc 
  } 
  #[getter] 
  fn get_vbus(&self) -> i32 {
    self.vbus 
  }
  #[getter] 
  fn get_vsense(&self) -> i32 {
    self.vsense 
  }
  #[getter]
  fn get_vbus_avg(&self) -> i32 {
    self.vbus_avg 
  }
  #[getter]
  fn get_vsense_avg(&self) -> i32 {
    self.vsense_avg 
  }
  #[getter]
  fn get_vpower(&self) -> i32 {
    self.vpower
  }
}

#[cfg_attr(feature="pybindings", pyclass)]
#[derive(Debug, Copy, Clone)]
pub struct Pac1934 {
  ctrl            : i32,
  acc_count       : i32,
  channels        : [PduChannel; 4],
  channel_dis     : i32,
  neg_pwr         : i32,
  slow            : i32,
  ctrl_act        : i32,
  channel_dis_act : i32,
  neg_pwr_act     : i32,
  ctrl_lat        : i32,
  channel_dis_lat : i32,
  neg_pwr_lat     : i32,
  pid             : i32,
  mid             : i32,
  rev             : i32,
}

impl Pac1934 {

  pub fn new() -> Self {
    Self {
      ctrl            : 0,
      acc_count       : 0,
      channels        : [PduChannel::new(); 4],
      channel_dis     : 0,
      neg_pwr         : 0,
      slow            : 0,
      ctrl_act        : 0,
      channel_dis_act : 0,
      neg_pwr_act     : 0,
      ctrl_lat        : 0,
      channel_dis_lat : 0,
      neg_pwr_lat     : 0,
      pid             : 0,
      mid             : 0,
      rev             : 0,
    }
  }
  
  pub fn from_bytestream(stream: &Vec<u8>,
                         pos: &mut usize)
    -> Result<Self, SerializationError> {
    let mut pac       = Pac1934::new();
    // size of a packet is 88 
    if stream.len() < 88 {
      return Err(SerializationError::StreamTooShort);
    }
    pac.ctrl         = parse_u8(stream, pos) as i32; 
    let acc_count_a  = parse_u8(stream, pos) as u64;
    let acc_count_b  = parse_u8(stream, pos) as u64; 
    let acc_count_c  = parse_u8(stream, pos) as u64; 
    pac.acc_count    = (acc_count_a << 16 | acc_count_b << 8 | acc_count_c) as i32;
    for ch in &mut pac.channels {
      let vpower_acc_a = parse_u8(stream, pos) as u64;
      let vpower_acc_b = parse_u8(stream, pos) as u64;
      let vpower_acc_c = parse_u8(stream, pos) as u64;
      let vpower_acc_d = parse_u8(stream, pos) as u64;
      let vpower_acc_e = parse_u8(stream, pos) as u64;
      let vpower_acc_f = parse_u8(stream, pos) as u64;
      ch.vpower_acc    = (vpower_acc_a << 40 | vpower_acc_b << 32 | vpower_acc_c << 24 | vpower_acc_d << 16 | vpower_acc_e << 8 | vpower_acc_f) as i64;
      let vbus_a       = parse_u8(stream, pos) as u64;
      let vbus_b       = parse_u8(stream, pos) as u64;
      ch.vbus          = (vbus_a << 8 | vbus_b) as i32;
      let vsense_a     = parse_u8(stream, pos) as u64;
      let vsense_b     = parse_u8(stream, pos) as u64;
      ch.vsense        = (vsense_a << 8 | vsense_b) as i32; 
      let vbus_avg_a   = parse_u8(stream, pos) as u64;
      let vbus_avg_b   = parse_u8(stream, pos) as u64;
      ch.vbus_avg      = (vbus_avg_a << 8 | vbus_avg_b) as i32;
      let vsense_avg_a = parse_u8(stream, pos) as u64;
      let vsense_avg_b = parse_u8(stream, pos) as u64;
      ch.vsense_avg    = (vsense_avg_a | vsense_avg_b) as i32;
      let vpower_a     = parse_u8(stream, pos) as u64;
      let vpower_b     = parse_u8(stream, pos) as u64;
      let vpower_c     = parse_u8(stream, pos) as u64; 
      let vpower_d     = parse_u8(stream, pos) as u64;
      ch.vpower        = (vpower_a << 24 | vpower_b << 16 | vpower_c << 8 | vpower_d) as i32;
    }
    pac.neg_pwr         = parse_u8(stream, pos) as i32;
    pac.slow            = parse_u8(stream, pos) as i32;
    pac.ctrl_act        = parse_u8(stream, pos) as i32;
    pac.channel_dis_act = parse_u8(stream, pos) as i32;
    pac.neg_pwr_act     = parse_u8(stream, pos) as i32;
    pac.ctrl_lat        = parse_u8(stream, pos) as i32;
    pac.channel_dis_lat = parse_u8(stream, pos) as i32;
    pac.neg_pwr_lat     = parse_u8(stream, pos) as i32;
    pac.pid             = parse_u8(stream, pos) as i32;
    pac.mid             = parse_u8(stream, pos) as i32;
    pac.rev             = parse_u8(stream, pos) as i32;
    Ok(pac)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl Pac1934 {

  #[getter]
  fn get_ctrl(&self) -> i32 {
    self.ctrl 
  }

  #[getter]
  fn get_acc_count(&self) -> i32 {
    self.acc_count 
  }

  #[getter]
  fn get_channel_dis(&self) -> i32 {
    self.channel_dis 
  }

  #[getter]
  fn get_neg_pwr(&self) -> i32 {
    self.neg_pwr 
  }

  #[getter]
  fn get_slow(&self) -> i32 {
    self.slow 
  }

  #[getter]
  fn get_ctrl_act(&self) -> i32 {
    self.ctrl_act 
  }

  #[getter]
  fn get_channel_dis_act(&self) -> i32 {
    self.channel_dis_act 
  }

  #[getter]
  fn neg_pwr_act(&self) -> i32 {
    self.neg_pwr_act 
  }

  #[getter]
  fn get_ctrl_lat(&self) -> i32 {
    self.ctrl_lat 
  }

  #[getter]
  fn get_channel_dis_lat(&self) -> i32 {
    self.channel_dis_lat
  }

  #[getter]
  fn get_neg_pwr_lat(&self) -> i32 {
    self.neg_pwr_lat 
  }

  #[getter]
  fn get_pid(&self) -> i32 {
    self.pid 
  }

  #[getter] 
  fn get_mid(&self) -> i32 {
    self.mid 
  }
  //      channels        : [PduChannel::new(); 4],
}


#[cfg_attr(feature="pybindings", pyclass)]
#[derive(Debug, Copy, Clone)]
pub struct PduHKPacket {
  pub header           : TelemetryPacketHeader,
  pub pdu_type         : i32,
  pub pdu_id           : i32,
  pub ads7828_voltages : [i32;8],
  pub pacs             : [Pac1934;2],
  pub vbat             : i32,
  pub pdu_count        : i32,
  pub error            : i32,
}

impl PduHKPacket {
  
  pub fn new() -> Self {
    Self {
      header           : TelemetryPacketHeader::new(),
      pdu_type         : 0,
      pdu_id           : 0,
      ads7828_voltages : [0;8],
      pacs             : [Pac1934::new();2],
      vbat             : 0,
      pdu_count        : 0,
      error            : 0,
    }
  }

  pub fn from_bytestream(stream: &Vec<u8>,
                         pos: &mut usize)
    -> Result<Self, SerializationError> {
    let mut pdu       = PduHKPacket::new();
    // size of a packet is 218 
    if stream.len() < 218 {
      return Err(SerializationError::StreamTooShort);
    }
    pdu.header   = TelemetryPacketHeader::from_bytestream(stream, pos)?;
    pdu.pdu_type = parse_u8(stream, pos) as i32;
    pdu.pdu_id   = parse_u8(stream, pos) as i32;
    //         pdu_type = b(i); i += 1;
//         pdu_id = b(i); i += 1;

    Ok(pdu)
  }
}

//impl Serialization for PduHKPacket {
//}

#[cfg(feature="pybindings")]
#[pymethods]
impl PduHKPacket {

  #[getter]
  fn get_header(&self) -> TelemetryPacketHeader {
    self.header.clone()
  }

  #[getter]
  fn get_pdu_type(&self) -> i32 {
    self.pdu_type 
  }

  #[getter]
  fn get_pdu_id(&self) -> i32 {
    self.pdu_id 
  }

  #[getter]
  fn get_vbat(&self) -> i32 {
    self.vbat 
  }

  #[getter]
  fn get_pdu_count(&self) -> i32 {
    self.pdu_count 
  }

  #[getter] 
  fn get_error(&self) -> i32 {
    self.error
  }
  //ads7828_voltages : [i32;8],
  //pacs             : [Pac1934;2],


}

//
//namespace pdu
//{
//   struct pac_1934
//   {
//      int ctrl;
//      int acc_count;
//      struct channel
//      {
//         int64_t vpower_acc;
//         int vbus;
//         int vsense;
//         int vbus_avg;
//         int vsense_avg;
//         int vpower;
//      };
//      std::array<channel, 4> channels;
//      int channel_dis;
//      int neg_pwr;
//      int slow;
//      int ctrl_act;
//      int channel_dis_act;
//      int neg_pwr_act;
//      int ctrl_lat;
//      int channel_dis_lat;
//      int neg_pwr_lat;
//      int pid;
//      int mid;
//      int rev;
//
//      template <typename T>
//      int parse(T bytes, int i)
//      {
//         const int expected_size = 88;
//         if((bytes.size() - i) < expected_size)
//            return -1;
//         int i_start = i;
//         auto b = [&bytes](int i){return static_cast<uint64_t>(bytes[i]);};
//         ctrl = b(i); i += 1;
//         acc_count = (b(i) << 16) | (b(i+1) << 8) | b(i+2); i += 3;
//         for(auto& ch : channels) {ch.vpower_acc = (b(i) << 40) | (b(i+1) << 32) | (b(i+2) << 24) | (b(i+3) << 16) | (b(i+4) << 8) | b(i+5); i += 6;}
//         for(auto& ch : channels) {ch.vbus = (b(i) << 8) | b(i+1); i += 2;}
//         for(auto& ch : channels) {ch.vsense = (b(i) << 8) | b(i+1); i += 2;}
//         for(auto& ch : channels) {ch.vbus_avg = (b(i) << 8) | b(i+1); i += 2;}
//         for(auto& ch : channels) {ch.vsense_avg = (b(i) << 8) | b(i+1); i += 2;}
//         for(auto& ch : channels) {ch.vpower = (b(i) << 24) | (b(i+1) << 16) | (b(i+2) << 8) | b(i+3); i += 4;}
//         channel_dis = b(i); i += 1;
//         neg_pwr = b(i); i += 1;
//         slow = b(i); i += 1;
//         ctrl_act = b(i); i += 1;
//         channel_dis_act = b(i); i += 1;
//         neg_pwr_act = b(i); i += 1;
//         ctrl_lat = b(i); i += 1;
//         channel_dis_lat = b(i); i += 1;
//         neg_pwr_lat = b(i); i += 1;
//         pid = b(i); i += 1;
//         mid = b(i); i += 1;
//         rev = b(i); i += 1;
//         assert((i - i_start) == expected_size);
//         return expected_size;
//
//      }
//   };
//   struct hkp_packet
//   {
//      bfsw::header header;
//      int pdu_type;
//      int pdu_id;
//      std::array<int,8> ads7828_voltages;
//      std::array<pac_1934,2> pacs;
//      int vbat;
//      int pdu_count;
//      int error;
//      template <typename T> int unpack(const T& bytes, size_t i)
//      {
//         const int expected_size = 218;
//         if((bytes.size() - i) < expected_size)
//            return -1;
//         size_t i_start {i};
//         int rc;
//         rc = header.unpack(bytes, i);
//         if(rc < 0)
//            return -100 + rc;
//         else
//            i += rc;
//         auto b = [&bytes](int i){return static_cast<uint64_t>(bytes[i]);};
//         pdu_type = b(i); i += 1;
//         pdu_id = b(i); i += 1;
//         for(auto& voltage : ads7828_voltages)
//         {
//            voltage = (b(i) << 8) | b(i+1); 
//            i += 2;
//         }
//         for(auto& pac : pacs)
//         {
//            int rc = pac.parse(bytes,i);
//            if(rc < 0)
//            {
//               spdlog::warn("hkp_packet::parse(): failed while parsing pac1934 data. rc = {}", rc);
//               return -3;
//            }
//            else
//            {
//               i += rc;
//            }
//         }
//         //NOTE: old code had vbat bytes little endian, but everything else is big endian.
//         //from my testing with pdu, i know that this value wasn't reading properly
//         //i assumed it was an issue with the board (wrong reistors or something)
//         //but maybe it was just an endianness error.
//         //vbat = (b(i) << 8) | b(i+1); i += 2;
//         vbat = (b(i+1) << 8) | b(i); i += 2;
//         i += 2;
//         pdu_count = b(i); i += 1;
//         error = b(i); i += 1;
//	 i += 5; //ACK\r\n
//         assert((i - i_start) == expected_size);
//         return expected_size;
//      }
//      void print()
//      {
//         fmt::print("pdu_type: {}\n", pdu_type);
//         fmt::print("pdu_id: {}\n", pdu_id);
//
//         for(auto& pac : pacs)
//         {
//            fmt::print("pac.pid: {}\n", pac.pid);
//            fmt::print("pac.mid: {}\n", pac.mid);
//            fmt::print("pac.rev: {}\n", pac.rev);
//         }
//         fmt::print("pdu_count: {}\n", pdu_count);
//      }
//   };


