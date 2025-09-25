// This file is part of gaps-online-software and published 
// under the GPLv3 license

pub mod rb_paddle_id;
pub mod algorithms;
pub mod detector_status;
pub use rb_paddle_id::RBPaddleID;
pub use detector_status::TofDetectorStatus;
pub mod config;
pub use config::*;
pub mod commands;
pub use commands::*;
pub mod settings;
pub use settings::*;
#[cfg(feature="database")]
pub mod analysis_engine;
#[cfg(feature="database")]
pub use analysis_engine::*;
pub mod cuts;
pub use cuts::*;
pub mod alerts;
pub use alerts::*;
pub mod tof_response;
pub use tof_response::*;
#[cfg(feature="tof-liftof")]
pub mod thread_control;
#[cfg(feature="tof-liftof")]
pub use thread_control::ThreadControl;
#[cfg(feature="tof-liftof")]
pub mod master_trigger;
pub use master_trigger::*;
#[cfg(feature="tof-liftof")]
pub use master_trigger::control::*;
#[cfg(feature="tof-liftof")]
pub use master_trigger::registers::*;
#[cfg(feature="tof-liftof")]
pub mod signal_handler;
#[cfg(feature="tof-liftof")]
pub use signal_handler::*;
pub mod analysis;
pub use analysis::*;

#[cfg(feature="pybindings")]
use pyo3::pyfunction;
/// Convert an int value to the board ID string.
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn to_board_id_string(rb_id: u32) -> String {
  format!("RB{:02}", rb_id)
}


