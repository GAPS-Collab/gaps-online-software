//! Readoutboard communication. Get events and 
//! monitoring data

use std::collections::HashMap;

use std::sync::{
  Arc,
  Mutex,
};

use std::time::{
  Instant,
  //Duration,
};

use crossbeam_channel::Sender;

use tof_dataclasses::database::ReadoutBoard;
use tof_dataclasses::events::RBEvent;
use tof_dataclasses::packets::{
  TofPacket,
  PacketType,
};

use tof_dataclasses::serialization::{
  Serialization,
  Packable
};
use tof_dataclasses::calibrations::{
  RBCalibrations,
  RBCalibrationsFlightT,
  RBCalibrationsFlightV,
  };

use liftof_lib::{
  waveform_analysis,
};

use liftof_lib::thread_control::ThreadControl;
use liftof_lib::settings::AnalysisEngineSettings;

/*************************************/

/// Receive data from a readoutboard
///
/// In case of binary event data ("blob") this can be analyzed here
/// It is also possible to save the data directly.
///
/// In case of monitoring/other tof packets, those will be forwarded
///
/// # Arguments:
///
/// * ev_to_builder       : This thread will receive RBEvent data from the assigned RB, 
///                         if desired (see run_analysis_engine) run analysis and extract
///                         TofHits and then pass the result on to the event builder.
/// * tp_to_sink          : Channel which should be connect to a (global) data sink.
///                         Packets which are of not event type (e.g. header/full binary data)
///                         will be forwarded to the sink.
/// * rb                  : ReadoutBoard instance, as loaded from the database. This will be used
///                         for readoutboard id as well as paddle assignment.
/// * ae_settings         : Settings to configure peakfinding algorithms etc. 
///                         These can be configured with an external .toml file
pub fn readoutboard_communicator(ev_to_builder       : Sender<RBEvent>,
                                 tp_to_sink          : Sender<TofPacket>,
                                 mut rb              : ReadoutBoard,
                                 thread_control      : Arc<Mutex<ThreadControl>>) {

  let mut this_status = HashMap::<u16, bool>::new();
  for k in 1..321 {
    this_status.insert(k,false);
  }
  match rb.load_latest_calibration() {
    Err(err) => error!("Unable to load calibration for RB {}! {}", rb.rb_id, err),
    Ok(_)    => {
      info!("Loaded calibration for board {} successfully!", rb.rb_id);
    }
  }

  let zmq_ctx = zmq::Context::new();
  let board_id = rb.rb_id; //rb.id.unwrap();
  info!("initializing RB thread for board {}!", board_id);
  let mut n_errors        = 0usize;
  // how many chunks ("buffers") we dealt with
  let mut n_chunk  = 0usize;
  // in case we want to do calibratoins
  let address = rb.guess_address();

  // FIXME - this panics, however, if we can't set up the socket, what's 
  // the point of this thread?
  let socket = zmq_ctx.socket(zmq::SUB).expect("Unable to create socket!");
  match socket.connect(&address) {
    Err(err) => error!("Can not connect to socket {}, {}", address, err),
    Ok(_)    => info!("Connected to {address}")
  }
  // no need to subscribe to a topic, since there 
  // is one port for each rb
  let topic = format!("RB{:02}", board_id);
  match socket.set_subscribe(&topic.as_bytes()) {
   Err(err) => error!("Unable to subscribe to topic! {err}"),
   Ok(_)    => info!("Subscribed to {:?}!", topic),
  }
  let mut tc_timer = Instant::now();
  let mut verification_active = false;
  
  let ae_settings         : AnalysisEngineSettings; 
  let run_analysis_engine : bool;
  match thread_control.lock() {
    Ok(tc) => {
      ae_settings         = tc.liftof_settings.analysis_engine_settings.clone();
      run_analysis_engine = tc.liftof_settings.run_analysis_engine;
    }
    Err(err) => {
      error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
      error!("Ending thread, unable to acquire settings!");
      return;
    }
  }
  if run_analysis_engine {
    info!("Will run analysis engine!");
    //println!("Will use the following settings! {}", ae_settings);
  } else {
    warn!("Will not run analysis engine!");
  }

  // start continuous thread activity, read data from RB sockets,
  // do analysis and pass them on.
  loop {
    if tc_timer.elapsed().as_secs_f32() > 2.1 {
      match thread_control.try_lock() {
        Ok(mut tc) => {
          //println!("== ==> [rbcomm] tc lock acquired!");
          if tc.end_all_rb_threads {
            //println!("= => [rbcomm] initiate ending thread for RB {}!", board_id);
            tc.thread_rbcomm_active.insert(rb.rb_id,false);
            // check if all threads have ended
            let mut all_done = true;
            for (_,value) in &tc.thread_rbcomm_active {
              if *value {
                all_done = false;
              }
            }
            if all_done {
              tc.thread_event_bldr_active = false;
            }
            break;
          }
          verification_active = tc.verification_active;
          if verification_active {
            tc.detector_status.update_from_map(this_status.clone());
          }
        },
        Err(err) => {
          error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
        },
      }
      tc_timer = Instant::now();
    }
    // check if we got new data
    // this is blocking the thread
    match socket.recv_bytes(0) {
      Err(err) => {
        n_errors += 1;
        error!("Receiving from socket raised error {}", err);
      }
      Ok(buffer) => {
        // strip the first 4 bytes, since they contain the 
        // board id
        match TofPacket::from_bytestream(&buffer, &mut 4) { 
          Err(err) => {
            error!("Unknown bytestream...{:?}", err);
            continue;  
          },
          Ok(tp) => {
            //n_received += 1;
            match tp.packet_type {
              PacketType::RBEvent | PacketType::RBEventMemoryView => {
                let mut event = RBEvent::from(&tp);
                // don't create the hits if the trigger is lost (the 
                // waveform field will be empty)
                if event.hits.len() == 0 
                && !event.header.drs_lost_trigger() 
                && run_analysis_engine {
                  match waveform_analysis(&mut event, 
                                          &rb,
                                          ae_settings) {
                    Ok(_) => (),
                    Err(err) => {
                      warn!("Unable to analyze waveforms for this event! {err}");
                    }
                  }
                }
                if verification_active {
                  for h in &event.hits {
                    // average charge/peak hit
                    let verification_charge_threshhold = 10.0f32;
                    if h.get_charge_a() >= verification_charge_threshhold {
                      let status_key = h.paddle_id as u16;
                      match this_status.insert(status_key, true) {
                        Some(_) => (),
                        None => error!("Unknown paddle id! {}", h.paddle_id)
                      }
                    }
                    if h.get_charge_b() >= verification_charge_threshhold {
                      let status_key = (h.paddle_id as u16) + 160;
                      match this_status.insert(status_key, true) {
                        Some(_) => (),
                        None => error!("Unknown paddle id! {}", h.paddle_id)
                      }
                    }
                  }
                }
                if !verification_active {
                  match ev_to_builder.send(event) {
                    Ok(_) => (),
                    Err(err) => {
                      error!("Unable to send event! Err {err}");
                    }
                  }
                }
                //n_events += 1;
                n_chunk += 1;
              } 
              PacketType::RBCalibration => {
                let mut flight_v = RBCalibrationsFlightV::new();
                let mut flight_t = RBCalibrationsFlightT::new(); 

                match tp.unpack::<RBCalibrations>() {
                  Ok(cali) => {
                    flight_v = cali.emit_flightvcal();
                    flight_t = cali.emit_flighttcal(); 
                    //println!("= => [rb_comm] Received RBCalibration!");
                    match thread_control.lock() {
                      Ok(mut tc) => {
                        tc.calibrations.insert(board_id, cali.clone()); 
                        *tc.finished_calibrations.get_mut(&board_id).unwrap() = true; 
                        rb.calibration = cali;
                      }
                      Err(err) => {
                        error!("Can't acquire lock for ThreadControl!! {err}");
                      },
                    }
                  }
                  Err(err) => {
                    error!("Received calibration package, but got error when unpacking! {err}");
                  }
                }
                let flight_v_tp = flight_v.pack();
                let flight_t_tp = flight_t.pack();
                match tp_to_sink.send(flight_v_tp) {
                  Err(err) => error!("Can not send tof packet to data sink! Err {err}"),
                  Ok(_)    => debug!("Packet sent"),
                }
                match tp_to_sink.send(flight_t_tp) {
                  Err(err) => error!("Can not send tof packet to data sink! Err {err}"),
                  Ok(_)    => debug!("Packet sent"),
                }
                match tp_to_sink.send(tp) {
                  Err(err) => error!("Can not send tof packet to data sink! Err {err}"),
                  Ok(_)    => debug!("Packet sent"),
                }
              }
              _ => {
                // Currently, we will just forward all other packets
                // directly to the data sink
                match tp_to_sink.send(tp) {
                  Err(err) => error!("Can not send tof packet to data sink! Err {err}"),
                  Ok(_)    => debug!("Packet sent"),
                }
              }
            } // end match packet type
          } // end OK
        } // end match from_bytestream
      } // end ok buffer 
    } // end match 
    debug!("Digested {n_chunk} chunks!");
    debug!("Noticed {n_errors} errors!");
  } // end loop
  println!("= => [rbcomm] thread for RB {} finished! (not recoverable)", board_id);
} // end fun

