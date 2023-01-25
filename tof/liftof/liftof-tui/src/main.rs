//! This follows the example for the pet database.
//!
//! The idea is to have a separate, hearbeat style
//! thread which either processes user input or
//! triggers the app to move on.
//!
//!
//!
//!
//!
//!

mod tab_commands;
mod tab_mt;
mod tab_status;
mod menu;

use chrono::prelude::*;
use thiserror::Error;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use std::collections::VecDeque;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

use liftof_lib::{get_rb_manifest,
                 ReadoutBoard};

use std::sync::mpsc;
use std::thread;

use std::time::{Duration, Instant};

use tui_logger::TuiLoggerWidget;

use std::io;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

extern crate crossbeam_channel;
use crossbeam_channel::{unbounded,
                        Sender,
                        Receiver};


use tui::{
    symbols,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    terminal::Frame,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{
        Block, Dataset, Axis, GraphType, BorderType, Chart, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,    },
    Terminal,
};

// system inforamtion
use sysinfo::{NetworkExt, NetworksExt, ProcessExt, System, SystemExt};


use tof_dataclasses::commands::{TofCommand, TofResponse};
use tof_dataclasses::packets::{TofPacket, PacketType};
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::events::blob::BlobData;

use crate::tab_commands::CommandTab;
use crate::tab_mt::MTTab;
use crate::tab_status::StatusTab;
use crate::menu::{MenuItem, Menu};

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
}

enum Event<I> {
    Input(I),
    Tick,
}

//#[derive(Serialize, Deserialize, Clone)]
//#[derive(Debug, Clone)]
//struct ReadoutBoard {
//  pub id: usize,
//  pub name: String,
//  //category: String,
//  //age: usize,
//  //created_at: DateTime<Utc>,
//}

//impl ReadoutBoard {
//  fn new() -> ReadoutBoard {
//    ReadoutBoard {
//      id   : 0,
//      name : String::from("ReadoutBoard")
//    }
//  }
//}

/// Communicate with the readoutboards on the 
/// CMD channel 
///
/// Opens 0MQ socket for the individual RB's
fn commander(cmd_from_main : Receiver<TofCommand>,
             rsp_to_main   : Sender<Vec<Option<TofResponse>>>,
             rb_list       : Vec<ReadoutBoard>) {

  // connect to the rb's 
  let ctx = zmq::Context::new();  
  // how zmq works is that one req socket can connect to multiple REP and 
  // broadcast the messages
  let cmd_socket = ctx.socket(zmq::REQ).expect("Unable to create 0MQ SUB socket!");
  let mut connected_rbs : u8 = 0;
  for rb in rb_list.iter() {
    if rb.ip_address.is_none() || rb.cmd_port.is_none() {
      warn!("This rb has no connection information");
      continue;
    }
    let address = "tcp::/".to_owned()
                  + &rb.ip_address.expect("No IP known for this board!").to_string()
                  + ":"
                  +  &rb.cmd_port.expect("No CMD port known for this board!").to_string();
    cmd_socket.connect(&address);
    // the process is only completed after an intiail back and forth
    //let ping : String = String::from("[PING]");
    //// we use expect here, since these calls have 
    //// to go through, otherwise it just won't work
    //cmd_socket.send(ping.as_bytes(), 0).expect("Can not communicate with RB!");
    //let response = cmd_socket.recv_bytes(0).expect("Can not communicate with RB!");
    //info!("Got response {}", String::from_utf8(response).expect("Did not receive string"));
    connected_rbs += 1;
    
  }
  if connected_rbs == 0 {
    panic!("I can not connect to any readout boards! Either auto-discovery did not discover them or none are (physically) connected!");
  }
  loop {
    let mut responses = Vec::<Option<TofResponse>>::new();
    match cmd_from_main.recv() {
      Err(err) => trace!("Did not get any response, err {err}"),
      Ok(cmd)  => {
        // the 0 in the send/recv section means 
        // it should wait (in contrast to zmq::DONTWAIT)
        // how this works is that we have to go through 
        // the connected boards 1 by 1 and do our 
        // send/recv spiel.
        // We will get one response per board
        cmd_socket.send(&cmd.to_bytestream(), 0);
        let resp = cmd_socket.recv_bytes(0);
        match resp {
          Err(err) => debug!("0MQ problem, can not receive response from RB!"),
          Ok(r)    => {
            let tof_response = TofResponse::from_bytestream(&r, 0).ok();
            responses.push(tof_response);
          }
        }
      }
    }
  if responses.len() != 0 {
    rsp_to_main.send(responses);
  }
  }// end loop
}


