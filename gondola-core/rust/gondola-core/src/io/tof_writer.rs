// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Write TofPackets to disk.
///
/// Operates sequentially, packets can 
/// be added one at a time, then will
/// be synced to disk.
pub struct TofPacketWriter {

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
  /// add timestamps to filenames
  pub file_type       : FileType,
  pub file_name       : String,

  file_id             : usize,
  /// internal packet counter, number of 
  /// packets which went through the writer
  n_packets           : usize,
  /// internal counter for bytes written in 
  /// this file
  file_nbytes_wr      : usize,
}

impl TofPacketWriter {

  /// Instantiate a new PacketWriter 
  ///
  /// # Arguments
  ///
  /// * file_prefix     : Prefix file with this string. A continuous number will get 
  ///                     appended to control the file size.
  /// * file_type       : control the behaviour of how the filename is
  ///                     assigned.
  pub fn new(mut file_path : String, file_type : FileType) -> Self {
    //let filename = file_prefix.clone() + "_0.tof.gaps";
    let file : File;
    let file_name : String;
    if !file_path.ends_with("/") {
      file_path += "/";
    }
    match file_type {
      FileType::Unknown => {
        let filename = file_path.clone() + "Data.tof.gaps";
        let path     = Path::new(&filename); 
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
        file_name = filename;
      }
      FileType::RunFile(runid) => {
        let filename = format!("{}{}", file_path, get_runfilename(runid, 0, None, None));
        let path     = Path::new(&filename); 
        println!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
        file_name = filename;
      }
      FileType::CalibrationFile(rbid) => {
        let filename = format!("{}{}", file_path, get_califilename(rbid, false));
        //let filename = file_path.clone() + &get_califilename(rbid,false);
        let path     = Path::new(&filename); 
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
        file_name = filename;
      }
      FileType::SummaryFile(ref fname) => {
        let filename = fname.replace(".tof.", ".tofsum.");
        let path     = Path::new(&filename);
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
        file_name = filename;
      }
    }
    Self {
      file,
      file_path        : file_path,
      pkts_per_file    : 0,
      mbytes_per_file  : 420,
      file_nbytes_wr   : 0,    
      file_type        : file_type,
      file_id          : 1,
      n_packets        : 0,
      file_name        : file_name,
    }
  }

  pub fn get_file(&self) -> File { 
    let file : File;
    match &self.file_type {
      FileType::Unknown => {
        let filename = self.file_path.clone() + "Data.tof.gaps";
        let path     = Path::new(&filename); 
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
      }
      FileType::RunFile(runid) => {
        let filename = format!("{}{}", self.file_path, get_runfilename(*runid, self.file_id as u64, None, None));
        //let filename = self.file_path.clone() + &get_runfilename(runid,self.file_id as u64, None);
        let path     = Path::new(&filename); 
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
      }
      FileType::CalibrationFile(rbid) => {
        //let filename = self.file_path.clone() + &get_califilename(rbid,false);
        let filename = format!("{}{}", self.file_path, get_califilename(*rbid, false));
        let path     = Path::new(&filename); 
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
      }
      FileType::SummaryFile(fname) => {
        let filename = fname.replace(".tof.", ".tofsum.");
        let path     = Path::new(&filename);
        info!("Writing to file {filename}");
        file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
      }
    }
    file
  }

  /// Induce serialization to disk for a TofPacket
  ///
  ///
  pub fn add_tof_packet(&mut self, packet : &TofPacket) {
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
        //let path  = Path::new(&filename);
        //println!("==> [TOFPACKETWRITER] Will start a new file {}", path.display());
        //self.file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
        //self.n_packets = 0;
        //self.file_id += 1;
      }
  debug!("TofPacket written!");
  }
}

impl Default for TofPacketWriter {
  fn default() -> TofPacketWriter {
    TofPacketWriter::new(String::from(""), FileType::Unknown)
  }
}

