//! MasterTriggerBoard registers
//!
//! This should be the same as the reference
//! doucmentation on the [UCLA gitlab server](https://gitlab.com/ucla-gaps-tof/firmware/-/blob/master/regmap/mt_address_table.org?ref_type=heads)
//! 
//! <div class="warning"> This currently does NOT sync itself with the reposity, so we have to take extra care that these numbers are correct!.</div>
//!
//! All registers are 32bit

use std::error::Error;
use std::fmt;
use crate::io::ipbus::IPBus;

/// The prescale values are defined by a single u32
/// This number represents 0 for trigger off and 
/// 1.0 for 2**32 - 1 (which is u32::MAX)
///
/// The range for the prescale value is [0,1.0]. 
/// If the given value is outside of the interval
/// boundaries, it will be converted to the next 
/// interval limit
pub fn prescale_to_u32(mut prescale : f32) -> u32 {
  if prescale > 1.0 {
    warn!("Prescale value > 1.0 will be converted to 1.0!");
    prescale = 1.0
  }
  if prescale < 0.0 {
    prescale = 0.0
  }
  // converion
  ((u32::MAX as f32) * prescale).floor() as u32
}


/// A single 32bit register on the MTB with an 
/// associated mask to mask parts of ig
pub struct MTBRegister<'a> {
  /// the address of the register on the MTB
  /// (not Addr8)
  pub addr  : u32,
  /// Some registers on the MTB share functionality, which 
  /// we are splitting in different MTBRegisters. So even
  /// if two MTBRegisters share the same address, they
  /// might have a different mask to achieve different 
  /// things and will thus have a different name.
  pub mask  : u32,
  /// Description about what is achieved when setting 
  /// this register?
  pub descr : &'a str,
  /// Is this register shared with other values? In that 
  /// case we need to do a rmw operation instead of 
  /// writing
  pub rmw   : bool,
  /// This register is only read-only (e.g. providing 
  /// a monitoring value
  pub ro    : bool,
  /// This register is "pulse" type. So it can be 
  /// asserted, but there is no point in reading it 
  /// out, since it will reset itself.
  pub pulse : bool,
}

impl MTBRegister<'_> {

  /// Set the register to desired value
  pub fn set(&self, bus : &mut IPBus, value : u32) 
    -> Result<(), Box<dyn Error>> {
    //println!("Settting {}", self);
    if self.rmw {
      self.rmw(bus, value)?;
    }
    else {
      self.write(bus, value)?;
    }
    // this can be used for debugging
    // (print back the value)
    //let rv = self.read_all(bus)?;
    //println!("Register reads {:x} {} {:x} after write ops!", self.addr, self.descr, rv);
    Ok(())
  }

  /// Get the value this register is set to
  pub fn get(&self, bus : &mut IPBus)
    -> Result<u32, Box<dyn Error>> {
    //FIXME - error type
    //if self.pulse {
    //  return(Err)
    //}
    let rv = self.read(bus)?;
    //if self.addr != 0x13 && self.addr != 0x11 {
    //  //
    //  //println!("Register reads {:x} {} {:x}!", self.addr, self.descr, rv);
    //}
    Ok(rv)
  }


  // FIXME - basically whenever we have a amsk 
  // != u32::MAX we need rmw
  fn write(&self, bus : &mut IPBus, value : u32)
    -> Result<(), Box<dyn Error>> {
      let masked_value = self.mask & value;
      //println!("Writing ... {:x}", masked_value);
      Ok(bus.write(self.addr, masked_value)?)
  }
  
  fn read_all(&self, bus : &mut IPBus)
    -> Result<u32, Box<dyn Error>> {
    let value = bus.read(self.addr)?;
    Ok(value)
  }

  fn read(&self, bus : &mut IPBus)
    -> Result<u32, Box<dyn Error>> {
    let mut value = bus.read(self.addr)?;
    value = value & self.mask; 
    //println!("Read all .. {:?}", self.read_all(bus));
    if self.mask > 255 {
      //println!("...shifting by {}", self.mask.trailing_zeros());
      value = value >> self.mask.trailing_zeros();
    }
    Ok(value)
  }
  
  /// Read-modify-write
  fn rmw(&self, bus : &mut IPBus, value : u32)
    -> Result<(), Box<dyn Error>> {
    let mut data = self.read_all(bus)?;
    //println!("step 1 ..{}",data);
    // leave everything else the same, but zero out
    // the masked part
    data         = data & !self.mask;
    //println!("step 2 ..{}",data);
    // reset the masked part and write again
    //println!("step 3 ..{}", value << self.mask.trailing_zeros());
    data         = data | (value << self.mask.trailing_zeros()); 
    //println!("step 4 ..{}",data);
    Ok(bus.write(self.addr, data)?)
  }
}

impl fmt::Display for MTBRegister<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut rmw_str  = "R/W";
    if self.rmw {
      rmw_str = "R/RMW"
    }
    if self.ro {
      rmw_str = "RO"
    }
    if self.pulse {
      rmw_str = "pulse"
    }
    let mut repr = format!("<MTBRegister [{}]:", rmw_str);
    repr += &(format!("\n  address : {:x}" , self.addr));
    repr += &(format!("\n  mask    : {:x}" , self.mask));
    repr += &(format!("\n  descr   : {}>", self.descr));
    write!(f, "{}", repr)
  }
}

/////////////////////////////////////////////////////////////////////

/////////////////////////////////////////////////////////////////////

// Module MT adr = 0x0
//
// Implements various control and monitoring functions of the DRS Logic

///Prescale value for the GAPS trigger. 0 == 0% (off), 2**32-1 == 100%
/// GAPS_TRIG_PRESCALE 0x248 0x920 \[31:0\] rw 0xffffffff
pub const GAPS_TRIG_PRESCALE : MTBRegister<'static> = MTBRegister {
  addr    : 0x248,
  mask    : 0xffffffff,
  descr   : "Prescale value for the GAPS trigger. 0 == 0% (off), 2**32-1 == 100%",
  rmw     : true,
  ro      : false,
  pulse   : false,
};


/// Force a trigger (has to be previously set)
/// FORCE_TRIGGER   0x8     0x20    0   w   Pulse   Write 1 to generate a trigger
pub const FORCE_TRIGGER : MTBRegister<'static> = MTBRegister {
  addr  : 0x8,
  mask  : 0xffffffff,
  descr : "Force the readout and issue a one-time trigger",
  rmw   : false,
  ro    : false,
  pulse : true,
};

///// TIU_USE_AUX_LINK 	0xe 	0x38 	1 	rw 	0x0 	1 to use J11; 0 to use J3
/// Check if the TIU link is bad
/// TIU_BAD 	0xf 	0x3c 	0 	r 		1 means that the tiu link is not working
pub const TIU_BAD : MTBRegister<'static> = MTBRegister {
  addr  : 0xf,
  mask  : 0x00000001,
  descr : "Check if the TIU link is bad",
  rmw   : false,
  ro    : true,
  pulse : false
};

/// Read out how many clock cycles the NTB has been busy because of the TIU busy signal
/// TIU_EMU_BUSY_CNT 	0xe 	0x38 	\[31:14\] 	rw 	0xC350 	Number of 10 ns clock cyles that the emulator will remain busy
pub const TIU_EMU_BUSY_CNT : MTBRegister<'static> = MTBRegister {
  addr  : 0xe,
  mask  : 0xffffc000,
  descr : "Read out emulated TIU busy time (in 10ns clock cycles)",
  rmw   : true,
  ro    : false,
  pulse : false
};

/// Enable cyclic trigger
/// TRIG_CYCLIC_EN 0x240 0x900 0 rw 0x0
pub const TRIG_CYCLIC_EN : MTBRegister<'static> = MTBRegister {
  addr  : 0x240,
  mask  : 0x00000001,
  descr : "Enable the use of a cyclic trigger",
  rmw   : true, 
  ro    : false,
  pulse : false
};

/// Set cyclic trigger interval (in # clock cycles, 1 clock cycle ~ 10 ns)
/// TRIG_CYCLIC_INTERVAL 0x241 0x904 \[31:0\] rw 0x0
pub const TRIG_CYCLIC_INTERVAL : MTBRegister<'static> = MTBRegister {
  addr   : 0x241,
  mask   : 0xffffffff,
  descr  : "Set the cyclic trigger interval in # clock cycles (int only) --> ie if desire trigger every 20 nsec, set interval = 2",
  rmw    : true,
  ro     : false, 
  pulse  : false
};

/// Toggle on/off LTBs 0-9
/// DSI 0 RX Link Enable 0x242 0x908 \[9:0\] rw 0x3FF
pub const LT_LINK_EN0 : MTBRegister<'static> = MTBRegister {
  addr    : 0x242,
  mask    : 0x000003ff,
  descr   : "Enable DSI link for LTBs 0-9",
  rmw     : true,
  ro      : false,
  pulse   : false
};
/// Toggle on/off LTBs 10-19
/// DSI 1 RX Link Enable 0x243 0x90c \[9:0\] rw 0x3FF
pub const LT_LINK_EN1 : MTBRegister<'static> = MTBRegister {
  addr    : 0x243,
  mask    : 0x000003ff,
  descr   : "Enable DSI link for LTBs 10-19",
  rmw     : true,
  ro      : false,
  pulse   : false
};
/// Toggle on/off LTBs 20-29
/// DSI 2 RX Link Enable 0x244 0x910 \[9:0\] rw 0x3FF
pub const LT_LINK_EN2 : MTBRegister<'static> = MTBRegister {
  addr    : 0x244,
  mask    : 0x000003ff,
  descr   : "Enable DSI link for LTBs 20-29",
  rmw     : true,
  ro      : false,
  pulse   : false
};
/// Toggle on/off LTBs 30-39
/// DSI 3 RX Link Enable 0x245 0x914 \[9:0\] rw 0x3FF
pub const LT_LINK_EN3 : MTBRegister<'static> = MTBRegister {
  addr    : 0x245,
  mask    : 0x000003ff,
  descr   : "Enable DSI link for LTBs 30-39",
  rmw     : true,
  ro      : false,
  pulse   : false
};
/// Toggle on/off LTBs 40-49
/// DSI 4 RX Link Enable 0x246 0x918 \[9:0\] rw 0x3FF
pub const LT_LINK_EN4 : MTBRegister<'static> = MTBRegister {
  addr    : 0x246,
  mask    : 0x000003ff,
  descr   : "Enable DSI link for LTBs 40-49",
  rmw     : true,
  ro      : false,
  pulse   : false
};
///Toggle on/off LTB Automasking
/// LT_LINK_AUTOMASK 0x247 0x91c 0 rw 0x1
/// 1 to enable automatic LT link masking
pub const LT_LINK_AUTOMASK : MTBRegister<'static> = MTBRegister {
  addr    : 0x247,
  mask    : 0x00000001,
  descr   : "Enable LT Automasking -> 1 to enable LTB link masking",
  rmw     : true,
  ro      : false,
  pulse   : false,
};

