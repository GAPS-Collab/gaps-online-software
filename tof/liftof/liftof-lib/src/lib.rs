pub mod master_trigger;
pub use master_trigger::{connect_to_mtb,
                         master_trigger};

use std::error::Error;
use std::fmt;
use std::{
    fs::File,
};
use std::path::PathBuf;
use std::fs::read_to_string;
use std::io::{
    self,
    Read,
    Write,
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use crossbeam_channel::Receiver;
use zmq;
use colored::{Colorize, ColoredString};

use serde_json::Value;

use log::Level;
use macaddr::MacAddr6;
use netneighbours::get_mac_to_ip_map;

#[macro_use] extern crate log;
extern crate env_logger;

use nalgebra::{DMatrix, DVector};
use nalgebra::linalg::SVD;

//use tof_dataclasses::manifest as mf;
use tof_dataclasses::DsiLtbRBMapping;
use tof_dataclasses::constants::NWORDS;
use tof_dataclasses::calibrations::RBCalibrations;
use tof_dataclasses::packets::{TofPacket,
                               PacketType};
use tof_dataclasses::errors::{SerializationError,
                              AnalysisError};
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::commands::RBCommand;
use tof_dataclasses::events::{
    RBEvent,
    TofHit,
};
use tof_dataclasses::io::TofPacketReader;

use tof_dataclasses::events::tof_hit::Peak;

use tof_dataclasses::analysis::{calculate_pedestal,
                                integrate,
                                cfd_simple,
                                find_peaks};

use tof_dataclasses::RBChannelPaddleEndIDMap;

pub const MT_MAX_PACKSIZE   : usize = 512;
pub const DATAPORT          : u32   = 42000;
pub const ASSET_DIR         : &str  = "/home/gaps/assets/"; 
pub const LIFTOF_LOGO_SHOW  : &str  = "
                                  ___                         ___           ___     
                                 /\\__\\                       /\\  \\         /\\__\\    
                    ___         /:/ _/_         ___         /::\\  \\       /:/ _/_   
                   /\\__\\       /:/ /\\__\\       /\\__\\       /:/\\:\\  \\     /:/ /\\__\\  
    ___     ___   /:/__/      /:/ /:/  /      /:/  /      /:/  \\:\\  \\   /:/ /:/  /  
   /\\  \\   /\\__\\ /::\\  \\     /:/_/:/  /      /:/__/      /:/__/ \\:\\__\\ /:/_/:/  /   
   \\:\\  \\ /:/  / \\/\\:\\  \\__  \\:\\/:/  /      /::\\  \\      \\:\\  \\ /:/  / \\:\\/:/  /    
    \\:\\  /:/  /   ~~\\:\\/\\__\\  \\::/__/      /:/\\:\\  \\      \\:\\  /:/  /   \\::/__/     
     \\:\\/:/  /       \\::/  /   \\:\\  \\      \\/__\\:\\  \\      \\:\\/:/  /     \\:\\  \\     
      \\::/  /        /:/  /     \\:\\__\\          \\:\\__\\      \\::/  /       \\:\\__\\    
       \\/__/         \\/__/       \\/__/           \\/__/       \\/__/         \\/__/    

          (LIFTOF - liftof is for tof, Version 0.8 'NIUHI', Dec 2023)

          * Documentation
          ==> GitHub   https://github.com/GAPS-Collab/gaps-online-software/tree/NIUHI-0.8
          ==> API docs https://gaps-collab.github.io/gaps-online-software/

  ";

/// Make sure that the loglevel is in color, even though not using pretty_env logger
pub fn color_log(level : &Level) -> ColoredString {
  match level {
    Level::Error    => String::from(" ERROR!").red(),
    Level::Warn     => String::from(" WARN  ").yellow(),
    Level::Info     => String::from(" Info  ").green(),
    Level::Debug    => String::from(" debug ").blue(),
    Level::Trace    => String::from(" trace ").cyan(),
  }
}

/// Set up the environmental (env) logger
/// with our format
///
/// Ensure that the lines and module paths
/// are printed in the logging output
pub fn init_env_logger() {
  env_logger::builder()
    .format(|buf, record| {
    writeln!( buf, "[{level}][{module_path}:{line}] {args}",
      level = color_log(&record.level()),
      module_path = record.module_path().unwrap_or("<unknown>"),
      line = record.line().unwrap_or(0),
      args = record.args()
      )
    }).init();
}

/// Keep track of run related statistics, errors
#[derive(Debug, Copy, Clone)]
pub struct RunStatistics {
  /// The number of events we have recorded
  pub n_events_rec      : usize,
  /// The number of packets going through 
  /// the event processing
  pub evproc_npack      : usize,
  /// The first event id we saw
  pub first_evid        : u32,
  /// The last event id we saw
  pub last_evid         : u32,
  /// The number of times we encountered 
  /// a deserialization issue
  pub n_err_deser       : usize,
  /// The number of times we encountered 
  /// an issue while sending over zmq
  pub n_err_zmq_send    : usize,
  /// The number of times we encountered
  /// an issue with a wrong channel identifier
  pub n_err_chid_wrong  : usize,
  /// How many times did we read out an incorrect
  /// tail?
  pub n_err_tail_wrong  : usize,
  /// The number of times we failed a crc32 check
  pub n_err_crc32_wrong : usize,
}

impl RunStatistics {
  
  pub fn new() -> Self {
    Self {
      n_events_rec      : 0,
      evproc_npack      : 0,
      first_evid        : 0,
      last_evid         : 0,
      n_err_deser       : 0,
      n_err_zmq_send    : 0,
      n_err_chid_wrong  : 0,
      n_err_tail_wrong  : 0,
      n_err_crc32_wrong : 0,
    }
  }

  pub fn get_n_anticipated(&self) -> i32 {
    self.last_evid as i32 - self.first_evid as i32
  }
}

impl fmt::Display for RunStatistics {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut resp = String::from("<RunStatistics:\n");
    resp += &(format!("  first event id : {}\n", self.first_evid));
    resp += &(format!("  last  event id : {}\n", self.last_evid));
    resp += &(format!("  --> expected {} event (ids)\n", self.get_n_anticipated()));
    resp += &(format!("  event_processing #packets : {}\n", self.evproc_npack));
    if self.get_n_anticipated() != self.evproc_npack as i32 {
      resp += &(format!("  --> discrepancy of {} event (ids)\n", self.get_n_anticipated() - self.evproc_npack as i32))
    }
    resp += &(format!("  event_processing n tail err : {}\n", self.n_err_tail_wrong));
    resp += &(format!("  event_processing n chid err : {}\n", self.n_err_chid_wrong));
    write!(f, "{}", resp)
  }
}

