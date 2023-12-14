//! Convenience functions to read/write
//!  the various control registers
//!
//!  
//!  For the mapping of registers/addresses, 
//!  see `registers.rs`
//!

use crate::registers::*;
use crate::memory::*;

use std::time::Duration;
use std::thread;

extern crate liftof_lib;

/// write header only packets when the drs is busyu
pub fn enable_evt_fragments() -> Result<(), RegisterError> {
  trace!("Enable event fragment writing!");
  write_control_reg(WRITE_EVENTFRAGMENT, 1)?;
  Ok(())
}

/// use the random self trigger
pub fn set_self_trig_rate(rate : u32) -> Result<(), RegisterError> {
  warn!("Setting self trigger rate, writing register {}", TRIG_GEN_RATE);
  write_control_reg(TRIG_GEN_RATE, rate)?;
  Ok(())
}

/// do not write header only packets when the drs is busyu
pub fn disable_evt_fragments() -> Result<(), RegisterError> {
  trace!("Disable event fragment writing!");
  write_control_reg(WRITE_EVENTFRAGMENT, 0)?;
  Ok(())
}

/// enable triggering
pub fn enable_trigger() -> Result<(), RegisterError> {
  trace!("Enable triggers!");
  write_control_reg(TRIGGER_ENABLE, 1)?;
  Ok(())
}

/// stop all triggers
pub fn disable_trigger() -> Result<(), RegisterError> {
  trace!("Enable triggers!");
  write_control_reg(TRIGGER_ENABLE, 0)?;
  Ok(())
}

/// Reset the board and prepare for a new run
///
/// Procedure as discussed in 12/2023
/// Trigger disable (0 to 0x11C[0])
/// Set desired trigger mode  (1 to 0x114[0] for MTB trigger or 0 to 0x114[0] for software trigger)
/// Soft reset (1 to 0x70[0])
/// Check for soft reset done (read 0x74[15])
/// Trigger enable (1 to 0x11C[0])
pub fn soft_reset_board() -> Result<(), RegisterError> {
  trace!("Initialize soft reset procedure!");
  let eight_cycles = Duration::from_micros(4);
  write_control_reg(SOFT_RESET, 0x1)?;
  thread::sleep(eight_cycles);
  while !soft_reset_done()? {
    thread::sleep(eight_cycles);
  }
  Ok(())
}

/// Check if the soft reset procedure has finished
pub fn soft_reset_done() -> Result<bool, RegisterError> {
  let mask : u32 = 1 << 15;
  let value = read_control_reg(SOFT_RESET_DONE)?;
  return Ok((value & mask) > 0)
}


/// Start DRS4 data acquistion
pub fn start_drs4_daq() -> Result<(), RegisterError> {
  trace!("SET DRS4 START");
  write_control_reg(DRS_START, 1)?;
  Ok(())
}


/// Put the daq in idle state, that is stop data taking
pub fn idle_drs4_daq() -> Result<(), RegisterError> {
  trace!("SET DRS4 IDLE");
  write_control_reg(DRS_REINIT, 1)?;
  Ok(())
}

/// Get the blob buffer occupancy for one of the two buffers
///  
/// This is a bit tricky. This will continuously change, as 
/// the DRS4 is writing into the memory. At some point, it 
/// will be full, not changing it's value anymore. At that 
/// point, if set, the firmware has switched automatically 
/// to the other buffer. 
///
/// Also, it will only read something like zero if the 
/// DMA has completly been reset (calling dma_reset).
/// 
/// # Arguments
///
/// * which : select the blob buffer to query
///
///
pub fn get_blob_buffer_occ(which : &BlobBuffer) -> Result<u32, RegisterError> {
  let address = match which {
    BlobBuffer::A => RAM_A_OCCUPANCY,
    BlobBuffer::B => RAM_B_OCCUPANCY,
  };

  let value = read_control_reg(address)?;
  Ok(value)
}

/// Check if teh TRIGGER_ENABLE register is set
pub fn get_triggers_enabled() -> Result<bool, RegisterError> {
  let value = read_control_reg(TRIGGER_ENABLE)?;
  Ok(value > 0)
}

/// FIXME
pub fn get_dma_pointer() -> Result<u32, RegisterError> {
  let value = read_control_reg(DMA_POINTER)?;
  Ok(value)
}

/// Reset the DMA memory (blob data) and write 0s
pub fn clear_dma_memory() -> Result<(), RegisterError> {
  trace!("SET DMA CLEAR");
  write_control_reg(DMA_CLEAR, 1)?;  
  // the reset takes 8 clock cycles at 33 MHz (about 3.4 micro)
  let eight_cycles = Duration::from_micros(4);
  thread::sleep(eight_cycles);
  Ok(())
}


/// Reset means, the memory can be used again, but it does not mean it 
/// clears the memory.
///
/// The writing into the memory thus can start anywhere in memory (does 
/// not have to be from 0)
pub fn reset_ram_buffer_occ(which : &BlobBuffer) -> Result<(), RegisterError> {
  match which { 
    BlobBuffer::A => write_control_reg(RAM_A_OCC_RST, 0x1)?,
    BlobBuffer::B => write_control_reg(RAM_B_OCC_RST, 0x1)?
  };
  // the reset takes 8 clock cycles at 33 MHz (about 3.4 micro)
  let eight_cycles = Duration::from_micros(4);
  thread::sleep(eight_cycles);
  Ok(())
}

/// Get the recorded triggers by the DRS4
pub fn get_trigger_rate() -> Result<u32, RegisterError> {
  let value = read_control_reg(TRIGGER_RATE)?;
  Ok(value)
}

