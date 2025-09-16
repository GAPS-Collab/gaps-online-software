//! The TelemetryPacketReader allows to read a (file) stream of serialized
//! TelemetryPackets, which are typically the .bin files as generated 
//! by the gcu
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use std::io::{
  BufReader,
  Read,
  Seek,
  SeekFrom,
};

use crate::prelude::*;

/// Read serialized TelemetryPackets from an existing file
///
/// Read GAPS binary files ("Berkeley binaries)
#[derive(Debug)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TelemetryPacketReader {
  /// Reader will emit packets from these files,
  /// if one file is exhausted, it moves on to 
  /// the next file automatically
  pub filenames        : Vec<String>,
  /// The index of the file the reader is 
  /// currently reading
  pub file_idx         : usize,
  /// depending on the source of the telemetry files, 
  /// there might be duplicates, because we get them
  /// over different streams. 
  /// Suppress these multiple packets
  pub dedup           : bool,
  /// Ignore packets that have a gcu time earlier than start_time 
  pub start_time      : Option<f64>,
  /// Ignore packets that have a gcu time later than end_time
  pub end_time        : Option<f64>,
  file_reader         : BufReader<File>,
  /// Current (byte) position in the file
  cursor              : usize,
  /// Read only packets of type == PacketType
  pub filter          : TelemetryPacketType,
  /// Number of read packets
  n_packs_read        : usize,
  /// Number of skipped packets
  n_packs_skipped     : usize,
  /// Skip the first n packets
  pub skip_ahead      : usize,
  /// Stop reading after n packets
  pub stop_after      : usize,
  /// Number of encountered duplicates 
  pub n_duplicates    : usize,
  /// A cache to allow to quench duplicates 
  /// pkt counter -> pkt checksum
  dedup_cache         : HashMap<u16, VecDeque<u16>>
}


impl TelemetryPacketReader {
  pub fn new(filename_or_directory : String, dedup : bool, start_time : Option<f64>, end_time : Option<f64>) -> Self {
    let firstfile : String;
    let re = Regex::new(r"RAW(\d{6})_(\d{6})\.bin$").unwrap();
    match list_path_contents_sorted(&filename_or_directory, Some(re)) {
      Err(err) => {
        error!("{} does not seem to be either a valid directory or an existing file! {err}", filename_or_directory);
        panic!("Unable to open files!");
      }
      Ok(mut files) => {
        
        if let Some(start) = start_time {
          info!("Removing files earlier than {}", start);
          files.retain(|x| { get_unix_timestamp_from_telemetry(&x).unwrap() as f64 >= start} );
        }
        if let Some(end)   = end_time {
          info!("Removing files later than {}", end);
          files.retain(|x| { get_unix_timestamp_from_telemetry(&x).unwrap() as f64 <= end } ); 
        }
        firstfile = files[0].clone();
        match OpenOptions::new().create(false).append(false).read(true).open(&firstfile) {
          Err(err) => {
            error!("Unable to open file {firstfile}! {err}");
            panic!("Unable to create reader from {filename_or_directory}!");
          }
          Ok(file) => {
            // prime the cache 
            let mut dedup_cache = HashMap::<u16, VecDeque<u16>>::with_capacity(u16::MAX as usize + 1);  
            for k in 0..u16::MAX as usize + 1 {
              dedup_cache.insert(k as u16, VecDeque::<u16>::with_capacity(4));
            }
            let packet_reader = Self { 
              filenames         : files,
              file_idx          : 0,
              dedup             : dedup,
              start_time        : start_time,
              end_time          : end_time,
              file_reader       : BufReader::new(file),
              cursor            : 0,
              filter            : TelemetryPacketType::Unknown,
              n_packs_read      : 0,
              skip_ahead        : 0,
              stop_after        : 0,
              n_packs_skipped   : 0,
              n_duplicates      : 0,
              dedup_cache       : dedup_cache,
            };
            packet_reader
          }
        }
      }
    }
  } 
  
  pub fn clear_dedup_cache(&mut self) {
    let mut dedup_cache = HashMap::<u16, VecDeque<u16>>::with_capacity(u16::MAX as usize + 1);  
    for k in 0..u16::MAX as usize + 1 {
      dedup_cache.insert(k as u16, VecDeque::<u16>::with_capacity(4));
    }
    self.dedup_cache = dedup_cache;
  }