/// FIXME - this comes straight right out of ChatGPT
fn fit_sine_function(time: Vec<f32>, data: Vec<f32>) -> (f32, f32, f32) {
    // Build the design matrix A
    let a = DMatrix::<f32>::from_fn(time.len(), 3, |i, j| {
        match j {
            0 => 1.0,
            1 => (2.0 * std::f32::consts::PI * time[i]).sin(),
            2 => (2.0 * std::f32::consts::PI * time[i]).cos(),
            _ => unreachable!(),
        }
    });

    // Create the target vector b
    let b = DVector::from_vec(data);

    // Solve the linear system Ax = b using Singular Value Decomposition
    let svd = SVD::new(a.clone(), true, true);
    let eps = 0.0001;
    match svd.solve(&b, eps) {
      Err(err) => {
        error!("Sinus fit failed! {err}");
        return (f32::MAX, f32::MAX, f32::MAX);
      },
      Ok(x) => {
        let amplitude = x[0];
        let frequency = x[1] / (2.0 * std::f64::consts::PI) as f32;
        let phase = -x[2].atan2(x[1]);
        return (amplitude as f32, frequency as f32, phase as f32);
      }
    }

    // Extract parameters from the solution vector x
    //let amplitude = x[0];
    //let frequency = x[1] / (2.0 * std::f64::consts::PI);
    //let phase = -x[2].atan2(x[1]);
    //(0.0,0.0,0.0)
    //(amplitude, frequency, phase)
}


