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

use tui_logger::TuiLoggerWidget;
use ratatui::{
  style::{
    Color,
    Style,
  },
  widgets::{
    Block,
    Borders,
  },
};

pub use crate::tabs::*;
pub use crate::layout::*;

use crate::colors::ColorTheme;
use gondola_core::prelude::*;

use crossbeam_channel::{
  Sender,
  Receiver
};

pub type WaveformCache = HashMap<u8,HashMap<u8, Arc<Mutex<VecDeque<RBWaveform>>>>>;

/// Create a global storage for RBWaveforms 
pub fn global_waveform_cache() -> WaveformCache {
  let mut cache = WaveformCache::new();
  //let rbs   = ReadoutBoard::all();
  //let paddles = TofPaddle::all_as_dict();
  for k in 1..50 {
    let ch_dict = HashMap::<u8, Arc<Mutex<VecDeque<RBWaveform>>>>::new();
    cache.insert(k as u8, ch_dict);
    for j in 1..10 {
      let wfs     = Arc::new(Mutex::new(VecDeque::<RBWaveform>::new()));
      cache.get_mut(&k).unwrap().insert(j as u8, wfs); 
    }
  }
  cache
}

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

/// Use the TuiLoggerWidget to display 
/// the most recent log messages
///
///
pub fn render_logs<'a>(theme : ColorTheme) -> TuiLoggerWidget<'a> {
  TuiLoggerWidget::default()
    .style_error(Style::default().fg(Color::Red))
    .style_debug(Style::default().fg(Color::Green))
    .style_warn(Style::default().fg(Color::Yellow))
    .style_trace(Style::default().fg(Color::Gray))
    .style_info(Style::default().fg(Color::Blue))
    .block(
      Block::default()
        .title("Logs")
        .border_style(theme.style())
        .borders(Borders::ALL),
    )   
    .style(theme.style())
}

