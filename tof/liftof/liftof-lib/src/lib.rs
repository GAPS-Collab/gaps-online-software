//pub mod misc;

use std::error::Error;
use std::time::{Duration, Instant};
use std::thread;
use std::fmt;
use std::{fs, fs::File, path::Path};
use std::fs::OpenOptions;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::net::{IpAddr, Ipv4Addr};
use std::io::{Read,
              Write};
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use crossbeam_channel::{Sender, unbounded};
//use zmq;

extern crate json;

use macaddr::MacAddr6;
use netneighbours::get_mac_to_ip_map;
use crossbeam_channel as cbc; 

extern crate pretty_env_logger;
#[macro_use] extern crate log;

#[macro_use] extern crate manifest_dir_macros;

//use tof_dataclasses::manifest::LocalTriggerBoard;
use tof_dataclasses::manifest as mf;
use tof_dataclasses::events::master_trigger::{read_daq, read_rate, reset_daq};
use tof_dataclasses::constants::{NCHN,
                                 NWORDS};
use tof_dataclasses::calibrations::{Calibrations,
                                    read_calibration_file};
use tof_dataclasses::events::blob::{BlobData,
                                    get_constant_blobeventsize};
use tof_dataclasses::packets::PacketType;
use tof_dataclasses::packets::paddle_packet::PaddlePacket;
use tof_dataclasses::errors::{BlobError, SerializationError};
use tof_dataclasses::commands::{TofCommand};//, TofResponse};
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::events::MasterTriggerEvent;
use tof_dataclasses::serialization::{Serialization,
                                     parse_u16,
                                     parse_u32};

pub const MT_MAX_PACKSIZE   : usize = 512;

//*************************************************
// I/O - read/write (general purpose) files
//
//



fn read_value_from_file(file_path: &str) -> io::Result<u32> {
  let mut file = File::open(file_path)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  let value: u32 = contents.trim().parse().map_err(|err| {
    io::Error::new(io::ErrorKind::InvalidData, err)
  })?;
  Ok(value)
}

/// The output is wrapped in a Result to allow matching on errors
/// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// Read a file as a vector of bytes
///
/// This reads the entire file in a 
/// single vector of chars.
///
/// # Arguments 
///
/// * fliename (String) : Name of the file to read in 
#[deprecated(since="0.4.0", note="please use `tof_dataclasses::io::read_file` instead")]
pub fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    return buffer;
}

/// Open a raw file with ReadoutBoard data ("blob") and run decoding and analysis
/// FIXME - this won't work!
pub fn get_blobs_from_file (rb_id : usize) {
//  let filepath = String::from("/data0/gfp-data-aug/Aug/run4a/d20220809_195753_4.dat");
//  let blobs = get_file_as_byte_vec(&filepath);
//  // FIXME - this must be thre real calibrations
//  let calibrations = [Calibrations {..Default::default()};NCHN];
//  //let sender = Sender::<PaddlePacket>();
//  let (sender, receiver) = unbounded();
//  todo!("Fix the paddle ids. This function needs to be given the Readoutboard!");
//  let paddle_ids : [u8;4] = [0,0,0,0];
//  let mut rb = ReadoutBoard::new();
//  rb.id = Some(rb_id as u8);
//  rb.sorted_pids = paddle_ids;
//  match analyze_blobs(&blobs,
//                      &sender,
//                      false,
//                      &rb,
//                      false,
//                      false,
//                      &calibrations,
//                      0) {
//      Ok(nblobs)   => info!("Read {} blobs from file", nblobs), 
//      Err(err)     => panic!("Was not able to read blobs! Err {}", err)
//  }
}




/// Open a new file and write TofPackets
/// in binary representation
///
/// One packet per line
///
pub struct TofPacketWriter {

  //pub filename : String,
  pub file        : File,
  pub file_prefix : String,
  pkts_per_file   : usize,
  file_id         : usize,
  n_packets       : usize,
}

impl TofPacketWriter {

  /// Instantiate a new PacketWriter 
  ///
  /// # Arguments
  ///
  /// * file_prefix : Prefix file with this string. A continuous number will get 
  ///                 appended to control the file size.
  pub fn new(file_prefix : String) -> TofPacketWriter {
    let filename = file_prefix.clone() + "_0.tof.gaps";
    let path = Path::new(&filename); 
    println!("Writing to file {filename}");
    let file = OpenOptions::new().create(true).append(true).open(path).expect("Unable to open file {filename}");
    TofPacketWriter {
      file,
      file_prefix   : file_prefix,
      pkts_per_file : 10000,
      file_id : 0,
      n_packets : 0,
    }
  }

  pub fn add_tof_packet(&mut self, packet : &TofPacket) {
    let buffer = packet.to_bytestream();
    match self.file.write_all(buffer.as_slice()) {
      Err(err) => error!("Writing to file with prefix {} failed. Err {}", self.file_prefix, err),
      Ok(_)    => ()
    }
    match self.file.sync_all() {
      Err(err) => error!("File syncing failed!"),
      Ok(_)    => ()
    }
    self.n_packets += 1;
    if self.n_packets == self.pkts_per_file {
      //drop(self.file);
      let filename = self.file_prefix.clone() + "_" + &self.file_id.to_string() + ".tof.gaps";
      let path  = Path::new(&filename);
      self.file = OpenOptions::new().append(true).open(path).expect("Unable to open file {filename}");
      self.n_packets = 0;
    }
  }
}