//*************************************************
// I/O - read/write (general purpose) files
//
//
pub fn read_value_from_file(file_path: &str) -> io::Result<u32> {
  let mut file = File::open(file_path)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  let value: u32 = contents.trim().parse().map_err(|err| {
    io::Error::new(io::ErrorKind::InvalidData, err)
  })?;
  Ok(value)
}





/**************************************************/


/// Helper function to generate a proper tcp string starting
/// from the ip one.
pub fn build_tcp_from_ip(ip: String, port: String) -> String {
  String::from("tcp://") + &ip + ":" + &port
}

/// Broadcast commands over the tof-computer network
/// socket via zmq::PUB to the rb network.
/// Currently, the only participants in the rb network
/// are the readoutboards.
///
/// After the reception of a TofCommand, this will currently be 
/// broadcasted to all readout boards.
///
/// ISSUE/FIXME  : Send commands only to specific addresses.
///
/// # Arguments 
///
/// * cmd        : a [crossbeam] receiver, to receive 
///                TofCommands.
pub fn readoutboard_commander(cmd : &Receiver<TofPacket>){
  debug!(".. started!");
  let ctx = zmq::Context::new();
  //let this_board_ip = local_ip().expect("Unable to obtainl local board IP. Something is messed up!");
  let this_board_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 1, 1));

  let address_ip;
  match this_board_ip {
    IpAddr::V4(ip) => address_ip = ip.to_string().clone(),
    IpAddr::V6(_) => panic!("Currently, we do not support IPV6!")
  }
  let data_address : String = build_tcp_from_ip(address_ip,DATAPORT.to_string());
  let data_socket = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  data_socket.bind(&data_address).expect("Unable to bind to data (PUB) socket {data_adress}");
  println!("==> 0MQ PUB socket bound to address {data_address}");
  loop {
    // check if we get a command from the main 
    // thread
    match cmd.try_recv() {
      Err(err) => trace!("Did not receive a new command, error {err}"),
      Ok(packet) => {
        // now we have several options
        match packet.packet_type {
          PacketType::TofCommand => {
            info!("Received TofCommand! Broadcasting to all TOF entities who are listening!");
            let mut payload  = String::from("BRCT").into_bytes();
            payload.append(&mut packet.to_bytestream());
            match data_socket.send(&payload,0) {
              Err(err) => error!("Unable to send command! Error {err}"),
              Ok(_)    => info!("BRCT command sent!")
            }
          },
          PacketType::RBCommand => {
            debug!("Received RBCommand!");
            let mut payload_str  = String::from("RB");
            match RBCommand::from_bytestream(&packet.payload, &mut 0) {
              Ok(rb_cmd) => {
                let to_rb_id = rb_cmd.rb_id;
                if rb_cmd.rb_id < 10 {
                  payload_str += &String::from("0");
                  payload_str += &to_rb_id.to_string();
                } else {
                  payload_str += &to_rb_id.to_string();
                }

                let mut payload = payload_str.into_bytes();
                payload.append(&mut packet.to_bytestream());
                match data_socket.send(&payload,0) {
                  Err(err) => error!("Unable to send command {}! Error {err}", rb_cmd),
                  Ok(_)    => debug!("Making event request! {}", rb_cmd)
                }
              }
              Err(err) => {
                error!("Can not construct RBCommand, error {err}");
              }
            }
          },
          _ => {
            error!("Received garbage package! {}", packet);
          }
        }// end match
      }
    }
  }
}

//**********************************************
//
// Analysis
//

