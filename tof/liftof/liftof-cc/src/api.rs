//! API for liftof-cc, these are basically the individual threads
//!
//!

//use crossbeam_channel::Sender;
use liftof_lib::ReadoutBoard;
use zmq;
use tof_dataclasses::commands::TofCommand;

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
  let mut sockets = Vec::<zmq::Socket>::new();

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

