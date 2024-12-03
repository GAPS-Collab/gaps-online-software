//! Interactive display for the tof system for the 
//! GAPS experiment
//!
//!
//!
//!

use std::sync::{
    Arc,
    Mutex,
};

//use std::path::Path;

use std::thread;
use std::time::{
    Duration,
    Instant
};
use std::io;
use std::collections::{
    VecDeque,
    HashMap
};
#[macro_use] extern crate log;

// third party widgets
use tui_logger::TuiLoggerWidget;
use tui_popup::Popup;

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

//extern crate crossbeam_channel;
use crossbeam_channel::{unbounded,
                        Sender,
                        Receiver};


use ratatui::{
    backend::CrosstermBackend,
    //terminal::Frame,
    Frame,
    layout::{
        Constraint,
        Direction,
        Layout,
        Rect
    },
    style::{
        Color,
        Style,
    },
    widgets::{
        Block,
        Borders,
    },
    Terminal,
};

use tof_dataclasses::packets::{
    TofPacket,
    PacketType
};

use tof_dataclasses::database::{
    connect_to_db,
    get_dsi_j_ch_pid_map,
    get_linkid_rbid_map,
    ReadoutBoard,
    Paddle,
};

use tof_dataclasses::serialization::{
    Serialization,
    Packable,
};
use tof_dataclasses::events::{
    MasterTriggerEvent,
    RBEvent,
    TofEvent,
    TofHit,
    TofEventSummary,
    //RBWaveform,
};
use tof_dataclasses::calibrations::RBCalibrations;
//use liftof_lib::settings::LiftofSettings;

use liftof_tui::menu::{
    UIMenuItem,
    MenuItem,
    MainMenu2,
    TriggerMenu,
    EventMenu,
    MoniMenu,
    ActiveMenu,
    RBMenu2,
    UIMenu,
    PAMoniMenu,
    SettingsMenu,
    THMenu,
    TSMenu,
    //TSMenuItem,
    RWMenu,
    HBMenu,
    //RWMenuItem,
};

use liftof_tui::colors::{
    ColorSet,
    ColorTheme,
    COLORSETBW, // current default (valkyrie)
    //COLORSETOMILU
};

use liftof_tui::{
    EventTab,
    HomeTab,
    SettingsTab,
    TofHitTab,
    RBTab,
    RBTabView,
    MTTab,
    CPUTab,
    RBWaveformTab,
    TofSummaryTab, 
    TelemetryTab,
    CommandTab,
    PaddleTab,
    HeartBeatTab,
    HeartBeatView
};


//extern crate clap;
use clap::{arg,
           command,
           //value_parser,
           //ArgAction,
           //Command,
           Parser
};

cfg_if::cfg_if! {
  if #[cfg(feature = "telemetry")]  {
    use telemetry_dataclasses::packets::{
      TelemetryHeader,
      TelemetryPacket,
    };

    /// Get the GAPS merged event telemetry stream and 
    /// broadcast it to the relevant tab
    ///
    /// # Arguments
    ///
    /// * tele_sender : Channel to forward the received telemetry
    ///                 packets
    /// * address     : Address to susbscribe to for telemetry 
    ///                 stream (must be zmq.PUB) on the Sender
    ///                 side
    fn socket_wrap_telemetry(address     : &str,
                             tele_sender : Sender<TelemetryPacket>) {
      let ctx = zmq::Context::new();
      // FIXME - don't hardcode this IP
      // typically how it is done is that this program runs either on a gse
      // or there is a local forwarding of the port thrugh ssh
      //let address : &str = "tcp://127.0.0.1:55555";
      let socket = ctx.socket(zmq::SUB).expect("Unable to create 0MQ SUB socket!");
      match socket.connect(&address) {
        Err(err) => {
          error!("Unable to connect to data (PUB) socket {address}! {err}");
          panic!("Can not connect to zmq PUB socket!");
        }
        Ok(_) => ()
      }
      socket.set_subscribe(b"") .expect("Can't subscribe to any message on 0MQ socket! {err}");
      loop {
        match socket.recv_bytes(0) {
          Err(err)    => error!("Can't receive TofPacket! {err}"),
          Ok(mut payload) => {
            match TelemetryHeader::from_bytestream(&payload, &mut 0) {
              Err(err) => {
                error!("Can not decode telemtry header! {err}");
                //for k in pos - 5 .. pos + 5 {
                //  println!("{}",stream[k]);
                //}
              }
              Ok(header) => {
                let mut packet = TelemetryPacket::new();
                if payload.len() > TelemetryHeader::SIZE {
                  payload.drain(0..TelemetryHeader::SIZE);
                }
                packet.header  = header;
                packet.payload = payload;
                match tele_sender.send(packet) {
                  Err(err) => error!("Can not send telemetry packet to downstream! {err}"),
                  Ok(_)    => ()
                }
              }
            }
          }
        }
      }
    }
  }
}


/// ZMQ socket wrapper for the zmq socket which is 
/// supposed to receive data from the TOF system.
fn socket_wrap_tofstream(address   : &str,
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


#[derive(Parser, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// Adjust the rendering rate for the application in Hz
  /// The higher the rate, the more strenous on the 
  /// system, but the more responsive it gets.
  /// On a decent system 1kHz should be ok.
  /// If screen flickering appears, try to change
  /// this parameter. 
  #[arg(short, long, default_value_t = 10.0)]
  refresh_rate: f32,
  /// Get the TofData not from the TOF stream, 
  /// but extract it from the telemetry data instead
  /// THIS NEEDS THAT THE CODE HAS BEEN COMPILED WITH 
  /// --features=telemetry
  #[arg(short, long, default_value_t = false)]
  from_telemetry : bool,
  /// Allow to control the liftof-cc server with commands
  /// WARNING - this is an expert feature. Only use if 
  /// you are know what you are doing
  #[arg(long, default_value_t = false)]
  allow_commands : bool

  ///// generic liftof config file. If not given, we 
  ///// assume liftof-config.toml in this directory
  //#[arg(short, long)]
  //config: Option<String>,
}

enum Event<I> {
    Input(I),
    Tick,
}


