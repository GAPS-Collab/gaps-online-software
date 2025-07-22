//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

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


pub mod tof_hit;
pub use tof_hit::TofHit;

pub mod rb_waveform;
pub use rb_waveform::RBWaveform;

pub mod rb_event_header;
pub use rb_event_header::RBEventHeader;

pub mod tracker_hit;
pub use tracker_hit::TrackerHit;

