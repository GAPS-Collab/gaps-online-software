//! Caraspace provides a system to read files 
//! for the GAPS experiment comprising different sources, specifically 
//! files from the TOF system written to disk as well as telemetry file
//!
//! While written for the GAPS experiment, the caraspace library is 
//! designed in a form that it should be easily adaptable for other 
//! purposes.
//!
//! This file contains the source for CRReader, a device to read a number
//! of "caraspace" files from a given source.
//
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Read binaries written through the caraspace i/o system
///
/// The file needs to contain subsequent CRFrames.
#[derive(Debug)] // deliberatly don't have a default() method, reader should fail in that case
#[cfg_attr(feature="pybindings", pyclass)]
pub struct CRReader {
  /// Read from this file
  pub filenames        : Vec<String>,
  /// The position of the current worked on file 
  /// in the filenames vector
  pub file_idx       : usize,
  /// A simple BufReader for reading generic binary
  /// files
  file_reader          : BufReader<File>,
  /// Current (byte) position in the current file
  /// This gets reset when we switch to a new file
  cursor               : usize,
  /// Number of read packets
  n_packs_read         : usize,
  /// Number of skipped packets
  n_packs_skipped      : usize,
  /// Number of deserialization errors occured
  /// since the beginning of the file
  pub n_errors         : usize,
  /// Skip the first n packets
  pub skip_ahead       : usize,
  /// Stop reading after n packets
  pub stop_after       : usize,
  /// Geometry of each TOF paddle
  /// e.g. paddles 
  pub tof_paddles      : Arc<HashMap<u8,TofPaddle>>,
  /// Geometry of each tracker strip
  pub trk_strips       : Arc<HashMap<u32, TrackerStrip>>,
  /// Mask tracker strips 
  pub trk_masks        : Arc<HashMap<u32, TrackerStripMask>>,
  /// Tracker pedestal values
  pub trk_ped          : Arc<HashMap<u32, TrackerStripPedestal>>,
  /// Transfer functions for tracker (adc -> energy)
  pub trk_tf           : Arc<HashMap<u32, TrackerStripTransferFunction>>,
  /// Common noise data for tracker
  pub trk_cmn          : Arc<HashMap<u32, TrackerStripCmnNoise>>, 
  ///// did paddle loading work
  pub db_loaded        : bool,
  /// TRK calibration - convert to energy
  pub do_trk_calib     : bool,
  /// TRK subtract CMN 
  pub do_trk_cmn_noise : bool,
}


impl CRReader {

  /// Create a new CRReader
  ///
  /// # Arguments:
  ///   * filename_or_directory : Can be either the name of a single file, or a directory with 
  ///                             caraspace files in it.
  ///   
  pub fn new(filename_or_directory : String) -> Result<Self, io::Error> {
    #[cfg(feature="database")]
    let mut paddles = HashMap::<u8, TofPaddle>::new();
    #[cfg(not(feature="database"))]
    let paddles = HashMap::<u8, TofPaddle>::new();
    #[cfg(feature="database")]
    let mut strips  = HashMap::<u32, TrackerStrip>::with_capacity(11520);
    #[cfg(not(feature="database"))]
    let strips  = HashMap::<u32, TrackerStrip>::with_capacity(11520);
    let trk_mask    = HashMap::<u32, TrackerStripMask>::with_capacity(11520);
    let trk_ped     = HashMap::<u32, TrackerStripPedestal>::with_capacity(11520);
    let trk_tf      = HashMap::<u32, TrackerStripTransferFunction>::with_capacity(11520);
    let trk_cmn     = HashMap::<u32, TrackerStripCmnNoise>::with_capacity(11520);
    //let db_path       = env::var("DATABASE_URL").unwrap_or_else(|_| "".to_string());
    #[cfg(feature="database")]
    let mut db_loaded = false;
    #[cfg(not(feature="database"))]
    let db_loaded = false;
    #[cfg(feature="database")]
    match TofPaddle::all_as_dict() {
      Err(err) => {
        error!("Unable to retrieve paddle information from DB! {err}");
      }
      Ok(pdls) => {
        db_loaded = true;
        paddles   = pdls;         
      }
    }
    #[cfg(feature="database")]
    match TrackerStrip::all_as_dict() {
      Err(err) => {
        error!("Unable to retrieve tracker strip information from DB! {err}");
        // if strips and paddles do not work, something is utterly fisy
        db_loaded = false;
      }
      Ok(strips_) => {
        strips   = strips_;         
      }
    }
    // check the input argument and get the filelist
    let infiles   = list_path_contents_sorted(&filename_or_directory, None)?;
    if infiles.len() == 0 {
      error!("Unable to read files from {filename_or_directory}. Is this a valid path?");
      return Err(io::Error::new(io::ErrorKind::NotFound, "Unable to find given path!"))
    }
    let firstfile = infiles[0].clone(); 
    let file = OpenOptions::new().create(false).append(false).read(true).open(&firstfile).expect("Unable to open file {filename}");
    let packet_reader = Self { 
      filenames        : infiles,
      file_idx         : 0,
      //file_reader      : BufReader::new(file),
      // we exploit the fact here that a file is typically ~500Mb
      // (tof file only is 420MB)
      file_reader      : BufReader::with_capacity(500*1024*1024,file),
      cursor           : 0,
      n_packs_read     : 0,
      n_errors         : 0,
      skip_ahead       : 0,
      stop_after       : 0,
      n_packs_skipped  : 0,
      tof_paddles      : Arc::new(paddles),
      trk_strips       : Arc::new(strips),
      trk_masks        : Arc::new(trk_mask),
      trk_ped          : Arc::new(trk_ped),
      trk_tf           : Arc::new(trk_tf),
      trk_cmn          : Arc::new(trk_cmn),
      db_loaded        : db_loaded,
      do_trk_calib     : false,
      do_trk_cmn_noise : false,
    };
    Ok(packet_reader)
  } 
    
