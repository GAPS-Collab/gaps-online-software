// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;
use crate::database::schema;

use diesel::prelude::*;
use std::io::BufRead;

#[cfg_attr(feature="pybindings", pyfunction)]
pub fn create_tof_paddle_temp_table( db_path: &str, paddle_temps: Vec<TofPaddleTemp>) { 
  use schema::tof_db_tofpaddletemp::dsl::*;
  let mut conn = SqliteConnection::establish(db_path).ok().unwrap();
  
  let mut _query_result = diesel::sql_query("
      CREATE TABLE IF NOT EXISTS tof_db_tofpaddletemp (
          data_id INTEGER PRIMARY KEY AUTOINCREMENT,
          paddle_id SMALLINT NOT NULL, 
          utc_timestamp BIGINT NOT NULL,
          temp_a FLOAT NOT NULL,
          temp_b FLOAT NOT NULL,
          meta TEXT
      )
  ").execute(&mut conn);
  match _query_result {
    Ok(_) => {
      println!("Created table successfully!");
    }
    Err(err) => {
      println!("Error occured when creating tof_db_tofpaddletemp table! {err}");
    }
  }
  _query_result = diesel::insert_into(tof_db_tofpaddletemp)
    .values(&paddle_temps)
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

#[derive(Debug, PartialEq, Clone)]
#[allow(non_snake_case)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TofPaddleTempDataSeries {
  pub temp_a         : Vec<f32>,
  pub temp_b         : Vec<f32>,
  pub utc_timestamps : Vec<u64>,
  pub paddle_id      : u8
}

impl TofPaddleTempDataSeries {
  pub fn new() -> Self {
    Self {
      temp_a         : Vec::<f32>::new(),
      temp_b         : Vec::<f32>::new(),
      utc_timestamps : Vec::<u64>::new(),
      paddle_id      : 0
    }
  }

  pub fn add(&mut self, pdl_t : &TofPaddleTemp) {
    if pdl_t.paddle_id == self.paddle_id as i16 {
      self.temp_a.push(pdl_t.temp_a);
      self.temp_b.push(pdl_t.temp_b);
      self.utc_timestamps.push(pdl_t.utc_timestamp as u64);
    }
  }

  pub fn get_for_ts(&self, utc_timestamp : u64) -> (f32, f32) {
    let mut result = (-273.0, -273.0); 
    let idx_opt = self.utc_timestamps.partition_point(|&t| t < utc_timestamp)
        .checked_sub(1);
    if let Some(idx) = idx_opt {
      if self.temp_a.len() > idx as usize && self.temp_b.len() > idx {
        result.0 = self.temp_a[idx]; 
        result.1 = self.temp_b[idx];
      }
    }
    result
  }

  pub fn sort(&mut self) {
    let mut indices: Vec<usize> = (0..self.utc_timestamps.len()).collect();
    indices.sort_unstable_by_key(|&i| self.utc_timestamps[i]);
    let timestamps_sorted: Vec<u64> =
      indices.iter().map(|&i| self.utc_timestamps[i]).collect();
    let values_a_sorted: Vec<f32> =
      indices.iter().map(|&i| self.temp_a[i]).collect();
    let values_b_sorted: Vec<f32> =
      indices.iter().map(|&i| self.temp_b[i]).collect();
    self.utc_timestamps = timestamps_sorted;
    self.temp_a = values_a_sorted;
    self.temp_b = values_b_sorted;
    //self.utc_timestamps.sort_unstable();
  }
}

impl fmt::Display for TofPaddleTempDataSeries {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TofPaddleTempDataSeries:");
    repr += &(format!("\n  Paddle ID : {}", self.paddle_id));
    repr += &(format!("\n  Values    : {}", self.temp_a.len()));
    repr += ">";
    write!(f, "{}", repr)
  }
}

/// Recorded Temperature of each paddle end. The SiPM preamp board 
/// is equipped with a temeperature sensor, which allows to calibrate 
/// the gain of the SiPm
#[derive(Debug,PartialEq, Clone, Queryable, Selectable, Insertable, serde::Serialize, serde::Deserialize)]
#[diesel(table_name = schema::tof_db_tofpaddletemp)]
#[diesel(primary_key(data_id))]
#[allow(non_snake_case)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TofPaddleTemp {
  #[diesel(deserialize_as = i32)]
  pub data_id             : Option<i32>,
  pub paddle_id           : i16, 
  pub utc_timestamp       : i64,
  pub temp_a              : f32,
  pub temp_b              : f32,
  pub meta                : Option<String>, 
}

impl TofPaddleTemp {

  pub fn new() -> Self {
    Self {
      data_id             : None,
      paddle_id           : 0,
      utc_timestamp       : 0,
      temp_a              : 0.0,
      temp_b              : 0.0,
      meta                : None, 
    }
  }
 
