// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, FromRepr, AsRefStr, EnumIter)]
#[cfg_attr(feature = "pybindings", pyclass(eq, eq_int))]
#[repr(u8)]
pub enum TofCommandCode {
  Unknown                  = 0u8,
  /// en empty command just to check if stuff is online
  Ping                     = 1u8,
  /// command code for getting the monitoring data from the component
  Moni                     = 2u8,
  /// Kill myself
  Kill                     = 4u8, // Shi!
  /// Reload a default (to be defined) config file
  ResetConfigWDefault      = 5u8,
  /// Make the current editable config the active config
  SubmitConfig           = 6u8,
  /// command code to configure the data publisher thread
  SetDataPublisherConfig   = 20u8,
  /// command code for "Set LTB Thresholds"
  SetLTBThresholds         = 21u8,         
  /// command code for "Configure MTB"
  SetMTConfig              = 22u8,     
  /// command code for chaning general run parameters
  SetTofRunConfig          = 23u8,
  /// command code for changing RB parameters
  SetTofRBConfig           = 24u8,
  /// command code for AnalysisEngineConfig
  SetAnalysisEngineConfig  = 27u8,   
  /// command code for "Set preamp bias"
  SetPreampBias            = 28u8,         
  /// Change the settings of the event builder
  SetTOFEventBuilderConfig = 29u8,
  /// command code for "Stop Data taking"
  DataRunStop              = 30u8,  
  /// command code for "Start Data taking"
  DataRunStart             = 31u8,    
  /// command code for "Start validation run"
  StartValidationRun       = 32u8,         
  /// command code for "Get all waveforms"
  GetFullWaveforms         = 41u8,
  /// command code for "Send the whole event cache over the wire"
  UnspoolEventCache        = 44u8,
  /// command code for "Run full calibration"
  RBCalibration            = 53u8, 
  /// command code for restarting systemd
  RestartLiftofRBClients  = 60u8,
  /// command code for putting liftof-cc in listening mode
  Listen                  = 70u8,
  /// command code for putting liftof-cc in staging mode
  Staging                 = 71u8,
  /// lock the cmd dispatcher
  Lock                    = 80u8,
  /// unlock the cmd dispatcher
  Unlock                  = 81u8,
  /// Enable sending of TOF packets
  SendTofEvents           = 90u8,
  /// Diesable sending of TofEventPacket
  NoSendTofEvents         = 91u8,
  /// Enable sending of RBWaveform packets
  SendRBWaveforms         = 92u8,
  /// Disable sending of RBWaveform packets
  NoSendRBWaveforms       = 93u8,
  /// Enable RB Channel Masks
  SetRBChannelMask        = 99u8,
  /// Shutdown RB - send shutdown now to RB
  ShutdownRB              = 100u8,
  /// Change the config file for the next run
  ChangeNextRunConfig     = 101u8,
  /// Shutdown RAT - send shutdown command to 2RBs in the same RAT
  ShutdownRAT             = 102u8,
  /// Shutdown a pair of RATs (as always two of them are hooked up to the 
  /// same PDU channel)
  ShutdownRATPair         = 103u8,
  /// Shutdown the TOF CPU
  ShutdownCPU             = 104u8,
  /// Upload a new config file
  UploadConfig            = 105u8,
}

expand_and_test_enum!(TofCommandCode, test_tofcommandcode_repr);

/// A general command class with an arbitrary payload
///
/// Since the commands should in general be small
/// the maixmal payload size is limited to 256 bytes
///
/// All commands will get broadcasted and the 
/// receiver has to figure out if they have 
/// to rect to that command
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature="pybindings", pyclass)]
pub struct TofCommand {
  pub command_code : TofCommandCode,
  pub payload      : Vec<u8>,
}

impl TofCommand {
  // BFSW command header 144, 235, 86, 248, 70, 41, 7, 15,
  pub fn new() -> Self {
    Self {
      command_code : TofCommandCode::Unknown,
      payload      : Vec::<u8>::new(),
    }
  }

