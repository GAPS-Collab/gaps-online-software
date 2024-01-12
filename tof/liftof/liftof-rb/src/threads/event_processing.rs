use std::fs;
use std::path::PathBuf;
use std::sync::{
    Arc,
    Mutex,
};

use crossbeam_channel::{Sender,
                        Receiver};

use tof_dataclasses::events::DataType;
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::packets::{
    TofPacket,
    PacketType
};
use tof_dataclasses::io::RBEventMemoryStreamer;
use tof_dataclasses::calibrations::RBCalibrations;
use tof_dataclasses::threading::ThreadControl;
use tof_dataclasses::events::EventStatus;
use tof_dataclasses::commands::{
    RBCommand,
    TofOperationMode,
};

use liftof_lib::{
    RunStatistics,
    get_rb_ch_pid_map,
    waveform_analysis,
};

use crate::control::get_board_id;

///  Transforms raw bytestream to TofPackets
///
///  This allows to get the eventid from the 
///  binrary form of the RBEvent
///
///  #Arguments
///  
///  * board_id            : The unique ReadoutBoard identifier
///                          (ID) of this RB
///  * tp_recv             : A receiver for TofPackets. This
///                          will receive RBCommands with 
///                          event ids to consider.
///  * bs_recv             : A receiver for bytestreams. The 
///                          bytestream comes directly from 
///                          the data buffers.
///  * get_op_mode         : The TOF operation mode. Typically,
///                          this is "StreamAny", meaning that 
///                          whatever the TOF produceds, it gets
///                          wrapped in TofPackets and send away.
///                          In "RequestReply" mode, liftof-rb
///                          waits for event requests sent by 
///                          a third party
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
                        tp_recv             : &Receiver<TofPacket>,
                        bs_recv             : &Receiver<Vec<u8>>,
                        get_op_mode         : &Receiver<TofOperationMode>, 
                        tp_sender           : &Sender<TofPacket>,
                        dtf_fr_runner       : &Receiver<DataType>,
                        verbose             : bool,
                        calc_crc32          : bool,
                        thread_control      : Arc<Mutex<ThreadControl>>,
                        stat                : Arc<Mutex<RunStatistics>>,
                        only_perfect_events : bool) {
  // reasonable default ? 
  let mut op_mode = TofOperationMode::StreamAny;

  // load calibration just in case?
  let mut cali_loaded = false;
  let mut cali   = RBCalibrations::new(0);
  let cali_path  = format!("/home/gaps/calib/rb_{:0>2}.cali.tof.gaps", board_id);
  let pmap_file  = String::from("/home/gaps/config/rb_paddle_map.json");
  let pmap_path  = PathBuf::from(&pmap_file);
  let cali_path_buf = PathBuf::from(&cali_path);
  let paddle_map = get_rb_ch_pid_map(pmap_path);
  if !fs::metadata(cali_path_buf).is_ok() {
    match RBCalibrations::from_file(cali_path) {
      Err(err) => {
        error!("Can't load calibration! {err}");
      },
      Ok(_c) => {
        cali = _c;
        cali_loaded = true;
      }
    }
  } else {
    warn!("Calibration file not available!");
    cali_loaded = false;
  }
  // FIXME - deprecate!
  let mut op_mode_stream  = true;
  let mut events_not_sent : u64 = 0;
  let mut data_type       : DataType   = DataType::Unknown;
  //let one_milli           = Duration::from_millis(1);
  let mut streamer        = RBEventMemoryStreamer::new();
  streamer.calc_crc32     = calc_crc32;
  // our cachesize is 50 events. This means each time we 
  // receive data over bs_recv, we have received 50 more 
  // events. This means we might want to wait for 50 MTE
  // events?
  let mut n_request = 0;
  let ev_buff_size = 50;
  let mut n_events = 0usize;
  'main : loop {
    match thread_control.lock() {
      Ok(tc) => {
        if tc.stop_flag {
          info!("Received stop signal. Will stop thread!");
          break;
        }
      },
      Err(err) => {
        trace!("Can't acquire lock! {err}");
      },
    }

    if !get_op_mode.is_empty() {
      match get_op_mode.try_recv() {
        Err(err) => trace!("No op mode change detected! Err {err}"),
        Ok(mode) => {
          warn!("Will change operation mode to {:?}!", mode);
          match mode {
            TofOperationMode::RequestReply => {
              streamer.request_mode = true;
              op_mode_stream = false;
              op_mode = mode;
            },
            TofOperationMode::StreamAny    => {
              op_mode_stream = true;
              streamer.request_mode = false;
              op_mode = mode;
            },
            TofOperationMode::RBWaveform   => {
              if !cali_loaded {
                error!("Requesting waveform analysis without having a calibration loaded!");
                error!("Can't do waveform analysis without calibration!");
                error!("Switching mode to StreamAny");
                op_mode = TofOperationMode::StreamAny;
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
          error!("Issues receiving datatype/format! Err {err}");
        }
        Ok(dtf) => {
          data_type = dtf; 
          info!("Will process events for data type {}!", data_type);
        }
      }
    }
    if !tp_recv.is_empty() && !op_mode_stream {
      //println!("==> We see that our streamer is ahead by {}", streamer.is_ahead_by);
      //println!("==> We see that our streamer is behind by {}", streamer.is_behind_by);
      let mut max_request = streamer.is_ahead_by + ev_buff_size;
      if streamer.is_behind_by > 0 && streamer.request_cache.len() != 0 {
        max_request = 0;
      }
      //println!("==> max request {}, n request {}", max_request, n_request);
      while n_request < max_request {
        match tp_recv.recv() {
          Err(_err) => (),
          Ok(tp) => {
            match tp.packet_type {
              PacketType::RBCommand => {
                match RBCommand::from_bytestream(&tp.payload, &mut 0) {
                  Err(err) => {
                    error!("Can't decode RBCommand! {err}");
                  },
                  Ok(request) => {
                    //println!("=> Got request {}",request);
                    n_request += 1;
                    //request_cache.push_back((request.payload, request.channel_mask));
                    streamer.request_cache.push_back((request.payload, request.channel_mask));
                  }
                }
              },
              _ => (),
            }
          }
        }
      }
      //println!("== ==> Digtested {} requests!", {n_request});
    }
    n_request = 0;
    //request_cache = request_cache.sort();
    //println!("==> Request cache {:?}", request_cache);
    //request_cache.clear();
    //println!("Len of BS RECV {}",bs_recv.len());
    if bs_recv.is_empty() {
      //println!("--> Empty bs_rec");
      // FIXME - benchmark
      //thread::sleep(one_milli/2);
      continue 'main;
    }
    // this can't be blocking anymore, since 
    // otherwise we miss the datatype
    let mut skipped_events : usize = 0;
    let mut bytestream : Vec<u8>;
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
        if !op_mode_stream {
          streamer.create_event_index();
          if streamer.is_behind_by > ev_buff_size {
            // we probably have not consumed enough
            // events yet
            //println!("--> Streamer is still behind, will consume more {}", streamer.is_behind_by);
            streamer.is_behind_by -= ev_buff_size;
            continue 'main;
          }
        }
        let mut packets_in_stream : u32 = 0;
        let mut last_event_id     : u32 = 0;
        //println!("Streamer::stream size {}", streamer.stream.len());
        'event_reader : loop {
          if streamer.is_depleted {
            info!("Streamer exhausted after sending {} packets!", packets_in_stream);
            break 'event_reader;
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
                  continue;
                },
                Some(tp) => {
                  tp_to_send = tp;
                }
              }
            },
            TofOperationMode::StreamAny |
            TofOperationMode::RequestReply |
            TofOperationMode::RBWaveform => {
              match streamer.next() {
                None => {
                  if streamer.request_mode {
                    //info!("Streamer exhausted after sending {} packets!", packets_in_stream);
                    if streamer.request_cache.len() == 0 {
                      break 'event_reader;
                    }
                    if streamer.is_ahead_by > 0 {
                      break;
                    }
                    //println!("Streamer behind {}", streamer.is_behind_by);
                    //println!("Streamer ahead  {}", streamer.is_ahead_by);
                    //println!("Streamer is depeleted {}", streamer.is_depleted);
                    //println!("Streamer requests {}", streamer.request_cache.len());
                    // we need to go the whole loop, so trigger streamer.is_depleted, 
                    // even though it might not
                    streamer.is_behind_by = 0;
                  }
                  streamer.is_depleted = true;
                  continue;
                },
                Some(mut event) => {
                  if last_event_id != 0 {
                    if event.header.event_id != last_event_id + 1 {
                      if event.header.event_id > last_event_id {
                          skipped_events += (event.header.event_id - last_event_id) as usize;
                      } else {
                        error!("Something with the event counter is messed up. Got event id {}, but the last event id was {}", event.header.event_id, last_event_id);
                      }
                    }
                  }
                  last_event_id = event.header.event_id;
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
                    debug!("Using paddle map {:?}", paddle_map);
                    match waveform_analysis(&mut event, &paddle_map, &cali) {
                      Err(err) => error!("Waveform analysis failed! {err}"),
                      Ok(_)    => ()
                    }
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
        }
      }, // end OK(recv)
    }// end match 
    if events_not_sent > 0 {
      error!("There were {events_not_sent} unsent events!");
    }
    if skipped_events > 0 {
      error!("We skipped {} events!", skipped_events);
    }
  } // end outer loop
}

