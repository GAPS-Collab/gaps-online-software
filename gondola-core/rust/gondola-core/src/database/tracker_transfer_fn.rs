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

/// Tracker transfer functions connect the tracker adc to a measurement of energy
#[derive(Debug,PartialEq, Clone,Queryable, Selectable, serde::Serialize, serde::Deserialize)]
#[diesel(table_name = schema::tof_db_trackerstriptransferfunction)]
#[diesel(primary_key(data_id))]
#[allow(non_snake_case)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TrackerStripTransferFunction {  
    pub data_id            : i32,
    pub strip_id           : i32,    
    pub volume_id          : i64,    
    pub utc_timestamp_start: i64,    
    pub utc_timestamp_stop : i64,    
    pub name               : Option<String>, 
    pub pol_a2_0           : f32, 
    pub pol_a2_1           : f32,    
    pub pol_a2_2           : f32, 
    pub pol_b3_0           : f32, 
    pub pol_b3_1           : f32, 
    pub pol_b3_2           : f32, 
    pub pol_b3_3           : f32, 
    pub pol_c3_0           : f32, 
    pub pol_c3_1           : f32, 
    pub pol_c3_2           : f32, 
    pub pol_c3_3           : f32, 
    pub pol_d3_0           : f32,     
    pub pol_d3_1           : f32, 
    pub pol_d3_2           : f32, 
    pub pol_d3_3           : f32, 
} 

impl TrackerStripTransferFunction {

  pub fn new() -> Self {
    Self {
      data_id             : 0,
      strip_id            : 0,    
      volume_id           : 0,    
      utc_timestamp_start : 0,    
      utc_timestamp_stop  : 0,
      name                : None, 
      pol_a2_0            : 0.0, 
      pol_a2_1            : 0.0,    
      pol_a2_2            : 0.0, 
      pol_b3_0            : 0.0, 
      pol_b3_1            : 0.0, 
      pol_b3_2            : 0.0, 
      pol_b3_3            : 0.0, 
      pol_c3_0            : 0.0, 
      pol_c3_1            : 0.0, 
      pol_c3_2            : 0.0, 
      pol_c3_3            : 0.0, 
      pol_d3_0            : 0.0,     
      pol_d3_1            : 0.0, 
      pol_d3_2            : 0.0, 
      pol_d3_3            : 0.0, 
    }
  }
  
