use std::env;
use std::path::Path;
use std::fs;
use std::fs::OpenOptions; 
use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use std::sync::{
    Arc,
    Mutex,
};

use crossbeam_channel::Receiver;

use tof_dataclasses::packets::{TofPacket,
                               PacketType};
//use local_ip_address::local_ip;
use tof_dataclasses::events::{RBEvent,
                              DataType};
use tof_dataclasses::serialization::Serialization;
//use tof_dataclasses::threading::ThreadControl;
use liftof_lib::thread_control::ThreadControl;

use crate::api::{
    //prefix_board_id,
    prefix_board_id_noquery,
    prefix_local,
};
use crate::control::get_board_id;

/// Manage the 0MQ PUB socket and send everything 
/// which comes in over the wire as a byte 
/// payload
///
/// # Arguments 
/// * write_to_disk : Write data to local disk (most likely
///                   a SD card). This option should be only
///                   used for diagnostic purposes.
/// * address       : IP address to use for the local PUB 
///                   socket to publish data over the 
///                   network
/// * output_fname  : In case a local file should be written,
///                   write it with this name.
///                   In case of a calibration file, then 
///                   also save it in the dedicated foler.
///                   If this is None, don't write anything.
/// * print_packets : Print outgoing packets to terminal
///
pub fn data_publisher(data           : &Receiver<TofPacket>,
                      address        : String,
                      output_fname   : Option<String> ,
                      print_packets  : bool,
                      thread_control : Arc<Mutex<ThreadControl>>) {

  let ctx = zmq::Context::new();
  let data_socket = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  data_socket.bind(&address).expect("Unable to bind to data (PUB) socket {data_adress}");
  info!("0MQ PUB socket bound to address {address}");

  let mut file_on_disk : Option<File>;//let mut output = File::create(path)?;
  //if write_to_disk {
  let fname : String;
  let write_to_disk;
  match output_fname {
    None => {
      fname = String::from("Unknown.tof.gaps");
      write_to_disk = false;
    }
    Some(_fname) => {
      fname = _fname;
      write_to_disk = true;
    }
  }
  let datafile_output_file = Path::new(&fname);
  // in case it is a calibration file, delete any old 
  // calibration and write it to a specific location
  let home      = env::var_os("HOME").unwrap_or(OsString::from("/home/gaps"));
  let calib_dir = home.to_string_lossy().to_string() + "/calib"; 
  if fname.ends_with("cali.tof.gaps") {
    match fs::metadata(&calib_dir) {
      Ok(metadata) => {
        // Check if the metadata is for a directory
        if !metadata.is_dir() {
          error!("The path exists, but it is not a directory.");
        }
      }
      Err(_) => {
        // An error occurred, which typically means the directory does not exist
        warn!("No calibration directory found. Will create {}", calib_dir);
        match fs::create_dir(calib_dir.clone()) {
          Ok(_) => (),
          Err(err) => {
            error!("Can not create {}! Err {err}", calib_dir)
          }
        }
      }
    } // end match
    let calib_file = Path::new(&calib_dir);
    let local_file = calib_file.join(fname);
    info!("Writing calibration to {}", local_file.display() );
    file_on_disk = OpenOptions::new().create(true).write(true).open(local_file).ok()
  } else {
    info!("Writing to local file {}!", fname );
    file_on_disk = OpenOptions::new().append(true).create(true).open(datafile_output_file).ok()
  }
 
  let board_id     = get_board_id().unwrap_or(0) as u8;
  if board_id == 0 {
    error!("We could not get the board id!");
  }
  let mut sigint_received = false;
  let mut kill_timer      = Instant::now();
  let mut n_sent          = 0usize;
  loop {
    // check if we should end this
    if sigint_received && kill_timer.elapsed().as_secs() > 10 {
      info!("Kill timer expired. Ending thread!");
      break;
    }
    match thread_control.lock() {
      Ok(tc) => {
        if tc.stop_flag {
          info!("Received stop signal. Will stop thread!");
          sigint_received = true;
          kill_timer      = Instant::now();
        }
      },
      Err(err) => {
        trace!("Can't acquire lock! {err}");
      },
    }
    let mut data_type = DataType::Unknown;
    match data.recv() {
      Err(err) => trace!("Error receiving TofPacket {err}"),
      Ok(packet)    => {
        if matches!(packet.packet_type, PacketType::RBEvent) {
          match RBEvent::extract_datatype(&packet.payload) {
            Ok(dtype) => {
              data_type = dtype;
            }
            Err(err) => {
              error!("Unable to extract data type! Err {err}");
            }
          }
        }
        if write_to_disk && !packet.no_write_to_disk {
          match &mut file_on_disk {
            None => error!("We want to write data, however the file is invalid!"),
            Some(f) => {
              match f.write_all(packet.to_bytestream().as_slice()) {
                Err(err) => error!("Writing file to disk failed! Err {err}"),
                Ok(()) => ()
              }
              // local file can be synced, rate should be in general 
              // low when we are writing to the local file.
              // Careful with SD card!
              match f.sync_all() {
                Err(err) => error!("Unable to sync file to disk! {err}"),
                Ok(()) => ()
              }
            }
          }
        }
        
        // prefix the board id, except for our Voltage, Timing and NOI 
        // packages. For those, we prefix with LOCAL 
        let tp_payload : Vec<u8>;
        match data_type {
          // FIXME - this makes that data types for 
          // calibration will be rerouted back to 
          // the same board. We have to make that 
          // behaviour configurable. 
          // It can simply subscribe to the same 
          // message?
          DataType::VoltageCalibration |
          DataType::TimingCalibration  | 
          DataType::Noi => {
            tp_payload = prefix_local(&mut packet.to_bytestream());
          },
          _ => {
            tp_payload = prefix_board_id_noquery(board_id, &mut packet.to_bytestream());
          }
        }
        match data_socket.send(tp_payload,zmq::DONTWAIT) {
          Ok(_)    => {
            trace!("0MQ PUB socket.send() SUCCESS!");
            n_sent += 1;
          },
          Err(err) => error!("Not able to send {} over 0MQ PUB socket! {err}", packet.packet_type),
        }
        if packet.packet_type == PacketType::RBCalibration {
          //info!("==> last data type {:?}", data_type);
          info!("==> Calibration packet {} sent!", packet );
        }
        if n_sent % 1000 == 0 && n_sent > 0 && print_packets {
          println!("==> We sent {n_sent} packets!");
          println!("==> Last Tofpacket type: {} with {} bytes!", packet.packet_type, packet.payload.len());
        }
      }
    }
  }
}