  #[cfg(feature="database")]
  pub fn set_tracker_calibrations_from_fnames(&mut self,
                                              mask        : Option<String>,
                                              pedestal    : Option<String>,
                                              transfer_fn : Option<String>,
                                              cmn_noise   : Option<String>) {
    // Tracker calibration parameters
    if let Some(maskname) = mask {
      match TrackerStripMask::as_dict_by_name(&maskname) {
        Err(err) => {
          error!("Unable to retrieve Trk strip mask information from DB! {err}");
          // if strips and paddles do not work, something is utterly fisy
          self.db_loaded = false;
        }
        Ok(strips_) => {
          self.trk_masks = Arc::new(strips_);
        }
      }
    }
    if let Some(pedname) = pedestal {
      match TrackerStripPedestal::as_dict_by_name(&pedname) {
        Err(err) => {
          error!("Unable to retrieve TRK pedestal information from DB! {err}");
          // if strips and paddles do not work, something is utterly fisy
          self.db_loaded = false;
        }
        Ok(strips_) => {
          self.trk_ped = Arc::new(strips_);
        }
      }
    }
    if let Some(trafoname) = transfer_fn { 
      match TrackerStripTransferFunction::as_dict_by_name(&trafoname) {
        Err(err) => {
          error!("Unable to retrieve TRK Transfer fn information from DB! {err}");
          // if strips and paddles do not work, something is utterly fisy
          self.db_loaded = false;
        }
        Ok(trafo_fns_) => {
          self.trk_tf = Arc::new(trafo_fns_);
          self.do_trk_calib = true;
        }
      }
    }
    if let Some(cmnname) = cmn_noise { 
      match TrackerStripCmnNoise::as_dict_by_name(&cmnname) {
        Err(err) => {
          error!("Unable to retrieve TRK common noise from DB! {err}");
          // if strips and paddles do not work, something is utterly fisy
          self.db_loaded = false;
        }
        Ok(cmn_) => {
          self.trk_cmn = Arc::new(cmn_);
          self.do_trk_cmn_noise = true;
        }
      }
    }
  }

