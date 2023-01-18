//! Higher level functions, to deal with events/binary reprentation of it, 
//!  configure the drs4, etc.

use std::collections::HashMap;

use std::{thread, time};
use crossbeam_channel::{Sender,
                        Receiver};

// just for fun
use indicatif::ProgressBar;
//use indicatif::ProgressStyle;

use crate::control::*;
use crate::memory::*;
use tof_dataclasses::commands::*;

use tof_dataclasses::events::blob::{BlobData,
                                    RBEventPayload};
use tof_dataclasses::serialization::search_for_u16;
use tof_dataclasses::commands::{TofCommand,
                                TofResponse,
                                TofOperationMode};
//use tof_dataclasses::threading::ThreadPool;

use time::Duration;
 
pub const HEARTBEAT : u64 = 5; // heartbeat in s

const SLEEP_AFTER_REG_WRITE : u32 = 1; // sleep time after register write in ms
const DMA_RESET_TRIES : u8 = 10;   // if we can not reset the DMA after this number
                                   // of retries, we'll panic!
const SAVE_RESTART_TRIES : u8 = 5; // if we are not successfull, to get it going, 
                                   // panic
/// Little helper
fn debug_and_ok() -> Result<(), RegisterError> {
  debug!("Raised RegisterError!");
  Ok(())
}

/// Send out a signal periodically
/// to all the threads. 
/// If they don't answer in timely 
/// manner, call a doctor.
fn heartbeat() {
  let mut now = time::Instant::now();
  loop {
    if now.elapsed().as_secs() >= HEARTBEAT {
      //FIXME
      println!("BEAT");
      now = time::Instant::now();
    }
  }
}


/// Somehow, it is not always successful to reset 
/// the DMA and the data buffers. Let's try an 
/// aggressive scheme and do it several times.
/// If we fail, something is wrong and we panic
fn reset_data_memory_aggressively() {
  let one_milli = time::Duration::from_millis(1);
  let five_milli = time::Duration::from_millis(5);
  let buf_a = BlobBuffer::A;
  let buf_b = BlobBuffer::B;
  let mut n_tries : u8 = 0;
  
  for _ in 0..DMA_RESET_TRIES {
    match reset_dma() {
      Ok(_)    => (),
      Err(err) => {
        debug!("Resetting dma failed, err {:?}", err);
        thread::sleep(five_milli);
        continue;
      }
    }
    thread::sleep(one_milli);
  }
  let mut buf_a_occ = UIO1_MAX_OCCUPANCY;
  let mut buf_b_occ = UIO2_MAX_OCCUPANCY;
  match get_blob_buffer_occ(&buf_a) {
    Err(_) => debug!("Error reseting blob buffer A"),
    Ok(val)  => {
      buf_a_occ = val;
    }
  }
  thread::sleep(one_milli);
  match get_blob_buffer_occ(&buf_b) {
    Err(_) => debug!("Error reseting blob buffer B"),
    Ok(val)  => {
      buf_b_occ = val;
    }
  }
  thread::sleep(one_milli);
  while buf_a_occ != UIO1_MIN_OCCUPANCY {
    blob_buffer_reset(&buf_a).or_else(|_| debug_and_ok() ); 
    thread::sleep(five_milli);
    match get_blob_buffer_occ(&buf_a) {
      Err(_) => debug!("Error reseting blob buffer A"),
      Ok(val)  => {
        buf_a_occ = val;
        thread::sleep(five_milli);
        n_tries += 1;
        if n_tries == DMA_RESET_TRIES {
          panic!("We were unable to reset DMA and the data buffers!");
        }
        continue;
      }
    }
  }
  n_tries = 0;
  while buf_b_occ != UIO2_MIN_OCCUPANCY {
    blob_buffer_reset(&buf_b).or_else(|_| debug_and_ok());
    match get_blob_buffer_occ(&buf_b) {
      Err(_) => debug!("Error reseting blob buffer B"),
      Ok(val)  => {
        buf_b_occ = val;
        thread::sleep(five_milli);
        n_tries += 1;
        if n_tries == DMA_RESET_TRIES {
          panic!("We were unable to reset DMA and the data buffers!");
        }
        continue;
      }
    }
  }
}