/// Extract peaks from waveforms
///
/// Helper for waveform analysis
pub fn get_peaks() -> Vec<Peak> {
  let peaks = Vec::<Peak>::new();
  peaks
}

/// FIXME - this shoudl be 
/// RBCalibrations::from_tofpacket
/// but for that we have to move the
/// TofPacketReader first
pub fn load_calibration(board_id : u8,
                        path     : String) -> Result<RBCalibrations, SerializationError> {
  let mut cali   = RBCalibrations::new(board_id);
  let mut reader = TofPacketReader::new(path);
  match reader.next() {
    None => {
      error!("Can't load calibration!");
    },
    Some(pack) => {
      cali = RBCalibrations::from_bytestream(&pack.payload, &mut 0)?;
    }
  }
  //cali
  Ok(cali)
}

/// Waveform analysis engine - identify waveform variables
///
/// This will populate the TofHits in an RBEvent
///
/// TofHits contain information about peak location,
/// charge, timing.
///
/// FIXME - I think this should take a HashMap with 
/// algorithm settings, which we can load from a 
/// json file
///
/// # Arguments
///
/// * event       : current RBEvent with waveforms to 
///                 work on
/// * channel_map : (HashMap) mapping providing channel
///                 to paddle ID information
/// * calibration : latest readoutboard calibration for 
///                 the same board
pub fn waveform_analysis(event         : &mut RBEvent,
                         channel_map   : &RBChannelPaddleEndIDMap,
                         calibration   : &RBCalibrations)