  /// Preview the number of frames in this reader
  pub fn count_packets(&mut self) -> (usize, usize, HashMap<TelemetryPacketType,usize>) {
    let _ = self.rewind();
    self.clear_dedup_cache();
    let mut nframes = 0usize;
    let mut buffer  = [0];
    let mut incomplete  = 0usize;
    let mut index   = HashMap::<TelemetryPacketType,usize>::new();
    for k in TelemetryPacketType::iter() {
      index.insert(k, 0);
    }
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
      
      //thead.sync      = 0x90eb;

      if buffer[0] != 0xeb {
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
        if buffer[0] != 0x90 { 
          continue;
        } else {
          // read packet type for index
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
              *index.get_mut(&TelemetryPacketType::from(buffer[0])).unwrap() += 1;
              self.cursor += 1;
            }
          }
          // read the the size of the packet
          // first we have to skip 6 bytes
          let mut buffer_skip = [0,0,0,0,0,0,0];
          match self.file_reader.read_exact(&mut buffer_skip) {
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
              self.cursor += 7;
            }
          }
          let mut buffer_psize = [0,0];
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
              self.cursor += 2;
            }
          }
          let vec_data = buffer_psize.to_vec();
          let size     = parse_u16(&vec_data, &mut 0);
          let mut temp_buffer = vec![0; size as usize];
          // skip 2 more bytes
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
              self.cursor += 2;
            }
          }
          match self.file_reader.read_exact(&mut temp_buffer) { 
          //match self.file_reader.seek(SeekFrom::Current(size as i64)) {
          //match self.file_reader.seek_relative(size as i64) {
            Err(err) => {
              incomplete += 1;
              warn!("Unable to read {size} bytes from {}! {err}", self.get_current_filename().unwrap());
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
    (nframes, incomplete, index)
  } // end fn

  /// Return the next TelemetryPacket in the stream
  ///
  /// Will return none if the file has been exhausted.
  /// Use ::rewind to start reading from the beginning
  /// again.
  pub fn read_next_item(&mut self) -> Option<TelemetryPacket> {
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
      if buffer[0] != 0xeb {
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

        if buffer[0] != 0x90 { 
          continue;
        } else {
          // FIXME - use TofPacketHeader::from_bytestream here
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
          let mut thead     = TelemetryPacketHeader::new();
          thead.sync        = 0x90eb;
          thead.packet_type = TelemetryPacketType::from(buffer[0]);
          let ptype    = TelemetryPacketType::from(buffer[0]);
          // read the the size of the packet
          let mut buffer_ts = [0,0,0,0];
          match self.file_reader.read_exact(&mut buffer_ts) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              self.prime_next_file()?;
              return self.read_next_item();
            }
            Ok(_) => {
              self.cursor += 4;
              thead.timestamp = u32::from_le_bytes(buffer_ts);
            }
          }
          let mut buffer_counter = [0,0];
          match self.file_reader.read_exact(&mut buffer_counter) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              self.prime_next_file()?;
              return self.read_next_item();
            }
            Ok(_) => {
              self.cursor += 2;
              thead.counter   = u16::from_le_bytes(buffer_counter);
            }
          }
          let mut buffer_length = [0,0];
          match self.file_reader.read_exact(&mut buffer_length) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              return None;
            }
            Ok(_) => {
              self.cursor += 2;
              thead.length    = u16::from_le_bytes(buffer_length);
            }
          }
          let mut buffer_checksum = [0,0];
          match self.file_reader.read_exact(&mut buffer_checksum) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              self.prime_next_file()?;
              return self.read_next_item();
            }
            Ok(_) => {
              self.cursor += 2;
              thead.checksum    = u16::from_le_bytes(buffer_checksum);
            }
          }
          
          let mut size     = thead.length;
          // This size includes the header
          if (size as usize) < TelemetryPacketHeader::SIZE {
            error!("This packet might be empty or corrupt!");
            return None;
          }
          size -= TelemetryPacketHeader::SIZE as u16;
          if ptype != self.filter && self.filter != TelemetryPacketType::Unknown {
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
          

          let mut tp = TelemetryPacket::new();
          tp.header  = thead;
          
          //tp.packet_type = ptype;
          //let mut payload = vec![0u8;TelemetryPacketHeader::SIZE];
          //match self.file_reader.read_exact(&mut payload) {
          //  Err(err) => {
          //    debug!("Unable to read from file! {err}");
          //    return None;
          //  }
          //  Ok(_) => {
          //    self.cursor += size as usize;
          //  }
          //}

          let mut payload = vec![0u8;size as usize];
          match self.file_reader.read_exact(&mut payload) {
            Err(err) => {
              debug!("Unable to read from file! {err}");
              self.prime_next_file()?;
              return self.read_next_item();
            }
            Ok(_) => {
              self.cursor += tp.header.length as usize;
            }
          }

          tp.payload = payload;
          self.n_packs_read += 1;
          // check if the packet has been seen already
          if self.dedup {
            let mut will_send       : bool;
            if self.dedup_cache[&tp.header.counter].len() == 0 {
              will_send = true;
            } else {
              if !self.dedup_cache.contains_key(&tp.header.counter) {
                panic!("The dedup cache does not contain {}", tp.header.counter);
              }
              will_send = true;
              for checksum in &self.dedup_cache[&tp.header.counter] {
                if checksum == &tp.header.checksum {
                  will_send = false;
                } 
              }
            }
            // this happens when we have seen the packet counter, but not the actual 
            // checksum
            if will_send {
              self.dedup_cache.get_mut(&tp.header.counter).unwrap().push_back(tp.header.checksum);  
              return Some(tp);
            } else {
              // make sure the caches won't get too long, limit 
              // our selves to 4 times the packet counter rollover 
              self.n_duplicates += 1;
              if self.dedup_cache[&tp.header.counter].len() > 4 {
                self.dedup_cache.get_mut(&tp.header.counter).unwrap().pop_front();
              }

              return self.read_next_item();
            }
          }
          if self.start_time.is_some() {
            if tp.header.get_gcutime() < self.start_time.unwrap() {
              return self.read_next_item(); 
            }
          }
          if self.end_time.is_some() {
            if tp.header.get_gcutime() > self.end_time.unwrap() {
              return self.read_next_item();
            }
          }
          return Some(tp);
        }
      } // if no 0xAA found
    } // end loop
  } // end fn
}

