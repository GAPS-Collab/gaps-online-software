//! Events  
//
//

pub mod tof_event;
pub mod master_trigger;
pub mod rb_event;
#[allow(deprecated)]
pub mod rb_eventmemoryview;
pub mod data_type;
pub mod tof_hit;

pub use master_trigger::MasterTriggerEvent;
pub use tof_event::{
    TofEvent,
    TofEventHeader,
    TofEventSummary
};
pub use tof_hit::TofHit;
pub use data_type::DataType;

#[allow(deprecated)]
pub use rb_eventmemoryview::RBEventMemoryView;
pub use rb_event::{
    RBEventHeader,
    RBEvent,
    RBMissingHit,
    RBWaveform,
};

use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum EventStatus {
  Unknown            = 0u8,
  CRC32Wrong         = 10u8,
  TailWrong          = 11u8,
  ChannelIDWrong     = 12u8,
  /// one of the channels cells CellSyncError bits 
  /// has been set (RB)
  CellSyncErrors     = 13u8,
  /// one of the channels ChannelSyncError bits 
  /// has been set (RB)
  ChnSyncErrors      = 14u8,
  IncompleteReadout  = 21u8,
  /// This can be used if there is a version
  /// missmatch and we have to hack something
  IncompatibleData   = 22u8,
  Perfect            = 42u8
}

impl fmt::Display for EventStatus {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this EventStatus"));
    write!(f, "<EventStatus: {}>", r)
  }
}

impl From<u8> for EventStatus {
  fn from(value: u8) -> Self {
    match value {
      0u8  => EventStatus::Unknown,
      10u8 => EventStatus::CRC32Wrong,
      11u8 => EventStatus::TailWrong,
      12u8 => EventStatus::ChannelIDWrong,
      13u8 => EventStatus::CellSyncErrors,
      14u8 => EventStatus::ChnSyncErrors,
      21u8 => EventStatus::IncompleteReadout,
      22u8 => EventStatus::IncompatibleData,
      42u8 => EventStatus::Perfect,
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
      EventStatus::IncompleteReadout,
      EventStatus::IncompatibleData,
      EventStatus::Perfect,
    ];
    let mut rng  = rand::thread_rng();
    let idx = rng.gen_range(0..choices.len());
    choices[idx]
  }
}

