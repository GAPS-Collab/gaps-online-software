//! Compact data structures for analysis. Won't get serialized. 
//! Summarize data of all boards and provide a more consice
//! table-like structure for analysis.
//! Currently under a lot of development. The goal is to get
//! something which easily translates to a polars data frame.

use std::fmt;

use std::collections::{
    HashMap,
    VecDeque,
};

use crate::monitoring::{
    MoniData,
    RBMoniData,
    LTBMoniData,
    PAMoniData,
    PBMoniData,
    MtbMoniData, 
    CPUMoniData,
};

#[cfg(feature = "polars")]
use polars::prelude::*;

/// A MoniSeries is a collection of (primarily) monitoring
/// data, which comes from multiple senders.
/// E.g. a MoniSeries could hold RBMoniData from all 
/// 40 ReadoutBoards.
pub trait MoniSeries<T>
  where T : Copy + MoniData {

  fn get_data(&self) -> &HashMap<u8,VecDeque<T>>;

  fn get_data_mut(&mut self) -> &mut HashMap<u8,VecDeque<T>>;
 
  fn get_max_size(&self) -> usize;

  /// A HashMap of -> rbid, Vec\<var\> 
  fn get_var(&self, varname : &str) -> HashMap<u8, Vec<f32>> {
    let mut values = HashMap::<u8, Vec<f32>>::new();
    for k in self.get_data().keys() {
      match self.get_var_for_board(varname, k) {
        None => (),
        Some(vals) => {
          values.insert(*k, vals);
        }
      }
      //values.insert(*k, Vec::<f32>::new());
      //match self.get_data().get(k) {
      //  None => (),
      //  Some(vec_moni) => {
      //    for moni in vec_moni {
      //      match moni.get(varname) {
      //        None => (),
      //        Some(val) => {
      //          values.get_mut(k).unwrap().push(val);
      //        }
      //      }
      //    }
      //  }
      //}
    }
    values
  }

  /// Get a certain variable, but only for a single board
  fn get_var_for_board(&self, varname : &str, rb_id : &u8) -> Option<Vec<f32>> {
    let mut values = Vec::<f32>::new();
    match self.get_data().get(&rb_id) {
      None => (),
      Some(vec_moni) => {
        for moni in vec_moni {
          match moni.get(varname) {
            None => {
              return None;
            },
            Some(val) => {
              values.push(val);
            }
          }
        }
      }
    }
    // FIXME This needs to be returning a reference,
    // not cloning
    Some(values)
  }

  #[cfg(feature = "polars")]
  fn get_dataframe(&self) -> PolarsResult<DataFrame> {
    let mut series = Vec::<Series>::new();
    for k in Self::keys() {
      match self.get_series(k) {
        None => {
          error!("Unable to get series for {}", k);
        }
        Some(ser) => {
          series.push(ser);
        }
      }
    }
    let df = DataFrame::new(series)?;
    Ok(df)
  }

  #[cfg(feature = "polars")]
  /// Get the variable for all boards. This keeps the order of the 
  /// underlying VecDeque. Values of all boards intermixed.
  /// To get a more useful version, use the Dataframe instead.
  ///
  /// # Arguments
  ///
  /// * varname : The name of the attribute of the underlying
  ///             moni structure
  fn get_series(&self, varname : &str) -> Option<Series> {
    let mut data = Vec::<f32>::with_capacity(self.get_data().len());
    for rbid in self.get_data().keys() {
      let dqe = self.get_data().get(rbid).unwrap(); //uwrap is fine, bc we checked earlier
      for moni in dqe {
        match moni.get(varname) {
          None => {
            error!("This type of MoniData does not have a key called {}", varname);
            return None;
          }
          Some(var) => {
            data.push(var);
          }
        }
      }
    }
    let series = Series::new(varname.into(), data);
    Some(series)
  }

  /// A list of the variables in this MoniSeries
  fn keys() -> Vec<&'static str> {
    T::keys()
  }

  /// A list of boards in this series
  fn get_board_ids(&self) -> Vec<u8> {
    self.get_data().keys().cloned().collect()
  }

  /// Add another instance of the data container to the series
  fn add(&mut self, data : T) {
    let board_id = data.get_board_id();
    if !self.get_data().contains_key(&board_id) {
      self.get_data_mut().insert(board_id, VecDeque::<T>::new());
    } 
    self.get_data_mut().get_mut(&board_id).unwrap().push_back(data);
    if self.get_data_mut().get_mut(&board_id).unwrap().len() > self.get_max_size() {
      error!("The queue is too large, returning the first element! If you need a larger series size, set the max_size field");
      self.get_data_mut().get_mut(&board_id).unwrap().pop_front();
    }
  }
  
  fn get_last_moni(&self, board_id : u8) -> Option<T> {
    let size = self.get_data().get(&board_id)?.len();
    Some(self.get_data().get(&board_id).unwrap()[size - 1])
  }
}

////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
pub struct RBMoniDataSeries {
  data        : HashMap<u8, VecDeque<RBMoniData>>,
  max_size    : usize
}

impl RBMoniDataSeries {
  pub fn new() -> Self {
    Self {
      data     : HashMap::<u8, VecDeque::<RBMoniData>>::new(),
      max_size : 10000
    }
  }
}

impl Default for RBMoniDataSeries {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for RBMoniDataSeries {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, ">RBMoniDataSeries : {} boards>", self.data.len())
  }
}

