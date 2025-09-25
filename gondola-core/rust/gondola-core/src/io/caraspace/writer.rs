// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Write CRFrames to disk.
///
/// Operates sequentially, frames can 
/// be added one at a time, then will
/// be synced to disk.
#[cfg_attr(feature="pybindings", pyclass)]
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
  /// it can also take a timestamp, in case we 
  /// don't want to use the current time when the 
  /// file is written
  pub file_timestamp  : Option<String>,
}

impl CRWriter {

  /// Instantiate a new PacketWriter 
  ///
  /// # Arguments
  ///
  /// * file_path       : Path to store the file under
  /// * run_id          : Run ID for this file (will be written in filename)
  /// * subrun_id       : Sub-Run ID for this file (will be written in filename. 
  ///                     If None, a generic "0" will be used
  /// * timestamp       : The writer will add an automatic timestamp to the current file
  ///                     based on the current time. This option allows to overwrite 
  ///                     that behaviour
  pub fn new(mut file_path : String, run_id : u32, subrun_id : Option<u64>, timestamp : Option<String>) -> Self {
    //let filename = file_prefix.clone() + "_0.tof.gaps";
    let file : File;
    let file_name : String;
    if !file_path.ends_with("/") {
      file_path += "/";
    }
    let filename : String;
    if let Some(subrun) = subrun_id {
      filename = format!("{}{}", file_path, get_runfilename(run_id, subrun, None, timestamp));
    } else {
      filename = format!("{}{}", file_path, get_runfilename(run_id, 0, None, timestamp));
    }
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
      file_timestamp   : None,
    }
  }

  pub fn get_file(&self, timestamp : Option<String>) -> File { 
    let file : File;
    let filename = format!("{}{}", self.file_path, get_runfilename(self.run_id, self.file_id as u64, None, timestamp));
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
    let buffer = frame.to_bytestream();
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
        self.file = self.get_file(self.file_timestamp.clone());
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
    CRWriter::new(String::from(""),0,None, None)
  }
}

impl fmt::Display for CRWriter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<CRWriter:");
    repr += &(format!("\n  path: {}", self.file_path)); 
    repr += ">";
    write!(f, "{}", repr)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl CRWriter {
  
  #[new]
  #[pyo3(signature = (filename, run_id, subrun_id = None, timestamp = None))]
  fn new_py(filename : String, run_id : u32, subrun_id : Option<u64>, timestamp : Option<String>) -> Self {
    Self::new(filename, run_id, subrun_id, timestamp)
  }
 
  fn set_mbytes_per_file(&mut self, fsize : usize) {
    self.mbytes_per_file = fsize;
  }

  fn set_file_timestamp(&mut self, timestamp : String) {
    self.file_timestamp = Some(timestamp);
  }
  
  #[pyo3(name="add_frame")]
  fn add_frame_py(&mut self, frame : CRFrame) {
    self.add_frame(&frame);  
  }
}

#[cfg(feature="pybindings")]
pythonize_display!(CRWriter);