  //  
  /// This is the file the current cursor is located 
  /// in and frames are currently read out from 
  pub fn get_current_filename(&self) -> Option<String> {
    // should only happen when it is empty
    if self.filenames.len() <= self.file_idx {
      return None;
    }
    Some(self.filenames[self.file_idx].clone())
  }
//  
//  
//  ///// Use the associated database to enrich paddle information
//  //fn add_paddleinfo(&self, event : &mut TofEventSummary) {
//  //  event.set_paddles(&self.paddles);
//  //}
//  
//
  /// Preview the number of frames in this reader
  pub fn count_frames(&mut self) -> usize {
    let _ = self.rewind();
    let mut nframes = 0usize;
    let mut buffer  = [0];
    let bar_template : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.blue/grey} {pos:>7}/{len:7}";
    let bar_style  = ProgressStyle::with_template(bar_template).expect("Unable to set progressbar style!");
    let bar = ProgressBar::new(self.filenames.len() as u64);
    bar.set_position(0);
    bar.set_message (String::from("Counting frames.."));
    bar.set_prefix  ("\u{2728}");
    bar.set_style   (bar_style);
    bar.set_position(self.file_idx as u64);
    loop {
      match self.file_reader.read_exact(&mut buffer) {
        Err(err) => {
          debug!("Unable to read from file! {err}");
          match self.prime_next_file() {
            None    => break,
            Some(_) => {
              bar.set_position(self.file_idx as u64);
              continue;
            }
          };
        }
        Ok(_) => {
          self.cursor += 1;
        }
      }
      if buffer[0] != 0xAA {
        continue;
      } else {
        match self.file_reader.read_exact(&mut buffer) {
          Err(err) => {
            debug!("Unable to read from file! {err}");
            match self.prime_next_file() {
              None    => break,
              Some(_) => {
                bar.set_position(self.file_idx as u64);
                continue;
              }
            };
          }
          Ok(_) => {
            self.cursor += 1;
          }
        }
        // check if the second byte of the header
        if buffer[0] != 0xAA { 
          continue;
        } else {
          // read the the size of the packet
          let mut buffer_psize = [0,0,0,0,0,0,0,0];
          match self.file_reader.read_exact(&mut buffer_psize) {
            Err(_err) => {
              match self.prime_next_file() {
                None    => break,
                Some(_) => {
                  bar.set_position(self.file_idx as u64);
                  continue;
                }
              }
            }
            Ok(_) => {
              self.cursor += 8;
            }
          }
          let vec_data = buffer_psize.to_vec();
          let size     = parse_u64(&vec_data, &mut 0);
          let mut temp_buffer = vec![0; size as usize];
          match self.file_reader.read_exact(&mut temp_buffer) { 
          //match self.file_reader.seek(SeekFrom::Current(size as i64)) {
          //match self.file_reader.seek_relative(size as i64) {
            Err(err) => {
              error!("Unable to read {size} bytes from {}! {err}", self.get_current_filename().unwrap());
              match self.prime_next_file() {
                None    => break,
                Some(_) => {
                  bar.set_position(self.file_idx as u64);
                  continue;
                }
              }
            }
            Ok(_) => {
              self.cursor += size as usize;
              nframes += 1;
            }
          }
        }
      } // if no 0xAA found
    } // end loop
    bar.finish_with_message("Done!");
    let _ = self.rewind();
    nframes
  } // end fn

  /// Return the next frame for the current files
  ///
  /// Will return none if the file has been exhausted.
  /// Use ::rewind to start reading from the beginning
  /// again.
  pub fn read_next_item(&mut self) -> Option<CRFrame> {
    // filter::Unknown corresponds to allowing any
  
    let mut buffer = [0];
    loop {
      match self.file_reader.read_exact(&mut buffer) {
        Err(err) => {
          debug!("Unable to read from file! {err}");
          // this is ok in case we are out of files
          self.prime_next_file()?;
          return self.read_next_item();
        }
        Ok(_) => {
          self.cursor += 1;
        }
      }
      if buffer[0] != 0xAA {
        continue;
      } else {
        match self.file_reader.read_exact(&mut buffer) {
          Err(err) => {
            debug!("Unable to read from file! {err}");
            self.prime_next_file()?;
            return self.read_next_item();
          }
          Ok(_) => {
            self.cursor += 1;
          }
        }
  
        if buffer[0] != 0xAA { 
          continue;
        } else {
          // read the the size of the packet
          let mut buffer_psize = [0,0,0,0,0,0,0,0];
          match self.file_reader.read_exact(&mut buffer_psize) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              self.prime_next_file()?;
              return self.read_next_item();
            }
            Ok(_) => {
              self.cursor += 8;
            }
          }
          
          let vec_data = buffer_psize.to_vec();
          //println!("vec_data {:?}", vec_data);
          let size     = parse_u64(&vec_data, &mut 0);
          //println!("Will read {size} bytes for payload!");
          // now at this point, we want the packet!
          // except we skip ahead or stop earlier
          if self.skip_ahead > 0 && self.n_packs_skipped < self.skip_ahead {
            // we don't want it
            match self.file_reader.seek(SeekFrom::Current(size as i64)) {
              Err(err) => {
                debug!("Unable to read more data! {err}");
                self.prime_next_file()?;
                return self.read_next_item();
              }
              Ok(_) => {
                self.n_packs_skipped += 1;
                self.cursor += size as usize;
              }
            }
            continue; // this is just not the packet we want
          }
          if self.stop_after > 0 && self.n_packs_read >= self.stop_after {
            // we don't want it
            match self.file_reader.seek(SeekFrom::Current(size as i64)) {
              Err(err) => {
                debug!("Unable to read more data! {err}");
                self.prime_next_file()?;
                return self.read_next_item();
              }
              Ok(_) => {
                self.cursor += size as usize;
              }
            }
            continue; // this is just not the packet we want
          }
  
          let mut frame = CRFrame::new();
          let mut payload = vec![0u8;size as usize];
  
          match self.file_reader.read_exact(&mut payload) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              self.prime_next_file()?;
              return self.read_next_item();
            }
            Ok(_) => {
              self.cursor += size as usize;
            }
          }
          let mut in_frame_pos = 0usize;
          frame.index = CRFrame::parse_index(&payload, &mut in_frame_pos);
          frame.bytestorage = payload[in_frame_pos..].to_vec();
  
          //tp.payload = payload;
          // we don't filter, so we like this packet
          let mut tail = vec![0u8; 2];
          match self.file_reader.read_exact(&mut tail) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              self.prime_next_file()?;
              return self.read_next_item();
            }
            Ok(_) => {
              self.cursor += 2;
            }
          }
          let tail = parse_u16(&tail,&mut 0);
          if tail != CRFrame::TAIL {
            debug!("CRFrame TAIL signature wrong!");
            return None;
          }
          self.n_packs_read += 1;
          // hand the database poitners over to the frame 
          frame.tof_paddles  = Arc::clone(&self.tof_paddles);
          frame.trk_strips   = Arc::clone(&self.trk_strips);
          frame.trk_masks    = Arc::clone(&self.trk_masks);
          frame.trk_ped      = Arc::clone(&self.trk_ped);
          frame.trk_tf       = Arc::clone(&self.trk_tf);
          frame.trk_cmn      = Arc::clone(&self.trk_cmn);
          frame.do_trk_calib = self.do_trk_calib;
          frame.subtract_trk_cmn = self.do_trk_cmn_noise;
          return Some(frame);
        }
      } // if no 0xAA found
    } // end loop
  } // end fn
}

