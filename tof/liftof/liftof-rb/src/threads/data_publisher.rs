use std::net::IpAddr;
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
use local_ip_address::local_ip;
use liftof_lib::DATAPORT;
use tof_dataclasses::events::{RBEvent,
                              DataType};
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::threading::ThreadControl;

use crate::api::{
    //prefix_board_id,
    prefix_board_id_noquery,
    prefix_local,
};
use crate::control::get_board_id;
// this is just used for the testing case
fn find_missing_elements(nums: &[u32]) -> Vec<u32> {
  let mut missing_elements = Vec::new();
  let mut expected = nums[0];

  for &num in nums {
      while expected < num {
          missing_elements.push(expected);
          expected += 1;
      }

      if expected == num {
          expected += 1;
      }
  }
  missing_elements
}

/// Manage the 0MQ PUB socket and send everything 
/// which comes in over the wire as a byte 
/// payload
///
/// # Arguments 
/// * write_to_disk : Write data to local disk (most likely
///                   a SD card). This option should be only
///                   used for diagnostic purposes.
/// * file_suffix   : basically the ending of the file. If None,
///                   this will be .gaps.tof. If cali.gaps.tof, 
///                   this will trigger to be stored in a 
///                   seperate calibration folder.
/// * print_packets : Print outgoing packets to terminal
///
pub fn data_publisher(data           : &Receiver<TofPacket>,
                      write_to_disk  : bool,
                      file_suffix    : Option<&str> ,
                      testing        : bool,
                      print_packets  : bool,
                      thread_control : Arc<Mutex<ThreadControl>>) {
  let mut address_ip = String::from("tcp://");
  let this_board_ip = local_ip().expect("Unable to obtainl local board IP. Something is messed up!");
  let data_port    = DATAPORT;
  if testing {
    warn!("Testing mode!");
  }

  match this_board_ip {
    IpAddr::V4(ip) => address_ip += &ip.to_string(),
    IpAddr::V6(_) => panic!("Currently, we do not support IPV6!")
  }
  let data_address : String = address_ip.clone() + ":" + &data_port.to_string();
  let ctx = zmq::Context::new();
  
  let data_socket = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  data_socket.bind(&data_address).expect("Unable to bind to data (PUB) socket {data_adress}");
  info!("0MQ PUB socket bound to address {data_address}");

  let board_id = address_ip.split_off(address_ip.len() -2);
  let outputfile_name = "rb_".to_owned()
                       + &board_id.to_string()
                       + file_suffix.unwrap_or(".gaps.tof");

  let blobfile_path = Path::new(&outputfile_name);
  

  let mut file_on_disk : Option<File> = None;//let mut output = File::create(path)?;
  if write_to_disk {
    // in case it is a calibration file, delete any old 
    // calibration and write it to a specific location
    let home      = env::var_os("HOME").unwrap_or(OsString::from("/home/gaps"));
    let calib_dir = home.to_string_lossy().to_string() + "/calib"; 
    if outputfile_name.ends_with("cali.tof.gaps") {
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
      let local_file = calib_file.join(outputfile_name);
      info!("Writing calibration to {}", local_file.display() );
      file_on_disk = OpenOptions::new().create(true).write(true).open(local_file).ok()
    } else {
      info!("Writing packets to {}", outputfile_name );
      file_on_disk = OpenOptions::new().append(true).create(true).open(blobfile_path).ok()
    }
  }
 
  // these are only required for testing
  let mut last_10k_evids = Vec::<u32>::new();
  if testing {
    last_10k_evids = Vec::<u32>::with_capacity(10000);
  }
  let mut n_tested : u32 = 0;
  let mut n_sent   : u64 = 0;
  let board_id     = get_board_id().unwrap_or(0) as u8;
  if board_id == 0 {
    error!("We could not get the board id!");
  }
  let mut sigint_received = false;
  let mut kill_timer      = Instant::now();
  loop {
    // check if we should end this
    if sigint_received && kill_timer.elapsed().as_secs() > 10 {
      info!("Kill timer expired. Ending thread!");
      break;
    }
    match thread_control.lock() {
      Ok(_) => {
        info!("Received stop signal. Will stop thread!");
        sigint_received = true;
        kill_timer      = Instant::now();
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
            }
          }
        }
        
        if testing {
          n_tested += 1;
          match RBEvent::from_bytestream(&packet.payload, &mut 0) {
            Ok(event) => {
              last_10k_evids.push(event.header.event_id);
            },
            Err(err) => {
               warn!("Error occured during testing! {err}");
               warn!("We are seing a payload of {} bytes", packet.payload.len());
               //warn!("Last few bytes:");
               //for k in packet.payload.len() - 20..packet.payload.len() {
               //  warn!("-- {}", packet.payload[k]);
               //}
            }
          }
          if n_tested == 10000 {
            println!("Testing batch complete! Will check the last 10000 events!");
            println!("-- first event id {}",  last_10k_evids[0]);
            println!("-- last event id {}", last_10k_evids[last_10k_evids.len() - 1]);
            // this is not efficient, but this is a testing case anyway
            let mut duplicates = false;
            for i in 0..last_10k_evids.len() {
              for j in (i + 1)..last_10k_evids.len() {
                if last_10k_evids[i] == last_10k_evids[j] {
                  println!("FAIL : Found eventid {} at positions {} and {}", last_10k_evids[i], i, j);
                  duplicates = true;
                }
              }
            }
            if !duplicates {
              println!("PASS - we did not observe any duplicate entries!");
            }
            let missing = find_missing_elements(&last_10k_evids);
            if missing.is_empty() {
              println!("PASS - we did not miss any event ids!");
            } else {
              println!("FAIL - we missed {} event ids ({}/100)", missing.len(), missing.len() as f32/10000.0);
              println!("MISSING {:?}", missing);
            }
            println!("----");
            println!("---- last 10k evids {:?}", last_10k_evids);
            last_10k_evids.clear();
          }
        } // end testing
        //
        //// prefix the board id, except for our Voltage, Timing and NOI 
        //// packages. For those, we prefix with LOCAL 
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
        if print_packets {
          println!("=> Tof packet type: {} with {} bytes!", packet.packet_type, packet.payload.len());
        }
        match data_socket.send(tp_payload,zmq::DONTWAIT) {
          Ok(_)    => {
            trace!("0MQ PUB socket.send() SUCCESS!");
            n_sent += 1;
          },
          Err(err) => error!("Not able to send over 0MQ PUB socket! Err {err}"),
        }
        if n_sent % 1000 == 0 && n_sent > 0 {
          info!("==> We sent {n_sent} packets!");
        }
      }
    }
  }
}

