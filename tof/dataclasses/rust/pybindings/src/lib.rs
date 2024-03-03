//use std::collections::VecDeque;

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

extern crate pyo3_log;
use numpy::PyArray1;

use tof_dataclasses::analysis::{
    find_peaks,
    find_peaks_zscore,
    interpolate_time,
    cfd_simple,
    integrate,
    time2bin
};

//use tof_dataclasses::constants::N_CHN_PER_LTB;

use tof_dataclasses::calibrations::{
    find_zero_crossings,
    get_periods,
    Edge,
};

use tof_dataclasses::events::MasterTriggerEvent;

use tof_dataclasses::ipbus as ipbus;

///helper
fn convert_pyarray1(arr : &PyArray1<f32>) -> Vec<f32> {
  let mut vec = Vec::<f32>::new();
  unsafe {
    vec.extend_from_slice(arr.as_slice().unwrap());
  }
  return vec;
}

#[pyfunction]
#[pyo3(name="get_periods")]
/// Get the periods of a (sine) wave
pub fn wrap_get_periods(trace   : &PyArray1<f32>,
                        dts     : &PyArray1<f32>,
                        nperiod : f32,
                        nskip   : f32)
    -> PyResult<(Vec<usize>, Vec<f32>)> {
  // we fix the edge here
  let edge = Edge::Rising;
  let wr_trace = convert_pyarray1(trace);
  let wr_dts   = convert_pyarray1(dts);
  let result   = get_periods(&wr_trace, &wr_dts, nperiod, nskip, &edge);
  Ok(result)
}

#[pyfunction]
#[pyo3(name="find_zero_crossings")]
/// Get a vector with the indizes where 
/// the input array crosses zero
pub fn wrap_find_zero_crossings(trace : &PyArray1<f32>) 
  -> PyResult<Vec<usize>> {
  let tr  = convert_pyarray1(trace);
  let zcs = find_zero_crossings(&tr);
  Ok(zcs)
}

#[pyfunction]
#[pyo3(name="cfd_simple")]
/// Find the peak onset time based on a cfd
/// "Constant fraction discrimination" algorithm
///
/// # Arguments
///
/// * start_peak : bin
/// * end_peak   : bin
/// * cfd_frac   : 0.2 is the typical default
pub fn wrap_cfd_simple(voltages    : &PyArray1<f32>,
                       nanoseconds : &PyArray1<f32>,
                       cfd_frac    : f32,
                       start_peak  : usize,
                       end_peak    : usize) -> PyResult<f32> {
  let voltages_vec    = convert_pyarray1(voltages);
  let nanoseconds_vec = convert_pyarray1(nanoseconds);
  match cfd_simple(&voltages_vec   ,
                   &nanoseconds_vec,
                   cfd_frac       ,
                   start_peak  ,
                   end_peak) {
    Ok(result) => Ok(result),
    Err(err)   => {
     return Err(PyValueError::new_err(err.to_string()));
    } 
  }
}

#[pyfunction]
#[pyo3(name="interpolate_time")]
pub fn wrap_interpolate_time(voltages    : &PyArray1<f32>,
                             nanoseconds : &PyArray1<f32>,
                             threshold   : f32,
                             idx         : usize,
                             size        : usize) -> PyResult<f32> {
  let mut voltages_vec    = Vec::<f32>::new();
  let mut nanoseconds_vec = Vec::<f32>::new(); 
  unsafe {
    voltages_vec.extend_from_slice(voltages.as_slice().unwrap());
    nanoseconds_vec.extend_from_slice(nanoseconds.as_slice().unwrap());
  }
  match interpolate_time (&voltages_vec   ,
                          &nanoseconds_vec, 
                          threshold      ,
                          idx            ,
                          size) {
   Ok(result) => Ok(result),
   Err(err)   => {
    return Err(PyValueError::new_err(err.to_string()));
   } 
  }
}