///  Ensure the buffers are filled and everything is prepared for data
///  taking
///
///  The whole procedure takes several seconds. We have to find out
///  how much we can sacrifice from our run time.
///
///  # Arguments 
///
///  * will_panic : The function calls itself recursively and 
///                 will panic after this many calls to itself
///
fn make_sure_it_runs(will_panic : &mut u8) {
  let when_panic : u8 = 5;
  *will_panic += 1;
  if *will_panic == when_panic {
    // it is hopeless. Let's give up.
    // Let's try to stop the DRS4 before
    // we're killing ourselves
    idle_drs4_daq().unwrap_or(());
    // FIXME - send out Alert
    panic!("I can not get this run to start. I'll kill myself!");
  }
  let five_milli = time::Duration::from_millis(5); 
  let two_secs   = time::Duration::from_secs(2);
  let five_secs  = time::Duration::from_secs(5);
  idle_drs4_daq().unwrap_or(());
  thread::sleep(five_milli);
  setup_drs4().unwrap_or(());
  thread::sleep(five_milli);
  reset_data_memory_aggressively();
  thread::sleep(five_milli);
  match start_drs4_daq() {
    Err(err) => {
      debug!("Got err {:?} when trying to start the drs4 DAQ!", err);
    }
    Ok(_)  => {
      trace!("Starting DRS4..");
    }
  }
  // check that the data buffers are filling
  let buf_a = BlobBuffer::A;
  let buf_b = BlobBuffer::B;
  let buf_size_a = get_buff_size(&buf_a).unwrap_or(0);
  let buf_size_b = get_buff_size(&buf_b).unwrap_or(0); 
  thread::sleep(five_secs);
  if get_buff_size(&buf_a).unwrap_or(0) == buf_size_a &&  
      get_buff_size(&buf_b).unwrap_or(0) == buf_size_b {
    warn!("Buffers are not filling! Running setup again!");
    make_sure_it_runs(will_panic);
  } 
}

// palceholder
#[derive(Debug)]
pub struct FIXME {
}


/// Make sure a run stops
///
/// This will recursively call 
/// drs4_idle to stop data taking
///
/// # Arguments:
///
/// * will_panic : After this many calls to 
///                itself, kill_run will 
///                panic.
///
fn kill_run(will_panic : &mut u8) {
  let when_panic : u8 = 5;
  *will_panic += 1;
  if when_panic == *will_panic {
    panic!("We can not kill the run! I'll kill myself!");
  }
  let one_milli        = time::Duration::from_millis(1);
  match idle_drs4_daq() {
    Ok(_)  => (),
    Err(_) => {
      warn!("Can not end run!");
      thread::sleep(one_milli);
      kill_run(will_panic)
    }
  }
}