/// Use the TuiLoggerWidget to display 
/// the most recent log messages
///
///
fn render_logs<'a>(theme : ColorTheme) -> TuiLoggerWidget<'a> {
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

#[derive(Debug, Clone)]
pub struct MasterLayout {
  pub rect : Vec<Rect>
}

impl MasterLayout {

  fn new(size : Rect) -> MasterLayout {
    let chunks = Layout::default()
    .direction(Direction::Vertical)
    //.margin(1)
    .constraints(
      [
        Constraint::Length(3),
        Constraint::Min(2),
        Constraint::Length(5),
      ]
      .as_ref(),
    )
    .split(size);
    MasterLayout {
      rect : chunks.to_vec()
    }
  }
}


/// Just produce a summary of all the packets we received
fn packet_sorter(packet_type : &PacketType,
                 packet_map  : &Arc<Mutex<HashMap<String,usize>>>) {
  match packet_map.lock() {
    Ok(mut pm) => {
      match packet_type {
        PacketType::Unknown            => {
          *pm.get_mut("Unknown").unwrap() += 1;
        },
        PacketType::RBEvent            => { 
          *pm.get_mut("RBEvent").unwrap() += 1;
        },
        PacketType::TofEvent           => { 
          *pm.get_mut("TofEvent").unwrap() += 1;
        },
        PacketType::HeartBeatDataSink  => { 
          *pm.get_mut("HeartBeatDataSink").unwrap() += 1;
        },
        PacketType::MTBHeartbeat => {
          *pm.get_mut("MTBHeartbeat").unwrap() += 1;
        }
        PacketType::EVTBLDRHeartbeat => { 
          *pm.get_mut("EVTBLDRHeartbeat").unwrap() += 1;
        }
        PacketType::MasterTrigger      => { 
          *pm.get_mut("MasterTrigger").unwrap() += 1;
        },
        PacketType::RBEventHeader      => { 
          *pm.get_mut("RBEventHeader").unwrap() += 1;
        },
        PacketType::CPUMoniData      => { 
          *pm.get_mut("CPUMoniData").unwrap() += 1;
        },
        PacketType::MonitorMtb         => { 
          *pm.get_mut("MonitorMtb").unwrap() += 1;
        },
        PacketType::RBMoniData         => { 
          *pm.get_mut("RBMoniData").unwrap() += 1;
        },
        PacketType::RBEventMemoryView  => { 
          *pm.get_mut("RBEventMemoryView").unwrap() += 1;
        },
        PacketType::RBCalibration      => { 
          *pm.get_mut("RBCalibration").unwrap() += 1;
        },
        PacketType::TofCommand         => { 
          *pm.get_mut("TofCommand").unwrap() += 1;
        },
        PacketType::TofResponse        => {
          *pm.get_mut("TofResponse").unwrap() += 1;
        },
        PacketType::RBCommand          => { 
          *pm.get_mut("RBCommand").unwrap() += 1;
        },
        PacketType::PAMoniData         => { 
          *pm.get_mut("PAMoniData").unwrap() += 1;
        },
        PacketType::PBMoniData         => { 
          *pm.get_mut("PBMoniData").unwrap() += 1;
        },
        PacketType::LTBMoniData        => { 
          *pm.get_mut("LTBMoniData").unwrap() += 1;
        },
        PacketType::MultiPacket        => { 
          *pm.get_mut("MultiPacket").unwrap() += 1;
        },
        PacketType::RBWaveform        => { 
          *pm.get_mut("RBWaveform").unwrap() += 1;
        },
        PacketType::TofEventSummary        => { 
          *pm.get_mut("TofEventSummary").unwrap() += 1;
        },
        PacketType::RBPing               => {
          *pm.get_mut("RBPing").unwrap() += 1;
        }
        _ => {
          error!("Packet type {packet_type} currently not supported!");
        }
      }
    },
    Err(err) => {
      error!("Can't lock shared memory! {err}");
    }
  }
}

/// Receive packets from an IP address
/// and distrubute them to their receivers
/// while taking notes of everything
///
/// This is a Pablo Pubsub kind of persona
/// (see a fantastic talk at RustConf 2023)
fn packet_distributor(tp_from_sock : Receiver<TofPacket>,
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
                      pck_map      : Arc<Mutex<HashMap<String, usize>>>) {
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

// make a "holder" for all the tabs and menus, 
// so that it can be put in an Arc(Mutex), so
// we can multithread it
pub struct TabbedInterface<'a> {
  pub ui_menu       :  MainMenu2<'a>,
  pub rb_menu       :  RBMenu2<'a>,
  pub mt_menu       :  TriggerMenu<'a>,
  pub st_menu       :  SettingsMenu,
  pub th_menu       :  THMenu,
  pub ts_menu       :  TSMenu,
  pub rw_menu       :  RWMenu,
  pub pa_menu       :  PAMoniMenu,
  pub te_menu       :  EventMenu<'a>,
  pub mo_menu       :  MoniMenu<'a>,
  pub hb_menu       :  HBMenu<'a>,
  pub active_menu   :  ActiveMenu,

  // The tabs
  pub mt_tab        : MTTab,
  pub cpu_tab       : CPUTab,
  // waifu tab
  pub wf_tab        : RBTab<'a>,
  pub settings_tab  : SettingsTab<'a>,
  pub home_tab      : HomeTab,
  pub event_tab     : EventTab,
  pub cmd_tab       : CommandTab<'a>,

  pub th_tab        : TofHitTab<'a>,
  // flight packets
  pub rbwf_tab      : RBWaveformTab,
  pub ts_tab        : TofSummaryTab,
  
  // telemetry 
  pub te_tab        : TelemetryTab,

  // paddles 
  pub pd_tab        : PaddleTab<'a>,

  pub hb_tab        : HeartBeatTab,

  // latest color set
  pub color_set     : ColorSet,

  pub quit_request  : bool,
} 

impl<'a> TabbedInterface<'a> {
  pub fn new(ui_menu      : MainMenu2<'a>,
             rb_menu      : RBMenu2<'a>,
             mt_menu      : TriggerMenu<'a>,
             st_menu      : SettingsMenu,
             th_menu      : THMenu,
             ts_menu      : TSMenu,
             rw_menu      : RWMenu,
             pa_menu      : PAMoniMenu,
             te_menu      : EventMenu<'a>,
             mo_menu      : MoniMenu<'a>,
             hb_menu      : HBMenu<'a>,
             active_menu  : ActiveMenu,
             mt_tab       : MTTab,
             cpu_tab      : CPUTab,
             wf_tab       : RBTab<'a>,
             settings_tab : SettingsTab<'a>,
             home_tab     : HomeTab,
             event_tab    : EventTab,
             th_tab       : TofHitTab<'a>,
             rbwf_tab     : RBWaveformTab,
             ts_tab       : TofSummaryTab,
             te_tab       : TelemetryTab,
             cmd_tab      : CommandTab<'a>,
             hb_tab       : HeartBeatTab,
             pd_tab       : PaddleTab<'a>) -> Self {
    Self {

      ui_menu     ,
      rb_menu     , 
      mt_menu     , 
      st_menu     ,
      th_menu     ,
      ts_menu     ,
      rw_menu     ,
      pa_menu     ,
      te_menu     ,
      mo_menu     ,
      hb_menu     ,
      active_menu ,
      mt_tab      , 
      cpu_tab     , 
      wf_tab      , 
      settings_tab,
      home_tab    , 
      event_tab   , 
      th_tab      ,
      rbwf_tab    ,
      ts_tab      ,
      te_tab      ,
      cmd_tab     ,
      pd_tab      , 
      hb_tab      ,
      color_set   : COLORSETBW,
      quit_request: false,
    }
  }

  pub fn get_colorset(&self) -> ColorSet {
    self.color_set.clone()
  }

  pub fn receive_packet(&mut self) {
    match self.mt_tab.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for MTTab! {err}"),
      Ok(_)    => ()
    }
    match self.pd_tab.receive_packet() {
      Err(err) => error!("Can not receive TofEvent for PaddleTab! {err}"),
      Ok(_)    => ()
    }
    match self.wf_tab.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for WfTab! {err}"),
      Ok(_)    => ()
    }
    match self.event_tab.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for EventTab! {err}"),
      Ok(_)    => ()
    }
    match self.cpu_tab.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for CPUTab! {err}"),
      Ok(_)    => ()
    }
    match self.th_tab.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for TofHitTab! {err}"),
      Ok(_)    => ()
    }
    match self.rbwf_tab.receive_packet() {
      Err(err) => error!("Can not receive RBWaveforms for RBWaveformTab! {err}"),
      Ok(_)    => ()
    }
    match self.ts_tab.receive_packet() {
      Err(err) => error!("Can not receive TofEventSummaries for TofEventSummaryTab! {err}"),
      Ok(_)    => ()
    }
    match self.hb_tab.receive_packet() {
      Err(err) => error!("Can not receive Heartbeats for HeartBeatTab! {err}"),
      Ok(_)    => ()
    }
    // the same though, so maybe for now it is fine
    match self.te_tab.receive_packet() {
      Err(err) => error!("Can not receive a new packet from the Telemetry Stream! {err}"),
      Ok(_)    => ()
    }
  }

  fn update_color_theme(&mut self, cs : ColorSet) {
    self.st_menu.theme.update(&cs);
    self.ui_menu.theme.update(&cs);
    self.rb_menu.theme.update(&cs);
    self.mt_menu.theme.update(&cs);
    self.rw_menu.theme.update(&cs);
    self.ts_menu.theme.update(&cs);
    self.th_menu.theme.update(&cs);
    self.te_menu.theme.update(&cs);
    self.hb_menu.theme.update(&cs);
    self.home_tab    .theme.update(&cs);
    self.event_tab   .theme.update(&cs);
    self.wf_tab      .theme.update(&cs);
    self.mt_tab      .theme.update(&cs);
    self.settings_tab.theme.update(&cs);
    self.cpu_tab     .theme.update(&cs);
    self.th_tab      .theme.update(&cs);
    self.rbwf_tab    .theme.update(&cs);
    self.ts_tab      .theme.update(&cs);
    self.te_tab      .theme.update(&cs);
    self.cmd_tab     .theme.update(&cs);
    self.pd_tab      .theme.update(&cs);
    self.hb_tab      .theme.update(&cs);
    self.color_set = cs;
  }
  
  pub fn render_home(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    self.ui_menu.render (&master_lo.rect[0], frame);
    self.home_tab.render(&master_lo.rect[1], frame);
  }

  pub fn render_events(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    self.te_menu.render  (&master_lo.rect[0], frame);
    self.event_tab.render(&master_lo.rect[1], frame);
  }

  pub fn render_monitoring(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    self.mo_menu.render(&master_lo.rect[0], frame);
    self.home_tab.render(&master_lo.rect[1], frame);
  }

  //pub fn render_cpu(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
  //  self.ui_menu.render(&master_lo.rect[0], frame);
  //  self.cpu_tab.render(&master_lo.rect[1], frame);
  //}
  
  pub fn render_mt(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    self.ui_menu.render(&master_lo.rect[0], frame);
    self.mt_tab.render (&master_lo.rect[1], frame);
  }
  
  pub fn render_rbs(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    match self.active_menu {
      ActiveMenu::RBMenu => {
        self.rb_menu.render(&master_lo.rect[0], frame);
        self.wf_tab.render (&master_lo.rect[1], frame);
      }
      _ => {
        self.ui_menu.render(&master_lo.rect[0], frame);
        self.wf_tab.render (&master_lo.rect[1], frame);
      }
    }
  }
  
  pub fn render_paddles(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    match self.active_menu {
      ActiveMenu::Paddles => {
        self.pd_tab.menu.render(&master_lo.rect[0], frame);
        self.pd_tab.render (&master_lo.rect[1], frame);
      }
      _ => {
        self.ui_menu.render(&master_lo.rect[0], frame);
        self.pd_tab.render (&master_lo.rect[1], frame);
      }
    }
  }

  pub fn render_heartbeats(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    match self.active_menu {
      ActiveMenu::Heartbeats => {
        self.hb_menu.render(&master_lo.rect[0], frame);
      }
      _ => {
        self.ui_menu.render(&master_lo.rect[0], frame);
      }
    }
    self.hb_tab.render(&master_lo.rect[1], frame);
  }

  pub fn render_commands(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    self.ui_menu.render(&master_lo.rect[0], frame);
    self.cmd_tab.render(&master_lo.rect[1], frame);
  }

  pub fn render_settings(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    self.ui_menu.render     (&master_lo.rect[0], frame);
    self.settings_tab.render(&master_lo.rect[1], frame);
  }
   
  pub fn render_quit(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    self.ui_menu.render(&master_lo.rect[0], frame);
    if self.quit_request {
      let popup = Popup::new("Quit liftof-tui?")
        .title("Press Y to confirm, any key to abort")
        .style(self.home_tab.theme.style());
      frame.render_widget(&popup, frame.area());
    }
  }

  pub fn render_tofhittab(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    self.th_menu.render(&master_lo.rect[0], frame);
    self.th_tab.render(&master_lo.rect[1], frame);
  }

  pub fn render_tofsummarytab(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    self.te_menu.render(&master_lo.rect[0], frame);
    self.ts_tab.render(&master_lo.rect[1], frame);
  }
  
  pub fn render_rbwaveformtab(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    self.te_menu.render(&master_lo.rect[0], frame);
    self.rbwf_tab.render(&master_lo.rect[1], frame);
  }

  //pub fn render_pamonidatatab(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
  //  self.pa_menu.render(&master_lo.rect[0], frame);
  //  self.wf_tab.render(&master_lo.rect[1], frame);
  //}
  
  pub fn render_telemetrytab(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    //self.ts_menu.render(&master_lo.rect[0], frame);
    self.ui_menu.render(&master_lo.rect[0], frame);
    self.te_tab.render(&master_lo.rect[1], frame);
  }

  pub fn render(&mut self,
                master_lo : &mut MasterLayout,
                frame     : &mut Frame) {
    

    match self.active_menu {
      ActiveMenu::MainMenu => {
        match self.ui_menu.get_active_menu_item() {
          UIMenuItem::Home => {
            self.render_home(master_lo, frame);
          }
          UIMenuItem::Events => {
            self.render_home(master_lo, frame);
          },
          UIMenuItem::ReadoutBoards => {
            self.wf_tab.view = RBTabView::SelectRB;
            self.render_rbs(master_lo, frame);
          }
          UIMenuItem::Trigger => {
            self.render_mt(master_lo, frame);
          }
          UIMenuItem::Monitoring => {
            self.render_home(master_lo, frame);
          }
          UIMenuItem::Telemetry => {
            self.render_telemetrytab(master_lo, frame);
          }
          UIMenuItem::Commands => {
            self.render_commands(master_lo, frame);
          }
          UIMenuItem::Settings => {
            self.render_settings(master_lo, frame);
          }
          UIMenuItem::Paddles => {
            self.render_paddles(master_lo, frame);
          }
          UIMenuItem::Heartbeats => {
            self.render_heartbeats(master_lo, frame);
          }
          UIMenuItem::Quit => {
            self.render_quit(master_lo, frame);
          }
          _ => ()
        }
      }
      ActiveMenu::RBMenu => {
        self.render_rbs(master_lo, frame);
      }
      ActiveMenu::Paddles => {
        self.render_paddles(master_lo, frame);
      }
      ActiveMenu::Heartbeats => {
        self.render_heartbeats(master_lo, frame);
      }
      //ActiveMenu::Trigger => {
      //  self.render_mt(master_lo, frame);
      //}
      ActiveMenu::Events => {
        match self.te_menu.active_menu_item {
          UIMenuItem::TofSummary => {
            self.render_tofsummarytab(master_lo, frame);
          }
          UIMenuItem::TofHits => {
            self.render_tofhittab(master_lo, frame);
          }
          UIMenuItem::RBWaveform => {
            self.render_rbwaveformtab(master_lo, frame);
          }
          UIMenuItem::Back => {
            self.render_events(master_lo, frame);
          }
          _ => ()
        }
      }
      ActiveMenu::Monitoring => {
        match self.mo_menu.active_menu_item {
          UIMenuItem::Back => {
            self.render_monitoring(master_lo, frame);
          }
          UIMenuItem::PreampBias => {
          }
          UIMenuItem::PreampTemp => {
          }
          UIMenuItem::LTBThresholds => {
          }
          _ => ()
        }
      }
      _ => ()
    }
  }

  /// Perform actions depending on the input.
  ///
  /// Returns a flag indicating if we should 
  /// close the app
  ///
  /// # Returns:
  ///
  /// * (bool, bool) - quit and tab_changed. Indicator if the app should quit or if a tab
  ///   has changed
  pub fn digest_input(&mut self, key_code : KeyCode)
  -> (bool, bool) {
    let mut tab_changed = false;
    if self.quit_request {
      match key_code {
        KeyCode::Char('Y') => {
          return (true, tab_changed);
        }
        _ => {
          self.quit_request = false;
        }
      }
    }
    if self.settings_tab.colortheme_popup {
      self.settings_tab.colortheme_popup = false;
    }

    match key_code {
      KeyCode::Char('a') => {
        if self.ui_menu.get_active_menu_item() == UIMenuItem::Settings {
          self.settings_tab.ctl_active = true;
          //self.settings_tab.ctl_state.select(Some(0));
        }
      }
      KeyCode::Enter => {
        //info!("{:?}", self.ui_menu.get_active_menu_item());
        self.settings_tab.ctl_active = false;
        match self.active_menu {
          ActiveMenu::Events  => {
            match self.te_menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.ui_menu.set_active_menu_item(UIMenuItem::Home);
                self.ui_menu.active_menu_item = MenuItem::Home;
                self.active_menu = ActiveMenu::MainMenu;
              }
              _ => ()
            }
          }
          //ActiveMenu::Trigger => {
          //  match self.mt_menu.get_active_menu_item() {
          //    UIMenuItem::Back => {
          //      self.ui_menu.set_active_menu_item(UIMenuItem::Home);
          //      self.ui_menu.active_menu_item = MenuItem::Home;
          //      self.active_menu = ActiveMenu::MainMenu;
          //    }
          //    _ => ()
          //  }
          //}
          ActiveMenu::Paddles => {
            match self.pd_tab.menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.ui_menu.set_active_menu_item(UIMenuItem::Paddles);
                //self.ui_menu.active_menu_item = MenuItem::Home;
                self.active_menu = ActiveMenu::MainMenu;
              }
              _ => ()
            }
          }
          ActiveMenu::Monitoring => {
            match self.mo_menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.ui_menu.set_active_menu_item(UIMenuItem::Home);
                self.ui_menu.active_menu_item = MenuItem::Home;
                self.active_menu = ActiveMenu::MainMenu;
              }
              _ => ()
            }
          }
          ActiveMenu::Heartbeats => {
            match self.hb_menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.ui_menu.set_active_menu_item(UIMenuItem::Home);
                self.ui_menu.active_menu_item = MenuItem::Home;
                self.active_menu = ActiveMenu::MainMenu;
              }
              _ => ()
            }
          }
          ActiveMenu::RBMenu => {
            match self.rb_menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.ui_menu.set_active_menu_item(UIMenuItem::ReadoutBoards);
                self.ui_menu.active_menu_item = MenuItem::ReadoutBoards;
                self.active_menu = ActiveMenu::MainMenu;
              }
              _ => ()
            }
          }
          ActiveMenu::MainMenu => {
            match self.ui_menu.get_active_menu_item() {
              UIMenuItem::ReadoutBoards => {
                //self.rb_menu.set_active_menu_item(UIMenuItem::Back);
                info!("Setting active menu to RBMenu!");
                self.active_menu = ActiveMenu::RBMenu;
              }
              UIMenuItem::Trigger => {
                self.active_menu = ActiveMenu::Trigger;
              }
              UIMenuItem::Events => {
                self.active_menu = ActiveMenu::Events;
              }
              UIMenuItem::Monitoring => {
                self.active_menu = ActiveMenu::Monitoring;
              }
              UIMenuItem::Paddles => {
                self.active_menu = ActiveMenu::Paddles;
              }
              UIMenuItem::Heartbeats => {
                self.active_menu = ActiveMenu::Heartbeats;
              }
              UIMenuItem::Commands => {
                self.cmd_tab.send_command(); 
                //self.active_menu = ActiveMenu::Paddles;
              }
              UIMenuItem::Settings => {
                self.settings_tab.ctl_active       = true;
                self.settings_tab.colortheme_popup = true;
              }
              UIMenuItem::Quit => {
                info!("Feeling a desire of the user to quit this application...");
                self.quit_request = true;
              }
              _ => ()
            }
          }
          _ => ()
        }
      }
      KeyCode::Right => {
        tab_changed = true;
        self.settings_tab.ctl_active = false;
        match self.active_menu {
          ActiveMenu::MainMenu => {
            self.ui_menu.next();
          }
          ActiveMenu::RBMenu => {
            self.rb_menu.next();
            match self.rb_menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.wf_tab.view = RBTabView::SelectRB; 
              }
              UIMenuItem::Waveforms => {
                self.wf_tab.view = RBTabView::Waveform; 
              }
              UIMenuItem::RBMoniData => {
                self.wf_tab.view = RBTabView::RBMoniData;
              }
              UIMenuItem::PAMoniData => {
                self.wf_tab.view = RBTabView::PAMoniData;
              }
              UIMenuItem::PBMoniData => {
                self.wf_tab.view = RBTabView::PBMoniData;
              }
              UIMenuItem::LTBMoniData => {
                self.wf_tab.view = RBTabView::LTBMoniData; 
              }
              _ => ()
            }
          }
          ActiveMenu::Paddles => {
            self.pd_tab.menu.next();
          }
          ActiveMenu::Heartbeats => {
            self.hb_menu.next();
            match self.hb_menu.get_active_menu_item() {
              UIMenuItem::EventBuilderHB => {
                self.hb_tab.view = HeartBeatView::EventBuilder; 
              }
              UIMenuItem::TriggerHB => {
                self.hb_tab.view = HeartBeatView::MTB; 
              }
              UIMenuItem::DataSenderHB => {
                self.hb_tab.view = HeartBeatView::DataSink; 
              }
              _ => ()
            }
          }
          //ActiveMenu::Trigger => {
          //  self.mt_menu.next();
          //}
          ActiveMenu::Events => {
            self.te_menu.next();
          }
          ActiveMenu::Monitoring => {
            self.mo_menu.next();
          }
          _ => ()
        }
      }
      KeyCode::Left => {
        tab_changed = true;
        self.settings_tab.ctl_active = false;
        match self.active_menu {
          ActiveMenu::MainMenu => {
            self.ui_menu.prev();
          }
          ActiveMenu::RBMenu => {
            self.rb_menu.prev();
            match self.rb_menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.wf_tab.view = RBTabView::SelectRB; 
              }
              UIMenuItem::Waveforms => {
                self.wf_tab.view = RBTabView::Waveform; 
              }
              UIMenuItem::RBMoniData => {
                self.wf_tab.view = RBTabView::RBMoniData;
              }
              UIMenuItem::PAMoniData => {
                self.wf_tab.view = RBTabView::PAMoniData;
              }
              UIMenuItem::PBMoniData => {
                self.wf_tab.view = RBTabView::PBMoniData;
              }
              UIMenuItem::LTBMoniData => {
                self.wf_tab.view = RBTabView::LTBMoniData; 
              }
              _ => ()
            }
          }
          //ActiveMenu::Trigger => {
          //  self.mt_menu.prev();
          //}
          ActiveMenu::Paddles => {
            self.pd_tab.menu.prev();
          }
          ActiveMenu::Events => {
            self.te_menu.prev();
          }
          ActiveMenu::Monitoring => {
            self.mo_menu.prev();
          }
          ActiveMenu::Heartbeats => {
            self.hb_menu.prev();
            match self.hb_menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.hb_tab.view = HeartBeatView::EventBuilder; 
              }
              UIMenuItem::EventBuilderHB => {
                self.hb_tab.view = HeartBeatView::EventBuilder; 
              }
              UIMenuItem::TriggerHB => {
                self.hb_tab.view = HeartBeatView::MTB; 
              }
              UIMenuItem::DataSenderHB => {
                self.hb_tab.view = HeartBeatView::DataSink; 
              }
              _ => ()
            }
          }
          _ => ()
        }
      }
      KeyCode::Down => {
        if self.settings_tab.ctl_active {
          self.settings_tab.next_ct();
          match self.settings_tab.get_colorset() {
            None => info!("Did not get a new colorset!"),
            Some(cs) => {
              self.update_color_theme(cs);
            }
          }
        }
        // Paddle lsit
        if self.active_menu == ActiveMenu::MainMenu && self.ui_menu.get_active_menu_item() == UIMenuItem::Paddles {
          self.pd_tab.next_pd();
          //info!("selected rb {}", self.wf_tab.rb_selector);
        }
        if self.active_menu == ActiveMenu::Paddles && self.pd_tab.menu.get_active_menu_item() == UIMenuItem::Back {
          self.pd_tab.next_pd();
        }
        // RB list
        if self.active_menu == ActiveMenu::MainMenu && self.ui_menu.get_active_menu_item() == UIMenuItem::ReadoutBoards {
          self.wf_tab.next_rb();
          //info!("selected rb {}", self.wf_tab.rb_selector);
        }
        if self.active_menu == ActiveMenu::RBMenu && self.rb_menu.get_active_menu_item() == UIMenuItem::Back {
          self.wf_tab.next_rb();
        }
        if self.active_menu == ActiveMenu::MainMenu && self.ui_menu.get_active_menu_item() == UIMenuItem::Commands {
          self.cmd_tab.next_cmd();
        }
      }
      KeyCode::Up => {
        if self.settings_tab.ctl_active {
          self.settings_tab.previous_ct();
          match self.settings_tab.get_colorset() {
            None => info!("Did not get a new colorset!"),
            Some(cs) => {
              self.update_color_theme(cs);
            }
          }
        }
        // Paddle lsit
        if self.active_menu == ActiveMenu::MainMenu && self.ui_menu.get_active_menu_item() == UIMenuItem::Paddles {
          self.pd_tab.prev_pd();
          //info!("selected rb {}", self.wf_tab.rb_selector);
        }
        if self.active_menu == ActiveMenu::Paddles && self.pd_tab.menu.get_active_menu_item() == UIMenuItem::Back {
          self.pd_tab.prev_pd();
        }
        if self.active_menu == ActiveMenu::MainMenu && self.ui_menu.get_active_menu_item() == UIMenuItem::ReadoutBoards {
          self.wf_tab.previous_rb();
        }
        if self.active_menu == ActiveMenu::RBMenu && self.rb_menu.get_active_menu_item() == UIMenuItem::Back {
          self.wf_tab.previous_rb();
        }
        if self.active_menu == ActiveMenu::MainMenu && self.ui_menu.get_active_menu_item() == UIMenuItem::Commands {
          self.cmd_tab.prev_cmd();
        }
      }
      _ => {
        self.settings_tab.ctl_active = false;
      }
    }
    info!("Returning false");
    (false, tab_changed) // if we arrive here, we don't
                        // want to exit the app
  }
}