#[pyfunction]
#[pyo3(name="time2bin")]
fn wrap_time2bin(nanoseconds : &PyArray1<f32>,
                 t_ns        : f32) -> PyResult<usize> {
 let mut nanoseconds_vec = Vec::<f32>::new(); 
 unsafe {
   nanoseconds_vec.extend_from_slice(nanoseconds.as_slice().unwrap());
 }
 match time2bin (&nanoseconds_vec,
                 t_ns){
   Ok(result) => Ok(result),
   Err(err)   => {
    return Err(PyValueError::new_err(err.to_string()));
   } 
 }
}

#[pyfunction]
#[pyo3(name="integrate")]
fn wrap_integrate(voltages    : &PyArray1<f32>,
                  nanoseconds : &PyArray1<f32>,
                  lower_bound  : f32,
                  size         : f32,
                  impedance    : f32) -> PyResult<f32>  {
 let mut voltages_vec    = Vec::<f32>::new();
 let mut nanoseconds_vec = Vec::<f32>::new(); 
 unsafe {
   voltages_vec.extend_from_slice(voltages.as_slice().unwrap());
   nanoseconds_vec.extend_from_slice(nanoseconds.as_slice().unwrap());
 }
 match integrate(&voltages_vec, &nanoseconds_vec, lower_bound, size, impedance) {
   Ok(result) => Ok(result),
   Err(err)   => {
    return Err(PyValueError::new_err(err.to_string()));
   }
 }
}

#[pyfunction]
#[pyo3(name = "find_peaks")]
/// The GAPS peak finding algorithm, based on 
/// legacy code written by the UCLA TOF team.
///
/// This needs to be applied AFTER the peakfinding
/// and takes a specific peak as input argument
///
/// # Arguments
/// 
/// * voltages     (np.ndarray) | These both together
/// * nanosecondes (np.ndarray) | are a calibrated waveform
/// * start_time   (float)      - begin peak search at this time
/// * window_size  (float)      - limit peak search to start_time 
///                               + start_time + window_size (in ns)
/// * min_peak_width (usize)    - If a peak has a lower width, it 
///                               will get discarded (in bins)
/// * threshold      (f32)      - Ingore peaks which fall below this
///                               voltage (in mV)
/// * max_peaks      (usize)    - Stop peak search after max_peaks are
///                              found
fn wrap_find_peaks(voltages       : &PyArray1<f32>,
                   nanoseconds    : &PyArray1<f32>,
                   start_time     : f32,
                   window_size    : f32,
                   min_peak_width : usize,
                   threshold      : f32,
                   max_peaks      : usize) -> PyResult<Vec<(usize,usize)>> {
 let mut voltages_vec    = Vec::<f32>::new();
 let mut nanoseconds_vec = Vec::<f32>::new(); 
 unsafe {
   voltages_vec.extend_from_slice(voltages.as_slice().unwrap());
   nanoseconds_vec.extend_from_slice(nanoseconds.as_slice().unwrap());
 }

 match find_peaks(&voltages_vec  , 
                  &nanoseconds_vec   , 
                  start_time    , 
                  window_size   , 
                  min_peak_width, 
                  threshold     , 
                  max_peaks     ) {
   Ok(result) => Ok(result),
   Err(err)   => {
    return Err(PyValueError::new_err(err.to_string()));
   }
 }
}

#[pyfunction]
#[pyo3(name = "find_peaks_zscore")]
fn wrap_find_peaks_zscore(voltages       : &PyArray1<f32>,
                          nanoseconds    : &PyArray1<f32>,
                          start_time     : f32,
                          window_size    : f32,
                          lag            : usize,
                          threshold      : f64,
                          influence      : f64) -> PyResult<Vec<(usize,usize)>> {
 let mut voltages_vec    = Vec::<f32>::new();
 let mut nanoseconds_vec = Vec::<f32>::new(); 
 unsafe {
   voltages_vec.extend_from_slice(voltages.as_slice().unwrap());
   nanoseconds_vec.extend_from_slice(nanoseconds.as_slice().unwrap());
 }

 match find_peaks_zscore(&nanoseconds_vec, 
                         &voltages_vec   ,   
                         start_time      , 
                         window_size     , 
                         lag             , 
                         threshold       , 
                         influence) {
   Ok(result) => Ok(result),
   Err(err)   => {
     return Err(PyValueError::new_err(err.to_string()));
   }
 }
}