///  A simple routine which runs until 
///  a certain amoutn of events are 
///  acquired
///
///  The runner will setup the DRS4, and 
///  set it to idle state when it is 
///  finished.
///
///  To be resource friendly, this thread
///  goes with 1 second precision.
///
///  # Arguments
///
///  * max_events  : Acqyire this number of events
///  * max_seconds : Let go for the specific runtime
///  * max_errors  : End myself when I see a certain
///                  number of errors
///  * kill_signal : End run when this line is at bool 
///                  1
///  * prog_op_ev  : An option for a progress bar which
///                  is helpful for debugging
///
pub fn runner(max_events  : Option<u64>,
              max_seconds : Option<u64>,
              max_errors  : Option<u64>,
              kill_signal : Option<&Receiver<bool>>,
              prog_op_ev  : Option<Box<ProgressBar>>) {
  
  let one_milli        = time::Duration::from_millis(1);
  let one_sec          = time::Duration::from_secs(1);
  let mut first_iter   = true; 
  let mut last_evt_cnt : u32 = 0;
  let mut evt_cnt      : u32 = 0;
  let mut delta_events : u64 = 0;
  let mut n_events     : u64 = 0;
  let mut n_errors     : u64 = 0;

  match prog_op_ev {
    None => (),
    Some(ref bar) => {
      bar.reset();
      match max_events {
        None    => (),
        Some(n) => {
          bar.set_length(n);
        }
      }
    }
  }


  let now = time::Instant::now();

  let mut terminate = false;
  // the runner will specifically set up the DRS4
  let mut will_panic : u8 = 0;
  make_sure_it_runs(&mut will_panic);
  info!("Begin Run!");

  loop {
    match get_event_count() {
      Err (err) => {
        debug!("Can not obtain event count! Err {:?}", err);
        thread::sleep(one_sec);    
        continue;
      }
      Ok (cnt) => {
        evt_cnt = cnt;
        if first_iter {
          last_evt_cnt = evt_cnt;
          first_iter = false;
          continue;
        }
        if evt_cnt == last_evt_cnt {
          thread::sleep(one_sec);
          info!("We didn't get an updated event count!");
          continue;
        }
      }
    } // end match
    delta_events = (evt_cnt - last_evt_cnt) as u64;
    n_events += delta_events;
    last_evt_cnt = evt_cnt;
    
    match prog_op_ev {
      None => (),
      Some(ref bar) => {
        bar.inc(delta_events);   
      }
    }
    info!("Checking for kill signal");
    // terminate if one of the 
    // criteria is fullfilled
    match kill_signal {
      Some(ks) => {
        match ks.recv() {
          Ok(signal) => {
            warn!("Have received kill signal!");
            terminate = signal;
          },
          Err(_) => {
            info!("Did not get kill signal!");
          }
        }
      },
      None => ()
    }
    match max_events {
      None => (),
      Some(max_e) => {
        if n_events > max_e {
          terminate = true;
        }
      }
    }
    
    match max_seconds {
      None => (),
      Some(max_t) => {
        if now.elapsed().as_secs() > max_t {
          terminate = true;
        }
      }
    }

    match max_errors {
      None => (),
      Some(max_e) => {
        if n_errors > max_e {
          terminate = true;
        }
      }
    }
    // exit loop on n event basis
    if terminate {
      match prog_op_ev {
        None => (),
        Some(ref bar) => {
          bar.finish();
        }
      }
      break;
    }
    // save cpu
    thread::sleep(one_sec);
  } // end loop 

  // if the end condition is met, we stop the run
  let mut will_panic : u8 = 0;
  kill_run(&mut will_panic);
}


/// Recieve the events and hold them in a cache 
/// until they are requested
/// 
/// The function should be wired to a producer
/// of RBEventPayloads
///
/// Requests come in as event ids through `recv_evid`
/// and will be answered through `send_ev_pl`, if 
/// they are in the cache, else None
///
/// # Arguments
///
/// * control_ch : Receive operation mode instructions
///
pub fn event_cache_worker(recv_ev_pl  : Receiver<RBEventPayload>,
                          send_ev_pl  : Sender<Option<RBEventPayload>>,
                          get_op_mode : Receiver<TofOperationMode>, 
                          recv_evid   : Receiver<u32>,
                          cache_size  : usize) {
  let mut n_send_errors = 0;
    
  let mut op_mode_stream = false;

  let mut oldest_event_id : u32 = 0;
  let mut event_cache : HashMap::<u32, RBEventPayload> = HashMap::new();
  loop {
    // check changes in operation mode
    match get_op_mode.try_recv() {
      Err(err) => trace!("No op mode change detected!"),
      Ok(mode) => {
        warn!("Will change operation mode to {:?}!", mode);
        match mode {
          TofOperationMode::TofModeRequestReply => {op_mode_stream = false;},
          TofOperationMode::TofModeStreamAny    => {op_mode_stream = true;},
        }
      }
    }
    // store incoming events in the cache  
    match recv_ev_pl.try_recv() {
      Err(err) => {
        trace!("No event payload! {err}");
        //continue;
      } // end err
      Ok(event)  => {
        trace!("Received next RBEvent!");
        if oldest_event_id == 0 {
          oldest_event_id = event.event_id;
        } //endif
        // store the event in the cache
        event_cache.insert(event.event_id, event);   
        // keep track of the oldest event_id
        info!("We have a cache size of {}", event_cache.len());
        if event_cache.len() > cache_size {
          event_cache.remove(&oldest_event_id);
          oldest_event_id += 1;
        } //endif
      }// end Ok
    } // end match
  
    // if we are in "stream_any" mode, we don't need to take care
    // of any fo the response/request.
    if op_mode_stream {
      //event_cache.as_ref().into_iter().map(|(evid, payload)| {send_ev_pl.try_send(Some(payload))});
      //let evids = event_cache.keys();
      for payload in event_cache.values() {
        // FIXME - this is bad! Too much allocation
        send_ev_pl.try_send(Some(payload.clone())); 
      }
      event_cache.clear();
      //for n in evids { 
      //  let payload = event_cache.remove(n).unwrap();
      //  send_ev_pl.try_send(Some(payload)); 
      //}
      continue;
    }
    match recv_evid.try_recv() {
      Err(err) => {
        trace!("Issue receiving event id! Err: {err}");
      },
      Ok(event_id) => {
        let has_it = event_cache.contains_key(&event_id);
        if !has_it {
          send_ev_pl.try_send(None);
          // hamwanich
          debug!("We don't have {event_id}!");
        } else {
          let event = event_cache.remove(&event_id).unwrap();
          send_ev_pl.try_send(Some(event));
        }
      },
    } // end match
  } // end loop
}

