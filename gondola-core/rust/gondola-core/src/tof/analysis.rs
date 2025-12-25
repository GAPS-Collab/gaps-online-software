//! A high level analysis which interplays with 
//! the tof cuts and can histogram TOF relevant 
//! quantities 
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;
use crate::tof::cuts::TofCuts;

#[cfg_attr(feature="pybindings", pyclass)]
#[derive(Debug, Clone)]
pub struct TofAnalysisCache {
  #[pyo3(get)]  
  pub dist       : Vec<f32>,
  #[pyo3(get)]  
  pub x_outer    : Vec<f32>,
  #[pyo3(get)]  
  pub y_outer    : Vec<f32>,
  #[pyo3(get)]  
  pub z_outer    : Vec<f32>,
  #[pyo3(get)]  
  pub x_inner    : Vec<f32>,
  #[pyo3(get)]  
  pub y_inner    : Vec<f32>,
  #[pyo3(get)]  
  pub z_inner    : Vec<f32>,
  #[pyo3(get)]  
  pub pid_inner  : Vec<u8> ,
  #[pyo3(get)]  
  pub pid_outer  : Vec<u8> ,
  #[pyo3(get)]  
  pub cos_theta  : Vec<f32>,
  #[pyo3(get)]  
  pub cos2_theta : Vec<f32>,
  #[pyo3(get)]  
  pub beta       : Vec<f32>,
  #[pyo3(get)]  
  pub t_outer    : Vec<f32>,
  #[pyo3(get)]  
  pub t_inner    : Vec<f32>,
  #[pyo3(get)]  
  pub t_diff     : Vec<f32>,
  #[pyo3(get)]  
  pub ph_delay   : Vec<f32>,
  // total energy desposition 
  pub edep       : Vec<f32>,
  // energy deposition per panel
  pub edep_panel : HashMap<u8,Vec<f32>>,
  pub edep_cbe   : Vec<f32>,
  pub edep_cor   : Vec<f32>,
  pub edep_umb   : Vec<f32>,
}

impl TofAnalysisCache {

  pub fn new() -> Self {
    let mut panel_edeps = HashMap::<u8,Vec<f32>>::new();
    for panel in 1..22u8 {
      panel_edeps.insert(panel, Vec::<f32>::new());
    }
    Self {
      dist       : Vec::<f32>::new(),
      x_outer    : Vec::<f32>::new(),
      y_outer    : Vec::<f32>::new(),
      z_outer    : Vec::<f32>::new(),
      x_inner    : Vec::<f32>::new(),
      y_inner    : Vec::<f32>::new(),
      z_inner    : Vec::<f32>::new(),
      pid_inner  : Vec::<u8>::new(),
      pid_outer  : Vec::<u8>::new(),
      cos_theta  : Vec::<f32>::new(),
      cos2_theta : Vec::<f32>::new(),
      beta       : Vec::<f32>::new(),
      t_outer    : Vec::<f32>::new(),
      t_inner    : Vec::<f32>::new(),
      t_diff     : Vec::<f32>::new(),
      ph_delay   : Vec::<f32>::new(),
      edep       : Vec::<f32>::new(),
      edep_cbe   : Vec::<f32>::new(),
      edep_cor   : Vec::<f32>::new(),
      edep_umb   : Vec::<f32>::new(),
      edep_panel : panel_edeps,
    }
  }

  fn clear(&mut self) {
    self.dist      .clear();
    self.x_outer   .clear();
    self.y_outer   .clear();
    self.z_outer   .clear();
    self.x_inner   .clear();
    self.y_inner   .clear();
    self.z_inner   .clear();
    self.pid_inner .clear();
    self.pid_outer .clear();
    self.cos_theta .clear();
    self.cos2_theta.clear();
    self.beta      .clear();
    self.t_outer   .clear();
    self.t_inner   .clear();
    self.t_diff    .clear();
    self.ph_delay  .clear();
    self.edep      .clear();
    self.edep_cbe  .clear();
    self.edep_umb  .clear();
    self.edep_cor  .clear();
    for k in 1..22u8 {
      self.edep_panel.get_mut(&k).unwrap().clear();
    }
  }