/// Set/Unset the TIU emulation mode
/// TIU_EMULATION_MODE  0xe     0x38    0   rw  0x0     1 to emulate the TIU
pub const TIU_EMULATION_MODE : MTBRegister<'static> = MTBRegister {
  addr  : 0xe,
  mask  : 0x00000001,
  descr : "Set/Unset the TIU emulation mode",
  rmw   : true,
  ro    : false,
  pulse : false
};

/// The length of the tiu busy signal 
/// TIU_BUSY_LENGTH 	0x11f 	0x47c 	\[31:0\] 	r 		Length in 10ns cycles of the last TIU busy flag
pub const TIU_BUSY_LENGTH : MTBRegister<'static> = MTBRegister {
  addr  : 0x11f,
  mask  : 0xffffffff,
  descr : "The length of the tiu busy signal in clock cycles [10ns/cycle]",
  rmw   : false,
  ro    : true,
  pulse : false
};

/// Read out the whole register 0xf at once and then 
pub const TIU_LT_AND_RB_MULT : MTBRegister<'static> = MTBRegister {
  addr  : 0xf,
  mask  : 0x3fff,
  descr : "Aggregated TIU, LT and RB general information",
  rmw   : false,
  ro    : true,
  pulse : false
};

/// Check if the TIU BUSY is stuck
/// TIU_BUSY_STUCK 	0xf 	0x3c 	1 	r 		1 means the TIU has been stuck high for a long time
pub const TIU_BUSY_STUCK : MTBRegister<'static> = MTBRegister {
  addr  : 0xf,
  mask  : 0x2,
  descr : "Tiu busy stuck high",
  rmw   : false,
  ro    : true,
  pulse : false
};

/// Set/unset the BUSY_INGORE 
/// TIU_BUSY_IGNORE 	0xf 	0x3c 	2 	rw 	0x0 	1 means the the MTB should ignore the TIU busy flag (e.g. because it is stuck)
pub const TIU_BUSY_IGNORE : MTBRegister<'static> = MTBRegister {
  addr  : 0xf,
  mask  : 0x4,
  descr : "Ignore the tiu busy signal",
  rmw   : true,
  ro    : false,
  pulse : false,
};


/// The global event id
/// EVENT_CNT   0xd     0x34    \[31:0\]  r       Event Counter
pub const EVENT_CNT : MTBRegister<'static> = MTBRegister {
  addr  : 0xd,
  mask  : 0xffffffff,
  descr : "Global event counter",
  rmw   : false,
  ro    : true,
  pulse : false
};

/// Reset the global event id (excecute with caution)
/// EVENT_CNT_RESET     0xc     0x30    0   w   Pulse   Write 1 to reset the event counter
pub const EVENT_CNT_RESET : MTBRegister<'static> = MTBRegister {
  addr  : 0xc,
  mask  : 0x1,
  descr : "Reset the event ID",
  rmw   : false,
  ro    : false,
  pulse : true
};

/// The RB integration window determines how long RBs should be read out 
/// after the trigger, basically the trigger window + x
/// RB_INTEGRATION_WINDOW   0xf     0x3c    \[12:8\]  rw  0x1     Number of 100MHz clock cycles to integrate the LTB hits to determine which RBs to read out.
pub const RB_INTEGRATION_WINDOW : MTBRegister<'static> = MTBRegister {
  addr  : 0xf,
  mask  : 0x00000f00,
  descr : "Determine how long RBs should be read out after the trigger. Default 1.",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// This setting is basically the 'trace suppression mode'. When asserted, only RBs for 
/// fired LTBs in the trigger window + RB_INTEGRATION_WINDOW will be read out.
/// RB_READ_ALL_CHANNELS    0xf     0x3c    13  rw  0x1     Set to 1 to read all channels from RB for any trigger
pub const RB_READ_ALL_CHANNELS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf,
  mask  : 0x00002000,
  descr : "Enable/Disable trace suppression mode. 1 = Enable",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// Set Readoutboard BUSY behaviour
/// RB_BLOCK_IF_BUSY_31_TO_0 	0x24a 	0x928 	\[31:0\] 	rw 	0x0 	Bitmask to specify if a readout board is BUSY then do not trigger (RB slots 31:0)
pub const RB_BLOCK_IF_BUSY_31_TO_0 : MTBRegister<'static> = MTBRegister {
  addr  : 0x24a,
  mask  : 0xffffffff,
  descr : "Define if triggers should suppressed if RBs are busy. 1 bit per board",
  rmw   : false,
  ro    : false,
  pulse : false,
};

/// Set Readoutboard BUSY behaviour
/// RB_BLOCK_IF_BUSY_49_TO_32 	0x24b 	0x92c 	\[17:0\] 	rw 	0x0 	Bitmask to specify if a readout board is BUSY then do not trigger (RB slots 49:32)
pub const RB_BLOCK_IF_BUSY_49_TO_32 : MTBRegister<'static> = MTBRegister {
  addr  : 0x24b,
  mask  : 0x0003ffff,
  descr : "Define if triggers should suppressed if RBs are busy. 1 bit per board",
  rmw   : false,
  ro    : false,
  pulse : false,
};


// MT.EVENT_QUEUE
// DAQ Buffer

/// Reset the DAQ buffer (this does NOT reset the event ID)
/// RESET   0x10    0x40    0   w   Pulse   DAQ Buffer Reset
pub const EVQ_RESET : MTBRegister<'static> = MTBRegister {
  addr  : 0x10,
  mask  : 0x00000001,
  descr : "Reset the DAQ buffer event queue! Will not reset event ID",
  rmw   : false,
  ro    : false,
  pulse : true,
};

/// The DAQ buffer is full 
/// FULL 	0x12 	0x48 	0 	r 		DAQ Buffer Full
pub const EVQ_FULL : MTBRegister<'static> = MTBRegister {
  addr  : 0x12,
  mask  : 0x1,
  descr : "Read FULL bit of the MT.EVENT_QUEUE",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// The DAQ buffer is empty
/// FULL 	0x12 	0x48 	1 	r 		DAQ Buffer Full
pub const EVQ_EMPTY : MTBRegister<'static> = MTBRegister {
  addr  : 0x12,
  mask  : 0x2,
  descr : "Read EMPTY bit of the MT.EVENT_QUEUE",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// DAQ data payload. An event will be broken down into 
/// multiple u32, which wlll be all accessible through
/// this register. The EVQ_SIZE register tells how many
/// times this register hast to be read (the actual value
/// is twice this number, since internally the MTB operates
/// on 16bit registers)
/// DATA    0x11    0x44    \[31:0\]  r       DAQ Read Data
pub const EVQ_DATA : MTBRegister<'static> = MTBRegister {
  addr  : 0x11,
  mask  : 0xffffffff,
  descr : "DAQ data payload field",
  rmw   : false,
  ro    : true,
  pulse : false
};

/// DAQ data queue SIZE. This stores twice the number of 
/// times the EVQ_DATA register has to be read out to 
/// obtain a complete event
/// SIZE    0x13    0x4c    \[31:16\]     r       DAQ Buffer Head Event Size
pub const EVQ_SIZE : MTBRegister<'static> = MTBRegister {
  addr   : 0x13,
  mask   : 0xffff0000,
  descr  : "DAQ buffer data queue size in 16bit words",
  rmw    : false,
  ro     : true,
  pulse  : false,
};

/// DAQ buffer size (in events)
/// NUM_EVENTS 	0x13 	0x4c 	\[13:0\] 	r 		DAQ Buffer Number of Event
pub const EVQ_NUM_EVENTS : MTBRegister<'static> = MTBRegister {
  addr  : 0x13,
  mask  : 0x0003fff,
  descr : "Number of events in DAQ buffer event queue",
  rmw   : false,
  ro    : true,
  pulse : false,
};

//FULL      0x12    0x48    0   r       DAQ Buffer Full
//EMPTY     0x12    0x48    1   r       DAQ Buffer Empty

/// Any 2 paddle combination. Can be used with a prescale
/// factor. If the prescale is 0, then it will be disabled
/// ANY_TRIG_PRESCALE     0x40    0x100   \[31:0\]  rw  0x0     Prescale value for the ANY trigger. 0 == 0% (off), 2**32-1 == 100%
pub const ANY_TRIG_PRESCALE : MTBRegister<'static> = MTBRegister {
  addr  : 0x40,
  mask  : 0xffffffff,
  descr : "Set the any trigger with a prescale factor. Prescale of 0 means disabled",
  rmw   : false,
  ro    : false,
  pulse : false
};

/// More strict condition, requiring a track pattern. Can 
/// be used with a prescale factor.
/// If the prescale is 0, then it will be disabled
/// TRACK_TRIGGER_PRESCALE    0x41    0x104   \[31:0\]  rw  0x0     Prescale value for the Inner + Outer Track Trigger. 0 == 0% (off), 2**32-1 == 100%
pub const TRACK_TRIG_PRESCALE : MTBRegister<'static> = MTBRegister {
  addr  : 0x41,
  mask  : 0xffffffff,
  descr : "Set the track trigger with a prescale factor. Prescale of 0 means disabled",
  rmw   : false,
  ro    : false,
  pulse : false
};

