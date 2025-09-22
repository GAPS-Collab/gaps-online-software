//! gaps-online-software i/o system
//!
// This file is part of gaps-online-software and published 
// under the GPLv3 license

#[cfg(feature="tof-liftof")]
pub mod ipbus;
pub mod parsers;
pub mod serialization;
pub use serialization::Serialization;
pub mod caraspace;
#[cfg(feature="root")]
pub mod root_reader;
#[cfg(feature="root")]
pub use root_reader::read_example;
pub mod tof_reader;
pub use tof_reader::TofPacketReader;
pub mod tof_writer;
pub use tof_writer::TofPacketWriter;
pub mod telemetry_reader;
pub use telemetry_reader::TelemetryPacketReader;
pub mod data_source;
pub use data_source::DataSource;
pub mod streamers;
pub use streamers::*;
use crate::prelude::*;

//----------------------------------------------------------

/// Types of files
#[derive(Debug, Clone)]
pub enum FileType {
  Unknown,
  /// Calibration file for specific RB with id
  CalibrationFile(u8),
  /// A regular run file with TofEvents
  RunFile(u32),
  /// A file created from a file with TofEvents which 
  /// contains only TofEventSummary
  SummaryFile(String),
}

//----------------------------------------------------------

/// Get all filenames in the current path sorted by timestamp if available
/// If the given path is a file and not a directory, return only that 
/// file instead
///
/// # Arguments:
///
///    * input   : name of the target directory
///    * pattern : the regex pattern to look for. That the sorting works,
///                the pattern needs to return a date for the first
///                captured argument and a time for the second captured argument
pub fn list_path_contents_sorted(input: &str, pattern: Option<Regex>) -> Result<Vec<String>, io::Error> {
  let path = Path::new(input);
  match fs::metadata(path) {
    Ok(metadata) => {
      if metadata.is_file() {
        let fname = String::from(input);
        return Ok(vec![fname]);
      } 
      if metadata.is_dir() {
        let re : Regex;
        match pattern {
          None => {
            // use a default pattern which matches mmost cases  
            //re = Regex::new(r"Run\d+_\d+\.(\d{6})_(\d{6})UTC(\.tof)?\.gaps$").unwrap();
            re = Regex::new(GENERIC_ONLINE_FILE_PATTERH).unwrap();
          }
          Some(_re) => {
            re = _re;
          }
        }
        let mut entries: Vec<(u32, u32, String)> = fs::read_dir(path)?
          .filter_map(Result::ok) // Ignore unreadable entries
          .filter_map(|entry| {
            let filename = format!("{}/{}", path.display(), entry.file_name().into_string().ok()?);
            re.captures(&filename.clone()).map(|caps| {
              let date = caps.get(1)?.as_str().parse::<u32>().ok()?;
              let time = caps.get(2)?.as_str().parse::<u32>().ok()?;
              Some((date, time, filename))
            })?
          })
          .collect();

        // Sort by (date, time)
        entries.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
        // Return only filenames
        return Ok(entries.into_iter().map(|(_, _, name)| name).collect());
      } 
      Err(io::Error::new(io::ErrorKind::Other, "Path exists but is neither a file nor a directory"))
    }
    Err(e) => Err(e),
  }
}

//----------------------------------------------------------

/// Get all filenames in the current path sorted by timestamp if available
/// If the given path is a file and not a directory, return only that 
/// file instead
///
/// # Arguments:
///
///    * input   : name of the target directory
///    * pattern : the regex pattern to look for. That the sorting works,
///                the pattern needs to return a date for the first
///                captured argument and a time for the second captured argument
#[cfg(feature="pybindings")]
#[pyfunction]
#[pyo3(name="list_path_contents_sorted")]
#[pyo3(signature = ( input, pattern = None ))]
pub fn list_path_contents_sorted_py(input: &str, pattern: Option<String>) -> PyResult<Option<Vec<String>>> {
  let mut regex_pattern : Option<Regex> = None;
  if let Some(pat_str) = pattern {
    match Regex::new(&pat_str) {
      Err(err) => {
        let msg = format!("Unable to compile regex {}! {}. Check your regex syntax! Also try a raw string.", &pat_str, err); 
        return Err(PyValueError::new_err(msg));
      }
      Ok(re) => {
        regex_pattern = Some(re);
      }
    }
  }
  match list_path_contents_sorted(input, regex_pattern) {
    Err(err) => {  
      error!("Unable to get files! {err}");
      return Err(PyValueError::new_err(err.to_string()));
    }
    Ok(files) => {
      return Ok(Some(files));
    }
  }
}

