use tof_dataclasses::io::TofPacketReader;
use tof_dataclasses::packets::{
    TofPacket,
    PacketType
};

extern crate pyo3_polars;
use pyo3_polars::{
    PyDataFrame,
    //PySeries
};

use tof_dataclasses::monitoring::{
    MoniSeries,
    PAMoniData,
    PBMoniData,
    RBMoniData,
    MtbMoniData, 
    CPUMoniData,
    LTBMoniData,
};

use tof_dataclasses::series::{
    PAMoniDataSeries,
    PBMoniDataSeries,
    RBMoniDataSeries,
    MtbMoniDataSeries,
    CPUMoniDataSeries,
    LTBMoniDataSeries,
};

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;


#[pyclass]
#[pyo3(name="PAMoniSeries")]
pub struct PyPAMoniSeries {
  pamoniseries : PAMoniDataSeries,
}

#[pymethods]
impl PyPAMoniSeries {
  #[new]
  fn new() -> Self {
    let pamoniseries = PAMoniDataSeries::new();
    Self {
      pamoniseries,
    }
  }
  
  fn from_file(&mut self, filename : String) -> PyResult<PyDataFrame> {
    let mut reader = TofPacketReader::new(filename);
    reader.set_filter(PacketType::PAMoniData);
    for tp in reader {
      if let Ok(moni) =  tp.unpack::<PAMoniData>() {
        self.pamoniseries.add(moni);
      }
    }
    match self.pamoniseries.get_dataframe() {
      Ok(df) => {
        let pydf = PyDataFrame(df);
        return Ok(pydf);
      },
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}

#[pyclass]
#[pyo3(name="PBMoniSeries")]
pub struct PyPBMoniSeries {
  pbmoniseries : PBMoniDataSeries,
}

#[pymethods]
impl PyPBMoniSeries {
  #[new]
  fn new() -> Self {
    let pbmoniseries = PBMoniDataSeries::new();
    Self {
      pbmoniseries,
    }
  }
  
  fn from_file(&mut self, filename : String) -> PyResult<PyDataFrame> {
    let mut reader = TofPacketReader::new(filename);
    reader.set_filter(PacketType::PBMoniData);
    for tp in reader {
      //if tp.packet_type == PacketType::PBMoniData {
      if let Ok(moni) =  tp.unpack::<PBMoniData>() {
        self.pbmoniseries.add(moni);
      }
      //}
    }
    match self.pbmoniseries.get_dataframe() {
      Ok(df) => {
        let pydf = PyDataFrame(df);
        return Ok(pydf);
      },
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}

#[pyclass]
#[pyo3(name="RBMoniSeries")]
pub struct PyRBMoniSeries {
  rbmoniseries : RBMoniDataSeries,
}

#[pymethods]
impl PyRBMoniSeries {
  #[new]
  fn new() -> Self {
    let rbmoniseries = RBMoniDataSeries::new();
    Self {
      rbmoniseries,
    }
  }
  
  fn from_file(&mut self, filename : String) -> PyResult<PyDataFrame> {
    let mut reader = TofPacketReader::new(filename);
    reader.set_filter(PacketType::RBMoniData);
    for tp in reader {
      if let Ok(moni) =  tp.unpack::<RBMoniData>() {
        self.rbmoniseries.add(moni);
      }
    }
    match self.rbmoniseries.get_dataframe() {
      Ok(df) => {
        let pydf = PyDataFrame(df);
        return Ok(pydf);
      },
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}

#[pyclass]
#[pyo3(name="MtbMoniSeries")]
pub struct PyMtbMoniSeries {
  mtbmoniseries : MtbMoniDataSeries,
}

#[pymethods]
impl PyMtbMoniSeries {
  #[new]
  fn new() -> Self {
    let mtbmoniseries = MtbMoniDataSeries::new();
    Self {
      mtbmoniseries,
    }
  }
  
  fn from_file(&mut self, filename : String) -> PyResult<PyDataFrame> {
    let mut reader = TofPacketReader::new(filename);
    reader.set_filter(PacketType::MonitorMtb);
    for tp in reader {
      if let Ok(moni) =  tp.unpack::<MtbMoniData>() {
        self.mtbmoniseries.add(moni);
      }
    }
    match self.mtbmoniseries.get_dataframe() {
      Ok(df) => {
        let pydf = PyDataFrame(df);
        return Ok(pydf);
      },
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}

#[pyclass]
#[pyo3(name="CPUMoniSeries")]
pub struct PyCPUMoniSeries {
  cpumoniseries : CPUMoniDataSeries,
}

#[pymethods]
impl PyCPUMoniSeries {
  #[new]
  fn new() -> Self {
    let cpumoniseries = CPUMoniDataSeries::new();
    Self {
      cpumoniseries,
    }
  }
  
  fn from_file(&mut self, filename : String) -> PyResult<PyDataFrame> {
    let mut reader = TofPacketReader::new(filename);
    reader.set_filter(PacketType::CPUMoniData);
    for tp in reader {
      if let Ok(moni) =  tp.unpack::<CPUMoniData>() {
        self.cpumoniseries.add(moni);
      }
    }
    match self.cpumoniseries.get_dataframe() {
      Ok(df) => {
        let pydf = PyDataFrame(df);
        return Ok(pydf);
      },
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}

#[pyclass]
#[pyo3(name="LTBMoniSeries")]
pub struct PyLTBMoniSeries {
  ltbmoniseries : LTBMoniDataSeries,
}

#[pymethods]
impl PyLTBMoniSeries {
  #[new]
  fn new() -> Self {
    let ltbmoniseries = LTBMoniDataSeries::new();
    Self {
      ltbmoniseries,
    }
  }
  
  fn from_file(&mut self, filename : String) -> PyResult<PyDataFrame> {
    let mut reader = TofPacketReader::new(filename);
    reader.set_filter(PacketType::LTBMoniData);
    for tp in reader {
      if let Ok(moni) =  tp.unpack::<LTBMoniData>() {
        self.ltbmoniseries.add(moni);
      }
    }
    match self.ltbmoniseries.get_dataframe() {
      Ok(df) => {
        let pydf = PyDataFrame(df);
        return Ok(pydf);
      },
      Err(err) => {
        return Err(PyValueError::new_err(err.to_string()));
      }
    }
  }
}


#[pyclass]
#[pyo3(name="TofPacket")]
pub struct PyTofPacket {
  packet : TofPacket,
}

impl PyTofPacket {
  pub fn set_tp(&mut self, tp : TofPacket) {
    self.packet = tp;
  }
}

#[pymethods]
impl PyTofPacket {
  #[new]
  pub fn new() -> Self {
    Self {
      packet : TofPacket::new(),
    }
  }
  
  fn __repr__(&self) -> PyResult<String> {
    Ok(format!("<PyO3Wrapper: {}>", self.packet)) 
  }
}
