//mod registers;
//mod memory;
//mod control;
//mod api;

extern crate liftof_rb;

use std::{thread, time};

use indicatif::{MultiProgress,
                ProgressBar, 
                ProgressStyle};
use tof_dataclasses::events::blob::BlobData;
use tof_dataclasses::serialization::search_for_u16;

use liftof_rb::api::*;
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

/// Return the bytes located at the memory
pub fn get_bytestream(addr_space : &str, 
                  addr : u32,
                  len  : usize) -> Result<Vec::<u8>, RegisterError> {

  let blobsize = BlobData::SERIALIZED_SIZE;
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

  info!("Setting daq to idle mode");
  match idle_drs4_daq() {
    Ok(_)    => info!("DRS4 set to idle:"),
    Err(err) => panic!("Can't set DRS4 to idle!!")
  }
  thread::sleep(one_milli);
  match setup_drs4() {
    Ok(_)    => info!("DRS4 setup routine complete!"),
    Err(err) => panic!("Failed to setup DRS4!!")
  }

  
  // get the current cache sizes
  let buf_a = BlobBuffer::A;
  let buf_b = BlobBuffer::B;
  reset_dma().unwrap();
  thread::sleep(one_milli);
  let mut buf_a_start = get_blob_buffer_occ(&buf_a).unwrap();
  let mut buf_b_start = get_blob_buffer_occ(&buf_b).unwrap();
  info!("We got start values for the blob buffers at {buf_a_start} and {buf_b_start}");
  // now we are ready to receive data 
  info!("Starting daq!");
  match start_drs4_daq() {
    Ok(_)    => info!(".. successful!"),
    Err(err) => panic!("DRS4 start failed!")
  }

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

  // set up some progress bars, so we 
  // can see what is going on 
  let multi_bar = MultiProgress::new();
  let bar_a  = multi_bar.add(setup_progress_bar(String::from("buff A"), UIO1_TRIP as u64, String::from(TEMPLATE_BAR_A)));  
  let bar_b  = multi_bar.insert_after(&bar_a,setup_progress_bar(String::from("buff B"), UIO2_TRIP as u64, String::from(TEMPLATE_BAR_B))); 
  let mut bar_ev = multi_bar.insert_after(&bar_b,setup_progress_bar(String::from("events"), max_event, String::from(TEMPLATE_BAR_EV)));         

  match enable_trigger() {
    Ok(_) => (),
    Err(err) => error!("Can not enable triggers, Error {err}")
  }

  loop {
    evt_cnt = get_event_count().unwrap();
    if first_iter {
      last_evt_cnt = evt_cnt;
      first_iter = false;
    }
    if evt_cnt == last_evt_cnt {
      thread::sleep(one_milli);
      continue;
    }
    // let's do some work
    buf_a_start = buff_handler(&buf_a, buf_a_start, Some(&bar_a)); 
    buf_b_start = buff_handler(&buf_b, buf_b_start, Some(&bar_b)); 
    //size_a = get_buff_size(&buf_a, &mut buf_a_start).unwrap();
    //size_b = get_buff_size(&buf_b, &mut buf_b_start).unwrap();
    //bar_a.set_position(size_a as u64);
    //bar_b.set_position(size_b as u64);
    //delta_size_a = size_a - delta_size_a;



    delta_events = (evt_cnt - last_evt_cnt) as u64;
    if delta_events > 1 {
      skipped_events += delta_events;
    }
    
    n_events += 1;
    bar_ev.inc(delta_events);   
    // exit loop on n event basis
    if n_events > max_event {
      idle_drs4_daq();
      println!("We skipped {skipped_events} events");
      thread::sleep(one_milli);
      bar_ev.finish();
      break;
    }

    //if n_events % n_evts_print == 0 {
    //  println!("Current event count {n_events}");
    //  println!("We skipped {skipped_events} events");
    //}
    
    //println!("Got {evt_cnt} event!");
    

    last_evt_cnt = evt_cnt;
  }
} // end main