/// The central track trigger requires hits in the Umbrella and upper cube.
/// Can be used with a prescale factor.
/// If the prescale is 0, then it will be disabled
/// TRACK_CENTRAL_PRESCALE    0x42    0x108   \[31:0\]  rw  0x0     Prescale value for the Umbrella + Cube Top Track Trigger. 0 == 0% (off), 2**32-1 == 100%
pub const TRACK_CENTRAL_PRESCALE : MTBRegister<'static> = MTBRegister {
  addr  : 0x42,
  mask  : 0xffffffff,
  descr : "Set the central track trigger with a prescale factor. Prescale of 0 means disabled",
  rmw   : false,
  ro    : false,
  pulse : false
};

/// Prescale factor for the CENTRAL UMBRELLA TRACK trigger
/// TRACK_UMB_CENTRAL_PRESCALE 	0x249 	0x924 	\[31:0\] 	rw 	0x0 	Prescale value for the Umbrella Center + Cube Top Track Trigger. 0 == 0% (off), 2**32-1 == 100%
pub const TRACK_UMB_CENTRAL_PRESCALE : MTBRegister<'static> = MTBRegister {
  addr  : 0x249,
  mask  : 0xffffffff,
  descr : "Set the umbrella central track trigger with a prescale factor. Prescale of 0 means disabled",
  rmw   : false,
  ro    : false,
  pulse : false
};

//Implements various control and monitoring functions of the DRS Logic

/// Total inner tof (cube) threshold (nhits)
///
/// <div class="warning"> This setting is shared between the triggers and can impact performance of the pre-implmented triggers!!</div>
///
/// INNER_TOF_THRESH    0x14    0x50    \[7:0\]   rw  0x3     Inner TOF hit threshold
pub const INNER_TOF_THRESH : MTBRegister<'static> = MTBRegister {
  addr  : 0x14,
  mask  : 0x000000ff,
  descr : "Set the nhit threshold for the inner TOF (cube)",
  rmw   : true,
  ro    : false,
  pulse : false
};


/// Total outer TOF (cortina + umbrella) threshhold (nhits)
///
/// <div class="warning"> This setting is shared between the triggers and can impact performance of the pre-implmented triggers!!</div>
///
/// OUTER_TOF_THRESH    0x14    0x50    \[15:8\]  rw  0x3     Outer TOF hit threshold
pub const OUTER_TOF_THRESH : MTBRegister<'static> = MTBRegister {
  addr  : 0x14,
  mask  : 0x0000ff00,
  descr : "Set the nhit threshold for the outer TOF (cube)",
  rmw   : true,
  ro    : false,
  pulse : false
};


/// Total TOF (cube + cortina + umbrella) threshhold (nhits)
///
/// <div class="warning"> This setting is shared between the triggers and can impact performance of the pre-implmented triggers!!</div>
///
/// TOTAL_TOF_THRESH    0x14    0x50    \[23:16\]     rw  0x8     Total TOF hit threshold
pub const TOTAL_TOF_THRESH : MTBRegister<'static> = MTBRegister {
  addr  : 0x14,
  mask  : 0x00ff0000,
  descr : "Set the nhit threshold for the entire TOF (nhit)",
  rmw   : true,
  ro    : false,
  pulse : false
};

/// Enable the Gaps (antiparticle) trigger
/// GAPS_TRIGGER_EN     0x14    0x50    24  rw  0x0     Enable the gaps trigger.
pub const GAPS_TRIGGER_EN : MTBRegister<'static> = MTBRegister {
  addr  : 0x14,
  mask  : 0x01000000,
  descr : "Set the GAPS antiparticle trigger (on/off)",
  rmw   : true,
  ro    : false,
  pulse : false
};


/// Require beta condition for the Gaps (antiparticle) trigger
/// REQUIRE_BETA    0x14    0x50    25  rw  0x1     Require beta in the gaps trigger
pub const REQUIRE_BETA : MTBRegister<'static> = MTBRegister {
  addr  : 0x14,
  mask  : 0x02000000,
  descr : "Enable beta condition for the Gaps (antiparticle) trigger",
  rmw   : true,
  ro    : false,
  pulse : false
};

/// Enable the configurable trigger (has to be enable for any fine-grained threshhold
/// setting to come into effect)
/// CONFIGURABLE_TRIGGER_EN     0x14    0x50    31  rw  0x0     Enable the configurable trigger
pub const CONFIGURABLE_TRIGGER_EN : MTBRegister<'static> = MTBRegister {
  addr  : 0x14,
  mask  : 0x10000000,
  descr : "Enable the configurable trigger. This allows to use the individual thresholds",
  rmw   : true,
  ro    : false,
  pulse : false,
};


/// Set the (nhit) threshold for the cube side panesl
///
/// <div class="warning"> Requires the configurable trigger to be enabled!</div>
///
/// CUBE_SIDE_THRESH    0x15    0x54    \[7:0\]   rw  0x0     Threshold for the hit bitmap. Threshold must be > this number.
pub const CUBE_SIDE_THRESH : MTBRegister<'static> = MTBRegister {
  addr  : 0x15,
  mask  : 0x000000ff,
  descr : "Set the nhit threshold for the cube sides. Needs configurable trigger enabled to be in effect.",
  rmw   : true,
  ro    : false,
  pulse : false
};

/// Set the (nhit) threshold for the cube top panesl
///
/// <div class="warning"> Requires the configurable trigger to be enabled!</div>
///
/// CUBE_TOP_THRESH   0x15    0x54    \[15:8\]  rw  0x0     Threshold for the hit bitmap. Threshold must be > this number.
pub const CUBE_TOP_THRESH : MTBRegister<'static> = MTBRegister {
  addr  : 0x15,
  mask  : 0x0000ff00,
  descr : "Set the nhit threshold for the cube top. Needs configurable trigger enabled to be in effect.",
  rmw   : true,
  ro    : false,
  pulse : false
};

/// Set the (nhit) threshold for the cube bottom panesl
///
/// <div class="warning"> Requires the configurable trigger to be enabled!</div>
///
/// CUBE_BOT_THRESH   0x15    0x54    \[23:16\]     rw  0x0     Threshold for the hit bitmap. Threshold must be > this number.
pub const CUBE_BOT_THRESH : MTBRegister<'static> = MTBRegister {
  addr  : 0x15,
  mask  : 0x00ff0000,
  descr : "Set the nhit threshold for the cube top. Needs configurable trigger enabled to be in effect.",
  rmw   : true,
  ro    : false,
  pulse : false
};


/// Set the (nhit) threshold for the cube bottom panesl
///
/// <div class="warning"> Requires the configurable trigger to be enabled!</div>
///
/// CUBE_CORNER_THRESH    0x15    0x54    \[31:24\]     rw  0x0     Threshold for the hit bitmap. Threshold must be > this number.
pub const CUBE_CORNER_THRESH : MTBRegister<'static> = MTBRegister {
  addr  : 0x15,
  mask  : 0xff000000,
  descr : "Set the nhit threshold for the cube corners. Needs configurable trigger enabled to be in effect.",
  rmw   : true,
  ro    : false,
  pulse : false
};

/// Set the (nhit) threshold for the cube bottom panesl
///
/// <div class="warning"> Requires the configurable trigger to be enabled!</div>
///
/// UMBRELLA_THRESH   0x16    0x58    \[7:0\]   rw  0x0     Threshold for the hit bitmap. Threshold must be > this number.
pub const UMBRELLA_THRESH : MTBRegister<'static> = MTBRegister {
  addr  : 0x16,
  mask  : 0x000000ff,
  descr : "Set the nhit threshold for the complete umbrella. Needs configurable trigger enabled to be in effect.",
  rmw   : true,
  ro    : false,
  pulse : false
};

/// Set the (nhit) threshold for the umbrealla center
///
/// <div class="warning"> Requires the configurable trigger to be enabled!</div>
///
/// UMBRELLA_CENTER_THRESH    0x16    0x58    \[15:8\]  rw  0x0     Threshold for the hit bitmap. Threshold must be > this number.
pub const UMBRELLA_CENTER_THRESH : MTBRegister<'static> = MTBRegister {
  addr  : 0x16,
  mask  : 0x0000ff00,
  descr : "Set the nhit threshold for the umbrella center. Needs configurable trigger enabled to be in effect.",
  rmw   : true,
  ro    : false,
  pulse : false
};

/// Set the (nhit) threshold for the cortina panesl
///
/// <div class="warning"> Requires the configurable trigger to be enabled!</div>
///
/// CORTINA_THRESH    0x16    0x58    \[23:16\]     rw  0x0     Threshold for the hit bitmap. Threshold must be > this number.
pub const CORTINA_THRESH : MTBRegister<'static> = MTBRegister {
  addr  : 0x16,
  mask  : 0x00ff0000,
  descr : "Set the nhit threshold for the cortina. Needs configurable trigger enabled to be in effect.",
  rmw   : true,
  ro    : false,
  pulse : false
};

/// Set the (nhit) threshold for the cortina panesl
///
/// <div class="warning"> Requires the configurable trigger to be enabled!</div>
///
/// LOST_TRIGGER_RATE     0x18    0x60    \[23:0\]  r       Rate of lost triggers in Hz
pub const LOST_TRIGGER_RATE : MTBRegister<'static> = MTBRegister {
  addr  : 0x18,
  mask  : 0x00ffffff,
  descr : "Get lost trigger rate in Hz",
  rmw   : false,
  ro    : true,
  pulse : false
};


