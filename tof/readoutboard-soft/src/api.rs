///! Higher level functions, to deal with events/binary reprentation of it, 
///  configure the drs4, etc.

use std::{thread, time};

// just for fun
use indicatif::{ProgressBar,
                ProgressStyle};

use crate::control::*;
use crate::registers::*;
use crate::memory::*;

use tof_dataclasses::events::blob::BlobData;
use tof_dataclasses::serialization::search_for_u16;


const SLEEP_AFTER_REG_WRITE : u32 = 1; // sleep time after register write in ms


///! Get the blob buffer size from occupancy register
///
///  Read out the occupancy register and compare to 
///  a previously recoreded value. 
///  Everything is u32 (the register can't hold more)
///
///  The size of the buffer can only be defined compared
///  to a start value. If the value rools over, the 
///  size then does not make no longer sense and needs
///  to be updated.
///
///  #Arguments: 
///
pub fn get_buff_size(which : &BlobBuffer, buff_start : &mut u32) ->Result<u32, RegisterError> {
  let mut size : u32;
  let occ = get_blob_buffer_occ(&which)?;
  if *buff_start > occ {
    debug!("The occupancy counter has rolled over!");
    debug!("It reads {occ}");
    //size = occ;
    //*buff_start = occ;
    return Err(RegisterError {});
  } else {
    size  = occ - *buff_start;
  }
  Ok(size)
}





///! FIXME - should become a feature
pub fn setup_progress_bar(msg : String, size : u64, format_string : String) -> ProgressBar {
  let mut bar = ProgressBar::new(size).with_style(
    //ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
    ProgressStyle::with_template(&format_string)
    .unwrap()
    .progress_chars("##-"));
  //);
  bar.set_message(msg);
  //bar.finish_and_clear();
  ////let mut style_found = false;
  //let style_ok = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}");
  //match style_ok {
  //  Ok(_) => { 
  //    style_found = true;
  //  },
  //  Err(ref err)  => { warn!("Can not go with chosen style! Not using any! Err {err}"); }
  //}  
  //if style_found { 
  //  bar.set_style(style_ok.unwrap()
  //                .progress_chars("##-"));
  //}
  bar
}

pub fn buff_handler(which      : &BlobBuffer,
                buff_start : u32,
                prog_bar   : Option<&ProgressBar>) -> u32 {
  
  let mut buff_start_temp = buff_start.clone();
  let mut buff_size : u32;

  match get_buff_size(&which, &mut buff_start_temp) {
    Ok(sz) => buff_size = sz,
    Err(_) => {
      debug!("Buffer {which:?} is full!");
      // the buffer is actually full and needs to be reset
      //switch_ram_buffer();
      //thread::sleep_ms(SLEEP_AFTER_REG_WRITE);
      blob_buffer_reset(&which);
      thread::sleep_ms(SLEEP_AFTER_REG_WRITE);
      let bytestream = get_bytestream(UIO1, buff_start_temp, 10).unwrap();
      let blob_size  = BlobData::SERIALIZED_SIZE;
      let mut a_blob = BlobData::new();
      let mut start_pos  = search_for_u16(BlobData::HEAD, &bytestream, blob_size*5).unwrap();
      a_blob.from_bytestream_experimental(&bytestream, start_pos, true);
      a_blob.print();
      match get_buff_size(&which, &mut buff_start_temp) {
        Ok(sz) => buff_size = sz,
        Err(_) => buff_size = 0
      }
      debug!("Got NEW buffer size of {buff_size} for buff {which:?}");
    }
  }
  trace!("Got buffer size of {buff_size} for buff {which:?}");
  if buff_size > UIO1_TRIP {
    debug!("Buff {which:?} tripped");  
    // reset the buffers
    //switch_ram_buffer();
    //thread::sleep_ms(SLEEP_AFTER_REG_WRITE);
    blob_buffer_reset(&which);
    thread::sleep_ms(SLEEP_AFTER_REG_WRITE);
    // get the new size after reset
    match get_buff_size(&which, &mut buff_start_temp) {
      Ok(sz) => buff_size = sz,
      Err(_) => buff_size = 0
    }
    debug!("Got NEW buffer size of {buff_size} for buff {which:?}");
  }
  match prog_bar {
    Some(bar) => bar.set_position(buff_size as u64),
    None      => () 
  }
  buff_start_temp
}


///! Prepare the whole readoutboard for data taking.
///
///  This sets up the drs4 and clears the memory of 
///  the data buffers.
///
/// 
pub fn setup_drs4() -> Result<(), RegisterError> {

  let buf_a = BlobBuffer::A;
  let buf_b = BlobBuffer::B;

  let one_milli   = time::Duration::from_millis(1);
  // DAQ defaults
  //let num_samples     : u32 = 3000;
  //let duration        : u32 = 0; // Default is 0 min (=> use events) 
  //let roi_mode        : u32 = 1;
  //let transp_mode     : u32 = 1;
  //let run_mode        : u32 = 0;
  //let run_type        : u32 = 0;        // 0 -> Events, 1 -> Time (default is Events)
  //let config_drs_flag : u32 = 1; // By default, configure the DRS chip
  // run mode info
  // 0 = free run (must be manually halted), ext. trigger
  // 1 = finite sample run, ext. trigger
  // 2 = finite sample run, software trigger
  // 3 = finite sample run, software trigger, random delays/phase for timing calibration
  let spike_clean     : bool = true;
  let read_ch9        : u32  = 1;

  // before we do anything, set the DRS in idle mode 
  // and set the configure bit
  idle_drs4_daq()?;
  thread::sleep(one_milli);
  set_drs4_configure()?;
  thread::sleep(one_milli);

  // Sanity checking
  let max_samples     : u32 = 65000;
  let max_duration    : u32 = 1440; // Minutes in 1 day

  reset_daq()?;
  thread::sleep(one_milli);
  
  reset_dma()?;
  thread::sleep(one_milli);
  clear_dma_memory()?;
  thread::sleep(one_milli);
  
  
  // for some reason, sometimes it 
  // takes a bit until the blob
  // buffers reset. Let's try a 
  // few times
  info!("Resetting blob buffers..");
  for _ in 0..5 {
    blob_buffer_reset(&buf_a);
    thread::sleep(one_milli);
    blob_buffer_reset(&buf_b);
    thread::sleep(one_milli);
  }

  // register 04 contains a lot of stuff:
  // roi mode, busy, adc latency
  // sample  count and spike removal
  let spike_clean_enable : u32 = 4194304; //bit 22
  if spike_clean {
    let mut value = read_reg(UIO0, 0x40).unwrap();  
    value = value | spike_clean_enable;
    write_reg(UIO0, 0x40, value);
    thread::sleep(one_milli);
  }
  
  set_readout_all_channels_and_ch9();
  thread::sleep(one_milli);
  set_master_trigger_mode();
  thread::sleep(one_milli);
  Ok(())
}



