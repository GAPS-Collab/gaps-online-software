//! Master trigger board control
//!
//! Set registers and enable/disable features,
//! readout event and monitoring data
//!
//! Many control functions in this module can go
//! away, since they can be replaced by the 
//! calls on the register implementation directly.

use std::error::Error;
use crate::io::ipbus::IPBus;

use crate::tof::master_trigger::registers::*;

/// Read event counter register of MTB
///
/// This gives the number of events read 
/// since the last counter reset
pub fn read_event_cnt(bus : &mut IPBus) //,
                      //buffer : &mut [u8;MT_MAX_PACKSIZE])
  -> Result<u32, Box<dyn Error>> {
  let event_count = bus.read(0xd)?;
  trace!("Got event count! {} ", event_count);
  Ok(event_count)
}

/// Set the RB readout mode - either 
/// read out all channels all the time
/// or use the MTB to indicate to the RBs
/// which channels to read out 
pub fn set_trace_suppression(bus : &mut IPBus,
                             sup : bool) 
  -> Result<(), Box<dyn Error>> {
  info!("Setting MTB trace suppression {}!", sup);
  let mut value = bus.read(0xf)?;
  // bit 13 has to be 1 for read all channels
  let read_all_ch = u32::pow(2, 13);
  if sup { // sup means !read_all_ch
    value = value & !read_all_ch;
  }
  else {
    value = value | read_all_ch; 
  }
  bus.write(0xf, value)?;
  Ok(())
}

/// Reset the state of the MTB DAQ buffer
/// This can be safely issued without 
/// resetting the event id
pub fn reset_daq(bus : &mut IPBus) 
  -> Result<(), Box<dyn Error>> {
  info!("Resetting DAQ!");
  bus.write(0x10, 1)?;
  Ok(())
}


/// Retrieve the status of the TIU link
///
/// The TIU is the trigger interface unit
/// and connects TOF and Tracker
///
/// # Returns:
///
///   bool : true if the link status is "good"
///          which means TIU is connected and 
///          link is ok
pub fn tiu_link_is_good(bus : &mut IPBus)
  -> Result<bool, Box<dyn Error>> {
  let mut tiu_good = 0x1u32;
  let value        = bus.read(0xf)?;
  tiu_good         = tiu_good & ( value & 0x1);
  Ok(tiu_good > 0)
}

/// The TIU emulation mode literally allows to emulate a TIU even if it 
/// is not connected.
///
/// This will inject a certain deadtime into the MasterTrigger triggering
/// system, as if a BUSY signal was received from an actual TIU
pub fn set_tiu_emulation_mode(bus : &mut IPBus, set_emulation_mode : bool) 
  -> Result<(), Box<dyn Error>> {
    info!("Setting TIU Emulation mode {}", set_emulation_mode);
    let mut value = bus.read(0xe)?;
    let bitset : u32;
    if set_emulation_mode {
      bitset = 0x1;
    } else {
      bitset = 0x0;
    }
    value = value & 0xfffffffe;
    value = value | bitset;
    bus.write(0xe, value)?;
    Ok(())
}

/// Set the busy count for the tiu emulation mode in 10ns clockcycles
pub fn set_tiu_emulation_mode_bsy_cnt(bus : &mut IPBus, cycles : u32)
  -> Result<(), Box<dyn Error>> {
  info!("Setting TIU Emulation mode bsy cnt to {} clock cycles (10ns each)", cycles);
  TIU_EMU_BUSY_CNT.set(bus, cycles)?;
  Ok(()) 
}

/// Get the number of clock cycles (1=10ns) that the emulator will remain busy
pub fn get_tiu_emu_busy_cnt(bus : &mut IPBus) 
  -> Result<u32, Box<dyn Error>> {
  TIU_EMU_BUSY_CNT.get(bus)
}

