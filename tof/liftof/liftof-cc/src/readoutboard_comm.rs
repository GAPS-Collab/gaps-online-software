///
///
///
///
///



use std::{fs, fs::File, path::Path};
use std::io::Read;
use std::sync::mpsc::{Sender, channel};

#[cfg(feature = "diagnostics")]
//use waveform::CalibratedWaveformForDiagnostics;
#[cfg(feature = "diagnostics")]
use hdf5;
#[cfg(feature = "diagnostics")]
use ndarray::{arr1};

use liftof_lib::ReadoutBoard;

use tof_dataclasses::packets::paddle_packet::PaddlePacket;
use crate::errors::BlobError;
//use crate::reduced_tofevent::PaddlePacket;
use tof_dataclasses::calibrations::{Calibrations,
                                    read_calibration_file};
                                    //remove_spikes,
                                    //voltage_calibration, 
                                    //timing_calibration};
use tof_dataclasses::events::blob::{BlobData,
                                    get_constant_blobeventsize};
use tof_dataclasses::constants::{NCHN,
                       NWORDS};
use crate::waveform::CalibratedWaveform;

extern crate json;

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


/// write a bytestream to a file on disk
fn write_stream_to_file(filename: &Path, bytestream: &Vec<u8>) -> Result<usize, std::io::Error>{
    fs::write(filename, bytestream)?;
    debug!("{} bytes written to {}", bytestream.len(), filename.display());
    Ok(bytestream.len())
}

/*************************************/

///
///
///
///
/// FIXME - we have to think again, which queues are really 
/// needed. I think:
/// BlobData queue : only needed when diagnostics feature is
/// set to write the waveforms to hdf
/// PaddlePacket queue : I don't think is needed for anything
/// since we are sending the packets right away
fn analyze_blobs(buffer               : &Vec<u8>,
                 pp_sender            : &Sender<PaddlePacket>,
                 send_packets         : bool,
                 rb_id                : usize,
                 print_events         : bool,
                 do_calibration       : bool,
                 calibrations         : &[Calibrations; NCHN],
                 n_chunk              : usize)
