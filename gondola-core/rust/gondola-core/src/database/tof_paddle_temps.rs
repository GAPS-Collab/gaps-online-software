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
          paddlie_id          SMALLINT NOT NULL, 
          utc_timestamp       BIGINT NOT NULLt,
          temp_a              FLOAT NOT NULL,
          temp_b              FLOAT NOT NULL,
          meta TEXT
      )
  ").execute(&mut conn);
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
 
  /// Retrieve all paddle calibration times as they are 
  /// stroed in the database
  ///
  /// # Returns:
  ///   * Vec<Self>  : All paddletimes as they are 
  ///                  stored in the db
  pub fn all() -> Option<Vec<Self>> {
    use schema::tof_db_tofpaddletemp::dsl::*;
    let mut conn = connect_to_db().ok()?;
    match tof_db_tofpaddletemp.load::<Self>(&mut conn) {
      Err(err) => {
        error!("Unable to load TOF paddle temperature data from db! {err}");
        return None;
      }
      Ok(cali_times) => {
        return Some(cali_times);
      }
    }
  }

  pub fn from_telemetry_packet(pack : &TelemetryPacket) -> Self { 
    let mut tpt = Self::new();
    tpt.utc_timestamp = pack.header.get_gcutime() as i64;
    return tpt;
  }


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
impl TofPaddleTemp {
  
  #[staticmethod]
  #[pyo3(name="all")]
  pub fn all_py() -> Option<Vec<Self>> {
    Self::all()
  } 
  

  #[getter]
  fn get_utc_timestamp(&self) -> i64 {
    self.utc_timestamp
  }
   
  #[getter]
  fn get_meta    (&self) -> Option<String> {
    self.meta.clone()
  }  
}

#[cfg(feature="pybindings")]
pythonize!(TofPaddleTemp);

