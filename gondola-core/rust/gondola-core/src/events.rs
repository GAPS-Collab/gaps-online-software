//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

pub mod tof_hit;
pub use tof_hit::TofHit;

pub mod rb_waveform;
pub use rb_waveform::RBWaveform;

pub mod rb_event_header;
pub use rb_event_header::RBEventHeader;

pub mod rb_event;
pub use rb_event::RBEvent;

pub mod tracker_hit;
pub use tracker_hit::TrackerHit;

use std::fmt;

#[cfg(feature="pybindings")]
use pyo3::pyclass;

#[cfg(feature="random")]
use crate::random::FromRandom;
#[cfg(feature="random")]
use rand::Rng;

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
pub fn strip_id(layer : u8, row :u8, module : u8, channel : u8) -> u32 {
  channel as u32 + (module as u32)*100 + (row as u32)*10000 + (layer as u32)*100000
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

impl fmt::Display for EventStatus {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = self.string_repr();
    write!(f, "<EventStatus: {}>", r)
  }
}

impl EventStatus {
  pub fn string_repr(&self) -> &str {
    match self {
      EventStatus::Unknown                => {return "Unknown"},
      EventStatus::CRC32Wrong             => {return "CRC32Wrong"},
      EventStatus::TailWrong              => {return "TailWrong"},
      EventStatus::ChannelIDWrong         => {return "ChannelIDWrong"},
      EventStatus::CellSyncErrors         => {return "CellSyncErrors"},
      EventStatus::ChnSyncErrors          => {return "ChnSyncErrors"},
      EventStatus::CellAndChnSyncErrors   => {return "CellAndChnSyncErrors"},
      EventStatus::AnyDataMangling        => {return "AnyDataMangling"},
      EventStatus::IncompleteReadout      => {return "IncompleteReadout"},
      EventStatus::IncompatibleData       => {return "IncompatibleData"},
      EventStatus::EventTimeOut           => {return "EventTimeOut"},
      EventStatus::NoChannel9             => {return "NoChannel9"},
      EventStatus::GoodNoCRCOrErrBitCheck => {return "GoodNoCRCOrErrBitCheck"},
      EventStatus::GoodNoCRCCheck         => {return "GoodNoCRCCheck"},
      EventStatus::GoodNoErrBitCheck      => {return "GoodNoErrBitCheck"},
      EventStatus::Perfect                => {return "Perfect"}
    }
  }
  //pub fn to_u8(&self) -> u8 {
  //  match self {
  //    EventStatus::Unknown => {
  //      return 0;
  //    }
  //    EventStatus::CRC32Wrong => {
  //      return 10;
  //    }
  //    EventStatus::TailWrong => {
  //      return 11;
  //    }
  //    EventStatus::ChannelIDWrong => {
  //      return 12;
  //    }
  //    EventStatus::CellSyncErrors => {
  //      return 13;
  //    }
  //    EventStatus::ChnSyncErrors => {
  //      return 14;
  //    }
  //    EventStatus::CellAndChnSyncErrors => {
  //      return 15;
  //    }
  //    EventStatus::AnyDataMangling => {
  //      return 16;
  //    }
  //    EventStatus::IncompleteReadout => {
  //      return 21;
  //    }
  //    EventStatus::IncompatibleData => {
  //      return 22;
  //    }
  //    EventStatus::EventTimeOut => {
  //      return 23;
  //    }
  //    EventStatus::NoChannel9 => {
  //      return 24;
  //    }
  //    EventStatus::GoodNoCRCOrErrBitCheck => {
  //      return 39;
  //    }
  //    EventStatus::GoodNoCRCCheck => {
  //      return 40;
  //    }
  //    EventStatus::GoodNoErrBitCheck => {
  //      return 41;
  //    }
  //    EventStatus::Perfect => {
  //      return 42;
  //    }
  //  }
  //}
}

