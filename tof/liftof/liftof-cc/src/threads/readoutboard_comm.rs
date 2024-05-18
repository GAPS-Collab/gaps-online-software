//! Routines for RB commiunication and data reception 

//use std::time::{SystemTime, UNIX_EPOCH};
//use std::path::{
//    Path,
//    PathBuf,
//};

use std::sync::{
    Arc,
    Mutex,
};
use std::time::{
    Instant,
    Duration,
};

use tof_dataclasses::threading::{
    ThreadControl,
};


use crossbeam_channel::Sender;

use tof_dataclasses::database::ReadoutBoard;
use tof_dataclasses::events::RBEvent;
use tof_dataclasses::packets::{
    TofPacket,
    PacketType
};
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::commands::TofResponse;

use liftof_lib::{
    waveform_analysis,
};

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
/// * print_packets       : Increase verbosity and print incoming packets from the RB
/// * run_analysis_engine : Extract TofHits from the waveforms and attach them to RBEvent
/// * ae_settings         : Settings to configure peakfinding algorithms etc. 
///                         These can be configured with an external .toml file
/// * ack_sender          : Intercept acknowledgement packets and forward them to elswhere
pub fn readoutboard_communicator(ev_to_builder       : Sender<RBEvent>,
                                 tp_to_sink          : Sender<TofPacket>,
                                 rb                  : ReadoutBoard,
                                 print_packets       : bool,
                                 run_analysis_engine : bool,
                                 ae_settings         : AnalysisEngineSettings,
                                 ack_sender          : Sender<TofResponse>,
                                 thread_control      : Arc<Mutex<ThreadControl>>) {
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
  //let topic = b"";
  match socket.set_subscribe(&topic.as_bytes()) {
   Err(err) => error!("Unable to subscribe to topic! {err}"),
   Ok(_)    => info!("Subscribed to {:?}!", topic),
  }
  let mut tc_timer = Instant::now();
  loop {
    if tc_timer.elapsed().as_secs() > 1 {
      match thread_control.lock() {
        Ok(tc) => {
          if tc.stop_flag {
            break;
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
            if print_packets {
              println!("==> Got {} for RB {}", tp.packet_type, rb.rb_id); 
            }
            //n_received += 1;
            match tp.packet_type {
              PacketType::TofResponse => {
                match tp.unpack::<TofResponse>() {
                  Err(err)   => error!("Unable to send ACK packet! {err}"),
                  Ok(tr)     => {
                    match ack_sender.send(tr) {
                      Err(err) => error!("Unable to send ACK packet! {err}"),
                      Ok(_)    => ()
                    }
                  }
                }
              }
              PacketType::RBEvent | PacketType::RBEventMemoryView => {
                let mut event = RBEvent::from(&tp);
                if event.hits.len() == 0 {
                  if run_analysis_engine {
                    match waveform_analysis(&mut event, 
                                            &rb,
                                            ae_settings) {
                      Ok(_) => (),
                      Err(err) => {
                        error!("Unable to analyze waveforms for this event! {err}");
                      }
                    }
                  }
                };
                match ev_to_builder.send(event) {
                  Ok(_) => (),
                  Err(err) => {
                    error!("Unable to send event! Err {err}");
                  }
                }
                //n_events += 1;
                n_chunk += 1;
              }, 
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
    //if n_received % 100000 == 0 {
    //  println!("[RBCOM] => Received {n_received} packets!");
    //}
  } // end loop
} // end fun