/// The lost trigger rate due to "tracker busy" signals
/// received by the TIU
/// TIU_LOST_TRIGGER_RATE 	0x24d 	0x934 	\[23:0\]
pub const TIU_LOST_TRIGGER_RATE : MTBRegister<'static> = MTBRegister {
  addr  : 0x24d,
  mask  : 0x00ffffff,
  descr : "Get tiu lost trigger rate in Hz",
  rmw   : false,
  ro    : true,
  pulse : false
};

/// The lost trigger rate due to RB busy timeouts
/// RB_LOST_TRIGGER_RATE 	0x24c 	0x930 	\[23:0\]
pub const RB_LOST_TRIGGER_RATE : MTBRegister<'static> = MTBRegister {
  addr  : 0x24c,
  mask  : 0x00ffffff,
  descr : "Get RB lost trigger rate in Hz",
  rmw   : false,
  ro    : true,
  pulse : false
};

//Node  Adr     Adr8    Bits    Perm    Def     Description
//HIT_THRESH    0x14    0x50    [29:28]     rw  0x0     Threshold for the hit bitmap. Threshold must be > this number.

/// The global trigger rate in Hz
/// TRIGGER_RATE    0x17    0x5c    \[23:0\]  r       Rate of triggers in Hz
pub const TRIGGER_RATE : MTBRegister<'static> = MTBRegister {
  addr  : 0x17,
  mask  : 0x00ffffff,
  descr : "Get the global trigger rate in Hz",
  rmw   : false,
  ro    : true,
  pulse : false,
};

// LTB Hit counter readout
// MT.HIT_COUNTERS
//
// Counters

/// LTB link 0 available and ready to receive data
//LT_LINK_READY0    0x1a    0x68    \[9:0\]   r       DSI 0 RX Link OK
pub const LT_LINK_READY0 : MTBRegister<'static> = MTBRegister {
  addr  : 0x1a,
  mask  : 0x1ff,
  descr : "LT link 0 ready",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB link 1 available and ready to receive data
//LT_LINK_READY1    0x1b    0x6c    \[9:0\]   r       DSI 1 RX Link OK
pub const LT_LINK_READY1 : MTBRegister<'static> = MTBRegister {
  addr  : 0x1b,
  mask  : 0x1ff,
  descr : "LT link 1 ready",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB link 2 available and ready to receive data
//LT_LINK_READY2    0x1c    0x70    \[9:0\]   r       DSI 2 RX Link OK
pub const LT_LINK_READY2 : MTBRegister<'static> = MTBRegister {
  addr  : 0x1c,
  mask  : 0x1ff,
  descr : "LT link 2 ready",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB link 3 available and ready to receive data
//LT_LINK_READY3    0x1d    0x74    \[9:0\]   r       DSI 3 RX Link OK
pub const LT_LINK_READY3 : MTBRegister<'static> = MTBRegister {
  addr  : 0x1d,
  mask  : 0x1ff,
  descr : "LT link 3 ready",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB link 4 available and ready to receive data
//LT_LINK_READY4    0x1e    0x78    \[9:0\]   r       DSI 4 RX Link OK
pub const LT_LINK_READY4 : MTBRegister<'static> = MTBRegister {
  addr  : 0x1e,
  mask  : 0x1ff,
  descr : "LT link 4 ready",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 0
//LT0   0x20    0x80    \[23:0\]  r       hit count on LT=0
pub const LT0 : MTBRegister<'static> = MTBRegister {
  addr  : 0x20,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=0",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 1
//LT1   0x21    0x80    \[23:0\]  r       hit count on LT=1
pub const LT1 : MTBRegister<'static> = MTBRegister {
  addr  : 0x21,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=1",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 2
//LT2   0x22    0x80    \[23:0\]  r       hit count on LT=2
pub const LT2 : MTBRegister<'static> = MTBRegister {
  addr  : 0x22,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=2",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 3
//LT3   0x23    0x80    \[23:0\]  r       hit count on LT=3
pub const LT3 : MTBRegister<'static> = MTBRegister {
  addr  : 0x23,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=3",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 4
//LT4   0x24    0x80    \[23:0\]  r       hit count on LT=4
pub const LT4 : MTBRegister<'static> = MTBRegister {
  addr  : 0x24,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=4",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 5
//LT5   0x20    0x80    \[23:0\]  r       hit count on LT=5
pub const LT5 : MTBRegister<'static> = MTBRegister {
  addr  : 0x25,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=5",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 6
/// LT6     0x26    0x80    \[23:0\]  r       hit count on LT=6
pub const LT6 : MTBRegister<'static> = MTBRegister {
  addr  : 0x26,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=6",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 7
/// LT7     0x27    0x80    \[23:0\]  r       hit count on LT=7
pub const LT7 : MTBRegister<'static> = MTBRegister {
  addr  : 0x27,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=7",
  rmw   : false,
  ro    : true,
  pulse : false,
};


/// LTB Hit counter for slot 8
/// LT8     0x28    0x80    \[23:0\]  r       hit count on LT=8
pub const LT8 : MTBRegister<'static> = MTBRegister {
  addr  : 0x28,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=8",
  rmw   : false,
  ro    : true,
  pulse : false,
};


/// LTB Hit counter for slot 9
/// LT8     0x29    0x80    \[23:0\]  r       hit count on LT=9
pub const LT9 : MTBRegister<'static> = MTBRegister {
  addr  : 0x29,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=9",
  rmw   : false,
  ro    : true,
  pulse : false,
};


/// LTB Hit counter for slot 10
/// LT10     0x2a    0x80    \[23:0\]  r       hit count on LT=10
pub const LT10 : MTBRegister<'static> = MTBRegister {
  addr  : 0x2a,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=10",
  rmw   : false,
  ro    : true,
  pulse : false,
};


/// LTB Hit counter for slot 11
/// LT11     0x2b    0x80    \[23:0\]  r       hit count on LT=11
pub const LT11 : MTBRegister<'static> = MTBRegister {
  addr  : 0x2b,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=11",
  rmw   : false,
  ro    : true,
  pulse : false,
};


/// LTB Hit counter for slot 12
/// LT12     0x2c    0x80    \[23:0\]  r       hit count on LT=12
pub const LT12 : MTBRegister<'static> = MTBRegister {
  addr  : 0x2c,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=12",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 13
/// LT13     0x2d    0x80    \[23:0\]  r       hit count on LT=13
pub const LT13 : MTBRegister<'static> = MTBRegister {
  addr  : 0x2d,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=13",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 14
/// LT14     0x2e    0x80    \[23:0\]  r       hit count on LT=14
pub const LT14 : MTBRegister<'static> = MTBRegister {
  addr  : 0x2e,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=14",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 15
/// LT15     0x2f    0x80    \[23:0\]  r       hit count on LT=15
pub const LT15 : MTBRegister<'static> = MTBRegister {
  addr  : 0x2f,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=15",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 16
/// LT16     0x30    0x80    \[23:0\]  r       hit count on LT=16
pub const LT16 : MTBRegister<'static> = MTBRegister {
  addr  : 0x30,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=16",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 17
/// LT17     0x31    0x80    \[23:0\]  r       hit count on LT=17
pub const LT17 : MTBRegister<'static> = MTBRegister {
  addr  : 0x31,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=17",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 18
/// LT18     0x32    0x80    \[23:0\]  r       hit count on LT=18
pub const LT18 : MTBRegister<'static> = MTBRegister {
  addr  : 0x32,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=18",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 19
/// LT19     0x33    0x80    \[23:0\]  r       hit count on LT=19
pub const LT19 : MTBRegister<'static> = MTBRegister {
  addr  : 0x33,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=19",
  rmw   : false,
  ro    : true,
  pulse : false,
};


/// LTB Hit counter for slot 20
/// LT20     0x34    0x80    \[23:0\]  r       hit count on LT=20
pub const LT20 : MTBRegister<'static> = MTBRegister {
  addr  : 0x34,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=20",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 0
/// LT21     0x35    0x80    \[23:0\]  r       hit count on LT=21
pub const LT21 : MTBRegister<'static> = MTBRegister {
  addr  : 0x35,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=21",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 22
/// LT22     0x36    0x80    \[23:0\]  r       hit count on LT=22
pub const LT22 : MTBRegister<'static> = MTBRegister {
  addr  : 0x36,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=22",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 23
/// LT23     0x37    0x80    \[23:0\]  r       hit count on LT=23
pub const LT23 : MTBRegister<'static> = MTBRegister {
  addr  : 0x37,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=23",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter for slot 24
/// LT24     0x38    0x80    \[23:0\]  r       hit count on LT=38
pub const LT24 : MTBRegister<'static> = MTBRegister {
  addr  : 0x38,
  mask  : 0x00ffffff,
  descr : "Hit count on LT=24",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// LTB Hit counter reset
/// RESET     0x39    0xe4    0   w   Pulse   Write 1 to reset hit counters.
pub const LT_HIT_CNT_RESET : MTBRegister<'static> = MTBRegister {
  addr  : 0x39,
  mask  : 0x00000001,
  descr : "Reset LT HIT counters 0-24",
  rmw   : false,
  ro    : false,
  pulse : true,
};

/// LTB Hit counter snap
/// SNAP  0x3a    0xe8    0   rw  0x1     1 to snap the hit counters.
pub const LT_HIT_CNT_SNAP : MTBRegister<'static> = MTBRegister {
  addr  : 0x3a,
  mask  : 0x00000001,
  descr : "Snap LT HIT counters 0-24",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LTB channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT0 	0x50 	0x140 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=0
pub const LT0_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x50,
  mask  : 0x000000f,
  descr : "Channel mask for LT=0",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT1 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT1 	0x51 	0x144 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=1
pub const LT1_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x51,
  mask  : 0x000000f,
  descr : "Channel mask for LT=1",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT2 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT2 	0x52 	0x148 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=2
pub const LT2_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x52,
  mask  : 0x000000f,
  descr : "Channel mask for LT=2",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT3 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT3 	0x53 	0x14c 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=3
pub const LT3_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x53,
  mask  : 0x000000f,
  descr : "Channel mask for LT=3",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT4 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT4 	0x54 	0x150 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=4
