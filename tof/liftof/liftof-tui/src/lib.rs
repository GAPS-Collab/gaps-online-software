#[macro_use] extern crate log;

pub mod menu;
pub mod colors;
pub mod widgets;
pub mod tabs;
pub mod layout;

use std::sync::Mutex;
use std::sync::Arc;

use std::collections::HashMap;
use std::collections::VecDeque;

pub use crate::tabs::*;
pub use crate::layout::*;

#[cfg(feature = "telemetry")]
use telemetry_dataclasses::packets::TelemetryPacketType;

use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::packets::PacketType;
use tof_dataclasses::events::MasterTriggerEvent;
use tof_dataclasses::events::TofEventSummary;
use tof_dataclasses::events::TofHit;
use tof_dataclasses::serialization::Packable;
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::io::TofPacketWriter;

use crossbeam_channel::{
  Sender,
  Receiver
};

#[cfg(feature = "telemetry")]
/// A map which keeps track of the types of telemetry packets 
/// received
pub fn telly_packet_counter(pack_map : &mut HashMap<&str, usize>, packet_type : &TelemetryPacketType) {
  let pack_key : &str;
  match packet_type {
    TelemetryPacketType::Unknown            => pack_key = "Unknown",
    TelemetryPacketType::CardHKP            => pack_key = "CardHKP",
    TelemetryPacketType::CoolingHK          => pack_key = "CoolingHKP",
    TelemetryPacketType::PDUHK              => pack_key = "PDUHK",
    TelemetryPacketType::Tracker            => pack_key = "Tracker",
    TelemetryPacketType::TrackerDAQCntr     => pack_key = "TrakcerDAQCntr",
    TelemetryPacketType::GPS                => pack_key = "GPS",
    TelemetryPacketType::TrkTempLeak        => pack_key = "TrkTempLeak",
    TelemetryPacketType::BoringEvent        => pack_key = "BoringEvent",
    TelemetryPacketType::RBWaveform         => pack_key = "RBWaveform",
    TelemetryPacketType::AnyTofHK           => pack_key = "AnyTofHK",
    TelemetryPacketType::GcuEvtBldSettings  => pack_key = "GcuEvtBldSettings",
    TelemetryPacketType::LabJackHK          => pack_key = "LabJackHK",
    TelemetryPacketType::MagHK              => pack_key = "MagHK",
    TelemetryPacketType::GcuMon             => pack_key = "GcuMon",
    TelemetryPacketType::InterestingEvent   => pack_key = "InterestingEvent",
    TelemetryPacketType::NoGapsTriggerEvent => pack_key = "NoGapsTriggerEvent",
    TelemetryPacketType::NoTofDataEvent     => pack_key = "NoTofDataEvent",
    TelemetryPacketType::Ack                => pack_key = "Ack",     
    TelemetryPacketType::AnyTrackerHK       => pack_key = "AnyTrackerHK",
    TelemetryPacketType::TmP33              => pack_key = "TmP33",
    TelemetryPacketType::TmP34              => pack_key = "TmP34",
    TelemetryPacketType::TmP37              => pack_key = "TmP37",
    TelemetryPacketType::TmP38              => pack_key = "TmP38",
    TelemetryPacketType::TmP55              => pack_key = "TmP55",
    TelemetryPacketType::TmP64              => pack_key = "TmP64",
    TelemetryPacketType::TmP96              => pack_key = "TmP96",
    TelemetryPacketType::TmP214             => pack_key = "TmP214",
  //_                              => pack_key = "Unknown",
  }
  if pack_map.get(pack_key).is_some() {
    *pack_map.get_mut(pack_key).unwrap() += 1;
  } else {
    pack_map.insert(pack_key, 0);
  }
}


