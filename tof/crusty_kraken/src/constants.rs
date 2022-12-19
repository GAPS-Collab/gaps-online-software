pub const NCHN          : usize = 9; // even though this is well in range of u8, 
                        // we need it to be u16 so it can be multiplied
pub const NWORDS        : usize = 1024;
// the maximum number or readout boards
pub const MAX_NBOARDS   : usize = 4;

pub const MAX_NUM_PEAKS : usize = 50;

///! Global event cache size for reduced (assembled)
///  tof events. If we collect 
///  more events than these, they will be dropped
pub const GLOBAL_EVENT_CACHE_SIZE : usize = 50;