  //pub fn from_config(cfg_file : String) -> Self {
  //  let mut cmd = TofCommand::new();
  //  cmd.command_code = TofCommandCode::UploadConfig:

  //}

  /// In case the command is supposed to change the next run configuration
  /// we can create it with this function
  ///
  /// # Arguments
  ///
  ///   * key_value :  a list of keys and a single value (last item of the 
  ///                  list
  pub fn forge_changerunconfig(key_value : &Vec<String>) -> Self {
    let mut cmd = TofCommand::new();
    cmd.command_code = TofCommandCode::ChangeNextRunConfig;
    if key_value.len() == 0 {
      error!("Empty command!");
      return cmd;
    }
    let mut payload_string = String::from("");
    for k in 0..key_value.len() - 1 {
      payload_string += &format!("{}::", key_value[k]);
    }
    payload_string += key_value.last().unwrap();
    let mut payload = Vec::<u8>::new();
    payload.extend_from_slice(payload_string.as_bytes());
    cmd.payload = payload;
    cmd
  }

  /// After the command has been unpackt, reconstruct the 
  /// key/value string
  pub fn extract_changerunconfig(&self) -> Option<Vec<String>> {
    if self.command_code != TofCommandCode::ChangeNextRunConfig {
      error!("Unable to extract configuration file changes from {}", self);
      return None;
    }
    let mut liftof_config = Vec::<String>::new();
    match String::from_utf8(self.payload.clone()) {
      Err(err) => {
        error!("Unable to extract the String payload! {err}");
      }
      Ok(concat_string) => {
        let foo = concat_string.split("::").collect::<Vec<&str>>().into_iter();
        for k in foo {
          liftof_config.push(String::from(k));
        }
      }
    }
    Some(liftof_config)
  }
}

impl TofPackable for TofCommand {
  const TOF_PACKET_TYPE : TofPacketType = TofPacketType::TofCommandV2;
}

impl Serialization for TofCommand {
  
  const HEAD : u16 = 0xAAAA;
  const TAIL : u16 = 0x5555;

  fn from_bytestream(stream    : &Vec<u8>, 
                     pos       : &mut usize) 
    -> Result<Self, SerializationError>{
    let mut command = TofCommand::new();
    if parse_u16(stream, pos) != Self::HEAD {
      error!("The given position {} does not point to a valid header signature of {}", pos, Self::HEAD);
      return Err(SerializationError::HeadInvalid {});
    }
    command.command_code = TofCommandCode::from(parse_u8(stream, pos));
    let payload_size     = parse_u8(stream, pos);
    let payload          = stream[*pos..*pos + payload_size as usize].to_vec();
    command.payload      = payload;
    *pos += payload_size as usize;
    let tail = parse_u16(stream, pos);
    if tail != Self::TAIL {
      error!("After parsing the event, we found an invalid tail signature {}", tail);
      return Err(SerializationError::TailInvalid);
    }
    Ok(command)
  }

  fn to_bytestream(&self) -> Vec<u8> {
    let mut stream = Vec::<u8>::with_capacity(9);
    stream.extend_from_slice(&Self::HEAD.to_le_bytes());
    stream.push(self.command_code as u8);
    stream.push(self.payload.len() as u8);
    stream.extend_from_slice(self.payload.as_slice());
    stream.extend_from_slice(&Self::TAIL.to_le_bytes());
    stream
  }
}

impl Default for TofCommand {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(feature = "random")]
impl FromRandom for TofCommand {
  fn from_random() -> Self {
    let mut rng      = rand::rng();
    let command_code = TofCommandCode::from_random();
    let payload_size = rng.random::<u8>();
    let mut payload  = Vec::<u8>::with_capacity(payload_size as usize);
    for _ in 0..payload_size {
      payload.push(rng.random::<u8>());
    }
    Self {
      command_code : command_code,
      payload      : payload
    }
  }
}