impl MoniSeries<RBMoniData> for RBMoniDataSeries {
  fn get_data(&self) -> &HashMap<u8,VecDeque<RBMoniData>> {
    return &self.data;
  }

  fn get_data_mut(&mut self) -> &mut HashMap<u8,VecDeque<RBMoniData>> {
    return &mut self.data;
  }
 
  fn get_max_size(&self) -> usize {
    return self.max_size;
  }
}

////////////////////////////////////////////////////////////////////


#[derive(Debug, Clone, PartialEq)]
pub struct PAMoniDataSeries {
  data        : HashMap<u8, VecDeque<PAMoniData>>,
  max_size    : usize,
}

impl PAMoniDataSeries {
  pub fn new() -> Self {
    Self {
      data     : HashMap::<u8, VecDeque<PAMoniData>>::new(),
      max_size : 10000,
    }
  }
} 

impl Default for PAMoniDataSeries {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for PAMoniDataSeries {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<PAMoniDataSeries : {} boards>", self.data.len())
  }
}

//impl<T: std::marker::Copy + HasBoardId> Series<T> for PAMoniDataSeries {
impl MoniSeries<PAMoniData> for PAMoniDataSeries {

  fn get_data(&self) -> &HashMap<u8,VecDeque<PAMoniData>> {
    return &self.data;
  }

  fn get_data_mut(&mut self) -> &mut HashMap<u8,VecDeque<PAMoniData>> {
    return &mut self.data;
  }
 
  fn get_max_size(&self) -> usize {
    return self.max_size;
  }
}


#[derive(Debug, Clone, PartialEq)]
pub struct PBMoniDataSeries {
  data        : HashMap<u8, VecDeque<PBMoniData>>,
  max_size    : usize,
}

impl PBMoniDataSeries {
  pub fn new() -> Self {
    Self {
      data     : HashMap::<u8, VecDeque<PBMoniData>>::new(),
      max_size : 10000,
    }
  }
} 

impl Default for PBMoniDataSeries {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for PBMoniDataSeries {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<PBMoniDataSeries : {} boards>", self.data.len())
  }
}

impl MoniSeries<PBMoniData> for PBMoniDataSeries {

  fn get_data(&self) -> &HashMap<u8,VecDeque<PBMoniData>> {
    return &self.data;
  }

  fn get_data_mut(&mut self) -> &mut HashMap<u8,VecDeque<PBMoniData>> {
    return &mut self.data;
  }
 
  fn get_max_size(&self) -> usize {
    return self.max_size;
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LTBMoniDataSeries {
  data        : HashMap<u8, VecDeque<LTBMoniData>>,
  max_size    : usize,
}

impl LTBMoniDataSeries {
  pub fn new() -> Self {
    Self {
      data     : HashMap::<u8, VecDeque<LTBMoniData>>::new(),
      max_size : 10000,
    }
  }
} 

impl Default for LTBMoniDataSeries {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for LTBMoniDataSeries {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<LTBMoniDataSeries : {} boards>", self.data.len())
  }
}

impl MoniSeries<LTBMoniData> for LTBMoniDataSeries {

  fn get_data(&self) -> &HashMap<u8,VecDeque<LTBMoniData>> {
    return &self.data;
  }

  fn get_data_mut(&mut self) -> &mut HashMap<u8,VecDeque<LTBMoniData>> {
    return &mut self.data;
  }
 
  fn get_max_size(&self) -> usize {
    return self.max_size;
  }
}

////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
pub struct MtbMoniDataSeries {
  data        : HashMap<u8, VecDeque<MtbMoniData>>,
  max_size    : usize,
}

impl MtbMoniDataSeries {
  pub fn new() -> Self {
    Self {
      data     : HashMap::<u8, VecDeque<MtbMoniData>>::new(),
      max_size : 10000,
    }
  }
} 

impl Default for MtbMoniDataSeries {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for MtbMoniDataSeries {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<MtbMoniDataSeries : {} boards>", self.data.len())
  }
}

impl MoniSeries<MtbMoniData> for MtbMoniDataSeries {

  fn get_data(&self) -> &HashMap<u8,VecDeque<MtbMoniData>> {
    return &self.data;
  }

  fn get_data_mut(&mut self) -> &mut HashMap<u8,VecDeque<MtbMoniData>> {
    return &mut self.data;
  }
 
  fn get_max_size(&self) -> usize {
    return self.max_size;
  }
}

////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
pub struct CPUMoniDataSeries {
  data        : HashMap<u8, VecDeque<CPUMoniData>>,
  max_size    : usize,
}

impl CPUMoniDataSeries {
  pub fn new() -> Self {
    Self {
      data     : HashMap::<u8, VecDeque<CPUMoniData>>::new(),
      max_size : 10000,
    }
  }
} 

impl Default for CPUMoniDataSeries {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Display for CPUMoniDataSeries {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<CPUMoniDataSeries : {} boards>", self.data.len())
  }
}

impl MoniSeries<CPUMoniData> for CPUMoniDataSeries {

  fn get_data(&self) -> &HashMap<u8,VecDeque<CPUMoniData>> {
    return &self.data;
  }

  fn get_data_mut(&mut self) -> &mut HashMap<u8,VecDeque<CPUMoniData>> {
    return &mut self.data;
  }
 
  fn get_max_size(&self) -> usize {
    return self.max_size;
  }
}


