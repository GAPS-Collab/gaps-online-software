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

use tof_dataclasses::packets::TofPacket;

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
  info!("ZMQ PUB Socket for globa data sink bound at {data_address}");

  let mut writer : Option<TofPacketWriter> = None;
  if write_stream {
    writer = Some(TofPacketWriter::new(String::from("stream")));
  }
  loop {
    match incoming.recv() {
      Err(err) => trace!("No new packet, err {err}"),
      Ok(pack) => {
        if writer.is_some() {
          writer.as_mut().unwrap().add_tof_packet(&pack);
        }
        match data_socket.send(pack.to_bytestream(),0) {
          Err(err) => warn!("Not able to send packet over 0MQ PUB"),
          Ok(_)    => trace!("TofPacket sent")
        }
      }
    }
  }

}