-> Result<usize, BlobError> {
  let mut blob_data              = BlobData {..Default::default()};
  let mut header_found_start     = false;
  let mut nblobs                 = 0usize;
  let mut ncorrupt_blobs         = 0usize;
  let mut pos                    = 0usize;
  let blobdata_size              = buffer.len();
  let mut byte                   : u8;

  // allocate some memory we are using in 
  // every iteration of the loop
  const NPADDLES : usize = (NCHN - 1)/2; // assuming one channel 
                                           // is the channel 9

  // each event has NPADDLES per readout board
  // this holds all for a single event
  // (needs to be reset each event)
  let mut pp_this_event = [PaddlePacket::new(); NPADDLES];        


  // binary switch - false for side a and
  // true for side b
  let mut is_bside : bool = false;
  let mut trace_out : [f64;NWORDS] = [0.0;NWORDS];
  let mut times     : [f64;NWORDS] = [0.0;NWORDS];

  // remove_spikes requires two dimensional array
  let mut all_channel_waveforms : [[f64;NWORDS];NCHN] = [[0.0;NWORDS];NCHN];
  let mut all_channel_times     : [[f64;NWORDS];NCHN] = [[0.0;NWORDS];NCHN];

  // bitmask to keep track which channels/paddles are
  // over threshold
  let mut channels_over_threshold = [false;NCHN];
  let mut paddles_over_threshold  = [false;NPADDLES];

  loop {
    #[cfg(feature = "diagnostics")]
    let mut diagnostics_wf : Vec<CalibratedWaveform> = Vec::new();
    
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
        blob_data.reset();
        blob_data.from_bytestream(&buffer, pos-2, false);
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

              let mut spikes : [i32;10] = [0;10];
              blob_data.calibrate(calibrations);
              blob_data.remove_spikes(&mut spikes);
              for ch in 0..NCHN {

                // reset our channels_over_threshold
                channels_over_threshold[ch] = false;

                // reset paddle packets for this event
                for n in 0..NPADDLES {
                  pp_this_event[n].reset();
                  paddles_over_threshold[n] = false;
                }

                // analysis part
                //let mut waveform = CalibratedWaveform::new(all_channel_waveforms[n],
                //                                           all_channel_times[n]);
                // first, subtract the pedestal
                blob_data.set_ped_begin(10.0, ch);// 10-100                               
                blob_data.set_ped_range(50.0, ch);
                blob_data.calc_ped_range(ch);
                blob_data.subtract_pedestal(ch);
                
                // then we set the threshold and check
                // if the wf went over threashold
                let is_ot = blob_data.set_threshold(10.0, ch);
                if !is_ot {continue;}
                channels_over_threshold[ch] = true;
                
                blob_data.set_cfds_fraction(0.20, ch);
                blob_data.integrate(270.0, 70.0, ch);
                blob_data.find_peaks(270.0,70.0, ch);
                // analysis
                let cfd_time = blob_data.find_cfd_simple(0, ch);
                //waveform.print();
                // packing part
                
                // FIXME - this is not independent
                // of the number of channels for the 
                // readout board
                match ch {
                  0 => {
                    paddles_over_threshold[0] = true;
                    pp_this_event[0].set_time_a(cfd_time);
                  },
                  1 => {
                    paddles_over_threshold[0] = true;
                    pp_this_event[0].set_time_b(cfd_time);
                  },
                  2 => {
                    paddles_over_threshold[1] = true;
                    pp_this_event[1].set_time_a(cfd_time);
                  },
                  3 => {
                    paddles_over_threshold[1] = true;
                    pp_this_event[1].set_time_b(cfd_time);
                  },
                  4 => {
                    paddles_over_threshold[2] = true;
                    pp_this_event[2].set_time_a(cfd_time);
                  },
                  5 => {
                    paddles_over_threshold[2] = true;
                    pp_this_event[2].set_time_b(cfd_time);
                  },
                  6 => {
                    paddles_over_threshold[3] = true;
                    pp_this_event[3].set_time_a(cfd_time);
                  },
                  7 => {
                    paddles_over_threshold[3] = true;
                    pp_this_event[3].set_time_b(cfd_time);
                  },
                  _ => {
                    trace!("Won't do anything for ch {}",ch);
                  }
                } // end match

                // now set general properties on the 
                // paddles
                for n in 0..NPADDLES {
                  // FIXME
                  pp_this_event[n].paddle_id = n as u8;
                  pp_this_event[n].event_id = blob_data.event_id;
                  if paddles_over_threshold[n] {
                    pp_this_event[n].event_id = blob_data.event_id;
                  }
                }
                
                #[cfg(feature = "diagnostics")]
                {  
                  let diag_wf = CalibratedWaveform::new(&blob_data, ch);
                  diagnostics_wf.push (diag_wf);
                }
              } // end loop over readout board channels
            }

            // put the finished paddle packets in 
            // our container
            for n in 0..NPADDLES {
              //if paddles_over_threshold[n] {
              if true {
                trace!("Sending pp to cache for evid {}", pp_this_event[n].event_id);
                pp_sender.send(pp_this_event[n]);
              }
            }
        } else {
          // the event is corrupt
          //println!("{}", blob_data.head);
          ncorrupt_blobs += 1;
        }
      } else {
          // it wasn't an actual header
          header_found_start = false;
      }
    } // endif header_found_start
  }// end loop

  // in case of diagnostics, we 
  // write an hdf file with calibrated 
  // waveforms for later analysis.
  #[cfg(feature = "diagnostics")]
  {
    let hdf_diagnostics_file =  "waveforms_".to_owned()
                                + &n_chunk.to_string()
                                + "_"
                                + &rb_id.to_string()
                                + ".hdf";
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
  Ok(nblobs)
}

