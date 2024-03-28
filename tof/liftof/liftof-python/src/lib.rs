//use std::collections::VecDeque;
use std::collections::HashMap;
use std::path::Path;

pub mod dataclasses;

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

extern crate pyo3_log;
use numpy::PyArray1;

use crate::dataclasses::{
    PyRBCalibration,
    PyMasterTriggerEvent,
    PyRBEvent,
};

use tof_dataclasses::analysis::{
    find_peaks,
    find_peaks_zscore,
    interpolate_time,
    cfd_simple,
    integrate,
    time2bin,
    calc_edep_simple
};

//use tof_dataclasses::constants::N_CHN_PER_LTB;

use tof_dataclasses::calibrations::{
    find_zero_crossings,
    get_periods,
    Edge,
};

use tof_dataclasses::events::{
    MasterTriggerEvent,
    RBEvent, 
    TofEvent
};

use tof_dataclasses::manifest::get_rbs_from_sqlite;

use tof_dataclasses::packets::PacketType;

use tof_dataclasses::io::TofPacketReader;

use tof_dataclasses::serialization::Serialization;

use tof_dataclasses::ipbus as ipbus;
use tof_dataclasses::manifest::ReadoutBoard;

use liftof_lib::waveform_analysis;
use liftof_lib::settings::AnalysisEngineSettings;

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
#[pyo3(name="calc_edep_simple")]
pub fn wrap_calc_edep_simple(peak_voltage : f32) -> f32 {
  calc_edep_simple(peak_voltage)
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

//#[pyfunction]
//#[pyo3(name="integrate")]
//fn wrap_integrate(voltages    : &PyArray1<f32>,
//                  nanoseconds : &PyArray1<f32>,
//                  lower_bound  : f32,
//                  size         : f32,
//                  impedance    : f32) -> PyResult<f32>  {
// let mut voltages_vec    = Vec::<f32>::new();
// let mut nanoseconds_vec = Vec::<f32>::new(); 
// unsafe {
//   voltages_vec.extend_from_slice(voltages.as_slice().unwrap());
//   nanoseconds_vec.extend_from_slice(nanoseconds.as_slice().unwrap());
// }
// match integrate(&voltages_vec, &nanoseconds_vec, lower_bound, size, impedance) {
//   Ok(result) => Ok(result),
//   Err(err)   => {
//    return Err(PyValueError::new_err(err.to_string()));
//   }
// }
//}

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

#[pyfunction]
#[pyo3(name = "test_waveform_analysis")]
fn test_waveform_analysis(filename : String) -> PyRBEvent {
  let mut settings   = AnalysisEngineSettings::new();
  settings.find_pks_t_start  = 60.0;
  settings.find_pks_t_window = 300.0;
  settings.min_peak_size     = 10;
  let rb             = ReadoutBoard::new();
  let pth            = Path::new("/srv/gaps/gaps-online-software/gaps-db/gaps_db/gaps_flight.db");
  let rbs            = get_rbs_from_sqlite(pth);
  let mut rb_map     = HashMap::<u8, ReadoutBoard>::new();
  for mut rb in rbs {
    rb.calib_file_path = String::from("/data0/gaps/nevis/calib/latest/"); 
    rb.load_latest_calibration();
    rb_map.insert(rb.rb_id, rb);
  }
  let mut reader = TofPacketReader::new(filename);
  let mut py_rbev = PyRBEvent::new();
  loop {
    match reader.next()  {
      Some(tp) => {
        match tp.packet_type {
          PacketType::TofEvent => {
            match TofEvent::from_tofpacket(&tp) {
              Err(err) => (),
              Ok(te) => {
                //println!("{}", te);
                if te.rb_events.is_empty() {
                  continue;
                }
                for mut rbev in te.rb_events {
                  let rb_id = rbev.header.rb_id;
                  println!("{}", rbev); 
                  py_rbev.set_event(rbev.clone());
                  waveform_analysis(
                      &mut rbev,
                      &rb_map[&rb_id],
                      settings.clone()
                  );
                  for h in rbev.hits {
                    println!("{}", h);
                  }
                  return py_rbev;
                  //break;
                }
              }
            }
          },
          _ => ()      
        }
      },
      None => {
        break;
      }
    }
  }
  return py_rbev;
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

  fn get_event(&mut self, read_until_footer : bool, verbose : bool)
    -> PyResult<PyMasterTriggerEvent> {
    let mut n_daq_words : u16;
    let mut n_daq_words_actual : u16;
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
          let rest = n_daq_words % 2;
          n_daq_words /= 2 + rest; //mtb internally operates in 16bit words, but 
          //                  //registers return 32bit words.
          
          break;
        }
      }
    }
    let mut data : Vec<u32>;
    if verbose {
      println!("[MasterTrigger::get_event] => Will query DAQ for {n_daq_words} words!");
    }
    n_daq_words_actual = n_daq_words;
    match self.ipbus.read_multiple(
                                   0x11,
                                   n_daq_words as usize,
                                   false) {
      Err(err) => {
        if verbose {
          println!("[MasterTrigger::get_event] => failed! {err}");
        }
        return Err(PyValueError::new_err(err.to_string()));
      }
      Ok(_data) => {
        data = _data;
        for (i,word) in data.iter().enumerate() {
          let desc : &str;
          let mut desc_str = String::from("");
          let mut nhit_words = 0;
          match i {
            0 => desc = "HEADER",
            1 => desc = "EVENTID",
            2 => desc = "TIMESTAMP",
            3 => desc = "TIU_TIMESTAMP",
            4 => desc = "TIU_GPS32",
            5 => desc = "TIU_GPS16 + TRIG_SOURCE",
            6 => desc = "RB MASK 0",
            7 => desc = "RB MASK 1",
            8 => {
              nhit_words = nhit_words / 2 + nhit_words % 2;
              desc_str  = format!("BOARD MASK ({} ltbs)", word.count_ones());
              desc  = &desc_str;
            },
            _ => desc = "?"
          }
          if verbose {
            println!("[MasterTrigger::get_event] => DAQ word {} \t({:x}) \t[{}]", word, word, desc);
          }
        }
      }
    }
    if data[0] != 0xAAAAAAAA {
      if verbose {
        println!("[MasterTrigger::get_event] => Got MTB data, but the header is incorrect {}", data[0]);
      }
      return Err(PyValueError::new_err(String::from("Incorrect header value!")));
    }
    let mut foot_pos = (n_daq_words - 1) as usize;
    if data.len() <= foot_pos {
      if verbose {
        println!("[MasterTrigger::get_event] => Got MTB data, but the format is not correct");
      }
      return Err(PyValueError::new_err(String::from("Empty data!")));
    }
    if data[foot_pos] != 0x55555555 {
      if verbose {
        println!("[MasterTrigger::get_event] => Did not read unti footer!");
      }
      if read_until_footer {
        if verbose {
          println!("[MasterTrigger::get_event] => .. will read additional words!");
        }
        loop {
          match self.ipbus.read(0x11) {
            Err(err) => {
              if verbose {
                println!("[MasterTrigger::get_event] => Issues reading from 0x11");
              }
              return Err(PyValueError::new_err(err.to_string()));
            },
            Ok(next_word) => {
              n_daq_words_actual += 1;
              data.push(next_word);
              if next_word == 0x55555555 {
                break;
              }
            }
          }
        }
        foot_pos = n_daq_words_actual as usize - 1;
        if verbose {
          println!("[MasterTrigger::get_event] => We read {} additional words!", n_daq_words_actual - n_daq_words);
        }
      } else {
        if verbose {
          println!("[MasterTrigger::get_event] => Got MTB data, but the footer is incorrect {}", data[foot_pos]);
        }
        return Err(PyValueError::new_err(String::from("Footer incorrect!")));
      }
    }

    // Number of words which will be always there. 
    // Min event size is +1 word for hits
    //const MTB_DAQ_PACKET_FIXED_N_WORDS : u32 = 9; 
    //let n_hit_packets = n_daq_words as u32 - MTB_DAQ_PACKET_FIXED_N_WORDS;
    //println!("We are expecting {}", n_hit_packets);
    let mut mte = MasterTriggerEvent::new();
    mte.event_id       = data[1];
    mte.timestamp      = data[2];
    mte.tiu_timestamp  = data[3];
    mte.tiu_gps32      = data[4];
    mte.tiu_gps16      = (data[5] & 0x0000ffff) as u16;
    mte.trigger_source = ((data[5] & 0xffff0000) >> 16) as u16;
    //mte.get_trigger_sources();
    let rbmask = (data[7] as u64) << 31 | data[6] as u64; 
    mte.mtb_link_mask  = rbmask;
    mte.dsi_j_mask     = data[8];
    let mut n_hit_words    = n_daq_words_actual - 9 - 2; // fixed part is 11 words
    if n_hit_words > n_daq_words_actual {
      n_hit_words = 0;
      println!("[MasterTrigger::get_event] N hit word calculation failed! fixing... {}", n_hit_words);
    }
    if verbose {
      println!("[MasterTrigger::get_event] => Will read {} hit word", n_hit_words);
    }
    for k in 1..n_hit_words+1 {
      if verbose {
        println!("[MasterTrigger::get_event] => Getting word {}", k);
      }
      let first  = (data[8 + k as usize] & 0x0000ffff) as u16;
      let second = ((data[8 + k as usize] & 0xffff0000) >> 16) as u16; 
      mte.channel_mask.push(first);
      if second != 0 {
        mte.channel_mask.push(second);
      }
    }
    if verbose {
      println!("[MasterTrigger::get_event] => Got MTE \n{}", mte);
    }
    let mut event = PyMasterTriggerEvent::new();
    event.set_event(mte);
    Ok(event)
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
    //m.add_function(wrap_pyfunction!(wrap_integrate,m)?)?;
    m.add_function(wrap_pyfunction!(wrap_interpolate_time,m)?)?;
    m.add_function(wrap_pyfunction!(wrap_cfd_simple,m)?)?;
    m.add_function(wrap_pyfunction!(wrap_find_zero_crossings,m)?)?;
    m.add_function(wrap_pyfunction!(wrap_get_periods,m)?)?;
    m.add_function(wrap_pyfunction!(test_waveform_analysis,m)?)?;
    m.add_function(wrap_pyfunction!(wrap_calc_edep_simple,m)?)?;
    m.add_class::<IPBus>()?;
    m.add_class::<MasterTrigger>()?;
    m.add_class::<PyMasterTriggerEvent>()?;
    m.add_class::<PyRBCalibration>()?;
    Ok(())
}