  fn keys() -> Vec<&'static str> {
    vec![
      "dist"     ,
      "x_outer" ,
      "y_outer"  ,
      "z_outer"  ,
      "x_inner"  ,
      "y_inner"  ,
      "z_inner"  ,
      "pid_inner"  ,
      "pid_outer"  ,
      "cos_theta"  ,
      "cos2_theta" ,
      "beta",
      "t_outer",
      "t_inner",
      "t_diff", 
      "phase_delay",
      "edep",
      "edep_panel",
      "edep_umb",
      "edep_cor",
      "edep_cbe"]
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TofAnalysisCache {

  fn add_other(&mut self, other : &TofAnalysisCache) {
    self.dist      .extend_from_slice(&other.dist      ); 
    self.x_outer   .extend_from_slice(&other.x_outer   ); 
    self.y_outer   .extend_from_slice(&other.y_outer   ); 
    self.z_outer   .extend_from_slice(&other.z_outer   ); 
    self.x_inner   .extend_from_slice(&other.x_inner   ); 
    self.y_inner   .extend_from_slice(&other.y_inner   ); 
    self.z_inner   .extend_from_slice(&other.z_inner   ); 
    self.pid_inner .extend_from_slice(&other.pid_inner ); 
    self.pid_outer .extend_from_slice(&other.pid_outer ); 
    self.cos_theta .extend_from_slice(&other.cos_theta ); 
    self.cos2_theta.extend_from_slice(&other.cos2_theta); 
    self.beta      .extend_from_slice(&other.beta      ); 
    self.t_outer   .extend_from_slice(&other.t_outer   ); 
    self.t_inner   .extend_from_slice(&other.t_inner   ); 
    self.t_diff    .extend_from_slice(&other.t_diff    ); 
    self.ph_delay  .extend_from_slice(&other.ph_delay  ); 
    self.edep      .extend_from_slice(&other.edep      );
    self.edep_umb  .extend_from_slice(&other.edep_umb  );
    self.edep_cor  .extend_from_slice(&other.edep_cbe  );
    self.edep_cbe  .extend_from_slice(&other.edep_cor  );
    for k in 1..22u8 {
      self.edep_panel.get_mut(&k).unwrap().extend_from_slice(&other.edep_panel.get(&k).unwrap());
    }
  }

  fn get_u8_data<'py>(&'py self, py: Python<'py>, name : &str) -> PyResult<Bound<'py, PyArray1<u8>>> {
    match name {
      "pid_inner" => {
        let slice = &self.pid_inner[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Ok(py_array);
      }
      "pid_outer" => {
        let slice = &self.pid_outer[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Ok(py_array);
      }
      _ => {
        let err_msg = format!("No key with name {}! See .keys() for a list of available keys!",name);
        return Err(PyKeyError::new_err(err_msg));
      }
    }
  }
  
  fn get_f32_data_panel<'py>(&'py self, py: Python<'py>, name : &str, panel : u8) -> Option<Bound<'py, PyArray1<f32>>> {
    match name {
      "edep" => {
        let slice = &self.edep_panel.get(&panel).unwrap()[..];
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      _ => {
        return None;
      }
    }
  }
  
  //#[pyo3(name="_get_data")]
  fn get_f32_data<'py>(&'py self, py: Python<'py>, name : &str) -> Option<Bound<'py, PyArray1<f32>>> {
    match name {
      "dist" => {
        let slice = &self.dist[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "x_outer" => {
        let slice = &self.x_outer[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "y_outer" => {
        let slice = &self.y_outer[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "z_outer" => {
        let slice = &self.z_outer[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "x_inner" => {
        let slice = &self.x_inner[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "y_inner" => {
        let slice = &self.y_inner[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "z_inner" => {
        let slice = &self.z_inner[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "cos_theta" => {
        let slice = &self.cos_theta[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "cos2_theta" => {
        let slice = &self.cos2_theta[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "beta" => {
        let slice = &self.beta[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "t_outer" => {
        let slice = &self.t_outer[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "t_inner" => {
        let slice = &self.t_inner[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "t_diff" => {
        let slice = &self.t_diff[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "ph_delay" => {
        let slice = &self.ph_delay[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "edep" => {
        let slice = &self.edep[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "edep_umb" => {
        let slice = &self.edep_umb[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "edep_cor" => {
        let slice = &self.edep_cor[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "edep_cbe" => {
        let slice = &self.edep_cbe[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      _ => {
        return None;
      }
    }
  }

  #[staticmethod]
  #[pyo3(name="keys")]
  fn keys_py() -> Vec<&'static str> {
    Self::keys()
  }
  
  #[pyo3(name="clear")]
  fn clear_py(&mut self) {
    self.clear();
  }
}


//------------------------------------------------------- 


#[cfg_attr(feature="pybindings", pyclass)]
#[derive(Debug, Clone)]
pub struct TofAnalysisPaddleCache {
  #[pyo3(get)]  
  pub charge2d    : HashMap<u8,Vec<(f32,f32)>>,
  #[pyo3(get)]  
  pub amp2d       : HashMap<u8,Vec<(f32,f32)>>,
  #[pyo3(get)]  
  pub amp_a       : HashMap<u8,Vec<f32>>,
  #[pyo3(get)]  
  pub amp_b       : HashMap<u8,Vec<f32>>,
  #[pyo3(get)]  
  pub time_a      : HashMap<u8,Vec<f32>>,
  #[pyo3(get)]  
  pub time_b      : HashMap<u8,Vec<f32>>,
  #[pyo3(get)]  
  pub charge_a    : HashMap<u8,Vec<f32>>,
  #[pyo3(get)]  
  pub charge_b    : HashMap<u8,Vec<f32>>,
  #[pyo3(get)]  
  pub bl_a        : HashMap<u8,Vec<f32>>,
  #[pyo3(get)]  
  pub bl_b        : HashMap<u8,Vec<f32>>,
  #[pyo3(get)]  
  pub bl_a_rms    : HashMap<u8,Vec<f32>>,
  #[pyo3(get)]  
  pub bl_b_rms    : HashMap<u8,Vec<f32>>,
  #[pyo3(get)]  
  pub x0          : HashMap<u8,Vec<f32>>,
  #[pyo3(get)]  
  pub t0          : HashMap<u8,Vec<f32>>,
  #[pyo3(get)]  
  pub edep        : HashMap<u8,Vec<f32>>,
  #[pyo3(get)]  
  pub pos_edep    : HashMap<u8,Vec<(f32, f32)>>,
} 

impl TofAnalysisPaddleCache {
  
  pub fn new() -> Self {
    let mut map2d = HashMap::<u8,Vec<(f32,f32)>>::new();
    let mut map1d = HashMap::<u8,Vec<f32>>::new();
    for pid in 1..161 {
      map2d.insert(pid, Vec::<(f32,f32)>::new());
      map1d.insert(pid, Vec::<f32>::new());
    }

    Self {
      charge2d    : map2d.clone(),
      amp2d       : map2d.clone(),
      amp_a       : map1d.clone(),
      amp_b       : map1d.clone(),
      time_a      : map1d.clone(),
      time_b      : map1d.clone(),
      charge_a    : map1d.clone(),
      charge_b    : map1d.clone(),
      bl_a        : map1d.clone(),
      bl_b        : map1d.clone(),
      bl_a_rms    : map1d.clone(),
      bl_b_rms    : map1d.clone(),
      x0          : map1d.clone(),
      t0          : map1d.clone(),
      edep        : map1d.clone(),
      pos_edep    : map2d.clone(),
    }
  }

  //pub fn clear(&mut self) {
  //  for k in 1..161 {
  //    self.charge2d.get_mut(&k).unwrap().clear();
  //    self.amp2d   .get_mut(&k).unwrap().clear();
  //    self.amp_a   .get_mut(&k).unwrap().clear();
  //    self.amp_b   .get_mut(&k).unwrap().clear();
  //    self.time_a  .get_mut(&k).unwrap().clear();
  //    self.time_b  .get_mut(&k).unwrap().clear();
  //    self.charge_a.get_mut(&k).unwrap().clear();
  //    self.charge_b.get_mut(&k).unwrap().clear();
  //    self.bl_a    .get_mut(&k).unwrap().clear();
  //    self.bl_b    .get_mut(&k).unwrap().clear();
  //    self.bl_a_rms.get_mut(&k).unwrap().clear();
  //    self.bl_b_rms.get_mut(&k).unwrap().clear();
  //    self.x0      .get_mut(&k).unwrap().clear();
  //    self.t0      .get_mut(&k).unwrap().clear();
  //    self.edep    .get_mut(&k).unwrap().clear();
  //    self.pos_edep.get_mut(&k).unwrap().clear();
  //  }
  //}
  
  fn keys() -> Vec<&'static str> {
    vec![
      "charge2d"     ,
      "amp2d" ,
      "amp_a"  ,
      "amp_b"  ,
      "time_a"  ,
      "time_b"  ,
      "charge_a"  ,
      "charge_b"  ,
      "bl_a"  ,
      "bl_b"  ,
      "bl_a_rms" ,
      "bl_b_rms",
      "x0",
      "t0",
      "edep", 
      "pos_edep"]
  }

  pub fn get(&self, varname : &str) -> Option<f32> {
    error!("Getting {} is not implemented yet!", varname);
    return None;
  }

  pub fn add_other(&mut self, other : &TofAnalysisPaddleCache) {
    for k in 1..161 {
      self.charge2d.get_mut(&k).unwrap().extend_from_slice(&other.charge2d.get(&k).unwrap());
      self.amp2d   .get_mut(&k).unwrap().extend_from_slice(&other.amp2d   .get(&k).unwrap());
      self.amp_a   .get_mut(&k).unwrap().extend_from_slice(&other.amp_a   .get(&k).unwrap());
      self.amp_b   .get_mut(&k).unwrap().extend_from_slice(&other.amp_b   .get(&k).unwrap());
      self.time_a  .get_mut(&k).unwrap().extend_from_slice(&other.time_a  .get(&k).unwrap());
      self.time_b  .get_mut(&k).unwrap().extend_from_slice(&other.time_b  .get(&k).unwrap());
      self.charge_a.get_mut(&k).unwrap().extend_from_slice(&other.charge_a.get(&k).unwrap());
      self.charge_b.get_mut(&k).unwrap().extend_from_slice(&other.charge_b.get(&k).unwrap());
      self.bl_a    .get_mut(&k).unwrap().extend_from_slice(&other.bl_a    .get(&k).unwrap());
      self.bl_b    .get_mut(&k).unwrap().extend_from_slice(&other.bl_b    .get(&k).unwrap());
      self.bl_a_rms.get_mut(&k).unwrap().extend_from_slice(&other.bl_a_rms.get(&k).unwrap());
      self.bl_b_rms.get_mut(&k).unwrap().extend_from_slice(&other.bl_b_rms.get(&k).unwrap());
      self.x0      .get_mut(&k).unwrap().extend_from_slice(&other.x0      .get(&k).unwrap());
      self.t0      .get_mut(&k).unwrap().extend_from_slice(&other.t0      .get(&k).unwrap());
      self.edep    .get_mut(&k).unwrap().extend_from_slice(&other.edep    .get(&k).unwrap());
      self.pos_edep.get_mut(&k).unwrap().extend_from_slice(&other.pos_edep.get(&k).unwrap());
    }
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TofAnalysisPaddleCache { 

  #[staticmethod]
  #[pyo3(name="keys")]
  fn keys_py() -> Vec<&'static str> {
    Self::keys()
  }
  
  //pub charge2d    : HashMap<u8,Vec<(f32,f32)>>,
  //pub amp2d       : HashMap<u8,Vec<(f32,f32)>>,
  //pub pos_edep    : HashMap<u8,Vec<(f32, f32)>>,
 
  fn cache_size(&self, pid : u8) -> usize {
    self.time_a.get(&pid).unwrap().len()
  }
  
  //fn get_tuple_data<'py>(&'py self, py: Python<'py>, name : &str, pid : u8) -> Option<Bound<'py, PyArray1<(f32,f32)>>> {
  //  match name {
  //    "charge2d" => {
  //      let slice = &self.charge2d.get(&pid).unwrap()[..];
  //      // this is supposed to be readonly
  //      // FIXME - check this!
  //      let py_array = PyArray1::from_slice(py, slice);
  //      return Some(py_array);
  //    }
  //    "amp2d" => {
  //      let slice = &self.amp2d.get(&pid).unwrap()[..];
  //      // this is supposed to be readonly
  //      // FIXME - check this!
  //      let py_array = PyArray1::from_slice(py, slice);
  //      return Some(py_array);
  //    }
  //    "pos_edep" => {
  //      let slice = &self.pos_edep.get(&pid).unwrap()[..];
  //      // this is supposed to be readonly
  //      // FIXME - check this!
  //      let py_array = PyArray1::from_slice(py, slice);
  //      return Some(py_array);
  //    }
  //    _ => {
  //      return None;
  //    }
  //  }
  //}

  //#[pyo3(name="_get_data")]
  fn get_f32_data<'py>(&'py self, py: Python<'py>, name : &str, pid : u8) -> Option<Bound<'py, PyArray1<f32>>> {
    match name {
      "amp_a" => {
        let slice = &self.amp_a.get(&pid).unwrap()[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "amp_b" => {
        let slice = &self.amp_b.get(&pid).unwrap()[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "time_a" => {
        let slice = &self.time_a.get(&pid).unwrap()[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "time_b" => {
        let slice = &self.time_b.get(&pid).unwrap()[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "charge_a" => {
        let slice = &self.charge_a.get(&pid).unwrap()[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "charge_b" => {
        let slice = &self.charge_b.get(&pid).unwrap()[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "bl_a" => {
        let slice = &self.bl_a.get(&pid).unwrap()[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "bl_b" => {
        let slice = &self.bl_b.get(&pid).unwrap()[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "bl_a_rms" => {
        let slice = &self.bl_a_rms.get(&pid).unwrap()[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "bl_b_rms" => {
        let slice = &self.bl_b_rms.get(&pid).unwrap()[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "x0" => {
        let slice = &self.x0.get(&pid).unwrap()[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "t0" => {
        let slice = &self.t0.get(&pid).unwrap()[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      "edep" => {
        let slice = &self.edep.get(&pid).unwrap()[..];
        // this is supposed to be readonly
        // FIXME - check this!
        let py_array = PyArray1::from_slice(py, slice);
        return Some(py_array);
      }
      _ => {
        return None;
      }
    }
  }
  
  /// Access the (data) members by name
  #[pyo3(name="get")]
  fn get_py(&self, varname : &str) -> PyResult<f32> {
    match self.get(varname) {
      None => {
        let err_msg = format!("{} does not have a key with name {}! See {}.keys() for a list of available keys!",stringify!($pyclass), stringify!($pyclass), varname);
        return Err(PyKeyError::new_err(err_msg));
      }
      Some(val) => {
        return Ok(val)
      }
    }
  }

  #[pyo3(name="clear")]
  fn clear_py(&mut self, name : &str, k : u8) -> PyResult<()> {
    match name {
      "charge2d"   =>   {
        self.charge2d.get_mut(&k).unwrap().clear();
        Ok(())
      }
      "amp2d"      =>   {
        self.amp2d   .get_mut(&k).unwrap().clear();
        Ok(())
      }
      "amp_a"      =>   {
        self.amp_a   .get_mut(&k).unwrap().clear();
        Ok(())
      }
      "amp_b"      =>   {
        self.amp_b   .get_mut(&k).unwrap().clear();
        Ok(())
      }
      "time_a"     =>   {
        self.time_a  .get_mut(&k).unwrap().clear();
        Ok(())
      }
      "time_b"     =>   {
        self.time_b  .get_mut(&k).unwrap().clear();
        Ok(())
      }
      "charge_a"   =>   {
        self.charge_a.get_mut(&k).unwrap().clear();
        Ok(())
      }
      "charge_b"   =>   {
        self.charge_b.get_mut(&k).unwrap().clear();
        Ok(())
      }
      "bl_a"       =>   {
        self.bl_a    .get_mut(&k).unwrap().clear();
        Ok(())
      }
      "bl_b"       =>   {
        self.bl_b    .get_mut(&k).unwrap().clear();
        Ok(())
      }
      "bl_a_rms"   =>   {
        self.bl_a_rms.get_mut(&k).unwrap().clear();
        Ok(())
      }
      "bl_b_rms"   =>   {
        self.bl_b_rms.get_mut(&k).unwrap().clear();
        Ok(())
      }
      "x0"         =>   {
        self.x0      .get_mut(&k).unwrap().clear();
        Ok(())
      }
      "t0"         =>   {
        self.t0      .get_mut(&k).unwrap().clear();
        Ok(())
      }
      "edep"       =>   {
        self.edep    .get_mut(&k).unwrap().clear();
        Ok(())
      }
      "pos_edep"   =>   {
        self.pos_edep.get_mut(&k).unwrap().clear();
        Ok(())
      }
      _ => {
        let err_msg = format!("No key with name {}", name);
        return Err(PyKeyError::new_err(err_msg));
      }
    }
  }
}

/// A container to hold a cut selection and allows 
/// to walk over files and fills a number of histograms 
///
/// FIXME - typically these monolithic structures are 
///         not a good idea
///
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TofAnalysis {
  #[pyo3(get,set)]
  pub skip_mangled  : bool,
  #[pyo3(get,set)]
  pub skip_timeout  : bool,
  #[pyo3(get)]
  pub beta_analysis : bool,
  pub nbins         : u64,
  #[pyo3(get,set)]
  pub cuts          : TofCuts,
  pub use_offsets   : bool,
  /// The "infamous" TOF Paddle timing constants
  pub timing_const  : HashMap<u8,f32>,
  #[pyo3(get)]
  pub pid_inner     : Option<u8>,
  #[pyo3(get)]
  pub pid_outer     : Option<u8>,
  pub active        : bool,
  #[pyo3(get,set)]  
  pub n_mangled     : u64,
  #[pyo3(get,set)]  
  pub n_timed_out   : u64, 
  #[pyo3(get,set)]  
  pub nhit          : u64, 
  #[pyo3(get,set)]  
  pub no_hitmiss    : u64, 
  #[pyo3(get,set)]  
  pub one_hitmiss   : u64, 
  #[pyo3(get,set)]  
  pub two_hitmiss   : u64, 
  #[pyo3(get,set)]  
  pub extra_hits    : u64, 
  #[pyo3(get,set)]  
  pub n_events      : u64,
  #[pyo3(get,set)]  
  pub n_hits        : u64, 
  #[pyo3(get)]  
  pub occupancy     : HashMap<u8,u64>,
  #[pyo3(get)]  
  pub occupancy_t   : HashMap<u8,u64>,
  #[pyo3(get)]  
  pub event_stati   : HashMap<EventStatus, u64>,
  pub event_cache   : Vec<TofEvent>,
  #[pyo3(get,set)]  
  pub first_event_t : u64,
  #[pyo3(get,set)]  
  pub last_event_t  : u64,
  pub hg_mapping    : DsiJChPidMapping,
  // caches 
  pub c_hit         : Vec<u8>,
  pub c_hit_umb     : Vec<u8>,
  pub c_hit_cor     : Vec<u8>,
  pub c_hit_cbe     : Vec<u8>,
  pub c_thit        : Vec<u8>,
  pub c_rblink      : Vec<u8>,
  pub c_miss_hit    : Vec<u8>,   
  pub c_nc_pid      : Vec<u8>,
  #[pyo3(get)]  
  pub cache         : TofAnalysisCache,
  #[pyo3(get)]  
  pub paddle_cache  : TofAnalysisPaddleCache,
  pub paddles       : HashMap<u8,TofPaddle>
}


impl TofAnalysis {

  fn clear_hit_stats(&mut self) {
    self.c_hit      .clear();
    self.c_hit_cbe  .clear();
    self.c_hit_umb  .clear();
    self.c_hit_cor  .clear();
    self.c_thit     .clear();
    self.c_rblink   .clear();
    self.c_miss_hit .clear();   
    self.c_nc_pid   .clear();
  }
 

  fn new() -> Self {
    let timing_offsets = TofPaddleTimingConstant::as_dict_by_name("GraceV1").expect("Unable to get timing constants!");
    let mut timing_const   = HashMap::<u8,f32>::new();
    for k in timing_offsets.keys() {
      timing_const.insert(*k,timing_offsets.get(&k).unwrap().timing_constant);
    }
    let paddles     = TofPaddle::all().expect("Can not load paddles from DB!");
    let paddle_dict = TofPaddle::all_as_dict().expect("Can not load paddles from DB!");
    let mut occu_map = HashMap::<u8,u64>::new();
    for k in 1..161 {
      occu_map.insert(k, 0);
    }
    Self {
      skip_mangled  : false,
      skip_timeout  : false,
      beta_analysis : true,
      nbins         : 0,
      cuts          : TofCuts::new(),
      use_offsets   : false,
      timing_const  : timing_const,
      pid_inner     : None,
      pid_outer     : None,
      active        : false,
      n_mangled     : 0,
      n_timed_out   : 0,
      nhit          : 0, 
      no_hitmiss    : 0, 
      one_hitmiss   : 0, 
      two_hitmiss   : 0, 
      extra_hits    : 0, 
      n_events      : 0,
      n_hits        : 0,
      occupancy     : occu_map.clone(),
      occupancy_t   : occu_map.clone(),
      event_cache   : Vec::<TofEvent>::new(),
      event_stati   : HashMap::<EventStatus,u64>::new(),
      first_event_t : u64::MAX,
      last_event_t  : 0,
      hg_mapping    : get_dsi_j_ch_pid_map(&paddles),
      c_miss_hit    : Vec::<u8>::new(),
      c_nc_pid      : Vec::<u8>::new(),
      c_hit         : Vec::<u8>::new(),
      c_hit_umb     : Vec::<u8>::new(),
      c_hit_cor     : Vec::<u8>::new(),
      c_hit_cbe     : Vec::<u8>::new(),
      c_thit        : Vec::<u8>::new(),
      c_rblink      : Vec::<u8>::new(),
      cache         : TofAnalysisCache::new(),
      paddle_cache  : TofAnalysisPaddleCache::new(),
      paddles       : paddle_dict,
    }
  }

  pub fn get_n_mangled(&self) -> u64 {
    *self.event_stati.get(&EventStatus::AnyDataMangling).unwrap_or(&0)
  }

  pub fn get_n_timedout(&self) -> u64 {
    *self.event_stati.get(&EventStatus::EventTimeOut).unwrap_or(&0)
  }

  fn add_event(&mut self, ev : &mut TofEvent) {
    if self.first_event_t == u64::MAX {
      self.first_event_t = ev.get_timestamp48();
    } else if self.first_event_t > ev.get_timestamp48() {
      self.first_event_t = ev.get_timestamp48();   
    }
    if ev.get_timestamp48() > self.last_event_t {
      self.last_event_t  = ev.get_timestamp48();
    }

    if self.event_stati.contains_key(&ev.status) {
      *self.event_stati.get_mut(&ev.status).unwrap() += 1;  
    } else {
      self.event_stati.insert(ev.status, 1);
    }
    
    if ev.status == EventStatus::AnyDataMangling {
      //debug!"Found mangled event with id {ev.event_id}");
      self.n_mangled += 1;
      if self.skip_mangled {
        return;
      }
    }
    if ev.status == EventStatus::EventTimeOut {
      //debug!(f'Found timed out event with id {ev.event_id}')
      self.n_timed_out += 1;
      if self.skip_timeout {
        return;
      }
    }
    let mut nhit_ev   = 0u8;
    let mut nhit_t_ev = 0u8;
    self.n_events += 1;
    //# at the very first, add the timings if desired
    if self.use_offsets {
      ev.set_timing_offsets(&self.timing_const);
      //print (ev)
    }
    ev.normalize_hit_times();
    ev.calc_gcu_variables();

    //# before cutting, calculate missing hits
    //# the problem for removing hits right now
    //# is the fact that if we do a hit cleaning,
    //# it will be only for the HG hits and not 
    //# the LG hits, so if we do a missing hit calculation 
    //# after the hit cleaning, we will artificially 
    //# increase the number of missing hits
    //# FIXME - this is currently a bit inconsistent.
    let mut missing  = ev.get_missing_paddles_hg(&self.hg_mapping);
    self.c_miss_hit.append(&mut missing);

    if !self.cuts.is_void() {
      let mut ev_for_cuts = ev.clone();
      if !self.cuts.accept(&mut ev_for_cuts) {
        return;
      }
    }
    
    //# if desired, apply the cleanings
    if self.cuts.only_causal_hits {
      let mut rm_pids = ev.remove_non_causal_hits();
      self.c_nc_pid.append(&mut rm_pids);
      //#hits_rmvd_csl  = len(rm_pids)
    }
    if self.cuts.ls_cleaning_t_err != NO_LIGHTSPEED_CUTS {
      // FIXME
      let _rm_pids = ev.lightspeed_cleaning(self.cuts.ls_cleaning_t_err as f32);
    }
    //    #hits_rmvd_ls   = len(rm_pids)

    for h in ev.get_trigger_hits() {
      let pid : u8;
      if self.hg_mapping.get(&h.0).unwrap().get(&h.1).unwrap().contains_key(&h.2.0) {
        pid = self.hg_mapping.get(&h.0).unwrap().get(&h.1).unwrap().get(&h.2.0).unwrap().0; 
      } else {
        pid = self.hg_mapping.get(&h.0).unwrap().get(&h.1).unwrap().get(&h.2.1).unwrap().0; 
      }
      *self.occupancy_t.get_mut(&pid).unwrap() += 1;
      nhit_t_ev += 1;
    }
    let mut outer_h    = Vec::<&TofHit>::new();
    let mut inner_h    = Vec::<&TofHit>::new();
    let mut event_edep = 0f32;
    for h in &ev.hits {
    //    # for gondola, the hits should have paddle information 
    //    # already
      let pdl = self.paddles.get(&h.paddle_id).unwrap();
      //h.set_paddle(10*pdl.length, pdl.cable_len, pdl.coax_cable_time, pdl.harting_cable_time)
      if pdl.panel_id < 22 {
        //let edep_key = f'edep_pnl{pdl.panel_id}'
        let pnl = pdl.panel_id as u8;
        self.cache.edep_panel.get_mut(&pnl).unwrap().push(h.get_edep());
        //self.edep_cache['edep'].append(h.edep);
      }
      if h.get_edep() > 0.0 {
        *self.occupancy.get_mut(&h.paddle_id).unwrap() += 1;
        event_edep += h.get_edep();
      }
      nhit_ev += 1;
      if self.beta_analysis {
        if self.pid_outer.is_none() {
          if h.paddle_id > 60 {
            outer_h.push(h)
          }
        } else { 
          if h.paddle_id == self.pid_outer.unwrap() {
            outer_h.push(h);
          }
        }
        if self.pid_inner.is_none() {
          if h.paddle_id < 61 {
            inner_h.push(h)
          }
        } else {
          if h.paddle_id == self.pid_inner.unwrap() {
            inner_h.push(h)
          }
        }
      }  
      // fill the caches
      if h.charge_a.to_f32() < 0.0 || h.charge_b.to_f32() < 0.0 {
      //#    print (h)
      //#    raise ValueError
      }
      self.paddle_cache.charge2d.get_mut(&h.paddle_id).unwrap().push((h.get_charge_a(), h.get_charge_b()));    
      self.paddle_cache.amp2d   .get_mut(&h.paddle_id).unwrap().push((h.get_peak_a(), h.get_peak_b()));        
      self.paddle_cache.amp_a   .get_mut(&h.paddle_id).unwrap().push(h.get_peak_a());                    
      self.paddle_cache.amp_b   .get_mut(&h.paddle_id).unwrap().push(h.get_peak_b());                    
      self.paddle_cache.time_a  .get_mut(&h.paddle_id).unwrap().push(h.get_time_a());                    
      self.paddle_cache.time_b  .get_mut(&h.paddle_id).unwrap().push(h.get_time_b());                    
      self.paddle_cache.charge_a.get_mut(&h.paddle_id).unwrap().push(h.get_charge_a());                  
      self.paddle_cache.charge_b.get_mut(&h.paddle_id).unwrap().push(h.get_charge_b());                  
      self.paddle_cache.bl_a    .get_mut(&h.paddle_id).unwrap().push(h.get_bl_a());                
      self.paddle_cache.bl_b    .get_mut(&h.paddle_id).unwrap().push(h.get_bl_b());                
      self.paddle_cache.bl_a_rms.get_mut(&h.paddle_id).unwrap().push(h.get_bl_a_rms());            
      self.paddle_cache.bl_b_rms.get_mut(&h.paddle_id).unwrap().push(h.get_bl_b_rms());            
      self.paddle_cache.x0      .get_mut(&h.paddle_id).unwrap().push(h.get_pos()/h.paddle_len);          
      self.paddle_cache.t0      .get_mut(&h.paddle_id).unwrap().push(h.event_t0);                  
      self.paddle_cache.edep    .get_mut(&h.paddle_id).unwrap().push(h.get_edep());                      
      self.paddle_cache.pos_edep.get_mut(&h.paddle_id).unwrap().push((h.get_pos()/h.paddle_len, h.get_edep())); 
    }
    //# hit counting 
    let n_rblink_ev = ev.get_rb_link_ids().len() as u8;
    self.n_hits     += nhit_ev as u64;
    if nhit_t_ev == nhit_ev {
      self.no_hitmiss += 1;
    } else if nhit_t_ev - nhit_ev == 1 {
      self.one_hitmiss += 1;
    } else if nhit_t_ev - nhit_ev > 1 {
      self.two_hitmiss += 1;
    } else if nhit_ev > nhit_t_ev {
      self.extra_hits += 1;
    }
    self.c_hit_umb.push(ev.n_hits_umb);
    self.c_hit_cor.push(ev.n_hits_cor);
    self.c_hit_cbe.push(ev.n_hits_cbe);
    self.c_hit.push(nhit_ev);
    self.c_thit.push(nhit_t_ev);
    self.c_rblink.push(n_rblink_ev);
    self.cache.edep_umb.push(ev.tot_edep_umb);
    self.cache.edep_cor.push(ev.tot_edep_cor);
    self.cache.edep_cbe.push(ev.tot_edep_cbe);
    if !self.beta_analysis {
      return
    }
    //println!("Len outer h {}", outer_h.len());
    //println!("Len innner h {}", inner_h.len());
    outer_h.sort_unstable_by(|x,y| x.event_t0.total_cmp(&y.event_t0));
    inner_h.sort_unstable_by(|x,y| x.event_t0.total_cmp(&y.event_t0));
    let mut beta = 0.0f32;
    if inner_h.len() > 0 && outer_h.len() > 0 {
      //println!("Doing advanced analysis!");
      //#first_hit = sorted([h for h in ev.hits], key=lambda x: x.phase_delay)
      //#last_hit  = first_hit[-1].phase_delay
      //#first_hit = first_hit[0].phase_delay
      //#print (inner_h, outer_h)
      let diff_h     = inner_h[0].event_t0 - outer_h[0].event_t0; 
      let dist      = inner_h[0].distance(outer_h[0])/1000.0;
      let cos_theta = (outer_h[0].z - inner_h[0].z).abs()/(1000.0*dist);  
      if diff_h != 0.0 {
        beta = dist/(diff_h*1e-9)/299792458.0;
      }
      self.cache.dist      .push(dist);
      self.cache.x_outer   .push(outer_h[0].x);
      self.cache.y_outer   .push(outer_h[0].y);
      self.cache.z_outer   .push(outer_h[0].z);
      self.cache.x_inner   .push(inner_h[0].x);
      self.cache.y_inner   .push(inner_h[0].y);
      self.cache.z_inner   .push(inner_h[0].z);
      self.cache.pid_inner .push(inner_h[0].paddle_id);
      self.cache.pid_outer .push(outer_h[0].paddle_id);
      self.cache.cos_theta .push(cos_theta);
      self.cache.cos2_theta.push(cos_theta*cos_theta);
      if beta < 0.0 {
        beta = -1.0*beta;
      }
      self.cache.beta    .push(beta);
      self.cache.t_outer .push(outer_h[0].event_t0);
      self.cache.t_inner .push(inner_h[0].event_t0); 
      self.cache.t_diff  .push(inner_h[0].event_t0 - outer_h[0].event_t0);  
      self.cache.ph_delay.push(inner_h[0].get_phase_delay() - outer_h[0].get_phase_delay());
      self.cache.edep    .push(event_edep);
    }
    //# fill is the massive bottleneck here, thus let's try to reduce the amount of calls 
    //self.fill_histograms()    
    //return 
  }
}

//--------------------------------------------------

impl fmt::Display for TofAnalysis {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = format!("<TofAnalysis [nevents: {}]", self.n_events);
    //repr += &(format!("\n  EventID          : {}", self.event_id));
    repr += ">";
    write!(f, "{}", repr)
  }
}

//--------------------------------------------------

#[cfg(feature="pybindings")]
#[pymethods]
impl TofAnalysis {

  fn add_other_cache(&mut self, other : &TofAnalysis) {
    self.cache.add_other(&other.cache);
  }

  fn add_other_paddle_cache(&mut self, other : &TofAnalysis) {
    self.paddle_cache.add_other(&other.paddle_cache);  
  }

  fn add_other_event_stati(&mut self, other : &TofAnalysis) {
    for st in other.event_stati.keys() {
      if self.event_stati.contains_key(st) {
        let value = other.event_stati.get(&st).unwrap();
        *self.event_stati.get_mut(&st).unwrap() += value; 
      } else {
        self.event_stati.insert(*st, *other.event_stati.get(st).unwrap());
      }
    }
  }

  fn add_other_hit_cache(&mut self, other :  &TofAnalysis) {
    self.c_hit     .extend_from_slice(&other.c_hit);
    self.c_hit_umb .extend_from_slice(&other.c_hit_umb);
    self.c_hit_cor .extend_from_slice(&other.c_hit_cor);
    self.c_hit_cbe .extend_from_slice(&other.c_hit_cbe);
    self.c_thit    .extend_from_slice(&other.c_thit);
    self.c_rblink  .extend_from_slice(&other.c_rblink);
    self.c_miss_hit.extend_from_slice(&other.c_miss_hit);   
    self.c_nc_pid  .extend_from_slice(&other.c_nc_pid);
  }

  fn add_other_occupancy_t(&mut self, other : HashMap<u8,u64>) {
    for k in 1..161u8 {
      *self.occupancy_t.get_mut(&k).unwrap() += other.get(&k).unwrap();
    }
  }

  fn add_other_occupancy(&mut self, other : HashMap<u8,u64>) {
    for k in 1..161u8 {
      *self.occupancy.get_mut(&k).unwrap() += other.get(&k).unwrap();
    }
  }

  #[getter]
  fn hit_cache_len(&self) -> usize {
    return self.c_hit.len()
  }

  #[pyo3(name="clear_hit_stats")]
  fn clear_hit_stats_py(&mut self) {
    self.clear_hit_stats();
  }
 
  /// This is the number of hits/event for each seen event
  #[pyo3(name="c_hit")]
  #[getter]
  fn c_hit_py<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<u8>>> {
    let slice = &self.c_hit[..];
    // this is supposed to be readonly
    // FIXME - check this!
    let py_array = PyArray1::from_slice(py, slice);
    return Ok(py_array);
  }
  
  /// This is the number of hits/event for each seen event
  #[pyo3(name="c_hit_cbe")]
  #[getter]
  fn c_hit_cbe_py<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<u8>>> {
    let slice = &self.c_hit_cbe[..];
    // this is supposed to be readonly
    // FIXME - check this!
    let py_array = PyArray1::from_slice(py, slice);
    return Ok(py_array);
  }
  
  /// This is the number of hits/event for each seen event
  #[pyo3(name="c_hit_umb")]
  #[getter]
  fn c_hit_umb_py<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<u8>>> {
    let slice = &self.c_hit_umb[..];
    // this is supposed to be readonly
    // FIXME - check this!
    let py_array = PyArray1::from_slice(py, slice);
    return Ok(py_array);
  }
  
  /// This is the number of hits/event for each seen event
  #[pyo3(name="c_hit_cor")]
  #[getter]
  fn c_hit_cor_py<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<u8>>> {
    let slice = &self.c_hit_cor[..];
    // this is supposed to be readonly
    // FIXME - check this!
    let py_array = PyArray1::from_slice(py, slice);
    return Ok(py_array);
  }
  
  /// This is the number of TRIGGER hits/event for each seen event
  #[pyo3(name="c_thit")]
  #[getter]
  fn c_thit_py<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<u8>>> {
    let slice = &self.c_thit[..];
    // this is supposed to be readonly
    // FIXME - check this!
    let py_array = PyArray1::from_slice(py, slice);
    return Ok(py_array);
  }

  /// This is the number of RBLINK IDs (participation RBs)/event for each seen event
  #[pyo3(name="c_rblink")]
  #[getter]
  fn c_rblink_py<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<u8>>> {
    let slice = &self.c_rblink[..];
    // this is supposed to be readonly
    // FIXME - check this!
    let py_array = PyArray1::from_slice(py, slice);
    return Ok(py_array);
  }
  /// This is the number of MISSING HG hits/event for each seen event
  #[pyo3(name="c_miss_hit")]
  #[getter]
  fn c_miss_hit_py<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<u8>>> {
    let slice = &self.c_miss_hit[..];
    // this is supposed to be readonly
    // FIXME - check this!
    let py_array = PyArray1::from_slice(py, slice);
    return Ok(py_array);
  }

  /// This is the number of REMOVED HITS (due to causal incompatibility) for each event
  #[pyo3(name="c_nc_pid")]
  #[getter]
  fn c_nc_pid_py<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<u8>>> {
    let slice = &self.c_nc_pid[..];
    // this is supposed to be readonly
    // FIXME - check this!
    let py_array = PyArray1::from_slice(py, slice);
    return Ok(py_array);
  }

  #[pyo3(name="add_event")]
  fn add_event_py(&mut self, ev : &mut TofEvent) {
    self.add_event(ev);
  }

}

//--------------------------------------------------

#[cfg(feature="pybindings")]
pythonize!(TofAnalysis);

