use zmq;
use std::net::IpAddr;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

use local_ip_address::local_ip;

use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::commands as cmd;
use tof_dataclasses::serialization::Serialization;


fn main() {
  pretty_env_logger::init();
  let mut address_ip = String::from("tcp://127.0.0.1");
  let mut tp = TofPacket::new();
  let mut test = vec![1];
  tp.payload = test;
  let foo = tp.to_bytestream();
  println!("{foo:?}");
  test = Vec::new();
  for n in 0..256 {
    test.push(1);
  }
  tp.payload = test;
  //let foo = tp.to_bytestream();
  //println!("{foo:?}");
  //test = Vec::new();
  //for n in 0..65536 {
  //  test.push(1);
  //}
  //tp.payload = test;
  let foo = tp.to_bytestream();
  let tp2 = TofPacket::from_bytestream(&foo, 0);
  //println!("{foo:?}");
  //let this_board_ip = local_ip().unwrap();
  //match this_board_ip {
  //  IpAddr::V4(ip) => address_ip += &ip.to_string(),
  //  IpAddr::V6(ip) => panic!("Currently, we do not support IPV6!")
  //
  //    //  Ipv4Addr(address) => address_ip( 
  //}
  //// the port will be 38830 + board id
  //address_ip += "10.0.1.1";


  let port = 38830;
  let address : String = address_ip + ":" + &port.to_string();
  debug!("Will set up zmq socket at address {address}");
  let ctx = zmq::Context::new();
  let socket = ctx.socket(zmq::REP).expect("Unable to create 0MQ REP socket!");
  socket.bind(&address);

  
  loop {
    let bytes = socket.recv_bytes(0);
    println!("{bytes:?}");
    socket.send("[SRV] ACK",0);

  }

}