/// Receive the data stream and forward 
/// it to a widget
fn receive_stream(tp_to_main : Sender<TofPacket>,
                  rb_list    : Vec<ReadoutBoard>) {
      
  let ctx = zmq::Context::new();  
  let data_socket = ctx.socket(zmq::SUB).expect("Unable to create 0MQ SUB socket!");

  for rb in rb_list.iter() {
    match rb.ip_address {
      None => {continue;},
      Some(ip) => {
        //let mut address_ip = String::from("tcp://127.0.0.1");
        //let data_port : u32 = 40000;
        //let data_address : String = address_ip + ":" + &data_port.to_string();
        let mut address_ip = "tcp://".to_owned() + &rb.ip_address.unwrap().to_string();
        match rb.data_port {
          None => {continue;}
          Some(port) => {
            address_ip += &":".to_owned();
            address_ip += &port.to_string();
            data_socket.connect(&address_ip);
            info!("0MQ SUB socket connected to address {address_ip}");
          }
        }
      }
    }
  }
  let topic = b"";
  data_socket.set_subscribe(topic);
  let recv_rate = Duration::from_millis(5);

  loop {
    // reduce the heat and take it easy
    //thread::sleep(recv_rate);
    match data_socket.recv_bytes(zmq::DONTWAIT) {
      Err(err) => trace!("[zmq] Nothing to receive/err {err}"),
      Ok(msg)  => {
        info!("[zmq] SUB - got msg of size {}", msg.len());
        let packet = TofPacket::from_bytestream(&msg, 0);
        match packet {
          Err(err) => { 
            warn!("Can't unpack packet!");
          },
          Ok(pk) => {
            match tp_to_main.try_send(pk) {
              Err(err) => warn!("Can't send packet!"),
              Ok(_)    => info!("Done"),
            }
          }
        }
      }
    }
  }
}


/// Use the TuiLoggerWidget to display 
/// the most recent log messages
///
///
fn render_logs<'a>() -> TuiLoggerWidget<'a> {
  TuiLoggerWidget::default()
    .style_error(Style::default().fg(Color::Red))
    .style_debug(Style::default().fg(Color::Green))
    .style_warn(Style::default().fg(Color::Yellow))
    .style_trace(Style::default().fg(Color::Gray))
    .style_info(Style::default().fg(Color::Blue))
    .block(
      Block::default()
        .title("Logs")
        .border_style(Style::default().fg(Color::White).bg(Color::Black))
        .borders(Borders::ALL),
    )   
    .style(Style::default().fg(Color::White).bg(Color::Black))
}

#[derive(Debug, Clone)]
struct MasterLayout {
  
  pub rect : Vec<Rect>

}

impl MasterLayout {

  fn new(size : Rect) -> MasterLayout {
    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .margin(1)
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
      rect : chunks
    }
  }
}



