//! Events  
//
//

pub mod tof_event;
pub mod master_trigger;
pub mod rb_event;
pub mod data_type;
pub mod data_format;

pub use master_trigger::MasterTriggerEvent;
pub use master_trigger::MasterTriggerMapping;
pub use tof_event::{TofEvent,
                    TofEventHeader};
pub use data_type::DataType;
pub use data_format::DataFormat;
pub use rb_event::{RBEventMemoryView,
                   RBEventHeader,
                   RBEvent,
                   RBMissingHit};

// TODO what is this file?