pub fn get_gaps_trigger_prescale(bus : &mut IPBus)
  -> Result<f32, Box<dyn Error>> {
    let prescale_bus = GAPS_TRIG_PRESCALE.get(bus)?;
    let prescale_val = (prescale_bus as f32)/f32::MAX;
    return Ok(prescale_val)
  }

pub fn set_gaps_trigger_prescale(bus : &mut IPBus, prescale : f32)
  -> Result<(), Box<dyn Error>> {
    let prescale_val = (u32::MAX as f32 * prescale).floor() as u32;
    info!("Setting gaps trigger with prescale {};)", prescale);
    bus.write(0x248, prescale_val)?;
  Ok(())
  }
/// The readoutboard integration window
///
/// The default setting is "1" and currently 
/// it should only be changed by experts for 
/// a good reason.
///
/// The RB integration window affects the readout
/// of RB pulses which are not part of the initial
/// Trigger, but come later instead.
/// This is reflected in the BOARD_MASKs/mtb_link_id
/// mas of the MasterTriggerEvent. The larger the 
/// number, the more boards we might expect. (?)
pub fn set_rb_int_window(bus : &mut IPBus, wind : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting RB_INT_WINDOW to {}!", wind);
  let mut value  =  bus.read(0xf)?;
  //println!("==> Retrieved {value} from register 0xf on MTB");
  let mask   = 0xffffe0ff;
  // switch the bins off
  value          = value & mask;
  let wind_bits  = (wind as u32) << 8;
  value = value | wind_bits;
  bus.write(0xf, value)?;
  trace!("++ Writing to register ++");
  value = bus.read(0xf)?;
  trace!("==> Reading back value {value} from register 0xf on MTB after writing to it!");
  Ok(())
}

/// Set the poisson trigger with a prescale
pub fn set_poisson_trigger(bus : &mut IPBus, rate : u32) 
  -> Result<(), Box<dyn Error>> {
  //let clk_period = 1e8u32; 
  let clk_period = 100000000;
  let rate_val = (u32::MAX*rate)/clk_period;//(1.0/ clk_period)).floor() as u32;
  
  //let rate_val   = (rate as f32 * u32::MAX as f32/1.0e8) as u32; 
  info!("Setting poisson trigger with rate {}!", rate);
  bus.write(0x9, rate_val)?;
  Ok(())
}

/// Set the any trigger with a prescale
pub fn set_any_trigger(bus : &mut IPBus, prescale : f32) 
  -> Result<(), Box<dyn Error>> {
  let prescale_val = (u32::MAX as f32 * prescale).floor() as u32;
  info!("Setting any trigger with prescale {}!", prescale);
  bus.write(0x40, prescale_val)?;
  Ok(())
}

/// Set the track trigger with a prescale
pub fn set_track_trigger(bus : &mut IPBus, prescale : f32) 
  -> Result<(), Box<dyn Error>> {
  let prescale_val = (u32::MAX as f32 * prescale).floor() as u32;
  info!("Setting track trigger with prescale {}!", prescale);
  bus.write(0x41, prescale_val)?;
  Ok(())
}

/// Set the CENTRAL track trigger with a prescale
pub fn set_central_track_trigger(bus : &mut IPBus, prescale : f32) 
  -> Result<(), Box<dyn Error>> {
  let prescale_val = (u32::MAX as f32 * prescale).floor() as u32;
  info!("Setting CENTRAL TRACK trigger with prescale {}!", prescale);
  bus.write(0x42, prescale_val)?;
  Ok(())
}

pub fn set_track_umb_central_trigger(bus : &mut IPBus, prescale : f32)
    -> Result<(), Box<dyn Error>> {
    let prescale_val = (u32::MAX as f32 * prescale).floor() as u32;
    info!("Setting TRACK UMB CENTRAL trigger with prescale {}!", prescale);
    bus.write(0x249, prescale_val)?;
    Ok(())
  }


