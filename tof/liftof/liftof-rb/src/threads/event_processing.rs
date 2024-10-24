//! Event processing deals with the raw memory input
//! from the buffers, send to it by the runner 
//! when reading out the system memory
//!
//! Different modes are available, from sending
//! TofPackets directly through the RBEventMemoryStreamer
//! without any further parsing of the events (this has 
//! to be done then on the TOF main computer) to doing 
//! waveform analysis on the RBs directly

use std::fs;
use std::path::PathBuf;
use std::sync::{
  Arc,
  Mutex,
};

use std::time::Instant;

use crossbeam_channel::{
  Sender,
  Receiver
};

use tof_dataclasses::events::DataType;
use tof_dataclasses::packets::{
  TofPacket,
};
use tof_dataclasses::io::RBEventMemoryStreamer;
use tof_dataclasses::calibrations::RBCalibrations;
use tof_dataclasses::events::EventStatus;
use tof_dataclasses::commands::{
  TofOperationMode,
};

use liftof_lib::{
  RunStatistics,
  //waveform_analysis,
};
use liftof_lib::thread_control::ThreadControl;

 use crate::control::get_deadtime;

///  Transforms raw bytestream to TofPackets
///
///  This allows to get the eventid from the 
///  binrary form of the RBEvent
///
///  #Arguments
///  
///  * board_id            : The unique ReadoutBoard identifier
///                          (ID) of this RB
///  * bs_recv             : A receiver for bytestreams. The 
///                          bytestream comes directly from 
///                          the data buffers.
///  * get_op_mode         : The TOF operation mode. Typically,
///                          this is "Default", meaning that the
///                          RBs will sent what is in the memory 
///                          buffer translated into RBEvents.
///                          In "RBHighThrougput" mode, it will not
///                          translate them into RBEvents, but just
///                          transmits the content of the buffers, and 
///                          RBWaveform mode will do waveform analysis
///                          on the boards
///  * tp_sender           : Send the resulting data product to 
///                          get processed further
///  * data_type           : If different from 0, do some processing
///                          on the data read from memory
///  * verbose             : More output to the console for debugging
///  * calc_crc32          : Calucalate crc32 checksum for the channel
///                          packet. Remember, this might impact 
///                          performance
///  * only_perfect_events : Only transmit events with EventStatus::Perfect.
///                          This only applies when the op mode is not 
///                          RBHighThroughput
pub fn event_processing(board_id            : u8,
                        bs_recv             : &Receiver<Vec<u8>>,
                        get_op_mode         : &Receiver<TofOperationMode>, 
                        tp_sender           : &Sender<TofPacket>,
                        dtf_fr_runner       : &Receiver<DataType>,
                        verbose             : bool,
                        calc_crc32          : bool,
                        thread_control      : Arc<Mutex<ThreadControl>>,
                        stat                : Arc<Mutex<RunStatistics>>,
                        only_perfect_events : bool) {
  
  let mut op_mode = TofOperationMode::Default;
  let mut thread_ctrl_check_timer = Instant::now();

  // load calibration just in case?
  let mut cali_loaded = false;
  let cali        : RBCalibrations;
  let cali_path       = format!("/home/gaps/calib/rb_{:0>2}.cali.tof.gaps", board_id);
  let cali_path_buf   = PathBuf::from(&cali_path);
  if fs::metadata(cali_path_buf.clone()).is_ok() {
    info!("Found valid calibration file path {cali_path_buf:?}");
    match RBCalibrations::from_file(cali_path, true) {
      Err(err) => {
        error!("Can't load calibration! {err}");
      },
      Ok(_c) => {
        cali = _c;
        cali_loaded = true;
        debug!("We loaded calibration {}", cali);
      }
    }
  } else {
    warn!("Calibration file not available!");
    cali_loaded = false;
  }
  
  // FIXME - deprecate!
  let mut events_not_sent : u64 = 0;
  let mut data_type       : DataType   = DataType::Unknown;
  // should we store drs deadtime instead of the FPGA temperature
  let mut deadtime_instead_temp : bool = false;
  
  let mut streamer        = RBEventMemoryStreamer::new();
  // FIXME
  streamer.check_channel_errors = true;
  
  match thread_control.lock() {
    Ok(tc) => {
      streamer.calc_crc32   = tc.liftof_settings.rb_settings.calc_crc32;
      deadtime_instead_temp = tc.liftof_settings.rb_settings.drs_deadtime_instead_fpga_temp; 
    },
    Err(err) => {
      trace!("Can't acquire lock! {err}");
    },
  }
  
  // loop variables
  // our cachesize is 50 events. This means each time we 
  // receive data over bs_recv, we have received 50 more 
  // events. This means we might want to wait for 50 MTE
  // events?
  let mut skipped_events : usize = 0;
  let mut n_events = 0usize;
  
  'main : loop {
    if thread_ctrl_check_timer.elapsed().as_secs() >= 1 {
      match thread_control.lock() {
        Ok(tc) => {
          if tc.stop_flag {
            info!("Received stop signal. Will stop thread!");
            break;
          }
          streamer.calc_crc32   = tc.liftof_settings.rb_settings.calc_crc32;
          deadtime_instead_temp = tc.liftof_settings.rb_settings.drs_deadtime_instead_fpga_temp; 
        },
        Err(err) => {
          trace!("Can't acquire lock! {err}");
        },
      }
      thread_ctrl_check_timer = Instant::now();
    }

    if !get_op_mode.is_empty() {
      match get_op_mode.try_recv() {
        Err(err) => trace!("No op mode change detected! Err {err}"),
        Ok(mode) => {
          warn!("Will change operation mode to {:?}!", mode);
          match mode {
            TofOperationMode::Default    => {
              streamer.request_mode = false;
              op_mode = mode;
            },
            TofOperationMode::RBWaveform   => {
              if !cali_loaded {
                error!("Requesting waveform analysis without having a calibration loaded!");
                error!("Can't do waveform analysis without calibration!");
                error!("Switching mode to Default");
                op_mode = TofOperationMode::Default;
              }
            }
            _ => (),
          }
        }
      }
    }
    if !dtf_fr_runner.is_empty() {
      match dtf_fr_runner.try_recv() {
        Err(err) => {
          error!("Issues receiving datatype/format! {err}");
        }
        Ok(dtf) => {
          data_type = dtf; 
          info!("Will process events for data type {}!", data_type);
        }
      }
    }
    if bs_recv.is_empty() {
      //println!("--> Empty bs_rec");
      // FIXME - benchmark
      //thread::sleep(one_milli/2);
      continue 'main;
    }
    // this can't be blocking anymore, since 
    // otherwise we miss the datatype
    let mut bytestream : Vec<u8>;
    if events_not_sent > 0 {
      error!("There were {events_not_sent} for this iteration of received bytes!");
    }
    if skipped_events > 0 {
      error!("We skipped {} events!", skipped_events);
    }
    // reset skipped events and events not sent, 
    // these are per iteration
    events_not_sent = 0;
    skipped_events  = 0;
    match bs_recv.recv() {
      Err(err) => {
        error!("Received Garbage! Err {err}");
        continue 'main;
      }
      Ok(_stream) => {
        info!("Received {} bytes!", _stream.len());
        bytestream = _stream;
        //streamer.add(&bytestream, bytestream.len());
        streamer.consume(&mut bytestream);
        let mut packets_in_stream : u32 = 0;
        let mut last_event_id     : u32 = 0;
        //println!("Streamer::stream size {}", streamer.stream.len());
        loop {
          if streamer.is_depleted {
            info!("Streamer exhausted after sending {} packets!", packets_in_stream);
            //break 'event_reader;
            // we immediatly want more data in the streamer
            continue 'main;
          }
          // FIXME - here we have the choice. 
          // streamer.next() will yield the next event,
          // decoded
          // streamer.next_tofpacket() instead will only
          // yield the next event, not deserialzed
          // but wrapped already in a tofpacket
          let mut tp_to_send = TofPacket::new();
          match op_mode {
            TofOperationMode::RBHighThroughput => {
              match streamer.next_tofpacket() {
                None => {
                  streamer.is_depleted = true;
                  continue 'main;
                },
                Some(tp) => {
                  tp_to_send = tp;
                }
              }
            },
            TofOperationMode::Default |
            TofOperationMode::RBWaveform => {
              match streamer.next() {
                None => {
                  streamer.is_depleted = true;
                  continue 'main;
                },
                Some(mut event) => {
                  if deadtime_instead_temp {
                    // in case we want to add the deadtime to the header, 
                    // we have to do that here!
                    event.header.deadtime_instead_temp = deadtime_instead_temp;
                    match get_deadtime() {
                      Err(err) => {
                        error!("Unable to get DRS4 deadtime! {err}");
                        event.header.drs_deadtime = u16::MAX;
                      }
                      Ok(d_time) => {
                        event.header.drs_deadtime = d_time as u16;
                      }
                    }
                  }
                  //println!("Got event id {}", event.header.event_id);
                  if last_event_id != 0 {
                    if event.header.event_id != last_event_id + 1 {
                      if event.header.event_id > last_event_id {
                          skipped_events += (event.header.event_id - last_event_id -1) as usize;
                      } else {
                        error!("Something with the event counter is messed up. Got event id {}, but the last event id was {}", event.header.event_id, last_event_id);
                      }
                    }
                  }
                  last_event_id = event.header.event_id;
                  //println!("This event id {}!", last_event_id);
                  event.data_type = data_type;
                  if verbose {
                    match stat.lock() {
                      Err(err) => error!("Unable to acquire lock on shared memory for RunStatisitcis! {err}"),
                      Ok(mut s) => {
                        if s.first_evid == 0 {
                          s.first_evid = event.header.event_id;
                        }
                        s.last_evid = event.header.event_id;
                        if event.status == EventStatus::ChannelIDWrong {
                          s.n_err_chid_wrong += 1;
                        }
                        if event.status == EventStatus::TailWrong {
                          s.n_err_tail_wrong += 1;
                        }
                      }
                    }
                  }
                  if event.status != EventStatus::Unknown {
                    if only_perfect_events && event.status != EventStatus::Perfect {
                      info!("Not sending this event, because it's event status is {} and we requested to send only events with EventStatus::Perfect!", event.status);
                      continue;
                    }
                  }
                  if op_mode == TofOperationMode::RBWaveform {
                    //debug!("Using paddle map {:?}", paddle_map);
                    //match waveform_analysis(&mut event, &paddle_map, &cali) {
                    //  Err(err) => error!("Waveform analysis failed! {err}"),
                    //  Ok(_)    => ()
                    //}
                  }
                  n_events += 1;
                  if verbose && n_events % 100 == 0 {
                    println!("[EVTPROC (verbose)] => Sending event {}", event);
                  }
                  tp_to_send = TofPacket::from(&event);
                },
              } 
            }, // end op mode ~ waveform/event
            _ => {
              error!("Operation mode {} not available yet!", op_mode);
            }
          }
          if verbose {
            match stat.lock() {
              Err(err) => error!("Unable to acquire lock on shared memory for RunStatisitcis! {err}"),
              Ok(mut _st)  => {
                _st.evproc_npack += 1; 
              }
            }
            //println!("[EVTPROC (verbose)] => Sending TofPacket {}", tp_to_send);
          }
          // set flags
          match data_type {
            DataType::VoltageCalibration |
            DataType::TimingCalibration  | 
            DataType::Noi => {
              tp_to_send.no_write_to_disk = true;
            },
            _ => ()
          }
          // send the packet
          match tp_sender.send(tp_to_send) {
            Ok(_) => {
              packets_in_stream += 1;
            },
            Err(err) => {
              error!("Problem sending TofPacket over channel! {err}");
              events_not_sent += 1;
            }
          }
        } // end 'event_reader
      }, // end OK(recv)
    }// end match 
  } // end outer loop
}