#[cfg(feature = "telemetry")]
/// Just produce a summary of all the packets we received
fn telly_packet_sorter(packet_type : &TelemetryPacketType,
                       packet_map  : &Arc<Mutex<HashMap<&str, usize>>>) {
  match packet_map.lock() {
    Ok(mut pm) => {
      let pack_key : &str;
      match packet_type {
        TelemetryPacketType::Unknown            => pack_key = "Unknown",
        TelemetryPacketType::CardHKP            => pack_key = "CardHKP",
        TelemetryPacketType::CoolingHK          => pack_key = "CoolingHKP",
        TelemetryPacketType::PDUHK              => pack_key = "PDUHK",
        TelemetryPacketType::Tracker            => pack_key = "Tracker",
        TelemetryPacketType::TrackerDAQCntr     => pack_key = "TrakcerDAQCntr",
        TelemetryPacketType::GPS                => pack_key = "GPS",
        TelemetryPacketType::TrkTempLeak        => pack_key = "TrkTempLeak",
        TelemetryPacketType::BoringEvent        => pack_key = "BoringEvent",
        TelemetryPacketType::RBWaveform         => pack_key = "RBWaveform",
        TelemetryPacketType::AnyTofHK           => pack_key = "AnyTofHK",
        TelemetryPacketType::GcuEvtBldSettings  => pack_key = "GcuEvtBldSettings",
        TelemetryPacketType::LabJackHK          => pack_key = "LabJackHK",
        TelemetryPacketType::MagHK              => pack_key = "MagHK",
        TelemetryPacketType::GcuMon             => pack_key = "GcuMon",
        TelemetryPacketType::InterestingEvent   => pack_key = "InterestingEvent",
        TelemetryPacketType::NoGapsTriggerEvent => pack_key = "NoGapsTriggerEvent",
        TelemetryPacketType::NoTofDataEvent     => pack_key = "NoTofDataEvent",
        TelemetryPacketType::Ack                => pack_key = "Ack",     
        TelemetryPacketType::AnyTrackerHK       => pack_key = "AnyTrackerHK",
        TelemetryPacketType::TmP33              => pack_key = "TmP33",
        TelemetryPacketType::TmP34              => pack_key = "TmP34",
        TelemetryPacketType::TmP37              => pack_key = "TmP37",
        TelemetryPacketType::TmP38              => pack_key = "TmP38",
        TelemetryPacketType::TmP55              => pack_key = "TmP55",
        TelemetryPacketType::TmP64              => pack_key = "TmP64",
        TelemetryPacketType::TmP96              => pack_key = "TmP96",
        TelemetryPacketType::TmP214             => pack_key = "TmP214",
        //_                              => pack_key = "Unknown",
      }
      if pm.get(pack_key).is_some() {
        *pm.get_mut(pack_key).unwrap() += 1;
      } else {
        pm.insert(pack_key, 0);
      }
    }
    Err(err) => {
      error!("Can't lock shared memory! {err}");
    }
  }
}

/// Just produce a summary of all the packets we received
fn packet_sorter(packet_type : &PacketType,
                 packet_map  : &Arc<Mutex<HashMap<&str,usize>>>) {
  match packet_map.lock() {
    Ok(mut pm) => {
      let pack_key : &str;
      match packet_type {
        PacketType::Unknown               => pack_key = "Unknown", 
        PacketType::RBEvent               => pack_key = "RBEvent",
        PacketType::TofEvent              => pack_key = "TofEvent",
        PacketType::RBWaveform            => pack_key = "RBWaveform",
        PacketType::TofEventSummary       => pack_key = "TofEventSummary",
        PacketType::HeartBeatDataSink     => pack_key = "HeartBeatDataSink",    
        PacketType::MasterTrigger         => pack_key = "MasterTrigger",
        PacketType::TriggerConfig         => pack_key = "TriggerConfig",
        PacketType::MTBHeartbeat          => pack_key = "MTBHeartbeat", 
        PacketType::EVTBLDRHeartbeat      => pack_key = "EVTBLDRHeartbeat",
        PacketType::RBChannelMaskConfig   => pack_key = "RBChannelMaskConfig",
        PacketType::TofRBConfig           => pack_key = "TofRBConfig",
        PacketType::AnalysisEngineConfig  => pack_key = "AnalysisEngineConfig",
        PacketType::RBEventHeader         => pack_key = "RBEventHeader",    // needs to go away
        PacketType::TOFEventBuilderConfig => pack_key = "TOFEventBuilderConfig",
        PacketType::DataPublisherConfig   => pack_key = "DataPublisherConfig",
        PacketType::TofRunConfig          => pack_key = "TofRunConfig",
        PacketType::CPUMoniData           => pack_key = "CPUMoniData",
        PacketType::MonitorMtb            => pack_key = "MonitorMtb",
        PacketType::RBMoniData            => pack_key = "RBMoniData",
        PacketType::PBMoniData            => pack_key = "PBMoniData",
        PacketType::LTBMoniData           => pack_key = "LTBMoniData",
        PacketType::PAMoniData            => pack_key = "PAMoniData",
        PacketType::RBEventMemoryView     => pack_key = "RBEventMemoryView", // We'll keep it for now - indicates that the event
        PacketType::RBCalibration         => pack_key = "RBCalibration",
        PacketType::TofCommand            => pack_key = "TofCommand",
        PacketType::TofCommandV2          => pack_key = "TofCommandV2",
        PacketType::TofResponse           => pack_key = "TofResponse",
        PacketType::RBCommand             => pack_key = "RBCommand",
        PacketType::RBPing                => pack_key = "RBPing",
        PacketType::PreampBiasConfig      => pack_key = "PreampBiasConfig",
        PacketType::RunConfig             => pack_key = "RunConfig",
        PacketType::LTBThresholdConfig    => pack_key = "LTBThresholdConfig",
        PacketType::TofDetectorStatus     => pack_key = "TofDetectorStatus",
        PacketType::ConfigBinary          => pack_key = "ConfigBinary",
        PacketType::LiftofRBBinary        => pack_key = "LiftofRBBinary",
        PacketType::LiftofBinaryService   => pack_key = "LiftofBinaryService",
        PacketType::LiftofCCBinary        => pack_key = "LiftofCCBinary",
        PacketType::RBCalibrationFlightV  => pack_key = "RBCalibrationFlightV",
        PacketType::RBCalibrationFlightT  => pack_key = "RBCalibrationFlightT",
        PacketType::BfswAckPacket         => pack_key = "BfswAckPacket",
        PacketType::MultiPacket           => pack_key = "MultiPacket",
      }
      if pm.get(pack_key).is_some() {
        *pm.get_mut(pack_key).unwrap() += 1;
      } else {
        pm.insert(pack_key, 0);
      }
    }
    Err(err) => {
      error!("Can't lock shared memory! {err}");
    }
  }
}