/// Disable all triggers
pub fn unset_all_triggers(bus : &mut IPBus) 
  -> Result<(), Box<dyn Error>> {
  // first the GAPS trigger, whcih is a more 
  // complicated register, where we only have
  // to flip 1 bit
  //zero_all_trigger_thresholds(bus)?;
  let mut trig_settings = bus.read(0x14)?;
  trig_settings         = trig_settings & !u32::pow(2,24);
  bus.write(0x14, trig_settings)?;
  set_poisson_trigger(bus, 0)?;
  set_any_trigger    (bus, 0.0)?;
  set_track_trigger  (bus, 0.0)?;
  set_central_track_trigger(bus, 0.0)?;
  set_configurable_trigger(bus, false)?;
  TRACK_TRIG_IS_GLOBAL.set(bus, 0)?; 
  ANY_TRIG_IS_GLOBAL.set(bus, 0)?;
  TRACK_CENTRAL_IS_GLOBAL.set(bus, 0)?;
  set_track_umb_central_trigger(bus, 0.0)?;
  Ok(())
}

/// Set the gaps trigger with a prescale
pub fn set_gaps_trigger(bus : &mut IPBus, use_beta : bool) 
  -> Result<(), Box<dyn Error>> {
  info!("Setting GAPS Antiparticle trigger, use beta {}!", use_beta);
  set_inner_tof_threshold(bus,0x3)?;
  set_outer_tof_threshold(bus,0x3)?;
  set_total_tof_threshold(bus,0x8)?;
  let mut trig_settings = bus.read(0x14)?;
  trig_settings = trig_settings | u32::pow(2,24);
  if use_beta {
    trig_settings = trig_settings | u32::pow(2,25);
  }
  bus.write(0x14, trig_settings)?;
  Ok(())
}

/// Set the gaps trigger with a prescale
pub fn set_gaps1044_trigger(bus : &mut IPBus, use_beta : bool) 
  -> Result<(), Box<dyn Error>> {
  info!("Setting GAPS Antiparticle trigger, use beta {}!", use_beta);
  set_inner_tof_threshold(bus,0x4)?;
  set_outer_tof_threshold(bus,0x4)?;
  set_total_tof_threshold(bus,0x10)?;
  let mut trig_settings = bus.read(0x14)?;
  trig_settings = trig_settings | u32::pow(2,24);
  if use_beta {
    trig_settings = trig_settings | u32::pow(2,25);
  }
  bus.write(0x14, trig_settings)?;
  Ok(())
}

pub fn set_gaps_track_trigger(bus : &mut IPBus, prescale : f32, use_beta : bool) 
  -> Result<(), Box<dyn Error>> {
  info!("Setting GAPS + Track trigger combo");
  TRACK_TRIG_IS_GLOBAL.set(bus, 1)?;
  set_gaps_trigger(bus, use_beta)?;
  set_track_trigger(bus, prescale)?;
  Ok(())
}

pub fn set_gaps_any_trigger(bus : &mut IPBus, prescale : f32, use_beta : bool)
  -> Result<(), Box<dyn Error>> {
    info!("Setting GAPS + Any trigger combo");
    ANY_TRIG_IS_GLOBAL.set(bus, 1)?;
    set_gaps_trigger(bus, use_beta)?;
    set_any_trigger(bus, prescale)?;
    Ok(())
  }

pub fn set_gaps_central_track_trigger(bus : &mut IPBus, prescale : f32, use_beta : bool)
  -> Result<(), Box<dyn Error>> {
    info!("Setting GAPS + Central Track trigger combo");
    TRACK_CENTRAL_IS_GLOBAL.set(bus, 1)?;
    set_gaps_trigger(bus, use_beta)?;
    set_central_track_trigger(bus, prescale)?;
    Ok(())
  }

