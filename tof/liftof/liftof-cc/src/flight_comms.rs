//! Communication with the flight computer
//!
//! Using two dedicated 0MQ wires - one for 
//! data, the other for commands
//!
//!

use local_ip_address::local_ip;
use std::net::IpAddr;

use std::sync::mpsc::Receiver;
use crossbeam_channel as cbc; 

use tof_dataclasses::packets::{TofPacket,
                               PacketType};
use tof_dataclasses::events::TofEvent;

use liftof_lib::TofPacketWriter;

/// Manages "outgoing" 0MQ PUB socket
///
/// Everything should send to here, and 
/// then it gets passed on over the 
/// connection to the flight computer
pub fn global_data_sink(incoming : &cbc::Receiver<TofPacket>,
                        write_stream : bool) {

  let ctx = zmq::Context::new();
  let mut address_ip = String::from("tcp://");
  let this_ip = local_ip().unwrap();
  let data_port    = 40000;
  match this_ip {
    IpAddr::V4(ip) => address_ip += &ip.to_string(),
    IpAddr::V6(_) => panic!("Currently, we do not support IPV6!")
  }
  let data_address : String = address_ip + ":" + &data_port.to_string();

  // FIXME - should we just move to another socket if that one is not working?
  let data_socket = ctx.socket(zmq::PUB).expect("Can not create socket!");

  match data_socket.bind(&data_address) {
    Err(err) => panic!("Can not bind to address {}, Err {}", data_address, err),
    Ok(_)    => ()
  }
  info!("ZMQ PUB Socket for global data sink bound at {data_address}");

  let mut writer : Option<TofPacketWriter> = None;
  if write_stream {
    writer = Some(TofPacketWriter::new(String::from("stream")));
  }
  let mut event_cache = Vec::<TofPacket>::with_capacity(100); 

  let mut n_pack_sent = 0;
  let mut last_evid   = 0u32;
  loop {
    if n_pack_sent % 1000 == 0 && n_pack_sent != 0 {
      println!("=> [SINK] Sent {n_pack_sent}, last evid {last_evid} ===");
    }
    match incoming.recv() {
      Err(err) => trace!("No new packet, err {err}"),
      Ok(pack) => {
        if writer.is_some() {
          writer.as_mut().unwrap().add_tof_packet(&pack);
        }
        if pack.packet_type == PacketType::TofEvent {
          if event_cache.len() != 100 {
            event_cache.push(pack);
            continue;
          } else {
            // sort the cache
            // FIXME - at this step, we should have checked if the 
            // packets are broken.
            event_cache.sort_by(| a, b| TofEvent::get_evid_from_bytestream(&a.payload,0).unwrap().cmp(
                                        &TofEvent::get_evid_from_bytestream(&b.payload,0).unwrap()));
           
            for ev in event_cache.iter() {
              last_evid = TofEvent::get_evid_from_bytestream(&ev.payload,0).unwrap();
              match data_socket.send(&ev.to_bytestream(),0) {
                Err(err) => error!("Not able to send packet over 0MQ PUB"),
                Ok(_)    => { 
                  trace!("TofPacket sent");
                  n_pack_sent += 1;
                }
              }
            }
            event_cache.clear();
          }

        } else {
          match data_socket.send(pack.to_bytestream(),0) {
            Err(err) => warn!("Not able to send packet over 0MQ PUB"),
            Ok(_)    => {
              trace!("TofPacket sent");
              n_pack_sent += 1;
            }
          } // end match
        } // end else
      } // end if pk == event packet
    } // end incoming.recv 
    if n_pack_sent % 1000 == 0 {
      info!("Sent {n_pack_sent} TofPacket!");
    }
  }

}