impl fmt::Display for CRReader {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut range_repr = String::from("");
    if self.skip_ahead > 0 {
      range_repr += &(format!("({}", self.skip_ahead));
    } else {
      range_repr += "(";
    }
    if self.stop_after > 0 {
      range_repr += &(format!("..{})", self.stop_after));
    } else {
      range_repr += "..)";
    }
    let mut repr = String::from("<CRReader :");
    repr += "\n -- files:";
    for k in &self.filenames {
      repr += &format!("\n     -- {k}");
    }
    if self.filenames.len() > 0 {
      repr += &format!("\n  current : {}", self.get_current_filename().unwrap());
    }
    repr += &String::from("\n -- -- -- -- -- -- -- -- -- -- -- --");
    repr += &format!("\n  read {} packets, {} errors, range {}>", self.n_packs_read, self.n_errors, range_repr);
    write!(f, "{}", repr)
  }
}

reader!(CRReader,CRFrame);

//--------------------------------------

#[cfg(feature="pybindings")]
#[pymethods]
impl CRReader {

  #[new]
  fn new_py(filename_or_directory : &Bound<'_,PyAny>) -> PyResult<Self> {
    let mut string_value = String::from("foo");
    if let Ok(s) = filename_or_directory.extract::<String>() {
       string_value = s;
    } //else if let Ok(p) = filename_or_directory.extract::<&Path>() {
    if let Ok(fspath_method) = filename_or_directory.getattr("__fspath__") {
      if let Ok(fspath_result) = fspath_method.call0() {
        if let Ok(py_string) = fspath_result.extract::<String>() {
          string_value = py_string;
        }
      }
    }
    match Self::new(string_value) {
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
      Ok(reader) => {
        return Ok(reader);
      }
    }
  }

  #[pyo3(name="set_tracker_calibrations_from_fnames")]
  #[pyo3(signature = (mask = None, pedestal = None, transfer_fn = None, cmn_noise = None))]
  fn set_tracker_calibrations_from_fnames_py(&mut self,
                                              mask        : Option<String>,
                                              pedestal    : Option<String>,
                                              transfer_fn : Option<String>,
                                              cmn_noise   : Option<String>) {
    self.set_tracker_calibrations_from_fnames(mask, pedestal, transfer_fn, cmn_noise);
  }

  /// This is the filename we are currently 
  /// extracting frames from 
  #[getter]
  #[pyo3(name="current_filename")]
  fn get_current_filename_py(&self) -> Option<String> {
    self.get_current_filename()
  }

  /// Start the reader from the beginning
  /// This is equivalent to a re-initialization
  /// of that reader.
  #[pyo3(name="rewind")]
  fn rewind_py(&mut self) -> PyResult<()> {
    match self.rewind() {
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
      Ok(_) => Ok(())
    }
  }

  #[pyo3(name="count_frames")]
  fn count_frames_py(&mut self) -> usize {
    self.count_frames()
  }

  fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
    slf 
  }
  
  fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<CRFrame> {
    match slf.next() { 
      Some(frame) => {
        return Some(frame)
      }   
      None => {
        return None;
      }   
    }   
  }
}

#[cfg(feature="pybindings")]
pythonize_display!(CRReader);
