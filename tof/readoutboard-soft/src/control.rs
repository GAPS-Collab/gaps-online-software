///! Convenience functions to set and read
///  from the various control registers
//

use crate::registers::*;
use crate::memory::*;

/// Start DRS4 data acquistion
pub fn start_drs4_daq() -> Result<(), RegisterError> {
  write_reg(UIO0, DRS_START, 1)?;
  Ok(())
}


///! Put the daq in idle state, that is stop data taking
pub fn idle_drs4_daq() -> Result<(), RegisterError> {
  write_reg(UIO0, DRS_REINIT, 1)?;
  Ok(())
}

//**********************************
// Convenience functions to read
// out the registers


///! Get the blob buffer occupancy for one of the two buffers
///  
///  This is a bit tricky. This will continuously change, as 
///  the DRS4 is writing into the memory. At some point, it 
///  will be full, not changing it's value anymore. At that 
///  point, if set, the firmware has switched automatically 
///  to the other buffer. 
///
///  Also, it will only read something like zero if the 
///  DMA has completly been reset (calling dma_reset).
///
pub fn get_blob_buffer_occ(which : &BlobBuffer) -> Result<u32, RegisterError> {
  let address = match which {
    BlobBuffer::A => RAM_A_OCCUPANCY,
    BlobBuffer::B => RAM_B_OCCUPANCY,
  };

  let value = read_reg(UIO0, address)?;
  Ok(value)
}

pub fn get_dma_pointer() -> Result<u32, RegisterError> {
  let value = read_reg(UIO0, DMA_POINTER)?;
  Ok(value)
}

///! Reset the DMA memory (blob data) and write 0s
pub fn clear_dma_memory() -> Result<(), RegisterError> {
  write_reg(UIO0, DMA_CLEAR, 1)?;  
  Ok(())
}


///! Reset means, the memory can be used again, but it does not mean it 
///  clears the memory.
///
///  The writing into the memory thus can start anywhere in memory (does 
///  not have to be from 0)
pub fn blob_buffer_reset(which : &BlobBuffer) -> Result<(), RegisterError> {
  match which { 
    BlobBuffer::A => write_reg(UIO0, RAM_A_OCC_RST, 0x1)?,
    BlobBuffer::B => write_reg(UIO0, RAM_B_OCC_RST, 0x1)?
  };
  Ok(())
}

///! Get the recorded triggers by the DRS4
pub fn get_trigger_rate() -> Result<u32, RegisterError> {
  let value = read_reg(UIO0, TRIGGER_RATE)?;
  Ok(value)
}

///! Get the rate of the lost triggers by the DRS4
pub fn get_lost_trigger_rate() -> Result<u32, RegisterError> {
  let value = read_reg(UIO0, LOST_TRIGGER_RATE)?;
  Ok(value)
}

///! Get the event counter from the DRS4
///
///  The event counter is NOT the event id comming from 
///  the master trigger. It is simply the number of 
///  events observed since the last reset.
///
pub fn get_event_count() -> Result<u32, RegisterError> {
  let value = read_reg(UIO0, CNT_EVENT)?;
  Ok(value)
}


///! Get the lost events event counter from the DRS4
pub fn get_lost_event_count() -> Result<u32, RegisterError> {
  let value = read_reg(UIO0, CNT_LOST_EVENT)?;
  Ok(value)
}


pub fn set_drs4_configure() -> Result<(), RegisterError> {
  write_reg(UIO0, DRS_CONFIGURE, 1);
  Ok(())
}



pub fn reset_drs_event_ctr() -> Result<(), RegisterError> {
  write_reg(UIO0, CNT_RESET, 1)?;
  Ok(())
}

pub fn reset_daq() -> Result<(), RegisterError> {
  write_reg(UIO0, DAQ_RESET, 1)?;
  Ok(())
}

pub fn reset_dma() -> Result<(), RegisterError> {
  write_reg(UIO0, DMA_RESET, 1)?;
  Ok(())
}

pub fn switch_ram_buffer() -> Result<(), RegisterError> {
  write_reg(UIO0, TOGGLE_RAM, 1)?;
  Ok(())
}

///! The device DNA is a unique identifier
pub fn get_device_dna() -> Result<u64, RegisterError> {
  let lsb = read_reg(UIO0, DNA_LSBS)?;
  let msb = read_reg(UIO0, DNA_MSBS)?;
  let mut value : u64 = 0;
  value = value | (msb as u64) << 32;
  value = value | lsb as u64;
  Ok(value)
}

pub fn set_readout_all_channels_and_ch9() -> Result<(), RegisterError> {

  let all_channels : u32 = 511;
  let ch_9         : u32 = 512;
  let value = all_channels | ch_9;
  write_reg(UIO0, READOUT_MASK, value);
  Ok(())
}

pub fn set_master_trigger_mode() -> Result<(), RegisterError> {
  write_reg(UIO0, MT_TRIGGER_MODE, 1)?;
  Ok(())
}