/// Receive packets from an incoming stream
/// and distrubute them to their receivers
/// while taking notes of everything
///
/// This is a Pablo Pubsub kind of persona
/// (see a fantastic talk at RustConf 2023)
pub fn packet_distributor(tp_from_sock : Receiver<TofPacket>,
                          tp_sender_mt : Sender<TofPacket>,
                          tp_sender_rb : Sender<TofPacket>,
                          tp_sender_ev : Sender<TofPacket>,
                          tp_sender_cp : Sender<TofPacket>,
                          tp_sender_tr : Sender<TofPacket>,
                          rbwf_sender  : Sender<TofPacket>,
                          ts_send      : Sender<TofEventSummary>,
                          th_send      : Sender<TofHit>,
                          tp_sender_hb : Sender<TofPacket>,
                          str_list     : Arc<Mutex<VecDeque<String>>>,
                          pck_map      : Arc<Mutex<HashMap<&str, usize>>>,
                          mut writer   : Option<TofPacketWriter>) {
  let mut n_pack = 0usize;
  // per default, we create master trigger packets from TofEventSummary, 
  // except we have "real" mtb packets
  let mut craft_mte_packets = true;

  loop {
    //match data_socket.recv_bytes(0) {
    match tp_from_sock.recv() {
      Err(err) => error!("Can't receive TofPacket! {err}"),
      Ok(tp) => {
        //println!("{:?}", pck_map);
        packet_sorter(&tp.packet_type, &pck_map);
        n_pack += 1;
        //println!("Got TP {}", tp);
        match str_list.lock() {
          Err(err) => error!("Can't lock shared memory! {err}"),
          Ok(mut _list)    => {
            //let prefix  = String::from_utf8(payload[0..4].to_vec()).expect("Can't get prefix!");
            //let message = format!("{}-{} {}", n_pack,prefix, tp.to_string());
            let message = format!("{} : {}", n_pack, tp);
            _list.push_back(message);
          }
        }
        // if captured, write file
        if writer.is_some() {
          writer.as_mut().unwrap().add_tof_packet(&tp);
        }
        match tp.packet_type {
          PacketType::TofResponse => { 
            match tp_sender_tr.send(tp) {
              Err(err) => error!("Can't send TP! {err}"),
              Ok(_)    => (),
            }
          }
          PacketType::MonitorMtb |
          PacketType::MasterTrigger => {
            // apparently, we are getting MasterTriggerEvents, 
            // sow we won't be needing to craft them from 
            // TofEventSummary packets
            if tp.packet_type == PacketType::MasterTrigger {
              craft_mte_packets = false;
            }
            match tp_sender_mt.send(tp) {
              Err(err) => error!("Can't send TP! {err}"),
              Ok(_)    => (),
            }
          },
          PacketType::RBWaveform => {
            match rbwf_sender.send(tp) {
              Err(err) => error!("Can't send TP! {err}"),
              Ok(_)    => (),
            }
          }
          PacketType::TofEventSummary => {
            match TofEventSummary::from_tofpacket(&tp) {
              Err(err) => {
                error!("Unable to unpack TofEventSummary! {err}");
              }
              Ok(ts) => {
                if craft_mte_packets {
                  let mte    = MasterTriggerEvent::from(&ts);
                  let mte_tp = mte.pack();
                  //error!("We are sending the following tp {}", mte_tp);
                  match tp_sender_mt.send(mte_tp) {
                    Err(err) => error!("Can't send MTE TP! {err}"),
                    Ok(_)    => ()
                  }
                }
                for h in &ts.hits {
                  match th_send.send(*h) {
                    Err(err) => error!("Can't send TP! {err}"),
                    Ok(_)    => (),
                  }
                }
                match ts_send.send(ts) {
                  Err(err) => error!("Can't send TP! {err}"),
                  Ok(_)    => (),
                }
              }
            }
          }
          PacketType::TofEvent => {
            // since the tof event contains MTEs, we don't need
            // to craft them
            craft_mte_packets = false;
            match tp_sender_ev.send(tp) {
              Err(err) => error!("Can't send TP! {err}"),
              Ok(_)    => (),
            }
            // Disasemble the packets
            //match TofEvent::from_bytestream(tp.payload, &mut 0) {
            //  Err(err) => {
            //    error!("Can't decode TofEvent");
            //  },
            //  Ok(ev) => {
            //    //for rbev in ev.rb_events {
            //    //  let 
            //    //  match tp_sender_rb.send
            //    //}
            //  }
            //}
          }
          PacketType::RBEvent |
          PacketType::RBEventMemoryView | 
          PacketType::LTBMoniData |
          PacketType::PAMoniData  |
          PacketType::PBMoniData  |
          PacketType::RBMoniData => {
            match tp_sender_rb.send(tp) {
              Err(err) => error!("Can't send TP! {err}"),
              Ok(_)    => (),
            }
          }
          PacketType::CPUMoniData => {
            match tp_sender_cp.send(tp) {
              Err(err) => error!("Can't send TP! {err}"),
              Ok(_)    => (),
            }
          }
          PacketType::HeartBeatDataSink |
          PacketType::EVTBLDRHeartbeat  | 
          PacketType::MTBHeartbeat      => {
            match tp_sender_hb.send(tp) {
              Err(err) => error!("Can't send TP! {err}"),
              Ok(_)    => {
              },
            }
          }
          _ => () 
        }
      }
    } 
  }
}

