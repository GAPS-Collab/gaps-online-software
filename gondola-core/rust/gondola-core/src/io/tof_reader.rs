//! The TofPacketReader allows to read a (file) stream of serialized
//! TofPackets
//!
//!
// The following file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Read serialized TofPackets from an existing file or directory
///
/// This can read the "TOF stream" files, typically suffixed with .tof.gaps
/// These files are typically written by a TofPacketReader instance, e.g. as 
/// on the TOF flight computer
#[derive(Debug)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TofPacketReader {
  /// Read from this file
  pub filenames       : Vec<String>,
  file_reader         : BufReader<File>,
  /// Current (byte) position in the file
  cursor              : usize,
  /// Read only packets of type == PacketType
  pub filter          : TofPacketType,
  /// Number of read packets
  n_packs_read        : usize,
  /// Number of skipped packets
  n_packs_skipped     : usize,
  /// Skip the first n packets
  pub skip_ahead      : usize,
  /// Stop reading after n packets
  pub stop_after      : usize,
  /// The index of the current file in the internal "filenames" vector.
  pub file_idx        : usize,
}

impl TofPacketReader {
  
  /// Setup a new Reader, allowing the argument to be either the name of a single file or 
  /// the name of a directory
  /// FIXME - make this return Result, like the Caraspace reader
  pub fn new(filename_or_directory : &str) -> Self {
    let firstfile : String;
    match list_path_contents_sorted(&filename_or_directory, None) {
      Err(err) => {
        error!("{} does not seem to be either a valid directory or an existing file! {err}", filename_or_directory);
        panic!("Unable to open files!");
      }
      Ok(files) => {
        firstfile = files[0].clone();
        match OpenOptions::new().create(false).append(false).read(true).open(&firstfile) {
          Err(err) => {
            error!("Unable to open file {firstfile}! {err}");
            panic!("Unable to create reader from {filename_or_directory}!");
          }
          Ok(file) => {
            let packet_reader = Self { 
              filenames       : files,
              file_reader     : BufReader::new(file),
              cursor          : 0,
              filter          : TofPacketType::Unknown,
              n_packs_read    : 0,
              skip_ahead      : 0,
              stop_after      : 0,
              n_packs_skipped : 0,
              file_idx        : 0,
            };
            packet_reader
          }
        }
      }
    } 
  }


  /// Return the next tofpacket in the stream
  ///
  /// Will return none if the file has been exhausted.
  /// Use ::rewind to start reading from the beginning
  /// again.
  ///
  /// If a filter is set, only packets of type as set 
  /// in the filter will be read, all others will be 
  /// ignored
  pub fn read_next_item(&mut self) -> Option<TofPacket> {
    // filter::Unknown corresponds to allowing any
    let mut buffer = [0];
    loop {
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
          // the 3rd byte is the packet type
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
          let ptype    = TofPacketType::from(buffer[0]);
          // read the the size of the packet
          let mut buffer_psize = [0,0,0,0];
          match self.file_reader.read_exact(&mut buffer_psize) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              self.prime_next_file()?;
              return self.read_next_item();
            } 
            Ok(_) => {
              self.cursor += 4;
            }
          }
          let vec_data = buffer_psize.to_vec();
          let size     = parse_u32(&vec_data, &mut 0);
          if ptype != self.filter && self.filter != TofPacketType::Unknown {
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

          let mut tp = TofPacket::new();
          tp.packet_type = ptype;
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
          tp.payload = payload;
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
          if tail != TofPacket::TAIL {
            debug!("TofPacket TAIL signature wrong!");
            return None;
          }
          self.n_packs_read += 1;
          return Some(tp);
        }
      } // if no 0xAA found
    } // end loop
  } // end fn

  /// This is the file the current cursor is located 
  /// in and frames are currently read out from 
  pub fn get_current_filename(&self) -> Option<String> {
    // should only happen when it is empty
    if self.filenames.len() <= self.file_idx {
      return None;
    }
    Some(self.filenames[self.file_idx].clone())
  }
  
  /// Preview the number of frames in this reader
  pub fn count_packets(&mut self) -> usize {
    let _ = self.rewind();
    let mut nframes = 0usize;
    let mut buffer  = [0];
    let bar_template : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.blue/grey} {pos:>7}/{len:7}";
    let bar_style  = ProgressStyle::with_template(bar_template).expect("Unable to set progressbar style!");
    let bar = ProgressBar::new(self.filenames.len() as u64);
    bar.set_position(0);
    bar.set_message (String::from("Counting packets.."));
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
          // first we have to skip one byte for the packet type
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
          let mut buffer_psize = [0,0,0,0];
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

  /// The very first TofPacket for a reader
  pub fn first_packet(&mut self) -> Option<TofPacket> {
    match self.rewind() {
      Err(err) => {
        error!("Error when rewinding files! {err}");
      }
      Ok(_) => ()
    }
    let pack = self.read_next_item();
    match self.rewind() {
      Err(err) => {
        error!("Error when rewinding files! {err}");
      }
      Ok(_) => ()
    }
    return pack;
  }

  /// The very last TofPacket for a reader
  pub fn last_packet(&mut self) -> Option<TofPacket> { 
    self.file_idx    = self.filenames.len() - 1;
    let lastfilename = self.filenames[self.file_idx].clone();
    let lastfile     = OpenOptions::new().create(false).append(false).read(true).open(lastfilename).expect("Unable to open file {nextfilename}");
    self.file_reader = BufReader::new(lastfile);
    self.cursor      = 0;
    let mut tp = TofPacket::new();
    let mut idx = 0;
    loop {
      match self.read_next_item() {
        None => {
          match self.rewind() {
            Err(err) => {
              error!("Error when rewinding files! {err}");
            }
            Ok(_) => ()
          }
          if idx == 0 {
            return None;
          } else {
            return Some(tp);
          }
        }
        Some(pack) => {
          idx += 1;
          tp = pack;
          continue;
        }
      }
    }
  }


}

