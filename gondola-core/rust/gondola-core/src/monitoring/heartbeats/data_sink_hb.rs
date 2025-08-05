//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

use std::fmt;
#[cfg(feature = "random")]  
use crate::random::FromRandom;
#[cfg(feature = "random")]  
use rand::Rng;

use crate::io::serialization::Serialization;
use crate::io::parsers::{
  parse_u64,
};

use crate::errors::SerializationError;

use crate::packets::{
  TofPackable,
  TofPacketType,
};

#[cfg(feature="pybindings")]
use pyo3::prelude::*;

#[cfg(feature="pybindings")]
use pyo3::exceptions::PyIOError;

#[cfg(feature="pybindings")]
use crate::packets::TofPacket;

#[cfg(feature="pybindings")]
use crate::impl_pythonize_display;

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct DataSinkHB {

  /// mission elapsed time in seconds
  pub met                : u64,
  pub n_packets_sent     : u64,
  pub n_packets_incoming : u64,
  /// bytes written to disk
  pub n_bytes_written    : u64,
  /// event id check - missing event ids
  pub n_evid_missing     : u64,
  /// event id check - chunksize
  pub n_evid_chunksize   : u64,
  /// length of incoming buffer for 
  /// the thread
  /// check for missing event ids
  pub evid_missing       : u64,
  /// probe size for missing event id check
  pub evid_check_len     : u64,
  /// number of packets written to disk
  pub n_pack_write_disk  : u64,
  /// length of the incoming channel, which 
  /// is basically packets queued to be sent
  pub incoming_ch_len    : u64,
}

impl DataSinkHB {

  pub fn new() -> Self {
    Self {
      met                : 0,
      n_packets_sent     : 0,
      n_packets_incoming : 0,
      n_bytes_written    : 0,
      n_evid_missing     : 0,
      n_evid_chunksize   : 0,
      evid_missing       : 0,
      evid_check_len     : 0,
      n_pack_write_disk  : 0,
      incoming_ch_len    : 0,
    }
  }

  pub fn get_sent_packet_rate(&self) -> f64 {
    if self.met == 0 {
      return 0.0;
    }
    self.n_packets_sent as f64 /  self.met as f64
  }

  pub fn get_mbytes_to_disk_per_sec(&self) -> f64 {
    if self.met == 0 {
      return 0.0;
    }
    self.n_bytes_written as f64/(1e6 * self.met as f64)
  }
}

impl Default for DataSinkHB {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl DataSinkHB {

  #[new]
  fn new_py() -> Self {
    Self::new()
  }
  
  /// Mission elapsed time
  #[getter]
  fn get_met(&self) -> PyResult<u64> {
    Ok(self.met)
  }
  
  #[getter]
  fn get_n_packets_sent(&self) -> PyResult<u64> {
    Ok(self.n_packets_sent)
  }
  
  #[getter]
  fn get_n_packets_incoming(&self) -> PyResult<u64> {
    Ok(self.n_packets_incoming)
  }
  
  #[getter]
  fn get_n_bytes_written(&self) -> PyResult<u64> {
    Ok(self.n_bytes_written)
  }
  #[getter]
  fn get_n_evid_chunksize(&self) -> PyResult<u64> {
    Ok(self.n_evid_chunksize)
  }
  #[getter]
  fn get_evid_missing(&self) -> PyResult<u64> {
    Ok(self.evid_missing)
  }
  
  #[getter]
  fn get_evid_check_len(&self) -> PyResult<u64> {
    Ok(self.evid_check_len)
  }
  
  #[getter]
  fn get_n_pack_write_disk(&self) -> PyResult<u64> {
    Ok(self.n_pack_write_disk)
  }
  