impl fmt::Display for TofCommand {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    //let cc = RBCommand::command_code_to_string(self.command_code);
    let mut repr = String::from("<TofCommand");
    repr += &(format!("\n  cmd code : {}", self.command_code)); 
    match self.command_code {
      TofCommandCode::ShutdownRB 
      | TofCommandCode::ShutdownRAT 
      | TofCommandCode::ShutdownRATPair => {
        repr += &(format!("\n Sending shutdown command to RBs {:?}>", self.payload));
      }
      _ => {
        repr += ">";
      }
    }
    write!(f, "{}", repr)
  }
}

#[cfg(feature="pybindings")]
#[pymethods]
impl TofCommand {

  #[getter]
  fn get_command_code(&mut self) -> TofCommandCode {
    self.command_code
  }
  
  #[setter]
  fn set_command_code(&mut self, command_code : TofCommandCode) {
    self.command_code = command_code;
  }

  /// Pack myself nicely in a TofPacket and 
  /// serialize myself
  ///
  /// Can be used to interface with BFSW/GSE
  /// systems
  fn wrap_n_pack(&self) -> Vec<u8> {
    self.pack().to_bytestream()
  }

  /// An explicit getter for the 
  /// command code, to interface 
  /// with BFSW/GSE systems
  fn get_cc_u8(&self) -> u8 {
    self.command_code as u8
  }

  #[pyo3(name="to_bytestream")]
  fn to_bytestream_py(&self) -> Vec<u8> {
    self.to_bytestream()
  }  
}

#[cfg(feature="pybindings")]
pythonize_packable!(TofCommand);

/// A hardwired map of RB -> RAT
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn get_rbratmap_hardcoded() ->  HashMap<u8,u8> {
  warn!("Using hardcoded rbratmap!");
  let mapping = HashMap::<u8,u8>::from(
      [(1, 10), 
       (2, 15), 
       (3,  1),  
       (4, 15), 
       (5, 20), 
       (6, 19), 
       (7, 17), 
       (8,  9),
       (9, 13),  
       (11,10),
       (13, 4), 
       (14, 2), 
       (15, 1), 
       (16, 8), 
       (17,17),
       (18,13),
       (19, 7), 
       (20, 7), 
       (21, 5), 
       (22,11),
       (23, 5), 
       (24, 6), 
       (25, 8), 
       (26,11),
       (27, 6), 
       (28,20),
       (29, 3), 
       (30, 9), 
       (31, 3), 
       (32, 2), 
       (33,18),
       (34,18),
       (35, 4), 
       (36,19),
       (39,12),
       (40,12),
       (41,14),
       (42,14),
       (44,16),
       (46,16)]);
  mapping
}

/// A hardwired map of RAT -> (RB1, RB2)
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn get_ratrbmap_hardcoded() ->  HashMap<u8,(u8,u8)> {
  warn!("Using hardcoded ratrb map!");
  let mapping = HashMap::<u8,(u8,u8)>::from(
      [(1, (3,15)), 
       (2, (32,14)), 
       (3, (31,29)),  
       (4, (35,13)), 
       (5, (23,21)), 
       (6, (27,24)), 
       (7, (20,19)), 
       (8, (16,25)),  
       (9, (8,30)),
       (10,(1,11)), 
       (11,(26,22)), 
       (12,(39,40)),
       (13,(9,18)), 
       (14,(41,42)),
       (15,(2,4)),
       (16,(46,44)), 
       (17,(7,17)), 
       (18,(33,34)), 
       (19,(36,6)), 
       (20,(28,5))]); 
  mapping
}