impl Default for TofPacketWriter {
  fn default() -> TofPacketWriter {
    TofPacketWriter::new(String::from(""))
  }

}

/// Meta information for a data run
#[deprecated(since="0.2.0", note="please use `tof_dataclasses::RunConfig` instead")]
#[derive(Debug, Copy, Clone)]
pub struct RunParams {
  pub forever   : bool,
  pub nevents   : u32,
  pub is_active : bool,
  pub nseconds  : u32,
}

impl RunParams {

  pub const SIZE               : usize = 14; // bytes
  pub const VERSION            : &'static str = "1.0";
  pub const HEAD               : u16  = 43690; //0xAAAA
  pub const TAIL               : u16  = 21845; //0x5555

  pub fn new() -> RunParams {
    RunParams {
      forever   : false,
      nevents   : 0,
      is_active : false,
      nseconds  : 0,
    }
  }

  pub fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(RunParams::SIZE);
    stream.extend_from_slice(&RunParams::HEAD.to_le_bytes());
    let mut forever = 0u8;
    if self.forever {
      forever = 1;
    }
    stream.extend_from_slice(&forever.to_le_bytes());
    stream.extend_from_slice(&self.nevents.to_le_bytes());
    let mut is_active = 0u8;
    if self.is_active {
      is_active = 1;
    }
    stream.extend_from_slice(&is_active.to_le_bytes());
    stream.extend_from_slice(&self.nseconds.to_le_bytes());
    stream
  }
}

impl Serialization for RunParams {
  
  fn from_bytestream(bytestream : &Vec<u8>,
                     pos        : &mut usize)
    -> Result<Self, SerializationError> {
    let mut pars = RunParams::new();
    if parse_u16(bytestream, pos) != RunParams::HEAD {
      return Err(SerializationError::HeadInvalid {});
    }
    let forever   = bytestream[*pos];
    *pos += 1;
    pars.nevents  = parse_u32(bytestream, pos);
    let is_active = bytestream[*pos];
    *pos += 1;
    pars.nseconds = parse_u32(bytestream, pos);
    if parse_u16(bytestream, pos) != RunParams::TAIL {
      return Err(SerializationError::TailInvalid {} );
    }
    pars.is_active = is_active > 0;
    pars.forever   = forever > 0;
    Ok(pars)
  }
}

impl Default for RunParams {
  fn default() -> RunParams {
    RunParams::new()
  }
}

impl fmt::Display for RunParams {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<RunParams : active {}>", self.is_active)
  }
}

//**********************************************
//
// Analysis
//

