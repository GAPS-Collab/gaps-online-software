extern crate liftof_rb;

use std::{thread, time};

use indicatif::{ProgressBar, 
                ProgressStyle};
use tof_dataclasses::events::RBEventMemoryView;
use tof_dataclasses::serialization::search_for_u16;
use tof_dataclasses::serialization::Serialization;

use liftof_rb::control::*;
use liftof_rb::memory::BlobBuffer;
use liftof_rb::memory::RegisterError;
use liftof_rb::memory::map_physical_mem_read;


const UIO1_TRIP : u32 = 66520576;
const UIO2_TRIP : u32 = 66520576;
const UIO0 : &'static str = "/dev/uio0";
const UIO1 : &'static str = "/dev/uio1";
const UIO2 : &'static str = "/dev/uio2";
const SLEEP_AFTER_REG_WRITE : u32 = 1; // sleep time after register write in ms

///! Return the bytes located at the memory
pub fn get_bytestream(addr_space : &str, 
                  addr : u32,
                  len  : usize) -> Result<Vec::<u8>, RegisterError> {

  let blobsize = RBEventMemoryView::SIZE;
  let vec_size = blobsize*len;
  // FIXME - allocate the vector elsewhere and 
  // pass it by reference
  let mut bytestream = Vec::<u8>::with_capacity(vec_size);

  let sz = std::mem::size_of::<u8>();
  let m = match map_physical_mem_read(addr_space, addr, vec_size * sz) {
    Ok(m) => m,
    Err(err) => {
      let error = RegisterError {};
      println!("Failed to mmap: Err={:?}", err);
      return Err(error);
    }
  };
  let p = m.as_ptr() as *const u8;
  (0..vec_size).for_each(|x| unsafe {
    let value = std::ptr::read_volatile(p.offset(x as isize));
    bytestream.push(value); // push is free, since we 
                            // allocated the vector in the 
                            // beginning
  });
  Ok(bytestream)
}


extern crate pretty_env_logger;
#[macro_use] extern crate log;