fn main () -> Result<(), Box<dyn std::error::Error>>{
  
  let args = Args::parse();                   
  let allow_commands = args.allow_commands;

  let home_stream_wd_cnt : Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
  let home_streamer      = home_stream_wd_cnt.clone();

  // calibrations for everybody!
  let rbcalibrations : Arc<Mutex<HashMap<u8, RBCalibrations>>> = Arc::new(Mutex::new(HashMap::<u8, RBCalibrations>::new()));
  // prepare calibrations
  let mut readoutboards = HashMap::<u8, ReadoutBoard>::new();
  let mut rb_conn = connect_to_db(String::from("gaps_flight2.db")).expect("Will need database access. Make sure gaps_flight2.db is installed!");
  let rbs     = ReadoutBoard::all(&mut rb_conn).expect("Will need database access. Make sure gaps_flight2.db is installed!");
  let mtlink_rb_map = get_linkid_rbid_map(&rbs);
  for mut rb in rbs {
    rb.calib_file_path = String::from("calibrations");
    match rb.load_latest_calibration() {
      Err(err) => error!("Unable to load calibration for {}! {}", rb, err),
      Ok(_) => {
        match rbcalibrations.lock() {
          Err(_err)  => error!("Unable to lock rbcalibrations mutex!"),
          Ok(mut rbcal) => {
            rbcal.insert(rb.rb_id as u8, rb.calibration.clone()); 
          }
        }
        readoutboards.insert(rb.rb_id as u8, rb);
      }
    }
  }
  let paddles = Paddle::all(&mut rb_conn).expect("Database corrupt!");
  let dsijch_paddle_map = get_dsi_j_ch_pid_map(&paddles);
  let mut paddle_map = HashMap::<u8, Paddle>::new();
  for pdl in paddles {
    paddle_map.insert(pdl.paddle_id as u8, pdl.clone());
  }
  let mut pm = HashMap::<String, usize>::new();
  pm.insert(String::from("Unknown"          ) ,0);
  pm.insert(String::from("RBEvent"          ) ,0); 
  pm.insert(String::from("TofEvent"         ) ,0); 
  pm.insert(String::from("HeartBeatDataSink") ,0); 
  pm.insert(String::from("MTBHeartbeat"     ) ,0); 
  pm.insert(String::from("EVTBLDRHeartbeat" ) ,0); 
  pm.insert(String::from("MasterTrigger"    ) ,0);
  pm.insert(String::from("RBEventHeader"    ) ,0);
  pm.insert(String::from("CPUMoniData"      ) ,0); 
  pm.insert(String::from("MonitorMtb"       ) ,0); 
  pm.insert(String::from("RBMoniData"       ) ,0); 
  pm.insert(String::from("PAMoniData"       ) ,0); 
  pm.insert(String::from("PBMoniData"       ) ,0); 
  pm.insert(String::from("LTBMoniData"      ) ,0); 
  pm.insert(String::from("RBEventMemoryView") ,0); 
  pm.insert(String::from("RBCalibration"    ) ,0); 
  pm.insert(String::from("TofCommand"       ) ,0); 
  pm.insert(String::from("TofResponse"      ) ,0); 
  pm.insert(String::from("RBCommand"        ) ,0); 
  pm.insert(String::from("MultiPacket"      ) ,0); 
  pm.insert(String::from("RBPing"           ) ,0); 
  pm.insert(String::from("RBWaveform"       ) ,0); 
  pm.insert(String::from("TofEventSummary"  ) ,0); 
  
  let packet_map : Arc<Mutex<HashMap<String, usize>>> = Arc::new(Mutex::new(pm));
  let packet_map_home = packet_map.clone();

  // this determines the source of all TofPackets
  let (tp_to_distrib, tp_from_sock)     : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();

  // sender receiver combo to subscribe to tofpackets
  let (mt_pack_send, mt_pack_recv)      : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  let (rb_pack_send, rb_pack_recv)      : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  let (ev_pack_send, ev_pack_recv)      : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  let (cp_pack_send, cp_pack_recv)      : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  let (rbwf_pack_send, rbwf_pack_recv)  : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  let (tr_pack_send, tr_pack_recv)      : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  #[cfg(feature = "telemetry")]
  let (te_pack_send,  te_pack_recv)     : (Sender<TelemetryPacket>, Receiver<TelemetryPacket>) = unbounded();

  // sender receiver for inter thread communication with decoded packets
  let (mte_send, mte_recv)         : (Sender<MasterTriggerEvent>, Receiver<MasterTriggerEvent>) = unbounded();
  let (rbe_send, rbe_recv)         : (Sender<RBEvent>, Receiver<RBEvent>)                       = unbounded();
  let (th_send, th_recv)           : (Sender<TofHit>, Receiver<TofHit>)                         = unbounded();
  let (ts_send, ts_recv)           : (Sender<TofEventSummary>, Receiver<TofEventSummary>)       = unbounded();
  let (te_send, te_recv)           : (Sender<TofEvent>, Receiver<TofEvent>)                     = unbounded();

  // send tof packets containing heartbeats
  let (hb_pack_send, hb_pack_recv)      : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  

  // depending on the switch in the 
  // commandline args, we either 
  // connect to the telemetry stream 
  // or directly to the tof data strea
  // ROUTING: In this case we reroute the 
  // packets through the TelemetryTab, 
  // which unpacks them and then it will 
  // funnel in the packet-distributor 
  // thread
  // This also means, that in case where we 
  // don't want the packets from telemetry,
  // we have to spawn an additional thread which 
  // receives the packets from the TOF stream
  // instead
  //if args.from_telemetry {
  if !args.from_telemetry {
    let _tp_to_distrib = tp_to_distrib.clone();
    let tofcpu_address : &str = "tcp://192.168.37.20:42000";
    let _packet_recv_thread = thread::Builder::new()
      .name("socket-wrap-tofstream".into())
      .spawn(move || {
        socket_wrap_tofstream(tofcpu_address,
                              _tp_to_distrib);
      }).expect("Unable to spawn socket-wrap-tofstream thread!");
  }
  //  let _packet_recv_thread = thread::Builder::new()
  //    .name("packet-receiver".into())
  //    .spawn(move || {
  //      let telemetry_address : &str = "tcp://127.0.0.1:55555";
  //      socket_wrap_telemetry(telemetry_address, 
  //                            tp_to_distrib );
  //    })
  //  .expect("Failed to spawn mt packet receiver thread!");


  // FIXME - spawn a new thread per each tab!
  let th_sender_c = th_send.clone();
  let _packet_dist_thread = thread::Builder::new()
    .name("packet-distributor".into())
    .spawn(move || {
      packet_distributor(tp_from_sock,
                         mt_pack_send, 
                         rb_pack_send,
                         ev_pack_send,
                         cp_pack_send,
                         tr_pack_send,
                         rbwf_pack_send,
                         ts_send,
                         th_sender_c,
                         hb_pack_send,
                         home_stream_wd_cnt,
                         packet_map,
                         );
    }).expect("Failed to spawn mt packet receiver thread!");
  
  // spawn the telemetry receiver thread
  cfg_if::cfg_if! {
    if #[cfg(feature = "telemetry")] {
      let telemetry_address : &str = "tcp://127.0.0.1:55555";
      let _telemetry_receiver_thread = thread::Builder::new()
        .name("socket-wrap-telemetry".into())
        .spawn(move || {
          socket_wrap_telemetry(
            telemetry_address,
            te_pack_send
          );
        })
        .expect("Failed to spawn mt packet receiver thread!");
    }
  }
  // Set max_log_level to Trace
  match tui_logger::init_logger(log::LevelFilter::Info) {
    Err(err) => panic!("Something bad just happened {err}"),
    Ok(_)    => (),
  }
  // Set default level for unknown targets to Trace ("Trace"/"Info") 
  tui_logger::set_default_level(log::LevelFilter::Info);
  
  
  // set up the terminal
  enable_raw_mode().expect("Unable to enter raw mode");
  let stdout       = io::stdout();
  let backend      = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;
  terminal.clear()?;
  
  //let (tx, rx) = mpsc::channel();
  let (tx, rx) = unbounded();

  // heartbeat, keeps it going
  let _heartbeat_thread = thread::Builder::new()
    .name("heartbeat".into())
    .spawn(move || {
                     // change this to make it more/less 
                     // responsive
                     let refresh_perioad = 1000.0/args.refresh_rate as f32;
                     let tick_rate = Duration::from_millis(refresh_perioad.round() as u64);
                     let mut last_tick = Instant::now();
                     loop {
                       let timeout = tick_rate
                         .checked_sub(last_tick.elapsed())
                         .unwrap_or_else(|| Duration::from_secs(0));
                       if event::poll(timeout).expect("poll works") {
                         if let CEvent::Key(key) = event::read().expect("can read events") {
                           tx.send(Event::Input(key)).expect("can send events");
                         }
                       }
                       if last_tick.elapsed() >= tick_rate {
                         if let Ok(_) = tx.send(Event::Tick) {
                           //info!("ticker : {}", last_tick.elapsed().as_micros());
                           last_tick = Instant::now();
                         }
                       }
                     }
                   }
    ).expect("Failed to spawn heartbeat thread!");


  // A color theme, can be changed later
  let mut color_theme     = ColorTheme::new();
  color_theme.update(&COLORSETBW);
  //let mut color_set_bw    = 
  
  // The menus
  let ui_menu         = MainMenu2::new(color_theme.clone());
  let rb_menu         = RBMenu2::new(color_theme.clone());
  //let mt_menu         = MTMenu::new(color_theme.clone());
  let mt_menu         = TriggerMenu::new(color_theme.clone());
  let st_menu         = SettingsMenu::new(color_theme.clone());
  let th_menu         = THMenu::new(color_theme.clone());
  let ts_menu         = TSMenu::new(color_theme.clone());
  let rw_menu         = RWMenu::new(color_theme.clone());
  let pa_menu         = PAMoniMenu::new(color_theme.clone());
  let te_menu         = EventMenu::new(color_theme.clone());
  let mo_menu         = MoniMenu::new(color_theme.clone());
  let hb_menu         = HBMenu::new(color_theme.clone());
  // The tabs
  let mt_tab          = MTTab::new(mt_pack_recv,
                                   mte_recv,
                                   dsijch_paddle_map,
                                   mtlink_rb_map,
                                   color_theme.clone());
 
  let cpu_tab         = CPUTab::new(cp_pack_recv,
                                    color_theme.clone());
  // waifu tab
  let wf_tab          = RBTab::new(rb_pack_recv,
                                   rbe_recv,
                                   readoutboards.clone(),
                                   color_theme.clone());
  let settings_tab    = SettingsTab::new(color_theme.clone());
  let home_tab        = HomeTab::new(color_theme.clone(), home_streamer, packet_map_home);
  let event_tab       = EventTab::new(ev_pack_recv, mte_send, rbe_send, th_send, te_send, color_theme);
  let hit_tab         = TofHitTab::new(th_recv,color_theme.clone());
  let rbwf_tab        = RBWaveformTab::new(rbwf_pack_recv,
                                           readoutboards,
                                           color_theme.clone());
  let ts_tab          = TofSummaryTab::new(ts_recv, color_theme.clone());
  let te_tab          : TelemetryTab;
  if args.from_telemetry {
    te_tab            = TelemetryTab::new(Some(tp_to_distrib),
                                          te_pack_recv,
                                          color_theme.clone());
  } else {
    te_tab      = TelemetryTab::new(None,
                                    te_pack_recv, 
                                    color_theme.clone());
  
  
  }
  let cmd_sender_addr = String::from("tcp://192.168.37.5:42000");
  let cmd_tab         = CommandTab::new(tr_pack_recv, cmd_sender_addr, color_theme.clone(), allow_commands);
  let pd_tab          = PaddleTab::new(te_recv, paddle_map, rbcalibrations, color_theme.clone());
  let hb_tab          = HeartBeatTab::new(hb_pack_recv, color_theme.clone());
  let active_menu     = ActiveMenu::MainMenu;
  let tabs            = TabbedInterface::new(ui_menu,
                                             rb_menu,
                                             mt_menu,
                                             st_menu,
                                             th_menu,
                                             ts_menu,
                                             rw_menu,
                                             pa_menu,
                                             te_menu,
                                             mo_menu,
                                             hb_menu,
                                             active_menu,
                                             mt_tab,
                                             cpu_tab,
                                             wf_tab,
                                             settings_tab,
                                             home_tab,
                                             event_tab,
                                             hit_tab,
                                             rbwf_tab,
                                             ts_tab,
                                             te_tab,
                                             cmd_tab,
                                             hb_tab,
                                             pd_tab);

  let shared_tabs : Arc<Mutex<TabbedInterface>> = Arc::new(Mutex::new(tabs));
  let shared_tabs_c = shared_tabs.clone();
  let _update_thread = thread::Builder::new()
    .name("tab-packet-receiver".into())
    .spawn(move || {
      loop {
        match shared_tabs_c.lock() {
          Err(err) => error!("Can't get lock on shared tabs! {err}"),
          Ok(mut tabs) => {
            tabs.receive_packet();
          }
        }
      }
    }).expect("Failed to spawn tab-packet-receiver thread!");

  let mut quit_app = false;
  loop {
    match rx.recv() {
      Err(err) => trace!("Err - no update! {err}"),
      Ok(event) => {
        //info!("next event!");
        match event {
          Event::Input(ev) => {
            //info!("input");
            match shared_tabs.lock() {
              Err(err) => error!("Unable to get lock on shared tabbed interface! {err}"),
              Ok(mut tabs) => {
                //tabs.receive_packet();
                let (want_quit, tab_changed) = tabs.digest_input(ev.code);
                quit_app = want_quit;
                if tab_changed {
                  let _ = terminal.clear();
                }
                //if want_quit {
                //  quit_app = true;
                //  // true means end program
                //}
                let cs = tabs.get_colorset();
                color_theme.update(&cs);
              }
            }
          }, 
          Event::Tick => {
            //info!("tick");
            match shared_tabs.lock() {
              Err(err) => error!("Unable to get lock on shared tabbed interface! {err}"),
              Ok(mut tabs) => {
                match terminal.draw(|frame| {
                  let size           = frame.area();
                  let mut main_lo    = MasterLayout::new(size); 
                  let w_logs         = render_logs(color_theme.clone());
                  frame.render_widget(w_logs, main_lo.rect[2]);
                  tabs.render(&mut main_lo, frame);
                }) {
                  Err(err) => error!("Can't render terminal! {err}"),
                  Ok(_)    => () ,
                } // end terminal.draw
              }
            }
          } // end Event::Tick
        }
      }
    }
    if quit_app {
      disable_raw_mode()?;
      terminal.clear()?;
      terminal.show_cursor()?;
      break;
    }
  } // end loop;
  Ok(())
}
