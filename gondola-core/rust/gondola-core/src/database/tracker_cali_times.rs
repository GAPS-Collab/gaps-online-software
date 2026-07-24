//! In-flight calibration times for the GAPS tracker system
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;
use crate::database::schema;

use diesel::prelude::*;
use std::io::BufRead;
//use diesel::SqliteConnection;
//use diesel::RunQueryDsl;
//use diesel::Connection;
//use crate::database::schema::tof_db_trackercalitime::dsl::tof_db_trackercalitime;


/// Check if the given (unix) time is within a calibration 
/// time window for the tracker 
// FIXME - this should be rewritten to be 2 functions so that 
// &Vec can be used as argument (Bound in the python version)
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn is_in_tracker_cali_window(ts : f64, cali_windows : Vec<TrackerCaliTimeWindow>) -> bool { 
  for cw in cali_windows {
    if ts.trunc() as i64 <= cw.utc_timestamp_stop && ts.trunc() as i64 >= cw.utc_timestamp_start {
      return true;
    }
  }
  false
}
  
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn create_trk_cali_tw_table( db_path: &str, cali_time_windows: Vec<TrackerCaliTimeWindow>) { 
  use schema::tof_db_trackercalitime::dsl::*;
  let mut conn = SqliteConnection::establish(db_path).ok().unwrap();
  
  let mut _query_result = diesel::sql_query("
      CREATE TABLE IF NOT EXISTS tof_db_trackercalitime (
          data_id INTEGER PRIMARY KEY AUTOINCREMENT,
          utc_timestamp_start BIGINT NOT NULL,
          utc_timestamp_stop BIGINT NOT NULL,
          meta TEXT
      )
  ").execute(&mut conn);
  _query_result = diesel::insert_into(tof_db_trackercalitime)
    .values(&cali_time_windows)
    .execute(&mut conn);
  match _query_result {
    Ok(_) => {
      println!("Entered data into db successfully!");
    }
    Err(err) => {
      println!("Error occured when entering data in the db! {err}");
    }
  }
}

/// Tracker calibrations were conducted on several ocasions during 
/// the flight. This data might be unusuable for physics analysis 
/// and might be excluded from any processing. 
#[derive(Debug,PartialEq, Clone, Queryable, Selectable, Insertable, serde::Serialize, serde::Deserialize)]
#[diesel(table_name = schema::tof_db_trackercalitime)]
#[diesel(primary_key(data_id))]
#[allow(non_snake_case)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TrackerCaliTimeWindow {
  #[diesel(deserialize_as = i32)]
  pub data_id             : Option<i32>,
  pub utc_timestamp_start : i64,
  pub utc_timestamp_stop  : i64,
  pub meta                : Option<String>, 
}

impl TrackerCaliTimeWindow {

  pub fn new() -> Self {
    Self {
      data_id             : None,
      utc_timestamp_start : 0,  
      utc_timestamp_stop  : 0,
      meta                : None, 
    }
  }
 
  /// Get all tracker calibration times from the database 
  ///
  /// # Returns:
  ///   * HashMap<u32 [strip id], TrackeCaliTime> 
  pub fn all() -> Option<Vec<Self>> {
    use schema::tof_db_trackercalitime::dsl::*;
    let mut conn = connect_to_db().ok()?;
    match tof_db_trackercalitime.load::<Self>(&mut conn) {
      Err(err) => {
        error!("Unable to load tracker calibration times from db! {err}");
        return None;
      }
      Ok(cali_times) => {
        return Some(cali_times);
      }
    }
  }

  pub fn parse_from_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<Self>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut results = Vec::new(); 
    for line in reader.lines() {
      let line = line?;
      let parts: Vec<&str> = line.split_whitespace().collect();
      if parts.len() == 2 {
        let start = parts[0].parse::<i64>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let stop = parts[1].parse::<i64>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        results.push((start, stop));
      }
    }
    let mut ctimes = Vec::<Self>::new();
    for k in results {
      let mut tctw  = TrackerCaliTimeWindow::new();
      tctw.utc_timestamp_start = k.0;
      tctw.utc_timestamp_stop  = k.1;
      ctimes.push(tctw);
    }
    Ok(ctimes)
  }
}

impl Default for TrackerCaliTimeWindow {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for TrackerCaliTimeWindow {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TrackerCaliTimeWindow:");
    repr += "\n   UTC Timestamps (Begin/End):";
    repr += &(format!("\n   {}/{}", self.utc_timestamp_start, self.utc_timestamp_stop));    
    if self.meta.is_some() {
      repr += &(format!("\n   meta info        : {}", self.meta.clone().unwrap())); 
    }
    write!(f, "{}", repr)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TrackerCaliTimeWindow {
  
  #[staticmethod]
  #[pyo3(name="all")]
  pub fn all_py() -> Option<Vec<Self>> {
    Self::all()
  } 
  
  #[staticmethod]
  #[pyo3(name="parse_from_file")]
  fn parse_from_file_py(fname : &str) -> Option<Vec<Self>> {
    let times = Self::parse_from_file(fname);
    if times.is_ok() {
      return Some(times.unwrap());
    } else {
      error!("An error occured when parsing {}", fname);
      return None;
    }
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
  fn get_meta    (&self) -> Option<String> {
    self.meta.clone()
  }  

}

#[cfg(feature="pybindings")]
pythonize!(TrackerCaliTimeWindow);





