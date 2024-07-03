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

//use tof_dataclasses::threading::ThreadControl;
use tof_dataclasses::config::{
    RunConfig,
    TriggerConfig,
    AnalysisEngineConfig
};
use tof_dataclasses::constants::PAD_CMD_32BIT;
use tof_dataclasses::commands::{
    TofCommand,
    TofCommandV2,
    TofCommandCode,
    TofResponse,
    TofResponseCode
};
use tof_dataclasses::packets::{
    PacketType,
    TofPacket
};
use tof_dataclasses::serialization::{
    Serialization,
    Packable,
    SerializationError
};

use liftof_lib::settings::CommandDispatcherSettings;
use liftof_lib::thread_control::ThreadControl;

use liftof_lib::constants::{
    DEFAULT_CALIB_VOLTAGE,
    DEFAULT_RB_ID,
    DEFAULT_CALIB_EXTRA
};

const MAX_CALI_TIME : u64 = 360; // calibration should be done within 6 mins?


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
  let mut log_file = OpenOptions::new().create(true).append(true).open("received-commands.log").expect("Unable to create file!");
  match OpenOptions::new().create(true).append(true).open(path) {
    Ok(_f) => {log_file = _f;},
    Err(err) => { 
      error!("Unable to write to path {filename}! {err} Falling back to default file path");
    }
  }

  let sleep_time   = Duration::from_secs(settings.cmd_listener_interval_sec);
  let mut locked   = settings.deny_all_requests; // do not allow the reception of commands if true

  loop {
    // check if we get a command from the main 
    // thread
    thread::sleep(sleep_time);
    match thread_ctrl.try_lock() {
      Ok(mut tc) => {
        //println!("== ==> [cmd_dispatcher] tc locked!");
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
        continue;
      }
      Ok(buffer) => {
        // identfiy if we have a GAPS packet
        if buffer[0] == 0xeb && buffer[1] == 0x90 {
          // We have a GAPS packet -> FIXME:
          error!("GAPS packet command receiving not supported yet! Currently, we can only process TofPackets!");
          // strip away the GAPS header!  
          continue;
        } 
        match TofPacket::from_bytestream(&buffer, &mut 0) {
          Err(err) => {
            error!("Unable to decode bytestream! {:?}", err);
            continue;  
          },
          Ok(packet) => {
            let mut resp = TofResponse::Unknown;
            match packet.packet_type {
              PacketType::TofCommandV2 => {
                let mut cmd = TofCommandV2::new();
                match packet.unpack::<TofCommandV2>() {
                  Ok(_cmd) => {cmd = _cmd;},
                  Err(err) => error!("Unable to decode TofCommand! {err}")
                }
                let now = Instant::now();
                let write_to_file = format!("{:?}: {}\n",now, cmd);
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
                  //if cmd == TofCommand::Unlock(81) {
                  if cmd.command_code == TofCommandCode::Unlock {
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

                match cmd.command_code {
                  TofCommandCode::Kill => {
                    match thread_ctrl.lock() {
                      Ok(mut tc) => {
                        //println!("== ==> [cmd_dispatcher] tc locked!");
                        tc.stop_flag = true;
                      }
                      Err(err) => error!("Unable to lock thread-control! {err}")
                    }
                  }
                  TofCommandCode::Lock => {
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
                  TofCommandCode::DataRunStart => {
                    let mut run_id : u32 = 0;
                    println!("== ==> Received DataRunStart!");
                    info!("Received data run start command");
                    // technically, it is run_typ, rb_id, event number
                    // all to the max means run start for all
                    // let payload: u32 =  PAD_CMD_32BIT | (255u32) << 16 | (255u32) << 8 | (255u32);
                    // make sure the relevant threads are active
                    // We don't need this - just need to make sure it gets broadcasted
                    match RunConfig::from_bytestream(&cmd.payload, &mut 0) {
                      Err(err) => error!("Unable to unpack run config! {err}"),
                      Ok(pld) => {
                        run_id = pld.runid;
                      }
                    }
                    match thread_ctrl.lock() {
                      Ok(mut tc) => {
                        tc.thread_master_trg_active  = true;
                        tc.thread_monitoring_active  = true;
                        tc.thread_event_bldr_active  = true;
                        tc.calibration_active        = false;
                        tc.run_id                    = run_id;
                        tc.write_data_to_disk        = true;
                        tc.new_run_start_flag        = true;
                      },
                      Err(err) => {
                        error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
                      },
                    }
                    let cmd_payload: u32 =  PAD_CMD_32BIT | (255u32) << 16 | (255u32) << 8 | (255u32);
                    let cmd          = TofCommand::DataRunStart(cmd_payload);
                    let packed_cmd   = cmd.pack();
                    let mut payload  = String::from("BRCT").into_bytes();
                    payload.append(&mut packed_cmd.to_bytestream());
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
                  TofCommandCode::Ping => {
                    info!("Received ping command");
                    let cmd          = TofCommand::Ping(0x0);
                    let packed_cmd   = cmd.pack();
                    let mut payload  = String::from("BRCT").into_bytes();
                    payload.append(&mut packed_cmd.to_bytestream());
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
                        info!("Ping command sent");
                        println!("=> TOF CPU responds to ping!");
                      }
                    }
                  }
                  TofCommandCode::RBCalibration => {
                    let voltage_level = DEFAULT_CALIB_VOLTAGE;
                    let rb_id         = DEFAULT_RB_ID;
                    let extra         = DEFAULT_CALIB_EXTRA;
                    println!("=> Received calibration default command! Will init calibration run of all RBs...");
                    let cmd_payload: u32
                      = (voltage_level as u32) << 16 | (rb_id as u32) << 8 | (extra as u32);
                    let default_calib = TofCommand::DefaultCalibration(cmd_payload);
                    let tp = default_calib.pack();
                    let mut payload  = String::from("BRCT").into_bytes();
                    payload.append(&mut tp.to_bytestream());
                    
                    match cmd_sender.send(&payload, 0) {
                      Err(err) => {
                        error!("Unable to send command, error{err}");
                      },
                      Ok(_) => {
                        println!("=> Calibration  initialized!");
                      }
                    }
                    println!("=> .. now we need to wait until the calibration is finished!");
                    // if that is successful, we need to wait
                    match thread_ctrl.lock() {
                      Ok(mut tc) => {
                        // deactivate the master trigger thread
                        tc.thread_master_trg_active  = false;
                        tc.thread_monitoring_active  = false;
                        tc.thread_event_bldr_active  = false;
                        tc.calibration_active = true;
                      },
                      Err(err) => {
                        error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
                      },
                    }
                    // this halts the thread while we are doing calibrations
                    let mut cali_received = 0u32;
                    let cali_timeout   = Instant::now();
                    let cali_sleeptime = Duration::from_secs(30); 
                    loop {
                      thread::sleep(cali_sleeptime);
                      match thread_ctrl.lock() {
                        Err(err)   => error!("Unable to acquire lock for thread ctrl! {err}"),
                        Ok(mut tc) => {
                          if tc.calibration_active {
                            let mut changed_keys = Vec::<u8>::new();
                            for rbid in tc.finished_calibrations.keys() {
                              // the global data sink sets these flags
                              if tc.finished_calibrations[&rbid] {
                                cali_received += 1;
                                changed_keys.push(*rbid);
                                println!("==> Received RBCalibration for board {rbid}");
                              }
                            }
                            for k in changed_keys {
                              // it got registered, now reset it 
                              *tc.finished_calibrations.get_mut(&k).unwrap() = false;
                            }
                            if cali_received  == tc.n_rbs || cali_timeout.elapsed().as_secs() >= MAX_CALI_TIME {
                              // re-enable the threads
                              tc.thread_master_trg_active = true;
                              tc.thread_monitoring_active = true;
                              tc.calibration_active = false;
                              tc.thread_event_bldr_active  = true;
                              if cali_timeout.elapsed().as_secs() > MAX_CALI_TIME {
                                error!("Calibration not finished, however, we give up since {} seconds have passed which ssems too long!", MAX_CALI_TIME);
                              }
                              println!("== ==> Calibration finished with {} of {} boards!", cali_received, tc.n_rbs);
                              info!("Calibration finished!");
                              break; 
                            }
                          }
                        } // end Ok
                      } // end match
                    } // end loop
                  }
                  TofCommandCode::SetMTConfig => {
                    match thread_ctrl.lock() {
                      Err(err)   => error!("Unable to acquire lock for thread ctrl! {err}"),
                      Ok(mut tc) => {
                        match TriggerConfig::from_bytestream(&packet.payload, &mut 0) {
                          Err(err) => error!("Unable to decode TriggerConfig!"),
                          Ok(config) => {
                            tc.liftof_settings.mtb_settings.trigger_prescale=config.prescale;
                            tc.liftof_settings.mtb_settings.trigger_type=config.trigger_type;
                            tc.liftof_settings.mtb_settings.gaps_trigger_use_beta=config.gaps_trigger_use_beta;
                            tc.liftof_settings.mtb_settings.tiu_emulation_mode=config.tiu_emulation_mode;
                          }
                        }
                      }
                    }
                  }
                  TofCommandCode::SetAnalysisEngineConfig => {
                    match thread_ctrl.lock() {
                      Err(err)   => error!("Unable to acquire lock for thread ctrl! {err}"),
                      Ok(mut tc) => {
                        match AnalysisEngineConfig::from_bytestream(&packet.payload, &mut 0) {
                          Ok(config: AnalysisEngineConfig) => {
                          tc.liftof_settings.analysis_engine_settings.integration_start=config.integration_start;
                          tc.liftof_settings.analysis_engine_settings.integration_window=config.integration_window;
                          tc.liftof_settings.analysis_engine_settings.pedestal_thresh=config.pedestal_thresh;
                          tc.liftof_settings.analysis_engine_settings.pedestal_begin_bin=config.pedestal_begin_bin;
                          tc.liftof_settings.analysis_engine_settings.pedestal_win_bins=config.pedestal_win_bins;
                          tc.liftof_settings.analysis_engine_settings.use_zscore=config.use_zscore;
                          tc.liftof_settings.analysis_engine_settings.find_pks_t_start=config.find_pks_t_start;
                          tc.liftof_settings.analysis_engine_settings.find_pks_t_window=config.find_pks_t_window;
                          tc.liftof_settings.analysis_engine_settings.min_peak_size=config.min_peak_size;
                          tc.liftof_settings.analysis_engine_settings.max_oeaks=config.max_peaks;
                          tc.liftof_settings.analysis_engine_settings.find_pks_thresh=config.find_pks_thresh;
                          tc.liftof_settings.analysis_engine_settings.cfd_fraction=config.cfd_fraction;
                          }  
                        }
                      }
                    }
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

