// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Write TelemetryPackets to disk.
///
/// Operates sequentially, packets can 
/// be added one at a time, then will
/// be synced to disk.
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TelemetryPacketWriter {

  pub file            : File,
  /// location to store the file
  pub file_path       : String,
  /// The maximum number of packets 
  /// for a single file. Ater this 
  /// number is reached, a new 
  /// file is started.
  pub pkts_per_file   : usize,
  /// The maximum number of (Mega)bytes
  /// per file. After this a new file 
  /// is started
  pub mbytes_per_file : usize,
  pub file_name       : String,
  pub last_timestamp  : String,
  file_id             : usize,
  /// internal packet counter, number of 
  /// packets which went through the writer
  n_packets           : usize,
  /// internal counter for bytes written in 
  /// this file
  file_nbytes_wr      : usize,
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TelemetryPacketWriter {

#[new]
  fn new_py(filepath : String, packet : &TelemetryPacket) -> PyResult<Self> {
    let writer = Self::new(filepath, packet);
    Ok(writer)
  }

  #[pyo3(name="add_telemetry_packet")]
  pub fn add_telemetry_packet_py(&mut self, packet : &TelemetryPacket) {
    self.add_telemetry_packet(packet);
  }
}

impl TelemetryPacketWriter {

  /// Instantiate a new PacketWriter 
  ///
  /// # Arguments
  ///
  pub fn new(mut file_path : String, first_packet : &TelemetryPacket) -> Self {
    let file : File;
    let file_name : String;
    if !file_path.ends_with("/") {
      file_path += "/";
    }
    let utc_timestamp = Self::get_timestamp_from_packet(first_packet);
    let filename = file_path.clone() + "RAW" + &utc_timestamp + ".bin";
    let path     = Path::new(&filename); 
    info!("Writing to file {filename}");
    file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
    file_name = filename;
    Self {
      file,
      file_path        : file_path,
      pkts_per_file    : 0,
      mbytes_per_file  : 420,
      file_nbytes_wr   : 0,    
      file_id          : 1,
      n_packets        : 0,
      file_name        : file_name,
      last_timestamp   : utc_timestamp,
    }
  }

  /// Extract the gcutime from the packet and use it as 
  /// the timestamp for the next file to be written
  pub fn get_timestamp_from_packet(packet : &TelemetryPacket) -> String {
    let gcutime = packet.header.get_gcutime();
    get_utc_timestamp_from_unix(gcutime).unwrap_or(String::from("000000_000000")) 
  }

  pub fn get_file(&self) -> File { 
    let file : File;
    let filename = format!("{}RAW{}.bin", self.file_path, self.last_timestamp);
    //let filename = self.file_path.clone() + &get_runfilename(runid,self.file_id as u64, None);
    let path     = Path::new(&filename); 
    info!("Writing to file {filename}");
    file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
    file
  }

  /// Induce serialization to disk for a TofPacket
  ///
  ///
  pub fn add_telemetry_packet(&mut self, packet : &TelemetryPacket) {
    self.last_timestamp = Self::get_timestamp_from_packet(packet);
    let buffer = packet.to_bytestream();
    self.file_nbytes_wr += buffer.len();
    match self.file.write_all(buffer.as_slice()) {
      Err(err) => error!("Writing to file to path {} failed! {}", self.file_path, err),
      Ok(_)    => ()
    }
    self.n_packets += 1;
    let mut newfile = false;
    if self.pkts_per_file != 0 {
      if self.n_packets == self.pkts_per_file {
        newfile = true;
        self.n_packets = 0;
      }
    } else if self.mbytes_per_file != 0 {
      // multiply by mebibyte
      if self.file_nbytes_wr >= self.mbytes_per_file * 1_048_576 {
        newfile = true;
        self.file_nbytes_wr = 0;
      }
    }
    if newfile {
        //let filename = self.file_prefix.clone() + "_" + &self.file_id.to_string() + ".tof.gaps";
        match self.file.sync_all() {
          Err(err) => {
            error!("Unable to sync file to disc! {err}");
          },
          Ok(_) => ()
        }
        self.file = self.get_file();
        self.file_id += 1;
      }
    debug!("TelemetryPacket written!");
  }
}

