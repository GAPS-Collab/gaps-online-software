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

extern crate json;

extern crate histo;

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
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{
        Block, Borders, },
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
  let mut ui_menu = MainMenu::new(color_theme.clone());
  let mut rb_menu = RBMenu::new(color_theme.clone());
  let mut mt_menu = MTMenu::new(color_theme.clone());
  let mut st_menu = SettingsMenu::new(color_theme.clone());

  let mut mt_tab2         = MTTab::new(mt_pack_recv,
                                       mte_recv,
                                       color_theme.clone());
 
  let mut cpu_tab         = CPUTab::new(cp_pack_recv,
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
      Err(err) => error!("Can not receive TofPackets for MTTab! {err}"),
      Ok(_)    => ()
    }
    match wf_tab.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for WfTab! {err}"),
      Ok(_)    => ()
    }
    match event_tab.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for EventTab! {err}"),
      Ok(_)    => ()
    }
    match cpu_tab.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for CPUTab! {err}"),
      Ok(_)    => ()
    }
    
    match rx.recv() {
      Err(err) => trace!("Err - no update! {err}"),
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
                          cpu_tab.theme.update(&cs);
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
                          cpu_tab.theme.update(&cs);
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
                  KeyCode::Char('c') => ui_menu.active_menu_item = MenuItem::TOFCpu,
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
    match terminal.draw(|rect| {
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
        },
        MenuItem::TOFCpu => { 
          ui_menu.render(&mster_lo.rect[0], rect);
          cpu_tab.render(&mster_lo.rect[1], rect);
        },
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
    }) {
      Err(err) => error!("Can't render terminal! {err}"),
      Ok(_)    => () ,
    }
    // end terminal.draw
  } // end loop;
  Ok(())
}