/// Count the different types of tofpackets and store the result 
/// in a HashMap
///
/// # Arguments:
///
///   * packet_type : TofPacket type to llokup it's position in 
///                   the map
///   * packet_map  : An arc/mutex to the HashMap we use to store
///                   the counted values in.
fn packet_sorter(packet_type : &TofPacketType,
                 packet_map  : &Arc<Mutex<HashMap<&str,usize>>>) {
  match packet_map.lock() {
    Ok(mut pm) => {
      let pack_key : &str;
      //let pt = packet_type.cl
      //let pt = packet_type.clone();
      //let pack_key = pt.as_ref();
      match packet_type {
        TofPacketType::Unknown               => pack_key = "Unknown", 
        TofPacketType::RBEvent               => pack_key = "RBEvent",
        TofPacketType::TofEvent              => pack_key = "TofEvent",
        TofPacketType::RBWaveform            => pack_key = "RBWaveform",
        TofPacketType::TofEventDeprecated    => pack_key = "TofEventDeprecated",
        TofPacketType::DataSinkHB            => pack_key = "DataSinkHB",    
        TofPacketType::MasterTrigger         => pack_key = "MasterTrigger",
        TofPacketType::TriggerConfig         => pack_key = "TriggerConfig",
        TofPacketType::MasterTriggerHB       => pack_key = "MasterTriggerHHB", 
        TofPacketType::EventBuilderHB        => pack_key = "EventBuilderHB",
        TofPacketType::RBChannelMaskConfig   => pack_key = "RBChannelMaskConfig",
        TofPacketType::TofRBConfig           => pack_key = "TofRBConfig",
        TofPacketType::AnalysisEngineConfig  => pack_key = "AnalysisEngineConfig",
        TofPacketType::RBEventHeader         => pack_key = "RBEventHeader",    // needs to go away
        TofPacketType::TOFEventBuilderConfig => pack_key = "TOFEventBuilderConfig",
        TofPacketType::DataPublisherConfig   => pack_key = "DataPublisherConfig",
        TofPacketType::TofRunConfig          => pack_key = "TofRunConfig",
        TofPacketType::CPUMoniData           => pack_key = "CPUMoniData",
        TofPacketType::MtbMoniData           => pack_key = "MtbMoniData",
        TofPacketType::RBMoniData            => pack_key = "RBMoniData",
        TofPacketType::PBMoniData            => pack_key = "PBMoniData",
        TofPacketType::LTBMoniData           => pack_key = "LTBMoniData",
        TofPacketType::PAMoniData            => pack_key = "PAMoniData",
        TofPacketType::RBEventMemoryView     => pack_key = "RBEventMemoryView", // We'll keep it for now - indicates that the event
        TofPacketType::RBCalibration         => pack_key = "RBCalibration",
        TofPacketType::TofCommand            => pack_key = "TofCommand",
        TofPacketType::TofCommandV2          => pack_key = "TofCommandV2",
        TofPacketType::TofResponse           => pack_key = "TofResponse",
        TofPacketType::RBCommand             => pack_key = "RBCommand",
        TofPacketType::RBPing                => pack_key = "RBPing",
        TofPacketType::PreampBiasConfig      => pack_key = "PreampBiasConfig",
        TofPacketType::RunConfig             => pack_key = "RunConfig",
        TofPacketType::LTBThresholdConfig    => pack_key = "LTBThresholdConfig",
        TofPacketType::TofDetectorStatus     => pack_key = "TofDetectorStatus",
        TofPacketType::ConfigBinary          => pack_key = "ConfigBinary",
        TofPacketType::LiftofRBBinary        => pack_key = "LiftofRBBinary",
        TofPacketType::LiftofBinaryService   => pack_key = "LiftofBinaryService",
        TofPacketType::LiftofCCBinary        => pack_key = "LiftofCCBinary",
        TofPacketType::RBCalibrationFlightV  => pack_key = "RBCalibrationFlightV",
        TofPacketType::RBCalibrationFlightT  => pack_key = "RBCalibrationFlightT",
        TofPacketType::BfswAckPacket         => pack_key = "BfswAckPacket",
        TofPacketType::PanicPacket           => pack_key = "PanicPacket",
        TofPacketType::MultiPacket           => pack_key = "MultiPacket",
        TofPacketType::PanicPacket           => pack_key = "PanicPacket",
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
                          //rbwf_sender  : Sender<TofPacket>,
                          ts_send      : Sender<TofPacket>,
                          th_send      : Sender<TofHit>,
                          tp_sender_hb : Sender<TofPacket>,
                          str_list     : Arc<Mutex<VecDeque<String>>>,
                          pck_map      : Arc<Mutex<HashMap<&str, usize>>>,
                          mut writer   : Option<TofPacketWriter>,
                          mut wf_cache : Box<WaveformCache>) {
  let mut n_pack = 0usize;

  loop {
    //match data_socket.recv_bytes(0) {
    //println! ("Incoming receiver length {}", tp_from_sock.len());
    //println!("len tp_sender_mt {}", tp_sender_mt.len());
    //println!("len tp_sender_rb {}", tp_sender_rb.len());
    //println!("len tp_sender_ev {}", tp_sender_ev.len());
    //println!("len tp_sender_cp {}", tp_sender_cp.len());
    //println!("len tp_sender_tr {}", tp_sender_tr.len());
    //println!("len rbwf_sender {}" , rbwf_sender.len());
    //println!("len th_send {}"     , th_send.len());
    //println!("len ts_send {}"     , ts_send.len());
    //println!("len tp_sender_hb {}", tp_sender_hb.len());
    match tp_from_sock.recv() {
      Err(err) => error!("Can't receive TofPacket! {err}"),
      Ok(tp) => {
        //println!("{:?}", pck_map);
        //println!("Before packet sorter!");
        packet_sorter(&tp.packet_type, &pck_map);
        //println!("After packet sorter!");
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
        //continue; 
        // if --capture, write file
        if writer.is_some() {
          writer.as_mut().unwrap().add_tof_packet(&tp);
        }
        match tp.packet_type {
          TofPacketType::TofResponse => { 
            match tp_sender_tr.send(tp) {
              Err(err) => error!("Can't send TP! {err}"),
              Ok(_)    => (),
            }
          }
          TofPacketType::MtbMoniData => {
            match tp_sender_mt.send(tp) {
              Err(err) => error!("Can't send TP! {err}"),
              Ok(_)    => (),
            }
          }
          TofPacketType::RBWaveform => {
            match tp.unpack::<RBWaveform>() {
              Ok(wf) => {
                 
                let rb_id = wf.rb_id;
                let ch_a  = wf.rb_channel_a + 1;
                let ch_b  = wf.rb_channel_b + 1;
                debug!("Successfully unpacked RBWaveform! {} {} {}", rb_id, ch_a, ch_b);
                //if rb_id != 25 {
                //  continue;
                //} 
                //if ch_a != 3 {
                //  continue
                //}
                //println!("{wf}");
                if rb_id > 50 || rb_id == 0 {
                  continue;
                }
                if ch_a > 9 || ch_a == 0 {
                  error!("Invalid channel B {}", ch_a);
                  continue;
                }
                if ch_b > 9 || ch_b == 0 {
                  error!("Invalid channel A {}", ch_b);
                  continue;
                }
                //error!("1 : Pushing wf with rb id {} to {}", wf.rb_id, rb_id);
                match wf_cache.get_mut(&wf.rb_id).unwrap().get_mut(&ch_a).unwrap().lock() {
                  Err(err)  => {
                    error!("Unable to lock waveform cache! {err}");
                  }
                  Ok(mut cache) => {
                    //error!("2 : Pushing wf with rb id {} to {}", wf.rb_id, rb_id);
                    cache.push_back(wf.clone());
                    //error!("3 : Pushing wf with rb id {} to {}", wf.rb_id, rb_id);
                    debug!("Last wf in cache has rb id {}", cache.back().unwrap().rb_id); 
                    if cache.len() > 1000 {
                      cache.pop_front();
                    }
                  }
                }
                match wf_cache.get_mut(&wf.rb_id).unwrap().get_mut(&ch_b).unwrap().lock() {
                  Err(err)  => {
                    error!("Unable to lock waveform cache!");
                  }
                  Ok(mut cache) => {
                    cache.push_back(wf.clone());
                    if cache.len() > 1000 {
                      cache.pop_front();
                    }
                   // println!("Pushing waveform for {rb_id} - {ch_b} to cache of len {}", cache.len());
                  }
                }
              }
              Err(err) => {
                error!("Unable to unpack RBWaveform! {err}");
              }
            }
            //match rbwf_sender.send(tp) {
            //  Err(err) => error!("Can't send TP! {err}"),
            //  Ok(_)    => (),
            //}
          }
          TofPacketType::TofEvent => {
            match ts_send.send(tp) {
              Err(err) => error!("Can't send TP! {err}"),
              Ok(_)    => (),
            }
            
            //match tp.unpack::<TofEvent>() {
            //  Err(err) => {
            //    error!("Unable to unpack TofEvent! {err}");
            //  }
            //  Ok(ts) => {
            //    // FIXME - this is not needed anymore
            //    //if craft_mte_packets {
            //    //  //let mte    = MasterTriggerEvent::from(&ts);
            //    //  //let mte_tp = mte.pack();
            //    //  //error!("We are sending the following tp {}", mte_tp);
            //    //  match tp_sender_mt.send(tp.clone()) { 
            //    //  //match tp_sender_mt.send(mte_tp) {
            //    //    Err(err) => error!("Can't send MTE TP! {err}"),
            //    //    Ok(_)    => ()
            //    //  }
            //    //}
            //    //for h in &ts.hits {
            //    //  match th_send.send(*h) {
            //    //    Err(err) => error!("Can't send TP! {err}"),
            //    //    Ok(_)    => (),
            //    //  }
            //    //}
            //    //match ts_send.send(ts) {
            //    //  Err(err) => error!("Can't send TP! {err}"),
            //    //  Ok(_)    => (),
            //    //}
            //    //match tp_sender_ev.send(tp.clone()) {
            //    //  Err(err) => error!("Can't send TP! {err}"),
            //    //  Ok(_)    => (),
            //    //}
            //  }
            //}
          }
          ////TofPacketType::TofEvent => {
          ////  // since the tof event contains MTEs, we don't need
          ////  // to craft them
          ////  craft_mte_packets = false;
          ////  match tp_sender_ev.send(tp) {
          ////    Err(err) => error!("Can't send TP! {err}"),
          ////    Ok(_)    => (),
          ////  }
          ////  // Disasemble the packets
          ////  //match TofEvent::from_bytestream(tp.payload, &mut 0) {
          ////  //  Err(err) => {
          ////  //    error!("Can't decode TofEvent");
          ////  //  },
          ////  //  Ok(ev) => {
          ////  //    //for rbev in ev.rb_events {
          ////  //    //  let 
          ////  //    //  match tp_sender_rb.send
          ////  //    //}
          ////  //  }
          ////  //}
          ////}
          TofPacketType::RBEvent |
          TofPacketType::RBEventMemoryView | 
          TofPacketType::LTBMoniData |
          TofPacketType::PAMoniData  |
          TofPacketType::PBMoniData  |
          TofPacketType::RBMoniData => {
            match tp_sender_rb.send(tp) {
              Err(err) => error!("Can't send TP! {err}"),
              Ok(_)    => (),
            }
          }
          TofPacketType::CPUMoniData => {
            match tp_sender_cp.send(tp) {
              Err(err) => error!("Can't send TP! {err}"),
              Ok(_)    => (),
            }
          }
          TofPacketType::DataSinkHB      |
          TofPacketType::EventBuilderHB  | 
          TofPacketType::MasterTriggerHB => {
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


