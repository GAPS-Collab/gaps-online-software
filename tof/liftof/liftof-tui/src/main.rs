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

use std::thread;
use std::time::{Duration, Instant};
use std::io;
use std::collections::{VecDeque, HashMap};
#[macro_use] extern crate log;

use tui_logger::TuiLoggerWidget;

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

extern crate crossbeam_channel;
use crossbeam_channel::{unbounded,
                        Sender,
                        Receiver};


use ratatui::{
    backend::CrosstermBackend,
    terminal::Frame,
    layout::{
        Constraint,
        Direction,
        Layout,
        Rect
    },
    style::{
        Color,
        Style
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
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::events::{
    MasterTriggerEvent,
    RBEvent,
};

use liftof_tui::menu::{
    MenuItem,
    MainMenu,
    RBMenuItem,
    RBMenu,
    MTMenu,
    SettingsMenu,
};

use liftof_tui::colors::{
    ColorSet,
    ColorTheme2,
    COLORSETOMILU, // current default
};

use liftof_tui::{
    EventTab,
    HomeTab,
    SettingsTab,
    RBTab,
    RBTabView,
    MTTab,
    CPUTab,
};

extern crate clap;
use clap::{arg,
           command,
           //value_parser,
           //ArgAction,
           //Command,
           Parser};


#[derive(Parser, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// Adjust the rendering rate for the application in Hz
  /// The higher the rate, the more strenous on the 
  /// system, but the more responsive it gets.
  /// On a decent system 1kHz should be ok.
  /// If screen flickering appears, try to change
  /// this parameter. 
  #[arg(short, long, default_value_t = 1000.0)]
  refresh_rate: f32,
}

enum Event<I> {
    Input(I),
    Tick,
}

/// Use the TuiLoggerWidget to display 
/// the most recent log messages
///
///
fn render_logs<'a>(theme : ColorTheme2) -> TuiLoggerWidget<'a> {
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
struct MasterLayout {
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
        PacketType::HeartBeat          => { 
          *pm.get_mut("HeartBeat").unwrap() += 1;
        },
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
        PacketType::RBMoni             => { 
          *pm.get_mut("RBMoni").unwrap() += 1;
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
        PacketType::Ping               => {
          *pm.get_mut("PingPacket").unwrap() += 1;
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
fn packet_receiver(tp_sender_mt : Sender<TofPacket>,
                   tp_sender_rb : Sender<TofPacket>,
                   tp_sender_ev : Sender<TofPacket>,
                   tp_sender_cp : Sender<TofPacket>,
                   str_list     : Arc<Mutex<VecDeque<String>>>,
                   pck_map      : Arc<Mutex<HashMap<String, usize>>>) {
  let ctx = zmq::Context::new();
  // FIXME - don't hardcode this IP
  // tof-computer tailscale is 100.101.96.10/32
  //let address    : &str = "tcp://100.96.207.91:42000";
  //let address_rb : &str = "tcp://100.96.207.91:42001";
  let address : &str = "tcp://192.168.37.20:42000";
  //let address : &str = "tcp://100.101.96.10:42000";
  let data_socket = ctx.socket(zmq::SUB).expect("Unable to create 0MQ SUB socket!");
  data_socket.connect(address).expect("Unable to connect to data (PUB) socket {adress}");
  //data_socket.connect(address_rb).expect("Unable to connect to (PUB) socket {address_rb}");
  match data_socket.set_subscribe(b"") {
    Err(err) => error!("Can't subscribe to any message on 0MQ socket! {err}"),
    Ok(_)    => (),
  }
  let mut n_pack = 0usize;
  info!("0MQ SUB socket connected to address {address}");
  loop {
    match data_socket.recv_bytes(0) {
      Err(err) => error!("Can't receive TofPacket! {err}"),
      Ok(payload)    => {
        match TofPacket::from_bytestream(&payload, &mut 0) {
          Err(err) => {
            debug!("Can't decode payload! {err}");
            // that might have an RB prefix, forward 
            // it 
            match TofPacket::from_bytestream(&payload, &mut 4) {
              Err(err) => {
                error!("Don't understand bytestream! {err}"); 
              },
              Ok(tp) => {
                println!("{:?}", pck_map);
                packet_sorter(&tp.packet_type, &pck_map);
                n_pack += 1;
                //println!("Got TP {}", tp);
                match str_list.lock() {
                  Err(err) => error!("Can't lock shared memory! {err}"),
                  Ok(mut _list)    => {
                    let prefix  = String::from_utf8(payload[0..4].to_vec()).expect("Can't get prefix!");
                    let message = format!("{}-{} {}", n_pack,prefix, tp.to_string());
                    _list.push_back(message);
                  }
                }
            
                match tp_sender_rb.send(tp) {
                  Err(err) => error!("Can't send TP! {err}"),
                  Ok(_)    => (),
                }
              }
            }
          },
          Ok(tp)   => {
            packet_sorter(&tp.packet_type, &pck_map);
            n_pack += 1;
            match str_list.lock() {
              Err(err) => error!("Can't lock shared memory! {err}"),
              Ok(mut _list)    => {
                let message = format!("{} {}", n_pack, tp.to_string());
                _list.push_back(message);
              }
            }
            match tp.packet_type {
              PacketType::MonitorMtb |
              PacketType::MasterTrigger => {
                match tp_sender_mt.send(tp) {
                  Err(err) => error!("Can't send TP! {err}"),
                  Ok(_)    => (),
                }
              },
              PacketType::TofEvent => {
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
              PacketType::RBMoni => {
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
              _ => () 
            }
          }
        }
      }
    }
  } 
}


// make a "holder" for all the tabs and menus, 
// so that it can be put in an Arc(Mutex), so
// we can multithread it
#[derive(Debug, Clone)]
struct TabbedInterface<'a> {
  pub ui_menu       :  MainMenu,
  pub rb_menu       :  RBMenu,
  pub mt_menu       :  MTMenu,
  pub st_menu       :  SettingsMenu,

  // The tabs
  pub mt_tab        : MTTab,
  pub cpu_tab       : CPUTab,
  // waifu tab
  pub wf_tab        : RBTab<'a>,
  pub settings_tab  : SettingsTab<'a>,
  pub home_tab      : HomeTab,
  pub event_tab     : EventTab,

  // latest color set
  pub color_set     : ColorSet,
} 

impl<'a> TabbedInterface<'a> {
  pub fn new(ui_menu      : MainMenu,
             rb_menu      : RBMenu,
             mt_menu      : MTMenu,
             st_menu      : SettingsMenu,
             mt_tab       : MTTab,
             cpu_tab      : CPUTab,
             wf_tab       : RBTab<'a>,
             settings_tab : SettingsTab<'a>,
             home_tab     : HomeTab,
             event_tab    : EventTab) -> Self {
    Self {
      ui_menu     ,
      rb_menu     , 
      mt_menu     , 
      st_menu     ,
      mt_tab      , 
      cpu_tab     , 
      wf_tab      , 
      settings_tab,
      home_tab    , 
      event_tab   , 
      color_set   : COLORSETOMILU,
    }
  }

  pub fn get_colorset(&self) -> ColorSet {
    self.color_set.clone()
  }

  //pub fn set_menu_item(&mut self, item : &MenuItem) {
  //  self.ui_menu.active_menu_item = item.clone();
  //}

  pub fn receive_packet(&mut self) {
    match self.mt_tab.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for MTTab! {err}"),
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
  }

  fn update_color_theme(&mut self, cs : ColorSet) {
    self.st_menu.theme.update(&cs);
    self.ui_menu.theme.update(&cs);
    self.rb_menu.theme.update(&cs);
    self.mt_menu.theme.update(&cs);
    self.home_tab.theme.update(&cs);
    self.event_tab.theme.update(&cs);
    self.wf_tab.theme.update(&cs);
    self.mt_tab.theme.update(&cs);
    self.settings_tab.theme.update(&cs);
    self.cpu_tab.theme.update(&cs);
    self.color_set = cs;
  }
  
  pub fn render_home(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    self.ui_menu.render (&master_lo.rect[0], frame);
    self.home_tab.render(&master_lo.rect[1], frame);
  }

  pub fn render_events(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    self.ui_menu.render  (&master_lo.rect[0], frame);
    self.event_tab.render(&master_lo.rect[1], frame);
  }

  pub fn render_cpu(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    self.ui_menu.render(&master_lo.rect[0], frame);
    self.cpu_tab.render(&master_lo.rect[1], frame);
  }
  
  pub fn render_mt(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    self.mt_menu.render(&master_lo.rect[0], frame);
    self.mt_tab.render (&master_lo.rect[1], frame);
  }
  
  pub fn render_rbs(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    self.rb_menu.render(&master_lo.rect[0], frame);
    self.wf_tab.render (&master_lo.rect[1], frame);
  }
  
  pub fn render_settings(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    self.st_menu.render     (&master_lo.rect[0], frame);
    self.settings_tab.render(&master_lo.rect[1], frame);
  }
      
  pub fn render(&mut self, master_lo : &mut MasterLayout, frame : &mut Frame) {
    match self.ui_menu.active_menu_item {
      MenuItem::Home => {
        self.render_home(master_lo, frame);
      },
      MenuItem::TofEvents => {
        self.render_events(master_lo, frame);
      },
      MenuItem::TOFCpu => { 
        self.render_cpu(master_lo, frame);
      },
      MenuItem::MasterTrigger => {
        self.render_mt(master_lo, frame);
      },
      MenuItem::ReadoutBoards => {
        self.render_rbs(master_lo, frame);
      },
      MenuItem::Settings => {
        self.render_settings(master_lo, frame);
      },
      _ => {
        self.ui_menu.render(&master_lo.rect[0], frame);
      }
    }
  }

  /// Perform actions depending on the input.
  ///
  /// Returns a flag indicating if we should 
  /// close the app
  pub fn digest_input(&mut self, key_code : KeyCode)
  -> bool {
    match self.ui_menu.active_menu_item {
      // if we are in the RBTab, 
      // route input accordingly
      MenuItem::Settings   => {
        match key_code {
          KeyCode::Char('h') => self.ui_menu.active_menu_item = MenuItem::Home,
          KeyCode::Char('a') => {
            self.settings_tab.ctl_active = true;
            self.settings_tab.ctl_state.select(Some(0));
          }
          KeyCode::Up  => {
            if self.settings_tab.ctl_active {
              self.settings_tab.previous_ct();
              match self.settings_tab.get_colorset() {
                None => info!("Did not get a new colorset!"),
                Some(cs) => {
                  //color_theme.update(&cs);
                  self.update_color_theme(cs);
                  //tabs.update_color_theme(cs);
                  //st_menu.theme.update(&cs);
                  //ui_menu.theme.update(&cs);
                  //rb_menu.theme.update(&cs);
                  //mt_menu.theme.update(&cs);
                  //home_tab.theme.update(&cs);
                  //event_tab.theme.update(&cs);
                  //wf_tab.theme.update(&cs);
                  //mt_tab2.theme.update(&cs);
                  //settings_tab.theme.update(&cs);
                  //cpu_tab.theme.update(&cs);
                }
              }
            }
          },
          KeyCode::Down => {
            if self.settings_tab.ctl_active {
              self.settings_tab.next_ct();
              match self.settings_tab.get_colorset() {
                None => info!("Did not get a new colorset!"),
                Some(cs) => {
                  //color_theme.update(&cs);
                  self.update_color_theme(cs);
                  //tabs.update_color_theme(cs);
                  //st_menu.theme.update(&cs);
                  //ui_menu.theme.update(&cs);
                  //rb_menu.theme.update(&cs);
                  //mt_menu.theme.update(&cs);
                  //home_tab.theme.update(&cs);
                  //event_tab.theme.update(&cs);
                  //wf_tab.theme.update(&cs);
                  //mt_tab2.theme.update(&cs);
                  //settings_tab.theme.update(&cs);
                  //cpu_tab.theme.update(&cs);
                }
              }
            }
          },
          KeyCode::Char('q') => {
            return true; // we want to quit
                         // the app
          },
          _ => (),
        }
      },
      MenuItem::ReadoutBoards => {
        self.settings_tab.ctl_active = false;

        match key_code {
          KeyCode::Up  => {
            if self.rb_menu.active_menu_item == RBMenuItem::SelectRB {
              self.wf_tab.previous_rb();
            }
          },
          KeyCode::Down => {
            if self.rb_menu.active_menu_item == RBMenuItem::SelectRB {
              self.wf_tab.next_rb();
            }
          },
          KeyCode::Char('h') => {
            self.ui_menu.active_menu_item = MenuItem::Home;
            self.rb_menu.active_menu_item = RBMenuItem::Home;
          },
          KeyCode::Char('i') => {
            self.rb_menu.active_menu_item = RBMenuItem::Info;
            self.wf_tab.view = RBTabView::Info;
          },
          KeyCode::Char('r') => {
            self.rb_menu.active_menu_item = RBMenuItem::RBMoniData;
            self.wf_tab.view = RBTabView::RBMoniData;
          },
          KeyCode::Char('w') => {
            self.rb_menu.active_menu_item = RBMenuItem::Waveforms;
            self.wf_tab.view = RBTabView::Waveform;
          },
          KeyCode::Char('s') => {
            self.rb_menu.active_menu_item = RBMenuItem::SelectRB;
            self.wf_tab.view = RBTabView::SelectRB;
          },
          KeyCode::Char('q') => {
            return true; // we want to quit the app
          },
          _ => ()
        }
      },
      _ => {
        self.settings_tab.ctl_active = false;
        match key_code {
          // it seems we have to carry thos allong for every tab
          KeyCode::Char('h') => self.ui_menu.active_menu_item = MenuItem::Home,
          KeyCode::Char('t') => self.ui_menu.active_menu_item = MenuItem::TofEvents,
          KeyCode::Char('r') => self.ui_menu.active_menu_item = MenuItem::ReadoutBoards,
          KeyCode::Char('s') => self.ui_menu.active_menu_item = MenuItem::Settings,
          KeyCode::Char('m') => self.ui_menu.active_menu_item = MenuItem::MasterTrigger,
          KeyCode::Char('c') => self.ui_menu.active_menu_item = MenuItem::TOFCpu,
          KeyCode::Char('q') => {
            return true; // trigger exit
          },
          _ => trace!("Some other key pressed!"),
        }
      }
    } // end match ui_menu
    false // if we arrive here, we don't
          // want to exit the app
  }
}

fn main () -> Result<(), Box<dyn std::error::Error>>{

  let home_stream_wd_cnt : Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
  let home_streamer      = home_stream_wd_cnt.clone();

  let mut pm = HashMap::<String, usize>::new();
  pm.insert(String::from("Unknown"          ) ,0);
  pm.insert(String::from("RBEvent"          ) ,0); 
  pm.insert(String::from("TofEvent"         ) ,0); 
  pm.insert(String::from("HeartBeat"        ) ,0); 
  pm.insert(String::from("MasterTrigger"    ) ,0);
  pm.insert(String::from("RBEventHeader"    ) ,0);
  pm.insert(String::from("CPUMoniData"      ) ,0); 
  pm.insert(String::from("MonitorMtb"       ) ,0); 
  pm.insert(String::from("RBMoni"           ) ,0); 
  pm.insert(String::from("PAMoniData"       ) ,0); 
  pm.insert(String::from("PBMoniData"       ) ,0); 
  pm.insert(String::from("LTBMoniData"      ) ,0); 
  pm.insert(String::from("RBEventMemoryView") ,0); 
  pm.insert(String::from("RBCalibration"    ) ,0); 
  pm.insert(String::from("TofCommand"       ) ,0); 
  pm.insert(String::from("RBCommand"        ) ,0); 
  pm.insert(String::from("MultiPacket"      ) ,0); 
  pm.insert(String::from("PingPacket"       ) ,0); 
  pm.insert(String::from("RBWaveform"       ) ,0); 
  pm.insert(String::from("TofEventSummary"  ) ,0); 
  
  let packet_map : Arc<Mutex<HashMap<String, usize>>> = Arc::new(Mutex::new(pm));
  let packet_map_home = packet_map.clone();

  // sender receiver combo to subscribe to tofpackets
  let (mt_pack_send, mt_pack_recv) : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  let (rb_pack_send, rb_pack_recv) : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  let (ev_pack_send, ev_pack_recv) : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  let (cp_pack_send, cp_pack_recv) : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();

  // sender receiver for inter thread communication with decoded packets
  let (mte_send, mte_recv)         : (Sender<MasterTriggerEvent>, Receiver<MasterTriggerEvent>) = unbounded();
  let (rbe_send, rbe_recv)         : (Sender<RBEvent>, Receiver<RBEvent>) = unbounded();

  //let (_tx, _rx)                     : (Sender<Event>, Receiver<Event<I>>) = unbounded();

  // FIXME - spawn a new thread per each tab!
  let _packet_recv_thread = thread::Builder::new()
    .name("mt_packet_receiver".into())
    .spawn(move || {
      packet_receiver(mt_pack_send, 
                      rb_pack_send,
                      ev_pack_send,
                      cp_pack_send,
                      home_stream_wd_cnt,
                      packet_map,
                      );
    })
    .expect("Failed to spawn mt packet receiver thread!");
  
  // Set max_log_level to Trace
  match tui_logger::init_logger(log::LevelFilter::Info) {
    Err(err) => panic!("Something bad just happened {err}"),
    Ok(_)    => (),
  }
  // Set default level for unknown targets to Trace ("Trace"/"Info") 
  tui_logger::set_default_level(log::LevelFilter::Info);
  
  let args = Args::parse();                   
  
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
                           last_tick = Instant::now();
                         }
                       }
                     }
                   }
    ).expect("Failed to spawn heartbeat thread!");


  // A color theme, can be changed later
  let mut color_theme     = ColorTheme2::new();
  color_theme.update(&COLORSETOMILU);
  //let mut color_set_bw    = 
  
  // The menus
  let ui_menu         = MainMenu::new(color_theme.clone());
  let rb_menu         = RBMenu::new(color_theme.clone());
  let mt_menu         = MTMenu::new(color_theme.clone());
  let st_menu         = SettingsMenu::new(color_theme.clone());

  // The tabs
  let mt_tab          = MTTab::new(mt_pack_recv,
                                       mte_recv,
                                       color_theme.clone());
 
  let cpu_tab         = CPUTab::new(cp_pack_recv,
                                        color_theme.clone());
  // waifu tab
  let wf_tab          = RBTab::new(rb_pack_recv,
                                       rbe_recv,
                                       color_theme.clone());
  let settings_tab    = SettingsTab::new(color_theme.clone());
  let home_tab        = HomeTab::new(color_theme.clone(), home_streamer, packet_map_home);
  let event_tab       = EventTab::new(ev_pack_recv, mte_send, rbe_send, color_theme);

  let tabs        = TabbedInterface::new(ui_menu,
                                         rb_menu,
                                         mt_menu,
                                         st_menu,
                                         mt_tab,
                                         cpu_tab,
                                         wf_tab,
                                         settings_tab,
                                         home_tab,
                                         event_tab);

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
                   }
    ).expect("Failed to spawn tab-packet-receiver thread!");

  let mut quit_app = false;
  loop {
    match rx.recv() {
      Err(err) => trace!("Err - no update! {err}"),
      Ok(event) => {
        match event {
          Event::Input(ev) => {
            match shared_tabs.lock() {
              Err(err) => error!("Unable to get lock on shared tabbed interface! {err}"),
              Ok(mut tabs) => {
                //tabs.receive_packet();
                if tabs.digest_input(ev.code) {
                  quit_app = true;
                  // true means end program
                }
                let cs = tabs.get_colorset();
                color_theme.update(&cs);
              }
            }
          }, 
          Event::Tick => {
            match shared_tabs.lock() {
              Err(err) => error!("Unable to get lock on shared tabbed interface! {err}"),
              Ok(mut tabs) => {
                match terminal.draw(|frame| {
                  let size           = frame.size();
                  let mut mster_lo   = MasterLayout::new(size); 
                  let w_logs         = render_logs(color_theme.clone());
                  frame.render_widget(w_logs, mster_lo.rect[2]);
                  tabs.render(&mut mster_lo, frame);
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