pub fn set_gaps422_central_track_trigger(bus : &mut IPBus, prescale : f32, use_beta : bool)
  -> Result<(), Box<dyn Error>> {
    info!("Setting GAPS + Central Track trigger combo");
    TRACK_CENTRAL_IS_GLOBAL.set(bus, 1)?;
    set_gaps422_trigger(bus, use_beta)?;
    set_central_track_trigger(bus, prescale)?;
    Ok(())
  }

pub fn set_gaps633_trigger(bus : &mut IPBus, use_beta : bool) 
  -> Result<(), Box<dyn Error>> {
  info!("Setting GAPS Antiparticle trigger, use beta {}!", use_beta);
  set_inner_tof_threshold(bus,0x3)?;
  set_outer_tof_threshold(bus,0x3)?;
  set_total_tof_threshold(bus,0x6)?;
  let mut trig_settings = bus.read(0x14)?;
  trig_settings = trig_settings | u32::pow(2,24);
  if use_beta {
    trig_settings = trig_settings | u32::pow(2,25);
  }
  bus.write(0x14, trig_settings)?;
  Ok(())
}

pub fn set_gaps422_trigger(bus : &mut IPBus, use_beta : bool) 
  -> Result<(), Box<dyn Error>> {
  info!("Setting GAPS Antiparticle trigger, use beta {}!", use_beta);
  set_inner_tof_threshold(bus,0x2)?;
  set_outer_tof_threshold(bus,0x2)?;
  set_total_tof_threshold(bus,0x4)?;
  let mut trig_settings = bus.read(0x14)?;
  trig_settings = trig_settings | u32::pow(2,24);
  if use_beta {
    trig_settings = trig_settings | u32::pow(2,25);
  }
  bus.write(0x14, trig_settings)?;
  Ok(())
}

pub fn set_gaps211_trigger(bus : &mut IPBus, use_beta : bool) 
  -> Result<(), Box<dyn Error>> {
  info!("Setting GAPS Antiparticle trigger, use beta {}!", use_beta);
  set_inner_tof_threshold(bus,0x1)?;
  set_outer_tof_threshold(bus,0x1)?;
  set_total_tof_threshold(bus,0x2)?;
  let mut trig_settings = bus.read(0x14)?;
  trig_settings = trig_settings | u32::pow(2,24);
  if use_beta {
    trig_settings = trig_settings | u32::pow(2,25);
  }
  bus.write(0x14, trig_settings)?;
  Ok(())
}

pub fn set_configurable_trigger(bus : &mut IPBus, enable : bool) 
  -> Result<(), Box<dyn Error>> {
  if enable {
    info!("Enabling configurable trigger!");
  } else {
    info!("Disabling configurable trigger!");
  }
  let mut trig_settings = bus.read(0x14)?;
  //println!("Got {} trig settings", trig_settings);
  if enable {
    trig_settings = trig_settings | u32::pow(2,31);
  } else {
    trig_settings = trig_settings & !(u32::pow(2,31));
  }
  //println!("Will write {} trig settings", trig_settings);
  bus.write(0x14, trig_settings)?;
  Ok(())
}

pub fn set_inner_tof_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting inner TOF threshold {}!", thresh);
  let mut trig_settings = bus.read(0x14)?;
  trig_settings = trig_settings & 0xffffff00;
  trig_settings = trig_settings | thresh as u32;
  bus.write(0x14, trig_settings)?;
  Ok(())
}

pub fn set_outer_tof_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting outer TOF threshold {}!", thresh);
  let mut trig_settings = bus.read(0x14)?;
  trig_settings = trig_settings & 0xffff00ff;
  trig_settings = trig_settings | ((thresh as u32) << 8);
  bus.write(0x14, trig_settings)?;
  Ok(())
}

pub fn set_total_tof_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting total TOF threshold {}!", thresh);
  let mut trig_settings = bus.read(0x14)?;
  trig_settings = trig_settings & 0xff00ffff;
  trig_settings = trig_settings | ((thresh as u32) << 16);
  bus.write(0x14, trig_settings)?;
  Ok(())
}