/// Non-register related constants
const TEMPLATE_BAR_A  : &str = "[{elapsed_precise}] {bar:60.blue/white} {pos:>7}/{len:7} {msg}";
const TEMPLATE_BAR_B  : &str = "[{elapsed_precise}] {bar:60.orange/white} {pos:>7}/{len:7} {msg}";
const TEMPLATE_BAR_EV : &str = "[{elapsed_precise}] {bar:60.red/white} {pos:>7}/{len:7} {msg}";



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
fn get_buff_size(which : &BlobBuffer, buff_start : &mut u32) ->Result<u32, RegisterError> {
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




fn buff_handler(which      : &BlobBuffer,
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
      reset_ram_buffer_occ(&which);
      thread::sleep_ms(SLEEP_AFTER_REG_WRITE);
      let bytestream = get_bytestream(UIO1, buff_start_temp, 10).unwrap();
      let blob_size  = RBEventMemoryView::SIZE;
      let mut a_blob = RBEventMemoryView::new();
      let mut start_pos  = search_for_u16(RBEventMemoryView::HEAD, &bytestream, blob_size*5).unwrap();
      match RBEventMemoryView::from_bytestream(&bytestream, &mut start_pos) {
        Err(err) => {
          error!("Unable to decode RBEventMemoryView! Err {err}");
        }
        Ok(ev) => {
          a_blob = ev;
        }
      }
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
    reset_ram_buffer_occ(&which);
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




fn main() {
  pretty_env_logger::init();
  // some pre-defined time units for 
  // sleeping
  let two_seconds = time::Duration::from_millis(2000);
  let one_milli   = time::Duration::from_millis(1);

  //info!("Setting daq to idle mode");
  //match idle_drs4_daq() {
  //  Ok(_)    => info!("DRS4 set to idle:"),
  //  Err(err) => panic!("Can't set DRS4 to idle!!")
  //}
  //thread::sleep(one_milli);
  //match setup_drs4() {
  //  Ok(_)    => info!("DRS4 setup routine complete!"),
  //  Err(err) => panic!("Failed to setup DRS4!!")
  //}

  
  // get the current cache sizes
  let buf_a = BlobBuffer::A;
  let buf_b = BlobBuffer::B;
  reset_dma().unwrap();
  thread::sleep(one_milli);
  let mut buf_a_start = get_blob_buffer_occ(&buf_a).unwrap();
  let mut buf_b_start = get_blob_buffer_occ(&buf_b).unwrap();
  info!("We got start values for the blob buffers at {buf_a_start} and {buf_b_start}");
  // now we are ready to receive data 
  //info!("Starting daq!");
  //match start_drs4_daq() {
  //  Ok(_)    => info!(".. successful!"),
  //  Err(err) => panic!("DRS4 start failed!")
  //}

  // let go for a few seconds to get a 
  // rate estimate
  println!("getting rate estimate..");
  thread::sleep(two_seconds);
  let mut rate = get_trigger_rate().unwrap();
  println!("Running at a trigger rate of {rate} Hz");
  // the trigger rate defines at what intervals 
  // we want to print out stuff
  // let's print out something apprx every 2
  // seconds
  let n_evts_print : u64 = 2*rate as u64;

  // event loop
  let mut evt_cnt          : u32 = 0;
  let mut last_evt_cnt     : u32 = 0;

  let mut n_events         : u64 = 0;

  let mut skipped_events   : u64 = 0;
  let mut delta_events     : u64 = 0;

  let mut first_iter       = true;
  
  // acquire this many events
  let max_event : u64 = 10000;

  // sizes of the buffers
  //let mut size_a = get_buff_size(&buf_a, &mut buf_a_start).unwrap();
  //let mut size_b = get_buff_size(&buf_b, &mut buf_b_start).unwrap();
  //let mut delta_size_a : u32 = 0;
  //let mut delta_size_b : u32 = 0;

  match enable_trigger() {
    Ok(_) => (),
    Err(err) => error!("Can not enable triggers, Error {err}")
  }
  let mut dma_min = std::u32::MAX;
  let mut dma_max = 0u32;

  let mut buff_a_min_occ : u32 = 4294967295;
  let mut buff_b_min_occ : u32 = 4294967295;
  let mut buff_a_max_occ : u32 = 0;
  let mut buff_b_max_occ : u32 = 0;
  let mut buff_a_occ : u32 = 0;
  let mut buff_b_occ : u32 = 0;
  let buff_a = BlobBuffer::A;
  let buff_b = BlobBuffer::B;
  let mut n_iter = 0;
  loop {
    n_iter += 1;
    evt_cnt = get_event_count().unwrap();
    //if first_iter {
    //  last_evt_cnt = evt_cnt;
    //  first_iter = false;
    //}
    //if evt_cnt == last_evt_cnt {
    //  thread::sleep(one_milli);
    //  continue;
    //}
    buff_a_occ  = get_blob_buffer_occ(&buff_a).unwrap();
    buff_b_occ  = get_blob_buffer_occ(&buff_b).unwrap();
    let dma_ptr = get_dma_pointer().unwrap();
    //println!("{}", dma_ptr);
    if buff_a_occ > buff_a_max_occ {
        buff_a_max_occ = buff_a_occ;
    }
    if buff_a_occ < buff_a_min_occ {
        buff_a_min_occ = buff_a_occ;
    }
    if buff_b_occ > buff_b_max_occ {
        buff_b_max_occ = buff_b_occ;
    }
    if buff_b_occ < buff_b_min_occ {
        buff_b_min_occ = buff_b_occ;
    }
    if dma_ptr > dma_max {
        dma_max = dma_ptr;
    }
    if dma_ptr < dma_min {
        dma_min = dma_ptr;
    }
    // let's do some work
    //buf_a_start = buff_handler(&buf_a, buf_a_start, Some(&bar_a)); 
    //buf_b_start = buff_handler(&buf_b, buf_b_start, Some(&bar_b)); 
    if n_iter % 100000 == 0 {
        println!("New MAX A occ {buff_a_max_occ}");
        println!("New MIN A occ {buff_a_min_occ}");
        println!("New MAX B occ {buff_b_max_occ}");
        println!("New MIN B occ {buff_b_min_occ}");
        println!("New MAX dma ptr {dma_max}");
        println!("New MIN dma ptr {dma_min}");
        println!("Estmated A buff size {}", buff_a_max_occ - buff_a_min_occ);
        println!("Estmated B buff size {}", buff_b_max_occ - buff_b_min_occ);
        println!("----- N iterations {n_iter}"); 
    }
  }
} // end main
