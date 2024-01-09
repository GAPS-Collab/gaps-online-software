//! Interactive display for the tof system for the 
//! GAPS experiment
//!
//!
//!
//!


//mod tab_commands;
mod tab_mt;
mod tab_home;
mod tab_status;
mod menu;
mod colors;
mod widgets;
mod tab_settings;
mod tab_events;

use std::sync::{
    mpsc,
    Arc,
    Mutex,
};

use std::thread;
use std::time::{Duration, Instant};
use std::io;
use std::path::{Path, PathBuf};
use std::collections::{VecDeque, HashMap};
#[macro_use] extern crate log;

extern crate json;

extern crate histo;
use histo::Histogram;

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
    symbols,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    terminal::Frame,
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{
        Block, Borders, ListState, Paragraph, Row, Table, Tabs,    },
    Terminal,
};



use tof_dataclasses::commands::{TofCommand, TofResponse};
use tof_dataclasses::packets::{TofPacket, PacketType};
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::events::{
    MasterTriggerEvent,
    RBEvent,
};
use tof_dataclasses::monitoring::MtbMoniData;
use tof_dataclasses::manifest::ReadoutBoard;

use crate::tab_mt::{
    MTTab,
};

use crate::tab_settings::{
    SettingsTab,
};

use crate::tab_home::{
    HomeTab,
};

use crate::tab_events::{
    EventTab,
};

use crate::tab_status::{
    //StatusTab,
    RBTab,
    RBTabView
};

use crate::menu::{
    MenuItem,
    MainMenu,
    RBMenuItem,
    RBMenu,
    MTMenu,
    SettingsMenu,
};

use crate::colors::{
    ColorTheme,
    ColorTheme2,
    COLORSETBW,
    COLORSETOMILU,
};

// keep at max this amount of tof packets
const STREAM_CACHE_MAX_SIZE : usize = 10;

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
  /// Don't discover readoutboards, but connect to some 
  /// local fake instances instead.
  #[arg(short, long, default_value_t = false)]
  debug_local: bool,
  /// Autodiscover connected readoutboards
  #[arg(short, long, default_value_t = false)]
  autodiscover_rb: bool,
  /// A json config file with detector information
  #[arg(short, long)]
  json_config: Option<std::path::PathBuf>,
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
          *pm.get_mut("Unkwown").unwrap() += 1;
        },
        PacketType::RBEvent            => { 
          *pm.get_mut("RBEvent").unwrap() += 1;
        },
        PacketType::TofEvent           => { 
          *pm.get_mut("TofEvent").unwrap() += 1;
        },
        PacketType::Monitor            => { 
          *pm.get_mut("Monitor").unwrap() += 1;
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
        PacketType::MonitorTofCmp      => { 
          *pm.get_mut("MonitorTofCmp").unwrap() += 1;
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
        }
      }
    },
    Err(err) => {
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
                   str_list     : Arc<Mutex<VecDeque<String>>>,
                   pck_map      : Arc<Mutex<HashMap<String, usize>>>) {
  let ctx = zmq::Context::new();
  // FIXME - don't hardcode this IP
  // tof-computer tailscale is 100.101.96.10/32
  //let address    : &str = "tcp://100.96.207.91:42000";
  //let address_rb : &str = "tcp://100.96.207.91:42001";
  let address : &str = "tcp://100.101.96.10:42000";
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
                packet_sorter(&tp.packet_type, &pck_map);
                n_pack += 1;
                //println!("Got TP {}", tp);
                match str_list.lock() {
                  Err(err) => error!("Can't lock shared memory! {err}"),
                  Ok(mut _list)    => {
                    let prefix  = String::from_utf8(payload[0..4].to_vec()).unwrap();
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
              Err(err) => error!("Can't lock shared memory!"),
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
              },
              _ => () 
            }
          }
        }
      }
    }
  } 
}