pub fn set_cube_side_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting cube side threshold {}!", thresh);
  let mut trig_settings = bus.read(0x15)?;
  trig_settings = trig_settings & 0xffffff00;
  trig_settings = trig_settings | thresh as u32;
  bus.write(0x15, trig_settings)?;
  Ok(())
}

pub fn set_cube_top_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting cube top threshold {}!", thresh);
  let mut trig_settings = bus.read(0x15)?;
  trig_settings = trig_settings & 0xffff00ff;
  trig_settings = trig_settings | ((thresh as u32) << 8);
  bus.write(0x15, trig_settings)?;
  Ok(())
}

pub fn set_cube_bottom_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting cube bottom threshold {}!", thresh);
  let mut trig_settings = bus.read(0x15)?;
  trig_settings = trig_settings & 0xff00ffff;
  trig_settings = trig_settings | ((thresh as u32) << 16);
  bus.write(0x15, trig_settings)?;
  Ok(())
}

pub fn set_cube_corner_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting cube corner threshold {}!", thresh);
  let mut trig_settings = bus.read(0x15)?;
  trig_settings = trig_settings & 0x00ffffff;
  trig_settings = trig_settings | ((thresh as u32) << 24);
  bus.write(0x15, trig_settings)?;
  Ok(())
}

pub fn set_umbrella_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting umbrella threshold {}!", thresh);
  let mut trig_settings = bus.read(0x16)?;
  // first zero out all the bits
  println!("Got trig settings {}", trig_settings);
  trig_settings = trig_settings & 0xffffff00;
  println!("Got trig settings {}", trig_settings);
  trig_settings = trig_settings | thresh as u32;
  println!("Got trig settings {}", trig_settings);
  let dbg = trig_settings & 0x000000ff;
  println!("The threshold is set to {}", dbg);
  bus.write(0x16, trig_settings)?;
  Ok(())
}

pub fn set_cortina_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting cortina threshold {}!", thresh);
  let mut trig_settings = bus.read(0x16)?;
  trig_settings = trig_settings & 0xff00ffff;
  trig_settings = trig_settings | ((thresh as u32) << 16);
  bus.write(0x16, trig_settings)?;
  Ok(())
}

pub fn set_umbcenter_threshold(bus : &mut IPBus, thresh : u8)
  -> Result<(), Box<dyn Error>> {
  info!("Setting Umbrella center threshold {}!", thresh);
  let mut trig_settings = bus.read(0x16)?;
  trig_settings = trig_settings & 0xffff00ff;
  trig_settings = trig_settings | ((thresh as u32) << 8);
  bus.write(0x16, trig_settings)?;
  Ok(())
}


/// 1 Hit on Umbrella && 1 Hit on Cube
pub fn set_umbcube_trigger(bus : &mut IPBus) 
  ->Result<(), Box<dyn Error>> {
  zero_config_trigger_thresholds(bus)?;
  // now set the deired thresholds,
  // because we zero'd out everything, 
  // we just need to enable here.
  set_configurable_trigger(bus,true)?;
  set_umbrella_threshold(bus,1)?;
  set_inner_tof_threshold(bus,1)?;
  Ok(())
}

/// 1 Hit on Umbrella + 1 Hit on cube top
pub fn set_umbcubez_trigger(bus : &mut IPBus) 
  ->Result<(), Box<dyn Error>> {
  zero_config_trigger_thresholds(bus)?;
  set_configurable_trigger(bus,true)?;
  set_umbrella_threshold(bus,1)?;
  set_cube_top_threshold(bus,1)?;
  Ok(())
}

/// 1 Hit on Umbrella && 1 Hit on Cortina && 1 Hit on Cube
pub fn set_umbcorcube_trigger(bus : &mut IPBus) 
  ->Result<(), Box<dyn Error>> {
  zero_config_trigger_thresholds(bus)?;
  set_configurable_trigger(bus,true)?;
  set_umbrella_threshold(bus,1)?;
  set_cortina_threshold(bus, 1)?;
  set_inner_tof_threshold(bus, 1)?;
  Ok(())
}