//fn main() {
//  pretty_env_logger::init();
//
//  // some pre-defined time units for 
//  // sleeping
//  let two_seconds = time::Duration::from_millis(2000);
//  let one_milli   = time::Duration::from_millis(1);
//  
//  idle_drs4_daq();
//  thread::sleep(one_milli);
//  let buf_a = BlobBuffer::A;
//  let buf_b = BlobBuffer::B;
//  thread::sleep(one_milli);
//  let mut buf_a_occ        = get_blob_buffer_occ(&buf_a).unwrap();
//  let mut buf_b_occ        = get_blob_buffer_occ(&buf_b).unwrap();
//  println! ("Size of blob buffer A is {buf_a_occ}");
//  println! ("Size of blob buffer B is {buf_b_occ}");
//  let mut buf_a_size : u32 = 0;
//  let mut buf_b_size : u32 = 0;
//  let mut buf_a_size_last : u32 = 0;
//  let mut buf_b_size_last : u32 = 0;
//  clear_dma_memory();
//  reset_drs_event_ctr();
//  setup_drs4();
//
//  let mut buf_a_start      = get_blob_buffer_occ(&buf_a).unwrap();
//  let mut buf_b_start      = get_blob_buffer_occ(&buf_b).unwrap();
//  println!("We have the buffers start at {buf_a_start} B: {buf_b_start}");
//  thread::sleep(two_seconds); 
//
//  let mut bar_a = ProgressBar::new(UIO1_TRIP as u64);
//  bar_a.set_message("Buf A");
//  bar_a.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
//    .unwrap()
//    .progress_chars("##-"));
//  let mut bar_b = ProgressBar::new(UIO2_TRIP as u64);
//  bar_b.set_message("Buf B");
//  bar_b.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.red/green} {pos:>7}/{len:7} {msg}")
//    .unwrap()
//    .progress_chars("##-"));
//
//  println! ("Size of blob buffer A is {buf_a_occ}");
//  println! ("Size of blob buffer B is {buf_b_occ}");
//  //println! ("Value of dma_ptr is {dma_ptr}");
//
//  start_drs4_daq();
//  println!("Will start daq..");
//  thread::sleep(2*two_seconds);
//  start_drs4_daq();
//  println!("..done");
//  //loop {
//  //  buf_a_occ      = get_blob_buffer_occ(&buf_a).unwrap();
//  //  buf_b_occ      = get_blob_buffer_occ(&buf_b).unwrap();
//  //  let mut dma_ptr        = get_dma_pointer().unwrap();
//  //  reset_ram_buffer_occ(&buf_a);
//  //  reset_ram_buffer_occ(&buf_b);
//  //  buf_a_occ      = get_blob_buffer_occ(&buf_a).unwrap();
//  //  buf_b_occ      = get_blob_buffer_occ(&buf_b).unwrap();
//  //  dma_ptr        = get_dma_pointer().unwrap();
//  //  println! ("Size of blob buffer A is {buf_a_occ}");
//  //  println! ("Size of blob buffer B is {buf_b_occ}");
//  //  println! ("Value of dma_ptr is {dma_ptr}");
//  //  let trigger        = get_trigger_rate().unwrap();
//  //  let lost_trg       = get_lost_trigger_rate().unwrap();
//  //  let event_cnt      = get_event_count().unwrap();
//  //  let lost_event_cnt = get_lost_event_count().unwrap();
//  //  let device_dna     = get_device_dna().unwrap();
//  //  let blob_size      = BlobData::SERIALIZED_SIZE;
//  //  // let's get the bytes for the first 100 blobs
//  //  let bytestream     = get_bytestream(UIO1, 0x0, 100).unwrap();
//  //  let mut a_blob = BlobData::new();
//  //  //let mut start_pos  = search_for_u16(BlobData::HEAD, &bytestream, blob_size*500).unwrap();
//  //  //a_blob.from_bytestream_experimental(&bytestream, start_pos, true);
//  //  //a_blob.print();
//  //  //for n in (blob_size - 200)..(blob_size + 200) {
//  //  //    let foo = bytestream[n];
//  //  //    println!("{foo}");
//  //  //}
//
//  //  //let mut pos = start_pos + blob_size - 200;
//  //  //for n in 0..10 {
//  //  //  println!("Blob {n}");
//  //  //  a_blob.from_bytestream_experimental(&bytestream, pos, true);
//  //  //  let end_pos = search_for_u16(BlobData::TAIL, &bytestream, pos + blob_size -10).unwrap();
//  //  //  let size = end_pos - pos;
//  //  //  println!("Found blob of size {size}");
//  //  //  a_blob.print();
//  //  //  pos += blob_size;
//  //  //}
//
//  //  println! ("Size of blob buffer A is {buf_a_occ}");
//  //  println! ("Size of blob buffer B is {buf_b_occ}");
//  //  println! ("We got {trigger} trg rate and {lost_trg} lost trg rate");
//  //  println! ("We saw {event_cnt} events and lost {lost_event_cnt}"); 
//  //  println! ("The device has dna {device_dna}");
//  //  break;
//  //}
//  //let now = time::Instant::now();
//
//  let mut last_event : u32 = get_event_count().unwrap();
//  //reset_ram_buffer_occ(&buf_a);
//  //reset_ram_buffer_occ(&buf_b);
// 
//  let approximate_blob_size :f32 = 18000.0;
//  let maxevent : u32 = 10000;
//  let mut total_events : u32 = 0;
//  let bar_ev = ProgressBar::new(maxevent as u64);
//  bar_ev.set_message("events");
//  bar_ev.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.white} {pos:>7}/{len:7} {msg}")
//    .unwrap()
//    .progress_chars("**-"));
//  loop {
//    let mut this_event = get_event_count().unwrap();
//    if this_event == last_event {
//        continue;
//    }
//    let n_events = this_event as i32 - last_event as i32;
//    if n_events > 1 {
//      println!("Warn! We skipped events... {n_events}");
//    }
//    bar_ev.inc(n_events as u64);
//    total_events += n_events as u32;
//    last_event = this_event;
//    
//    buf_a_occ        = get_blob_buffer_occ(&buf_a).unwrap();
//    buf_b_occ        = get_blob_buffer_occ(&buf_b).unwrap();
//    buf_a_size = buf_a_occ - buf_a_start;
//    buf_b_size = buf_b_occ - buf_b_start;
//
//
//    //println!("got ahead {n_events}"); 
//    //let dma_ptr        = get_dma_pointer().unwrap();
//    println! ("Size of blob buffer A is {buf_a_size:.4}");
//    println! ("Size of blob buffer B is {buf_b_size:.4}");
//    if buf_a_size >= UIO1_TRIP {
//      println!("Buff A tripped!");
//      //println!("Switching buffers!");
//      println! ("Size of blob buffer A is {buf_a_size:.4}");
//      println! ("Size of blob buffer B is {buf_b_size:.4}");
//      switch_ram_buffer();
//      reset_ram_buffer_occ(&buf_a);
//      buf_a_start = get_blob_buffer_occ(&buf_a).unwrap();
//      bar_a.finish();
//      bar_a = ProgressBar::new(UIO1_TRIP as u64);
//      bar_a.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
//        .unwrap()
//        .progress_chars("##-"));
//
//      //buf_a_size = (buf_a_occ as f32 - buf_a_start as f32)/approximate_blob_size;
//      continue;
//    }
//    if buf_b_size >= UIO2_TRIP {
//      println!("Buff B tripped!");
//      //println!("Switching buffers!");
//      println! ("Size of blob buffer A is {buf_a_size:.4}");
//      println! ("Size of blob buffer B is {buf_b_size:.4}");
//      switch_ram_buffer();
//      reset_ram_buffer_occ(&buf_b);
//      buf_b_start = get_blob_buffer_occ(&buf_b).unwrap();
//      bar_b.finish();
//      bar_b = ProgressBar::new(UIO2_TRIP as u64);
//      bar_b.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
//        .unwrap()
//        .progress_chars("##-"));
//      //start_drs4_daq();
//      //buf_b_size = (buf_b_occ as f32 - buf_b_start as f32)/approximate_blob_size;
//      continue;
//    }
//    //buf_a_size = (buf_a_occ as f32 - buf_a_start as f32)/approximate_blob_size;
//    //buf_b_size = (buf_b_occ as f32 - buf_b_start as f32)/approximate_blob_size;
//    if (buf_a_size - buf_a_size_last) > 0 {
//      bar_a.inc((buf_a_size - buf_a_size_last) as u64);
//    }
//    if (buf_b_size - buf_b_size_last) > 0 {
//      bar_b.inc((buf_b_size - buf_b_size_last) as u64);
//    }
//    buf_a_size_last = buf_a_size;
//    buf_b_size_last = buf_b_size;
//
//    //println! ("Size of blob buffer A is {buf_a_size:.4}");
//    //println! ("Size of blob buffer B is {buf_b_size:.4}");
//    //println! ("We saw {this_event} events!"); 
//    thread::sleep(one_milli);
//    if (total_events >= maxevent) {
//      bar_ev.finish();
//      idle_drs4_daq();
//      break;
//    }
//    continue;
//
//    //println! ("Size of blob buffer A is {buf_a_size:.4}");
//    //println! ("Size of blob buffer B is {buf_b_size:.4}");
//    //println! ("Value of dma_ptr is {dma_ptr}");
//    //let event_cnt      = get_event_count().unwrap();
//    //let lost_event_cnt = get_lost_event_count().unwrap();
//    //println! ("We saw {this_event} events and lost {lost_event_cnt}"); 
//    ////switch_ram_buffer();
//    //println! ("-----");
//    ////thread::sleep(two_seconds);
//    //// effectively limit the rate to 1kHz
//    thread::sleep(one_milli);
//  }
  //dump_mem::<u8>(UIO1,0x0, 1000);

//}
