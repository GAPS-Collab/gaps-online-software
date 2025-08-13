//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

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


use std::fmt;

#[cfg(feature="pybindings")]
use pyo3::prelude::*;

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

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
pub enum EventQuality {
  Unknown        =  0u8,
  Silver         = 10u8,
  Gold           = 20u8,
  Diamond        = 30u8,
  FourLeafClover = 40u8,
}

impl fmt::Display for EventQuality {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r : &str;
    match self {
      EventQuality::Unknown        => {r = "Unknown"},
      EventQuality::Silver         => {r = "Silver"},
      EventQuality::Gold           => {r = "Gold"},
      EventQuality::Diamond        => {r = "Diamond"},
      EventQuality::FourLeafClover => {r = "FourLeafClover"},
    }
    write!(f, "<EventQuality: {}>", r)
  }
}

impl From<u8> for EventQuality {
  fn from(value: u8) -> Self {
    match value {
      0u8  => EventQuality::Unknown,
      10u8 => EventQuality::Silver,
      20u8 => EventQuality::Gold,
      30u8 => EventQuality::Diamond,
      40u8 => EventQuality::FourLeafClover,
      _    => EventQuality::Unknown
    }
  }
}

impl FromRandom for EventQuality {
  fn from_random() -> Self {
    let choices = [
      Self::Unknown,
      Self::Silver,
      Self::Gold,
      Self::Diamond,
    ];
    let mut rng  = rand::rng();
    let idx = rng.random_range(0..choices.len());
    choices[idx]
  }
}

//--------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
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

impl fmt::Display for TriggerType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r : &str;
    match self {
      TriggerType::Unknown             => {r = "Unknown"},
      TriggerType::Any                 => {r = "Any"},
      TriggerType::Track               => {r = "Track"},
      TriggerType::TrackCentral        => {r = "TrackCentral"},
      TriggerType::Gaps1044            => {r = "Gaps1044"},
      TriggerType::Gaps                => {r = "Gaps"},
      TriggerType::Gaps633             => {r = "Gaps633"}, 
      TriggerType::Gaps422             => {r = "Gaps422"},
      TriggerType::Gaps211             => {r = "Gaps211"},
      TriggerType::TrackUmbCentral     => {r = "TrackUmbCentral"},
      TriggerType::UmbCube             => {r = "UmbCube"},
      TriggerType::UmbCubeZ            => {r = "UmbCubeZ"},
      TriggerType::UmbCorCube          => {r = "UmbCorCube"},
      TriggerType::CorCubeSide         => {r = "CorCubeSide"},
      TriggerType::Umb3Cube            => {r = "Umb3Cube"},
      TriggerType::Poisson             => {r = "Poisson"},
      TriggerType::Forced              => {r = "Forced"},
      TriggerType::FixedRate           => {r = "FixedRate"},
      TriggerType::ConfigurableTrigger => {r = "ConfigurableTrigger"},
    }
    write!(f, "<TriggerType: {}>", r)
  }
}

impl From<u8> for TriggerType {
  fn from(value: u8) -> Self {
    match value {
      0   => TriggerType::Unknown,
      100 => TriggerType::Poisson,
      101 => TriggerType::Forced,
      102 => TriggerType::FixedRate,
      1   => TriggerType::Any,
      2   => TriggerType::Track,
      3   => TriggerType::TrackCentral,
      4   => TriggerType::Gaps,
      5   => TriggerType::Gaps633,
      6   => TriggerType::Gaps422,
      7   => TriggerType::Gaps211,
      8   => TriggerType::TrackUmbCentral,
      9   => TriggerType::Gaps1044,
      21  => TriggerType::UmbCube,
      22  => TriggerType::UmbCubeZ,
      23  => TriggerType::UmbCorCube,
      24  => TriggerType::CorCubeSide,
      25  => TriggerType::Umb3Cube,
      200 => TriggerType::ConfigurableTrigger,
      _   => TriggerType::Unknown
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for TriggerType {
  
  fn from_random() -> Self {
    let choices = [
      TriggerType::Unknown,
      TriggerType::Poisson,
      TriggerType::Forced,
      TriggerType::FixedRate,
      TriggerType::Any,
      TriggerType::Track,
      TriggerType::TrackCentral,
      TriggerType::Gaps,
      TriggerType::Gaps633,
      TriggerType::Gaps422,
      TriggerType::Gaps211,
      TriggerType::TrackUmbCentral,
      TriggerType::Gaps1044,
      TriggerType::UmbCube,
      TriggerType::UmbCubeZ,
      TriggerType::UmbCorCube,
      TriggerType::CorCubeSide,
      TriggerType::Umb3Cube,
      TriggerType::ConfigurableTrigger,
    ];
    let mut rng  = rand::rng();
    let idx = rng.random_range(0..choices.len());
    choices[idx]
  }
}

//--------------------------------------------

/// LTB Thresholds as passed on by the MTB
/// [See also](https://gaps1.astro.ucla.edu/wiki/gaps/images/gaps/5/52/LTB_Data_Format.pdf)
#[derive(Debug, Copy, Clone, PartialEq)]
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

impl fmt::Display for LTBThreshold {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r : &str;
    match self {
      LTBThreshold::NoHit   => { r = "NoHit"},
      LTBThreshold::Hit     => { r = "Hit"},
      LTBThreshold::Beta    => { r = "Beta"},
      LTBThreshold::Veto    => { r = "Veto"},
      LTBThreshold::Unknown => { r = "Unknown"}
    }
    write!(f, "<LTBThreshold: {}>", r)
  }
}

impl From<u8> for LTBThreshold {
  fn from(value: u8) -> Self {
    match value {
      0 => LTBThreshold::NoHit,
      1 => LTBThreshold::Hit,
      2 => LTBThreshold::Beta,
      3 => LTBThreshold::Veto,
      _ => LTBThreshold::Unknown
    }
  }
}

#[cfg(feature = "random")]
impl FromRandom for LTBThreshold {
  
  fn from_random() -> Self {
    let choices = [
      LTBThreshold::NoHit,
      LTBThreshold::Hit,
      LTBThreshold::Beta,
      LTBThreshold::Veto,
      LTBThreshold::Unknown
    ];
    let mut rng  = rand::rng();
    let idx = rng.random_range(0..choices.len());
    choices[idx]
  }
}

//--------------------------------------------

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