pub const LT4_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x54,
  mask  : 0x000000f,
  descr : "Channel mask for LT=4",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT5 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT5 	0x55 	0x154 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=5
pub const LT5_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x55,
  mask  : 0x000000f,
  descr : "Channel mask for LT=5",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT6 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT6 	0x56 	0x158 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=6
pub const LT6_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x56,
  mask  : 0x000000f,
  descr : "Channel mask for LT=6",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT7 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT7 	0x57 	0x15c 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=7
pub const LT7_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x57,
  mask  : 0x000000f,
  descr : "Channel mask for LT=7",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT8 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT8 	0x58 	0x160 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=8
pub const LT8_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x58,
  mask  : 0x000000f,
  descr : "Channel mask for LT=8",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT9 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT9 	0x59 	0x164 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=9
pub const LT9_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x59,
  mask  : 0x000000f,
  descr : "Channel mask for LT=9",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT10 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT10 	0x5a 	0x168 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=10
pub const LT10_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x5a,
  mask  : 0x000000f,
  descr : "Channel mask for LT=10",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT11 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT11 	0x5b 	0x16c 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=11
pub const LT11_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x5b,
  mask  : 0x000000f,
  descr : "Channel mask for LT=11",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT12 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT12 	0x5c 	0x170 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=12
pub const LT12_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x5c,
  mask  : 0x000000f,
  descr : "Channel mask for LT=12",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT13 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT13 	0x5d 	0x174 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=13
pub const LT13_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x5d,
  mask  : 0x000000f,
  descr : "Channel mask for LT=13",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT14 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT14 	0x5e 	0x178 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=14
pub const LT14_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x5e,
  mask  : 0x000000f,
  descr : "Channel mask for LT=14",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT15 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT15 	0x5f 	0x17c 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=15
pub const LT15_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x5f,
  mask  : 0x000000f,
  descr : "Channel mask for LT=15",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT16 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT16 	0x60 	0x180 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=16
pub const LT16_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x60,
  mask  : 0x000000f,
  descr : "Channel mask for LT=16",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT17 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT17 	0x61 	0x184 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=17
pub const LT17_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x61,
  mask  : 0x000000f,
  descr : "Channel mask for LT=17",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT18 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT18 	0x62 	0x188 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=18
pub const LT18_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x62,
  mask  : 0x000000f,
  descr : "Channel mask for LT=18",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT19 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT19 	0x63 	0x18c 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=19
pub const LT19_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x63,
  mask  : 0x000000f,
  descr : "Channel mask for LT=19",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT20 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT20 	0x64 	0x190 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=20
pub const LT20_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x64,
  mask  : 0x000000f,
  descr : "Channel mask for LT=20",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT21 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT21 	0x65 	0x194 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=21
pub const LT21_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x65,
  mask  : 0x000000f,
  descr : "Channel mask for LT=21",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT22 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT22 	0x66 	0x198 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=22
pub const LT22_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x66,
  mask  : 0x000000f,
  descr : "Channel mask for LT=22",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT23 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT23 	0x67 	0x19c 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=23
pub const LT23_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x67,
  mask  : 0x000000f,
  descr : "Channel mask for LT=23",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// LT24 channel mask, set 1 to a channel to mask (deactivate it)
///
/// Sets the mask for all channels 0-7 for this LTB simultaneously,
/// one bit per channel
/// LT24 	0x68 	0x1a0 	\[7:0\] 	rw 	0x0 	1 to mask a channel of LT=24
pub const LT24_CHMASK : MTBRegister<'static> = MTBRegister {
  addr  : 0x68,
  mask  : 0x000000f,
  descr : "Channel mask for LT=24",
  rmw   : true,
  ro    : false,
  pulse : false,
};


//MT.RB_READOUT_CNTS
//
//Counters

/// Readout counter on RB=0
/// CNTS_0    0xf2    0x3c8   \[7:0\]   r       Readout count on RB=0
pub const RB0_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf2,
  mask  : 0x000000ff,
  descr : "Counts on RB0/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=1
/// CNTS_1    0xf2    0x3c8   \[15:8\]  r       Readout count on RB=1
pub const RB1_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf2,
  mask  : 0x0000ff00,
  descr : "Counts on RB1/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=2
/// CNTS_2    0xf2    0x3c8   \[23:16\]     r       Readout count on RB=2
pub const RB2_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf2,
  mask  : 0x00ff0000,
  descr : "Counts on RB2/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=3
/// CNTS_3    0xf2    0x3c8   \[31:24\]     r       Readout count on RB=3
pub const RB3_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf2,
  mask  : 0xff000000,
  descr : "Counts on RB3/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=4
/// CNTS_4    0xf3    0x3cc   \[7:0\]   r       Readout count on RB=4
pub const RB4_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf3,
  mask  : 0x000000ff,
  descr : "Counts on RB4/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=5
/// CNTS_5    0xf3    0x3cc   \[15:8\]  r       Readout count on RB=5
pub const RB5_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf3,
  mask  : 0x0000ff00,
  descr : "Counts on RB5/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=6
/// CNTS_6    0xf3    0x3cc   \[23:16\]     r       Readout count on RB=6
pub const RB6_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf3,
  mask  : 0x00ff0000,
  descr : "Counts on RB6/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=7
/// CNTS_7    0xf3    0x3cc   \[31:24\]     r       Readout count on RB=7
pub const RB7_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf3,
  mask  : 0xff000000,
  descr : "Counts on RB7/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=8
/// CNTS_8    0xf4    0x3d0   \[7:0\]   r       Readout count on RB=8
pub const RB8_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf4,
  mask  : 0x000000ff,
  descr : "Counts on RB8/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=9
/// CNTS_9    0xf4    0x3d0   \[15:8\]  r       Readout count on RB=9
pub const RB9_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf4,
  mask  : 0x0000ff00,
  descr : "Counts on RB9/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=10
/// CNTS_10   0xf4    0x3d0   \[23:16\]     r       Readout count on RB=10
pub const RB10_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf4,
  mask  : 0x00ff0000,
  descr : "Counts on RB10/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=11
/// CNTS_11   0xf4    0x3d0   \[31:24\]     r       Readout count on RB=11
pub const RB11_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf4,
  mask  : 0xff000000,
  descr : "Counts on RB11/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=12
/// CNTS_12   0xf5    0x3d4   \[7:0\]   r       Readout count on RB=12
pub const RB12_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf5,
  mask  : 0x000000ff,
  descr : "Counts on RB12/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=13
/// CNTS_13   0xf5    0x3d4   \[15:8\]  r       Readout count on RB=13
pub const RB13_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf5,
  mask  : 0x0000ff00,
  descr : "Counts on RB13/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=14
/// CNTS_14   0xf5    0x3d4   \[23:16\]     r       Readout count on RB=14
pub const RB14_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf5,
  mask  : 0x00ff0000,
  descr : "Counts on RB14/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=15
/// CNTS_15   0xf5    0x3d4   \[31:24\]     r       Readout count on RB=15
pub const RB15_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf5,
  mask  : 0xff000000,
  descr : "Counts on RB15/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=16
/// CNTS_16   0xf6    0x3d8   \[7:0\]   r       Readout count on RB=16
pub const RB16_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf6,
  mask  : 0x000000ff,
  descr : "Counts on RB16/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=17
/// CNTS_17   0xf6    0x3d8   \[15:8\]  r       Readout count on RB=17
pub const RB17_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf6,
  mask  : 0x0000ff00,
  descr : "Counts on RB14/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=18
/// CNTS_18   0xf6    0x3d8   \[23:16\]     r       Readout count on RB=18
pub const RB18_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf6,
  mask  : 0x00ff0000,
  descr : "Counts on RB18/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=19
/// CNTS_19   0xf6    0x3d8   \[31:24\]     r       Readout count on RB=19
pub const RB19_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf6,
  mask  : 0xff000000,
  descr : "Counts on RB19/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=20
/// CNTS_20   0xf7    0x3dc   \[7:0\]   r       Readout count on RB=20
pub const RB20_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf7,
  mask  : 0x000000ff,
  descr : "Counts on RB20/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=21
/// CNTS_21   0xf7    0x3dc   \[15:8\]  r       Readout count on RB=21
pub const RB21_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf7,
  mask  : 0x0000ff00,
  descr : "Counts on RB21/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=22
/// CNTS_22   0xf7    0x3dc   \[23:16\]     r       Readout count on RB=22
pub const RB22_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf7,
  mask  : 0x00ff0000,
  descr : "Counts on RB22/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=23
/// CNTS_23   0xf7    0x3dc   \[31:24\]     r       Readout count on RB=23
pub const RB23_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf7,
  mask  : 0xff000000,
  descr : "Counts on RB23/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=24
/// CNTS_24   0xf8    0x3e0   \[7:0\]   r       Readout count on RB=24
pub const RB24_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf8,
  mask  : 0x000000ff,
  descr : "Counts on RB24/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=25
/// CNTS_25   0xf8    0x3e0   \[15:8\]  r       Readout count on RB=25
pub const RB25_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf8,
  mask  : 0x0000ff00,
  descr : "Counts on RB25/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=26
/// CNTS_26   0xf8    0x3e0   \[23:16\]     r       Readout count on RB=26
pub const RB26_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf8,
  mask  : 0x00ff0000,
  descr : "Counts on RB26/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=27
/// CNTS_27   0xf8    0x3e0   \[31:24\]     r       Readout count on RB=27
pub const RB27_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf8,
  mask  : 0xff000000,
  descr : "Counts on RB27/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=28
/// CNTS_28   0xf9    0x3e4   \[7:0\]   r       Readout count on RB=28
pub const RB28_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf9,
  mask  : 0x000000ff,
  descr : "Counts on RB28/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=29
/// CNTS_29   0xf9    0x3e4   \[15:8\]  r       Readout count on RB=29
pub const RB29_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf9,
  mask  : 0x0000ff00,
  descr : "Counts on RB29/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=30