/// A hardwired map of PDU #id PDUCHANNEL #id to (RAT,RAT)
///
/// Can be used to synchronize powering down proces for 
/// RATs
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn get_ratpdumap_hardcoded() ->  HashMap<u8,HashMap::<u8, (u8,u8)>> {
  warn!("Using hardcoded rat-pdu map!");
  let mut mapping = HashMap::<u8,HashMap::<u8,(u8,u8)>>::new();
  let mut ch_map = HashMap::<u8, (u8,u8)>::from([(3, (15,16)), (7, (8,9))]);
  mapping.insert(0u8, ch_map.clone());
  ch_map = HashMap::<u8, (u8, u8)>::from([(2, (2,17)), (3, (4,5)), (5, (13,14))]);
  mapping.insert(1u8, ch_map.clone());
  ch_map = HashMap::<u8, (u8, u8)>::from([(3, (12,20)), (4, (10,11)), (5, (8,9))]);
  mapping.insert(2u8, ch_map.clone());
  ch_map = HashMap::<u8, (u8, u8)>::from([(2, (6,7)), (3, (1,3))]);
  mapping.insert(3u8, ch_map.clone());
  mapping
}

/// Send the 'sudo shutdown now' command to a single RB
///
/// # Arguements:
///   * rb :  The RB id of the RB to be shutdown 
///           (NOT RAT)
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn shutdown_rb(rb : u8) -> Option<TofCommand> {
  let code = TofCommandCode::ShutdownRB;
  let mut cmd  = TofCommand::new();
  cmd.command_code = code;
  cmd.payload = vec![rb];
  Some(cmd)
}

/// Send the 'sudo shutdown now' command to all RBs in a RAT
///
/// # Arguments:
///   * rat : The RAT id for the rat the RBs to be 
///           shutdown live in 
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn shutdown_rat(rat : u8) -> Option<TofCommand> {
  let code = TofCommandCode::ShutdownRAT;
  let mut cmd  = TofCommand::new();
  cmd.command_code = code;
  cmd.payload = Vec::<u8>::new();
  let ratmap = get_ratrbmap_hardcoded();
  match ratmap.get(&rat) {
    None => {
      error!("Don't know RBs in RAT {}", rat);
      return None
    }
    Some(pair) => {
      cmd.payload.push(pair.0);
      cmd.payload.push(pair.1);
    }
  }
  Some(cmd)
}

/// Send the 'sudo shutdown now' command to all RBs in a RAT
/// 
/// This will prepare the shutdown command for the RBs in the 
/// RATs which are connected to a specific pdu channel
///
/// # Arguments:
///   * pdu        : PDU ID (0-3)
///   * pduchannel : PDU Channel (0-7)
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn shutdown_ratpair(pdu : u8, pduchannel : u8) -> Option<TofCommand> {
  let code     = TofCommandCode::ShutdownRATPair;
  let mut cmd  = TofCommand::new();
  cmd.command_code = code;
  cmd.payload = Vec::<u8>::new();
  let ratmap    = get_ratrbmap_hardcoded();
  let ratpdumap = get_ratpdumap_hardcoded();
  match ratpdumap.get(&pdu) {
    None => {
      error!("Don't know that there is a RAT connected to PDU {}!", pdu);
      return None;
    }
    Some(select_pdu) => {
      match select_pdu.get(&pduchannel) {
        None => {
          error!("Don't know that there is a RAT connected to PDU {}, channel {}!", pdu, pduchannel);
          return None;
        }
        Some(rats) => {
          match ratmap.get(&rats.0) {
            Some(rbs) => {
              cmd.payload.push(rbs.0);
              cmd.payload.push(rbs.1);
            }
            None => {
              error!("RAT mapping incorrect!");
              return None;
            }
          }
          match ratmap.get(&rats.1) {
            Some(rbs) => {
              cmd.payload.push(rbs.0);
              cmd.payload.push(rbs.1);
            },
            None => {
              error!("RAT mapping incorrect!");
              return None;
            }
          }
        }
      }
    }
  }
  Some(cmd)
}