impl From<u8> for EventStatus {
  fn from(value: u8) -> Self {
    match value {
      0  => EventStatus::Unknown,
      10 => EventStatus::CRC32Wrong,
      11 => EventStatus::TailWrong,
      12 => EventStatus::ChannelIDWrong,
      13 => EventStatus::CellSyncErrors,
      14 => EventStatus::ChnSyncErrors,
      15 => EventStatus::CellAndChnSyncErrors,
      16 => EventStatus::AnyDataMangling,
      21 => EventStatus::IncompleteReadout,
      22 => EventStatus::IncompatibleData,
      23 => EventStatus::EventTimeOut,
      24 => EventStatus::NoChannel9,
      39 => EventStatus::GoodNoCRCOrErrBitCheck,
      40 => EventStatus::GoodNoCRCCheck,
      41 => EventStatus::GoodNoErrBitCheck,
      42 => EventStatus::Perfect,
      _    => EventStatus::Unknown
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for EventStatus {
 
  fn from_random() -> Self {
    let choices = [
      EventStatus::Unknown,
      EventStatus::CRC32Wrong,
      EventStatus::TailWrong,
      EventStatus::ChannelIDWrong,
      EventStatus::CellSyncErrors,
      EventStatus::ChnSyncErrors,
      EventStatus::CellAndChnSyncErrors,
      EventStatus::AnyDataMangling,
      EventStatus::IncompleteReadout,
      EventStatus::IncompatibleData,
      EventStatus::EventTimeOut,
      EventStatus::NoChannel9,
      EventStatus::GoodNoCRCOrErrBitCheck,
      EventStatus::GoodNoCRCCheck,
      EventStatus::GoodNoErrBitCheck,
      EventStatus::Perfect,
    ];
    let mut rng  = rand::rng();
    let idx = rng.random_range(0..choices.len());
    choices[idx]
  }
}

//--------------------------------------------

/// A generic data type
///
/// Describe the purpose of the data. This
/// is the semantics behind it.
#[derive(Debug, Copy, Clone, PartialEq)]
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

impl fmt::Display for DataType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = self.string_repr();
    write!(f, "<DataType: {}>", r)
  }
}

impl DataType {
  pub fn string_repr(&self) -> &str {
    match self {
      DataType::Unknown            => {return "Unknown"},
      DataType::VoltageCalibration => {return "VoltageCalibration"},
      DataType::TimingCalibration  => {return "TimingCalibration"},
      DataType::Noi                => {return "Noi"},
      DataType::Physics            => {return "Physics"},
      DataType::RBTriggerPeriodic  => {return "RBTriggerPeriodic"},
      DataType::RBTriggerPoisson   => {return "RBTriggerPoisson"},
      DataType::MTBTriggerPoisson  => {return "MTBTriggerPoisson"},
    }
  }
}
impl From<u8> for DataType {
  fn from(value: u8) -> Self {
    match value {
      0u8  => DataType::Unknown,
      10u8 => DataType::VoltageCalibration,
      20u8 => DataType::TimingCalibration,
      30u8 => DataType::Noi,
      40u8 => DataType::Physics,
      50u8 => DataType::RBTriggerPeriodic,
      60u8 => DataType::RBTriggerPoisson,
      70u8 => DataType::MTBTriggerPoisson,
      _    => DataType::Unknown
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for DataType {
  
  fn from_random() -> Self {
    let choices = [
      DataType::Unknown,
      DataType::VoltageCalibration,
      DataType::TimingCalibration,
      DataType::Noi,
      DataType::Physics,
      DataType::RBTriggerPeriodic,
      DataType::RBTriggerPoisson,
      DataType::MTBTriggerPoisson
    ];
    let mut rng  = rand::rng();
    let idx = rng.random_range(0..choices.len());
    choices[idx]
  }
}

//--------------------------------------------

#[test]
fn test_data_type() {
  let mut type_codes = Vec::<u8>::new();
  type_codes.push(DataType::Unknown as u8); 
  type_codes.push(DataType::VoltageCalibration as u8); 
  type_codes.push(DataType::TimingCalibration as u8); 
  type_codes.push(DataType::Noi as u8); 
  type_codes.push(DataType::Physics as u8); 
  type_codes.push(DataType::MTBTriggerPoisson as u8); 
  type_codes.push(DataType::RBTriggerPeriodic as u8); 
  type_codes.push(DataType::RBTriggerPoisson as u8); 
  for tc in type_codes.iter() {
    assert_eq!(*tc,DataType::try_from(*tc).unwrap() as u8);
  }
}

#[test]
#[cfg(feature = "random")]
fn test_event_status() {
  for _ in 0..100 {
    let ev_stat    = EventStatus::from_random();
    let ev_stat_u8 = ev_stat as u8;
    let u8_ev_stat = EventStatus::from(ev_stat_u8);
    assert_eq!(ev_stat, u8_ev_stat);
  }
}

