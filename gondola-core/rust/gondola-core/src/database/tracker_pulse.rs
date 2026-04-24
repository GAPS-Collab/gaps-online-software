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

/// The db insert companion to TrackerStripPulse
#[derive(Debug,PartialEq, Clone, Insertable)]
#[diesel(table_name = schema::tof_db_trackerstrippulse)]
#[allow(non_snake_case)]
#[cfg_attr(feature="pybindings", pyclass)]
struct NewTrackerStripPulse {
  pub strip_id            : i32,    
  pub volume_id           : i64,    
  pub utc_timestamp_start : i64,
  pub utc_timestamp_stop  : i64,
  pub name                : Option<String>,
  pub pulse_chn           : i32,
  pub pulse_avg           : f32,
  pub pulse_is_mean       : bool,
}

impl NewTrackerStripPulse {
  pub fn from(gain : &TrackerStripPulse) -> Self {
    Self {
      strip_id            : gain.strip_id            ,    
      volume_id           : gain.volume_id           ,    
      utc_timestamp_start : gain.utc_timestamp_start ,
      utc_timestamp_stop  : gain.utc_timestamp_stop  ,
      name                : gain.name.clone()        ,
      pulse_chn           : gain.pulse_chn           ,
      pulse_avg           : gain.pulse_avg           ,
      pulse_is_mean       : gain.pulse_is_mean       ,
    }
  }
}

/// Common noise subtraction - pulse channels on the wafers and get the average adc. 
/// The gain is available as well. Data from Mengjiao's group 
#[derive(Debug,PartialEq, Clone,Queryable, Selectable, serde::Serialize, serde::Deserialize)]
#[diesel(table_name = schema::tof_db_trackerstrippulse)]
#[diesel(primary_key(data_id))]
#[allow(non_snake_case)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TrackerStripPulse {   
  pub data_id              : i32,
  pub strip_id             : i32,    
  pub volume_id            : i64,    
  pub utc_timestamp_start  : i64,    
  pub utc_timestamp_stop   : i64,    
  pub name                 : Option<String>, 
  pub pulse_chn            : i32,
  pub pulse_avg            : f32,
  pub pulse_is_mean        : bool,
} 

impl TrackerStripPulse {

  pub fn new() -> Self {
    Self {
      data_id             : 0,
      strip_id            : 0,    
      volume_id           : 0,    
      utc_timestamp_start : 0,   
      utc_timestamp_stop  : 0,
      name                : None, 
      pulse_chn           : 0,
      pulse_avg           : 0.0,
      pulse_is_mean       : false,
    }
  }
  
