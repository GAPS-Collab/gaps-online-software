mod readoutboard_blob;
mod calibrations;
mod readoutboard_comm;
mod threading;
mod reduced_tofevent;
mod constants;
mod waveform;

use crate::calibrations::{Calibrations, read_calibration_file};

//use crate::readoutboard_blob::{BlobData, BLOBEVENTSIZE};
use crate::constants::{NBOARDS, NCHN};

use crate::readoutboard_comm::readoutboard_communicator;

use crate::threading::ThreadPool;

use std::{thread,
          time,
          path::Path};



/*************************************/

fn main() {

    // read calibration data
    let mut calibrations = [[Calibrations {..Default::default()}; NCHN]; NBOARDS];
    let mut rb_id = 0usize;
    for n in 0..NBOARDS {
        rb_id = n + 1;
        let file_name = "/srv/gaps/gfp-data/gaps-gfp/TOFsoftware/server/datafiles/rb".to_owned() + &rb_id.to_string() + "_cal.txt";
        println!("Reading calibrations from file {}", file_name);
        let file_path = Path::new(&file_name);
        calibrations[n] = read_calibration_file(file_path); 
    }

    // each readoutboard gets its own worker
    let rbcomm_workers = ThreadPool::new(NBOARDS);

    // open a zmq context
    let ctx = zmq::Context::new();
    // FIXME - port and address need to be 
    // configurable
    let mut port = 38830usize;
    let address_ip = "tcp://127.0.0.1";
    
    let mut address : String;
    for n in 0..NBOARDS {
      let rb_comm_socket = ctx.socket(zmq::REP).unwrap();
      address = address_ip.to_owned() + ":" + &port.to_string();
      println!("Will bind to port for rb comm at {}", address);
      let result = rb_comm_socket.bind(&address);
      match result {
          Ok(_)    => println!("Bound socket to {}", address),
          Err(err) => panic!("Can not communicate with rb at address {}, error {}",address, err)
      }
      rbcomm_workers.execute(move || {
          readoutboard_communicator(&rb_comm_socket, n); 
      });
      port += 1;
    }

//    let mut blob_data = BlobData {..Default::default()};
//    let filepath = String::from("/data0/gfp-data-aug/Aug/run4a/d20220809_195753_4.dat");
//    let blobs = get_file_as_byte_vec(&filepath);
//
//    let mut header_found_start    = false;
//
//    let mut nblobs = 0usize;
//    let mut ncorrupt_blobs = 0usize;
//    let mut pos = 0usize;
//    let blobdata_size = blobs.len();
//    let mut byte;
//    
//    loop {
//      if pos + BLOBEVENTSIZE() >= (blobdata_size -1) {break;}
//      byte = blobs[pos];
//
//      if !header_found_start {
//        if byte == 0xaa {
//          header_found_start = true;
//        }
//        pos +=1;
//        continue;
//      }
//
//      if header_found_start {
//        pos += 1;
//        if byte == 0xaa {
//          header_found_start = false;
//          blob_data.deserialize(&blobs, pos-2);
//          nblobs += 1;
//          blob_data.print();
//          if blob_data.tail == 0x5555 {
//              pos += BLOBEVENTSIZE() - 2; 
//          } else {
//              // the event is corrupt
//              println!("{}", blob_data.head);
//              ncorrupt_blobs += 1;
//          }
//        } else {
//            // it wasn't an actual header
//            header_found_start = false;
//        }
//      }
//    }// end loop
//    println!("==> Deserialized {} blobs! {} blobs were corrupt", nblobs, ncorrupt_blobs);
let one_minute = time::Duration::from_millis(60000);
//let now = time::Instant::now();

thread::sleep(2*one_minute);



}