-> Result<(), AnalysisError> {
  //if event.status != EventStatus::Perfect {
  //if event.header.broken {
  //  // just return the analysis error, there 
  //  // is probably nothing else we can do?
  //  return Err(AnalysisError::InputBroken);
  //}

  let mut paddles    = HashMap::<u8, TofHit>::new();
  let channels       = event.header.get_channels();
  let channels_c     = channels.clone();
  // first loop over channels - construct pids
  let mut pid        : u8;
  // will become a parameter
  let fit_sinus = true;
  for raw_ch in channels {
    if raw_ch == 8 {
      continue;
    }
    // +1 channel convention
    let ch = raw_ch + 1;
    //let mut TofHit::new();
    let p_end_id = channel_map.get(&ch).unwrap_or(&0);
    if p_end_id < &1000 {
      error!("Invalid paddle end id {} for channel {}!", p_end_id, ch);
      continue;
    }
    if p_end_id > &2000 {
      // it is the B side then
      pid = (p_end_id - 2000) as u8;
    } else {
      pid = (p_end_id - 1000) as u8;
    }
    if !paddles.contains_key(&pid) {
      let mut hit   = TofHit::new();
      hit.paddle_id = pid;
      paddles.insert(pid, hit);
    }
  }
  // second loop over channels. Now we have
  // all the paddles set up in the hashmap
  for raw_ch in channels_c {
    if raw_ch == 8 {
      if fit_sinus {
        // +1 channel convention
        let ch = raw_ch + 1;
        
        let mut ch_voltages : Vec<f32>= vec![0.0; NWORDS];
        let mut ch_times    : Vec<f32>= vec![0.0; NWORDS];
        calibration.voltages(ch.into(),
                             event.header.stop_cell as usize,
                             &event.adc[8],
                             &mut ch_voltages);
        warn!("We have to rework the spike cleaning!");
        //match RBCalibrations::spike_cleaning(&mut ch_voltages,
        //                                     event.header.stop_cell) {
        //  Err(err) => {
        //    error!("Spike cleaning failed! {err}");
        //  }
        //  Ok(_)    => ()
        //}
        calibration.nanoseconds(ch.into(),
                                event.header.stop_cell as usize,
                                &mut ch_times);
        let fit_result = fit_sine_function(ch_times, ch_voltages);
        //println!("FIT RESULT = {:?}", fit_result);
        event.header.set_sine_fit(fit_result);
        continue;
      } else {
        continue;
      }
    }
    // +1 channel convention
    let ch = raw_ch + 1;
    // FIXME - copy/paste from above, wrap in a 
    // function
    let p_end_id  = channel_map.get(&ch).unwrap_or(&0);
    let mut is_a_side = false; 
    if p_end_id < &1000 {
      error!("Invalid paddle end id: {}!" ,p_end_id);
      continue;
    }
    if p_end_id > &2000 {
      // it is the B side then
      pid = (p_end_id - 2000) as u8;
    } else {
      pid = (p_end_id - &1000) as u8;
      is_a_side = true;
    }
    // allocate memory for the calbration results
    let mut ch_voltages : Vec<f32>= vec![0.0; NWORDS];
    let mut ch_times    : Vec<f32>= vec![0.0; NWORDS];
    calibration.voltages(ch.into(),
                         event.header.stop_cell as usize,
                         &event.adc[ch as usize],
                         &mut ch_voltages);
    warn!("We have to rework the spike cleaning!");
    //match RBCalibrations::spike_cleaning(&mut ch_voltages,
    //                                     event.header.stop_cell) {
    //  Err(err) => {
    //    error!("Spike cleaning failed! {err}");
    //  }
    //  Ok(_)    => ()
    //}
    calibration.nanoseconds(ch.into(),
                            event.header.stop_cell as usize,
                            &mut ch_times);
    let (ped, ped_err) = calculate_pedestal(&ch_voltages,
                                            10.0, 10, 50);
    debug!("Got pedestal of {} +- {}", ped, ped_err);
    for n in 0..ch_voltages.len() {
      ch_voltages[n] -= ped;
    }
    let mut charge : f32 = 0.0;
    warn!("Check impedance value! Just using 50 [Ohm]");
    match integrate(&ch_voltages,
                    &ch_times,
                    270.0, 70.0, 50.0) {
      Err(err) => {
        error!("Integration failed! Err {err}");
      }
      Ok(chrg)   => {
        charge = chrg;
      }
    }
    //let peaks : Vec::<(usize, usize)>;
    let mut cfd_times = Vec::<f32>::new();
    // We actually might have multiple peaks 
    // here
    // FIXME 
    match find_peaks(&ch_voltages ,
                     &ch_times    ,
                     270.0, 
                     70.0 ,
                     3    ,
                     10.0 ,
                     5      ) {
      Err(err) => {
        error!("Unable to find peaks for ch {ch}! Ignoring this channel!");
        error!("We won't be able to calculate timing information for this channel! Err {err}");
      },
      Ok(peaks)  => {
        //peaks = pks;
        for pk in peaks.iter() {
          match cfd_simple(&ch_voltages,
                           &ch_times,
                           0.2,pk.0, pk.1) {
            Err(err) => {
              error!("Unable to calculate cfd for peak {} {}! Err {}", pk.0, pk.1, err);
            }
            Ok(cfd) => {
              cfd_times.push(cfd);
            }
          }
        }
      }// end OK
    } // end match find_peaks 
    let this_hit      = paddles.get_mut(&pid).unwrap();
    let mut this_time = 0.0f32;
    if cfd_times.len() > 0 {
      this_time = cfd_times[0];
    }
    
    if is_a_side {
      this_hit.set_time_a(this_time);
      this_hit.set_charge_a(charge);
    } else {
      this_hit.set_time_b(this_time);
      this_hit.set_charge_b(charge);
    }
    // Technically, we can have more than one peak. 
    // We need to adjust the integration window to 
    // the peak min/max and then create Peak instances
    // and sort them into TofHits


    // FIXME - do more analysis here!
  } // end loop over channels
  let result = paddles.into_values().collect();
  event.hits = result;
  Ok(())
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum ReadoutBoardError {
  NoConnectionInfo,
  NoResponse,
}

impl fmt::Display for ReadoutBoardError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this ReadoutBoardError"));
    write!(f, "<ReadoutBoardError: {}>", r)
  }
}