/// Deal with incoming commands
///
///
///
///
pub struct Commander<'a> {

  pub evid_send      : Sender<u32>,
  pub change_op_mode : Sender<TofOperationMode>, 
  pub rb_evt_recv    : Receiver<Option<RBEventPayload>>,
  pub zmq_pub_socket : &'a zmq::Socket,
}

impl Commander<'_> {

  pub fn new (socket          : &zmq::Socket,
              send_ev         : Sender<u32>,
              evpl_from_cache : Receiver<Option<RBEventPayload>>,
              change_op_mode  : Sender<TofOperationMode>)
    -> Commander {

    Commander {
      evid_send      : send_ev,
      change_op_mode : change_op_mode,
      rb_evt_recv    : evpl_from_cache,
      zmq_pub_socket : socket,
    }
  }


  /// Interpret an incoming command 
  ///
  /// The command comes most likely somehow over 
  /// the wir from the tof computer
  ///
  /// Match with a list of known commands and 
  /// take action.
  ///
  /// # Arguments
  ///
  /// * command : A TofCommand instructing the 
  ///             commander what to do
  ///             Will generate a TofResponse 
  ///             
  pub fn command (&self, cmd : &TofCommand)
    -> Result<TofResponse, FIXME> {
    match cmd {
      TofCommand::PowerOn   (mask) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::PowerOff  (mask) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::PowerCycle(mask) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::RBSetup   (mask) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      }, 
      TofCommand::SetThresholds   (thresholds) =>  {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::StartValidationRun  (_) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::RequestWaveforms (eventid) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::UnspoolEventCache   (_) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::StreamOnlyRequested (_) => {
        let op_mode = TofOperationMode::TofModeRequestReply;
        self.change_op_mode.try_send(op_mode);
        return Ok(TofResponse::Success(RESP_SUCC_FINGERS_CROSSED));
      },
      TofCommand::StreamAnyEvent      (_) => {
        let op_mode = TofOperationMode::TofModeStreamAny;
        self.change_op_mode.try_send(op_mode);
        return Ok(TofResponse::Success(RESP_SUCC_FINGERS_CROSSED));
      },
      //TofCommand::DataRunStart (max_event) => {
      //  // let's start a run. The value of the TofCommnad shall be 
      //  // nevents
      //  self.workforce.execute(move || {
      //      runner(Some(*max_event as u64),
      //             None,
      //             None,
      //             self.get_killed_chn,
      //             None);
      //  }); 
      //  return Ok(TofResponse::Success(RESP_SUCC_FINGERS_CROSSED));
      //}, 
      //TofCommand::DataRunEnd   => {
      //  if !self.run_active {
      //    return Ok(TofResponse::GeneralFail(RESP_ERR_NORUNACTIVE));
      //  }
      //  warn!("Will kill current run!");
      //  self.kill_chn.send(true);
      //  return Ok(TofResponse::Success(RESP_SUCC_FINGERS_CROSSED));
      //},
      TofCommand::VoltageCalibration (_) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::TimingCalibration  (_) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::CreateCalibrationFile (_) => {
        warn!("Not implemented");
        return Ok(TofResponse::GeneralFail(RESP_ERR_NOTIMPLEMENTED));
      },
      TofCommand::RequestEvent(eventid) => {
        match self.evid_send.send(*eventid) {
          Err(err) => {
            debug!("Problem sending event id to cache! Err {err}");
            return Ok(TofResponse::GeneralFail(*eventid));
          },
          Ok(event) => (),
        }
        match self.rb_evt_recv.recv() {
          Err(err) => {
            return Ok(TofResponse::EventNotReady(*eventid));
          }
          Ok(event_op) => {
            // FIXME - prefix topic
            match event_op {
              None => {
                return Ok(TofResponse::EventNotReady(*eventid));
              },
              Some(event) => {
                match self.zmq_pub_socket.send(event.payload, zmq::DONTWAIT) {
                  Ok(_)  => {
                    return Ok(TofResponse::Success(*eventid));
                  }
                  Err(err) => {
                    debug!("Problem with PUB socket! Err {err}"); 
                    return Ok(TofResponse::ZMQProblem(*eventid));
                  }
                }
              }
            }
          }
        }
      },
      TofCommand::RequestMoni (_) => {
      },
      TofCommand::Unknown (_) => {
      }
      _ => {
      }
    }
  
    let response = TofResponse::Success(1);
    Ok(response)
  }
}