//----------------------------------------------------------

/// Get a human readable timestamp for NOW
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn get_utc_timestamp() -> String {
  let now: DateTime<Utc> = Utc::now();
  //let timestamp_str = now.format("%Y_%m_%d-%H_%M_%S").to_string();
  let timestamp_str = now.format(HUMAN_TIMESTAMP_FORMAT).to_string();
  timestamp_str
}

/// Retrieve the utc timestamp from any telemetry (binary) file 
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn get_unix_timestamp_from_telemetry(fname : &str) -> Option<u64> { 
  let tformat_re = Regex::new(GENERIC_TELEMETRY_FILE_PATTERN_CAPUTRE).unwrap();
  let res = tformat_re.captures(fname).and_then(|caps| {
    let map : HashMap<String, String> = tformat_re.capture_names()
      .filter_map(|name| name)
      .filter_map(|name| { 
        //caps.name(name).map(|m| (m.as_str().to_string(), m.as_str().to_string()))
        caps.name(name).map(|m| (name.to_string(), m.as_str().to_string()))
      })
      .collect();
    Some(map)
  });
  return get_unix_timestamp(&res.unwrap()["utctime"], None); 
}

//----------------------------------------------------------

/// Create date string in YYMMDD format
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn get_utc_date() -> String {
  let now: DateTime<Utc> = Utc::now();
  //let timestamp_str = now.format("%Y_%m_%d-%H_%M_%S").to_string();
  let timestamp_str = now.format("%y%m%d").to_string();
  timestamp_str
}

//----------------------------------------------------------

/// A standardized name for calibration files saved by 
/// the liftof suite
///
/// # Arguments
///
/// * rb_id   : unique identfier for the 
///             Readoutboard (1-50)
/// * default : if default, just add 
///             "latest" instead of 
///             a timestamp
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn get_califilename(rb_id : u8, latest : bool) -> String {
  let ts = get_utc_timestamp();
  if latest {
    format!("RB{rb_id:02}_latest.cali.tof.gaps")
  } else {
    format!("RB{rb_id:02}_{ts}.cali.tof.gaps")
  }
}

//----------------------------------------------------------