/// FIXME - we have to think again, which queues are really 
/// needed. I think:
/// BlobData queue : only needed when diagnostics feature is
/// set to write the waveforms to hdf
/// PaddlePacket queue : I don't think is needed for anything
/// since we are sending the packets right away
///
/// # Arguments
///
/// * paddle_ids     :  A sorted list of paddle ids connected 
///                     to this readoutboard. 
///                     The is sorted like (0,1) -> pid1
///                                        (2,3) -> pid2
///                                        (4,5) -> pid3
///                                        (6,7) -> pid4
///
///
pub fn analyze_blobs(buffer               : &Vec<u8>,
                     pp_sender            : &Sender<PaddlePacket>,
                     send_packets         : bool,
                     readoutboard         : &mf::ReadoutBoard,
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
  //const NPADDLES : usize = (NCHN - 1)/2; // assuming one channel 
  //                                         // is the channel 9
  const NPADDLES : usize = 4;


  // each event has NPADDLES per readout board
  // this holds all for a single event
  // (needs to be reset each event)
  let mut pp_this_event = [PaddlePacket::new(); NPADDLES];        
  
  // either all or only the triggered paddle ids
  let mut all_pids = readoutboard.get_all_pids();
  //all_pids         = readoutboard.get_triggered_pids();
  let mut paddles = HashMap::<u8, PaddlePacket>::new();
  for k in all_pids.iter() {
    match paddles.insert(*k, PaddlePacket::new()) {
      None => (),
      Some(v) => {error!("We have seen paddle id {k} already!");}
    };
  }
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
  // reset paddle packets for this event
  for n in 0..NPADDLES {
    pp_this_event[n].reset();
    paddles_over_threshold[n] = false;
  }

  // the stream might have a certain number of events, 
  // but then there might be a number of extra bytes.

  loop {
    #[cfg(feature = "diagnostics")]
    let mut diagnostics_wf : Vec<CalibratedWaveform> = Vec::new();
    
    // if the following is true, we scanned throught the whole stream  
    //println!("{pos} {blobdata_size}");
    //let foo = get_constant_blobeventsize();
    //println!("{foo}");
    // this is of constant annoyance!
    //if pos + get_constant_blobeventsize() >= (blobdata_size -1) {break;}
    //if pos + get_constant_blobeventsize() > (blobdata_size ) {break;}
    if pos > buffer.len() - 1 {
      trace!("too big");
      break;
    }
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
        // if there is not enough bytes for another blob, 
        // lets break the loop
        if pos -2 + get_constant_blobeventsize() > buffer.len()
          {break;}
        blob_data.reset();
        //if (pos-2 > buffer.len() -1) {break;}
        blob_data.from_bytestream(&buffer, pos-2, false);
        nblobs += 1;

        // reset the paddles for this board
        for k in all_pids.iter() {
          paddles.get_mut(k).map(|val| val.reset());
        }

        if blob_data.tail == 0x5555 {
            if print_events {blob_data.print();}
            pos += get_constant_blobeventsize() - 2; 
            for k in all_pids.iter() {
              paddles.get_mut(k).unwrap().paddle_id    = *k;
              paddles.get_mut(k).unwrap().event_id     = blob_data.event_id;
              paddles.get_mut(k).unwrap().timestamp_32 = blob_data.timestamp_32;
              paddles.get_mut(k).unwrap().timestamp_16 = blob_data.timestamp_16;
            }

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
              for ch in 0..8 {

                // reset our channels_over_threshold
                channels_over_threshold[ch] = false;


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
                //if !is_ot {continue;}
                channels_over_threshold[ch] = true;
                
                blob_data.set_cfds_fraction(0.20, ch);
                blob_data.find_peaks(270.0,70.0, ch);
                // analysis
                let cfd_time = blob_data.find_cfd_simple(0, ch);
                let charge = blob_data.integrate(270.0, 70.0, ch).unwrap_or(42.0);
                let pid = readoutboard.get_pid_for_ch(ch + 1 );
                let end = readoutboard.get_paddle_end(ch + 1 );
                match end {
                  // unwraps can't fail due to construction of paddles
                  mf::PaddleEndIdentifier::A => {
                    paddles.get_mut(&pid).unwrap().set_time_a(cfd_time);
                    paddles.get_mut(&pid).unwrap().set_charge_a(charge);
                  },
                  mf::PaddleEndIdentifier::B => {
                    paddles.get_mut(&pid).unwrap().set_time_a(cfd_time);
                    paddles.get_mut(&pid).unwrap().set_charge_a(charge);
                  }
                }//waveform.print();
                // packing part
                
                //// FIXME - this is not independent
                //// of the number of channels for the 
                //// readout board
                //match ch {
                //  0 => {
                //    paddles_over_threshold[0] = true;
                //    pp_this_event[0].set_time_a(cfd_time);
                //    pp_this_event[0].set_charge_a(charge);
                //  },
                //  1 => {
                //    paddles_over_threshold[0] = true;
                //    pp_this_event[0].set_time_b(cfd_time);
                //    pp_this_event[0].set_charge_b(charge);

                //  },
                //  2 => {
                //    paddles_over_threshold[1] = true;
                //    pp_this_event[1].set_time_a(cfd_time);
                //    pp_this_event[1].set_charge_a(charge);
                //  },
                //  3 => {
                //    paddles_over_threshold[1] = true;
                //    pp_this_event[1].set_time_b(cfd_time);
                //    pp_this_event[1].set_charge_b(charge);
                //  },
                //  4 => {
                //    paddles_over_threshold[2] = true;
                //    pp_this_event[2].set_time_a(cfd_time);
                //    pp_this_event[2].set_charge_a(charge);
                //  },
                //  5 => {
                //    paddles_over_threshold[2] = true;
                //    pp_this_event[2].set_time_b(cfd_time);
                //    pp_this_event[2].set_charge_b(charge);
                //  },
                //  6 => {
                //    paddles_over_threshold[3] = true;
                //    pp_this_event[3].set_time_a(cfd_time);
                //    pp_this_event[3].set_charge_a(charge);
                //  },
                //  7 => {
                //    paddles_over_threshold[3] = true;
                //    pp_this_event[3].set_time_b(cfd_time);
                //    pp_this_event[3].set_charge_b(charge);
                //  },
                //  _ => {
                //    trace!("Won't do anything for ch {}",ch);
                //  }
                //} // end match

                // now set general properties on the 
                // paddles
                //for n in 0..readoutboard.sorted_pids.len() {
                //  // FIXME
                //  pp_this_event[n].paddle_id    = readoutboard.sorted_pids[n];
                //  pp_this_event[n].event_id     = blob_data.event_id;
                //  pp_this_event[n].timestamp_32 = blob_data.timestamp_32;
                //  pp_this_event[n].timestamp_16 = blob_data.timestamp_16;
                //  //pp_this_event[n].print();
                //  //if paddles_over_threshold[n] {
                //  //  pp_this_event[n].event_id = blob_data.event_id;
                //  //}
                //}
                
                #[cfg(feature = "diagnostics")]
                {  
                  let diag_wf = CalibratedWaveform::new(&blob_data, ch);
                  diagnostics_wf.push (diag_wf);
                }
              } // end loop over readout board channels
            }

            // put the finished paddle packets in 
            // our container
            //for n in 0..NPADDLES {
            for pid in all_pids.iter() {
            //if paddles_over_threshold[n] {
              //if true {
                //trace!("Sending pp to cache for evid {}", pp_this_event[n].event_id);
                //trace!("==> [RBCOM]  Sending {:?}", pp_this_event[n]);
                //pp_sender.send(pp_this_event[n]);
                match pp_sender.send(paddles[pid]) {
                  Err(err) => error!("Can not send padlle packet to cache! Error {err}"),
                  Ok(_)    => ()
                }
              //}
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
                                + &readoutboard.id.to_string()
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
  debug!("==> Deserialized {} blobs! {} blobs were corrupt", nblobs, ncorrupt_blobs);
  Ok(nblobs)
}

//**********************************************
//
// Subsystem communication
//


/// construct a request string which can be broadcast over 0MQ to all the boards
/// ///
/// /// Boards will only send paddle information when this request string is received
pub fn construct_event_request(rb_id : u8) -> String {
  let mut request = String::from("RB");
  if rb_id < 10 {
    request += "0";
  }
  request += &rb_id.to_string();
  request
}


/// Connect to MTB Utp socket
///
/// This will try a number of options to bind 
/// to the local port.
/// 
/// # Arguments 
///
/// * mtb_ip    : IP Adress of the MTB
/// * mtb_port  : Port of the MTB
///
pub fn connect_to_mtb(mt_address : &String) 
  ->io::Result<UdpSocket> {
  let local_port = "0.0.0.0:50100";
  let local_addrs = [
    SocketAddr::from(([0, 0, 0, 0], 50100)),
    SocketAddr::from(([0, 0, 0, 0], 50101)),
    SocketAddr::from(([0, 0, 0, 0], 50102)),
  ];
  //let local_socket = UdpSocket::bind(local_port);
  let local_socket = UdpSocket::bind(&local_addrs[..]);
  let socket : UdpSocket;
  match local_socket {
    Err(err)   => {
      error!("Can not create local UDP port for master trigger connection at {}!, err {}", local_port, err);
      return Err(err);
    }
    Ok(value)  => {
      info!("Successfully bound UDP socket for master trigger communcations to {}", local_port);
      socket = value;
      // this is not strrictly necessary, but 
      // it is nice to limit communications
      match socket.set_read_timeout(Some(Duration::from_millis(1))) {
        Err(err) => error!("Can not set read timeout for Udp socket! Error {err}"),
        Ok(_)    => ()
      }
      match socket.connect(&mt_address) {
        Err(err) => {
          error!("Can not connect to master trigger at {}, err {}", mt_address, err);
          return Err(err);
        }
        Ok(_)    => info!("Successfully connected to the master trigger at {}", mt_address)
      }
      return Ok(socket);
    }
  } // end match
}  

/// Communications with the master trigger over Udp
///
/// The master trigger can send packets over the network.
/// These packets contain timestamps as well as the 
/// eventid and a hitmaks to identify which LTBs have
/// participated in the trigger.
/// The packet format is described
/// [here](https://gitlab.com/ucla-gaps-tof/firmware/-/tree/develop/)
///
/// # Arguments
///
/// * mt_ip       : ip address of the master trigger, most likely 
///                 something like 10.0.1.10
/// * mt_port     : 
///
/// * sender_rate : 
/// 
/// * 
///
/// * verbose     : Print "heartbeat" output 
///
pub fn master_trigger(mt_ip          : &str, 
                      mt_port        : usize,
                      sender_rate    : &cbc::Sender<u32>,
                      evid_sender    : &cbc::Sender<MasterTriggerEvent>,
                      verbose        : bool) {

  let mt_address = mt_ip.to_owned() + ":" + &mt_port.to_string();
 
  let mut socket = connect_to_mtb(&mt_address).expect("Can not create local UDP socket for MTB connection!"); 
  //socket.set_nonblocking(true).unwrap();
  // we only allocate the buffer once
  // and reuse it for all operations
  let mut buffer = [0u8;MT_MAX_PACKSIZE];  
  
  //let mut event_cnt      = 0u32;
  let mut last_event_cnt = 0u32;
  let mut missing_evids  = 0usize;
  //let mut event_missing  = false;
  let mut n_events       = 0usize;
  // these are the number of expected events
  // (missing included)
  let mut n_events_expected = 0usize;
  let mut n_paddles_expected : u32;
  let mut rate : f64;
  // for rate measurement
  let start = Instant::now();

  let mut next_beat = true;
  
  // FIXME - this is a good idea
  // limit polling rate to a maximum
  //let max_rate = 200.0; // hz
    
  // reset the master trigger before acquisiton
  info!("Resetting master trigger");
  match reset_daq(&socket, &mt_address) {
    Err(err) => error!("Can not reset DAQ, error {err}"),
    Ok(_)    => ()
  }
  // the event counter has to be reset before 
  // we connect to the readoutboards
  //reset_event_cnt(&socket, &mt_address); 
  let mut ev : MasterTriggerEvent;// = read_daq(&socket, &mt_address, &mut buffer);
  let mut timeout = Instant::now();
  //let timeout = Duration::from_secs(5);
  info!("Starting MT event loop at {:?}", timeout);
  let mut timer = Instant::now();


  loop {
    // a heartbeat every 10 s
    let elapsed = start.elapsed().as_secs();
    if (elapsed % 10 == 0) && next_beat {
      rate = n_events as f64 / elapsed as f64;
      let expected_rate = n_events_expected as f64 / elapsed as f64; 
      if verbose {
        println!("== == == == == == == == MT HEARTBEAT! {} seconds passed!", elapsed);
        println!("==> {} events recorded, trigger rate: {:.3} Hz", n_events, rate);
        println!("==> -- expected rate {:.3} Hz", expected_rate);   
        println!("== == == == == == == == END HEARTBEAT!");
      }
      next_beat = false;
    } else if elapsed % 10 != 0 {
      next_beat = true;
    }
    if timeout.elapsed().as_secs() > 10 {
      drop(socket);
      socket = connect_to_mtb(&mt_address).expect("Can not create local UDP socket for MTB connection!"); 
      timeout = Instant::now();
    }
    if timer.elapsed().as_secs() > 10 {
      match read_rate(&socket, &mt_address, &mut buffer) {
        Err(err) => {
          error!("Unable to obtain MT rate information! error {err}");
          continue;
        }
        Ok(rate) => {
          info!("Got rate from MTB {rate}");
          match sender_rate.try_send(rate) {
            Err(err) => error!("Can't send rate, error {err}"),
            Ok(_)    => ()
          }
        }
      }
      timer = Instant::now();
    }

    //info!("Next iter...");
    // limit the max polling rate
    
    //let milli_sleep = Duration::from_millis((1000.0/max_rate) as u64);
    //thread::sleep(milli_sleep);
    

    //info!("Done sleeping..."); 
    //match socket.connect(&mt_address) {
    //  Err(err) => panic!("Can not connect to master trigger at {}, err {}", mt_address, err),
    //  Ok(_)    => info!("Successfully connected to the master trigger at {}", mt_address)
    //}
    //  let received = socket.recv_from(&mut buffer);

    //  match received {
    //    Ok((size, addr)) => println!("Received {} bytes from address {}", size, addr),
    //    Err(err)         => {
    //      println!("Received nothing! err {}", err);
    //      continue;
    //    }
    //  } // end match
    
    // daq queue states
    // 0 - full
    // 1 - something
    // 2 - empty
    //if 0 != (read_register(&socket, &mt_address, 0x12, &mut buffer) & 0x2) {
    //if read_register(&socket, &mt_address, 0x12, &mut buffer) == 2 {
    //  trace!("No new information from DAQ");
    //  //reset_daq(&socket, &mt_address);  
    //  continue;
    //}
    
    //event_cnt = read_event_cnt(&socket, &mt_address, &mut buffer);
    //println!("Will read daq");
    //mt_event = read_daq(&socket, &mt_address, &mut buffer);
    //println!("Got event");
    match read_daq(&socket, &mt_address, &mut buffer) {
      Err(err) => {
        trace!("Did not get new event, Err {err}");
        continue;
      }
      Ok(new_event) => {
        ev = new_event; 
      }
    }
    if ev.event_id == last_event_cnt {
      trace!("Same event!");
      continue;
    }

    // sometimes, the counter will just read 0
    // throw these away. 
    // FIXME - there is actually an event with ctr 0
    // but not sure how to address that yet
    if ev.event_id == 0 {
      trace!("event 0 encountered! Continuing...");
      //continue;
    }

    // FIXME
    if ev.event_id == 2863311530 {
      warn!("Magic event number! continuing! 2863311530");
      //continue;
    }

    // we have a new event
    //println!("** ** evid: {}",event_cnt);
    
    // if I am correct, there won't be a counter
    // overflow for a 32bit counter in 99 days 
    // for a rate of 500Hz
    if ev.event_id < last_event_cnt {
      error!("Event counter id overflow! this cntr: {} last cntr: {last_event_cnt}!", ev.event_id);
      last_event_cnt = 0;
      continue;
    }
    
    if ev.event_id - last_event_cnt > 1 {
      let mut missing = ev.event_id - last_event_cnt;
      error!("We missed {missing} eventids"); 
      // FIXME
      if missing < 200 {
        missing_evids += missing as usize;
      } else {
        warn!("We missed too many event ids from the master trigger!");
        //missing = 0;
      }
      //error!("We missed {} events!", missing);
      //event_missing = true;
    }
    
    trace!("Got new event id from master trigger {}",ev.event_id);
    match evid_sender.send(ev) {
      Err(err) => trace!("Can not send event, err {err}"),
      Ok(_)    => ()
    }
    last_event_cnt = ev.event_id;
    n_events += 1;
    n_events_expected = n_events + missing_evids;

    if n_events % 1000 == 0 {
      //let pk = TofPacket::new();
      error!("Sending of mastertrigger packets down the global data sink not supported yet!");
    }

    let elapsed = start.elapsed().as_secs();
    // measure rate every 100 events
    if n_events % 1000 == 0 {
      rate = n_events as f64 / elapsed as f64;
      if verbose {
        println!("==> [MASTERTRIGGER] {} events recorded, trigger rate: {:.3} Hz", n_events, rate);
      }
      rate = n_events_expected as f64 / elapsed as f64;
      if verbose {
        println!("==> -- expected rate {:.3} Hz", rate);   
      }
    } 
    // end new event
  } // end loop
}


/// Get the tof channel/paddle mapping and involved components
///
/// This reads the configuration from a json file and panics 
/// if there are any problems.
///
pub fn get_tof_manifest(json_config : PathBuf) -> (Vec::<LocalTriggerBoard>, Vec::<ReadoutBoard>) {
  let mut ltbs = Vec::<LocalTriggerBoard>::new();
  let mut rbs  = Vec::<ReadoutBoard>::new();
  let js_file = json_config.as_path();
   if !js_file.exists() {
     panic!("The file {} does not exist!", js_file.display());
   }
   info!("Found config file {}", js_file.display());
   let json_content = std::fs::read_to_string(js_file).expect("Unable to read file!");
   let config = json::parse(&json_content).expect("Unable to parse json!");
   for n in 0..config["ltbs"].len() {
     ltbs.push(LocalTriggerBoard::from(&config["ltbs"][n]));
   }
   for n in 0..config["rbs"].len() {
     rbs.push(ReadoutBoard::from(&config["rbs"][n]));
   }
  (ltbs, rbs)
}


#[deprecated(since="0.1.0", note="please use `get_tof_manifest` instead")]
pub fn get_rb_manifest() -> Vec<ReadoutBoard> {
  let rb_manifest_path = path!("assets/rb.manifest");
  let mut connected_boards = Vec::<ReadoutBoard>::new();
  let mac_table = get_mac_to_ip_map();
  if let Ok(lines) = read_lines(rb_manifest_path) {
    // Consumes the iterator, returns an (Optional) String
    for line in lines {
      if let Ok(ip) = line {
        if ip.starts_with("#") {
          continue;
        }
        if ip.len() == 0 {
          continue;
        }
        let identifier: Vec<&str> = ip.split(";").collect();
        debug!("{:?}", identifier);
        let mut rb = ReadoutBoard::new();
        let mc_address = identifier[1].replace(" ","");
        let mc_address : Vec<&str> = mc_address.split(":").collect();
        println!("{:?}", mc_address);
        let mc_address : Vec<u8>   = mc_address.iter().map(|&x| {u8::from_str_radix(x,16).unwrap()} ).collect();
        assert!(mc_address.len() == 6);
        let mac = MacAddr6::new(mc_address[0],
                                mc_address[1],
                                mc_address[2],
                                mc_address[3],
                                mc_address[4],
                                mc_address[5]);

        rb.id          = Some(identifier[0].parse::<u8>().expect("Invalid RB ID!"));
        rb.mac_address = Some(mac);
        let rb_ip = mac_table.get(&mac);
        println!("Found ip address {:?}", rb_ip);
        match rb_ip {
          None => println!("Can not resolve RBBoard with MAC address {:?}, it is not in the system's ARP tables", mac),
          Some(ip)   => match ip[0] {
            IpAddr::V6(a) => panic!("IPV6 {a} not suppported!"),
            IpAddr::V4(a) => {
              rb.ip_address = Some(a);
              rb.data_port  = Some(42000);
              connected_boards.push(rb);
              // now we will try and check if the ports are open
              //let mut all_data_ports = Vec::<String>::new();//scan_ports_range(30000..39999);
              //let mut all_cmd_ports  = Vec::<String>::new();//scan_ports_range(40000..49999);
              //// FIXME - the ranges here are somewhat arbitrary
              //for n in 30000..39999 {
              //  all_data_ports.push(rb.ip_address.unwrap().to_string() + ":" + &n.to_string());
              //  //scan_ports_addrs(
              //}
              //for n in 40000..49999 {
              //  all_cmd_ports.push(rb.ip_address.unwrap().to_string() + ":" + &n.to_string());
              //}
              //let open_data_ports = scan_ports_addrs(all_data_ports);
              //let open_cmd_ports  = scan_ports_addrs(all_cmd_ports);
              //assert!(open_cmd_ports.len() < 2);
              //assert!(open_data_ports.len() < 2);
              //if open_cmd_ports.len() == 1 {
              //  rb.cmd_port = Some(open_cmd_ports[0].port());
              //  match rb.ping() {
              //    Ok(_)    => println!("... connected!"),
              //    Err(err) => println!("Can't connect to RB, err {err}"),
              //  }
              //} else {
              //  rb.cmd_port = None;
              //}
              //

              //println!("Found open data ports {:?}", open_data_ports);
              //if open_data_ports.len() == 1 {
              //  rb.data_port = Some(open_data_ports[0].port());
              //} else {
              //  rb.data_port = None;
              //}
              //if rb.is_connected {
              //  connected_boards.push(rb);
              //}
            }
          }
        }

        
        println!("{:?}", connected_boards);
      }
    }
  }
  return connected_boards;
}



#[derive(Debug)]
pub enum ReadoutBoardError {
  NoConnectionInfo,
  NoResponse,
}


impl fmt::Display for ReadoutBoardError{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let disp : String;
    match self {
      ReadoutBoardError::NoConnectionInfo => {disp = String::from("NoConnectionInfo");},
      ReadoutBoardError::NoResponse       => {disp = String::from("NoResponse");},
    } 
    write!(f, "<ReadoutBoardError : {}>", disp)
  }
}

impl Error for ReadoutBoardError {
}

/// Find boards in the network
///
///
///
//pub fn discover_boards() -> Vec<ReadoutBoard> {
//  let board_list = Vec::<ReadoutBoard>::new();
//  board_list
//}


/// A generic representation of a LocalTriggerBoard
///
/// This is important to make the mapping between 
/// trigger information and readoutboard.
#[derive(Debug, Clone)]
pub struct LocalTriggerBoard {
  pub id : u8,
  /// The LTB has 16 channels, 
  /// which are connected to the RBs
  /// Each channel corresponds to a 
  /// specific RB channel, represented
  /// by the tuple (RBID, CHANNELID)
  pub ch_to_rb : [(u8,u8);16],
  /// the MTB bit in the MTEvent this 
  /// LTB should reply to
  pub mt_bitmask : u32,
}

impl LocalTriggerBoard {
  pub fn new() -> LocalTriggerBoard {
    LocalTriggerBoard {
      id : 0,
      ch_to_rb : [(0,0);16],
      mt_bitmask : 0
    }
  }

  /// Calculate the position in the bitmask from the connectors
  pub fn get_mask_from_dsi_and_j(dsi : u8, j : u8) -> u32 {
    if dsi == 0 || j == 0 {
      warn!("Invalid dsi/J connection!");
      return 0;
    }
    let mut mask : u32 = 1;
    mask = mask << (dsi*5 + j -1) ;
    mask
  }
}

impl fmt::Display for LocalTriggerBoard {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<LTB: \n ID \t\t: {} \n bitmask \t\t: {} \n channels \t: {:?} >", 
            self.id.to_string() ,
            self.mt_bitmask.to_string(),
            self.ch_to_rb
    )
  }
}

