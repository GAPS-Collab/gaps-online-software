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

use crate::prelude::*;

#[cfg(feature="pybindings")]
use polars::frame::column::Column;
#[cfg(feature="pybindings")]
use polars::prelude::NamedFrom; 

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

  fn get_timestamp(&self) -> u64 {
    0
  }

  fn set_timestamp(&mut self, ts : u64) {
  
  }
  ///// access the internal timestamps as obtained from 
  ///// MoniDat 
  //fn get_timestamps_mut(&mut self) -> &Vec<u64> {
  //}   


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

  fn get_timestamps(&self) -> &Vec<u64>;

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

  #[cfg(feature = "pybindings")]
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
    //if self.get_timestamps().len() > 0 {
    //  let timestamps  = Series::new("timestamps".into(), self.get_timestamps());
    //  series.push(timestamps.into());
    //}
    // each column is now the specific variable but in terms for 
    // all rbs
    let df = DataFrame::new(series)?;
    Ok(df)
  }

  #[cfg(feature = "pybindings")]
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
    let sorted_keys: Vec<u8> = self.get_data().keys().cloned().collect();
    for rbid in sorted_keys.iter() {
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
      timestamps  : Vec<u64>,
    }
    
    impl $name {
      pub fn new() -> Self {
        Self {
          data       : HashMap::<u8, VecDeque<$class>>::new(),
          max_size   : 10000,
          timestamps : Vec::<u64>::new()
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
    
      fn get_timestamps(&self) -> &Vec<u64> {
        return &self.timestamps;
      }
    }
  
    #[cfg(feature="pybindings")]
    #[pymethods]
    impl $name {
      #[new]
      fn new_py() -> Self {
        Self::new() 
      }
   
      /// The maximum size of the series. If more data 
      /// are added, data from the front will be removed 
      #[getter]
      #[pyo3(name="max_size")]
      fn get_max_size_py(&self) -> usize {
        self.get_max_size()
      }

      /// If monitoring is retrieved from telemetry, we 
      /// save the gcu timestamp of the packet, wich 
      /// herein can be accessed.
      #[getter] 
      #[pyo3(name="timestamps")] 
      fn get_timestamps_py(&self) -> Vec<u64> {
        warn!("This returns a full copy and is a performance bottleneck!");
        return self.timestamps.clone();
      }

      /// Add an additional (Caraspace) file to the series 
      ///
      /// # Arguments:
      ///   * filename    : The name of the (caraspace) file to add
      ///   * from_object : Since this adds caraspace files, it is possible 
      ///                   to choose from where to get the information.
      ///                   Either the telemetry packet, or the tofpacket, if 
      ///                   either is present in the frame. When 
      ///                   CRFrameObjectType = Unknown, it will figure it out 
      ///                   automatically, preferring the telemetry since it has
      ///                   the gcu timestamp
      #[pyo3(signature = (filename, from_object = CRFrameObjectType::TelemetryPacket))]
      fn add_crfile(&mut self, filename : String, from_object : CRFrameObjectType) {
        let reader = CRReader::new(filename).expect("Unable to open file!");
        // now we have a problem - from which frame should we get it?
        // if we get it from the dedicated TOF stream it will be much 
        // faster (if that is available) since it will be it's own 
        // presence in the frame
        //let address = &source.clone();
        //let mut try_from_telly = false;
        let tp_source     = String::from("TofPacketType.") + stringify!($class);
        let tp_source_alt = String::from("PacketType.") + stringify!($class);
        let tel_source    = "TelemetryPacketType.AnyTofHK";
        for frame in reader {
          match from_object { 
            CRFrameObjectType::TofPacket =>  {
              if frame.has(&tp_source) || frame.has(&tp_source_alt) {
                if frame.has(&tp_source) {
                  let moni_res = frame.get::<TofPacket>(&tp_source).unwrap().unpack::<$class>();
                  match moni_res {
                    Err(err) => {
                      println!("Error unpacking! {err}");
                    }
                    Ok(moni) => {
                      self.add(moni);
                    }
                  }
                }
                if frame.has(&tp_source_alt) {
                  let moni_res = frame.get::<TofPacket>(&tp_source_alt).unwrap().unpack::<$class>();
                  match moni_res { 
                    Err(err) => {
                      println!("Error unpacking! {err}");
                    }
                    Ok(moni) => {
                      self.add(moni);
                    }
                  }
                }
              } 
            }
            CRFrameObjectType::TelemetryPacket | CRFrameObjectType::Unknown => {
              if frame.has(tel_source) {
                let hk_res = frame.get::<TelemetryPacket>(tel_source);
                match hk_res {
                  Err(err) => {
                    println!("Error unpacking! {err}");
                  }
                  Ok(hk) => {
                    let mut pos = 0;
                    let gcutime = hk.header.get_gcutime() as u64;
                    match TofPacket::from_bytestream(&hk.payload, &mut pos) {
                      Err(err) => {
                        println!("Error unpackin! {err}");
                      }
                      Ok(tp) => {
                        if tp.packet_type == <$class>::TOF_PACKET_TYPE  {
                          match tp.unpack::<$class>() {
                            Err(err) => {
                              println!("Error unpacking! {err}");
                            }
                            Ok(mut moni) => {
                              moni.set_timestamp(gcutime); 
                              self.add(moni);
                              //self.timestamps.push(gcutime);
                            }
                          }
                        }
                      }
                    } 
                  }
                }
              }
            }
          }
        }
      }

      /// Generate a polars dataframe with monitoring data from the 
      /// given TOF file.
      /// This will load ONLY data of the specific type of the 
      /// MoniSeries itself
      ///
      /// # Arguments:
      ///   * filename : A single .tof.gaps file with monitoring 
      ///                information 
      #[staticmethod]
      fn from_tof_file(filename : String) -> PyResult<PyDataFrame> {
        let mut reader = TofPacketReader::new(&filename);
        let mut series = Self::new();
        // it would be nice to set the filter here, but I 
        // don't know how that can be done in the macro
        reader.filter  = <$class>::TOF_PACKET_TYPE;
        for tp in reader {
          if let Ok(moni) =  tp.unpack::<$class>() {
            series.add(moni);
          }
          // other packets will get thrown away 
        }
        match series.get_dataframe() {
          Ok(df) => {
            let pydf = PyDataFrame(df);
            return Ok(pydf);
          },
          Err(err) => {
            return Err(PyValueError::new_err(err.to_string()));
          }
        }
      }
      
      #[pyo3(name="get_var_for_board")]
      fn get_var_for_board_py(&self, varname : &str, rb_id : u8) -> Option<Vec<f32>> {
        self.get_var_for_board(varname, &rb_id)
      }

      /// Reduces the MoniSeries to a single polars data frame
      /// The structure itself will not be changed
      #[pyo3(name="get_dataframe")]
      fn get_dataframe_py(&self) -> PyResult<PyDataFrame> {
        match self.get_dataframe() {
          Ok(df) => {
            let pydf = PyDataFrame(df);
            return Ok(pydf);
          },
          Err(err) => {
            return Err(PyValueError::new_err(err.to_string()));
          }
        }
      }

      //fn get_pl_series_py(&self) -> PyResult<PyS
      //fn get_data(&self) -> &HashMap<u8,VecDeque<$class>> {
      //  return &self.data;
      //}
    
      //fn get_data_mut(&mut self) -> &mut HashMap<u8,VecDeque<$class>> {
      //  return &mut self.data;
      //}
     
      //fn get_max_size(&self) -> usize {
      //  return self.max_size;
      //}
    }
    
    #[cfg(feature="pybindings")]
    pythonize_display!($name);
  }
}

