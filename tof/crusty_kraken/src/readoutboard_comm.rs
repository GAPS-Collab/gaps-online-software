//mod reduced_tofevent;

use crate::reduced_tofevent::PaddlePacket;

use crate::calibrations::{Calibrations,
                          read_calibration_file,
                          remove_spikes,
                          voltage_calibration, 
                          timing_calibration};


use std::{fs, fs::File, path::Path};
use std::io::Read;


use crate::readoutboard_blob::{BlobData,
                               BLOBEVENTSIZE};

use crate::constants::{NBOARDS,
                       NCHN,
                       NWORDS};

use crate::waveform::CalibratedWaveform;


/*************************************/

macro_rules! tvec [
    ($t:ty; $($e:expr),*) => { vec![$($e as $t),*] as Vec<$t> }
];

/*************************************/

fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    return buffer;
}

/*************************************/

fn analyze_blobs(buffer         : &Vec<u8>,
                 rb_id          : usize,
                 print_events   : bool,
                 do_calibration : bool,
                 pack_data      : bool)
{
  let mut blob_data = BlobData {..Default::default()};
  let mut header_found_start    = false;
  let mut nblobs = 0usize;
  let mut ncorrupt_blobs = 0usize;
  let mut pos = 0usize;
  let blobdata_size = buffer.len();
  let mut byte;
  
  let mut events : Vec<BlobData> = Vec::new();

  // in case we want to do calibratoins
  let mut calibrations = [Calibrations {..Default::default()};NCHN];

  if do_calibration {
    let cal_file_name = "/srv/gaps/gfp-data/gaps-gfp/TOFsoftware/server/datafiles/rb".to_owned() + &rb_id.to_string() + "_cal.txt";
    println!("Reading calibrations from file {}", cal_file_name);
    let cal_file_path = Path::new(&cal_file_name);
    calibrations = read_calibration_file(cal_file_path); 
  }


  loop {
    if pos + BLOBEVENTSIZE() >= (blobdata_size -1) {break;}
    byte = buffer[pos];

    if !header_found_start {
      if byte == 0xaa {
        header_found_start = true;
      }
      pos +=1;
      continue;
    }

    if header_found_start {
      pos += 1;
      if byte == 0xaa {
        header_found_start = false;
        blob_data.deserialize(&buffer, pos-2);
        nblobs += 1;
        
        if blob_data.tail == 0x5555 {
            if print_events
                {blob_data.print();}
            pos += BLOBEVENTSIZE() - 2; 
            if do_calibration {
              
              // the order of tasks should be something 
              // like this
              // 1) read-out
              // 2) calibration
              // 3) paak-finding
              // 4) cfd algorithm
              // 5) paddle packaging
              const NPADDLES : usize = (NCHN - 1)/2; // assuming one channel 
                                           // is the channel 9
              let mut paddle_packets_this_rb = [PaddlePacket::new(); NPADDLES];             
              // binary switch - false for side a and
              // true for side b
              let mut is_bside : bool = false;
              // the paddle mapping is HARDCODED here
              // FIXME: We make the assumption that nchanel -> paddle side
              //                                    0 -> Paddle0/A Side
              //                                    1 -> Paddle0/B Side
              //                                    2 -> Paddle1/A Side
              //                                    3 -> Paddle1/B Side
              //                                    4 -> Paddle2/A Side
              //                                    5 -> Paddle2/B Side
              //                                    6 -> Paddle3/A Side
              //                                    7 -> Paddle3/B Side

              let mut paddle_number = 0;
              for n in 0..NCHN {
                let mut trace_out : [f64;NWORDS] = [0.0;NWORDS];
                let mut times     : [f64;NWORDS] = [0.0;NWORDS];
                voltage_calibration(&blob_data.ch_adc[n],
                                    &mut trace_out,
                                    blob_data.stop_cell,
                                    &calibrations[n]);
                timing_calibration(&mut times,
                                   blob_data.stop_cell,
                                   &calibrations[n]);

                // analysis part
                let waveform = CalibratedWaveform::new(&trace_out, &times);
                let cfd_time = waveform.find_cfd_simple(0);

                // packing part
                if n == 0 || n == 1 {paddle_number = 0;}
                if n == 2 || n == 3 {paddle_number = 1;}
                if n == 4 || n == 5 {paddle_number = 2;}
                if n == 6 || n == 7 {paddle_number = 3;}
                paddle_packets_this_rb[paddle_number].set_time(cfd_time, n%2);
              }
            }
            events.push(blob_data);
        } else {
            // the event is corrupt
            //println!("{}", blob_data.head);
            ncorrupt_blobs += 1;
        }
      } else {
          // it wasn't an actual header
          header_found_start = false;
      }
    }
  }// end loop
  println!("==> Deserialized {} blobs! {} blobs were corrupt", nblobs, ncorrupt_blobs);
}

/*************************************/

fn get_blobs_from_file (rb_id : usize) {
  let filepath = String::from("/data0/gfp-data-aug/Aug/run4a/d20220809_195753_4.dat");
  let blobs = get_file_as_byte_vec(&filepath);
  analyze_blobs(&blobs, rb_id, false, false, false);  
}

/*************************************/

fn identifiy_readoutboard(msg : &zmq::Message) -> bool
{
  let size     = msg.len();
  if size == 0 {
      return false;
    }
  let result = msg.as_str();
  if !result.is_some() {
      return false;
  }
  // the signature for RB's is "RBXX"
  if size < 5 {
      // FIXME - pattern recognition, 
      // extract rb id
      let rb_ping = msg.as_str().unwrap();
      println!("Received RB ping signal {}", rb_ping);
      return true;
  } else {
      println!("Received RB {}", msg.as_str().unwrap());
  }
  return false;
}

/*************************************/

pub fn readoutboard_communicator(socket      : &zmq::Socket,
                                 board_id    : usize)
{ 
  println!("readoutboard_communicator initializing for board {}!", board_id);
  let mut msg = zmq::Message::new();
  let mut n_errors = 0usize;
  loop {
    match socket.recv(&mut msg, 0) {
      Ok(_) => {
          println!("Working...");
          // check for rb ping signal
          let rb_ping = identifiy_readoutboard(&msg);
          if rb_ping {
            let result = socket.send_str("[SVR]: R'cvd RBping", 0);
            match result {
              Ok(_) => println!("RB board ping received"),
              Err(_) => println!("Can not send ping!")
            }
            continue;
          }
          let size = msg.len();
          if size == 0 {continue;}
          let mut buffer = tvec![u8;msg.len()];
          buffer = msg.to_vec();
          println!("received message with len : {}", size);
          let result = socket.send_str("[SVR]: Received data",0);
          match result {
              Ok(_) => println!("Reply sent!"),
              Err(_) => println!("Warn - remote socket problems")
          }
          // do the work
          analyze_blobs(&buffer, board_id, false, false, false);

          //thread::sleep(Duration::from_millis(1500));
      }
      Err(_) => {
          n_errors += 1;
          //println!("Terminating rb commmunications");
          //println!("Received garbage or nothing...");
          //break;
      }
    }
  }
}

/*************************************/

