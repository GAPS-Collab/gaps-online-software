//! Routines for RB commiunication and data reception 

use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;
use crossbeam_channel::Sender;

use liftof_lib::waveform_analysis;

use tof_dataclasses::manifest::ReadoutBoard;
use tof_dataclasses::events::RBEvent;
use tof_dataclasses::packets::{TofPacket,
                               PacketType};
use tof_dataclasses::calibrations::RBCalibrations;

use tof_dataclasses::serialization::Serialization;

use liftof_lib::build_tcp_from_ip;

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
/// * pp_pusher        :
/// * reso_to_main     :
/// * tp_to_sink       : Channel which should be connect to a (global) data sink.
///                      Packets which are of not event type (e.g. header/full binary data)
///                      will be forwarded to the sink.
/// * write_rb_raw     : Should readoutboard raw data be written to disk?
/// * storage_savepath :
/// * events_per_file  :
/// * rb               : 
/// * runid            : Current assigned runid. Will be used in the filenames of saved 
///                      readoutboard raw data.
/// * print_packets    : 
pub fn readoutboard_communicator(ev_to_builder       : &Sender<RBEvent>,
                                 tp_to_sink          : Sender<TofPacket>,
                                 rb                  : &ReadoutBoard,
                                 runid               : usize,
                                 print_packets       : bool,
                                 run_analysis_engine : bool) {
  let zmq_ctx = zmq::Context::new();
  let board_id = rb.rb_id; //rb.id.unwrap();
  info!("initializing RB thread for board {}!", board_id);
  let mut n_errors        = 0usize;
  // how many chunks ("buffers") we dealt with
  let mut n_chunk  = 0usize;
  // in case we want to do calibratoins
  let mut calibrations = RBCalibrations::new(rb.rb_id);
  let do_calibration = true;
  if do_calibration {
    info!("Reading calibrations from file {}", &rb.calib_file);
    let cal_file_path = Path::new(&rb.calib_file);//calibration_file);
    calibrations = RBCalibrations::from(cal_file_path);
  }
  let address = build_tcp_from_ip(rb.ip_address.to_string(),
                                       rb.port.to_string());

  // FIXME - this panics, however, if we can't set up the socket, what's 
  // the point of this thread?
  let socket = zmq_ctx.socket(zmq::SUB).expect("Unable to create socket!");
  match socket.connect(&address) {
    Err(err) => error!("Can not connect to socket {}, {}", address, err),
    Ok(_)    => info!("Connected to {address}")
  }
  // no need to subscribe to a topic, since there 
  // is one port for each rb
  let topic = b"";
  match socket.set_subscribe(topic) {
   Err(err) => error!("Unable to subscribe to topic! {err}"),
   Ok(_) => ()

  }
  let mut secs_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
  let mut n_events   = 0usize;
  let mut n_received = 0usize;
  loop {

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
            error!("Unknown packet...{:?}", err);
            continue;  
          },
          Ok(tp) => {
            if print_packets {
              println!("==> Got {} for RB {}", tp.packet_type, rb.rb_id); 
            }
            n_received += 1;
            match tp.packet_type {
              PacketType::RBEvent => {
                let mut event = RBEvent::from(&tp);
                if event.hits.len() == 0 {
                  if run_analysis_engine {
                    match waveform_analysis(&mut event, 
                                            &rb,
                                            &calibrations) {
                        
                      Ok(_) => (),
                      Err(err) => {
                        error!("Unable to analyze waveforms for this event! Err {err}");
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
                n_events += 1;
                n_chunk += 1;
              }, 
              _ => {
                // Currently, we will just forward all other packets
                // directly to the data sink
                match tp_to_sink.send(tp) {
                  Err(err) => error!("Can not send tof packet to data sink! Err {err}"),
                  Ok(_)    => info!("Packet sent"),
                }
              }
            } // end match packet type
          } // end OK
        } // end match from_bytestream
      } // end ok buffer 
    } // end match 
    debug!("Digested {n_chunk} chunks!");
    debug!("Noticed {n_errors} errors!");
    if n_received % 100 == 0 {
      println!("==> Received {n_received} packets!");
    }
  } // end loop
} // end fun

