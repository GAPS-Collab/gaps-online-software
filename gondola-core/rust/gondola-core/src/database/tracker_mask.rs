//! Database access & entities for gaps-online-software
//!
//! A local .sqlite database is shipped with gaps-online-software,
//! pre-populated with relevant meta data for the GAPS experiment.
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

use crate::database::schema;
use diesel::prelude::*;

use std::io::{
  self,
  BufRead,
  BufReader
};


/// Masking of unusable strips as curated by the tracker team 
#[derive(Debug,PartialEq, Clone,Queryable, Selectable, serde::Serialize, serde::Deserialize)]
#[diesel(table_name = schema::tof_db_trackerstripmask)]
#[diesel(primary_key(data_id))]
#[allow(non_snake_case)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TrackerStripMask {
  pub data_id             : i32,
  pub strip_id            : i32,    
  pub volume_id           : i64,    
  pub utc_timestamp_start : i64,
  pub utc_timestamp_stop  : i64,
  pub name                : Option<String>, 
  pub active              : bool,   
}

impl TrackerStripMask {

  pub fn new() -> Self {
    Self {
      data_id             : 0,
      strip_id            : 0,    
      volume_id           : 0,    
      utc_timestamp_start : 0,  
      utc_timestamp_stop  : 0,
      name                : None, 
      active              : true
    }
  }
 
  pub fn all_names() -> Result<Vec<String>, ConnectionError> {
    let mut conn = connect_to_db()?;
    let mut names = Vec::<String>::new();
    let unique_names =
      schema::tof_db_trackerstripmask::table.select(
      schema::tof_db_trackerstripmask::name)
      .distinct()
      .load::<Option<String>>(&mut conn).expect("Error getting names from db!");
    for k in unique_names {
      if let Some(n) = k {
        names.push(n);
      }
    }
    Ok(names)
  }

  /// Get Tracker strip mask 
  ///
  /// # Returns:
  ///   * HashMap<u32 [strip id], TrackerStripMask> 
  pub fn as_dict_by_name(fname : &str) -> Result<HashMap<u32,Self>, ConnectionError> {
    use schema::tof_db_trackerstripmask::dsl::*;
    let mut strips = HashMap::<u32, Self>::new();
    if fname == "" {
      match Self::all() {
        None => {
          error!("Unable to retrive ANY TrackerStripMask");
          return Ok(strips);
        }
        Some(_strips) => {
          for k in _strips {
            strips.insert(k.strip_id as u32, k);
          }
          return Ok(strips);
        }
      }
    }
    let mut conn = connect_to_db()?;
    match tof_db_trackerstripmask.filter(
      schema::tof_db_trackerstripmask::name.eq(fname)).load::<Self>(&mut conn) {
      Err(err) => {
        error!("We can't find any tracker strip masks in the database! {err}");
        return Ok(strips);
      }
      Ok(masks_) => {
        for s in masks_ {
          strips.insert(s.strip_id as u32, s );
        }
      }
    }
    return Ok(strips);
  }

  /// Get all tracker strip mask from the database
  ///
  /// # Returns:
  ///   * HashMap<u32 [strip id], TrackeStripMask> 
  pub fn all() -> Option<Vec<Self>> {
    use schema::tof_db_trackerstripmask::dsl::*;
    let mut conn = connect_to_db().ok()?;
    match tof_db_trackerstripmask.load::<Self>(&mut conn) {
      Err(err) => {
        error!("Unable to load tracker strips from db! {err}");
        return None;
      }
      Ok(strips) => {
        return Some(strips);
      }
    }
  }

