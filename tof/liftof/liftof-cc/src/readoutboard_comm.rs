//! Routines for RB commiunication and data reception 

use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs::File, path::Path};
//use std::io::Read;
use std::io::Write;
use std::fs::OpenOptions;
use crossbeam_channel::Sender;

#[cfg(feature = "diagnostics")]
//use waveform::CalibratedWaveformForDiagnostics;
#[cfg(feature = "diagnostics")]
use hdf5;
#[cfg(feature = "diagnostics")]
use ndarray::{arr1};

use liftof_lib::analyze_blobs;

use tof_dataclasses::manifest::ReadoutBoard;
use tof_dataclasses::packets::PacketType;
use tof_dataclasses::packets::paddle_packet::PaddlePacket;
use tof_dataclasses::calibrations::{Calibrations,
                                    read_calibration_file};
use tof_dataclasses::constants::NCHN;

use tof_dataclasses::commands::TofResponse;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::serialization::Serialization;


/*************************************/

macro_rules! tvec [
    ($t:ty; $($e:expr),*) => { vec![$($e as $t),*] as Vec<$t> }
];

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
pub fn readoutboard_communicator(pp_pusher        : Sender<PaddlePacket>,
                                 resp_to_main     : Sender<TofResponse>,
                                 tp_to_sink       : Sender<TofPacket>,
                                 write_rb_raw     : bool,
                                 storage_savepath : &String,
                                 events_per_file  : &usize,
                                 rb               : &ReadoutBoard,
                                 runid            : usize,
                                 print_packets    : bool) {
  let zmq_ctx = zmq::Context::new();
  let board_id = rb.rb_id; //rb.id.unwrap();
  info!("initializing RB thread for board {}!", board_id);
  let mut n_errors        = 0usize;
  // how many chunks ("buffers") we dealt with
  let mut n_chunk  = 0usize;
  // in case we want to do calibratoins
  let mut calibrations = [Calibrations {..Default::default()};NCHN];
  let do_calibration = true;
  if do_calibration {
    info!("Reading calibrations from file {}", &rb.calib_file);
    let cal_file_path = Path::new(&rb.calib_file);//calibration_file);
    calibrations = read_calibration_file(cal_file_path); 
  }
  let address = "tcp://".to_owned() 
              + &rb.ip_address.to_string()
              + ":"
              +  &rb.port.to_string();
  let socket = zmq_ctx.socket(zmq::SUB).expect("Unable to create socket!");
  match socket.connect(&address) {
    Err(err) => error!("Can not connect to socket {}, {}", address, err),
    Ok(_)    => info!("Connected to {address}")
  }
  // FIXME - do not subscribe to all, only this 
  // specific RB
  let topic = b"";
  //let mut topic : String;
  //if rb.id.unwrap() < 10 {
  //  topic = String::from("RB0") + &rb.id.unwrap().to_string();
  //} else {
  //  topic = String::from("RB") + &rb.id.unwrap().to_string();
  //}
  match socket.set_subscribe(topic) {
   Err(err) => error!("Unable to subscribe to topic! {err}"),
   Ok(_) => ()

  }
  //socket.set_subscribe(topic.as_bytes());
  let mut secs_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
  let mut blobfile_name = storage_savepath.to_owned() + "/RB" 
                       + &board_id.to_string() + "_" 
                       + &secs_since_epoch.to_string()
                       + ".blob";
  //let mut topic : String;
  //if rb.id.unwrap() < 10 {
  //  topic = String::from("RB0") + &rb.id.unwrap().to_string();
  //} else {
  //  topic = String::from("RB") + &rb.id.unwrap().to_string();
  //}
  //socket.set_subscribe(topic.as_bytes());
  //socket.set_subscribe(topic);
  //let blobfile_name = storage_savepath.to_owned() + "blob_" 
  //let mut topic : String;
  //if rb.id.unwrap() < 10 {
  //  topic = String::from("RB0") + &rb.id.unwrap().to_string();
  //} else {
  //  topic = String::from("RB") + &rb.id.unwrap().to_string();
  //}
  //socket.set_subscribe(topic.as_bytes());
  //                     + &board_id.to_string()
  let mut blobfile_path = Path::new(&blobfile_name);
  let mut file_on_disc : Option<File> = None;//let mut output = File::create(path)?;
  if write_rb_raw {
    info!("Writing readoutboard raw data to {}", blobfile_name );
    file_on_disc = OpenOptions::new().append(true).create(true).open(blobfile_path).ok()
  }
  let mut n_events = 0usize;
  loop {

    // check if we got new data
    // this is blocking the thread
    match socket.recv_bytes(0) {
      Err(err) => {
        n_errors += 1;
        error!("Receiving from socket raised error {}", err);
      }
      Ok(buffer) => {
        //trace!("Working...");
        //// check for rb ping signal
        //let rb_ping = identifiy_readoutboard(&msg);
        //if rb_ping {
        //  //let result = socket.send_str("[SVR]: R'cvd RBping", 0);
        //  let result = socket.send("[SVR]: R'cvd RBping", 0);
        //  match result {
        //    Ok(_)    => debug!("RB {} handshake complete!", board_id),
        //    Err(err) => error!("Not able to send back reply when negotiating RB comms, handshake possibly failed..")
        //  }
        //  continue;
        //}
        //let size = msg.len();
        //if size == 0 {continue;}
        //let mut buffer = tvec![u8;msg.len()];
        //buffer = msg.to_vec();
        //debug!("received message with len : {}", size);
        ////let result = socket.send_str("[SVR]: Received data",0);
        //let result = socket.send("[SVR]: Received data",0);
        //match result {
        //    Ok(_)    => debug!("Received data of len {} and acknowledged!", size),
        //    Err(err) => error!("Not able to send back reply to acknowleded received data!")
        //}
        // do the work
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
            match tp.packet_type {
              PacketType::RBHeader => {
                // in that case, just write the header to 
                // the file and continue
                match &mut file_on_disc {
                  None => (),
                  Some(f) => {
                    trace!("writing {} bytes", tp.payload.len());
                    match f.write_all(&tp.payload) {
                      Err(err) => error!("Can not write to file, err {err}"),
                      Ok(_)    => ()
                    }
                  }
                }
              },
              PacketType::RBEvent => {
                error!("RBEvent is an advanced feature which is not ready yet!");
              },
              PacketType::RBEventPayload => {
                match analyze_blobs(&tp.payload,
                                    &pp_pusher,
                                    true,
                                    &rb,
                                    false,
                                    true,
                                    &calibrations,
                                    n_chunk) {
                  Ok(nblobs)   => debug!("Read {} blobs from buffer", nblobs),
                  Err(err)     => error!("Was not able to read blobs! {}", err )
                }
                // write blob to disk if desired
                match &mut file_on_disc {
                  None => (),
                  Some(f) => {
                    // if the readoutboard prefixes it's payload with 
                    // "RBXX", we have to get rid of that first
                    //match search_for_u16(0xaa, &buffer, 0) {
                    //  Err(err) => {
                    //    error!("Can not find header in this payload! {err}");
                    //  }
                    //  Ok(_)    => {
                        trace!("writing {} bytes", &tp.payload.len());
                        match f.write_all(&tp.payload) {
                          Err(err) => error!("Can not write to file, err {err}"),
                          Ok(_)    => ()
                        }
                  }
                } // end match file_on_disk
                n_events += 1;
                if (n_events >= *events_per_file) && write_rb_raw {
                  // start a new file
                  secs_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                  blobfile_name = storage_savepath.to_owned() + "/RB" 
                                 + &board_id.to_string() + "_" 
                                 + &runid.to_string() + "_"
                                 + &secs_since_epoch.to_string()
                                 + ".blob";
                  info!("Writing blobs to {}", blobfile_name );
                  blobfile_path = Path::new(&blobfile_name);
                  file_on_disc = OpenOptions::new().append(true).create(true).open(blobfile_path).ok();
                  n_events = 0;
                } //end if
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
        //println!("{:?}", tp.payload);
        //for n in 0..5 {
        //  println!("{}", tp.payload[n]);
        //}
        //println!("...");
        //for n in 0..5 {
        //  println!("{}", tp.payload[tp.payload.len() - 1 - n]);
        //}
      } // end ok buffer 
    } // end match 
  } // end loop
} // end fun