  #[staticmethod]
  #[pyo3(name="from_tofpacket")]
  fn from_tofpacket_py(packet : &TofPacket) -> PyResult<Self> {
    match packet.unpack::<DataSinkHB>() {
      Ok(hb) => {
        return Ok(hb);
      }
      Err(err) => {
        let err_msg = format!("Unable to unpack TofPacket! Is this really a DataSinkHeartbeat? {err}");
        return Err(PyIOError::new_err(err_msg));
      }
    }
  }
}

impl TofPackable for DataSinkHB {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::DataSinkHB;
}

impl Serialization for DataSinkHB {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;
  const SIZE : usize = 84; 
  
  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    Self::verify_fixed(stream, pos)?;  
    let mut hb            = Self::new();
    hb.met                = parse_u64(stream, pos);
    hb.n_packets_sent     = parse_u64(stream, pos);
    hb.n_packets_incoming = parse_u64(stream, pos);
    hb.n_bytes_written    = parse_u64(stream, pos);
    hb.n_evid_missing     = parse_u64(stream, pos);
    hb.n_evid_chunksize   = parse_u64(stream, pos);
    hb.evid_missing       = parse_u64(stream, pos);
    hb.evid_check_len     = parse_u64(stream, pos);
    hb.n_pack_write_disk  = parse_u64(stream, pos);
    hb.incoming_ch_len    = parse_u64(stream, pos);
    *pos += 2;
    Ok(hb)
  }
  
  fn to_bytestream(&self) -> Vec<u8> {
    let mut bs = Vec::<u8>::with_capacity(Self::SIZE);
    bs.extend_from_slice(&Self::HEAD.to_le_bytes());
    bs.extend_from_slice(&self.met.to_le_bytes());
    bs.extend_from_slice(&self.n_packets_sent.to_le_bytes());
    bs.extend_from_slice(&self.n_packets_incoming.to_le_bytes());
    bs.extend_from_slice(&self.n_bytes_written.to_le_bytes());
    bs.extend_from_slice(&self.n_evid_missing.to_le_bytes());
    bs.extend_from_slice(&self.n_evid_chunksize.to_le_bytes());
    bs.extend_from_slice(&self.evid_missing     .to_le_bytes() );
    bs.extend_from_slice(&self.evid_check_len   .to_le_bytes() );
    bs.extend_from_slice(&self.n_pack_write_disk.to_le_bytes() );
    bs.extend_from_slice(&self.incoming_ch_len.to_le_bytes());
    bs.extend_from_slice(&Self::TAIL.to_le_bytes());
    bs
  }
}

#[cfg(feature = "random")]
impl FromRandom for DataSinkHB {
  fn from_random() -> Self {
    let mut rng            = rand::rng();
    Self {
      met                : rng.random::<u64>(),
      n_packets_sent     : rng.random::<u64>(),
      n_packets_incoming : rng.random::<u64>(),
      n_bytes_written    : rng.random::<u64>(),
      n_evid_missing     : rng.random::<u64>(),
      n_evid_chunksize   : rng.random::<u64>(),
      evid_missing       : rng.random::<u64>(),
      evid_check_len     : rng.random::<u64>(),
      n_pack_write_disk  : rng.random::<u64>(),
      incoming_ch_len    : rng.random::<u64>()
    }
  }
}

impl fmt::Display for DataSinkHB {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<DataSinkHB");
    repr += &(format!("\n \u{1F98B} \u{1F98B} \u{1F98B} \u{1F98B} \u{1F98B} DATA SENDER HEARTBEAT \u{1F98B} \u{1F98B} \u{1F98B} \u{1F98B} \u{1F98B}"));
    repr += &(format!("\n Sent {} TofPackets! (packet rate {:.2}/s)", self.n_packets_sent , self.get_sent_packet_rate()));
    repr += &(format!("\n Writing events to disk: {} packets written, data write rate {:.2} MB/sec", self.n_pack_write_disk, self.get_mbytes_to_disk_per_sec()));
    repr += &(format!("\n Missing evid analysis:  {} of {} a chunk of events missing ({:.2}%)", self.evid_missing, self.evid_check_len, 100.0*(self.evid_missing as f64/self.evid_check_len as f64)));
    repr += &(format!("\n Incoming channel length: {}", self.incoming_ch_len));
    repr += &(format!("\n \u{1F98B} \u{1F98B} \u{1F98B} \u{1F98B} \u{1F98B} END HEARTBEAT \u{1F98B} \u{1F98B} \u{1F98B} \u{1F98B} \u{1F98B}"));
    write!(f, "{}", repr)
  }
}

#[cfg(feature="pybindings")]
impl_pythonize_display!(DataSinkHB, |s: &DataSinkHB | s.to_string());

