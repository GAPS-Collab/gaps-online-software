pub const NCHN          : usize = 9; // even though this is well in range of u8, 
                        // we need it to be u16 so it can be multiplied
pub const NWORDS        : usize = 1024;
// the maximum number or readout boards
pub const MAX_NBOARDS   : usize = 4;

pub const MAX_NUM_PEAKS : usize = 50;

///! Limit the size of the internal paddle packet cache
/// - all packets abvoe this value will be dropped
pub const PADDLE_PACKET_CACHE_SIZE : usize = 20000;

///! Limit the size of the evids the event builder
///  is currently waiting to get paddles for
///  FIXME - this should maybe be rate dependent?
pub const EVENT_BUILDER_EVID_CACHE_SIZE : usize = 10000;