/// Send the 'sudo shutdown now' command to
/// ALL RBs
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn shutdown_all_rbs() -> Option<TofCommand> {
  Some(TofCommand {
    command_code : TofCommandCode::ShutdownRB,
    payload      : Vec::<u8>::new()
  })
}

/// Send the 'sudo shutdown now command to
/// the TOF main computer ("TOFCPU")
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn shutdown_tofcpu() -> Option<TofCommand> {
  Some(TofCommand {
    command_code : TofCommandCode::ShutdownCPU,
    payload      : Vec::<u8>::new()
  })
}

/// Restart the liftof-rb clients on the given boards
///
/// # Arguments
///   * rbs: restart the client on the given rb ids, 
///          if empty, restart on all of them
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn restart_liftofrb(rbs : Vec<u8>) -> Option<TofCommand> {
  // We don't use & for Vec here, since we need to give it to payload 
  // so there would be a .clone() anyway and so python can understand
  // the function argument
  Some(TofCommand {
    command_code : TofCommandCode::RestartLiftofRBClients,
    payload      : rbs
  })
}

/// Trigger the start of a new data run with 
/// the next active config
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn start_run() -> Option<TofCommand> {
  Some(TofCommand {
    command_code : TofCommandCode::DataRunStart,
    payload      : Vec::<u8>::new(),
  })
}

/// Stop the current active run and idle
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn stop_run() -> Option<TofCommand> {
  Some(TofCommand {
    command_code : TofCommandCode::DataRunStop,
    payload      : Vec::<u8>::new(),
  })
}

/// Run a calibration of all RBs
///
/// # Arguments:
///   * pre_run_calibration : Run the RBCalibration routine before 
///                           every run start
///   * send_packetes       : Send the RBCalibration packets
///   * save_events         : Save the events to the RBCalibration
///                           packets
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn calibrate_rbs(pre_run_calibration : bool, send_packets : bool, save_events : bool) -> Option<TofCommand> {
  let payload = vec![pre_run_calibration as u8, send_packets as u8, save_events as u8];
  Some(TofCommand {
    command_code : TofCommandCode::RBCalibration,
    payload      : payload,
  })
}

/// Change the MTBSettings in the config file with relevant trigger settings
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn change_triggerconfig(cfg : &TriggerConfig) -> Option<TofCommand> {
  let payload = cfg.to_bytestream();
  Some(TofCommand {
    command_code : TofCommandCode::SetMTConfig,
    payload      : payload,
  })
}

/// Change the EventBuilderSettings in the config file
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn change_tofeventbuilderconfig(cfg : &TOFEventBuilderConfig) -> Option<TofCommand> {
  let payload = cfg.to_bytestream();
  Some(TofCommand {
    command_code : TofCommandCode::SetTOFEventBuilderConfig,
    payload      : payload,
  })
}

/// Change the data publisher config part of the config file
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn change_datapublisherconfig(cfg : &DataPublisherConfig) -> Option<TofCommand> {
  let payload = cfg.to_bytestream();
  Some(TofCommand {
    command_code : TofCommandCode::SetDataPublisherConfig,
    payload      : payload,
  })
}

/// Change the data publisher config part of the config file
#[cfg_attr(feature="pybindings", pyfunction)]
pub fn change_tofrunconfig(cfg : &TofRunConfig) -> Option<TofCommand> {
  let payload = cfg.to_bytestream();
  Some(TofCommand {
    command_code : TofCommandCode::SetTofRunConfig,
    payload      : payload,
  })
}

#[cfg_attr(feature="pybindings", pyfunction)]
pub fn change_tofrbconfig(cfg : &TofRBConfig) -> Option<TofCommand> {
  let payload = cfg.to_bytestream();
  Some(TofCommand {
    command_code : TofCommandCode::SetTofRBConfig,
    payload      : payload,
  })
}