#[pyclass]
pub struct IPBus {
  ipbus : ipbus::IPBus,
}

#[pymethods]
impl IPBus {
  #[new]
  fn new(target_address : String) -> Self {
    let ipbus = ipbus::IPBus::new(target_address).expect("Unable to connect to {target_address}");
    Self {
      ipbus : ipbus,
    }
  }

  /// Make a IPBus status query
  pub fn get_status(&mut self) -> PyResult<()> {
    match self.ipbus.get_status() {
      Ok(_) => {
        return Ok(());
      },
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
 
  pub fn get_buffer(&self) -> [u8;ipbus::MT_MAX_PACKSIZE] {
    return self.ipbus.buffer.clone();
  }

  pub fn set_packet_id(&mut self, pid : u16) {
    self.ipbus.pid = pid;
  }
 
  pub fn get_packet_id(&self) -> u16 {
    self.ipbus.pid
  }

  pub fn get_expected_packet_id(&self) -> u16 {
    self.ipbus.expected_pid
  }

  /// Set the packet id to that what is expected from the targetr
  pub fn realign_packet_id(&mut self) -> PyResult<()> {
    match self.ipbus.realign_packet_id() {
      Ok(_) => {
        return Ok(());
      },
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
  
  /// Get the next packet id, which is expected by the target
  pub fn get_target_next_expected_packet_id(&mut self) 
    -> PyResult<u16> {
    match self.ipbus.get_target_next_expected_packet_id() {
      Ok(result) => {
        return Ok(result);
      },
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  pub fn read_multiple(&mut self,
                       addr           : u32,
                       nwords         : usize,
                       increment_addr : bool) 
    -> PyResult<Vec<u32>> {
  
    match self.ipbus.read_multiple(addr,
                                   nwords,
                                   increment_addr) {
      Ok(result) => {
        return Ok(result);
      },
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  pub fn write(&mut self,
               addr   : u32,
               data   : u32) 
    -> PyResult<()> {
    
    match self.ipbus.write(addr, data) {
      Ok(_) => Ok(()),
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
 

  pub fn read(&mut self, addr   : u32) 
    -> PyResult<u32> {
    match self.ipbus.read(addr) {
      Ok(result) => {
        return Ok(result);
      },
      Err(err)   => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}


#[pyclass]
pub struct MasterTrigger {
  ipbus : ipbus::IPBus,
}

#[pymethods]
impl MasterTrigger {
  #[new]
  fn new(target_address : String) -> Self {
    let ipbus = ipbus::IPBus::new(target_address).expect("Unable to connect to {target_address}");
    Self {
      ipbus : ipbus,
    }
  }

  fn reset_daq(&mut self) -> PyResult<()>{
    match self.ipbus.write(0x10,1) {
      Ok(result) => {
        return Ok(result); 
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }


  fn get_expected_pid(&mut self) -> PyResult<u16> {
    match self.ipbus.get_target_next_expected_packet_id(){
      Ok(result) => {
        return Ok(result); 
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn realign_packet_id(&mut self) -> PyResult<()> {
    match self.ipbus.realign_packet_id() {
      Ok(_) => {
        return Ok(()); 
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }

  fn set_packet_id(&mut self, pid : u16) {
    self.ipbus.pid = pid;
  }

  fn get_packet_id(&mut self) -> u16 {
    self.ipbus.pid
  }

  fn get_mtevent(&mut self) {
    let mut n_daq_words : u16;
    loop {
      match self.ipbus.read(0x13) { 
        Err(_err) => {
          // A timeout does not ncecessarily mean that there 
          // is no event, it can also just mean that 
          // the rate is low.
          //trace!("Timeout in read_register for MTB! {err}");
          continue;
        },
        Ok(_n_words) => {
          n_daq_words = (_n_words >> 16) as u16;
          if _n_words == 0 {
            continue;
          }
          //trace!("Got n_daq_words {n_daq_words}");
          n_daq_words /= 2; //mtb internally operates in 16bit words, but 
          //                  //registers return 32bit words.
          break;
        }
      }
    }
    let data : Vec<u32>;
    println!("[MasterTrigger::get_mtevent] => Will query DAQ for {n_daq_words} words!");
    match self.ipbus.read_multiple(
                                   0x11,
                                   n_daq_words as usize,
                                   false) {
      Err(err) => {
        println!("[MasterTrigger::get_mtevent] => failed! {err}");
        return;
      }
      Ok(_data) => {
        data = _data;
        for word in data.iter() {
          println!("[MasterTrigger::get_mtevent] => DAQ word {:x}", word);
        }
      }
    }
    if data[0] != 0xAAAAAAAA {
      println!("[MasterTrigger::get_mtevent] => Got MTB data, but the header is incorrect {}", data[0]);
      //return Err(MasterTriggerError::PackageHeaderIncorrect);
      return;
    }
    let foot_pos = (n_daq_words - 1) as usize;
    if data.len() <= foot_pos {
      println!("[MasterTrigger::get_mtevent] => Got MTB data, but the format is not correct");
      //return Err(MasterTriggerError::PackageHeaderIncorrect);
      return;
    }
    if data[foot_pos] != 0x55555555 {
      println!("[MasterTrigger::get_mtevent] => Got MTB data, but the footer is incorrect {}", data[foot_pos]);
      //return Err(MasterTriggerError::PackageFooterIncorrect);
      return;
    }

    // Number of words which will be always there. 
    // Min event size is +1 word for hits
    //const MTB_DAQ_PACKET_FIXED_N_WORDS : u32 = 9; 
    //let n_hit_packets = n_daq_words as u32 - MTB_DAQ_PACKET_FIXED_N_WORDS;
    //println!("We are expecting {}", n_hit_packets);
    let mut mte = MasterTriggerEvent::new(0,0);
    mte.event_id      = data[1];
    mte.timestamp     = data[2];
    mte.tiu_timestamp = data[3];
    mte.tiu_gps_32    = data[4];
    mte.tiu_gps_16    = data[5] & 0x0000ffff;
    //mte.board_mask    = decode_board_mask(data[6]);
    //let mut hitmasks = VecDeque::<[bool;N_CHN_PER_LTB]>::new();
    //for k in 0..n_hit_packets {
    //  //println!("hit packet {:?}", data[7usize + k as usize]);
    //  (hits_a, hits_b) = decode_hit_mask(data[7usize + k as usize]);
    //  hitmasks.push_back(hits_a);
    //  hitmasks.push_back(hits_b);
    //}
    //for k in 0..mte.board_mask.len() {
    //  if mte.board_mask[k] {
    //    match hitmasks.pop_front() { 
    //      None => {
    //        //error!("MTE hit assignment wrong. We expect hits for a certain LTB, but we don't see any!");
    //      },
    //      Some(_hits) => {
    //        mte.hits[k] = _hits;
    //      }
    //    }
    //  }
    //}
    mte.n_paddles = mte.get_hit_paddles(); 
    println!("[MasterTrigger::get_mtevent] => Got MTE {}", mte);
  }

  fn get_rate(&mut self) -> PyResult<u32> {
    match self.ipbus.read(0x17) {
      Ok(rate) => {
        return Ok(rate & 0x00ffffff); 
      }
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}

/// Python API to rust version of tof-dataclasses.
///
/// Currently, this contains only the analysis 
/// functions
#[pymodule]
#[pyo3(name = "rust_dataclasses")]
fn rust_dataclasses(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    pyo3_log::init();
    m.add_function(wrap_pyfunction!(wrap_time2bin,m)?)?;
    m.add_function(wrap_pyfunction!(wrap_find_peaks,m)?)?;
    m.add_function(wrap_pyfunction!(wrap_find_peaks_zscore,m)?)?;
    m.add_function(wrap_pyfunction!(wrap_integrate,m)?)?;
    m.add_function(wrap_pyfunction!(wrap_interpolate_time,m)?)?;
    m.add_function(wrap_pyfunction!(wrap_cfd_simple,m)?)?;
    m.add_function(wrap_pyfunction!(wrap_find_zero_crossings,m)?)?;
    m.add_function(wrap_pyfunction!(wrap_get_periods,m)?)?;
    m.add_class::<IPBus>()?;
    m.add_class::<MasterTrigger>()?;
    Ok(())
}