impl From<&json::JsonValue> for LocalTriggerBoard {
  fn from(json : &json::JsonValue) -> Self {
    let id  = json["id"].as_u8().expect("id value json problem");
    let dsi = json["DSI"].as_u8().expect("DSI value json problem");
    let j   = json["J"].as_u8().expect("J value json problem");
    //let mask = LocalTriggerBoard::get_mask_from_dsi_and_j(dsi, j);
    let channels = &json["ch_to_rb"];//.members();
    let mut rb_channels = [(0, 0);16];
    for ch in 0..channels.len() {
      if channels.has_key(&ch.to_string()) {
        rb_channels[ch] = (channels[&ch.to_string()][0].as_u8().unwrap(),
                           channels[&ch.to_string()][1].as_u8().unwrap());  
      }
    }
    let bitmask = LocalTriggerBoard::get_mask_from_dsi_and_j(dsi, j);
    LocalTriggerBoard {
      id : id,
      ch_to_rb : rb_channels,
      mt_bitmask : bitmask
    }
  }
}

/// A generic representation of a Readout board
///
///
///
#[derive(Debug, Clone)]
pub struct ReadoutBoard {
  pub id           : Option<u8>,
  pub mac_address  : Option<MacAddr6>,
  pub ip_address   : Option<Ipv4Addr>, 
  pub data_port    : Option<u16>,
  pub cmd_port     : Option<u16>,
  pub is_connected : bool,
  pub uptime       : u32,
  pub ch_to_pid    : [u8;8],
  pub sorted_pids  : [u8;4],
  pub calib_file   : String,
  pub configured   : bool,
}