/// ZMQ socket wrapper for the zmq socket which is 
/// supposed to receive data from the TOF system.
pub fn socket_wrap_tofstream(address   : &str,
                             tp_sender : Sender<TofPacket>) {
  let ctx = zmq::Context::new();
  // FIXME - don't hardcode this IP
  let socket = ctx.socket(zmq::SUB).expect("Unable to create 0MQ SUB socket!");
  socket.connect(address).expect("Unable to connect to data (PUB) socket {adress}");
  socket.set_subscribe(b"").expect("Can't subscribe to any message on 0MQ socket!");
  //let mut n_pack = 0usize;
  info!("0MQ SUB socket connected to address {address}");
  // per default, we create master trigger packets from TofEventSummary, 
  // except we have "real" mtb packets
  //let mut craft_mte_packets = true;
  loop {
    match socket.recv_bytes(0) {
      Err(err) => error!("Can't receive TofPacket! {err}"),
      Ok(payload)    => {
        match TofPacket::from_bytestream(&payload, &mut 0) {
          Ok(tp) => {
            match tp_sender.send(tp) {
              Ok(_) => (),
              Err(err) => error!("Can't send TofPacket over channel! {err}")
            }
          }
          Err(err) => {
            debug!("Can't decode payload! {err}");
            // that might have an RB prefix, forward 
            // it 
            match TofPacket::from_bytestream(&payload, &mut 4) {
              Err(err) => {
                error!("Don't understand bytestream! {err}"); 
              },
              Ok(tp) => {
                match tp_sender.send(tp) {
                  Ok(_) => (),
                  Err(err) => error!("Can't send TofPacket over channel! {err}")
                }
              }
            }
          }  
        }
      }
    }
  }
}