impl fmt::Display for TofPacketReader {
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
    let repr = format!("<TofPacketReader :read {} packets, filter {}, range {}\n files {:?}>", self.n_packs_read, self.filter, range_repr, self.filenames);
    write!(f, "{}", repr)
  }
}

reader!(TofPacketReader,TofPacket);

#[cfg(feature="pybindings")]
#[pymethods]
impl TofPacketReader {

  #[new]
  #[pyo3(signature = (filename_or_directory, filter = None))] 
  fn new_py(filename_or_directory : &Bound<'_,PyAny>, filter : Option<TofPacketType>) -> PyResult<Self> {
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
    let mut reader = Self::new(&string_value);
    match filter {
      None => (),
      Some(ftr) => {
        reader.filter = ftr;
      }
    }
    Ok(reader)
    //match Self::new(&string_value) {
    //  Err(err) => {
    //    return Err(PyValueError::new_err(err.to_string()));
    //  }
    //  Ok(reader) => {
    //    return Ok(reader);
    //  }
    //}
  }

  #[getter]
  fn first(&mut self) -> Option<TofPacket> {
    self.first_packet()
  }

  #[getter]
  fn last(&mut self) -> Option<TofPacket> {
    self.last_packet()
  }
 
  #[getter]
  fn filenames(&self) -> Vec<String> {
    self.filenames.clone()
  }
  //#[pyo3(name="set_tracker_calibrations_from_fnames")]
  //#[pyo3(signature = (mask = None, pedestal = None, transfer_fn = None, cmn_noise = None))]
  //fn set_tracker_calibrations_from_fnames_py(&mut self,
  //                                            mask        : Option<String>,
  //                                            pedestal    : Option<String>,
  //                                            transfer_fn : Option<String>,
  //                                            cmn_noise   : Option<String>) {
  //  self.set_tracker_calibrations_from_fnames(mask, pedestal, transfer_fn, cmn_noise);
  //}

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

  #[pyo3(name="count_packets")]
  fn count_packets_py(&mut self) -> usize {
    self.count_packets()
  }

  fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
    slf 
  }
  
  fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<TofPacket> {
    slf.next()
    //match slf.next() { 
    //  Some(packet) => {
    //    return Some(packet)
    //  }   
    //  None => {
    //    return None;
    //  }   
    //}   
  }
}

#[cfg(feature="pybindings")]
pythonize_display!(TofPacketReader);


