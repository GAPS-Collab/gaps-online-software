//! Command receiving, processing and sending
//! We need to fullfill the following requirements
//! 1) Receive a command from the flight computer/elsewhere
//! 2) Parse it
//! 3) In case we can execute it, execute
//! 4) Otherwise pass on to proper receipient
//! 5) Achknowledge

use std::path::Path;
use std::thread;
use std::sync::{
    Arc,
    Mutex,
};
use std::fs::{
    OpenOptions,
};

use std::io::Write;
use std::time::{
    Instant,
    Duration,
};

use crossbeam_channel::{
    Receiver,
    Sender
};

use tof_dataclasses::threading::ThreadControl;
use tof_dataclasses::constants::PAD_CMD_32BIT;
use tof_dataclasses::commands::{
    TofCommand,
    RBCommand,
    //TofCommandCode,
    TofResponse,
    TofResponseCode
};
use tof_dataclasses::packets::{
    PacketType,
    TofPacket
};
use tof_dataclasses::serialization::{
    Serialization,
    Packable
};

use liftof_lib::settings::CommandDispatcherSettings;

/// The command dispatcher listens for incoming commands and either executes
/// them or passes them on to the intended receiver
/// 
/// The acknowledgement packets will be just put into the general data stream
/// and then be further processed by the receiver of that stream. This means,
/// when we are taking data, they will be also logged to the disks on file
///
/// # Arguments:
///
/// * settings        : Configure command_dispatcher with .toml config file
/// 
/// * thread_ctrl     : Interface with main program loop. E.g. shutdown,
///                     heartbeat signals.
/// * tof_ack_sender  : A channel in which we are putting the acknowledgement
///                     packets so that they can be further processed. 
///                     This channel should connect to a data sink.
/// * rb_ack_recv     : Receive RB acknowledgements over this channel                
pub fn command_dispatcher(settings        : CommandDispatcherSettings,
                          thread_ctrl     : Arc<Mutex<ThreadControl>>,
                          tof_ack_sender  : Sender<TofPacket>, 
                          rb_ack_recv     : Receiver<TofResponse>) {
  
  let fc_sub_addr = settings.fc_sub_address.clone();
  let cc_pub_addr = settings.cc_server_address.clone();
  let ctx = zmq::Context::new();
  
  // socket to receive commands
  info!("Connecting to flight computer at {}", fc_sub_addr);
  let cmd_receiver = ctx.socket(zmq::SUB).expect("Unable to create 0MQ SUB socket!");
  cmd_receiver.set_subscribe(b"").expect("Unable to subscribe to empty topic!");
  cmd_receiver.connect(&fc_sub_addr).expect("Unable to subscribe to flight computer PUB");
  info!("ZMQ SUB Socket for flight cpu listener bound to {fc_sub_addr}");

  // socket to send commands on the RB network
  info!("Binding socket for command dispatching to rb network to {}", cc_pub_addr);
  let cmd_sender = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  cmd_sender.bind(&cc_pub_addr).expect("Unable to bind to (PUB) socket!");

  // open the logfile for commands
  let mut filename = settings.cmd_log_path.clone();
  if !filename.ends_with("/") {
    filename += "/";
  }
  filename        += "received-commands.log";
  let path         = Path::new(&filename);
  info!("Writing cmd log to file {filename}");
  let mut log_file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");

  let sleep_time   = Duration::from_secs(settings.cmd_listener_interval_sec);
  let mut locked   = settings.deny_all_requests; // do not allow the reception of commands if true

  loop {
    // check if we get a command from the main 
    // thread
    match thread_ctrl.lock() {
      Ok(mut tc) => {
        if tc.stop_flag {
          info!("Received stop signal. Will stop thread!");
          info!("Will end all Run activity on the RBs and send >>StopRun<< signal to all RBs!");
          let payload: u32 = PAD_CMD_32BIT | (255u32);
          let run_stop = TofCommand::DataRunStop(payload);
          let tp  = run_stop.pack();
          let ack : TofResponse;
          let payload = tp.zmq_payload_brdcast();
          match cmd_sender.send(payload,0) {
            Err(err) => {
              error!("Unable to send command, error{err}");
              ack = TofResponse::ZMQProblem(0);
            },
            Ok(_)    => {
              debug!("Stop run command sent");
              ack = TofResponse::Success(TofResponseCode::RespSuccFingersCrossed as u32);
            }
          }
          let tp_ack = ack.pack();
          match tof_ack_sender.send(tp_ack) {
            Err(err) => {
              error!("Unable to send ack packet! {err}");
            }
            Ok(_) => ()  
          }
          break;
        }
        tc.thread_cmd_dispatch_active = true;
      }
      Err(err) => {
        trace!("Can't acquire lock! {err}");
      },
    }
    // check if we get a command from the main 
    // thread
    match cmd_receiver.recv_bytes(zmq::DONTWAIT) {
      Err(_)   => {
        // no need for error catching, typically it 
        // just means that nobody sent anythin
        trace!("No cmd received! Sleeping for {:?}...", sleep_time);
        // take out the heat and sleep a bit
        thread::sleep(sleep_time);
        continue;
      }
      Ok(mut buffer) => {
        // identfiy if we have a GAPS packet
        if buffer[0] == 0xeb && buffer[1] == 0x90 {
          // We have a GAPS packet -> FIXME:
          error!("GAPS packet command receiving not supported yet! Currently, we can only process TofPackets!");
          // strip away the GAPS header!  
          continue;
        } 
        match TofPacket::from_bytestream(&buffer, &mut 1) {
          Err(err) => {
            error!("Unable to decode bytestream! {:?}", err);
            continue;  
          },
          Ok(packet) => {
            let mut resp = TofResponse::Unknown;
            match packet.packet_type {
              PacketType::TofCommand => {
                // Here we have to decide - is that command for us, or for the RBs?
                let cmd : TofCommand = packet.unpack().unwrap(); // we just checked if
                                                                 // that is really a
                                                                 // TofCommand
                let write_to_file = format!("{}", cmd);
                match log_file.write_all(&write_to_file.into_bytes()) {
                  Err(err) => {
                    error!("Writing to file to path {} failed! {}", filename, err)
                  }
                  Ok(_)    => ()
                }
                match log_file.sync_all() {
                  Err(err) => {
                    error!("Unable to sync file to disc! {err}");
                  },
                  Ok(_) => ()
                }

                // intercept commands when we are in lockdown
                if locked {
                  let mut resp = TofResponse::Success(0);
                  if cmd == TofCommand::Unlock(81) {
                    locked = false;
                  } else {
                    resp = TofResponse::AccessDenied(403u32);
                  }
                  info!("Command requested, but we have locked down the reception of commands!");
                  let ack_tp = resp.pack();
                  match tof_ack_sender.send(ack_tp) {
                    Err(err) => {
                      error!("Unable to send ACK packet! {err}");
                    }
                    Ok(_)    => ()
                  }
                  continue;
                }

                match cmd {
                  TofCommand::Kill(_value) => {
                    // FIXME - end all threads, maybe end run?
                  }
                  TofCommand::Lock(_value) => {
                    locked = true;
                    let resp = TofResponse::Success(0);
                    let ack_tp = resp.pack();
                    match tof_ack_sender.send(ack_tp) {
                      Err(err) => {
                        error!("Unable to send ACK packet! {err}");
                      }
                      Ok(_)    => ()
                    }
                  }
                  TofCommand::DataRunStart (_value) => {
                    info!("Received data run start command");
                    // technically, it is run_typ, rb_id, event number
                    // all to the max means run start for all
                    // let payload: u32 =  PAD_CMD_32BIT | (255u32) << 16 | (255u32) << 8 | (255u32);
                    // We don't need this - just need to make sure it gets broadcasted
                    let mut payload  = String::from("BRCT").into_bytes();
                    payload.append(&mut packet.to_bytestream());
                    match cmd_sender.send(&payload, 0) {
                      Err(err) => {
                        error!("Unable to send command, error{err}");
                        resp = TofResponse::ZMQProblem(0x0); // response code not assigned, 
                                                                 // let's just let it be 0 for now
                        let ack_tp = resp.pack();
                        match tof_ack_sender.send(ack_tp) {
                          Err(err) => {
                            error!("Unable to send ACK packet! {err}");
                          }
                          Ok(_)    => ()
                        }
                      },
                      Ok(_)    => {
                        info!("Start run command sent");
                        // Now we wait for the RB acknowledgement packets and see if our command
                        // went through
                        let mut n_rb_ack_rcved = 0u8;
                        let run_start_timeout  = Instant::now();
                        // let's wait 20 seconds here
                        resp = TofResponse::TimeOut(0x0);
                        while run_start_timeout.elapsed().as_secs() < 20 {
                          match rb_ack_recv.recv() {
                            Err(_) => {
                              continue;
                            }
                            Ok(_ack_pack) => {
                              //FIXME - do something with it
                              n_rb_ack_rcved += 1;
                            }
                          }
                          if n_rb_ack_rcved == 40 {
                            resp = TofResponse::Success(0);
                          }
                        }
                        let ack_tp = resp.pack();
                        match tof_ack_sender.send(ack_tp) {
                          Err(err) => {
                            error!("Unable to send ACK packet! {err}");
                          }
                          Ok(_)    => ()
                        }
                      }
                    }
                  }
                  //TofCommand::Ping (_value) => {
                  //TofCommand::Ping (_value) => {
                  TofCommand::Ping (_value) => {
                    //info!("Received ping command");
                    //// MSB third 8 bits are
                    //let tof_component: TofComponent = TofComponent::from(((value & (MASK_CMD_8BIT << 8)) >> 8) as u8);
                    //// MSB fourth 8 bits are 
                    //let id: u8 = (value & MASK_CMD_8BIT) as u8;
                    //if tof_component == TofComponent::Unknown {
                    //  info!("The command is not valid for {}", TofComponent::Unknown);
                    //  // The packet was not for this RB so bye
                    //  continue;
                    //} else {
                    //  match tof_component {
                    //    TofComponent::TofCpu => return_val = crate::send_ping_response(resp_socket),
                    //    TofComponent::RB  |
                    //    TofComponent::LTB |
                    //    TofComponent::MT     => return_val = crate::send_ping(resp_socket, outgoing_c,  tof_component, id),
                    //    _                    => {
                    //      error!("The ping command is not implemented for this TofComponent!");
                    //      return_val = Err(CmdError::NotImplementedError);
                    //    }
                    //  }
                    //}
                  }
                  _ => {
                    error!("Command {} is currently not implemented!", cmd); 
                    let ack_tp = resp.pack();
                    match tof_ack_sender.send(ack_tp) {
                      Err(err) => {
                        error!("Unable to send ACK packet! {err}");
                      }
                      Ok(_)    => ()
                    }
                  }
                }// end match cmd
              }
              // deprecated, should go away, kept here right now
              // for compatiblity and not sure what it is neeeded for
              PacketType::RBCommand => {
                debug!("Received RBCommand!");
                match RBCommand::from_bytestream(&packet.payload, &mut 0) {
                  Ok(rb_cmd) => {
                    let mut payload = format!("RB{:2}", rb_cmd.rb_id).into_bytes();
                    // We just relay the whole thing, the only use of the excercise
                    // here was to get the RBID from within the packet and slab it 
                    // onto its front
                    //let payload = &rb_topic.into_bytes() + buffer.to_slice();
                    payload.append(&mut buffer);
                    match cmd_sender.send(&payload,0) {
                      Err(err) => error!("Unable to send command {}! Error {err}", rb_cmd),
                      Ok(_)    => trace!("Sent RBCommand {}", rb_cmd)
                    }
                  }
                  Err(err) => {
                    error!("Can not construct RBCommand, error {err}");
                  }
                }
              },
              _ => {
                error!("Received garbage package! {}", packet);
              }
            }// end match
          }
        }
        // now we have several options
      }
    }
  }
}

