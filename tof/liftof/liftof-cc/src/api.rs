//! API for liftof-cc, these are basically the individual threads
//!
//!

//use crossbeam_channel::Sender;
//use liftof_lib::ReadoutBoard;
use std::net::{IpAddr, Ipv4Addr};
use zmq;
use local_ip_address::local_ip;

use tof_dataclasses::manifest::ReadoutBoard;
use tof_dataclasses::commands::TofCommand;

pub const DATAPORT : u32 = 42000;


/// This is listening to commands from the flight computer 
/// and relays them to the RadoutBoards
/// 
/// # Arguments 
///
/// * rbs 
/// * rp_to_main
pub fn commander(rbs : &Vec<ReadoutBoard>){
                 //rp_to_main : &Sender<RunParams>) {
             

  let ctx = zmq::Context::new();
  //let mut sockets = Vec::<zmq::Socket>::new();

  let mut address_ip = String::from("tcp://");
  //let this_board_ip = local_ip().expect("Unable to obtainl local board IP. Something is messed up!");
  let data_port    = DATAPORT;
  let this_board_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 1, 1));

  match this_board_ip {
    IpAddr::V4(ip) => address_ip += &ip.to_string(),
    IpAddr::V6(_) => panic!("Currently, we do not support IPV6!")
  }
  let data_address : String = address_ip.clone() + ":" + &data_port.to_string();
  let data_socket = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  data_socket.bind(&data_address).expect("Unable to bind to data (PUB) socket {data_adress}");
  info!("0MQ PUB socket bound to address {data_address}");
  let init_run = TofCommand::DataRunStart(100000);
  let mut payload_cmd  = init_run.to_bytestream();
  let mut payload  = String::from("BRCT").into_bytes();
  payload.append(&mut payload_cmd);

  match data_socket.send(&payload,0) {
    Err(err) => error!("Can not start run! Error {err}"),
    Ok(_)    => ()
  }
  //for rb in rbs.iter() {
  //  let sock = ctx.socket(zmq::REQ).expect("Unable to create socket!");
  //  let address = "tcp://".to_owned()
  //            + &rb.ip_address.expect("No IP known for this board!").to_string()
  //            + ":"
  //            +  &rb.cmd_port.expect("No CMD port known for this board!").to_string();
  //  sock.connect(&address);
  //  sockets.push(sock);
  //}
  //let init_run = TofCommand::DataRunStart(100000);
  ////let init_run = RunParams::new();
  //for s in sockets.iter() {
  //  match s.send(init_run.to_bytestream(), 0) {
  //    Err(err) => warn!("Could not initalize run, err {err}"),
  //    Ok(_)    => info!("Initialized run!")
  //  }
  //}
}