impl ReadoutBoard {

  pub fn new() -> ReadoutBoard {
    ReadoutBoard {
      id            : None,
      mac_address   : None,
      ip_address    : None,
      data_port     : None,
      cmd_port      : None,
      is_connected  : false,
      uptime        : 0,
      ch_to_pid     : [0;8],
      sorted_pids   : [0;4], 
      calib_file    : String::from(""),
      configured    : false
    }
  }

  pub fn get_connection_string(&mut self) -> String {
    if !self.configured {
      panic!("Can not get connection string. This board has not been configured. Get the information from corresponding json tof manifest");
    }

    self.get_ip();
    let mut address_ip = String::from("tcp://");
    match self.ip_address {
      None => panic!("This board does not have an ip address. Unable to obtain connection information"),
      Some(ip) => {
        address_ip = address_ip + &ip.to_string();
      }
    }
    match self.data_port {
      None => panic!("This board does not have a known data port. Typically, this should be 42000. Please check your tof-manifest.jsdon"),
      Some(port) => {
        address_ip += &":".to_owned();
        address_ip += &port.to_string();
      }
    }
    address_ip
  }

  /// Get the readoutboard ip address from 
  /// the ARP tables
  pub fn get_ip(&mut self) {
    let mac_table = get_mac_to_ip_map();
    let rb_ip = mac_table.get(&self.mac_address.unwrap());
    info!("Found ip address {:?} for RB {}", rb_ip, self.id.unwrap_or(0));
    match rb_ip {
      None => panic!("Can not resolve RBBoard with MAC address {:?}, it is not in the system's ARP tables", &self.mac_address),
      Some(ip)   => match ip[0] {
        IpAddr::V6(a) => panic!("IPV6 {a} not suppported!"),
        IpAddr::V4(a) => {
          self.ip_address = Some(a); 
        }
      }
    }
  }
    