/// CNTS_30   0xf9    0x3e4   \[23:16\]     r       Readout count on RB=30
pub const RB30_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf9,
  mask  : 0x00ff0000,
  descr : "Counts on RB30/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=31
/// CNTS_31   0xf9    0x3e4   \[31:24\]     r       Readout count on RB=31
pub const RB31_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xf9,
  mask  : 0xff000000,
  descr : "Counts on RB31/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=32
/// CNTS_32   0xfa    0x3e8   \[7:0\]   r       Readout count on RB=32
pub const RB32_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfa,
  mask  : 0x000000ff,
  descr : "Counts on RB32/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=33
/// CNTS_33   0xfa    0x3e8   \[15:8\]  r       Readout count on RB=33
pub const RB33_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfa,
  mask  : 0x0000ff00,
  descr : "Counts on RB33/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=34
/// CNTS_34   0xfa    0x3e8   \[23:16\]     r       Readout count on RB=34
pub const RB34_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfa,
  mask  : 0x00ff0000,
  descr : "Counts on RB34/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=35
/// CNTS_35   0xfa    0x3e8   \[31:24\]     r       Readout count on RB=35
pub const RB35_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfa,
  mask  : 0xff000000,
  descr : "Counts on RB35/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=36
/// CNTS_36   0xfb    0x3ec   \[7:0\]   r       Readout count on RB=36
pub const RB36_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfb,
  mask  : 0x000000ff,
  descr : "Counts on RB36/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=37
/// CNTS_37   0xfb    0x3ec   \[15:8\]  r       Readout count on RB=37
pub const RB37_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfb,
  mask  : 0x0000ff00,
  descr : "Counts on RB37/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=38
/// CNTS_38   0xfb    0x3ec   \[23:16\]     r       Readout count on RB=38
pub const RB38_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfb,
  mask  : 0x00ff0000,
  descr : "Counts on RB38/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=39
/// CNTS_39   0xfb    0x3ec   \[31:24\]     r       Readout count on RB=39
pub const RB39_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfb,
  mask  : 0xff000000,
  descr : "Counts on RB39/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=40
/// CNTS_40   0xfc    0x3f0   \[7:0\]   r       Readout count on RB=40
pub const RB40_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfc,
  mask  : 0x000000ff,
  descr : "Counts on RB40/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=41
/// CNTS_41   0xfc    0x3f0   \[15:8\]  r       Readout count on RB=41
pub const RB41_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfc,
  mask  : 0x0000ff00,
  descr : "Counts on RB41/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=42
/// CNTS_42   0xfc    0x3f0   \[23:16\]     r       Readout count on RB=42
pub const RB42_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfc,
  mask  : 0x00ff0000,
  descr : "Counts on RB42/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=43
/// CNTS_43   0xfc    0x3f0   \[31:24\]     r       Readout count on RB=43
pub const RB43_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfc,
  mask  : 0xff000000,
  descr : "Counts on RB43/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=44
/// CNTS_44   0xfd    0x3f4   \[7:0\]   r       Readout count on RB=44
pub const RB44_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfd,
  mask  : 0x000000ff,
  descr : "Counts on RB44/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=45
/// CNTS_45   0xfd    0x3f4   \[15:8\]  r       Readout count on RB=45
pub const RB45_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfd,
  mask  : 0x0000ff00,
  descr : "Counts on RB45/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=46
/// CNTS_46   0xfd    0x3f4   \[23:16\]     r       Readout count on RB=46
pub const RB46_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfd,
  mask  : 0x00ff0000,
  descr : "Counts on RB46/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};


/// Readout counter on RB=47
/// CNTS_47   0xfd    0x3f4   \[31:24\]     r       Readout count on RB=47
pub const RB47_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfd,
  mask  : 0xff000000,
  descr : "Counts on RB47/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};


/// Readout counter on RB=48
/// CNTS_48   0xfe    0x3f8   \[7:0\]   r       Readout count on RB=48
pub const RB48_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfe,
  mask  : 0x000000ff,
  descr : "Counts on RB48/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Readout counter on RB=49
/// CNTS_49   0xfe    0x3f8   \[15:8\]  r       Readout count on RB=49
pub const RB49_CNTS : MTBRegister<'static> = MTBRegister {
  addr  : 0xfe,
  mask  : 0x0000ff00,
  descr : "Counts on RB49/(link id?)",
  rmw   : false,
  ro    : true,
  pulse : false,
};

/// Reset RB counters 0-49
/// RESET     0xff    0x3fc   0   w   Pulse   Write 1 to reset hit counters.
pub const RB_CNTS_RESET : MTBRegister<'static> = MTBRegister {
  addr  : 0xff,
  mask  : 0x00000001,
  descr : "Reset RB counters 1-49",
  rmw   : false,
  ro    : false,
  pulse : true,
};

/// Snap RB counters 0-49
/// SNAP  0x100   0x400   0   rw  0x1     1 to snap the hit counters.
pub const RB_CNTS_SNAP : MTBRegister<'static> = MTBRegister {
  addr  : 0x100,
  mask  : 0x00000001,
  descr : "Snap RB counters 1-49",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// Add the central track trigger to all triggers
/// TRACK_CENTRAL_IS_GLOBAL   0xb     0x2c    2   rw  0x0     1 makes the TRACK central read all paddles.
pub const TRACK_CENTRAL_IS_GLOBAL : MTBRegister<'static> = MTBRegister {
  addr  : 0xb,
  mask  : 0x4,
  descr : "1 makes the TRACK central read all paddles",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// Add the umbrella central track trigger to all triggers
/// TRACK_UMB_CENTRAL_IS_GLOBAL 	0xb 	0x2c 	3 	rw 	0x0 	1 makes the TRACK UMB central read all paddles.
pub const TRACK_UMB_CENTRAL_IS_GLOBAL : MTBRegister<'static> = MTBRegister {
  addr  : 0xb,
  mask  : 0x8,
  descr : "1 makes the TRACK UMB CENTRAL trigger read all paddles",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// Add the track trigger to all triggers
/// TRACE_TRIG_IS_GLOBAL 0xb 0x2c 1 rw 0x0
pub const TRACK_TRIG_IS_GLOBAL : MTBRegister<'static> = MTBRegister {
  addr  : 0xb,
  mask  : 0x2,
  descr : "1 makes the TRACK trigger read all paddles",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// Add any trigger to all triggers
/// ANY_TRIG_IS_GLOBAL 0xb 0x2c 0 rw 0x0
pub const ANY_TRIG_IS_GLOBAL : MTBRegister<'static> = MTBRegister {
  addr  : 0xb,
  mask  : 0x1,
  descr : "1 makes the ANY trigger read all paddles",
  rmw   : true,
  ro    : false,
  pulse : false,
};

/// Choose between the 2 different TIU links
/// 1 for J11, 0 for J3
/// 0xe     0x38    1   rw  0x0     1 to use J11; 0 to use J3
pub const TIU_USE_AUX_LINK : MTBRegister<'static> = MTBRegister {
  addr  : 0xe,
  mask  : 0x2,
  descr : "Choose between the 2 tiu links",
  rmw   : true,
  ro    : false,
  pulse : false,
};

///TRIG_GEN_RATE     0x9     0x24    \[31:0\]  rw  0x0     Rate of generated triggers f_trig = (1/clk_period) * rate/0xffffffff
/// Set a random forced trigger
pub const TRIG_GEN_RATE : MTBRegister<'static> = MTBRegister {
  addr  : 0x9,
  mask  : 0xffffffff,
  descr : "Set a forced trigger (1/clk_period) * rate/0xffffffff",
  rmw   : false,
  ro    : false,
  pulse : false,
};

// All the trigger settings

// .... WIP!!! So many are not implemented yet....