  /// Create a channel maks from a pulse file to mask all pulsed channels
  ///
  /// This is not the regular mask file, but will mask additional strips which 
  /// have been re-purposed
  pub fn parse_from_pulse_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<Self>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut results = Vec::new(); 
    for line in reader.lines() {
      let line = line?;
      let parts: Vec<&str> = line.split_whitespace().collect();
      if parts.len() == 6 {
        let layer     = parts[0].parse::<u32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let row       = parts[1].parse::<u32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let module    = parts[2].parse::<u32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let channel   = parts[3].parse::<u32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let pulse_chn = parts[4].parse::<i32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let pulse_avg = parts[5].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        results.push((layer, row, module, channel, pulse_chn, pulse_avg));
      }
    }
    let mut masks = Vec::<Self>::new();
    let hid_vid_map = get_hid_vid_maps().unwrap().1;
    for k in results {
      let layer     = k.0; 
      let row       = k.1;
      let module    = k.2;
      let channel   = k.3;
      let p_channel = k.4;
      // discard for this - we don't need to know 
      // this when just creating the mask
      //let p_avg     = k.5;

      let mut strip = TrackerStrip::new();
      strip.module  = module as i32;
      strip.row     = row    as i32;
      strip.layer   = layer  as i32;
      let mut active = false;
      if p_channel < 0 || p_channel as u32 != channel {
        active = true;
      }
      let mut mask  = TrackerStripMask::new();
      mask.strip_id  = strip.get_stripid() as i32; 
      mask.volume_id = *hid_vid_map.get(&(mask.strip_id as u32)).unwrap() as i64; // critical error is good here,
      mask.active    = active;
      masks.push(mask);
    }
    Ok(masks)
  }

  pub fn parse_from_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<Self>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut results = Vec::new(); 
    for line in reader.lines() {
      let line = line?;
      let parts: Vec<&str> = line.split_whitespace().collect();
      if parts.len() == 2 {
        // Parse the first number as a standard decimal
        let index = parts[0].parse::<u32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
  
        // Parse the second number as Hex. 
        // We strip the "0x" prefix before parsing.
        let hex_str = parts[1].trim_start_matches("0x");
        let value = u32::from_str_radix(hex_str, 16)
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
  
        results.push((index, value));
      }
    }
    let mut masks = Vec::<Self>::new();
    for k in results {
      let layer     = (k.0 / 100) % 10; 
      let row       = (k.0 / 10) % 10;  
      let module    = k.0 % 10;         
      let active_ch = k.1;

      let mut strip = TrackerStrip::new();
      strip.module  = module as i32;
      strip.row     = row    as i32;
      strip.layer   = layer  as i32;
      for n in 0..32 {
        let active    = ( active_ch >> n) & 0x1;
        strip.channel = n;
        let mut mask  = TrackerStripMask::new();
        mask.strip_id = strip.get_stripid() as i32; 
        mask.active  = active > 0;
        masks.push(mask);
      }
    }
    Ok(masks)
  }
}