/*************************************/

fn get_blobs_from_file (rb_id : usize) {
  let filepath = String::from("/data0/gfp-data-aug/Aug/run4a/d20220809_195753_4.dat");
  let blobs = get_file_as_byte_vec(&filepath);
  // FIXME - this must be thre real calibrations
  let calibrations = [Calibrations {..Default::default()};NCHN];
  //let sender = Sender::<PaddlePacket>();
  let (sender, receiver) = channel();
  match analyze_blobs(&blobs,
                      &sender,
                      false,
                      rb_id,
                      false,
                      false,
                      &calibrations,
                      0) {
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


///
/// Receive binary blobs from readout boards,
/// and perform specified tasks
///
///
pub fn readoutboard_communicator(//socket           : &zmq::Socket,
                                 //zmq_ctx          : &zmq::Context,
                                 pp_pusher        : Sender<PaddlePacket>,
                                 //board_id         : usize,
                                 write_blob       : bool,
                                 rb               : &ReadoutBoard,
                                 calibration_file : &str)
{
  let zmq_ctx = zmq::Context::new();
  let board_id = rb.id.unwrap();
  info!("initializing RB thread for board {}!", board_id);
  let mut msg             = zmq::Message::new();
  let mut n_errors        = 0usize;
  let mut lost_blob_files = 0usize;
  // how many chunks ("buffers") we dealt with
  let mut n_chunk  = 0usize;
  // in case we want to do calibratoins
  let mut calibrations = [Calibrations {..Default::default()};NCHN];
  let do_calibration = true;
  if do_calibration {
    info!("Reading calibrations from file {}", calibration_file);
    let cal_file_path = Path::new(&calibration_file);
    calibrations = read_calibration_file(cal_file_path); 
  }
  let address = "tcp::/".to_owned() 
              + &rb.ip_address.expect("No IP known for this board!").to_string()
              + ":"
              +  &rb.data_port.expect("No CMD port known for this board!").to_string();
  let socket = zmq_ctx.socket(zmq::SUB).expect("Unable to create socket!");
  socket.connect(&address);
  // FIXME - do not subscribe to all, only this 
  // specific RB
  let topic = b"";
  socket.set_subscribe(topic);
  loop {

    // check if we got new data
    // this is blocking the thread
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
              Err(err) => error!("Not able to send back reply when negotiating RB comms, handshake possibly failed..")
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
              Err(err) => error!("Not able to send back reply to acknowleded received data!")
          }
          // do the work
          match analyze_blobs(&buffer,
                              &pp_pusher,
                              true,
                              board_id as usize,
                              false,
                              true,
                              &calibrations,
                              n_chunk) {
            Ok(nblobs)   => debug!("Read {} blobs from buffer", nblobs),
            Err(err)     => error!("Was not able to read blobs! {}", err )
          }
          // write blob to disk if desired
          if write_blob {
            let blobfile_name = "blob_".to_owned() 
                                 + &n_chunk.to_string() 
                                 + "_"
                                 + &board_id.to_string()
                                 + ".blob";
            info!("Writing blobs to {}", blobfile_name );
            let blobfile_path = Path::new(&blobfile_name);
            match write_stream_to_file(blobfile_path, &buffer) {
              Ok(size)  => debug!("Writing blob file successful! {} bytes written", size),
              Err(err)  => {
                error!("Unable to write blob to disk! {}", err );
                lost_blob_files += 1;
              }
            } // end match
          } // end if write_blob
          //thread::sleep(Duration::from_millis(1500));
          n_chunk += 1;

          // currently, for debugging just stop after one 
          // chunk
          //panic!("You shall not pass!");
          
      }
      Err(err) => {
          n_errors += 1;
          error!("Receiving from socket raised error {}", err);
      }
    }
  }
}

