use std::thread;
use std::time::Duration;
//use std::collections::VecDeque;

use crossbeam_channel::{Sender,
                        Receiver};

use tof_dataclasses::events::DataType;
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::packets::{
    TofPacket,
    PacketType
};
use tof_dataclasses::io::RBEventMemoryStreamer;
use tof_dataclasses::commands::{
    RBCommand,
    TofOperationMode,
};


///  Transforms raw bytestream to TofPackets
///
///  This allows to get the eventid from the 
///  binrary form of the RBEvent
///
///  #Arguments
///  
///  * tp_recv     : A receiver for TofPackets. This
///                  will receive RBCommands with 
///                  event ids to consider.
///  * bs_recv     : A receiver for bytestreams. The 
///                  bytestream comes directly from 
///                  the data buffers.
///  * tp_sender   : Send the resulting data product to 
///                  get processed further
///  * data_type   : If different from 0, do some processing
///                  on the data read from memory
///
pub fn event_processing(tp_recv           : &Receiver<TofPacket>,
                        bs_recv           : &Receiver<Vec<u8>>,
                        get_op_mode       : &Receiver<TofOperationMode>, 
                        tp_sender         : &Sender<TofPacket>,
                        dtf_fr_runner     : &Receiver<DataType>,
                        verbose           : bool,
                        calc_crc32        : bool) {
  let mut op_mode_stream  = true;
  let mut events_not_sent : u64 = 0;
  let mut data_type       : DataType   = DataType::Unknown;
  let one_milli           = Duration::from_millis(1);
  let mut streamer        = RBEventMemoryStreamer::new();
  streamer.calc_crc32     = calc_crc32;
  // our cachesize is 50 events. This means each time we 
  // receive data over bs_recv, we have received 50 more 
  // events. This means we might want to wait for 50 MTE
  // events?
  let mut n_request = 0;
  let ev_buff_size = 50;
  'main : loop {
    if !get_op_mode.is_empty() {
      match get_op_mode.try_recv() {
        Err(err) => trace!("No op mode change detected! Err {err}"),
        Ok(mode) => {
          warn!("Will change operation mode to {:?}!", mode);
          match mode {
            TofOperationMode::RequestReply => {
              streamer.request_mode = true;
              op_mode_stream = false;
            },
            TofOperationMode::StreamAny    => {
              op_mode_stream = true;
              streamer.request_mode = false;
            },
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
      thread::sleep(one_milli);
      continue;
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
        println!("Streamer::stream size {}", streamer.stream.len());
        'event_reader : loop {
          if streamer.is_depleted {
            info!("Streamer exhausted after sending {} packets!", packets_in_stream);
            break 'event_reader;
          }
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
            Some(_event) => {
              let mut event = _event;
              if verbose {
                println!("{}", event);
              }
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
              println!("==> Sending event with header {}", event.header);
              let mut tp = TofPacket::from(&event);
              // set flags
              match data_type {
                DataType::VoltageCalibration |
                DataType::TimingCalibration  | 
                DataType::Noi => {
                  tp.no_write_to_disk = true;
                },
                _ => ()
              }
              // send the packet
              //println!("[EVENTPROC] => TofPacket to be send {}",tp);
              match tp_sender.send(tp) {
                Ok(_) => {
                  packets_in_stream += 1;
                },
                Err(err) => {
                  error!("Problem sending TofPacket over channel! {err}");
                  events_not_sent += 1;
                }
              }
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

