//! Global constants for TOF operations
//!
//! ISSUES:
//! * there might be constants defined elsewhere,
//!   also we are defining constants in .toml files
//!   now. There is an active issue #18
//!

/// Speed of light in the scintillator paddles
/// (divine number from the TOF team)
/// This value is in cm/ns
pub const C_LIGHT_PADDLE : f32 = 15.4; 

/// Number of AVAILABLE slots for LocalTriggerBoards
pub const N_LTBS : usize = 25;

/// Number of AVAILABLE channels per each LocalTriggerBoard
pub const N_CHN_PER_LTB : usize = 16;

/// Number of Channels on the readoutboards
pub const NCHN          : usize = 9;  

/// Number of entries for each waveform (voltage and timing each)
pub const NWORDS        : usize = 1024;

/// Masks for 32 bits commands (byte packets)
///
pub const MASK_CMD_8BIT  : u32 = 0x000000FF;
pub const MASK_CMD_16BIT : u32 = 0x0000FFFF;
pub const MASK_CMD_24BIT : u32 = 0x00FFFFFF;
pub const MASK_CMD_32BIT : u32 = 0xFFFFFFFF;
/// Padding for 32 bits commands (byte packets)
///
pub const PAD_CMD_32BIT  : u32 = 0x00000000;