/// A standardized name for regular run files saved by
/// the liftof suite
///
/// # Arguments
///
/// * run       : run id (identifier)
/// * subrun    :  subrun id (identifier of file # within
///                the run
/// * rb_id     :  in case this should be used on the rb, 
///                a rb id can be specified as well
/// * timestamp :  substitute the current time with this timestamp
///                (or basically any other string) instead.
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn get_runfilename(run : u32, subrun : u64, rb_id : Option<u8>, timestamp : Option<String>) -> String {
  let ts : String;
  match timestamp {
    Some(_ts) => {
      ts = _ts;
    }
    None => {
      ts = get_utc_timestamp();
    }
  }
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

//----------------------------------------------------------
    
/// Get the timestamp from a .tof.gaps file
///
/// # Arguments:
///   * fname : Filename of .tof.gaps file
#[cfg_attr(feature="pybindings", pyfunction)]
#[cfg_attr(feature="pybindings", pyo3(signature = (fname , pattern = None)))]
pub fn get_rundata_from_file(fname : &str, pattern : Option<String>) -> Option<HashMap<String,String>> {
  let regex_pattern : Regex;
  if let Some(pat_str) = pattern {
    match Regex::new(&pat_str) {
      Err(err) => {
        let msg = format!("Unable to compile regex {}! {}. Check your regex syntax! Also try a raw string.", &pat_str, err); 
        //return Err(PyValueError::new_err(msg));
        //return Err(err);
        error!("{}",msg);
        return None;
      }
      Ok(re) => {
        regex_pattern = re;
      }
    }
  } else {
    regex_pattern = Regex::new(GENERIC_ONLINE_FILE_PATTERH_CAPTURE).unwrap();
  }
  let res : Option<HashMap<String,String>>;
  res = regex_pattern.captures(fname).and_then(|caps| {
    let map : HashMap<String, String> = regex_pattern.capture_names()
      .filter_map(|name| name)
      .filter_map(|name| { 
        //caps.name(name).map(|m| (m.as_str().to_string(), m.as_str().to_string()))
        caps.name(name).map(|m| (name.to_string(), m.as_str().to_string()))
      })
      .collect();
    Some(map)
  });
  //ts = pattern.search(str(fname)).groupdict()['tdate']
  //#print (ts)
  //ts = datetime.strptime(ts, '%y%m%d_%H%M%S')
  //ts = ts.replace(tzinfo=timezone.utc)
  //return ts
  res
}

//----------------------------------------------------------

/// Retrieve the DateTime object from a string as used in 
/// the names of the run files
///
/// # Arguments:
///
///   * input  : The input string the datetime shall be extracted
///              from 
///   * format : The format of the date string. Something like 
///              %y%m%d_%H%M%S
#[cfg_attr(feature="pybindings", pyfunction)]
#[cfg_attr(feature="pybindings", pyo3(signature = (input , tformat = None )))]
pub fn get_datetime(input : &str, tformat : Option<String>) -> Option<DateTime<Utc>> {
  // this is the default format 
  let mut date_time_format = String::from("%y%m%d_%H%M%S");
  if let Some(tform) = tformat {
    date_time_format = tform.to_string(); 
  }
  if let Ok(ndtime) = NaiveDateTime::parse_from_str(input, &date_time_format) {
    //let dt_utc : DateTime<Utc> = DateTime::<Utc>::from_utc(ndtime, Utc); 
    let dt_utc : DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(ndtime, Utc); 
    return Some(dt_utc);
  } else { 
    error!("Unable to parse {} for format {}! You can specify formats trhough the tformat keyword", input, date_time_format);
    return None;
  }
}

//--------------------------------------------------------------

/// Retrieve the UNIX timestamp from a string as used in 
/// the names of the run files
///
/// # Arguments:
///
///   * input  : The input string the datetime shall be extracted
///              from 
///   * format : The format of the date string. Something like 
///              %y%m%d_%H%M%S
#[cfg_attr(feature="pybindings", pyfunction)]
#[cfg_attr(feature="pybindings", pyo3(signature = (input , tformat = None )))]
pub fn get_unix_timestamp(input : &str, tformat : Option<String>) -> Option<u64> {
  let dt = get_datetime(input, tformat);
  if let Some(dt_) = dt {
    // FIXME - we are only supporting times later than 
    //         the UNIX epoch!
    return Some(dt_.timestamp() as u64);
  } else {
    return None;
  }
}

//--------------------------------------------------------------

/// Identifier for different data sources
#[derive(Debug, Copy, Clone, PartialEq,FromRepr, AsRefStr, EnumIter)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
#[repr(u8)]
pub enum DataSourceKind {
  Unknown            = 0,
  /// The "classic" written to the TOF-CPU on disk in flight
  /// season 24/25 style
  TofFiles           = 10,
  /// As TofFiles, but sent over the network
  TofStream          = 11,
  /// The files as written on disk when received by a GSE 
  /// system
  TelemetryFiles     = 20,
  /// Flight telemetry stream as sent out directly by the 
  /// instrument
  TelemetryStream    = 21,
  /// Caraspace is a comprehensive, highly efficient data 
  /// format which is used to combine Telemetry + TofStream 
  /// data. Data written in this format as stored on disk.
  CaraspaceFiles     = 30,
  /// The same as above, however, represented as a network
  /// stream
  CaraspaceStream    = 31,
  /// Philip's SimpleDet ROOT files
  ROOTFiles          = 40,
}

expand_and_test_enum!(DataSourceKind, test_datasourcekind_repr);

//--------------------------------------------------------------

// in case we have pybindings for this type, 
// expand it so that it can be used as keys
// in dictionaries
#[cfg(feature = "pybindings")]
#[pymethods]
impl DataSourceKind {

  #[getter]
  fn __hash__(&self) -> usize {
    (*self as u8) as usize
  } 
}

#[cfg(feature="pybindings")]
pythonize_display!(DataSourceKind);

//--------------------------------------------------------------

/// Implement the Reader trait and necessary getters/setters to 
/// make a struct an actual reader
#[macro_export]
macro_rules! reader {
  ($struct_name:ident, $element_type:ident) => {
 
    use crate::io::DataReader; 
    use crate::io::Serialization;

    impl Iterator for $struct_name {
      type Item = $element_type;
      fn next(&mut self) -> Option<Self::Item> {
        self.read_next()
      }
    }

    impl DataReader<$element_type> for $struct_name {
      fn get_header0(&self) -> u8 {
        ($element_type::HEAD & 0x1) as u8 
      }

      fn get_header1(&self) -> u8 {
        ($element_type::HEAD & 0x2) as u8
      }

      fn get_file_idx(&self) -> usize {
        self.file_idx // Setting the specified field
      }
    
      fn set_file_idx(&mut self, file_idx : usize) {
        self.file_idx = file_idx;
      }
      
      fn get_filenames(&self) -> &Vec<String> {
          &self.filenames
      }
      
      fn set_cursor(&mut self, pos : usize) {
        self.cursor = pos;
      }
 
      fn set_file_reader(&mut self, reader : BufReader<File>) {
        self.file_reader = reader;
      }
    
      fn read_next(&mut self) -> Option<$element_type> {
        self.read_next_item()
      }
    
      /// Get the next file ready
      fn prime_next_file(&mut self) -> Option<usize> {
        if self.file_idx == self.filenames.len() -1 {
          return None;
        } else {
          self.file_idx += 1;
          let nextfilename : &str = self.filenames[self.file_idx].as_str();
          let nextfile     = OpenOptions::new().create(false).append(false).read(true).open(nextfilename).expect("Unable to open file {nextfilename}");
          self.file_reader = BufReader::new(nextfile);
          self.cursor      = 0;
          return Some(self.file_idx);
        }
      }
    }
  }
}

/// Generics for packet reading (TofPacket, Telemetry packet,...)
/// FIXME - not implemented yet
pub trait DataReader<T> 
  where T : Default + Serialization {
  ///// header bytes, e.g. 0xAAAA for TofPackets, first byte
  //const HEADER0 : u8 = 0;
  ///// header bytes, e.g. 0xAAAA for TofPackets, second byte
  //const HEADER1 : u8 = 0;

  fn get_header0(&self) -> u8;
  fn get_header1(&self) -> u8;

  /// Return all filenames the reader is primed with   
  fn get_filenames(&self) -> &Vec<String>;

  /// The current index corresponding to the file the 
  /// reader is currently working on
  fn get_file_idx(&self) -> usize;

  /// Set a new file idx corresponding to a file the reader 
  /// is currently working on
  fn set_file_idx(&mut self, idx : usize);

  /// reset a new reader
  fn set_file_reader(&mut self, freader : BufReader<File>);
  
  /// Get the next file ready
  fn prime_next_file(&mut self) -> Option<usize>;

  /// The name of the file the reader is currently 
  /// working on
  fn get_current_filename(&self) -> Option<&str> {
    // should only happen when it is empty
    if self.get_filenames().len() <= self.get_file_idx() {
      return None;
    }
    Some(self.get_filenames()[self.get_file_idx()].as_str())
  }

  /// Manage the internal cursor attribute
  fn set_cursor(&mut self, pos : usize);

  /// Get the next frame/packet from the stream. Can be used to 
  /// implement iterators
  fn read_next(&mut self) -> Option<T>; 

  /// Get the first entry in all of the files the reader is 
  /// primed with
  fn first(&mut self)     -> Option<T> {
      match self.rewind() {
      Err(err) => {
        error!("Error when rewinding files! {err}");
        return None;
      }
      Ok(_) => ()
    }
    let pack = self.read_next();
    match self.rewind() {
      Err(err) => {
        error!("Error when rewinding files! {err}");
      }
      Ok(_) => ()
    }
    return pack;
  }

  /// Get the last entry in all of the files the reader is 
  /// primed with
  fn last(&mut self)      -> Option<T> {
    self.set_file_idx(self.get_filenames().len() - 1);
    let lastfilename = self.get_filenames()[self.get_file_idx()].as_str();
    let lastfile     = OpenOptions::new().create(false).append(false).read(true).open(lastfilename).expect("Unable to open file {nextfilename}");
    self.set_file_reader(BufReader::new(lastfile));
    self.set_cursor(0);
    let mut tp    = T::default();
    let mut idx = 0;
    loop {
      match self.read_next() {
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

  /// Rewind the current file and set the file index to the 
  /// first file, so data can be read again from the 
  /// beginning
  fn rewind(&mut self) -> io::Result<()> {
    let firstfile = &self.get_filenames()[0];
    let file = OpenOptions::new().create(false).append(false).read(true).open(&firstfile)?;
    self.set_file_reader(BufReader::new(file));
    self.set_cursor(0);
    self.set_file_idx(0);
    Ok(())
  }
}

//// blanket implementation: every `T` that implements Reader also implements Iterator
//impl<T:std::default::Default + Serialization> Iterator for DataReader<T>  { 
//  type Item = T;
//  fn next(&mut self) -> Option<Self::Item> {
//    self.read_next()
//  }
//}

