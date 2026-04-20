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

/// The db insert companion to TrackerStripGain
#[derive(Debug,PartialEq, Clone, Insertable)]
#[diesel(table_name = schema::tof_db_trackerstripgain)]
#[allow(non_snake_case)]
#[cfg_attr(feature="pybindings", pyclass)]
struct NewTrackerStripGain {
  pub strip_id            : i32,    
  pub volume_id           : i64,    
  pub utc_timestamp_start : i64,
  pub utc_timestamp_stop  : i64,
  pub name                : Option<String>,
  pub gain                : f32,
  pub gain_is_mean        : bool,
}

impl NewTrackerStripGain {
  pub fn from(gain : &TrackerStripGain) -> Self {
    Self {
      strip_id            : gain.strip_id            ,    
      volume_id           : gain.volume_id           ,    
      utc_timestamp_start : gain.utc_timestamp_start ,
      utc_timestamp_stop  : gain.utc_timestamp_stop  ,
      name                : gain.name.clone()        ,
      gain                : gain.gain       , 
      gain_is_mean        : gain.gain_is_mean      , 
    }
  }
}

/// Common noise subtraction - pulse channels on the wafers and get the average adc. 
/// The gain is available as well. Data from Mengjiao's group 
#[derive(Debug,PartialEq, Clone,Queryable, Selectable, serde::Serialize, serde::Deserialize)]
#[diesel(table_name = schema::tof_db_trackerstripgain)]
#[diesel(primary_key(data_id))]
#[allow(non_snake_case)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TrackerStripGain {   
  pub data_id              : i32,
  pub strip_id             : i32,    
  pub volume_id            : i64,    
  pub utc_timestamp_start  : i64,    
  pub utc_timestamp_stop   : i64,    
  pub name                 : Option<String>, 
  pub gain                 : f32,
  pub gain_is_mean         : bool,
} 

impl TrackerStripGain {

  pub fn new() -> Self {
    Self {
      data_id             : 0,
      strip_id            : 0,    
      volume_id           : 0,    
      utc_timestamp_start : 0,   
      utc_timestamp_stop  : 0,
      name                : None, 
      gain                : 0.0, 
      gain_is_mean        : false,
    }
  }
  
