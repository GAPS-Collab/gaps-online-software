use std::collections::HashMap;

use crossbeam_channel::{Sender,
                        Receiver};

use tof_dataclasses::commands::{RBCommand,
                                TofOperationMode};
use tof_dataclasses::packets::{TofPacket,
                               PacketType};
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::events::RBEvent;

/// Recieve the events and hold them in a cache 
/// until they are requested
/// 
/// Requests come in as event ids through `recv_evid`
/// and will be answered through `send_ev_pl`, if 
/// they are in the cache, else None
///
/// # Arguments
/// 
/// * tp_recv           : receive tofpackets from the commander/
///                       or event processing
///
///
/// * control_ch : Receive operation mode instructions
///
/// * waveform_analysis : For the events requested, do the waveform processing 
///                       already
pub fn event_cache(tp_recv           : Receiver<TofPacket>,
                   tp_to_pub         : &Sender<TofPacket>,
                   get_op_mode       : &Receiver<TofOperationMode>, 
                   waveform_analysis : bool,
                   cache_size   : usize) {
  if waveform_analysis {
    warn!("Waveform analysis is not implemented, won't do it!");
  }

  let mut n_send_errors     = 0u64;   
  let mut op_mode_stream    = false;
  let mut packet_evid       : u32;

  let mut event_cache        : HashMap::<u32, TofPacket> = HashMap::with_capacity(cache_size);
  let mut request_cache      : HashMap::<u32, TofPacket> = HashMap::with_capacity(cache_size);
  let mut n_iter_loop        : usize = 0;
  let max_len_request_cache  : usize = 10000;


  loop {
    // check changes in operation mode
    match get_op_mode.try_recv() {
      Err(err) => trace!("No op mode change detected! Err {err}"),
      Ok(mode) => {
        warn!("Will change operation mode to {:?}!", mode);
        match mode {
          TofOperationMode::RequestReply => {op_mode_stream = false;},
          TofOperationMode::StreamAny    => {op_mode_stream = true;},
          _ => (),
        }
      }
    }

    match tp_recv.try_recv() {
      Err(err) =>   {
        trace!("No new TofPacket received! {err}");
      }
      Ok(packet) => {
        // FIXME - there need to be checks what the 
        // packet type is
        match packet.packet_type {
          PacketType::RBCommand => {
            // we only care if we are not in stream mode
            if op_mode_stream {
              debug!("Received RBCommand, but we are in StreamAny mode, ignoring...");
            } else {
              // this will be the event requests
              match RBCommand::from_bytestream(&packet.payload, &mut 0) {
                Err(err) => {
                  error!("Unable to understand bytestream! Err {err}");
                  continue;
                }
                Ok(request) => {
                  // if we can serve the request, we are good, if not we put it in the 
                  // queue
                  info!("Received reqauest {}", request);
                  if request.command_code != RBCommand::REQUEST_EVENT {
                    error!("Can't deal with RBCommand {}", request);
                    continue;
                  }
                  if !event_cache.contains_key(&request.payload) {
                    // we will need to decode RBRequest later again, 
                    // but since it is small it is probably fine.
                    request_cache.insert(request.payload, packet);
                  } else { // immediatly answer the request and 
                           // throw the request away after.
                    // unwrap is fine, since we checked we have it
                    let tp = event_cache.remove(&request.payload).unwrap();
                    match tp_to_pub.try_send(tp) {
                      Err(err) => trace!("Error sending event {}! {err}", request.payload),
                      Ok(_)    => ()
                    }
                  }
                }
              }
            } // end if not op_mode_stream
          },
          PacketType::RBEvent   => {
            // FIXME - proper matching, however, if implemented
            // correctly this should never fail since broken 
            // packets should not end up in the cache
            packet_evid = RBEvent::extract_eventid(&packet.payload).unwrap_or(0);
            if !event_cache.contains_key(&packet_evid) && packet_evid != 0 {
              event_cache.insert(packet_evid, packet);
            } else {
              error!("We have seen event {packet_evid} before! This seems to be a dupiclicate!");
              continue;
            }
          },
          _ => {
            error!("RB event cache can not deal with packet type {}", packet.packet_type);
            continue;
          }
        }
      }
    }  // end of tp_recv.try_recv() 
  
    // if we are in "stream_any" mode, we don't need to take care
    // of any fo the response/request.
    if op_mode_stream {
      for (_,tp) in event_cache.drain() {
        match tp_to_pub.try_send(tp) {
          Err(err) => {
            error!("Error sending! {err}");
            n_send_errors += 1;
          }
          Ok(_)    => ()
        }
      }
      continue; // makes rest of code unreachable
                // for this case
    } else {
      // Here now, we have to make sure that the 
      // caches get emptied. So we have to check for every request in our request cache,
      // if the event_cache has it. Do this only every 10 iterations. (number should be configurable)
      if n_iter_loop == 9 {
        let mut bad_keys = Vec::<u32>::new();
        for event_key in request_cache.keys() {
          if event_cache.contains_key(&event_key) {
            // unwrap is fine, since we checked we have it
            //let this_request = request_cache.remove(&event_key).unwrap();
            let tp = event_cache.remove(&event_key).unwrap();
            info!("Responding with event {}", event_key);
            match tp_to_pub.try_send(tp) {
              Err(err) => trace!("Error sending event {}! {err}", event_key),
              Ok(_)    => ()
            }
            bad_keys.push(*event_key);
          } 
        }
        for k in bad_keys.iter() {
          request_cache.remove(&k);
        }
        if request_cache.len() > max_len_request_cache {
          warn!("Request too large! Will remove oldest entries!");
          request_cache.retain(|_, v| !v.has_timed_out());
        }
        if event_cache.len() > cache_size {
          warn!("Event too large! Will remove oldest entries!");
          event_cache.retain(|_, v| !v.has_timed_out());
        } //endif
        n_iter_loop = 0; 
        continue;
      }
    }
    if n_send_errors > 0 {
      error!("There were {n_send_errors} errors during sending!");
    }
    // make sure cache won't overflow
    n_iter_loop += 1;
  } // end loop
}

