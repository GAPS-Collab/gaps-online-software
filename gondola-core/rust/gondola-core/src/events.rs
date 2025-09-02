// This file is part of gaps-online-software and published 
// under the GPLv3 license

pub mod tof_hit;
pub use tof_hit::TofHit;

pub mod rb_waveform;
pub use rb_waveform::RBWaveform;

pub mod rb_event_header;
pub use rb_event_header::RBEventHeader;

pub mod tof_event;
pub use tof_event::TofEvent;

pub mod rb_event;
pub use rb_event::{
  RBEvent,
  unpack_traces
};

pub mod tracker_hit;
pub use tracker_hit::TrackerHit;

pub mod telemetry_event;
pub use telemetry_event::TelemetryEvent;

use std::fmt;

use strum_macros::{
  AsRefStr,
  FromRepr,
  EnumIter
};
// needed for enum macro
// FIXME 
#[cfg(feature="random")]
use strum::IntoEnumIterator;
use crate::expand_and_test_enum;

#[cfg(feature="pybindings")]
use pyo3::prelude::*;

#[cfg(feature="random")]
use crate::random::FromRandom;
#[cfg(feature="random")]
use rand::Rng;

/// mask to decode LTB hit masks
pub const LTB_CH0 : u16 = 0x3   ;
/// mask to decode LTB hit masks
pub const LTB_CH1 : u16 = 0xc   ;
/// mask to decode LTB hit masks
pub const LTB_CH2 : u16 = 0x30  ; 
/// mask to decode LTB hit masks
pub const LTB_CH3 : u16 = 0xc0  ;
/// mask to decode LTB hit masks
pub const LTB_CH4 : u16 = 0x300 ;
/// mask to decode LTB hit masks
pub const LTB_CH5 : u16 = 0xc00 ;
/// mask to decode LTB hit masks
pub const LTB_CH6 : u16 = 0x3000;
/// mask to decode LTB hit masks
pub const LTB_CH7 : u16 = 0xc000;
/// mask to decode LTB channels from bitmask
pub const LTB_CHANNELS : [u16;8] = [
  LTB_CH0,
  LTB_CH1,
  LTB_CH2,
  LTB_CH3,
  LTB_CH4,
  LTB_CH5,
  LTB_CH6,
  LTB_CH7
];

/// Calculate an unique identifier for 
/// tracker strips from the position in 
/// the tracker stack
///
/// # Arguments:
///   * layer   : tracker layer (0-9)
///   * row     : row in layer  (0-6)
///   * module  : module in row (0-6)
///   * channel : channel in module (0-32) 
///
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn strip_id(layer : u8, row :u8, module : u8, channel : u8) -> u32 {
  channel as u32 + (module as u32)*100 + (row as u32)*10000 + (layer as u32)*100000
}
  
/// Get absolute timestamp as sent by the GPS and 
/// as seen by the MTB
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn mt_event_get_timestamp_abs48(mtb_timestamp : u32, gps_timestamp : u32, tiu_timestamp : u32) -> u64 {
  let gps = gps_timestamp as u64;
  let mut timestamp = mtb_timestamp as u64;
  if timestamp < tiu_timestamp as u64 {
    // it has wrapped
    timestamp += u32::MAX as u64 + 1;
  }
  let gps_mult = match 100_000_000u64.checked_mul(gps) {
  //let gps_mult = match 100_000u64.checked_mul(gps) {
    Some(result) => result,
    None => {
        // Handle overflow case here
        // Example: log an error, return a default value, etc.
        0 // Example fallback value
    }
  };

  let ts = gps_mult + (timestamp - tiu_timestamp as u64);
  ts
}

#[derive(Debug, Copy, Clone, PartialEq,FromRepr, AsRefStr, EnumIter)]
#[repr(u8)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
pub enum EventQuality {
  Unknown        =  0u8,
  Silver         = 10u8,
  Gold           = 20u8,
  Diamond        = 30u8,
  FourLeafClover = 40u8,
}

expand_and_test_enum!(EventQuality, test_eventquality_repr);

//--------------------------------------------

// Need serde here, so that we can add it to the liftof configs
#[derive(Debug, Copy, Clone, PartialEq,FromRepr, AsRefStr, EnumIter, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
pub enum TriggerType {
  Unknown         = 0u8,
  /// -> 1-10 "pysics" triggers
  Any             = 1u8,
  Track           = 2u8,
  TrackCentral    = 3u8,
  Gaps            = 4u8,
  Gaps633         = 5u8, 
  Gaps422         = 6u8,
  Gaps211         = 7u8,
  TrackUmbCentral = 8u8,
  Gaps1044        = 9u8,
  /// -> 20+ "Philip's triggers"
  /// Any paddle HIT in UMB  + any paddle HIT in CUB
  UmbCube         = 21u8,
  /// Any paddle HIT in UMB + any paddle HIT in CUB top
  UmbCubeZ        = 22u8,
  /// Any paddle HIT in UMB + any paddle hit in COR + any paddle hit in CUB 
  UmbCorCube      = 23u8,
  /// Any paddle HIT in COR + any paddle HIT in CUB SIDES
  CorCubeSide     = 24u8,
  /// Any paddle hit in UMB + any three paddles HIT in CUB
  Umb3Cube        = 25u8,
  /// > 100 -> Debug triggers
  Poisson         = 100u8,
  Forced          = 101u8,
  FixedRate       = 102u8,
  /// > 200 -> These triggers can not be set, they are merely
  /// the result of what we read out from the trigger mask of 
  /// the ltb
  ConfigurableTrigger = 200u8,
}

