use std::thread;
use std::time::Duration;
use crossbeam_channel::{Sender,
                        Receiver};

use tof_dataclasses::events::{RBEvent,
                              DataType};

use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::serialization::search_for_u16;

///  Transforms raw bytestream to RBEventPayload
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
                        verbose           : bool) {
  let mut n_events : u32;
  let mut event_id : u32 = 0;
  let mut last_event_id   : u32 = 0; // for checks
  let mut events_not_sent : u64 = 0;
  let mut data_type   : DataType   = DataType::Unknown;
  let one_milli   = Duration::from_millis(1);
  'main : loop {
    let mut start_pos : usize = 0;
    n_events = 0;
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
    let mut tail_pos = 0usize;
    let mut skipped_events : usize = 0;
    match bs_recv.recv() {
      Ok(bytestream) => {
        //println!("Getting new bytestream of len {}", bytestream.len());
        let mut packets_in_stream : u32 = 0;
        'bytestream : loop {
          //println!("Starting at {start_pos}");
          match search_for_u16(RBEvent::HEAD, &bytestream, start_pos) {
            Err(err) => {
              debug!("Send {n_events} events. Got last event_id! {event_id}");
              if start_pos == 0 {
                error!("Got bytestream, but can not find HEAD bytes, err {err:?}");
              }
              break 'bytestream;},
            Ok(head_pos) => {
              //println!("HEAD found at {head_pos}");
              match search_for_u16(RBEvent::TAIL, &bytestream, head_pos) {
                Err(err) => {
                  error!("Unable to find complementing TAIL for HEAD at {} in bytestream! Err {err}", head_pos);
                  start_pos = head_pos + 1; // the event in memory is broken, who knows where 
                                  // the next start is.
                  continue;
                },
                Ok(_tail_pos) => {
                  tail_pos = _tail_pos;
                }
              }
              if tail_pos >= bytestream.len() - 1 {
                // we are finished here
                warn!("Got a trunctaed event, discarding..");
                trace!("Work on current blob complete. Extracted {n_events} events. Got last event_id! {event_id}");
                break 'bytestream;
              }
              n_events += 1;
              start_pos = tail_pos;
              let mut tp = TofPacket::new();
              let mut pos_in_stream = head_pos;
              match RBEvent::get_channel_packet_len(&bytestream, pos_in_stream) {
                Err(err)   => {
                  error!("Unable to extract RBEvent from memory! Error {err}");
                  events_not_sent += 1;
                  warn!("Got a trunctaed event, discarding..");
                  trace!("Work on current blob complete. Extracted {n_events} events. Got last event_id! {event_id}");
                  break 'bytestream;
                },
                Ok(data) => {
                  let packet_size = data.0;
                  let ch_ids = data.1;
                }
              }

              match RBEvent::extract_from_rbeventmemoryview(&bytestream, &mut pos_in_stream) {
                Err(err)   => {
                  error!("Unable to extract RBEvent from memory! Error {err}");
                  events_not_sent += 1;
                },
                Ok (mut event) => {
                  if event.header.event_id != last_event_id + 1 {
                    if last_event_id != 0 {
                      skipped_events += event.header.event_id as usize - last_event_id as usize - 1;
                    }
                    if event.header.lost_trigger { 
                      warn!("Lost trigger!");
                    } else {
                      warn!("Event id not rising continuously! This {}, last {}", event.header.event_id, last_event_id);
                      //println!("{}", event);
                    }
                  }
                  last_event_id = event.header.event_id;
                  if verbose {
                    println!("{}", event);
                  }
                  event.data_type = data_type;
                  tp = TofPacket::from(&event);
                }
              }
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
                Err(err) => error!("Problem sending TofPacket over channel! Err {err}"),
              }
            }
          } // end match search_for_u16 
          info!("We have sent {packets_in_stream} packets for this bytestream of len {}", bytestream.len());
        } // end 'bytestream loop
      }, // end OK(recv)
      Err(err) => {
        error!("Received Garbage! Err {err}");
        continue 'main;
      }
    }// end match 
    if events_not_sent > 0 {
      error!("There were {events_not_sent} unsent events!");
    }
    if skipped_events > 0 {
      error!("We skipped {} events!", skipped_events);
    }
  } // end outer loop
}