fn main () -> Result<(), Box<dyn std::error::Error>>{

  let home_stream_wd_cnt : Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
  let home_streamer      = home_stream_wd_cnt.clone();

  // a shared location for frame and main window, so that the individual tabs can render to it
  let shared_frame : Arc<Mutex<Frame>>;//  = Arc::new(Mutex::new(Frame::new()));

  let mut pm = HashMap::<String, usize>::new();
  pm.insert(String::from("Unknown"          ) ,0);
  pm.insert(String::from("RBEvent"          ) ,0); 
  pm.insert(String::from("TofEvent"         ) ,0); 
  pm.insert(String::from("Monitor"          ) ,0);
  pm.insert(String::from("HeartBeat"        ) ,0); 
  pm.insert(String::from("MasterTrigger"    ) ,0);
  pm.insert(String::from("RBEventHeader"    ) ,0);
  pm.insert(String::from("MonitorTofCmp"    ) ,0); 
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
 
  let packet_map : Arc<Mutex<HashMap<String, usize>>> = Arc::new(Mutex::new(pm));
  let packet_map_home = packet_map.clone();

  // sender receiver combo to subscribe to tofpackets
  let (mt_pack_send, mt_pack_recv) : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  let (rb_pack_send, rb_pack_recv) : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  let (ev_pack_send, ev_pack_recv) : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();

  // sender receiver for inter thread communication with decoded packets
  let (mte_send, mte_recv)         : (Sender<MasterTriggerEvent>, Receiver<MasterTriggerEvent>) = unbounded();
  let (rbe_send, rbe_recv)         : (Sender<RBEvent>, Receiver<RBEvent>) = unbounded();


  // FIXME - spawn a new thread per each tab!
  let _packet_recv_thread = thread::Builder::new()
         .name("mt_packet_receiver".into())
         .spawn(move || {
           packet_receiver(mt_pack_send, 
                           rb_pack_send,
                           ev_pack_send,
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
  //let debug_local       = args.debug_local;         
  //let autodiscover_rb   = args.autodiscover_rb;    
  //
  let mut ten_second_update = Instant::now();
  let mission_elapsed_time  = Instant::now();
 
  let mut rb_list = Vec::<ReadoutBoard>::new();
  let mut tick_count = 0;
  //let json_content  : String;
  //let config        : json::JsonValue;

  //match args.json_config {
  //  None => panic!("No .json config file provided! Please provide a config file with --json-config or -j flag!"),
  //  Some(_) => {
  //    json_content = std::fs::read_to_string(args.json_config.as_ref().unwrap()).expect("Can not open json file");
  //    config = json::parse(&json_content).expect("Unable to parse json file");
  //  } // end Some
  //} // end match
  //let calib_file_path  = config["calibration_file_path"].as_str().unwrap().to_owned();
  //let db_path          = Path::new(config["db_path"].as_str().unwrap());
  //let db_path_c        = db_path.clone();
  //let ltb_list         = get_ltbs_from_sqlite(db_path);

  //let rb_ignorelist =  &config["rb_ignorelist"];
  ////exit(0);
  //let mut rb_list       = get_rbs_from_sqlite(db_path_c);
  //for k in 0..rb_ignorelist.len() {
  //  println!("=> We will remove RB {} due to it being marked as IGNORE in the config file!", rb_ignorelist[k]);
  //  let bad_rb = rb_ignorelist[k].as_u8().unwrap();
  //  rb_list.retain(|x| x.rb_id != bad_rb);
  //}
  //println!("=> We will use the following tof manifest:");
  //println!("== ==> LTBs [{}]:", ltb_list.len());
  //for ltb in &ltb_list {
  //  println!("\t {}", ltb);
  //}
  //println!("== ==> RBs [{}]:", rb_list.len());
  //for rb in &rb_list {
  //  println!("\t {}", rb);
  //}

  //let master_trigger_ip      = config["master_trigger"]["ip"].as_str().unwrap().to_owned();
  //let master_trigger_port    = config["master_trigger"]["port"].as_usize().unwrap();
  //let mtb_address = master_trigger_ip.clone() + ":" + &master_trigger_port.to_string();

  //let rb_list_c  = rb_list.clone();
  //let rb_list_c2 = rb_list.clone();
  //// first set up comms etc. before 
  //// we go into raw_mode, so we can 
  //// see the log messages during setup
  let (mt_to_main, mt_from_mt)     : (Sender<MasterTriggerEvent>, Receiver<MasterTriggerEvent>) = unbounded();
  let (mt_rate_to_main, mt_rate_from_mt)     : (Sender<u32>, Receiver<u32>) = unbounded();
  let (tp_to_main, tp_from_recv)   : (Sender<TofPacket>, Receiver<TofPacket>)       = unbounded();
  let (cmd_to_cmdr, cmd_from_main) : (Sender<TofCommand>, Receiver<TofCommand>)     = unbounded();
  let (rsp_to_main, rsp_from_cmdr) :
    (Sender<Vec<Option<TofResponse>>>, Receiver<Vec<Option<TofResponse>>>) = unbounded();
  //let ev_to_main, ev_from_thread) : Sender
  let (rb_id_to_receiver, rb_id_from_main) : (Sender<u8>, Receiver<u8>) = unbounded();


  // set up the terminal
  enable_raw_mode().expect("Unable to enter raw mode");
  let stdout       = io::stdout();
  let backend      = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;
  terminal.clear()?;
  
  let (tx, rx) = mpsc::channel();

  // heartbeat, keeps it going
  let _heartbeat_thread = thread::Builder::new()
    .name("heartbeat".into())
    .spawn(move || {
                     // change this to make it more/less 
                     // responsive
                     let tick_rate = Duration::from_millis(1);
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

  //let mut rb_list_state = ListState::default();
  //rb_list_state.select(Some(0));

  //  containers for the auto-updating data which will be shown 
  //  in the different widgets
  let mut stream_cache    = VecDeque::<TofPacket>::new();
  let mut mt_stream_cache = VecDeque::<MasterTriggerEvent>::new();
  let mut packets         = VecDeque::<String>::new();
  
  // containers for the values monitoring the MTB
  //let mut rates           = VecDeque::<(f64,f64)>::new();
  //let mut fpga_temps      = VecDeque::<(f64,f64)>::new();
  //let mut mtb_moni        = MtbMoniData::new();


  //let mut n_paddle_data   = VecDeque::<u8>::new();
  //let mut n_paddle_hist   = Histogram::<u64>::new_with_bounds(1, 160,1).unwrap();
  //let mut n_paddle_hist   = Histogram::with_buckets(160);

  // A color theme, can be changed later
  let mut color_theme     = ColorTheme2::new();
  color_theme.update(&COLORSETOMILU);
  //let mut color_set_bw    = 
  
  // The menus
  let mut ui_menu = MainMenu::new(color_theme.clone());
  let mut rb_menu = RBMenu::new(color_theme.clone());
  let mut mt_menu = MTMenu::new(color_theme.clone());
  let mut st_menu = SettingsMenu::new(color_theme.clone());

  let mut mt_tab2         = MTTab::new(mt_pack_recv,
                                       mte_recv,
                                       color_theme.clone());
  
  // waifu tab
  let mut wf_tab          = RBTab::new(rb_pack_recv,
                                       rbe_recv,
                                       color_theme.clone());
  let mut settings_tab    = SettingsTab::new(color_theme.clone());
  let mut home_tab        = HomeTab::new(color_theme.clone(), home_streamer, packet_map_home);
  let mut event_tab       = EventTab::new(ev_pack_recv, mte_send, rbe_send, color_theme);

  // FIXME - multithread it
  loop {
    match mt_tab2.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for MTTab!"),
      Ok(_)    => ()
    }
    match wf_tab.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for WfTab!"),
      Ok(_)    => ()
    }
    match event_tab.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for EventTab!"),
      Ok(_)    => ()
    }
    
    match rx.recv() {
      Err(err) => trace!("No update"),
      Ok(event) => {
        match event {
          Event::Input(ev) => {
            match ui_menu.active_menu_item {
              // if we are in the RBTab, 
              // route input accordingly
              MenuItem::Settings   => {
                match ev.code {
                  KeyCode::Char('h') => ui_menu.active_menu_item = MenuItem::Home,
                  KeyCode::Char('a') => {
                    settings_tab.ctl_active = true;
                    settings_tab.ctl_state.select(Some(0));
                  }
                  KeyCode::Up  => {
                    if settings_tab.ctl_active {
                      settings_tab.previous_ct();
                      match settings_tab.get_colorset() {
                        None => info!("Did not get a new colorset!"),
                        Some(cs) => {
                          st_menu.theme.update(&cs);
                          ui_menu.theme.update(&cs);
                          rb_menu.theme.update(&cs);
                          mt_menu.theme.update(&cs);
                          home_tab.theme.update(&cs);
                          event_tab.theme.update(&cs);
                          wf_tab.theme.update(&cs);
                          mt_tab2.theme.update(&cs);
                          settings_tab.theme.update(&cs);
                          color_theme.update(&cs);
                        }
                      }
                    }
                  },
                  KeyCode::Down => {
                    if settings_tab.ctl_active {
                      settings_tab.next_ct();
                      match settings_tab.get_colorset() {
                        None => info!("Did not get a new colorset!"),
                        Some(cs) => {
                          st_menu.theme.update(&cs);
                          ui_menu.theme.update(&cs);
                          rb_menu.theme.update(&cs);
                          mt_menu.theme.update(&cs);
                          home_tab.theme.update(&cs);
                          event_tab.theme.update(&cs);
                          wf_tab.theme.update(&cs);
                          mt_tab2.theme.update(&cs);
                          settings_tab.theme.update(&cs);
                          color_theme.update(&cs);
                        }
                      }
                    }
                  },
                  KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.clear()?;
                    terminal.show_cursor()?;
                    break;
                  },
                  _ => (),
                }
              },
              MenuItem::ReadoutBoards => {
                settings_tab.ctl_active = false;


                match ev.code {
                  KeyCode::Up  => {
                    if rb_menu.active_menu_item == RBMenuItem::SelectRB {
                      wf_tab.previous_rb();
                    }
                  },
                  KeyCode::Down => {
                    if rb_menu.active_menu_item == RBMenuItem::SelectRB {
                      wf_tab.next_rb();
                    }
                  },
                  KeyCode::Char('h') => {
                    ui_menu.active_menu_item = MenuItem::Home;
                    rb_menu.active_menu_item = RBMenuItem::Home;
                  },
                  KeyCode::Char('i') => {
                    rb_menu.active_menu_item = RBMenuItem::Info;
                    wf_tab.view = RBTabView::Info;
                  },
                  KeyCode::Char('r') => {
                    rb_menu.active_menu_item = RBMenuItem::RBMoniData;
                    wf_tab.view = RBTabView::RBMoniData;
                  },
                  KeyCode::Char('w') => {
                    rb_menu.active_menu_item = RBMenuItem::Waveforms;
                    wf_tab.view = RBTabView::Waveform;
                  },
                  KeyCode::Char('s') => {
                    rb_menu.active_menu_item = RBMenuItem::SelectRB;
                    wf_tab.view = RBTabView::SelectRB;
                  },
                  KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.clear()?;
                    terminal.show_cursor()?;
                    break;
                  },
                  _ => ()
                }
              },
              _ => {
                settings_tab.ctl_active = false;
                match ev.code {
                  // it seems we have to carry thos allong for every tab
                  KeyCode::Char('h') => ui_menu.active_menu_item = MenuItem::Home,
                  KeyCode::Char('t') => ui_menu.active_menu_item = MenuItem::TofEvents,
                  KeyCode::Char('r') => ui_menu.active_menu_item = MenuItem::ReadoutBoards,
                  KeyCode::Char('s') => ui_menu.active_menu_item = MenuItem::Settings,
                  KeyCode::Char('m') => ui_menu.active_menu_item = MenuItem::MasterTrigger,
                  KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.clear()?;
                    terminal.show_cursor()?;
                    break;
                  },
                  _ => trace!("Some other key pressed!"),
                }
              }
            } // end match ui_menu
          },
          Event::Tick => {
          }
        }
      }
    } // end rx.recv()
    // FIXME - terminal draw should run in its own thread
    terminal.draw(|rect| {
      let size           = rect.size();
      let mster_lo       = MasterLayout::new(size); 
      let w_logs         = render_logs(color_theme.clone());
      rect.render_widget(w_logs, mster_lo.rect[2]);
      match ui_menu.active_menu_item {
        MenuItem::Home => {
          ui_menu.render(&mster_lo.rect[0], rect);
          trace!("Rendering HomeTab!");
          home_tab.render(&mster_lo.rect[1], rect);
        },
        MenuItem::TofEvents => {
          ui_menu.render(&mster_lo.rect[0], rect);
          event_tab.render(&mster_lo.rect[1], rect);
        }
        MenuItem::MasterTrigger => {
          //rect.render_widget(ui_menu.tabs.clone(), mster_lo.rect[0]);
          mt_menu.render(&mster_lo.rect[0], rect);
          trace!("Rendering MasterTriggerTab!");
          mt_tab2.render(&mster_lo.rect[1], rect);
        },
        MenuItem::ReadoutBoards => {
          rb_menu.render(&mster_lo.rect[0], rect);
          trace!("Rendering RBTab!");
          wf_tab.render(&mster_lo.rect[1], rect);
        },
        MenuItem::Settings => {
          st_menu.render(&mster_lo.rect[0], rect);
          trace!("Rendering SettingsTab!");
          settings_tab.render(&mster_lo.rect[1], rect);
        },
        _ => {
          ui_menu.render(&mster_lo.rect[0], rect);
        }
      }

      //let mut cmd_tab    = CommandTab::new(mster_lo.rect[1],
      //                                     &packets,
      //                                     rsp_from_cmdr.clone(),
      //                                     cmd_to_cmdr.clone());
      ////let mut status_tab = StatusTab::new(mster_lo.rect[1],
      ////                                    &rb_list,
      ////                                    rb_list_state.clone());
      //match ui_menu.active_menu_item {
      //  MenuItem::MasterTrigger => {
      //    match rx.recv() {
      //      Err(err) => trace!("No update"),
      //      Ok(event) => {
      //        match event {
      //          Event::Input(ev) => {
      //            match ev.code {
      //              // it seems we have to carry thos allong for every tab
      //              KeyCode::Char('h') => ui_menu.active_menu_item = MenuItem::Home,
      //              KeyCode::Char('c') => ui_menu.active_menu_item = MenuItem::Commands,
      //              KeyCode::Char('r') => ui_menu.active_menu_item = MenuItem::Status,
      //              KeyCode::Char('m') => ui_menu.active_menu_item = MenuItem::MasterTrigger,
      //              _ => trace!("Some other key pressed!"),
      //            }
      //          },
      //          Event::Tick => {
      //            
      //            let mut event = MasterTriggerEvent::new(0,0);
      //            match mt_from_mt.try_recv() {
      //              Err(err) => {
      //                trace!("No event!");
      //              }
      //              Ok(pk)  => {
      //                event = pk;
      //                //let mut event = TofPacket::new();
      //                //event.packet_type = PacketType::RBEvent;
      //                // if the cache is too big, remove the oldest events
      //                //let new_tof_events = vec![event];
      //                mt_stream_cache.push_back(event);
      //                n_paddle_hist.add(event.get_hit_paddles().into());
      //                if mt_stream_cache.len() > STREAM_CACHE_MAX_SIZE {
      //                  mt_stream_cache.pop_front();
      //                }
      //                //for n in 0..mt_stream_cache.len() {
      //                //  let pretty = CommandTab::<'_>::get_pk_repr(&stream_cache[n]);
      //                //  packets.push_back(pretty);
      //                //}
      //              }
      //            }
      //          },
      //        }
      //      }
      //    }    
      //    let update_detail = ten_second_update.elapsed().as_secs() > 10;
      //    
      //    //monitor_mtb(&mtb_address, &mut mtb_moni);
      //    rates.push_back((mission_elapsed_time.elapsed().as_secs() as f64, mtb_moni.rate as f64));
      //    //fpga_temps.push_back((mission_elapsed_time.elapsed().as_secs() as f64, mtb_moni.fpga_temp as f64));
      //    info!("Received MtbMoniData {}", mtb_moni); 
      //    if update_detail {
      //        warn!("Ten seconds have passed!");
      //    }

      //    if rates.len() > MAX_LEN_RATE {
      //      rates.pop_front();
      //    }

      //    if fpga_temps.len() > MAX_LEN_MT_FPGA_TEMP {
      //      fpga_temps.pop_front();
      //    }

      //    info!("Rate chart with {} entries", rates.len());
      //    let mut x_labels = Vec::<String>::new();
      //    let mut y_labels = Vec::<String>::new();
      //    let mut r_min : i64 = 0;
      //    let mut r_max : i64 = 0;
      //    let mut t_min : i64 = 0;
      //    let mut t_max : i64 = 0;
      //    if rates.len() > 0 {
      //      //let max_rate = rates.iter().max_by(|x,y| x.1.cmp(y.1)).unwrap();
      //      let r_only : Vec::<i64> = rates.iter().map(|z| z.1.round() as i64).collect();
      //      r_max = *r_only.iter().max().unwrap() + 5;
      //      r_min = *r_only.iter().min().unwrap() - 5;
      //      let y_spacing = (r_max - r_min)/5;
      //      y_labels = vec![r_min.to_string(),
      //                     (r_min + y_spacing).to_string(),
      //                     (r_min + 2*y_spacing).to_string(),
      //                     (r_min + 3*y_spacing).to_string(),
      //                     (r_min + 4*y_spacing).to_string(),
      //                     (r_min + 5*y_spacing).to_string()];
      //      let t_only : Vec::<i64> = rates.iter().map(|z| z.0.round() as i64).collect();
      //      t_max = *t_only.iter().max().unwrap();
      //      t_min = *t_only.iter().min().unwrap();
      //      let x_spacing = (t_max - t_min)/5;
      //      x_labels = vec![t_min.to_string(),
      //                     (t_min + x_spacing).to_string(),
      //                     (t_min + 2*x_spacing).to_string(),
      //                     (t_min + 3*x_spacing).to_string(),
      //                     (t_min + 4*x_spacing).to_string(),
      //                     (t_min + 5*x_spacing).to_string()];

      //    }
      //    //println!("{:?}", rates.make_contiguous()); 
      //    let rate_dataset = vec![Dataset::default()
      //        .name("MTB Rate")
      //        .marker(symbols::Marker::Braille)
      //        .graph_type(GraphType::Line)
      //        .style(Style::default().fg(Color::White))
      //        .data(rates.make_contiguous())];
      //    let rate_chart = Chart::new(rate_dataset)
      //      .block(
      //        Block::default()
      //          .borders(Borders::ALL)
      //          .style(Style::default().fg(Color::White))
      //          .title("MT rate ".to_owned() )
      //          .border_type(BorderType::Double),
      //      )
      //      .x_axis(Axis::default()
      //        .title(Span::styled("MET [s]", Style::default().fg(Color::White)))
      //        .style(Style::default().fg(Color::White))
      //        .bounds([t_min as f64, t_max as f64])
      //        //.bounds([0.0, 1000.0])
      //        .labels(x_labels.clone().iter().cloned().map(Span::from).collect()))
      //      .y_axis(Axis::default()
      //        .title(Span::styled("Hz", Style::default().fg(Color::White)))
      //        .style(Style::default().fg(Color::White))
      //        .bounds([r_min as f64, r_max as f64])
      //        //.bounds([0.0,1000.0])
      //        .labels(y_labels.clone().iter().cloned().map(Span::from).collect()));
      //    
      //    info!("MT FPGA T chart with {} entries", fpga_temps.len());
      //    let mut fpga_y_labels = Vec::<String>::new();
      //    let mut fpga_t_min : i64 = 0;
      //    let mut fpga_t_max : i64 = 0;
      //    if fpga_temps.len() > 0 {
      //      //let max_rate = rates.iter().max_by(|x,y| x.1.cmp(y.1)).unwrap();
      //      let fpga_only : Vec::<i64> = fpga_temps.iter().map(|z| z.1.round() as i64).collect();
      //      fpga_t_max = *fpga_only.iter().max().unwrap() + 5;
      //      fpga_t_min = *fpga_only.iter().min().unwrap() - 5;
      //      let y_spacing = (fpga_t_max - fpga_t_min)/5;
      //      y_labels = vec![fpga_t_min.to_string(),
      //                     (fpga_t_min + y_spacing).to_string(),
      //                     (fpga_t_min + 2*y_spacing).to_string(),
      //                     (fpga_t_min + 3*y_spacing).to_string(),
      //                     (fpga_t_min + 4*y_spacing).to_string(),
      //                     (fpga_t_min + 5*y_spacing).to_string()];
      //    }
      //    let fpga_temp_dataset = vec![Dataset::default()
      //        .name("FPGA T")
      //        .marker(symbols::Marker::Braille)
      //        .graph_type(GraphType::Line)
      //        .style(Style::default().fg(Color::White))
      //        .data(fpga_temps.make_contiguous())];
      //    let fpga_temp_chart = Chart::new(fpga_temp_dataset)
      //      .block(
      //        Block::default()
      //          .borders(Borders::ALL)
      //          .style(Style::default().fg(Color::White))
      //          .title("FPGA T [\u{00B0}C] ".to_owned() )
      //          .border_type(BorderType::Double),
      //      )
      //      .x_axis(Axis::default()
      //        .title(Span::styled("MET [s]", Style::default().fg(Color::White)))
      //        .style(Style::default().fg(Color::White))
      //        .bounds([t_min as f64, t_max as f64])
      //        //.bounds([0.0, 1000.0])
      //        .labels(x_labels.clone().iter().cloned().map(Span::from).collect()))
      //      .y_axis(Axis::default()
      //        //.title(Span::styled("T [\u{00B0}C]", Style::default().fg(Color::White)))
      //        .style(Style::default().fg(Color::White))
      //        .bounds([fpga_t_min as f64, fpga_t_max as f64])
      //        //.bounds([0.0,1000.0])
      //        .labels(y_labels.clone().iter().cloned().map(Span::from).collect()));
      //    
      //    //print!("{} {} {} {}", t_min, t_max, r_min, r_max);
      //    match mt_tab.update(&mt_stream_cache, update_detail) {
      //      None => (),
      //      Some(val) => detail_string = Some(val)
      //    }
      //    let mut max_pop_bin = 0;
      //    let mut vec_index   = 0;
      //    let mut bins = Vec::<(u64, u64)>::new();
      //    for bucket in n_paddle_hist.buckets() {
      //      bins.push((vec_index, bucket.count()));
      //      if bucket.count() > 0 {
      //        max_pop_bin = vec_index;
      //      }
      //      vec_index += 1;
      //      //do_stuff(bucket.start(), bucket.end(), bucket.count());
      //    }
      //    bins.retain(|&(x,y)| x <= max_pop_bin);
      //    let mut bins_for_bc = Vec::<(&str, u64)>::new();
      //    //let mut label;
      //    let mut labels = Vec::<&str>::with_capacity(160);
      //    let mut n_iter = 0;
      //    debug!("bins: {:?}", bins);
      //    for n in bins.iter() {
      //      bins_for_bc.push((hist_labels[n_iter], n.1));
      //      //bins_for_bc.push((foo, n.1));
      //      n_iter += 1;
      //    }

      //    let n_paddle_dist = BarChart::default()
      //        .block(Block::default().title("N Paddle").borders(Borders::ALL))
      //        .data(bins_for_bc.as_slice())
      //        .bar_width(1)
      //        .bar_gap(1)
      //        .bar_style(Style::default().fg(Color::Blue))
      //        .value_style(
      //            Style::default()
      //                .bg(Color::Blue)
      //                .add_modifier(Modifier::BOLD),
      //        );

      //    //rect.render_stateful_widget(mt_tab.list_widget, mt_tab.list_rect, &mut rb_list_state);
      //    rect.render_widget(rate_chart,         mt_tab.rate_rect); 
      //    rect.render_widget(fpga_temp_chart,    mt_tab.fpga_t_rect); 
      //    rect.render_widget(mt_tab.stream,      mt_tab.stream_rect);
      //   // rect.render_widget(mt_tab.network_moni, mt_tab.paddle_dist_rect); 
      //    rect.render_widget(n_paddle_dist,      mt_tab.paddle_dist_rect);
      //    rect.render_widget(mt_tab.detail,      mt_tab.detail_rect); 
      //    if update_detail {
      //      ten_second_update = Instant::now();
      //    }
      //    info!("Updating MasterTrigger tab!");
      //  },
      //  //MenuItem::Commands => {
      //  //  match rx.recv() {
      //  //    Err(err) => trace!("No update"),
      //  //    Ok(event) => {
      //  //      match event {
      //  //        Event::Input(ev) => {
      //  //          match ev.code {
      //  //            // it seems we have to carry thos allong for every tab
      //  //            KeyCode::Char('h') => ui_menu.active_menu_item = MenuItem::Home,
      //  //            KeyCode::Char('c') => ui_menu.active_menu_item = MenuItem::Commands,
      //  //            KeyCode::Char('r') => ui_menu.active_menu_item = MenuItem::Status,
      //  //            KeyCode::Char('m') => ui_menu.active_menu_item = MenuItem::MasterTrigger,
      //  //            KeyCode::Down => {
      //  //              if let Some(selected) = rb_list_state.selected() {
      //  //                if selected >= cmd_tab.cmd_list.len() -1 {
      //  //                  rb_list_state.select(Some(0));
      //  //                } else {
      //  //                  rb_list_state.select(Some(selected + 1));
      //  //                }
      //  //              }
      //  //            }
      //  //            KeyCode::Up => {
      //  //              if let Some(selected) = rb_list_state.selected() {
      //  //                info!("Attempting to select item {selected}");
      //  //                if selected < 1 {
      //  //                    rb_list_state.select(Some(0));
      //  //                } else {
      //  //                    rb_list_state.select(Some(selected - 1));
      //  //                }
      //  //              }
      //  //            }

      //  //            KeyCode::Enter => {
      //  //              if matches!(ui_menu.active_menu_item, MenuItem::Commands) {
      //  //                info!("Enter pressed, will send highlighted tof command!");
      //  //                warn!("This is not yet implemented!");
      //  //                if let Some(selected) = rb_list_state.selected() {
      //  //                  // We hope (it *should* be) that the command list vector 
      //  //                  // and the actual command vector are aligned
      //  //                  let this_command = cmd_tab.cmd_list[selected];
      //  //                  match cmd_to_cmdr.send(this_command) {
      //  //                    Err(err) => warn!("There was a problem sending the command!"),
      //  //                    Ok(_)    => info!("Command sent!")
      //  //                  }
      //  //                }
      //  //              }
      //  //            },
      //  //            _ => trace!("Some other key pressed!"),
      //  //          }
      //  //        },
      //  //        Event::Tick => {
      //  //          let foo : String = "Tick :".to_owned() + &tick_count.to_string();
      //  //          
      //  //          // check the zmq socket
      //  //          let mut event = TofPacket::new();
      //  //          let mut last_response = Vec::<Option<TofResponse>>::new();

      //  //          match rsp_from_cmdr.try_recv() {
      //  //            Err(err)     => trace!("No response!"),
      //  //            Ok(response) => {
      //  //              last_response = response;             
      //  //            }
      //  //          }
      //  //          match tp_from_recv.try_recv() {
      //  //            Err(err) => {
      //  //              trace!("No event!");
      //  //            }
      //  //            Ok(pk)  => {
      //  //              //event = pk;
      //  //              //let mut event = TofPacket::new();
      //  //              //event.packet_type = PacketType::RBEvent;
      //  //              // if the cache is too big, remove the oldest events
      //  //              //let new_tof_events = vec![event];
      //  //              //stream_cache.push_back(event);
      //  //              //if stream_cache.len() > STREAM_CACHE_MAX_SIZE {
      //  //              //  stream_cache.pop_front();
      //  //              //  packets.pop_front(); 
      //  //              //}
      //  //              let string_repr = CommandTab::<'_>::get_pk_repr(&pk);
      //  //              packets.push_back(string_repr);
      //  //              if packets.len() > STREAM_CACHE_MAX_SIZE {
      //  //                packets.pop_front();
      //  //              }
      //  //              //for n in 0..stream_cache.len() {
      //  //              //  let foo = CommandTab::<'_>::get_pk_repr(&stream_cache[n]);
      //  //              //  packets.push_back(foo);
      //  //              //}
      //  //              info!("Updating Command tab!");
      //  //              cmd_tab.update(&packets,
      //  //                             &last_response);
      //  //            }
      //  //          }
      //  //        },
      //  //      }
      //  //    }
      //  //  }    

      //  //  rect.render_stateful_widget(cmd_tab.list_widget, cmd_tab.list_rect, &mut rb_list_state);
      //  //  rect.render_widget(cmd_tab.tof_resp, cmd_tab.rsp_rect); 
      //  //  rect.render_widget(cmd_tab.stream,   cmd_tab.stream_rect);
      //  //}

      //  //MenuItem::Status => {
      //  //  
      //  //  match rx.recv() {
      //  //    Err(err) => trace!("No update"),
      //  //    Ok(event) => {
      //  //      match event {
      //  //        Event::Input(ev) => {
      //  //          match ev.code {
      //  //            KeyCode::Down => {
      //  //              if let Some(selected) = rb_list_state.selected() {
      //  //                let mut select_board = selected;
      //  //                if selected >= rb_list.len() {
      //  //                  rb_list_state.select(Some(0));
      //  //                  select_board = 0;
      //  //                } else {
      //  //                  rb_list_state.select(Some(selected + 1));
      //  //                }
      //  //              rb_id_to_receiver.send(rb_list[selected].rb_id);
      //  //              }
      //  //            }
      //  //            KeyCode::Up => {
      //  //              if let Some(selected) = rb_list_state.selected() {
      //  //                let mut select_board = selected;
      //  //                if selected > rb_list.len() {
      //  //                    rb_list_state.select(Some(selected - 1));
      //  //                    select_board = 0;
      //  //                } else {
      //  //                    rb_list_state.select(Some(rb_list.len() - 1));
      //  //                }
      //  //                rb_id_to_receiver.send(rb_list[selected -1].rb_id);

      //  //              }
      //  //            }
      //  //          _ => trace!("Some other key pressed!"),
      //  //          }
      //  //        },
      //  //        Event::Tick => (),
      //  //      }
      //  //    }
      //  //  }

      //  //  let empty_data = vec![(0.0,0.0);1024]; 
      //  //  let mut data = vec![empty_data;9];
      //  //  let mut update_channels = false;
      //  //  match tp_from_recv.try_recv() {
      //  //    Err(err) => {trace!("Did not receive new data!");},
      //  //    Ok(dt)   => {
      //  //      if dt.packet_type == PacketType::RBEvent {
      //  //        data = Vec::<Vec<(f64,f64)>>::new();
      //  //        let mut event = RBEventMemoryView::new();
      //  //        let mut pos = 0usize;
      //  //        event = RBEventMemoryView::from_bytestream(&dt.payload, &mut pos).unwrap();
      //  //        if event.ch_adc.len() == 9 {
      //  //          for n in 0..9 {
      //  //            data.push(Vec::<(f64,f64)>::new());
      //  //            let adc = event.ch_adc[n];
      //  //            for j in 0..adc.len() {
      //  //              data[n].push((j as f64, adc[j] as f64));
      //  //            }
      //  //            update_channels = true;
      //  //          }
      //  //        }
      //  //      }
      //  //    }
      //  //  }
      //  //  let xlabels = vec!["0", "200", "400", "600", "800", "1000"];
      //  //  let ylabels = vec!["0","50", "100"];
      //  //  //let cdata = data.clone();


      //  //  let datasets = vec![
      //  //    Dataset::default()
      //  //      .name("Ch1")
      //  //      .marker(symbols::Marker::Braille)
      //  //      .graph_type(GraphType::Line)
      //  //      .style(Style::default().fg(Color::White))
      //  //      .data(&data[0]),
      //  //    Dataset::default()
      //  //      .name("Ch2")
      //  //      .marker(symbols::Marker::Braille)
      //  //      .graph_type(GraphType::Line)
      //  //      .style(Style::default().fg(Color::White))
      //  //      .data(&data[1]),
      //  //    Dataset::default()
      //  //      .name("Ch3")
      //  //      .marker(symbols::Marker::Braille)
      //  //      .graph_type(GraphType::Line)
      //  //      .style(Style::default().fg(Color::White))
      //  //      .data(&data[2]),
      //  //    Dataset::default()
      //  //      .name("Ch4")
      //  //      .marker(symbols::Marker::Braille)
      //  //      .graph_type(GraphType::Line)
      //  //      .style(Style::default().fg(Color::White))
      //  //      .data(&data[3]),
      //  //    Dataset::default()
      //  //      .name("Ch5")
      //  //      .marker(symbols::Marker::Braille)
      //  //      .graph_type(GraphType::Line)
      //  //      .style(Style::default().fg(Color::White))
      //  //      .data(&data[4]),
      //  //    Dataset::default()
      //  //      .name("Ch6")
      //  //      .marker(symbols::Marker::Braille)
      //  //      .graph_type(GraphType::Line)
      //  //      .style(Style::default().fg(Color::White))
      //  //      .data(&data[5]),
      //  //    Dataset::default()
      //  //      .name("Ch7")
      //  //      .marker(symbols::Marker::Braille)
      //  //      .graph_type(GraphType::Line)
      //  //      .style(Style::default().fg(Color::White))
      //  //      .data(&data[6]),
      //  //    Dataset::default()
      //  //      .name("Ch8")
      //  //      .marker(symbols::Marker::Braille)
      //  //      .graph_type(GraphType::Line)
      //  //      .style(Style::default().fg(Color::White))
      //  //      .data(&data[7]),
      //  //    Dataset::default()
      //  //      .name("Ch9")
      //  //      .marker(symbols::Marker::Braille)
      //  //      .graph_type(GraphType::Line)
      //  //      .style(Style::default().fg(Color::White))
      //  //      .data(&data[8]),
      //  //  ];
      //  //  
      //  //  let mut charts  = Vec::<Chart>::new();
      //  //  for n in 0..datasets.len() {
      //  //    let this_chart_dataset = vec![datasets[n].clone()];
      //  //    let chart = Chart::new(this_chart_dataset)
      //  //    .block(
      //  //      Block::default()
      //  //        .borders(Borders::ALL)
      //  //        .style(Style::default().fg(Color::White))
      //  //        .title("Ch ".to_owned() + &n.to_string() )
      //  //        .border_type(BorderType::Plain),
      //  //    )
      //  //    .x_axis(Axis::default()
      //  //      .title(Span::styled("bin", Style::default().fg(Color::White)))
      //  //      .style(Style::default().fg(Color::White))
      //  //      .bounds([0.0, 1024.0])
      //  //      .labels(xlabels.clone().iter().cloned().map(Span::from).collect()))
      //  //    .y_axis(Axis::default()
      //  //      .title(Span::styled("ADC", Style::default().fg(Color::White)))
      //  //      .style(Style::default().fg(Color::White))
      //  //      .bounds([0.0, 17000.0])
      //  //      .labels(ylabels.clone().iter().cloned().map(Span::from).collect()));
      //  //    charts.push(chart);
      //  //  }
      //  //  
      //  //  rect.render_stateful_widget(status_tab.list_widget, status_tab.list_rect, &mut rb_list_state);
      //  //  rect.render_widget(status_tab.detail, status_tab.detail_rect); 
      //  //  if update_channels { 
      //  //    for n in 0..status_tab.ch_rect.len() {
      //  //      rect.render_widget(charts[n].clone(), status_tab.ch_rect[n]);
      //  //    }
      //  //  }
      //  //  // chart for "ch9"
      //  //  let ch9 = vec![ Dataset::default()
      //  //      .name("Ch8 ('Ninth')")
      //  //      .marker(symbols::Marker::Braille)
      //  //      .graph_type(GraphType::Line)
      //  //      .style(Style::default().fg(Color::Magenta))
      //  //      .data(&data[8])
      //  //  ];
      //  //  let ch9_chart = Chart::new(ch9)
      //  //    .block(
      //  //      Block::default()
      //  //        .borders(Borders::ALL)
      //  //        .style(Style::default().fg(Color::White))
      //  //        .title("Ch 9")
      //  //        .border_type(BorderType::Plain),
      //  //    )
      //  //    .x_axis(Axis::default()
      //  //      .title(Span::styled("bin", Style::default().fg(Color::White)))
      //  //      .style(Style::default().fg(Color::White))
      //  //      .bounds([0.0, 1024.0])
      //  //      .labels(xlabels.clone().iter().cloned().map(Span::from).collect()))
      //  //    .y_axis(Axis::default()
      //  //      .title(Span::styled("ADC", Style::default().fg(Color::White)))
      //  //      .style(Style::default().fg(Color::White))
      //  //      .bounds([0.0, 17000.0])
      //  //      .labels(ylabels.clone().iter().cloned().map(Span::from).collect()));
      //  //  rect.render_widget(ch9_chart.clone(), status_tab.ch9_rect);
      //  //  



      //  //  //return charts;
      //  //  //self.ch_charts = charts;
      //  //  //}
      //  //  //for n in 0..status_tab.ch_rect.len() {
      //  //  //  rect.render_widget(status_tab.ch_charts[n].clone(), status_tab.ch_rect[n]);
      //  //  //}
      //  //  //let status_chunks = Layout::default()
      //  //  //  .direction(Direction::Horizontal)
      //  //  //  .constraints(
      //  //  //      [Constraint::Percentage(10), Constraint::Percentage(20), Constraint::Percentage(70)].as_ref(),
      //  //  //  )
      //  //  //  .split(mster_lo.rect[1]);
      //  //  //let ch_chunks = Layout::default()
      //  //  //  .direction(Direction::Vertical)
      //  //  //  .constraints(
      //  //  //      [Constraint::Percentage(11),
      //  //  //       Constraint::Percentage(11),
      //  //  //       Constraint::Percentage(11),
      //  //  //       Constraint::Percentage(11),
      //  //  //       Constraint::Percentage(11),
      //  //  //       Constraint::Percentage(11),
      //  //  //       Constraint::Percentage(11),
      //  //  //       Constraint::Percentage(11),
      //  //  //       Constraint::Percentage(12)].as_ref(),
      //  //  //  )
      //  //  //  .split(status_chunks[2]);
      //  //  //let (left, center, mut right) = render_status(rb_list_state.clone(), rb_list.clone());
      //  //  ////let (left, center, mut right) = render_status(rb_list.clone());
      //  //  //rect.render_stateful_widget(left, status_chunks[0], &mut rb_list_state);
      //  //  //rect.render_widget(center, status_chunks[1]);
      //  //  //for n in 0..ch_chunks.len() - 1 {
      //  //  //  let ch = right.remove(0);
      //  //  //  rect.render_widget(ch, ch_chunks[n]);
      //  //  //}
      //  //},
      //  _ => (),
      //} 

    }); // end terminal.draw
    //match rx.recv()? {
    //  Event::Tick => {
    //    match ui_menu.active_menu_item {
    //      MenuItem::Commands => {
    //      },
    //      _ => ()
    //    }
    //  },
    //  Event::Input(event) => {
    //    match event.code {
    //      KeyCode::Char('q') => {
    //          disable_raw_mode()?;
    //          terminal.clear()?;
    //          terminal.show_cursor()?;
    //          break;
    //      },
    //      KeyCode::Char('h') => ui_menu.active_menu_item = MenuItem::Home,
    //      KeyCode::Char('c') => ui_menu.active_menu_item = MenuItem::Commands,
    //      KeyCode::Char('r') => ui_menu.active_menu_item = MenuItem::Status,
    //      KeyCode::Char('m') => ui_menu.active_menu_item = MenuItem::MasterTrigger,
    //      //KeyCode::Down => {
    //      //  if let Some(selected) = rb_list_state.selected() {
    //      //    //let amount_pets = read_db().expect("can fetch pet list").len();
    //      //    let max_rb = 40;
    //      //    if selected >= rb_list.len() {
    //      //      rb_list_state.select(Some(0));
    //      //    } else {
    //      //      rb_list_state.select(Some(selected + 1));
    //      //    }
    //      //  }
    //      //}
    //      //KeyCode::Up => {
    //      //  if let Some(selected) = rb_list_state.selected() {
    //      //    //let amount_pets = read_db().expect("can fetch pet list").len();
    //      //    let max_rb = 40;
    //      //    if max_rb > 0 {
    //      //        rb_list_state.select(Some(selected - 1));
    //      //    } else {
    //      //        rb_list_state.select(Some(rb_list.len() - 1));
    //      //    }
    //      //  }
    //      //}
    //      _ => (),
    //    }
    //  }
    //}
  } // end loop;
  Ok(())
}
