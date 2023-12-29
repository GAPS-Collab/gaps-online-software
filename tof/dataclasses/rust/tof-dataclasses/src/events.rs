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
pub use master_trigger::MasterTriggerMapping;
pub use tof_event::{TofEvent,
                    TofEventHeader};
pub use tof_hit::TofHit;
pub use data_type::DataType;

#[allow(deprecated)]
pub use rb_eventmemoryview::RBEventMemoryView;
pub use rb_event::{
    RBEventHeader,
    RBEvent,
    RBMissingHit,
    EventStatus
};

// TODO what is this file?
