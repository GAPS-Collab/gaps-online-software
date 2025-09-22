// This file is part of gaps-online-software and published 
// under the GPLv3 license

pub use std::sync::Arc;
pub use std::collections::{
  HashMap,
  VecDeque,
  HashSet
};

pub use std::path::Path;
pub use std::thread;
pub use std::error::Error;
pub use std::fmt;
pub use std::env;
pub use std::io::{
  self,
  BufReader,
  Read,
  Seek,
  SeekFrom,
  Write,
};

pub use std::net::{
  UdpSocket,
  SocketAddr
};

pub use std::time::{
  Duration,
  Instant
};

pub use std::fs::{
  self,
  File,
  OpenOptions,
};

pub use crossbeam_channel::Sender;

pub use chrono::{
  Utc,
  DateTime,
  NaiveDateTime,
  TimeZone,
  LocalResult
};

pub use glob::glob;
pub use indicatif::{
  ProgressBar,
  ProgressStyle
};

// avoiding boilerplate for enums
pub use strum_macros::{
  AsRefStr,
  FromRepr,
  EnumIter,
};
pub use strum::IntoEnumIterator;

pub use half::f16;

pub use num_traits::{
  NumAssign,
  NumCast,
  Float,
  FromBytes,
  FloatConst,
  NumOps,
  NumAssignOps,
};

pub use regex::Regex;    
pub use statistical::median;

#[cfg(feature="advanced-algorithms")]
pub use smoothed_z_score::{
  Peak,
  PeaksDetector,
  PeaksFilter
};

#[cfg(feature="random")]
pub use rand::Rng;

#[cfg(feature="pybindings")]
pub use pyo3::prelude::*; 
#[cfg(feature="pybindings")]
pub use pyo3::wrap_pymodule; 
#[cfg(feature="pybindings")]
pub use pyo3::wrap_pyfunction; 

#[cfg(feature="pybindings")]
pub use pyo3::exceptions::{
  PyIOError,
  PyValueError,
  PyKeyError
};

#[cfg(feature="pybindings")]
pub use pyo3::types::PyBytes;

#[cfg(feature="pybindings")]
pub use numpy::{
  ToPyArray,
  PyArrayMethods,
  PyArray1,
  PyReadonlyArray1,
  PyArray2, 
};

pub use crate::stats::{
  mean,
  calculate_column_stat,
  standard_deviation
};

//#[cfg(feature = "pybindings")]
//pub use polars::prelude::*;

#[cfg(feature = "pybindings")]
pub use pyo3_polars::{
  PyDataFrame,
  //PySeries
};

#[cfg(feature = "pybindings")] 
pub use polars::frame::DataFrame;
#[cfg(feature = "pybindings")]
pub use polars::series::Series; 
#[cfg(feature = "pybindings")]
pub use polars::error::PolarsResult;

#[cfg(not(feature="database"))]
#[derive(Debug, Default, Clone)]
pub struct TofPaddle {}
#[cfg(not(feature="database"))]
#[derive(Debug, Default, Clone)]
pub struct TrackerStrip {}
#[cfg(not(feature="database"))]
#[derive(Debug, Default, Clone)]
pub struct TrackerStripMask {}
#[cfg(not(feature="database"))]
#[derive(Debug, Default, Clone)]
pub struct TrackerStripPedestal {}
#[cfg(not(feature="database"))]
#[derive(Debug, Default, Clone)]
pub struct TrackerStripTransferFunction {}
#[cfg(not(feature="database"))]
#[derive(Debug, Default, Clone)]
pub struct TrackerStripCmnNoise {}



#[cfg(feature="pybindings")]
pub use crate::pythonize_packable;
#[cfg(feature="pybindings")]
pub use crate::pythonize_packable_no_new;
#[cfg(feature="pybindings")]
pub use crate::pythonize;
#[cfg(feature="pybindings")]
pub use crate::pythonize_telemetry;
#[cfg(feature="pybindings")]
pub use crate::pythonize_display;
#[cfg(feature="pybindings")]
pub use crate::pythonize_monidata;
pub use crate::moniseries;

#[cfg(feature="random")]
pub use crate::random::FromRandom;

pub use crate::version::ProtocolVersion;
pub use crate::constants::*;
pub use crate::errors::*;
pub use crate::io::*;
pub use crate::io::parsers::*;
pub use crate::io::caraspace::*;
pub use crate::io::serialization::*;
pub use crate::events::*;
pub use crate::packets::*;
#[cfg(feature="database")]
pub use crate::database::*;
//#[cfg(not(feature="database"))]
//pub use crate::TrackerStrip;
//#[cfg(not(feature="database"))]
//pub use crate::TofPaddle;

pub use crate::calibration::tof::*;
pub use crate::monitoring::*;
pub use crate::tof::*;
pub use crate::tracker::*;
pub use crate::tof::algorithms::*;
//pub use crate::algorithms::*;

pub use crate::packets::{
  TofPacket,
  TofPackable,
  TofPacketType
};

pub use crate::io::parsers::{
  parse_u8,
  parse_u16,
  parse_u32,
  parse_u64,
  parse_string,
  u8_to_u16,
};

pub use crate::constants::{
  NWORDS,
  NCHN
};

pub use crate::errors::{
  SerializationError,
  AnalysisError,
  UserError
};

// macro to avoid boring enum boilerplate
pub use crate::expand_and_test_enum;
pub use crate::reader;

