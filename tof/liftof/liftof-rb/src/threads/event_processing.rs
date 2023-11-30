use std::thread;
use std::time::Duration;
use crossbeam_channel::{Sender,
                        Receiver};

use tof_dataclasses::events::DataType;

use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::io::RBEventMemoryStreamer;

///  Transforms raw bytestream to TofPackets
///
///  This allows to get the eventid from the 
///  binrary form of the RBEvent
///
///  #Arguments
/// 
///  * bs_recv     : A receiver for bytestreams. The 
///                  bytestream comes directly from 
///                  the data buffers.
///  * tp_sender   : Send the resulting data product to 
///                  get processed further
///  * data_type   : If different from 0, do some processing
///                  on the data read from memory
///
pub fn event_processing(bs_recv           : &Receiver<Vec<u8>>,
                        tp_sender         : &Sender<TofPacket>,
                        dtf_fr_runner     : &Receiver<DataType>,
                        verbose           : bool,
                        calc_crc32        : bool) {
  let mut events_not_sent : u64 = 0;
  let mut data_type       : DataType   = DataType::Unknown;
  let one_milli           = Duration::from_millis(1);
  let mut streamer        = RBEventMemoryStreamer::new();
  streamer.calc_crc32     = calc_crc32;
  'main : loop {
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
    if bs_recv.is_empty() {
      // FIXME - benchmark
      thread::sleep(5*one_milli);
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
        info!("[EVPROC] - Received {} bytes!", _stream.len());
        bytestream = _stream;
        //streamer.add(&bytestream, bytestream.len());
        streamer.consume(&mut bytestream);
        let mut packets_in_stream : u32 = 0;
        let mut last_event_id     : u32 = 0;
        loop {
          if streamer.is_depleted {
            info!("Streamer exhausted after sending {} packets!", packets_in_stream);
            break;
          }
          match streamer.next() {
            None => {
              //info!("Streamer exhausted after sending {} packets!", packets_in_stream);
              //break;
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