impl Default for TelemetryPacketReader {
  fn default() -> Self {
    TelemetryPacketReader::new(String::from(""), false, None, None)
  }
}

impl fmt::Display for TelemetryPacketReader {
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
    let repr = format!("<TelemetryPacketReader : read {} packets, filter {}, range {},\n files {:?}>", self.n_packs_read, self.filter, range_repr, self.filenames);
    write!(f, "{}", repr)
  }
}

reader!(TelemetryPacketReader, TelemetryPacket);

#[cfg(feature="pybindings")]
#[pymethods]
impl TelemetryPacketReader {

  #[new]
  #[pyo3(signature = (filenames_or_directory, dedup = true, start_time = None, end_time = None))]
  fn new_py(filenames_or_directory : &Bound<'_,PyAny>, dedup : bool, start_time : Option<f64>, end_time : Option<f64>) -> PyResult<Self> {
    
    let mut string_value = String::from("foo");
    let mut fnames       = Vec::<String>::new();
    if let Ok(s) = filenames_or_directory.extract::<String>() {
       string_value = s;
    } //else if let Ok(p) = filename_or_directory.extract::<&Path>() {
    if let Ok(fspath_method) = filenames_or_directory.getattr("__fspath__") {
      if let Ok(fspath_result) = fspath_method.call0() {
        if let Ok(py_string) = fspath_result.extract::<String>() {
          string_value = py_string;
        }
      }
    }
    if let Ok(list) = filenames_or_directory.extract::<Vec<String>>() {
      for k in list {
          fnames.push(k);
      } //else if let Ok(p) = filename_or_directory.extract::<&Path>() {
      //  if let Ok(py_pth_mth) = k.getattr("__fspath__") {
      //    if let Ok(py_pth_rs) = py_pth_mth.call0(py_pth_mth) {
      //      if let Ok(py_string) = py_pth_rs.extract::<String>() {
      //        fnames.push(py_string);
      //      }
      //    }
      //  }
      //}
    }
    let mut reader : Self;
    if fnames.len() > 0 {
      string_value = fnames[0].clone();
      reader = Self::new(string_value, dedup, start_time, end_time);
      reader.filenames = fnames;
    } else {
      reader = Self::new(string_value, dedup, start_time, end_time);
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
  fn get_n_duplicates(&self) -> usize {
    self.n_duplicates
  }

  #[pyo3(name = "count_packets")]
  fn count_packets_py(&mut self) -> (usize,usize,HashMap<TelemetryPacketType,usize>) {
    self.count_packets()
  }

  //#[getter]
  //fn first(&mut self) -> Option<TofPacket> {
  //  self.first_packet()
  //}

  //#[getter]
  //fn last(&mut self) -> Option<TofPacket> {
  //  self.last_packet()
  //}
 
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
  fn get_current_filename_py(&self) -> Option<&str> {
    self.get_current_filename()
  }

  /// Start the reader from the beginning
  /// This is equivalent to a re-initialization
  /// of that reader.
  #[pyo3(name="rewind")]
  fn rewind_py(&mut self) -> PyResult<()> {
    // HOTFIX to avoid python segfault. However, this has to go 
    // into the rust rewind as well!
    self.clear_dedup_cache();
    match self.rewind() {
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
      Ok(_) => Ok(())
    }
  }

  //#[pyo3(name="count_packets")]
  //fn count_packets_py(&mut self) -> usize {
  //  self.count_packets()
  //}

  fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
    slf 
  }
  
  fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<TelemetryPacket> {
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
pythonize_display!(TelemetryPacketReader);