///// Send the 'sudo shutdown now' command to a single RB
/////
///// # Arguements:
/////   * rb :  The RB id of the RB to be shutdown 
/////           (NOT RAT)
//#[pyfunction]
//#[pyo3(name="shutdown_rb")]
//pub fn py_shutdown_rb(rb : u8) -> PyResult<TofCommand> {
//  let cmd = shutdown_rb(rb).unwrap();
//  Ok(TofCommand { 
//    command : cmd
//  })
//}
//
//
///// Send the 'sudo shutdown now' command to
///// ALL RBs
//#[pyfunction]
//#[pyo3(name="shutdown_all_rbs")]
//pub fn py_shutdown_all_rbs() -> PyResult<TofCommand> {
//  let cmd = shutdown_all_rbs().unwrap();
//  let pycmd = TofCommand { 
//    command : cmd
//  };
//  return Ok(pycmd);
//}
//
///// Send the 'sudo shutdown now' command to all RBs in a RAT
/////
///// # Arguments:
/////   * rat : The RAT id for the rat the RBs to be 
/////           shutdown live in 
//#[pyfunction]
//#[pyo3(name="shutdown_rat")]
//pub fn py_shutdown_rat(rat : u8) -> PyResult<TofCommand> {
//  match shutdown_rat(rat) {
//    None => {
//      return Err(PyValueError::new_err(format!("There might not be a RAT{}!", rat)));
//    }
//    Some(cmd) => {
//      let pycmd = TofCommand { 
//       command : cmd
//      };
//      return Ok(pycmd);
//    }
//  }
//}
//
///// Send the 'sudo shutdown now' command to all RBs 
///// in the 2 RATs connected to a certain PDU channel
///// 
///// This will prepare the shutdown command for the RBs in the 
///// RATs which are connected to a specific pdu channel
/////
///// # Arguments:
/////   * pdu        : PDU ID (0-3)
/////   * pduchannel : PDU Channel (0-7)
//#[pyfunction]
//#[pyo3(name="shutdown_ratpair")]
//pub fn py_shutdown_ratpair(pdu : u8, pduchannel : u8) -> PyResult<TofCommand> {
//  match shutdown_ratpair(pdu, pduchannel) {
//    None => {
//      return Err(PyValueError::new_err(format!("There might be an issue with the pdu mapping. Can nto find RATs at PDU {} channel {}!", pdu, pduchannel)));
//    }
//    Some(cmd) => {
//      let pycmd = TofCommand { 
//       command : cmd
//      };
//      return Ok(pycmd);
//    }
//  }
//}
//
///// Send the 'sudo shutdown now command to
///// the TOF main computer ("TOFCPU")
//#[pyfunction]
//#[pyo3(name="shutdown_cpu")]
//pub fn py_shutdown_tofcpu() -> PyResult<TofCommand> {
//  match shutdown_tofcpu() {
//    None => {
//      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
//    }
//    Some(cmd) => {
//      let pycmd = TofCommand { 
//       command : cmd
//      };
//      return Ok(pycmd);
//    }
//  }
//}
//
//
///// Restart the liftof-rb clients on the given boards
/////
///// # Arguments
/////   * rbs: restart the client on the given rb ids, 
/////          if empty, restart on all of them
//#[pyfunction]
//#[pyo3(name="restart_liftofrb")]
//pub fn py_restart_liftofrb(rbs : Vec<u8>) -> PyResult<TofCommand> {
//  match restart_liftofrb(&rbs) {
//    None => {
//      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
//    }
//    Some(cmd) => {
//      let pycmd = TofCommand { 
//       command : cmd
//      };
//      return Ok(pycmd);
//    }
//  }
//}
//
///// Trigger the start of a new data run with 
///// the next active config
//#[pyfunction]
//#[pyo3(name="start_run")]
//pub fn py_start_run() -> PyResult<TofCommand> {
//  match start_run() {
//    None => {
//      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
//    }
//    Some(cmd) => {
//      let pycmd = TofCommand { 
//       command : cmd
//      };
//      return Ok(pycmd);
//    }
//  }
//}
//
///// Stop the current active run and idle
//#[pyfunction]
//#[pyo3(name="stop_run")]
//pub fn py_stop_run() -> PyResult<TofCommand> {
//  match stop_run() {
//    None => {
//      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
//    }
//    Some(cmd) => {
//      let pycmd = TofCommand { 
//       command : cmd
//      };
//      return Ok(pycmd);
//    }
//  }
//}
//
///// Run a calibration of all RBs
/////
///// # Arguments:
/////   * pre_run_calibration : Run the RBCalibration routine before 
/////                           every run start
/////   * send_packetes       : Send the RBCalibration packets
/////   * save_events         : Save the events to the RBCalibration
/////                           packets
//#[pyfunction]
//#[pyo3(name="rb_calibration")]
//pub fn py_rb_calibration(pre_run_calibration : bool, send_packets : bool, save_events : bool) -> PyResult<TofCommand> {
//  match rb_calibration(pre_run_calibration,send_packets, save_events) {
//    None => {
//      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
//    }
//    Some(cmd) => {
//      let pycmd = TofCommand { 
//       command : cmd
//      };
//      return Ok(pycmd);
//    }
//  }
//}
//
//
///// Change the MTBSettings in the config file with relevant trigger settings
//#[pyfunction]
//#[pyo3(name="change_triggerconfig")]
//pub fn py_change_triggerconfig(cfg : &PyTriggerConfig) -> PyResult<TofCommand> {
//  match change_triggerconfig(&cfg.config) {
//    None => {
//      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
//    }
//    Some(cmd) => {
//      let pycmd = TofCommand { 
//       command : cmd
//      };
//      return Ok(pycmd);
//    }
//  }
//}
//
//
///// Change the TOFEventBuilderSettings in the config
//#[pyfunction]
//#[pyo3(name="change_tofeventbuilderconfig")]
//pub fn py_change_tofeventbuilderconfig(cfg : &PyTOFEventBuilderConfig) -> PyResult<TofCommand> {
//  match change_tofeventbuilderconfig(&cfg.config) {
//    None => {
//      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
//    }
//    Some(cmd) => {
//      let pycmd = TofCommand { 
//       command : cmd
//      };
//      return Ok(pycmd);
//    }
//  }
//}
//
///// Change the data publisher config part of the config file
//#[pyfunction]
//#[pyo3(name="change_datapublisherconfig")]
//pub fn py_change_datapublisherconfig(cfg : &PyDataPublisherConfig) -> PyResult<TofCommand> {
//  match change_datapublisherconfig(&cfg.config) {
//    None => {
//      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
//    }
//    Some(cmd) => {
//      let pycmd = TofCommand { 
//       command : cmd
//      };
//      return Ok(pycmd);
//    }
//  }
//}
//
///// Change the run config part of the config file
//#[pyfunction]
//#[pyo3(name="change_tofrunconfig")]
//pub fn py_change_tofrunconfig(cfg : &PyTofRunConfig) -> PyResult<TofCommand> {
//  match change_tofrunconfig(&cfg.config) {
//    None => {
//      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
//    }
//    Some(cmd) => {
//      let pycmd = TofCommand { 
//       command : cmd
//      };
//      return Ok(pycmd);
//    }
//  }
//}
//
///// Change the RB config part of the config file
//#[pyfunction]
//#[pyo3(name="change_tofrbconfig")]
//pub fn py_change_tofrbconfig(cfg : &PyTofRBConfig) -> PyResult<TofCommand> {
//  match change_tofrbconfig(&cfg.config) {
//    None => {
//      return Err(PyValueError::new_err(format!("You encounterd a dragon \u{1f409}! We don't know what's going on either.")));
//    }
//    Some(cmd) => {
//      let pycmd = TofCommand { 
//       command : cmd
//      };
//      return Ok(pycmd);
//    }
//  }
//}

