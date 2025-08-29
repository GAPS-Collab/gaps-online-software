// This file is part of gaps-online-software and published 
// under the GPLv3 license

pub mod pa_moni_data;
pub use pa_moni_data::{
  PAMoniData,
  PAMoniDataSeries
};
pub mod pb_moni_data;
pub use pb_moni_data::{
  PBMoniData,
  PBMoniDataSeries
};
pub mod mtb_moni_data;
pub use mtb_moni_data::{
  MtbMoniData,
  MtbMoniDataSeries
};
pub mod ltb_moni_data;
pub use ltb_moni_data::{
  LTBMoniData,
  LTBMoniDataSeries
};
pub mod rb_moni_data;
pub use rb_moni_data::{
  RBMoniData,
  RBMoniDataSeries
};
pub mod cpu_moni_data;
pub use cpu_moni_data::{
  CPUMoniData,
  CPUMoniDataSeries
};

pub mod heartbeats;
pub use heartbeats::{
  DataSinkHB,
  DataSinkHBSeries,
  MasterTriggerHB,
  MasterTriggerHBSeries,
  EventBuilderHB,
  EventBuilderHBSeries,
};

pub mod run_statistics;
pub use run_statistics::RunStatistics;

use std::collections::VecDeque;
use std::collections::HashMap;

#[cfg(feature = "polars")]
use polars::prelude::*;


/// Monitoring data shall share the same kind 
/// of interface. 
pub trait MoniData {
  /// Monitoring data is always tied to a specific
  /// board. This might not be its own board, but 
  /// maybe the RB the data was gathered from
  /// This is an unique identifier for the 
  /// monitoring data
  fn get_board_id(&self) -> u8;
  
  /// Access the (data) members by name 
  fn get(&self, varname : &str) -> Option<f32>;

  /// A list of the variables in this MoniData
  fn keys() -> Vec<&'static str>;
}

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
    let mut series = Vec::<Column>::new();
    for k in Self::keys() {
      match self.get_series(k) {
        None => {
          error!("Unable to get series for {}", k);
        }
        Some(ser) => {
          series.push(ser.into());
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

//--------------------------------------------------

/// Implements the moniseries trait for a MoniData 
/// type of class
#[macro_export]
macro_rules! moniseries {
  ($name : ident, $class:ty) => {
    
    use std::collections::VecDeque;
    use std::collections::HashMap;

    use crate::monitoring::MoniSeries;

    #[cfg_attr(feature="pybindings",pyclass)]
    #[derive(Debug, Clone, PartialEq)]
    pub struct $name {
      data        : HashMap<u8, VecDeque<$class>>,
      max_size    : usize,
    }
    
    impl $name {
      pub fn new() -> Self {
        Self {
          data     : HashMap::<u8, VecDeque<$class>>::new(),
          max_size : 10000,
        }
      }
    } 
    
    impl Default for $name {
      fn default() -> Self {
        Self::new()
      }
    }
    
    impl fmt::Display for $name {
      fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{} : {} boards>", stringify!($name), self.data.len())
      }
    }
    
    impl MoniSeries<$class> for $name {
    
      fn get_data(&self) -> &HashMap<u8,VecDeque<$class>> {
        return &self.data;
      }
    
      fn get_data_mut(&mut self) -> &mut HashMap<u8,VecDeque<$class>> {
        return &mut self.data;
      }
     
      fn get_max_size(&self) -> usize {
        return self.max_size;
      }
    }
  }
}

