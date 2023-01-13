pub const NCHN          : usize = 9; // even though this is well in range of u8, 
                        // we need it to be u16 so it can be multiplied
pub const NWORDS        : usize = 1024;
// the maximum number or readout boards
pub const MAX_NBOARDS   : usize = 4;

pub const MAX_NUM_PEAKS : usize = 50;

///! Expected maximum trigger rate
///
///  This impacts the cache sizes, the 
///  frequnecy we can poll the master 
///  trigger, etc.
///  Value in Hz
///
pub const MAX_TRIGGER_RATE : usize = 200;

///! How long to wait for paddles packets for each event 
///  in microseconds. This does highly depend on the 
///  frequency with which the readoutboards are emitting
///  For now, lets use 30s. 
///  This will impact also the size of the caches 
///  (see below)
pub const EVENT_TIMEOUT : u128 = 30000000;


///! Limit the size of the internal paddle packet cache
/// - all packets abvoe this value will be dropped
pub const PADDLE_PACKET_CACHE_SIZE : usize = 20000;

///! This should be rate dependent
pub const EVENT_CACHE_SIZE : usize = 30000;

///! Limit the size of the evids the event builder
///  is currently waiting to get paddles for
///  (this shoudl be rate*event_timeout
pub const EVENT_BUILDER_EVID_CACHE_SIZE : usize = 10000;

///! Average number of paddle packets per event
pub const EXP_N_PADDLES_PER_EVENT : usize = 10;