  ///// Ping it  
  //pub fn ping(&mut self) -> Result<(), Box<dyn Error>> { 
  //  // connect to the command port and send a ping
  //  // message
  //  let ctx =  zmq::Context::new();
  //  if matches!(self.ip_address, None) || matches!(self.cmd_port, None) {
  //    self.is_connected = false;
  //    return Err(Box::new(ReadoutBoardError::NoConnectionInfo));
  //  }
  //  let address = "tcp://".to_owned() + &self.ip_address.unwrap().to_string() + ":" + &self.cmd_port.unwrap().to_string(); 
  //  let socket  = ctx.socket(zmq::REQ)?;
  //  socket.connect(&address)?;
  //  info!("Have connected to adress {address}");
  //  // if the readoutboard is there, it should send *something* back
  //  let p = TofCommand::Ping(1);

  //  socket.send(p.to_bytestream(), 0)?;
  //  info!("Sent ping signal, waiting for response!");
  //  let data = socket.recv_bytes(0)?;
  //  if data.len() != 0 {
  //    self.is_connected = true;
  //    return Ok(());
  //  }
  //  self.is_connected = false;
  //  return Err(Box::new(ReadoutBoardError::NoResponse));
  //}
}

impl fmt::Display for ReadoutBoard {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let default_ip  = Ipv4Addr::new(0,0,0,0);
    let default_mac = MacAddr6::default();
    write!(f, "<ReadoutBoard: \n ID \t\t: {} \n MAC addr \t: {} \n IP addr \t: {} \n 0MQ PUB \t: {} \n 0MQ REP \t: {} \n connected \t: {}\n calib file \t: {} \n uptime \t: {} >", 
            self.id.unwrap_or(0).to_string()           ,      
            self.mac_address.unwrap_or(default_mac).to_string()  ,
            self.ip_address.unwrap_or(default_ip).to_string()   ,
            self.data_port.unwrap_or(0).to_string()    ,
            self.cmd_port.unwrap_or(0)     , 
            self.is_connected.to_string() , 
            "?",
            //&self.calib_file.unwrap_or(String::from("")),
            self.uptime.to_string()       ,
    )
  }
}

