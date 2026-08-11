//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license
//!
//! Heartbeats are a specialized class of monitoring
//! observables. Instead of gathering "hardware" 
//! parameters, they gather properties of the running
//! threads. In our case, that is only implemented
//! for the TOF system. The beating heart are 
//! 3 main threads: 
//! * communication with the trigger
//! * builidng TOF evnets
//! * writing data to disk and sending it out
//!
//! For each of these threads, there is a dedicated
//! heartbeat.
//!
//!
pub mod data_sink_hb;
pub mod event_builder_hb;
pub mod master_trigger_hb;

pub use data_sink_hb::{
  DataSinkHB,
  DataSinkHBSeries};
pub use event_builder_hb::{
  EventBuilderHB,
  EventBuilderHBSeries};
pub use master_trigger_hb::{
  MasterTriggerHB,
  MasterTriggerHBSeries,
};
