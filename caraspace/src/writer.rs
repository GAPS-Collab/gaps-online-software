//! Writer for caraspace files. CRFrames can be 
//! added sequentially to a file
//!

//use std::fmt;
use chrono::{
  DateTime,
  Utc
};

use std::path::Path;
use std::fs::{
    File,
    OpenOptions
};
use std::io::Write;

use crate::frame::CRFrame;
use crate::serialization::CRSerializeable;

/// The TimeStamp format for Human readable timestamps
pub static HUMAN_TIMESTAMP_FORMAT : &str = "%y%m%d_%H%M%S%Z";

/// A standardized name for regular run files saved by
/// the liftof suite
///
/// # Arguments
///
/// * run    : run id (identifier)
/// * subrun : subrun id (identifier of file # within
///            the run
/// * rb_id  : in case this should be used on the rb, 
///            a rb id can be specified as well
pub fn get_runfilename(run : u32, subrun : u64, rb_id : Option<u8>) -> String {
  let ts = get_utc_timestamp();
  let fname : String;
  match rb_id {
    None => {
      fname = format!("Run{run}_{subrun}.{ts}.gaps");
    }
    Some(rbid) => {
      fname = format!("Run{run}_{subrun}.{ts}.RB{rbid:02}.gaps");
    }
  }
  fname
}

/// Get a human readable timestamp
pub fn get_utc_timestamp() -> String {
  let now: DateTime<Utc> = Utc::now();
  //let timestamp_str = now.format("%Y_%m_%d-%H_%M_%S").to_string();
  let timestamp_str = now.format(HUMAN_TIMESTAMP_FORMAT).to_string();
  timestamp_str
}

/// Write CRFrames to disk.
///
/// Operates sequentially, frames can 
/// be added one at a time, then will
/// be synced to disk.
pub struct CRWriter {

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
  pub run_id          : u32,
  file_id             : usize,
  /// internal packet counter, number of 
  /// packets which went through the writer
  n_packets           : usize,
  /// internal counter for bytes written in 
  /// this file
  file_nbytes_wr      : usize,
}

impl CRWriter {

  /// Instantiate a new PacketWriter 
  ///
  /// # Arguments
  ///
  /// * file_prefix     : Prefix file with this string. A continuous number will get 
  ///                     appended to control the file size.
  pub fn new(mut file_path : String, run_id : u32) -> Self {
    //let filename = file_prefix.clone() + "_0.tof.gaps";
    let file : File;
    let file_name : String;
    if !file_path.ends_with("/") {
      file_path += "/";
    }
    let filename = format!("{}{}", file_path, get_runfilename(run_id, 0, None));
    let path     = Path::new(&filename); 
    println!("Writing to file {filename}");
    file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
    file_name = filename;
    Self {
      file,
      file_path        : file_path,
      pkts_per_file    : 0,
      mbytes_per_file  : 420,
      run_id           : run_id,
      file_nbytes_wr   : 0,    
      file_id          : 1,
      n_packets        : 0,
      file_name        : file_name,
    }
  }

  pub fn get_file(&self) -> File { 
    let file : File;
    let filename = format!("{}{}", self.file_path, get_runfilename(self.run_id, self.file_id as u64, None));
    //let filename = self.file_path.clone() + &get_runfilename(runid,self.file_id as u64, None);
    let path     = Path::new(&filename); 
    info!("Writing to file {filename}");
    file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
    file
  }

  /// Induce serialization to disk for a CRFrame
  ///
  ///
  pub fn add_frame(&mut self, frame : &CRFrame) {
    let buffer = frame.serialize();
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
  debug!("CRFrame written!");
  }
}

impl Default for CRWriter {
  fn default() -> Self {
    CRWriter::new(String::from(""),0)
  }
}