//LOOPBACK  0x0     0x0     [31:0]  rw  0x0     Loopback register
//CLOCK_RATE    0x1     0x4     [31:0]  r       System clock frequency
//FB_CLOCK_RATE_0   0x2     0x8     [31:0]  r       Feedback clock frequency
//FB_CLOCK_RATE_1   0x3     0xc     [31:0]  r       Feedback clock frequency
//FB_CLOCK_RATE_2   0x4     0x10    [31:0]  r       Feedback clock frequency
//FB_CLOCK_RATE_3   0x5     0x14    [31:0]  r       Feedback clock frequency
//FB_CLOCK_RATE_4   0x6     0x18    [31:0]  r       Feedback clock frequency
//DSI_ON    0x7     0x1c    [4:0]   rw  0x1F    Bitmask 1 = enable DSI
//RESYNC    0xa     0x28    0   w   Pulse   Write 1 to resync
//ANY_TRIG_IS_GLOBAL    0xb     0x2c    0   rw  0x0     1 makes the ANY trigger read all paddles.
//TRACK_TRIG_IS_GLOBAL  0xb     0x2c    1   rw  0x0     1 makes the TRACK trigger read all paddles.
//TRACK_CENTRAL_IS_GLOBAL   0xb     0x2c    2   rw  0x0     1 makes the TRACK central read all paddles.
//TRACK_CENTRAL_IS_GLOBAL   0xb     0x2c    2   rw  0x0     1 makes the TRACK central read all paddles.
//TIU_EMU_BUSY_CNT  0xe     0x38    [31:14]     rw  0xC350  Number of 10 ns clock cyles that the emulator will remain busy
//TIU_BAD   0xf     0x3c    0   r       1 means that the tiu link is not working
//LT_INPUT_STRETCH  0xf     0x3c    [7:4]   rw  0xF     Number of clock cycles to stretch the LT inputs by
//MT.EVENT_QUEUE
//
//DAQ Buffer
//Node  Adr     Adr8    Bits    Perm    Def     Description
//RESET     0x10    0x40    0   w   Pulse   DAQ Buffer Reset
//DATA  0x11    0x44    [31:0]  r       DAQ Read Data
//FULL  0x12    0x48    0   r       DAQ Buffer Full
//EMPTY     0x12    0x48    1   r       DAQ Buffer Empty
//SIZE  0x13    0x4c    [31:16]     r       DAQ Buffer Head Event Size
//
//MT
//
//
//MT.HIT_COUNTERS
//
//Counters
//Node  Adr     Adr8    Bits    Perm    Def     Description
//LT0   0x20    0x80    [23:0]  r       hit count on LT=0
//LT1   0x21    0x84    [23:0]  r       hit count on LT=1
//LT2   0x22    0x88    [23:0]  r       hit count on LT=2
//LT3   0x23    0x8c    [23:0]  r       hit count on LT=3
//LT4   0x24    0x90    [23:0]  r       hit count on LT=4
//LT5   0x25    0x94    [23:0]  r       hit count on LT=5
//LT6   0x26    0x98    [23:0]  r       hit count on LT=6
//LT7   0x27    0x9c    [23:0]  r       hit count on LT=7
//LT8   0x28    0xa0    [23:0]  r       hit count on LT=8
//LT9   0x29    0xa4    [23:0]  r       hit count on LT=9
//LT10  0x2a    0xa8    [23:0]  r       hit count on LT=10
//LT11  0x2b    0xac    [23:0]  r       hit count on LT=11
//LT12  0x2c    0xb0    [23:0]  r       hit count on LT=12
//LT13  0x2d    0xb4    [23:0]  r       hit count on LT=13
//LT14  0x2e    0xb8    [23:0]  r       hit count on LT=14
//LT15  0x2f    0xbc    [23:0]  r       hit count on LT=15
//LT16  0x30    0xc0    [23:0]  r       hit count on LT=16
//LT17  0x31    0xc4    [23:0]  r       hit count on LT=17
//LT18  0x32    0xc8    [23:0]  r       hit count on LT=18
//LT19  0x33    0xcc    [23:0]  r       hit count on LT=19
//LT20  0x34    0xd0    [23:0]  r       hit count on LT=20
//LT21  0x35    0xd4    [23:0]  r       hit count on LT=21
//LT22  0x36    0xd8    [23:0]  r       hit count on LT=22
//LT23  0x37    0xdc    [23:0]  r       hit count on LT=23
//LT24  0x38    0xe0    [23:0]  r       hit count on LT=24
//RESET     0x39    0xe4    0   w   Pulse   Write 1 to reset hit counters.
//SNAP  0x3a    0xe8    0   rw  0x1     1 to snap the hit counters.
//
//MT
//
//Implements various control and monitoring functions of the DRS Logic
//Node  Adr     Adr8    Bits    Perm    Def     Description
//ETH_RX_BAD_FRAME_CNT  0x3d    0xf4    [15:0]  r       Ethernet MAC bad frame error
//ETH_RX_BAD_FCS_CNT    0x3d    0xf4    [31:16]     r       Ethernet MAC bad fcs
//
//MT.CHANNEL_MASK
//
//1 to mask a channel
//Node  Adr     Adr8    Bits    Perm    Def     Description
//LT0   0x50    0x140   [7:0]   rw  0x0     1 to mask a channel of LT=0
//LT1   0x51    0x144   [7:0]   rw  0x0     1 to mask a channel of LT=1
//LT2   0x52    0x148   [7:0]   rw  0x0     1 to mask a channel of LT=2
//LT3   0x53    0x14c   [7:0]   rw  0x0     1 to mask a channel of LT=3
//LT4   0x54    0x150   [7:0]   rw  0x0     1 to mask a channel of LT=4
//LT5   0x55    0x154   [7:0]   rw  0x0     1 to mask a channel of LT=5
//LT6   0x56    0x158   [7:0]   rw  0x0     1 to mask a channel of LT=6
//LT7   0x57    0x15c   [7:0]   rw  0x0     1 to mask a channel of LT=7
//LT8   0x58    0x160   [7:0]   rw  0x0     1 to mask a channel of LT=8
//LT9   0x59    0x164   [7:0]   rw  0x0     1 to mask a channel of LT=9
//LT10  0x5a    0x168   [7:0]   rw  0x0     1 to mask a channel of LT=10
//LT11  0x5b    0x16c   [7:0]   rw  0x0     1 to mask a channel of LT=11
//LT12  0x5c    0x170   [7:0]   rw  0x0     1 to mask a channel of LT=12
//LT13  0x5d    0x174   [7:0]   rw  0x0     1 to mask a channel of LT=13
//LT14  0x5e    0x178   [7:0]   rw  0x0     1 to mask a channel of LT=14
//LT15  0x5f    0x17c   [7:0]   rw  0x0     1 to mask a channel of LT=15
//LT16  0x60    0x180   [7:0]   rw  0x0     1 to mask a channel of LT=16
//LT17  0x61    0x184   [7:0]   rw  0x0     1 to mask a channel of LT=17
//LT18  0x62    0x188   [7:0]   rw  0x0     1 to mask a channel of LT=18
//LT19  0x63    0x18c   [7:0]   rw  0x0     1 to mask a channel of LT=19
//LT20  0x64    0x190   [7:0]   rw  0x0     1 to mask a channel of LT=20
//LT21  0x65    0x194   [7:0]   rw  0x0     1 to mask a channel of LT=21
//LT22  0x66    0x198   [7:0]   rw  0x0     1 to mask a channel of LT=22
//LT23  0x67    0x19c   [7:0]   rw  0x0     1 to mask a channel of LT=23
//LT24  0x68    0x1a0   [7:0]   rw  0x0     1 to mask a channel of LT=24
//
//MT.COARSE_DELAYS
//Node  Adr     Adr8    Bits    Perm    Def     Description
//LT0   0xc0    0x300   [3:0]   rw  0x0     Integer clock delay of LT LINK 0
//LT1   0xc1    0x304   [3:0]   rw  0x0     Integer clock delay of LT LINK 1
//LT2   0xc2    0x308   [3:0]   rw  0x0     Integer clock delay of LT LINK 2
//LT3   0xc3    0x30c   [3:0]   rw  0x0     Integer clock delay of LT LINK 3
//LT4   0xc4    0x310   [3:0]   rw  0x0     Integer clock delay of LT LINK 4
//LT5   0xc5    0x314   [3:0]   rw  0x0     Integer clock delay of LT LINK 5
//LT6   0xc6    0x318   [3:0]   rw  0x0     Integer clock delay of LT LINK 6
//LT7   0xc7    0x31c   [3:0]   rw  0x0     Integer clock delay of LT LINK 7
//LT8   0xc8    0x320   [3:0]   rw  0x0     Integer clock delay of LT LINK 8
//LT9   0xc9    0x324   [3:0]   rw  0x0     Integer clock delay of LT LINK 9
//LT10  0xca    0x328   [3:0]   rw  0x0     Integer clock delay of LT LINK 10
//LT11  0xcb    0x32c   [3:0]   rw  0x0     Integer clock delay of LT LINK 11
//LT12  0xcc    0x330   [3:0]   rw  0x0     Integer clock delay of LT LINK 12
//LT13  0xcd    0x334   [3:0]   rw  0x0     Integer clock delay of LT LINK 13
//LT14  0xce    0x338   [3:0]   rw  0x0     Integer clock delay of LT LINK 14
//LT15  0xcf    0x33c   [3:0]   rw  0x0     Integer clock delay of LT LINK 15
//LT16  0xd0    0x340   [3:0]   rw  0x0     Integer clock delay of LT LINK 16
//LT17  0xd1    0x344   [3:0]   rw  0x0     Integer clock delay of LT LINK 17
//LT18  0xd2    0x348   [3:0]   rw  0x0     Integer clock delay of LT LINK 18
//LT19  0xd3    0x34c   [3:0]   rw  0x0     Integer clock delay of LT LINK 19
//LT20  0xd4    0x350   [3:0]   rw  0x0     Integer clock delay of LT LINK 20
//LT21  0xd5    0x354   [3:0]   rw  0x0     Integer clock delay of LT LINK 21
//LT22  0xd6    0x358   [3:0]   rw  0x0     Integer clock delay of LT LINK 22
//LT23  0xd7    0x35c   [3:0]   rw  0x0     Integer clock delay of LT LINK 23
//LT24  0xd8    0x360   [3:0]   rw  0x0     Integer clock delay of LT LINK 24
//LT25  0xd9    0x364   [3:0]   rw  0x0     Integer clock delay of LT LINK 25
//LT26  0xda    0x368   [3:0]   rw  0x0     Integer clock delay of LT LINK 26
//LT27  0xdb    0x36c   [3:0]   rw  0x0     Integer clock delay of LT LINK 27
//LT28  0xdc    0x370   [3:0]   rw  0x0     Integer clock delay of LT LINK 28
//LT29  0xdd    0x374   [3:0]   rw  0x0     Integer clock delay of LT LINK 29
//LT30  0xde    0x378   [3:0]   rw  0x0     Integer clock delay of LT LINK 30
//LT31  0xdf    0x37c   [3:0]   rw  0x0     Integer clock delay of LT LINK 31
//LT32  0xe0    0x380   [3:0]   rw  0x0     Integer clock delay of LT LINK 32
//LT33  0xe1    0x384   [3:0]   rw  0x0     Integer clock delay of LT LINK 33
//LT34  0xe2    0x388   [3:0]   rw  0x0     Integer clock delay of LT LINK 34
//LT35  0xe3    0x38c   [3:0]   rw  0x0     Integer clock delay of LT LINK 35
//LT36  0xe4    0x390   [3:0]   rw  0x0     Integer clock delay of LT LINK 36
//LT37  0xe5    0x394   [3:0]   rw  0x0     Integer clock delay of LT LINK 37
//LT38  0xe6    0x398   [3:0]   rw  0x0     Integer clock delay of LT LINK 38
//LT39  0xe7    0x39c   [3:0]   rw  0x0     Integer clock delay of LT LINK 39
//LT40  0xe8    0x3a0   [3:0]   rw  0x0     Integer clock delay of LT LINK 40
//LT41  0xe9    0x3a4   [3:0]   rw  0x0     Integer clock delay of LT LINK 41
//LT42  0xea    0x3a8   [3:0]   rw  0x0     Integer clock delay of LT LINK 42
//LT43  0xeb    0x3ac   [3:0]   rw  0x0     Integer clock delay of LT LINK 43
//LT44  0xec    0x3b0   [3:0]   rw  0x0     Integer clock delay of LT LINK 44
//LT45  0xed    0x3b4   [3:0]   rw  0x0     Integer clock delay of LT LINK 45
//LT46  0xee    0x3b8   [3:0]   rw  0x0     Integer clock delay of LT LINK 46
//LT47  0xef    0x3bc   [3:0]   rw  0x0     Integer clock delay of LT LINK 47
//LT48  0xf0    0x3c0   [3:0]   rw  0x0     Integer clock delay of LT LINK 48
//LT49  0xf1    0x3c4   [3:0]   rw  0x0     Integer clock delay of LT LINK 49
//
//MT.RB_READOUT_CNTS
//
//Counters
//Node  Adr     Adr8    Bits    Perm    Def     Description
//CNTS_0    0xf2    0x3c8   [7:0]   r       Readout count on RB=0
//CNTS_1    0xf2    0x3c8   [15:8]  r       Readout count on RB=1
//CNTS_2    0xf2    0x3c8   [23:16]     r       Readout count on RB=2
//CNTS_3    0xf2    0x3c8   [31:24]     r       Readout count on RB=3
//CNTS_4    0xf3    0x3cc   [7:0]   r       Readout count on RB=4
//CNTS_5    0xf3    0x3cc   [15:8]  r       Readout count on RB=5
//CNTS_6    0xf3    0x3cc   [23:16]     r       Readout count on RB=6
//CNTS_7    0xf3    0x3cc   [31:24]     r       Readout count on RB=7
//CNTS_8    0xf4    0x3d0   [7:0]   r       Readout count on RB=8
//CNTS_9    0xf4    0x3d0   [15:8]  r       Readout count on RB=9
//CNTS_10   0xf4    0x3d0   [23:16]     r       Readout count on RB=10
//CNTS_11   0xf4    0x3d0   [31:24]     r       Readout count on RB=11
//CNTS_12   0xf5    0x3d4   [7:0]   r       Readout count on RB=12
//CNTS_13   0xf5    0x3d4   [15:8]  r       Readout count on RB=13
//CNTS_14   0xf5    0x3d4   [23:16]     r       Readout count on RB=14
//CNTS_15   0xf5    0x3d4   [31:24]     r       Readout count on RB=15
//CNTS_16   0xf6    0x3d8   [7:0]   r       Readout count on RB=16
//CNTS_17   0xf6    0x3d8   [15:8]  r       Readout count on RB=17
//CNTS_18   0xf6    0x3d8   [23:16]     r       Readout count on RB=18
//CNTS_19   0xf6    0x3d8   [31:24]     r       Readout count on RB=19
//CNTS_20   0xf7    0x3dc   [7:0]   r       Readout count on RB=20
//CNTS_21   0xf7    0x3dc   [15:8]  r       Readout count on RB=21
//CNTS_22   0xf7    0x3dc   [23:16]     r       Readout count on RB=22
//CNTS_23   0xf7    0x3dc   [31:24]     r       Readout count on RB=23
//CNTS_24   0xf8    0x3e0   [7:0]   r       Readout count on RB=24
//CNTS_25   0xf8    0x3e0   [15:8]  r       Readout count on RB=25
//CNTS_26   0xf8    0x3e0   [23:16]     r       Readout count on RB=26
//CNTS_27   0xf8    0x3e0   [31:24]     r       Readout count on RB=27
//CNTS_28   0xf9    0x3e4   [7:0]   r       Readout count on RB=28
//CNTS_29   0xf9    0x3e4   [15:8]  r       Readout count on RB=29
//CNTS_30   0xf9    0x3e4   [23:16]     r       Readout count on RB=30
//CNTS_31   0xf9    0x3e4   [31:24]     r       Readout count on RB=31
//CNTS_32   0xfa    0x3e8   [7:0]   r       Readout count on RB=32
//CNTS_33   0xfa    0x3e8   [15:8]  r       Readout count on RB=33
//CNTS_34   0xfa    0x3e8   [23:16]     r       Readout count on RB=34
//CNTS_35   0xfa    0x3e8   [31:24]     r       Readout count on RB=35
//CNTS_36   0xfb    0x3ec   [7:0]   r       Readout count on RB=36
//CNTS_37   0xfb    0x3ec   [15:8]  r       Readout count on RB=37
//CNTS_38   0xfb    0x3ec   [23:16]     r       Readout count on RB=38
//CNTS_39   0xfb    0x3ec   [31:24]     r       Readout count on RB=39
//CNTS_40   0xfc    0x3f0   [7:0]   r       Readout count on RB=40
//CNTS_41   0xfc    0x3f0   [15:8]  r       Readout count on RB=41
//CNTS_42   0xfc    0x3f0   [23:16]     r       Readout count on RB=42
//CNTS_43   0xfc    0x3f0   [31:24]     r       Readout count on RB=43
//CNTS_44   0xfd    0x3f4   [7:0]   r       Readout count on RB=44
//CNTS_45   0xfd    0x3f4   [15:8]  r       Readout count on RB=45
//CNTS_46   0xfd    0x3f4   [23:16]     r       Readout count on RB=46
//CNTS_47   0xfd    0x3f4   [31:24]     r       Readout count on RB=47
//CNTS_48   0xfe    0x3f8   [7:0]   r       Readout count on RB=48
//CNTS_49   0xfe    0x3f8   [15:8]  r       Readout count on RB=49
//RESET     0xff    0x3fc   0   w   Pulse   Write 1 to reset hit counters.
//SNAP  0x100   0x400   0   rw  0x1     1 to snap the hit counters.
//
//MT.PULSER
//
//LTB Channel Pulser
//Node  Adr     Adr8    Bits    Perm    Def     Description
//FIRE  0x100   0x400   0   w   Pulse   Write 1 to Fire the Pulser.
//CH_0_24   0x101   0x404   [24:0]  rw  0x0     Set fire bits for channels 0 to 24
//CH_25_49  0x102   0x408   [24:0]  rw  0x0     Set fire bits for channels 25 to 49
//CH_50_74  0x103   0x40c   [24:0]  rw  0x0     Set fire bits for channels 50 to 74
//CH_75_99  0x104   0x410   [24:0]  rw  0x0     Set fire bits for channels 75 to 99
//CH_100_124    0x105   0x414   [24:0]  rw  0x0     Set fire bits for channels 100 to 124
//CH_125_149    0x106   0x418   [24:0]  rw  0x0     Set fire bits for channels 125 to 149
//CH_150_174    0x107   0x41c   [24:0]  rw  0x0     Set fire bits for channels 150 to 174
//CH_175_199    0x108   0x420   [24:0]  rw  0x0     Set fire bits for channels 175 to 199
//
//MT.XADC
//
//Zynq XADC
//Node  Adr     Adr8    Bits    Perm    Def     Description
//CALIBRATION   0x120   0x480   [11:0]  r       XADC Calibration
//VCCPINT   0x120   0x480   [27:16]     r       XADC vccpint
//VCCPAUX   0x121   0x484   [11:0]  r       XADC Calibration
//VCCODDR   0x121   0x484   [27:16]     r       XADC vccoddr
//TEMP  0x122   0x488   [11:0]  r       XADC Temperature
//VCCINT    0x122   0x488   [27:16]     r       XADC vccint
//VCCAUX    0x123   0x48c   [11:0]  r       XADC VCCAUX
//VCCBRAM   0x123   0x48c   [27:16]     r       XADC vccbram
//
//MT.HOG
//
//HOG Parameters
//Node  Adr     Adr8    Bits    Perm    Def     Description
//GLOBAL_DATE   0x200   0x800   [31:0]  r       HOG Global Date
//GLOBAL_TIME   0x201   0x804   [31:0]  r       HOG Global Time
//GLOBAL_VER    0x202   0x808   [31:0]  r       HOG Global Version
//GLOBAL_SHA    0x203   0x80c   [31:0]  r       HOG Global SHA
//TOP_SHA   0x204   0x810   [31:0]  r       HOG Top SHA
//TOP_VER   0x205   0x814   [31:0]  r       HOG Top Version
//HOG_SHA   0x206   0x818   [31:0]  r       HOG SHA
//HOG_VER   0x207   0x81c   [31:0]  r       HOG Version
//Module SPI adr = 0x1000
//
//SPI
//Node  Adr     Adr8    Bits    Perm    Def     Description
//d0    0x1000  0x4000  [31:0]  rw  ~~  Data reg 0
//d1    0x1001  0x4004  [31:0]  rw  ~~  Data reg 1
//d2    0x1002  0x4008  [31:0]  rw  ~~  Data reg 2
//d3    0x1003  0x400c  [31:0]  rw  ~~  Data reg 3
//ctrl  0x1004  0x4010  [31:0]  rw  ~~  Control reg
//divider   0x1005  0x4014  [31:0]  rw  ~~  Clock divider reg
//ss    0x1006  0x4018  [31:0]  rw  ~~  Slave select reg
//Module I2C adr = 0x1100
//
//I2C master controller
//
//I2C
//
//I2C master controller
//Node  Adr     Adr8    Bits    Perm    Def     Description
//ps_lo     0x1100  0x4400  [7:0]   rw  ~~  Prescale low byte
//ps_hi     0x1101  0x4404  [7:0]   rw  ~~  Prescale low byte
//ctrl  0x1102  0x4408  [7:0]   rw  ~~  Control
//data  0x1103  0x440c  [7:0]   rw  ~~  Data
//cmd_stat  0x1104  0x4410  [7:0]   rw  ~~  Command / status