///  Get the blob buffer size from occupancy register
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
pub fn get_buff_size(which : &BlobBuffer) ->Result<u32, RegisterError> {
  let size : u32;
  let occ = get_blob_buffer_occ(&which)?;
  trace!("Got occupancy of {occ} for buff {which:?}");

  // the buffer sizes is UIO1_MAX_OCCUPANCY -  occ
  match which {
    BlobBuffer::A => {size = occ - UIO1_MIN_OCCUPANCY;},
    BlobBuffer::B => {size = occ - UIO2_MIN_OCCUPANCY;}
  }
  Ok(size)
}

///  Deal with the raw data buffers.
///
///  Read out when they exceed the 
///  tripping threshold and pass 
///  on the result.
///
///  # Arguments:
///
///  * buff_trip : size which triggers buffer readout.
pub fn buff_handler(which       : &BlobBuffer,
                    buff_trip   : u32,
                    bs_sender   : Option<&Sender<Vec<u8>>>,
                    prog_bar    : &Option<Box<ProgressBar>>,
                    switch_buff : bool) {
  let sleep_after_reg_write = Duration::from_millis(SLEEP_AFTER_REG_WRITE as u64);
  let buff_size : u32;
  match get_buff_size(&which) {
    Ok(bf)   => { 
      buff_size = bf;
    },
    Err(err) => { 
      debug!("Error getting buff size! {:?}", err);
      buff_size = 0;
    }
  }

  let has_tripped = buff_size >= buff_trip;

  if has_tripped {
    debug!("Buff {which:?} tripped at a size of {buff_size}");  
    debug!("Buff size {buff_size}");
    // reset the buffers
    if switch_buff {
      match switch_ram_buffer() {
        Ok(_)  => debug!("Ram buffer switched!"),
        Err(_) => warn!("Unable to switch RAM buffers!") 
      }
    }
    //thread::sleep_ms(SLEEP_AFTER_REG_WRITE);
    let bytestream = read_data_buffer(&which, buff_size as usize).unwrap();
    match bs_sender {
      Some(snd) => snd.send(bytestream),
      None      => Ok(()),
    };
    
    match blob_buffer_reset(&which) {
      Ok(_)  => debug!("Successfully reset the buffer occupancy value"),
      Err(_) => warn!("Unable to reset buffer!")
    }
    match prog_bar {
      Some(bar) => bar.set_position(0),
      None      => () 
    }
    thread::sleep(sleep_after_reg_write);
  } else { // endf has tripped
    match prog_bar {
      Some(bar) => bar.set_position(buff_size as u64),
      None      => () 
    }
  }
}