/// 1 Hit in Cortina && 1 Hit in Cube side
pub fn set_corcubeside_trigger(bus : &mut IPBus) 
  ->Result<(), Box<dyn Error>> {
  zero_config_trigger_thresholds(bus)?;
  set_configurable_trigger(bus,true)?;
  set_cortina_threshold(bus,1)?;
  set_cube_side_threshold(bus,1)?;
  Ok(())
}

/// 1 Hit in Umbrella && 3 Hits in Cube
pub fn set_umb3cube_trigger(bus : &mut IPBus) 
  ->Result<(), Box<dyn Error>> {
  set_configurable_trigger(bus,true)?;
  zero_config_trigger_thresholds(bus)?;
  set_umbrella_threshold(bus,1)?;
  set_inner_tof_threshold(bus,3)?;
  Ok(())
}

/// Zero out the configurable trigger thresholds
pub fn zero_config_trigger_thresholds(bus : &mut IPBus)
  ->Result<() ,Box<dyn Error>> {
  INNER_TOF_THRESH.set(bus,0)?;
  OUTER_TOF_THRESH.set(bus,0)?;
  TOTAL_TOF_THRESH.set(bus,0)?;
  CUBE_SIDE_THRESH.set(bus,0)?; 	
  CUBE_TOP_THRESH.set(bus, 0)?; 	
  CUBE_BOT_THRESH.set(bus, 0)?; 	
  CUBE_CORNER_THRESH.set(bus, 0)?; 	
  UMBRELLA_THRESH.set(bus, 0)?; 	
  UMBRELLA_CENTER_THRESH.set(bus, 0)?;
  Ok(())
}

/// Force a single trigger (just once)
pub fn force_trigger(bus : &mut IPBus)
  -> Result<(), Box<dyn Error>> {
  //println!("==> Generating trigger!");
  bus.write(0x8, 0x1)?;
  Ok(())
}

pub fn use_tiu_aux_link(bus :&mut IPBus, use_it : bool) 
  -> Result<(), Box<dyn Error>> {
  if use_it {
    TIU_USE_AUX_LINK.set(bus, 1)?;
  } else {
    TIU_USE_AUX_LINK.set(bus, 0)?;
  }
  Ok(())
}


pub fn set_fire_bits(bus : &mut IPBus, channel : u8)
  -> Result<(), Box<dyn Error>> {
  if channel < 25 {
    let mut ch = channel as u32;
    ch = ch << channel;
    bus.write(0x101,ch)?;
  } else if channel < 50 {
    let mut ch = channel as u32 - 25;
    ch = ch << channel;
    bus.write(0x102,ch)?;
  } else if channel < 75 {
    let mut ch = channel as u32 - 50;
    ch = ch << channel;
    bus.write(0x103,ch)?;
  } else if channel < 100 {
    let mut ch = channel as u32 - 75;
    ch = ch << channel;
    bus.write(0x104,ch)?;
  } else if channel < 125 {
    let mut ch = channel as u32 - 100;
    ch = ch << channel;
    bus.write(0x105,ch)?;
  } else if channel < 150 {
    let mut ch = channel as u32 - 125;
    ch = ch << channel;
    bus.write(0x106,ch)?;
  } else if channel < 175 {
    let mut ch = channel as u32 - 150;
    ch = ch << channel;
    bus.write(0x107,ch)?;
  } else if channel < 200 {
    let mut ch = channel as u32 - 175;
    ch = ch << channel;
    bus.write(0x107,ch)?;
  }
  Ok(())
}

pub fn fire_ltb(bus : &mut IPBus)
  -> Result<(), Box<dyn Error>> {
  bus.write(0x100,0x1)?;
  Ok(())
}