  pub fn parse_from_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<Self>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut transfer_fns = Vec::<Self>::new();
    let hid_vid_map = get_hid_vid_maps().unwrap().1;
    for line in reader.lines() {
      let line = line?;
      if line.starts_with("#") || line.starts_with("Layer") || line.starts_with("layer") {
        continue;
      }
      let mut trfn = Self::new();
      let parts: Vec<&str> = line.split(",").collect();
      if parts.len() == 19 {
        // Parse the first number as a standard decimal
        let layer   = parts[0].parse::<u8>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let row     = parts[1].parse::<u8>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let module   = parts[2].parse::<u8>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let channel  = parts[3].parse::<u8>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        trfn.strip_id  = TrackerStrip::create_stripid(layer, row, module, channel) as i32; 
        trfn.volume_id = *hid_vid_map.get(&(trfn.strip_id as u32)).unwrap() as i64; // critical error is good here,
        trfn.pol_a2_0  = parts[4].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        trfn.pol_a2_1  = parts[5].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        trfn.pol_a2_2  = parts[6].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        trfn.pol_b3_0  = parts[7].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        trfn.pol_b3_1  = parts[8].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        trfn.pol_b3_2  = parts[9].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        trfn.pol_b3_3  = parts[10].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        trfn.pol_c3_0  = parts[11].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        trfn.pol_c3_1  = parts[12].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        trfn.pol_c3_2  = parts[13].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        trfn.pol_c3_3  = parts[14].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        trfn.pol_d3_0  = parts[15].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        trfn.pol_d3_1  = parts[16].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        trfn.pol_d3_2  = parts[17].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        trfn.pol_d3_3  = parts[18].parse::<f32>()
          .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        transfer_fns.push(trfn); 
      }
    }
    Ok(transfer_fns)
  }
  
  /// Get Tracker strip transfer fns for a certain dataset 
  ///
  /// # Returns:
  ///   * HashMap<u32 [strip id], TrackerStripTransferFn> 
  pub fn as_dict_by_name(fname : &str) -> Result<HashMap<u32,Self>, ConnectionError> {
    use schema::tof_db_trackerstriptransferfunction::dsl::*;
    let mut strips = HashMap::<u32, Self>::new();
    if fname == "" {
      match Self::all() {
        None => {
          error!("Unable to retrive ANY TrackerStripTransferFunction");
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
    match tof_db_trackerstriptransferfunction.filter(
      schema::tof_db_trackerstriptransferfunction::name.eq(fname)).load::<Self>(&mut conn) {
      Err(err) => {
        error!("We can't find any tracker strip transferfunction in the database! {err}");
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
  
  pub fn all_names() -> Result<Vec<String>, ConnectionError> {
    let mut conn = connect_to_db()?;
    let mut names = Vec::<String>::new();
    let unique_names =
      schema::tof_db_trackerstriptransferfunction::table.select(
      schema::tof_db_trackerstriptransferfunction::name)
      .distinct()
      .load::<Option<String>>(&mut conn).expect("Error getting names from db!");
    for k in unique_names {
      if let Some(n) = k {
        names.push(n);
      }
    }
    Ok(names)
  }

  /// Get all tracker strip transfer functions from the database
  ///
  /// # Returns:
  ///   * HashMap<u32 [strip id], TrackeStripTransferFunction> 
  pub fn all() -> Option<Vec<Self>> {
    use schema::tof_db_trackerstriptransferfunction::dsl::*;
    let mut conn = connect_to_db().ok()?;
    match tof_db_trackerstriptransferfunction.load::<Self>(&mut conn) {
      Err(err) => {
        error!("Unable to load tracker transfer functions from db! {err}");
        return None;
      }
      Ok(strips) => {
        return Some(strips);
      }
    }
  }

  /// The actual transfer function for this 
  /// strip. Calculate energy from adc values
  pub fn transfer_fn(&self, adc : f32) -> f32 {
    if adc < 0.0 {
      return 0.0;
    }
    if adc <= 190.0 {
      return self.pol_a2_0 + self.pol_a2_1*adc + self.pol_a2_2*(adc.powi(2));
    }
    if 190.0 < adc && adc <= 500.0 {
      return self.pol_b3_0 + self.pol_b3_1*adc + self.pol_b3_2*(adc.powi(2)) + self.pol_b3_3*(adc.powi(3));
    }
    if 500.0 < adc && adc <= 900.0 {
      return self.pol_c3_0 + self.pol_c3_1*adc + self.pol_c3_2*(adc.powi(2)) + self.pol_c3_3*(adc.powi(3));
    }
    //if 900.0 < adc && adc <= 2047.0 {
    if 900.0 < adc && adc <= 1600.0 {
      return self.pol_d3_0 + self.pol_d3_1*adc + self.pol_d3_2*(adc.powi(2)) + self.pol_d3_3*(adc.powi(3));
    }
    0.0
  }
}

impl fmt::Display for TrackerStripTransferFunction {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = format!("<TrackerStripTransferFunction [{}]:", self.strip_id);
    repr += &(format!("\n   vid           : {}", self.volume_id));
    repr += "\n   UTC Timestamps (Begin/End):";
    repr += &(format!("\n   {}/{}", self.utc_timestamp_start, self.utc_timestamp_stop));    
    if self.name.is_some() {
      repr += &(format!("\n   name     : {}", self.name.clone().unwrap())); 
    }
    repr += &(format!("\n  Poly A {}*adc + {}*adc + {}*(adc**2) for adc < 190", self.pol_a2_0, self.pol_a2_1, self.pol_a2_2));
    repr += &(format!("\n  Poly B    :{}*adc + {}*adc + {}*(adc**2) + {}*(adc**3) for 190 < adc <= 500", self.pol_b3_0, self.pol_b3_1, self.pol_b3_2, self.pol_b3_3));
    repr += &(format!("\n  Poly C    :{}*adc + {}*adc + {}*(adc**2) + {}*(adc**3) for 500 < adc <= 900", self.pol_c3_0, self.pol_c3_1, self.pol_c3_2, self.pol_c3_3));
    repr += &(format!("\n  Poly D    :{}*adc + {}*adc + {}*(adc**2) + {}*(adc**3) for 900 < adc <= 1600>", self.pol_d3_0, self.pol_d3_1, self.pol_d3_2, self.pol_d3_3));
    write!(f, "{}", repr)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TrackerStripTransferFunction {
  
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
        error!("Unable to retrieve tracker strip transfer fn dictionary. {err}. Did you laod the setup-env.sh shell?");
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

  #[pyo3(name="transfer_fn")]
  fn transfer_fn_py(&self, adc : f32) -> f32 {
    if adc > 1600.0 {
      warn!("ADC value larger than 1600! {}. Transfer fn not defined beyond 1600.", adc);
    }
    return self.transfer_fn(adc);
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
pythonize!(TrackerStripTransferFunction);

//-------------------------------------------------

#[derive(Debug,PartialEq, Clone, Insertable)]
#[diesel(table_name = schema::tof_db_trackerstriptransferfunction)]
#[allow(non_snake_case)]
#[cfg_attr(feature="pybindings", pyclass)]
struct NewTrackerStripTransferFunction {  
  pub strip_id           : i32,    
  pub volume_id          : i64,    
  pub utc_timestamp_start: i64,    
  pub utc_timestamp_stop : i64,    
  pub name               : Option<String>, 
  pub pol_a2_0           : f32, 
  pub pol_a2_1           : f32,    
  pub pol_a2_2           : f32, 
  pub pol_b3_0           : f32, 
  pub pol_b3_1           : f32, 
  pub pol_b3_2           : f32, 
  pub pol_b3_3           : f32, 
  pub pol_c3_0           : f32, 
  pub pol_c3_1           : f32, 
  pub pol_c3_2           : f32, 
  pub pol_c3_3           : f32, 
  pub pol_d3_0           : f32,     
  pub pol_d3_1           : f32, 
  pub pol_d3_2           : f32, 
  pub pol_d3_3           : f32, 
} 

impl NewTrackerStripTransferFunction {
  pub fn from(tf : &TrackerStripTransferFunction) -> Self {
    Self {
      strip_id            : tf.strip_id            ,    
      volume_id           : tf.volume_id           ,    
      utc_timestamp_start : tf.utc_timestamp_start ,    
      utc_timestamp_stop  : tf.utc_timestamp_stop  ,    
      name                : tf.name.clone()        , 
      pol_a2_0            : tf.pol_a2_0            , 
      pol_a2_1            : tf.pol_a2_1            ,    
      pol_a2_2            : tf.pol_a2_2            , 
      pol_b3_0            : tf.pol_b3_0            , 
      pol_b3_1            : tf.pol_b3_1            , 
      pol_b3_2            : tf.pol_b3_2            , 
      pol_b3_3            : tf.pol_b3_3            , 
      pol_c3_0            : tf.pol_c3_0            , 
      pol_c3_1            : tf.pol_c3_1            , 
      pol_c3_2            : tf.pol_c3_2            , 
      pol_c3_3            : tf.pol_c3_3            , 
      pol_d3_0            : tf.pol_d3_0            ,     
      pol_d3_1            : tf.pol_d3_1            , 
      pol_d3_2            : tf.pol_d3_2            , 
      pol_d3_3            : tf.pol_d3_3            , 
    }
  }
}

#[cfg_attr(feature="pybindings", pyfunction)]
pub fn create_trk_transfer_fn_table( db_path: &str, transfer_fns: Vec<TrackerStripTransferFunction>) { 
  use schema::tof_db_trackerstriptransferfunction::dsl::*;
  let mut conn = SqliteConnection::establish(db_path).ok().unwrap(); 
  let mut _query_result = diesel::sql_query("
      CREATE TABLE IF NOT EXISTS tof_db_trackerstriptransferfunction (
          data_id INTEGER PRIMARY KEY AUTOINCREMENT,
          strip_id INTEGER NOT NULL,
          volume_id BIGINT NOT NULL,
          utc_timestamp_start BIGINT NOT NULL,
          utc_timestamp_stop BIGINT NOT NULL,
          name TEXT,
          pol_a2_0 FLOAT NOT NULL, 
          pol_a2_1 FLOAT NOT NULL,    
          pol_a2_2 FLOAT NOT NULL, 
          pol_b3_0 FLOAT NOT NULL, 
          pol_b3_1 FLOAT NOT NULL, 
          pol_b3_2 FLOAT NOT NULL, 
          pol_b3_3 FLOAT NOT NULL, 
          pol_c3_0 FLOAT NOT NULL, 
          pol_c3_1 FLOAT NOT NULL, 
          pol_c3_2 FLOAT NOT NULL, 
          pol_c3_3 FLOAT NOT NULL, 
          pol_d3_0 FLOAT NOT NULL,     
          pol_d3_1 FLOAT NOT NULL, 
          pol_d3_2 FLOAT NOT NULL, 
          pol_d3_3 FLOAT NOT NULL 
      )
  ").execute(&mut conn);
  let mut new_data = Vec::<NewTrackerStripTransferFunction>::new();
  for p in transfer_fns {
    let np = NewTrackerStripTransferFunction::from(&p);
    new_data.push(np);
  }
  println!("Will insert {} tfns!", new_data.len());
  _query_result = diesel::insert_into(tof_db_trackerstriptransferfunction)
    .values(&new_data)
    .execute(&mut conn);
  println!("Inserted {}", _query_result.unwrap());
}