  /// Retrieve all paddle temperatures
  /// stroed in the database
  ///
  /// # Returns:
  ///   * Vec<Self>  : All paddle temperatures as they are 
  ///                  stored in the db
  pub fn all() -> Option<Vec<Self>> {
    use schema::tof_db_tofpaddletemp::dsl::*;
    let mut conn = connect_to_db().ok()?;
    match tof_db_tofpaddletemp.load::<Self>(&mut conn) {
      Err(err) => {
        error!("Unable to load TOF paddle temperature data from db! {err}");
        return None;
      }
      Ok(tpts) => {
        return Some(tpts);
      }
    }
  }
  
  /// Retrieve all paddle temperatures for a specific paddle id
  ///
  /// # Returns:
  ///   * Vec<Self>  : All paddle tempereaturs as they are 
  ///                  stored in the db for this paddle id
  pub fn all_data() -> Option<HashMap<u8,TofPaddleTempDataSeries>> {
    use schema::tof_db_tofpaddletemp::dsl::*;
    let mut conn = connect_to_db().ok()?;
    let results = tof_db_tofpaddletemp
      //.filter(paddle_id.eq(pid as i16))
      .load::<Self>(&mut conn);
    let mut all_ds = HashMap::<u8, TofPaddleTempDataSeries>::new();
    for k in 1..161u8 {
      let mut new_series = TofPaddleTempDataSeries::new();
      new_series.paddle_id = k;
      all_ds.insert(k, new_series); 
    }
    
    match results {
      Err(err) => {
        error!("Unable to load TOF paddle temperature data from db! {err}");
        return None;
      }
      Ok(tpts) => {
        for tp in tpts {
          all_ds.get_mut(&(tp.paddle_id as u8)).unwrap().add(&tp);
          //ds.add(&tp);
        }
      }
    }
    for k in 1..161 {
      all_ds.get_mut(&k).unwrap().sort(); 
    }
    //return Some(tpts);
    return Some(all_ds);
  }

  //pub fn from_telemetry_packet(pack : &TelemetryPacket) -> Self { 
  //  let mut tpt = Self::new();
  //  tpt.utc_timestamp = pack.header.get_gcutime() as i64;
  //  return tpt;
  //}


  //pub fn from_pa_monidata(&PAMoniData) -> Self {
  //  let mut tpt = Self::new();  
  //  tpt
  //  return tpt;
  //}
}

impl Default for TofPaddleTemp {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for TofPaddleTemp {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<TofPaddleTemp:");
    repr += &(format!("\n  Paddle ID     : {}", self.paddle_id));
    repr += &(format!("\n  UTC Timstamp  : {} [{}|]", self.utc_timestamp, get_utc_timestamp_from_unix(self.utc_timestamp as f64).unwrap_or(String::from("0"))));    
    repr += &(format!("\n  Temp [C]      : A {} // B {}", self.temp_a, self.temp_b)); 
    if self.meta.is_some() {
      repr += &(format!("\n  meta info     : {}", self.meta.clone().unwrap())); 
    }
    repr += ">";
    write!(f, "{}", repr)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TofPaddleTempDataSeries {
  #[pyo3(name="get_for_ts")]
  pub fn get_for_ts_py(&self, timestamp : u64) -> (f32, f32) {
    self.get_for_ts(timestamp)
  }
  
  #[getter]
  fn get_first_ts(&self) -> Option<&u64> {
    self.utc_timestamps.first() 
  }
  
  #[getter]
  fn get_last_ts(&self) -> Option<&u64> {
    self.utc_timestamps.last() 
  }
}

#[cfg(feature="pybindings")]
pythonize!(TofPaddleTempDataSeries);

#[cfg(feature="pybindings")]
#[pymethods]
impl TofPaddleTemp {
  
  #[staticmethod]
  #[pyo3(name="all")]
  pub fn all_py() -> Option<Vec<Self>> {
    Self::all()
  } 
  
  /// Retrieve all paddle temperatures for a specific paddle id
  ///
  /// # Returns:
  ///   * Vec<Self>  : All paddle tempereaturs as they are 
  ///                  stored in the db for this paddle id
  #[staticmethod]
  #[pyo3(name="all_data")]
  fn all_data_py() -> Option<HashMap<u8,TofPaddleTempDataSeries>> {
    Self::all_data()
  } 

  #[getter]
  fn get_utc_timestamp(&self) -> i64 {
    self.utc_timestamp
  }
  
  #[setter]
  fn set_utc_timestamp(&mut self, ts : i64) {
    self.utc_timestamp = ts;
  }
   
  #[getter]
  fn get_meta    (&self) -> Option<String> {
    self.meta.clone()
  }  
  
  #[getter]
  fn get_paddle_id(&self) -> i16 { 
    self.paddle_id 
  }
  
  #[getter]
  fn get_temp_a(&self)    -> f32 { 
    self.temp_a 
  }
  
  #[getter]
  fn get_temp_b(&self)    -> f32 {
    self.temp_b
  }
  
  #[setter]
  fn set_paddle_id(&mut self, pid : i16) { 
    self.paddle_id = pid;
  }
  
  #[setter]
  fn set_temp_a(&mut self, t : f32) { 
    self.temp_a = t;
  }
  
  #[setter]
  fn set_temp_b(&mut self, t : f32) {
    self.temp_b = t;
  }
}

#[cfg(feature="pybindings")]
pythonize!(TofPaddleTemp);