  pub fn parse_from_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<Self>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut gains = Vec::<Self>::new();
    let hid_vid_map = get_hid_vid_maps().unwrap().1;
    let mut n_entries = 0u64;
    let mut mean_gain = 0.0f64;
    for line in reader.lines() {
      let line = line?;
      if line.starts_with("#") || line.starts_with("Layer") || line.starts_with("layer") {
        continue;
      }
      let mut gain = Self::new();
      let parts: Vec<&str> = line.split_whitespace().collect();
      if parts.len() == 5 {
        // Parse the first number as a standard decimal
        let layer   = parts[0].parse::<u8>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let row     = parts[1].parse::<u8>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let module   = parts[2].parse::<u8>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let channel  = parts[3].parse::<u8>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        gain.strip_id  = TrackerStrip::create_stripid(layer, row, module, channel) as i32; 
        gain.volume_id = *hid_vid_map.get(&(gain.strip_id as u32)).unwrap() as i64; // critical error is good here,
        gain.gain      = parts[4].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        mean_gain += gain.gain as f64;
        gains.push(gain); 
        n_entries += 1;
      }
    }
    mean_gain /= n_entries as f64;
    let mut fixed_gains = Vec::<TrackerStripGain>::with_capacity(gains.len());
    for g in gains.iter_mut() {
      if g.gain == 0.0 {
        g.gain = mean_gain as f32;
        g.gain_is_mean = true;
      }
      fixed_gains.push(g.clone());
    }
    Ok(fixed_gains)
  }
  
  pub fn all_names() -> Result<Vec<String>, ConnectionError> {
    let mut conn = connect_to_db()?;
    let mut names = Vec::<String>::new();
    let unique_names =
      schema::tof_db_trackerstripgain::table.select(
      schema::tof_db_trackerstripgain::name)
      .distinct()
      .load::<Option<String>>(&mut conn).expect("Error getting names from db!");
    for k in unique_names {
      if let Some(n) = k {
        names.push(n);
      }
    }
    Ok(names)
  }
  
  /// Get Tracker strip cmn noise data for a certain dataset 
  ///
  /// # Returns:
  ///   * HashMap<u32 [strip id], TrackerStripTransferFn> 
  pub fn as_dict_by_name(fname : &str) -> Result<HashMap<u32,Self>, ConnectionError> {
    use schema::tof_db_trackerstripgain::dsl::*;
    let mut strips = HashMap::<u32, Self>::new();
    if fname == "" {
      match Self::all() {
        None => {
          error!("Unable to retrive ANY TrackerStripCMNNoise Data (pulser)");
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
    match tof_db_trackerstripgain.filter(
      schema::tof_db_trackerstripgain::name.eq(fname)).load::<Self>(&mut conn) {
      Err(err) => {
        error!("We can't find any tracker strip common noise information with that name in the database! {err}");
        return Ok(strips);
      }
      Ok(peds_) => {
        for s in peds_ {
          strips.insert(s.strip_id as u32, s );
        }
      }
    }
    return Ok(strips);
  }

  /// Get all tracker strip transfer functions from the database
  ///
  /// # Returns:
  ///   * HashMap<u32 [strip id], TrackeStripTransferFunction> 
  pub fn all() -> Option<Vec<Self>> {
    use schema::tof_db_trackerstripgain::dsl::*;
    let mut conn = connect_to_db().ok()?;
    match tof_db_trackerstripgain.load::<Self>(&mut conn) {
      Err(err) => {
        error!("Unable to load tracker transfer functions from db! {err}");
        return None;
      }
      Ok(strips) => {
        return Some(strips);
      }
    }
  }
}

impl fmt::Display for TrackerStripGain {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = format!("<TrackerStripGain [{}]:", self.strip_id);
    repr += &(format!("\n   vid              : {}", self.volume_id));
    repr += "\n   UTC Timestamps (Begin/End):";
    repr += &(format!("\n   {}/{}", self.utc_timestamp_start, self.utc_timestamp_stop));    
    if self.gain_is_mean {
      repr += &(String::from("\n -- Gain is mean value!"));
    }
    if self.name.is_some() {
      repr += &(format!("\n   name     : {}", self.name.clone().unwrap())); 
    }
    repr += &(format!("\n   gain : {}>", self.gain));
    write!(f, "{}", repr)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TrackerStripGain {
  
  #[staticmethod]
  #[pyo3(name="all")]
  pub fn all_py() -> Option<Vec<Self>> {
    Self::all()
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
        error!("Unable to retrieve tracker strip gain dictionary. {err}. Did you laod the setup-env.sh shell?");
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
  fn get_name(&self) -> Option<String> {
    self.name.clone()
  }
  
  #[setter]
  fn set_name(&mut self, value : String) {
    self.name = Some(value);
  }
      
  #[getter]
  fn get_gain(&self) -> f32 {
    self.gain
  }
  
  #[getter]
  fn get_gain_is_mean(&self) -> bool {
    self.gain_is_mean
  }
  
  #[staticmethod]
  #[pyo3(name="parse_from_file")]
  fn parse_from_file_py(fname : &str) -> Option<Vec<Self>> {
    let result = Self::parse_from_file(fname);
    if result.is_ok() {
      return Some(result.unwrap());
    } else {
      error!("An error occured when parsing {} : '{}'", fname, result.unwrap_err());
      return None;
    }
  } 
}

#[cfg(feature="pybindings")]
pythonize!(TrackerStripGain);

#[cfg_attr(feature="pybindings", pyfunction)]
pub fn create_trk_gain_table( db_path: &str, gains: Vec<TrackerStripGain>) { 
  use schema::tof_db_trackerstripgain::dsl::*;
  let mut conn = SqliteConnection::establish(db_path).ok().unwrap(); 
  let mut _query_result = diesel::sql_query("
      CREATE TABLE IF NOT EXISTS tof_db_trackerstripgain (
          data_id INTEGER PRIMARY KEY AUTOINCREMENT,
          strip_id INTEGER NOT NULL,
          volume_id BIGINT NOT NULL,
          utc_timestamp_start BIGINT NOT NULL,
          utc_timestamp_stop BIGINT NOT NULL,
          name TEXT,
          gain FLOAT,
          gain_is_mean BOOL
      )
  ").execute(&mut conn);
  let mut new_gains = Vec::<NewTrackerStripGain>::new();
  for g in gains {
    let ng = NewTrackerStripGain::from(&g);
    new_gains.push(ng);
  }
  _query_result = diesel::insert_into(tof_db_trackerstripgain)
    .values(&new_gains)
    .execute(&mut conn);
}