  pub fn parse_from_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<Self>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut pulses = Vec::<Self>::new();
    let hid_vid_map = get_hid_vid_maps().unwrap().1;
    let mut n_entries  = 0u64;
    let mut mean_pulse = 0.0f64;
    //let mut mean_avg   = 0.0f64;
    let mut all_strip_ids : Vec<_> = hid_vid_map.keys().collect();
    for line in reader.lines() {
      let line = line?;
      if line.starts_with("#") || line.starts_with("Layer") || line.starts_with("layer") {
        continue;
      }
      let mut pulse = Self::new();
      let parts: Vec<&str> = line.split_whitespace().collect();
      if parts.len() == 6 {
        // Parse the first number as a standard decimal
        let layer   = parts[0].parse::<u8>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let row     = parts[1].parse::<u8>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let module   = parts[2].parse::<u8>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let channel  = parts[3].parse::<u8>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        pulse.strip_id  = TrackerStrip::create_stripid(layer, row, module, channel) as i32; 
        pulse.volume_id = *hid_vid_map.get(&(pulse.strip_id as u32)).unwrap() as i64; // critical error is good here,
        pulse.pulse_chn = parts[4].parse::<i32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        pulse.pulse_avg = parts[5].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        mean_pulse += pulse.pulse_chn as f64;
        //mean_avg   += pulse.pulse_avg as f64;
        all_strip_ids.retain(|x| *x != &(pulse.strip_id as u32));
        pulses.push(pulse); 
        n_entries += 1;
      }
    }
    mean_pulse /= n_entries as f64;
    //mean_avg   /= n_entries as f64;
    // it seems we should create an entry even for the 
    // strips which are not in the file 
    //for stripid in all_strip_ids {
    //  let mut pulse = Self::new(); 
    //  pulse.strip_id = *stripid as i32;
    //  pulse.volume_id = *hid_vid_map.get(&(pulse.strip_id as u32)).unwrap() as i64; // critical error is good here,
    //  pulse.pulse_is_mean = true;
    //  pulse.pulse_avg = 0.0; // for some reason, don't set it here 
    //  pulse.pulse_chn  = mean_pulse.floor() as i32;
    //  pulses.push(pulse);
    //}
    Ok(pulses)
  }
  
  pub fn all_names() -> Result<Vec<String>, ConnectionError> {
    let mut conn = connect_to_db()?;
    let mut names = Vec::<String>::new();
    let unique_names =
      schema::tof_db_trackerstrippulse::table.select(
      schema::tof_db_trackerstrippulse::name)
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
    use schema::tof_db_trackerstrippulse::dsl::*;
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
    match tof_db_trackerstrippulse.filter(
      schema::tof_db_trackerstrippulse::name.eq(fname)).load::<Self>(&mut conn) {
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
    use schema::tof_db_trackerstrippulse::dsl::*;
    let mut conn = connect_to_db().ok()?;
    match tof_db_trackerstrippulse.load::<Self>(&mut conn) {
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

impl fmt::Display for TrackerStripPulse {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = format!("<TrackerStripPulse [{}]:", self.strip_id);
    repr += &(format!("\n   vid              : {}", self.volume_id));
    repr += "\n   UTC Timestamps (Begin/End):";
    repr += &(format!("\n   {}/{}", self.utc_timestamp_start, self.utc_timestamp_stop));    
    if self.name.is_some() {
      repr += &(format!("\n   name     : {}", self.name.clone().unwrap())); 
    }
    if self.pulse_is_mean {
      repr += &(String::from("\n -- Pulse is mean value!"));
    }
    repr += &(format!("\n   pulse ch : {} pulse avg : {}>", self.pulse_chn, self.pulse_avg));
    write!(f, "{}", repr)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TrackerStripPulse {
  
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
  fn get_pulse_chn(&self) -> i32 {
    self.pulse_chn
  }
  
  #[getter]
  fn get_pulse_avg(&self) -> f32 {
    self.pulse_avg
  } 

  #[getter]
  fn get_pulse_is_mean(&self) -> bool {
    self.pulse_is_mean
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
pythonize!(TrackerStripPulse);

#[cfg_attr(feature="pybindings", pyfunction)]
pub fn create_trk_pulse_table( db_path: &str, pulses: Vec<TrackerStripPulse>) { 
  use schema::tof_db_trackerstrippulse::dsl::*;
  let mut conn = SqliteConnection::establish(db_path).ok().unwrap(); 
  let mut _query_result = diesel::sql_query("
      CREATE TABLE IF NOT EXISTS tof_db_trackerstrippulse (
          data_id INTEGER PRIMARY KEY AUTOINCREMENT,
          strip_id INTEGER NOT NULL,
          volume_id BIGINT NOT NULL,
          utc_timestamp_start BIGINT NOT NULL,
          utc_timestamp_stop BIGINT NOT NULL,
          name TEXT,
          pulse_chn INTEGER,
          pulse_avg FLOAT,
          pulse_is_mean BOOL
      )
  ").execute(&mut conn);
  let mut new_pulses = Vec::<NewTrackerStripPulse>::new();
  for p in pulses {
    let np = NewTrackerStripPulse::from(&p);
    new_pulses.push(np);
  }
  _query_result = diesel::insert_into(tof_db_trackerstrippulse)
    .values(&new_pulses)
    .execute(&mut conn);
}


