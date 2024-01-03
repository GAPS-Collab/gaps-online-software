//! Communication with the flight computer
//!
//! Using two dedicated 0MQ wires - one for 
//! data, the other for commands
//!
//!

use std::time::{
    Instant,
    Duration,
};

extern crate crossbeam_channel;
use crossbeam_channel::Receiver; 

use tof_dataclasses::packets::{
    TofPacket,
    PacketType
};

use tof_dataclasses::monitoring::{RBMoniData,
                                  MtbMoniData,
                                  TofCmpMoniData};

use tof_dataclasses::events::TofEvent;
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::io::TofPacketWriter;

/// Manages "outgoing" 0MQ PUB socket
///
/// Everything should send to here, and 
/// then it gets passed on over the 
/// connection to the flight computer
///
/// # Arguments
///
/// * flight_address   : The address the flight computer
///                      (or whomever) wants to listen.
///                      A 0MQ PUB socket witll be bound 
///                      to this address.
pub fn global_data_sink(incoming           : &Receiver<TofPacket>,
                        flight_address     : &str,
                        write_stream       : bool,
                        write_stream_path  : String,
                        runid              : usize,
                        print_moni_packets : bool) {

  let ctx = zmq::Context::new();
  // FIXME - should we just move to another socket if that one is not working?
  let data_socket = ctx.socket(zmq::PUB).expect("Can not create socket!");
  let unlim : i32 = 1000000;
  data_socket.set_sndhwm(unlim).unwrap();
  match data_socket.bind(flight_address) {
    Err(err) => panic!("Can not bind to address {}! {}", flight_address, err),
    Ok(_)    => ()
  }
  info!("ZMQ PUB Socket for global data sink bound to {flight_address}");

  let mut writer : Option<TofPacketWriter> = None;
  if write_stream {
    let mut streamfile_name = write_stream_path + "/run_";
    streamfile_name += &runid.to_string();
    println!("==> Writing stream to file with prefix {}", streamfile_name);
    writer = Some(TofPacketWriter::new(streamfile_name));
  }
  //let mut event_cache = Vec::<TofPacket>::with_capacity(100); 

  let mut n_pack_sent = 0;
  //let mut last_evid   = 0u32;

  // for debugging/profiling
  let mut timer = Instant::now();
  loop {
    match incoming.recv() {
      Err(err) => trace!("No new packet, err {err}"),
      Ok(pack) => {
        debug!("Got new tof packet {}", pack.packet_type);
        if writer.is_some() {
          writer.as_mut().unwrap().add_tof_packet(&pack);
        }
        if print_moni_packets {
          let mut pos = 0;
          // some output to the console
          match pack.packet_type {
            PacketType::RBMoni => {
              let moni = RBMoniData::from_bytestream(&pack.payload, &mut pos);
              match moni {
                Ok(data) => {
                  debug!("Sending RBMoniData {}", data);
                },
                Err(err) => error!("Can not unpack RBMoniData! {err}")}
              }, 
            PacketType::MonitorTofCmp => {
              let moni = TofCmpMoniData::from_bytestream(&pack.payload, &mut pos);
              match moni {
                Ok(data) => {println!("{}", data);},
                Err(err) => error!("Can not unpack TofCmpData! {err}")}
              },
            PacketType::MonitorMtb => {
              let moni = MtbMoniData::from_bytestream(&pack.payload, &mut pos);
              match moni {
                Ok(data) => {println!("{}", data);},
                Err(err) => error!("Can not unpack MtbMoniData! {err}")}
              }, 
            _ => ()
          } // end match 
        }
        //if pack.packet_type == PacketType::TofEvent {
        //  if event_cache.len() != 100 {
        //    event_cache.push(pack);
        //    continue;
        //  } else {
        //    if n_pack_sent % 1000 == 0 && n_pack_sent != 0 {
        //      println!("=> [SINK] Sent {n_pack_sent}, last evid {last_evid} ===");
        //    }
        //    // sort the cache
        //    // FIXME - at this step, we should have checked if the 
        //    // packets are broken.
        //    event_cache.sort_by(| a, b|  TofEvent::extract_event_id_from_stream(&a.payload).unwrap().cmp(
        //                                &TofEvent::extract_event_id_from_stream(&b.payload).unwrap()));
        //   
        //    for ev in event_cache.iter() {
        //      last_evid = TofEvent::extract_event_id_from_stream(&ev.payload).unwrap();
        //      match data_socket.send(&ev.to_bytestream(),0) {
        //        Err(err) => error!("Not able to send packet over 0MQ PUB, {err}"),
        //        Ok(_)    => { 
        //          trace!("TofPacket sent");
        //          n_pack_sent += 1;
        //        }
        //      }
        //    }
        //    event_cache.clear();
        //  }

        //} else {
        // FIXME - disentangle network and disk I/O?
        match data_socket.send(pack.to_bytestream(),0) {
          Err(err) => error !("Not able to send packet over 0MQ PUB! {err}"),
          Ok(_)    => {
            trace!("TofPacket sent");
            n_pack_sent += 1;
          }
        } // end match
        //} // end else
      } // end if pk == event packet
    } // end incoming.recv 
    if n_pack_sent % 1000 == 0 {
      println!("[FLIGHT] Sent {n_pack_sent} TofPacket in {} sec!", timer.elapsed().as_secs());
      timer = Instant::now();
    }
  }
}