impl Default for TrackerStripMask {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for TrackerStripMask {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = format!("<TrackerStripMask [{}]:", self.strip_id);
    repr += &(format!("\n   vid           : {}", self.volume_id));
    repr += "\n   UTC Timestamps (Begin/End):";
    repr += &(format!("\n   {}/{}", self.utc_timestamp_start, self.utc_timestamp_stop));    
    if self.name.is_some() {
      repr += &(format!("\n   name        : {}", self.name.clone().unwrap())); 
    }
    repr += &(format!("\n   active        : {}", self.active));   
    write!(f, "{}", repr)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TrackerStripMask {
  
  #[staticmethod]
  #[pyo3(name="all")]
  pub fn all_py() -> Option<Vec<Self>> {
    Self::all()
  } 
  
  /// Create a channel maks from a pulse file to mask all pulsed channels
  ///
  /// This is not the regular mask file, but will mask additional strips which 
  /// have been re-purposed
  #[staticmethod]
  #[pyo3(name="parse_from_pulse_file")]
  pub fn parse_from_pulse_file_py(fname : &str) -> Option<Vec<Self>> {
    let masks = Self::parse_from_pulse_file(fname);
    if masks.is_ok() {
      return Some(masks.unwrap());
    } else {
      error!("An error occured when parsing {}", fname);
      return None;
    }
  }

  #[staticmethod]
  #[pyo3(name="parse_from_file")]
  fn parse_from_file_py(fname : &str) -> Option<Vec<Self>> {
    let masks = Self::parse_from_file(fname);
    if masks.is_ok() {
      return Some(masks.unwrap());
    } else {
      error!("An error occured when parsing {}", fname);
      return None;
    }
  }

  #[staticmethod]
  #[pyo3(name="all_names")]
  /// Get all names for registered datasets. These
  /// can be used in .as_dict_by_name() to query 
  /// the db for a set of values
  pub fn all_names_py() -> Option<Vec<String>> {
    match Self::all_names() {
      Err(_) => {
        return None;
      }
      Ok(names) => {
        return Some(names);
      }
    }
  }
  
  #[staticmethod]
  #[pyo3(name="as_dict_by_name")]
  pub fn all_as_dict_py(name : &str) -> Option<HashMap<u32,Self>> {
    match Self::as_dict_by_name(name) {
      Err(err) => {
        error!("Unable to retrieve tracker strip mask dictionary. {err}. Did you laod the setup-env.sh shell?");
        return None;
      }
      Ok(_data) => {
        return Some(_data);
      }
    }
  } 
  
  #[getter]
  fn get_strip_id     (&self) -> i32 {    
    self.strip_id
  }
  
  #[getter]
  fn get_volume_id    (&self) -> i64 {    
    self.volume_id
  }
  
  #[getter]
  fn get_utc_timestamp_start(&self) -> i64 {
    self.utc_timestamp_start
  }
  
  #[getter]
  fn get_utc_timestamp_stop(&self) -> i64 {
    self.utc_timestamp_stop
  }
  
  #[setter]
  fn set_utc_timestamp_start(&mut self, value : i64) {
    self.utc_timestamp_start = value;
  }
  
  #[setter]
  fn set_utc_timestamp_stop(&mut self, value : i64) {
    self.utc_timestamp_stop = value;
  }
  
  #[getter]
  fn get_name    (&self) -> Option<String> {
    self.name.clone()
  }
  
  #[setter]
  #[pyo3(name="name")]
  fn set_name_py(&mut self, value : String) {
    self.name = Some(value);
  }
  
  #[getter]
  fn get_active       (&self) -> bool { 
    self.active
  }
}

#[cfg(feature="pybindings")]
pythonize!(TrackerStripMask);



/// The db insert companion to TrackerStripMask
#[derive(Debug,PartialEq, Clone, Insertable)]
#[diesel(table_name = schema::tof_db_trackerstripmask)]
#[allow(non_snake_case)]
#[cfg_attr(feature="pybindings", pyclass)]
struct NewTrackerStripMask {
  pub strip_id            : i32,    
  pub volume_id           : i64,    
  pub utc_timestamp_start : i64,
  pub utc_timestamp_stop  : i64,
  pub name                : Option<String>, 
  pub active              : bool,   
}

impl NewTrackerStripMask {
  pub fn from(mask : &TrackerStripMask) -> Self {
    Self {
      strip_id            : mask.strip_id             ,    
      volume_id           : mask.volume_id            ,    
      utc_timestamp_start : mask.utc_timestamp_start  ,
      utc_timestamp_stop  : mask.utc_timestamp_stop   ,
      name                : mask.name.clone()         , 
      active              : mask.active               ,   
    }
  }
}

#[cfg_attr(feature="pybindings", pyfunction)]
pub fn create_trk_mask_table( db_path: &str, masks: Vec<TrackerStripMask>) { 
  use schema::tof_db_trackerstripmask::dsl::*;
  //use schema::tof_db_trackerstripmask;
  let mut conn = SqliteConnection::establish(db_path).ok().unwrap();
  
  let mut _query_result = diesel::sql_query("
      CREATE TABLE IF NOT EXISTS tof_db_trackerstripmask (
          data_id INTEGER PRIMARY KEY AUTOINCREMENT,
          strip_id INTEGER NOT NULL,
          volume_id BIGINT NOT NULL,
          utc_timestamp_start BIGINT NOT NULL,
          utc_timestamp_stop BIGINT NOT NULL,
          name TEXT,
          active BOOLEAN NOT NULL
      )
  ").execute(&mut conn);
  let mut new_masks = Vec::<NewTrackerStripMask>::new();
  for m in masks {
    let nm = NewTrackerStripMask::from(&m);
    new_masks.push(nm);
  }
  _query_result = diesel::insert_into(tof_db_trackerstripmask)
    .values(&new_masks)
    .execute(&mut conn);
}

