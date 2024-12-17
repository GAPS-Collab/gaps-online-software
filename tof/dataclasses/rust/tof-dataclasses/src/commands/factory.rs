//! The actual command list - one function per ocmmand
//!
//! These are factory functions and each of them will 
//! return a single command

use std::collections::HashMap;

use crate::commands::{
  TofCommandV2,
  TofCommandCode
};

use crate::commands::config::{
  TriggerConfig,
};

use crate::serialization::Serialization;

/// A hardwired map of RB -> RAT
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
pub fn shutdown_rb(rb : u8) -> Option<TofCommandV2> {
  let code = TofCommandCode::ShutdownRB;
  let mut cmd  = TofCommandV2::new();
  cmd.command_code = code;
  cmd.payload = vec![rb];
  Some(cmd)
}

/// Send the 'sudo shutdown now' command to all RBs in a RAT
///
/// # Arguments:
///   * rat : The RAT id for the rat the RBs to be 
///           shutdown live in 
pub fn shutdown_rat(rat : u8) -> Option<TofCommandV2> {
  let code = TofCommandCode::ShutdownRAT;
  let mut cmd  = TofCommandV2::new();
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
pub fn shutdown_ratpair(pdu : u8, pduchannel : u8) -> Option<TofCommandV2> {
  let code     = TofCommandCode::ShutdownRATPair;
  let mut cmd  = TofCommandV2::new();
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


/// Send the 'sudo shutdown now command to
/// the TOF main computer ("TOFCPU")
pub fn shutdown_tofcpu() -> Option<TofCommandV2> {
  Some(TofCommandV2 {
    command_code : TofCommandCode::ShutdownCPU,
    payload      : Vec::<u8>::new()
  })
}

/// Restart the liftof-rb clients on the given boards
///
/// # Arguments
///   * rbs: restart the client on the given rb ids, 
///          if empty, restart on all of them
pub fn restart_liftofrb(rbs : &Vec<u8>) -> Option<TofCommandV2> {
  Some(TofCommandV2 {
    command_code : TofCommandCode::RestartLiftofRBClients,
    payload      : rbs.clone()
  })
}

/// Trigger the start of a new data run with 
/// the next active config
pub fn start_run() -> Option<TofCommandV2> {
  Some(TofCommandV2 {
    command_code : TofCommandCode::DataRunStart,
    payload      : Vec::<u8>::new(),
  })
}

/// Stop the current active run and idle
pub fn stop_run() -> Option<TofCommandV2> {
  Some(TofCommandV2 {
    command_code : TofCommandCode::DataRunStop,
    payload      : Vec::<u8>::new(),
  })
}

/// Run a calibration of all RBs
///
/// # Arguments:
///   * send_packetes  : Send the RBCalibration packets
///   * save_events    : Save the events to the RBCalibration
///                      packets
pub fn rb_calibration(send_packets : bool, save_events : bool) -> Option<TofCommandV2> {
  let payload = vec![send_packets as u8, save_events as u8];
  Some(TofCommandV2 {
    command_code : TofCommandCode::RBCalibration,
    payload      : payload,
  })
}

/// Change the MTBSettings in the config file with relevant trigger settings
pub fn change_triggerconfig(cfg : TriggerConfig) -> Option<TofCommandV2> {
  let payload = cfg.to_bytestream();
  Some(TofCommandV2 {
    command_code : TofCommandCode::SetMTConfig,
    payload      : payload,
  })
}



