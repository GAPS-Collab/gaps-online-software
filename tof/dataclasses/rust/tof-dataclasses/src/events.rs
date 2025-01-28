//! Events  
//
//
#[cfg(feature = "pybindings")]
use pyo3::pyclass;
pub mod tof_event;
pub mod master_trigger;
pub mod rb_event;
#[allow(deprecated)]
pub mod rb_eventmemoryview;
pub mod data_type;
pub mod tof_hit;

pub use master_trigger::{
  MasterTriggerEvent,
  TriggerType,
};
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
    //RBMissingHit,
    RBWaveform,
};
  
cfg_if::cfg_if! {
  if #[cfg(feature = "random")]  {
    use crate::FromRandom;
    extern crate rand;
    use rand::Rng;
  }
}

use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
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
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this EventStatus"));
    write!(f, "<EventStatus: {}>", r)
  }
}

impl EventStatus {
  pub fn to_u8(&self) -> u8 {
    match self {
      EventStatus::Unknown => {
        return 0;
      }
      EventStatus::CRC32Wrong => {
        return 10;
      }
      EventStatus::TailWrong => {
        return 11;
      }
      EventStatus::ChannelIDWrong => {
        return 12;
      }
      EventStatus::CellSyncErrors => {
        return 13;
      }
      EventStatus::ChnSyncErrors => {
        return 14;
      }
      EventStatus::CellAndChnSyncErrors => {
        return 15;
      }
      EventStatus::AnyDataMangling => {
        return 16;
      }
      EventStatus::IncompleteReadout => {
        return 21;
      }
      EventStatus::IncompatibleData => {
        return 22;
      }
      EventStatus::EventTimeOut => {
        return 23;
      }
      EventStatus::NoChannel9 => {
        return 24;
      }
      EventStatus::GoodNoCRCOrErrBitCheck => {
        return 39;
      }
      EventStatus::GoodNoCRCCheck => {
        return 40;
      }
      EventStatus::GoodNoErrBitCheck => {
        return 41;
      }
      EventStatus::Perfect => {
        return 42;
      }
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
    let mut rng  = rand::thread_rng();
    let idx = rng.gen_range(0..choices.len());
    choices[idx]
  }
}

/// Get the trigger sources from trigger source byte
/// FIXME! (Does not return anything)
pub fn transcode_trigger_sources(trigger_sources : u16) -> Vec<TriggerType> {
  let mut t_types    = Vec::<TriggerType>::new();
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


#[test]
#[cfg(feature = "random")]
fn test_event_status() {
  for _ in 0..100 {
    let ev_stat    = EventStatus::from_random();
    let ev_stat_u8 = ev_stat.to_u8();
    let u8_ev_stat = EventStatus::from(ev_stat_u8);
    assert_eq!(ev_stat, u8_ev_stat);
  }
}