impl Default for ReadoutBoard {
  fn default() -> ReadoutBoard {
    ReadoutBoard::new()
  }
}

impl From<&json::JsonValue> for ReadoutBoard {
  fn from(json : &json::JsonValue) -> Self {
    let mut board =  ReadoutBoard::new();
    board.id = Some(json["id"].as_u8().unwrap());
    //let identifier: Vec<&str> = ip.split(";").collect();
    let identifier = json["mac_address"].as_str().unwrap();
    let mc_address = identifier.replace(" ","");
    let mc_address : Vec<&str> = mc_address.split(":").collect();
    println!("{:?}", mc_address);
    let mc_address : Vec<u8>   = mc_address.iter().map(|&x| {u8::from_str_radix(x,16).unwrap()} ).collect();
    assert!(mc_address.len() == 6);
    let mac = MacAddr6::new(mc_address[0],
                            mc_address[1],
                            mc_address[2],
                            mc_address[3],
                            mc_address[4],
                            mc_address[5]);
    let data_port = Some(json["port"].as_u16().unwrap());
    let calib_file = json["calibration_file"].as_str().unwrap();
    board.mac_address = Some(mac);
    board.data_port   = data_port;
    board.calib_file  = calib_file.to_string();
    board.get_ip();
    let ch_to_pid = &json["ch_to_pid"];
    let mut ch_true : usize = 1;
    for ch in 0..ch_to_pid.len() {
      ch_true = ch + 1;
      //println!("{ch}");
      //println!("{:?}", json["ch_to_pid"]);
      match json["ch_to_pid"][&ch_true.to_string()].as_u8() {
        Some(foo) => {board.ch_to_pid[ch] = foo;}
        None => {
          error!("Can not get data for ch {ch}");
          board.ch_to_pid[ch] = 0;
        }
      }
      //board.ch_to_pid[ch] = json["ch_to_pid"][&ch_true.to_string()].as_u8().unwrap();
    }
    let mut paddle_ids : [u8;4] = [0,0,0,0];
    let mut counter = 0;
    for ch in board.ch_to_pid.iter().step_by(2) {
      paddle_ids[counter] = *ch;
      counter += 1;
    }
    board.sorted_pids = paddle_ids;
    board.configured  = true;
    board
  }
}

#[test]
fn test_display() {
  let rb = ReadoutBoard::default();
  println!("Readout board {}", rb);
  assert_eq!(1,1);
}


#[test]
fn show_manifest() {
  get_rb_manifest();
  assert_eq!(1,1);
}
