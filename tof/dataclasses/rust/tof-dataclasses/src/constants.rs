//! Global constants for tof operations
//!
//!
//!


/// Number of Channels on the readoutboards
pub const NCHN          : usize = 9; // even though this is well in range of u8, 
                                     // we need it to be u16 so it can be multiplied

/// Number of entries for each waveform (voltage and timing each)
pub const NWORDS        : usize = 1024;

/// The maximum number of supported readout boards
#[deprecated(since="0.2")]
pub const MAX_NBOARDS   : usize = 4;

/// The maximum number of detectable peaks in a waveform
pub const MAX_NUM_PEAKS : usize = 50;

/// Expected maximum trigger rate
///
/// TODO/FIXME 
///
/// Rationale: The code/threads sleeps at 
/// certain points, either to actively
/// wait for something (writing to registers)
/// or to ease resource consumption.
///
/// This might scale with the rate and lead
/// to missing events. If we set a maximum
/// rate here directly and adjust the sleep 
/// times accordingly, we do not get any 
/// surprises.
///
/// # Example:
/// 
/// If the `MAX_TRIGGER_RATE` is 1000, we 
/// can nowhere sleep longer than
/// 1 milli second.
///
pub const MAX_TRIGGER_RATE : usize = 200;

/// How long to wait for paddles packets for each event 
/// in microseconds. This does highly depend on the 
/// frequency with which the readoutboards are emitting
/// For now, lets use 30s. 
///
/// This will impact also the size of the caches 
/// (see below)
///
/// This might NOT be relevant when we run 
/// with the master trigger.
///
pub const EVENT_TIMEOUT : u128 = 30000000;


/// Limit the size of the internal paddle packet cache
/// - all packets abvoe this value will be dropped
pub const PADDLE_PACKET_CACHE_SIZE : usize = 20000;

/// Limit the size of the internanl event cache
///
/// This cache holds assembled events.
/// FIXME: This should be rate dependent
///
pub const EVENT_CACHE_SIZE : usize = 30000;

/// Limit the size of the evids the event builder
/// is currently waiting to get paddles for.
///
/// FIXME: (this shoudl be rate*event_timeout
pub const EVENT_BUILDER_EVID_CACHE_SIZE : usize = 10000;

/// Average number of paddle packets per event
///
/// This might be useful to calculate 
/// cache/memory sizes
///
pub const EXP_N_PADDLES_PER_EVENT : usize = 10;

