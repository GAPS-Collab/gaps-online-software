//! The following file is part of gaps-online-software and published 
//! under the GPLv3 license
//! Global constants for TOF operations
//!
//! ISSUES:
//! * there might be constants defined elsewhere,
//!   also we are defining constants in .toml files
//!   now. There is an active issue #18
//!

/// The TimeStamp format for Human readable timestamps
pub static HUMAN_TIMESTAMP_FORMAT : &str = "%y%m%d_%H%M%S%Z"; 

/// Speed of light in the scintillator paddles
/// (divine number from the TOF team)
/// This value is in cm/ns
pub const C_LIGHT_PADDLE : f32 = 15.4; 

/// Speed of light in the harting cables
/// (divine number from the TOF team)
/// This value is in cm/ns
pub const C_LIGHT_CABLE : f32 = 24.6;

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

/// Si(Li) wafer detector radius, with guardring and all
pub const SILI_RADIUS : f32 = 5.0;

// These are just for fun 

/// Make a nice ASCII logo for the liftof of flight code
pub const LIFTOF_LOGO_SHOW  : &str  = "
                                  ___                         ___           ___     
                                 /\\__\\                       /\\  \\         /\\__\\    
                    ___         /:/ _/_         ___         /::\\  \\       /:/ _/_   
                   /\\__\\       /:/ /\\__\\       /\\__\\       /:/\\:\\  \\     /:/ /\\__\\  
    ___     ___   /:/__/      /:/ /:/  /      /:/  /      /:/  \\:\\  \\   /:/ /:/  /  
   /\\  \\   /\\__\\ /::\\  \\     /:/_/:/  /      /:/__/      /:/__/ \\:\\__\\ /:/_/:/  /   
   \\:\\  \\ /:/  / \\/\\:\\  \\__  \\:\\/:/  /      /::\\  \\      \\:\\  \\ /:/  / \\:\\/:/  /    
    \\:\\  /:/  /   ~~\\:\\/\\__\\  \\::/__/      /:/\\:\\  \\      \\:\\  /:/  /   \\::/__/     
     \\:\\/:/  /       \\::/  /   \\:\\  \\      \\/__\\:\\  \\      \\:\\/:/  /     \\:\\  \\     
      \\::/  /        /:/  /     \\:\\__\\          \\:\\__\\      \\::/  /       \\:\\__\\    
       \\/__/         \\/__/       \\/__/           \\/__/       \\/__/         \\/__/    

          (LIFTOF - liftof is for tof, Version 0.11.x 'PAKII', Aug 2025)
          >> with undying support from the Hawaiian islands \u{1f30a}\u{1f308}\u{1f965}\u{1f334}

          * Documentation
          ==> GitHub   https://github.com/GAPS-Collab/gaps-online-software/tree/LELEWAA-0.11
          ==> API docs https://gaps-collab.github.io/gaps-online-software/

  ";




