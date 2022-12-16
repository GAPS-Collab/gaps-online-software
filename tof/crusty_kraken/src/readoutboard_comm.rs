use std::{fs, fs::File, path::Path};
use std::io::Read;
#[cfg(feature = "diagnostics")]
//use waveform::CalibratedWaveformForDiagnostics;
#[cfg(feature = "diagnostics")]
use hdf5;
#[cfg(feature = "diagnostics")]
use ndarray::{arr1};


use crate::errors::BlobError;
use crate::reduced_tofevent::PaddlePacket;
use crate::calibrations::{Calibrations,
                          read_calibration_file};
                          //remove_spikes,
                          //voltage_calibration, 
                          //timing_calibration};
use crate::readoutboard_blob::{BlobData,
                               get_constant_blobeventsize};
use crate::constants::{NCHN,
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

fn analyze_blobs(buffer               : &Vec<u8>,
                 rb_id                : usize,
                 print_events         : bool,
                 do_calibration       : bool,
                 pack_data            : bool)
-> Result<usize, BlobError> {
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
    info!("Reading calibrations from file {}", cal_file_name);
    let cal_file_path = Path::new(&cal_file_name);
    calibrations = read_calibration_file(cal_file_path); 
  }

  // allocate some memory we are using in 
  // every iteration of the loop
  const NPADDLES : usize = (NCHN - 1)/2; // assuming one channel 
                                           // is the channel 9
  let mut paddle_packets_this_rb = [PaddlePacket::new(); NPADDLES];             
  // binary switch - false for side a and
  // true for side b
  let mut is_bside : bool = false;
  let mut trace_out : [f64;NWORDS] = [0.0;NWORDS];
  let mut times     : [f64;NWORDS] = [0.0;NWORDS];

  // remove_spikes requires two dimensional array
  let mut all_channel_waveforms : [[f64;NWORDS];NCHN] = [[0.0;NWORDS];NCHN];
  let mut all_channel_times     : [[f64;NWORDS];NCHN] = [[0.0;NWORDS];NCHN];

  #[cfg(feature = "diagnostics")]
  let mut diagnostics_wf : Vec<CalibratedWaveform> = Vec::new();

  loop {
    // if the following is true, we scanned throught the whole stream  
    if pos + get_constant_blobeventsize() >= (blobdata_size -1) {break;}
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
        // we have found our 0xaaaa marker!
        // include it in the stream to deserialize
        blob_data.from_bytestream(&buffer, pos-2);
        nblobs += 1;
        
        if blob_data.tail == 0x5555 {
            if print_events {blob_data.print();}
            pos += get_constant_blobeventsize() - 2; 
            if do_calibration {
              
              // the order of tasks should be something 
              // like this
              // 1) read-out
              // 2) calibration
              // 3) paak-finding
              // 4) cfd algorithm
              // 5) paddle packaging
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
              let mut spikes : [i32;10] = [0;10];
              blob_data.calibrate(&calibrations);
              blob_data.remove_spikes(&mut spikes);
              for n in 0..NCHN {

                // analysis part
                //let mut waveform = CalibratedWaveform::new(all_channel_waveforms[n],
                //                                           all_channel_times[n]);
                blob_data.set_threshold(10.0, n);
                blob_data.set_cfds_fraction(0.20, n);
                blob_data.set_ped_begin(10.0, n);// 10-100                               
                blob_data.set_ped_range(50.0, n);
                blob_data.calc_ped_range(n);
                blob_data.subtract_pedestal(n);
                blob_data.integrate(270.0, 70.0, n);
                blob_data.find_peaks(270.0,70.0, n);
                let cfd_time = blob_data.find_cfd_simple(0, n);
                //waveform.print();
                // packing part
                if n == 0 || n == 1 {paddle_number = 0;}
                if n == 2 || n == 3 {paddle_number = 1;}
                if n == 4 || n == 5 {paddle_number = 2;}
                if n == 6 || n == 7 {paddle_number = 3;}
                paddle_packets_this_rb[paddle_number].set_time(cfd_time, n%2);
                
                #[cfg(feature = "diagnostics")]
                {  
                  //events.push(blob_data);
                  let diag_wf = CalibratedWaveform::new(&blob_data, n);
                  diagnostics_wf.push (diag_wf);
                }
              } // end loop over nchannel
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

  // in case of diagnostics, we 
  // write an hdf file with calibrated 
  // waveforms for later analysis.
  #[cfg(feature = "diagnostics")]
  {
    let hdf_diagnostics_file =  "waveforms_".to_owned() + &rb_id.to_string() + ".hdf";
    let hdf_file    = hdf5::File::create(hdf_diagnostics_file).unwrap(); // open for writing
    hdf_file.create_group("waveforms");
    let hdf_group = hdf_file.group("waveforms").unwrap();
    let hdf_dataset = hdf_group.new_dataset::<CalibratedWaveform>().shape(diagnostics_wf.len()).create("wf").unwrap();
    //let hdf_dataset = hdf_group.new_dataset::<BlobData>().shape(events.len()).create("wf").unwrap();
    //hdf_dataset.write(&arr1(&diagnostics_wf))?;
    hdf_dataset.write(&arr1(&diagnostics_wf))?;
    hdf_file.close()?;
  }
  info!("==> Deserialized {} blobs! {} blobs were corrupt", nblobs, ncorrupt_blobs);
  panic!("You shall not pass!");
  Ok(nblobs)
}

/*************************************/

fn get_blobs_from_file (rb_id : usize) {
  let filepath = String::from("/data0/gfp-data-aug/Aug/run4a/d20220809_195753_4.dat");
  let blobs = get_file_as_byte_vec(&filepath);
  match analyze_blobs(&blobs,
                      rb_id,
                      false,
                      false,
                      false) {
      Ok(nblobs)   => info!("Read {} blobs from file", nblobs), 
      Err(err)     => panic!("Was not able to read blobs! Err {}", err)
  }
}

/*************************************/

///
/// Check an incoming message for readout board 
/// handshake/ping signal
///
///
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
    debug!("Received RB ping signal {}", rb_ping);
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
  trace!("initializing for board {}!", board_id);
  let mut msg = zmq::Message::new();
  let mut n_errors = 0usize;
  loop {
    match socket.recv(&mut msg, 0) {
      Ok(_) => {
          trace!("Working...");
          // check for rb ping signal
          let rb_ping = identifiy_readoutboard(&msg);
          if rb_ping {
            //let result = socket.send_str("[SVR]: R'cvd RBping", 0);
            let result = socket.send("[SVR]: R'cvd RBping", 0);
            match result {
              Ok(_)    => debug!("RB {} handshake complete!", board_id),
              Err(err) => warn!("Not able to send back reply when negotiating RB comms, handshake possibly failed..")
            }
            continue;
          }
          let size = msg.len();
          if size == 0 {continue;}
          let mut buffer = tvec![u8;msg.len()];
          buffer = msg.to_vec();
          debug!("received message with len : {}", size);
          //let result = socket.send_str("[SVR]: Received data",0);
          let result = socket.send("[SVR]: Received data",0);
          match result {
              Ok(_)    => debug!("Received data of len {} and acknowledged!", size),
              Err(err) => warn!("Not able to send back reply to acknowleded received data!")
          }
          // do the work
          match analyze_blobs(&buffer,
                              board_id,
                              false,
                              true,
                              false) {
            Ok(nblobs)   => debug!("Read {} blobs from file", nblobs),
            Err(err)     => warn!("Was not able to read blobs! {}", err )
          }

          //thread::sleep(Duration::from_millis(1500));
      }
      Err(err) => {
          n_errors += 1;
          warn!("Receiving from socket raised error {}", err);
          //println!("Terminating rb commmunications");
          //println!("Received garbage or nothing...");
          //break;
      }
    }
  }
}

/*************************************/