impl Error for ReadoutBoardError {
}



/// A generic representation of a LocalTriggerBoard
///
/// This is important to make the mapping between 
/// trigger information and readoutboard.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
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
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this LTB"));
    write!(f, "<LTB: {}>", r)
  }
}

/// A generic representation of a Readout board
///
///
///
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
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
    let r = serde_json::to_string(self).unwrap_or(
      String::from("Error: cannot unwrap this ReadoutBoard"));
    write!(f, "<ReadoutBoard: {}>", r)
  }
}

impl Default for ReadoutBoard {
  fn default() -> ReadoutBoard {
    ReadoutBoard::new()
  }
}

/// This will load the map as in the file. Channels go from 1-8
pub fn get_rb_ch_pid_map(map_file : PathBuf) -> RBChannelPaddleEndIDMap {
  let mut mapping = RBChannelPaddleEndIDMap::new();
  let json_content : String;
  match read_to_string(&map_file) {
    Ok(_json_content) => {
      json_content = _json_content;
    },
    Err(err) => { 
      error!("Unable to parse json file {}. Error {err}", map_file.display());
      return mapping;
    }      
  }
  let json : Value;
  match serde_json::from_str(&json_content) {
    Ok(_json) => {
      json = _json;
    },
    Err(err) => { 
      error!("Unable to parse json file {}. Error {err}", map_file.display());
      return mapping;
    }
  }
  for ch in 0..8 {
    let tmp_val = &json[(ch +1).to_string()];
    let val = tmp_val.to_string().parse::<u16>().unwrap_or(0);
    mapping.insert(ch as u8 + 1, val);
  }
  mapping
}

pub fn get_ltb_dsi_j_ch_mapping(mapping_file : PathBuf) -> DsiLtbRBMapping {
  let mut mapping = HashMap::<u8,HashMap::<u8,HashMap::<u8,(u8,u8)>>>::new();
  for dsi in 1..6 {
    mapping.insert(dsi, HashMap::<u8,HashMap::<u8, (u8, u8)>>::new());
    for j in 1..6 {
      mapping.get_mut(&dsi).unwrap().insert(j, HashMap::<u8,(u8, u8)>::new());
      for ch in 1..17 {
        mapping.get_mut(&dsi).unwrap().get_mut(&j).unwrap().insert(ch, (0,0));
      }
    }
  }
  let json_content : String;
  match read_to_string(&mapping_file) {
    Ok(_json_content) => {
      json_content = _json_content;
    },
    Err(err) => { 
      error!("Unable to parse json file {}. Error {err}", mapping_file.display());
      return mapping;
    }      
  }
  let json : Value;
  match serde_json::from_str(&json_content) {
    Ok(_json) => {
      json = _json;
    },
    Err(err) => { 
      error!("Unable to parse json file {}. Error {err}", mapping_file.display());
      return mapping;
    }
  }
  for dsi in 1..6 { 
    for j in 1..6 {
      for ch in 1..17 {
        let val = mapping.get_mut(&dsi).unwrap().get_mut(&j).unwrap().get_mut(&ch).unwrap();
        //println!("Checking {} {} {}", dsi, j, ch);
        let tmp_val = &json[dsi.to_string()][j.to_string()][ch.to_string()];
        *val = (tmp_val[0].to_string().parse::<u8>().unwrap_or(0), tmp_val[1].to_string().parse::<u8>().unwrap_or(0));
      }
    }
  }
  debug!("Mapping {:?}", mapping);
  mapping
}

/// Convert an int value to the board ID string.
pub fn to_board_id_string(rb_id: u32) -> String {
  String::from("RB") + &format!("{:02}", rb_id)
}

#[test]
fn test_display() {
  let rb = ReadoutBoard::default();
  println!("Readout board {}", rb);
  assert_eq!(1,1);
}


