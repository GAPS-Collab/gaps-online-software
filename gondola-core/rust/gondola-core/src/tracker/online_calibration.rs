// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

use std::io::BufRead;
#[cfg(feature="pybindings")]
use std::path::PathBuf;

/// The tracker online (in-flight) calibration. This converts 
/// adc to energy,  according to a transfer function. 
/// There is only a single transfer function for all the strips
#[cfg_attr(feature="pybindings", pyclass)] 
pub struct TrackerOnlineCalibration {
  pub tf_map     : HashMap<u32,TrackerStripTransferFunction>, 
  pub ped_map    : HashMap<u32,f32>,
  pub pulser_map : HashMap<u32,bool>
}

impl TrackerOnlineCalibration {

  pub fn new() -> Self {
    Self {
      tf_map     : HashMap::<u32,TrackerStripTransferFunction>::new(), 
      ped_map    : HashMap::<u32,f32>::new(),
      pulser_map : HashMap::<u32,bool>::new()
    }
  }

  /// Load the online tracker calibration from the file 
  /// as it was used in the GAPSI flight.
  pub fn from_default() -> Self {
    let cali_path  = env::var("GONDOLA_TRK_ONLINE_CAL").unwrap_or_else(|_| "".to_string());
    Self::from_file(&cali_path)
  }

  //pub fn from_file(fname : &str) -> Self {
  pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
    let mut cali = Self::new();
    let file_res = File::open(path);
    match file_res {
      Err(_err) => {
        error!("Unable to open file!");
        return cali; 
      }
      Ok(file) => {
        let reader = BufReader::new(file);
        //let mut data = Vec::new();

        // Use .skip(5) to bypass the header lines
        for line in reader.lines().skip(5) {
          let line = line.unwrap();
          let trimmed = line.trim();
          if trimmed.is_empty() {
            continue;
          }
          let row: Vec<f32> = trimmed
            .split(',')
            .map(|s| s.trim().parse::<f32>().unwrap_or(0.0))
            .collect();
          let mut strip = TrackerStrip::new();
          strip.layer   = row[0] as i32;
          strip.row     = row[1] as i32;
          strip.module  = row[2] as i32;
          strip.channel = row[3] as i32;
          let mut tf    = TrackerStripTransferFunction::new();
          let strip_id  = strip.get_stripid();
          tf.strip_id   = strip_id as i32;
          tf.pol_a2_0   = row[4]; 
          tf.pol_a2_1   = row[5];    
          tf.pol_a2_2   = row[6]; 
          tf.pol_b3_0   = row[7]; 
          tf.pol_b3_1   = row[8]; 
          tf.pol_b3_2   = row[9]; 
          tf.pol_b3_3   = row[10]; 
          tf.pol_c3_0   = row[11]; 
          tf.pol_c3_1   = row[12]; 
          tf.pol_c3_2   = row[13]; 
          tf.pol_c3_3   = row[14]; 
          tf.pol_d3_0   = row[15];     
          tf.pol_d3_1   = row[16]; 
          tf.pol_d3_2   = row[17]; 
          tf.pol_d3_3   = row[18]; 
          let pedestal  = row[19];
          let is_puls   = row[20] > 0.0;
          cali.tf_map.insert(strip_id, tf);
          cali.pulser_map.insert(strip_id, is_puls);
          cali.ped_map.insert(strip_id, pedestal as f32);
          //data.push(row);
        }
      }
    }
    cali
  }

  /// Fill out the energy field in the tracker hit
  /// 
  /// FIXME - this panics if the calibration is not 
  /// loaded properly
  pub fn calibrate(&self, hit : &mut TrackerHit) {
    let strip_id = hit.get_stripid();
    let scale   : f32 = 0.841/1000.0;
    let mut adc : f32 = hit.adc as f32 - self.ped_map[&strip_id];
    if adc > 1500.0 {
      adc = 1500.0;
    } 
    hit.energy = scale*self.tf_map[&strip_id].evaluate(adc);
  }

  pub fn is_pulsed(&self, hit : &TrackerHit) -> bool {
    self.pulser_map[&hit.get_stripid()] 
  }
} 

#[cfg(feature="pybindings")]
#[pymethods]
impl TrackerOnlineCalibration {

  #[new]
  fn new_py() -> Self {
    Self::new()
  }
 
  /// Fill out the energy field in the tracker hit
  /// 
  /// FIXME - this panics if the calibration is not 
  /// loaded properly
  ///
  /// # Arguments: 
  ///   * hit : A tracker hit. This hit will have its 
  ///           .energy field populated
  #[pyo3(name="calibrate")]
  fn calibrate_py(&self, hit : &mut TrackerHit) {
    self.calibrate(hit);
  }

  #[pyo3(name="is_pulsed")]
  fn is_pulsed_py(&self, hit : &TrackerHit) -> bool {
    self.is_pulsed(hit)
  }
  
  /// Load the online tracker calibration from the file 
  /// as it was used in the GAPSI flight.
  ///
  /// FIXME - if this fails, we won't know
  #[staticmethod]
  #[pyo3(name="from_default")]
  pub fn from_default_py() -> PyResult<Self> {
    let cali_path  = env::var("GONDOLA_TRK_ONLINE_CAL").unwrap_or_else(|_| "".to_string());
    Ok(Self::from_file(&cali_path))
  }

  #[staticmethod]
  #[pyo3(name="from_file")]
  fn from_file_py(path: String) -> PyResult<Self> {
    // Convert the Python String to a PathBuf
    let path_buf = PathBuf::from(path);
    Ok(Self::from_file(path_buf))
  }
}