fn main () -> Result<(), Box<dyn std::error::Error>>{


  // Set max_log_level to Trace
  match tui_logger::init_logger(log::LevelFilter::Info) {
    Err(err) => panic!("Something bad just happened {err}"),
    Ok(_)    => (),
  }
  // Set default level for unknown targets to Trace
  tui_logger::set_default_level(log::LevelFilter::Trace);
  
  let args = Args::parse();                   
  let debug_local       = args.debug_local;         
  let autodiscover_rb   = args.autodiscover_rb;    
  
  //pretty_env_logger::init();

  let mut rb_list = Vec::<ReadoutBoard>::new();
  let mut tick_count = 0;
  if debug_local {
    let mut rb = ReadoutBoard::default();
    rb.ip_address = Some(Ipv4Addr::new(127, 0, 0, 1));
    rb.cmd_port   = Some(30000);
    rb.data_port  = Some(40000);
    rb.id = Some(0);
    rb_list = vec![rb];

    // make sure the rb is connected
    rb.ping().unwrap();
  }  
  if autodiscover_rb {
    rb_list = get_rb_manifest();
    if rb_list.len() == 0 {
      println!("Could not discover boards, inserting dummy");
      let mut rb = ReadoutBoard::default();
      rb.id = Some(0);
      rb_list = vec![rb];
    }
  }
  let rb_list_c  = rb_list.clone();
  let rb_list_c2 = rb_list.clone();
  // first set up comms etc. before 
  // we go into raw_mode, so we can 
  // see the log messages during setup
  let (tp_to_main, tp_from_recv)   : (Sender<TofPacket>, Receiver<TofPacket>)       = unbounded();
  let (cmd_to_cmdr, cmd_from_main) : (Sender<TofCommand>, Receiver<TofCommand>)     = unbounded();
  let (rsp_to_main, rsp_from_cmdr) :
    (Sender<Vec<Option<TofResponse>>>, Receiver<Vec<Option<TofResponse>>>) = unbounded();
  //let ev_to_main, ev_from_thread) : Sender
  println!("We have the following ReadoutBoards");
  for n in 0..rb_list_c2.len() {
    println!("{}",rb_list_c2[n]);
  }
  println!("Starting threads");
  // set up Threads
  let n_threads = 2;
  let workforce = ThreadPool::new(n_threads);
  workforce.execute(move || {
      commander(cmd_from_main,
                rsp_to_main,
                rb_list_c2);
  });
  workforce.execute(move || {
      receive_stream(tp_to_main, rb_list_c);
  });

  //panic!("Until here");

  // set up the terminal
  enable_raw_mode().expect("can run in raw mode");
  let stdout = io::stdout();
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;
  terminal.clear()?;

  
  let (tx, rx) = mpsc::channel();

  // change this to make it more/less 
  // responsive
  let tick_rate = Duration::from_millis(100);
  
  // heartbeat, keeps it going
  thread::spawn(move || {
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
  }); 


  //let menu_titles = vec!["Home", "RBStatus", "Commands", "Alerts", "Dashboard", "Logs" ];
  //let mut active_menu_item = MenuItem::Home;
  let mut rb_list_state = ListState::default();
  rb_list_state.select(Some(0));
 
  // components which are in all tabs
  let mut ui_menu = Menu::new();

  // a message cache for the stream
  let mut stream_cache = VecDeque::<TofPacket>::new();
  let mut packets = VecDeque::<String>::new();
  loop {
    terminal.draw(|rect| {
      let size = rect.size();
      let mster_lo = MasterLayout::new(size); 
      let mut cmd_tab    = CommandTab::new(mster_lo.rect[1],
                                           &packets,
                                           rsp_from_cmdr.clone(),
                                           cmd_to_cmdr.clone());
      let mut mt_tab     = MTTab::new(mster_lo.rect[1], &packets);
      let mut status_tab = StatusTab::new(mster_lo.rect[1],
                                          &rb_list,
                                          rb_list_state.clone());
      rect.render_widget(ui_menu.tabs.clone(), mster_lo.rect[0]);
      let w_logs = render_logs();
      rect.render_widget(w_logs, mster_lo.rect[2]);
      match ui_menu.active_menu_item {
        MenuItem::MasterTrigger => {
          rect.render_stateful_widget(mt_tab.list_widget, mt_tab.list_rect, &mut rb_list_state);
          rect.render_widget(mt_tab.rate,         mt_tab.rate_rect); 
          rect.render_widget(mt_tab.stream,       mt_tab.stream_rect);
          rect.render_widget(mt_tab.network_moni, mt_tab.nw_mon_rect); 
          rect.render_widget(mt_tab.detail,       mt_tab.detail_rect); 
        

        },
        MenuItem::Commands => {
          match rx.recv() {
            Err(err) => trace!("No update"),
            Ok(event) => {
              match event {
                Event::Input(ev) => {
                  match ev.code {
                    // it seems we have to carry thos allong for every tab
                    KeyCode::Char('h') => ui_menu.active_menu_item = MenuItem::Home,
                    KeyCode::Char('c') => ui_menu.active_menu_item = MenuItem::Commands,
                    KeyCode::Char('r') => ui_menu.active_menu_item = MenuItem::Status,
                    KeyCode::Char('m') => ui_menu.active_menu_item = MenuItem::MasterTrigger,
                    KeyCode::Down => {
                      if let Some(selected) = rb_list_state.selected() {
                        //let amount_pets = read_db().expect("can fetch pet list").len();
                        if selected >= cmd_tab.cmd_list.len() {
                          rb_list_state.select(Some(0));
                        } else {
                          rb_list_state.select(Some(selected + 1));
                        }
                      }
                    }
                    KeyCode::Up => {
                      if let Some(selected) = rb_list_state.selected() {
                        //let amount_pets = read_db().expect("can fetch pet list").len();
                        if selected < 1 {
                            rb_list_state.select(Some(0));
                        } else {
                            rb_list_state.select(Some(selected - 1));
                        }
                      }
                    }

                    KeyCode::Enter => {
                      if matches!(ui_menu.active_menu_item, MenuItem::Commands) {
                        info!("Enter pressed, will send highlighted tof command!");
                        warn!("This is not yet implemented!");
                        if let Some(selected) = rb_list_state.selected() {
                          // We hope (it *should* be) that the command list vector 
                          // and the actual command vector are aligned
                          let this_command = cmd_tab.cmd_list[selected];
                          match cmd_to_cmdr.send(this_command) {
                            Err(err) => warn!("There was a problem sending the command!"),
                            Ok(_)    => info!("Command sent!")
                          }
                        }
                      }
                    },
                    _ => trace!("Some other key pressed!"),
                  }
                },
                Event::Tick => {
                  let foo : String = "Tick :".to_owned() + &tick_count.to_string();
                  
                  // check the zmq socket
                  let mut event = TofPacket::new();
                  let mut last_response = Vec::<Option<TofResponse>>::new();

                  match rsp_from_cmdr.try_recv() {
                    Err(err)     => trace!("No response!"),
                    Ok(response) => {
                      last_response = response;             
                    }
                  }
                  match tp_from_recv.try_recv() {
                    Err(err) => {
                      trace!("No event!");
                    }
                    Ok(pk)  => {
                      event = pk;
                      //let mut event = TofPacket::new();
                      //event.packet_type = PacketType::RBEvent;
                      // if the cache is too big, remove the oldest events
                      //let new_tof_events = vec![event];
                      stream_cache.push_back(event);
                      if stream_cache.len() > STREAM_CACHE_MAX_SIZE {
                        stream_cache.pop_front();
                        packets.pop_front(); 
                      }
                      for n in 0..stream_cache.len() {
                        let foo = CommandTab::<'_>::get_pk_repr(&stream_cache[n]);
                        packets.push_back(foo);
                      }
                      info!("Updating Command tab!");
                      cmd_tab.update(&packets,
                                     &last_response);
                    }
                  }
                },
              }
            }
          }    

          rect.render_stateful_widget(cmd_tab.list_widget, cmd_tab.list_rect, &mut rb_list_state);
          rect.render_widget(cmd_tab.tof_resp, cmd_tab.rsp_rect); 
          rect.render_widget(cmd_tab.stream,   cmd_tab.stream_rect);
        }

        MenuItem::Status => {
          
          match rx.recv() {
            Err(err) => trace!("No update"),
            Ok(event) => {
              match event {
                Event::Input(ev) => {
                  match ev.code {
                    KeyCode::Down => {
                      if let Some(selected) = rb_list_state.selected() {
                        //let amount_pets = read_db().expect("can fetch pet list").len();
                        let max_rb = 40;
                        if selected >= rb_list.len() {
                          rb_list_state.select(Some(0));
                        } else {
                          rb_list_state.select(Some(selected + 1));
                        }
                      }
                    }
                    KeyCode::Up => {
                      if let Some(selected) = rb_list_state.selected() {
                        //let amount_pets = read_db().expect("can fetch pet list").len();
                        let max_rb = 40;
                        if max_rb > 0 {
                            rb_list_state.select(Some(selected - 1));
                        } else {
                            rb_list_state.select(Some(rb_list.len() - 1));
                        }
                      }
                    }
                  _ => trace!("Some other key pressed!"),
                  }
                },
                Event::Tick => (),
              }
            }
          }


          let empty_data = vec![(0.0,0.0);1024]; 
          let mut data = vec![empty_data;9];
          let mut update_channels = false;
          match tp_from_recv.recv() {
            Err(err) => {trace!("Did not receive new data!");},
            Ok(dt)   => {
              if dt.packet_type == PacketType::RBEvent {
                data = Vec::<Vec<(f64,f64)>>::new();
               
                let mut event = BlobData::new();
                event.from_bytestream(&dt.payload, 0, false);
                if event.ch_adc.len() == 9 {
                  for n in 0..9 {
                    data.push(Vec::<(f64,f64)>::new());
                    let adc = event.ch_adc[n];
                    for j in 0..adc.len() {
                      data[n].push((j as f64, adc[j] as f64));
                    }
                    update_channels = true;
                  }
                }
              }
            }
          }
          let xlabels = vec!["0", "200", "400", "600", "800", "1000"];
          let ylabels = vec!["0","50", "100"];
          //let cdata = data.clone();

          let datasets = vec![
            Dataset::default()
              .name("Ch0")
              .marker(symbols::Marker::Dot)
              .graph_type(GraphType::Line)
              .style(Style::default().fg(Color::White))
              .data(&data[0]),
            Dataset::default()
              .name("Ch1")
              .marker(symbols::Marker::Braille)
              .graph_type(GraphType::Line)
              .style(Style::default().fg(Color::White))
              .data(&data[1]),
            Dataset::default()
              .name("Ch2")
              .marker(symbols::Marker::Braille)
              .graph_type(GraphType::Line)
              .style(Style::default().fg(Color::White))
              .data(&data[2]),
            Dataset::default()
              .name("Ch3")
              .marker(symbols::Marker::Braille)
              .graph_type(GraphType::Line)
              .style(Style::default().fg(Color::White))
              .data(&data[3]),
            Dataset::default()
              .name("Ch4")
              .marker(symbols::Marker::Braille)
              .graph_type(GraphType::Line)
              .style(Style::default().fg(Color::White))
              .data(&data[4]),
            Dataset::default()
              .name("Ch5")
              .marker(symbols::Marker::Braille)
              .graph_type(GraphType::Line)
              .style(Style::default().fg(Color::Magenta))
              .data(&data[5]),
            Dataset::default()
              .name("Ch6")
              .marker(symbols::Marker::Braille)
              .graph_type(GraphType::Line)
              .style(Style::default().fg(Color::Magenta))
              .data(&data[6]),
            Dataset::default()
              .name("Ch7")
              .marker(symbols::Marker::Braille)
              .graph_type(GraphType::Line)
              .style(Style::default().fg(Color::Magenta))
              .data(&data[7]),
            Dataset::default()
              .name("Ch8 ('Ninth')")
              .marker(symbols::Marker::Braille)
              .graph_type(GraphType::Line)
              .style(Style::default().fg(Color::Magenta))
              .data(&data[8]),
          ];
          
          let mut charts  = Vec::<Chart>::new();
          for n in 0..datasets.len() {
            let this_chart_dataset = vec![datasets[n].clone()];
            let chart = Chart::new(this_chart_dataset)
            .block(
              Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Ch ".to_owned() + &n.to_string() )
                .border_type(BorderType::Plain),
            )
            .x_axis(Axis::default()
              .title(Span::styled("bin", Style::default().fg(Color::White)))
              .style(Style::default().fg(Color::White))
              .bounds([0.0, 1024.0])
              .labels(xlabels.clone().iter().cloned().map(Span::from).collect()))
            .y_axis(Axis::default()
              .title(Span::styled("ADC", Style::default().fg(Color::White)))
              .style(Style::default().fg(Color::White))
              .bounds([0.0, 100.0])
              .labels(ylabels.clone().iter().cloned().map(Span::from).collect()));
            charts.push(chart);
          }
          
          rect.render_stateful_widget(status_tab.list_widget, status_tab.list_rect, &mut rb_list_state);
          rect.render_widget(status_tab.detail, status_tab.detail_rect); 
          if update_channels { 
            for n in 0..status_tab.ch_rect.len() {
              rect.render_widget(charts[n].clone(), status_tab.ch_rect[n]);
            }
          }
          // chart for "ch9"
          let ch9 = vec![ Dataset::default()
              .name("Ch8 ('Ninth')")
              .marker(symbols::Marker::Braille)
              .graph_type(GraphType::Line)
              .style(Style::default().fg(Color::Magenta))
              .data(&data[8])
          ];
          let ch9_chart = Chart::new(ch9)
            .block(
              Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Ch 9")
                .border_type(BorderType::Plain),
            )
            .x_axis(Axis::default()
              .title(Span::styled("bin", Style::default().fg(Color::White)))
              .style(Style::default().fg(Color::White))
              .bounds([0.0, 1024.0])
              .labels(xlabels.clone().iter().cloned().map(Span::from).collect()))
            .y_axis(Axis::default()
              .title(Span::styled("ADC", Style::default().fg(Color::White)))
              .style(Style::default().fg(Color::White))
              .bounds([0.0, 100.0])
              .labels(ylabels.clone().iter().cloned().map(Span::from).collect()));
          



          //return charts;
          //self.ch_charts = charts;
          //}
          //for n in 0..status_tab.ch_rect.len() {
          //  rect.render_widget(status_tab.ch_charts[n].clone(), status_tab.ch_rect[n]);
          //}
          //let status_chunks = Layout::default()
          //  .direction(Direction::Horizontal)
          //  .constraints(
          //      [Constraint::Percentage(10), Constraint::Percentage(20), Constraint::Percentage(70)].as_ref(),
          //  )
          //  .split(mster_lo.rect[1]);
          //let ch_chunks = Layout::default()
          //  .direction(Direction::Vertical)
          //  .constraints(
          //      [Constraint::Percentage(11),
          //       Constraint::Percentage(11),
          //       Constraint::Percentage(11),
          //       Constraint::Percentage(11),
          //       Constraint::Percentage(11),
          //       Constraint::Percentage(11),
          //       Constraint::Percentage(11),
          //       Constraint::Percentage(11),
          //       Constraint::Percentage(12)].as_ref(),
          //  )
          //  .split(status_chunks[2]);
          //let (left, center, mut right) = render_status(rb_list_state.clone(), rb_list.clone());
          ////let (left, center, mut right) = render_status(rb_list.clone());
          //rect.render_stateful_widget(left, status_chunks[0], &mut rb_list_state);
          //rect.render_widget(center, status_chunks[1]);
          //for n in 0..ch_chunks.len() - 1 {
          //  let ch = right.remove(0);
          //  rect.render_widget(ch, ch_chunks[n]);
          //}
        },
        _ => (),
      } 

    }); // end terminal.draw

    match rx.recv()? {
      Event::Tick => {
        match ui_menu.active_menu_item {
          MenuItem::Commands => {
          },
          _ => ()
        }
      },
      Event::Input(event) => {
        match event.code {
          KeyCode::Char('q') => {
              disable_raw_mode()?;
              terminal.clear()?;
              terminal.show_cursor()?;
              break;
          },
          KeyCode::Char('h') => ui_menu.active_menu_item = MenuItem::Home,
          KeyCode::Char('c') => ui_menu.active_menu_item = MenuItem::Commands,
          KeyCode::Char('r') => ui_menu.active_menu_item = MenuItem::Status,
          KeyCode::Char('m') => ui_menu.active_menu_item = MenuItem::MasterTrigger,
          //KeyCode::Down => {
          //  if let Some(selected) = rb_list_state.selected() {
          //    //let amount_pets = read_db().expect("can fetch pet list").len();
          //    let max_rb = 40;
          //    if selected >= rb_list.len() {
          //      rb_list_state.select(Some(0));
          //    } else {
          //      rb_list_state.select(Some(selected + 1));
          //    }
          //  }
          //}
          //KeyCode::Up => {
          //  if let Some(selected) = rb_list_state.selected() {
          //    //let amount_pets = read_db().expect("can fetch pet list").len();
          //    let max_rb = 40;
          //    if max_rb > 0 {
          //        rb_list_state.select(Some(selected - 1));
          //    } else {
          //        rb_list_state.select(Some(rb_list.len() - 1));
          //    }
          //  }
          //}
          _ => (),
        }
      }
    }
  } // end loo;
  Ok(())
        //KeyCode::Char('h') => active_menu_item = MenuItem::Home,
        //KeyCode::Char('p') => active_menu_item = MenuItem::Pets,
        //KeyCode::Char('a') => {
        //    add_random_pet_to_db().expect("can add new random pet");
        //}
        //KeyCode::Char('d') => {
        //    remove_pet_at_index(&mut pet_list_state).expect("can remove pet");
        //}
        //KeyCode::Down => {
        //    if let Some(selected) = pet_list_state.selected() {
        //        let amount_pets = read_db().expect("can fetch pet list").len();
        //        if selected >= amount_pets - 1 { 
        //            pet_list_state.select(Some(0));
        //        } else {
        //            pet_list_state.select(Some(selected + 1));
        //        }
        //    }
        //}
        //KeyCode::Up => {
        //    if let Some(selected) = pet_list_state.selected() {
        //        let amount_pets = read_db().expect("can fetch pet list").len();
        //        if selected > 0 { 
        //            pet_list_state.select(Some(selected - 1));
        //        } else {
        //            pet_list_state.select(Some(amount_pets - 1));
        //        }
        //    }
        //}
        //  _ => (),
        //} // end match key
  //    }// end match event/tick
  //  }; // end terminal.draw
  //} // end loop
  //return Ok(()); 
}