///// Broadcast commands over the tof-computer network
///// socket via zmq::PUB to the rb network.
///// Currently, the only participants in the rb network
///// are the readoutboards.
/////
///// After the reception of a TofCommand, this will currently be 
///// broadcasted to all readout boards.
/////
///// ISSUE/FIXME  : Send commands only to specific addresses.
/////
///// # Arguments 
/////
///// * cmd        : a \[crossbeam\] receiver, to receive 
/////                TofCommands.
//pub fn readoutboard_commander(cmd : &Receiver<TofPacket>){
//  debug!(".. started!");
//  let this_board_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 1, 1));
//
//  let address_ip;
//  match this_board_ip {
//    IpAddr::V4(ip) => address_ip = ip.to_string().clone(),
//    IpAddr::V6(_) => panic!("Currently, we do not support IPV6!")
//  }
//  let data_address : String = build_tcp_from_ip(address_ip,DATAPORT.to_string());
//  let data_socket = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
//  data_socket.bind(&data_address).expect("Unable to bind to data (PUB) socket {data_adress}");
//  println!("==> 0MQ PUB socket bound to address {data_address}");
//  loop {
//    // check if we get a command from the main 
//    // thread
//    match cmd.try_recv() {
//      Err(err) => trace!("Did not receive a new command, error {err}"),
//      Ok(packet) => {
//        // now we have several options
//        match packet.packet_type {
//          PacketType::TofCommand => {
//            info!("Received TofCommand! Broadcasting to all TOF entities who are listening!");
//            let mut payload  = String::from("BRCT").into_bytes();
//            payload.append(&mut packet.to_bytestream());
//            match data_socket.send(&payload,0) {
//              Err(err) => error!("Unable to send command! Error {err}"),
//              Ok(_)    => info!("BRCT command sent!")
//            }
//          },
//          PacketType::RBCommand => {
//            debug!("Received RBCommand!");
//            let mut payload_str  = String::from("RB");
//            match RBCommand::from_bytestream(&packet.payload, &mut 0) {
//              Ok(rb_cmd) => {
//                let to_rb_id = rb_cmd.rb_id;
//                if rb_cmd.rb_id < 10 {
//                  payload_str += &String::from("0");
//                  payload_str += &to_rb_id.to_string();
//                } else {
//                  payload_str += &to_rb_id.to_string();
//                }
//
//                let mut payload = payload_str.into_bytes();
//                payload.append(&mut packet.to_bytestream());
//                match data_socket.send(&payload,0) {
//                  Err(err) => error!("Unable to send command {}! Error {err}", rb_cmd),
//                  Ok(_)    => debug!("Making event request! {}", rb_cmd)
//                }
//              }
//              Err(err) => {
//                error!("Can not construct RBCommand, error {err}");
//              }
//            }
//          },
//          _ => {
//            error!("Received garbage package! {}", packet);
//          }
//        }// end match
//      }
//    }
//  }
//}
//
//
//}
