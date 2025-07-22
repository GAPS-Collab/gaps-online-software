//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license

pub mod pa_moni_data;
pub use pa_moni_data::PAMoniData;
pub mod pb_moni_data;
pub use pb_moni_data::PBMoniData;
pub mod mtb_moni_data;
pub use mtb_moni_data::MtbMoniData;
pub mod ltb_moni_data;
pub use ltb_moni_data::LTBMoniData;

pub mod heartbeats;
pub use heartbeats::{
  DataSinkHB,
  MasterTriggerHB,
  EventBuilderHB,
};

/// Monitoring data shall share the same kind 
/// of interface. 
pub trait MoniData {
  /// Monitoring data is always tied to a specific
  /// board. This might not be its own board, but 
  /// maybe the RB the data was gathered from
  /// This is an unique identifier for the 
  /// monitoring data
  fn get_board_id(&self) -> u8;
  
  /// Access the (data) members by name 
  fn get(&self, varname : &str) -> Option<f32>;

  /// A list of the variables in this MoniData
  fn keys() -> Vec<&'static str>;
}