/// Get the rate of the lost triggers by the DRS4
pub fn get_lost_trigger_rate() -> Result<u32, RegisterError> {
  let value = read_control_reg(LOST_TRIGGER_RATE)?;
  Ok(value)
}

/// Get the event counter from the DRS4
///
/// The event counter is NOT the event id comming from 
/// the master trigger. It is simply the number of 
/// events observed since the last reset.
///
pub fn get_event_count() -> Result<u32, RegisterError> {
  let value = read_control_reg(CNT_EVENT)?;
  Ok(value)
}

/// Get the event counter as sent from the MTB
pub fn get_event_count_mt() -> Result<u32, RegisterError> {
  let value = read_control_reg(MT_EVENT_CNT)?;
  Ok(value)
}

/// Get the rate as sent from the MTB
pub fn get_event_rate_mt() -> Result<u32, RegisterError> {
  let value = read_control_reg(MT_TRIG_RATE)?;
  Ok(value)
}

/// Get the lost events event counter from the DRS4
pub fn get_lost_event_count() -> Result<u32, RegisterError> {
  let value = read_control_reg(CNT_LOST_EVENT)?;
  Ok(value)
}

/// This simply sets the configure bit.
///
/// Unclear what it actually does.
/// FIXME
pub fn set_drs4_configure() -> Result<(), RegisterError> {
  trace!("SET DRS4 CONFIGURE");
  write_control_reg(DRS_CONFIGURE, 1)?;
  Ok(())
}

/// Force a trigger
///
/// _If I understand it correctly, this is a single trigger_
///
pub fn trigger() -> Result<(), RegisterError> {
  //warn!("Setting force trigger mode!");
  write_control_reg(FORCE_TRIG, 1)?;
  Ok(())
}

/// Reset of the internal event counter
///
///  This is NOT the event id.
pub fn reset_drs_event_ctr() -> Result<(), RegisterError> {
  trace!("SET DRS4 EV CNT RESET");
  write_control_reg(CNT_RESET, 1)?;
  Ok(())
}

pub fn reset_daq() -> Result<(), RegisterError> {
  trace!("SET DAQ RESET");
  write_control_reg(DAQ_RESET, 1)?;
  Ok(())
}

pub fn reset_drs() -> Result<(), RegisterError> {
  trace!("SET DRS RESET");
  write_control_reg(DRS_REINIT, 1)?;
  Ok(())
}

///! Resets the DMA state machine.
pub fn reset_dma() -> Result<(), RegisterError> {
  trace!("SET DMA RESET");
  write_control_reg(DMA_RESET, 1)?;
  // the reset takes 8 clock cycles at 33 MHz (about 3.4 micro)
  let eight_cycles = Duration::from_micros(4);
  thread::sleep(eight_cycles);
  Ok(())
}


///! Toggle between the data buffers A and B
pub fn switch_ram_buffer() -> Result<(), RegisterError> {
  trace!("SET DMA DATA BUFF TOGGLE");
  write_control_reg(TOGGLE_RAM, 1)?;
  Ok(())
}

///! The device DNA is a unique identifier
pub fn get_device_dna() -> Result<u64, RegisterError> {
  let lsb = read_control_reg(DNA_LSBS)?;
  let msb = read_control_reg(DNA_MSBS)?;
  let mut value : u64 = 0;
  value = value | (msb as u64) << 32;
  value = value | lsb as u64;
  Ok(value)
}


/// Enable the readout of all channels + the 9th channel
pub fn set_readout_all_channels_and_ch9() -> Result<(), RegisterError> {
  warn!("This might be buggy!");
  let all_channels : u32 = 511;
  let ch_9         : u32 = 512;
  let value = all_channels | ch_9;
  trace!("SET DRS4 READOUT MASK");
  write_control_reg(READOUT_MASK, value)?;
  Ok(())
}

/// Enable active channels by not touching the ch9 bits
pub fn set_active_channel_mask(ch_mask : u8) -> Result<(), RegisterError> {
  let mut value   = read_control_reg(READOUT_MASK)?;
  // FIXME - do debug! instead.
  println!("==> Got current channel mask! {value}");
  let ch9_part     = value & 0xFF00; // set all ch to 0;
  value            = ch9_part | ch_mask as u32;
  write_control_reg(READOUT_MASK, value)?;
  println!("==> Wrote {value} to channel mask register!");
  Ok(())
}

pub fn set_active_channel_mask_with_ch9(ch_mask : u32) -> Result<(), RegisterError> {
  let ch_9  : u32 = 256;
  let value = ch_mask | ch_9;
  write_control_reg(READOUT_MASK, value)?;
  Ok(())
}

/// Enable the master trigger mode
pub fn set_master_trigger_mode() -> Result<(), RegisterError> {
  trace!("SET DRS4 MT MODE");
  write_control_reg(MT_TRIGGER_MODE, 1)?;
  Ok(())
}

/// Disable the master trigger
pub fn disable_master_trigger_mode() -> Result<(), RegisterError> {
  warn!("Disabeling master trigger mode");
  write_control_reg(MT_TRIGGER_MODE, 0)?;
  Ok(())
}


///! Get the board ID from the control registers.
pub fn get_board_id() -> Result<u32, RegisterError> { 
  let board_id = read_control_reg(BOARD_ID)?;
  Ok(board_id)
}

///! Get the board ID from the control registers.
pub fn get_board_id_string() -> Result<String, RegisterError> { 
  let board_id = get_board_id()?;
  let board_id_string = liftof_lib::to_board_id_string(board_id);
  Ok(board_id_string)
}