impl TriggerType {

  /// In the serialized data, trigger sources are represented by 2bytes. 
  /// This will regenerate a vector of trigger sources from these bytes
  pub fn transcode_trigger_sources(trigger_sources : u16) -> Vec<Self> {
    let mut t_types    = Vec::<Self>::new();
    let gaps_trigger   = trigger_sources >> 5 & 0x1 == 1;
    if gaps_trigger {
      t_types.push(TriggerType::Gaps);
    }
    let any_trigger    = trigger_sources >> 6 & 0x1 == 1;
    if any_trigger {
      t_types.push(TriggerType::Any);
    }
    let forced_trigger = trigger_sources >> 7 & 0x1 == 1;
    if forced_trigger {
      t_types.push(TriggerType::Forced);
    }
    let track_trigger  = trigger_sources >> 8 & 0x1 == 1;
    if track_trigger {
      t_types.push(TriggerType::Track);
    }
    let central_track_trigger
                       = trigger_sources >> 9 & 0x1 == 1;
    if central_track_trigger {
      t_types.push(TriggerType::TrackCentral);
    }
    t_types
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TriggerType {
  #[staticmethod]
  #[pyo3(name="transcode_trigger_sources")]
  fn transcode_trigger_sources_py(trigger_sources : u16) -> Vec<Self> {
    TriggerType::transcode_trigger_sources(trigger_sources)
  }
}

expand_and_test_enum!(TriggerType, test_triggertype_repr);

//--------------------------------------------

/// LTB Thresholds as passed on by the MTB
/// [See also](https://gaps1.astro.ucla.edu/wiki/gaps/images/gaps/5/52/LTB_Data_Format.pdf)
#[derive(Debug, Copy, Clone, PartialEq,FromRepr, AsRefStr, EnumIter)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
#[repr(u8)]
pub enum LTBThreshold {
  NoHit = 0u8,
  /// First threshold, 40mV, about 0.75 minI
  Hit   = 1u8,
  /// Second threshold, 32mV (? error in doc ?, about 2.5 minI
  Beta  = 2u8,
  /// Third threshold, 375mV about 30 minI
  Veto  = 3u8,
  /// Use u8::MAX for Unknown, since 0 is pre-determined for 
  /// "NoHit, 
  Unknown = 255u8
}

expand_and_test_enum!(LTBThreshold, test_ltbthreshold_repr);

//--------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq,FromRepr, AsRefStr, EnumIter)]
#[repr(u8)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
pub enum EventStatus {
  Unknown                = 0u8,
  CRC32Wrong             = 10u8,
  TailWrong              = 11u8,
  ChannelIDWrong         = 12u8,
  /// one of the channels cells CellSyncError bits 
  /// has been set (RB)
  CellSyncErrors         = 13u8,
  /// one of the channels ChannelSyncError bits 
  /// has been set (RB)
  ChnSyncErrors          = 14u8,
  /// Both of the bits (at least one for the cell sync errors)
  /// have been set
  CellAndChnSyncErrors   = 15u8,
  /// If any of the RBEvents have Sync erros, we flag the tof 
  /// event summary to indicate there were issues
  AnyDataMangling        = 16u8,
  IncompleteReadout      = 21u8,
  /// This can be used if there is a version
  /// missmatch and we have to hack something
  IncompatibleData       = 22u8,
  /// The TofEvent timed out while waiting for more Readoutboards
  EventTimeOut           = 23u8,
  /// A RB misses Ch9 data
  NoChannel9             = 24u8,
  GoodNoCRCOrErrBitCheck = 39u8,
  /// The event status is good, but we did not 
  /// perform any CRC32 check
  GoodNoCRCCheck         = 40u8,
  /// The event is good, but we did not perform
  /// error checks
  GoodNoErrBitCheck      = 41u8,
  Perfect                = 42u8
}

// in case we have pybindings for this type, 
// expand it so that it can be used as keys
// in dictionaries
#[cfg(feature = "pybindings")]
#[pymethods]
impl EventStatus {

  #[getter]
  fn __hash__(&self) -> usize {
    (*self as u8) as usize
  } 
}


expand_and_test_enum!(EventStatus, test_eventstatus_repr);

//--------------------------------------------

/// A generic data type
///
/// Describe the purpose of the data. This
/// is the semantics behind it.
#[derive(Debug, Copy, Clone, PartialEq,FromRepr, AsRefStr, EnumIter, serde::Deserialize, serde::Serialize)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
#[repr(u8)]
pub enum DataType {
  Unknown            = 0u8,
  VoltageCalibration = 10u8,
  TimingCalibration  = 20u8,
  Noi                = 30u8,
  Physics            = 40u8,
  RBTriggerPeriodic  = 50u8,
  RBTriggerPoisson   = 60u8,
  MTBTriggerPoisson  = 70u8,
  // future extension for different trigger settings!
}

expand_and_test_enum!(DataType, test_datatype_repr);

//--------------------------------------------