/////! FIXME - should become a feature
//pub fn setup_progress_bar(msg : String, size : u64, format_string : String) -> ProgressBar {
//  let mut bar = ProgressBar::new(size).with_style(
//    //ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
//    ProgressStyle::with_template(&format_string)
//    .unwrap()
//    .progress_chars("##-"));
//  //);
//  bar.set_message(msg);
//  //bar.finish_and_clear();
//  ////let mut style_found = false;
//  //let style_ok = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}");
//  //match style_ok {
//  //  Ok(_) => { 
//  //    style_found = true;
//  //  },
//  //  Err(ref err)  => { warn!("Can not go with chosen style! Not using any! Err {err}"); }
//  //}  
//  //if style_found { 
//  //  bar.set_style(style_ok.unwrap()
//  //                .progress_chars("##-"));
//  //}
//  bar
//}


///  Transforms raw bytestream to RBEventPayload
///
///  This allows to get the eventid from the 
///  binrary form of the RBEvent
///
///  #Arguments
/// 
///  * bs_recv   : A receiver for bytestreams. The 
///                bytestream comes directly from 
///                the data buffers.
///  * ev_sender : Send the the payload to the event cache
pub fn event_payload_worker(bs_recv   : &Receiver<Vec<u8>>,
                            ev_sender : Sender<RBEventPayload>) {
  let mut n_events : u32;
  let mut event_id : u32 = 0;
  'main : loop {
    let mut start_pos : usize = 0;
    n_events = 0;
    match bs_recv.recv() {
      Ok(bytestream) => {
        'bytestream : loop {
          match search_for_u16(BlobData::HEAD, &bytestream, start_pos) {
            Ok(head_pos) => {
              let tail_pos   = head_pos + BlobData::SERIALIZED_SIZE;
              if tail_pos > bytestream.len() - 1 {
                // we are finished here
                debug!("Work on current blob complete. Extracted {n_events} events. Got last event_id! {event_id}");
                break 'bytestream;
              }
              event_id   = BlobData::decode_event_id(&bytestream[head_pos..tail_pos]);
              n_events += 1;
              start_pos = tail_pos;
              let mut payload = Vec::<u8>::new();
              payload.extend_from_slice(&bytestream[head_pos..tail_pos]);
              let rb_payload = RBEventPayload::new(event_id, payload); 
              match ev_sender.send(rb_payload) {
                Ok(_) => (),
                Err(err) => debug!("Problem sending RBEventPayload over channel!"),
              }
              continue 'bytestream;
            },
            Err(err) => {
              println!("Send {n_events} events. Got last event_id! {event_id}");
              warn!("Got bytestream, but can not find HEAD bytes, err {err:?}");
              break 'bytestream;}
          } // end loop
        } // end ok
      }, // end Ok(bytestream)
      Err(err) => {
        warn!("Received Garbage! Err {err}");
        continue 'main;
      }
    }// end match 
  } // end outer loop
}


///  Prepare the whole readoutboard for data taking.
///
///  This sets up the drs4 and c
///  lears the memory of 
///  the data buffers.
///  
///  This will make several writes to the /dev/uio0
///  memory map.
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
  //let read_ch9        : u32  = 1;

  // before we do anything, set the DRS in idle mode 
  // and set the configure bit
  idle_drs4_daq()?;
  thread::sleep(one_milli);
  set_drs4_configure()?;
  thread::sleep(one_milli);

  // Sanity checking
  //let max_samples     : u32 = 65000;
  //let max_duration    : u32 = 1440; // Minutes in 1 day

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
    blob_buffer_reset(&buf_a)?;
    thread::sleep(one_milli);
    blob_buffer_reset(&buf_b)?;
    thread::sleep(one_milli);
  }

  // register 04 contains a lot of stuff:
  // roi mode, busy, adc latency
  // sample  count and spike removal
  let spike_clean_enable : u32 = 4194304; //bit 22
  if spike_clean {
    let mut value = read_control_reg(0x40).unwrap();  
    value = value | spike_clean_enable;
    write_control_reg(0x40, value)?;
    thread::sleep(one_milli);
  }
  
  set_readout_all_channels_and_ch9()?;
  thread::sleep(one_milli);
  set_master_trigger_mode()?;
  thread::sleep(one_milli);
  Ok(())
}


