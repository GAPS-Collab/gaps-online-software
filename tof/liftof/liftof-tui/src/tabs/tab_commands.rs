//! Commands tab
//! 
//use chrono::Utc;

use ratatui::prelude::*;
use ratatui::widgets::{
    Block,
    BorderType,
    Borders,
    //Paragraph,
    //BarChart,
    List,
    ListItem,
    ListState,
};
//use tui::{
//    symbols,
//    backend::CrosstermBackend,
//    layout::{Alignment, Constraint, Direction, Layout, Rect},
//    style::{Color, Modifier, Style},
//    text::{Span, Spans, Text},
//    widgets::{
//        Block, Dataset, Axis, GraphType, BorderType, Chart, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,    },
//    Terminal,
//};

//use std::collections::VecDeque;
use crossbeam_channel::{
    //unbounded,
    //Sender,
    Receiver
};

//use tof_dataclasses::packets::{TofPacket, PacketType};
use tof_dataclasses::commands::{
    TofCommandV2,
    TofCommandCode,
    TofResponse
};

use tof_dataclasses::serialization::{
    Serialization,
    Packable
};

use crate::colors::ColorTheme;

pub struct CommandTab<'a> {
  pub resp_rc            : Receiver<TofResponse>,
  pub theme              : ColorTheme,
  pub cmd_sender         : zmq::Socket,
  // list for the command selector
  pub cmdl_state         : ListState,
  pub cmdl_items         : Vec::<ListItem<'a>>,
  pub cmdl_active        : bool,
  pub cmdl_selector      : usize,
  pub allow_commands     : bool,
  pub active_cmd         : TofCommandV2,
}

impl CommandTab<'_> {

  pub fn new<'a>(resp_rc        : Receiver<TofResponse>,
                 cmd_pub_addr   : String,
                 theme          : ColorTheme,
                 allow_commands : bool) -> CommandTab<'a> {  
    let mut ping_cmd = TofCommandV2::new();
    ping_cmd.command_code = TofCommandCode::Ping;
    let mut start_cmd = TofCommandV2::new();
    start_cmd.command_code = TofCommandCode::DataRunStart;
    let mut stop_cmd = TofCommandV2::new();
    stop_cmd.command_code = TofCommandCode::DataRunStop;
    let mut cali_cmd = TofCommandV2::new();
    cali_cmd.command_code = TofCommandCode::RBCalibration;
    let commands = vec![ping_cmd, start_cmd, stop_cmd, cali_cmd];
    let mut cmd_select_items = Vec::<ListItem>::new();
    for k in commands {
      let this_item = format!("{:?}", k.command_code);
      cmd_select_items.push(ListItem::new(Line::from(this_item)));
    } 
    let ctx = zmq::Context::new();
    let cmd_sender = ctx.socket(zmq::PUB).expect("Can not create 0MQ PUB socket!"); 
    if allow_commands {
      cmd_sender.bind(&cmd_pub_addr).expect("Unable to bind to (PUB) socket!");
    }
    //thread::sleep(10*one_second);

    CommandTab {
      theme   ,
      resp_rc ,
      cmd_sender,
      cmdl_state     : ListState::default(),
      cmdl_items     : cmd_select_items,
      cmdl_active    : false,
      cmdl_selector  : 0,
      allow_commands : allow_commands,
      active_cmd     : TofCommandV2::new(),
    }
  }
  
  pub fn next_cmd(&mut self) {
    let i = match self.cmdl_state.selected() {
      Some(i) => {
        if i >= self.cmdl_items.len() - 1 {
          self.cmdl_items.len() - 1
        } else {
          i + 1
        }
      }
      None => 0,
    };
    self.cmdl_state.select(Some(i));
    //info!("Selecting {}", i);
  }

  pub fn prev_cmd(&mut self) {
    let i = match self.cmdl_state.selected() {
      Some(i) => {
        if i == 0 {
          0 
        } else {
          i - 1
        }
      }
      None => 0,
    };
    self.cmdl_state.select(Some(i));
  }
 
  /// send the selected dommand
  pub fn send_command(&self) {
    if !self.allow_commands {
      error!("To send commands, run program with --send-commands!");
      return;
    }
    info!("Sending TOF cmd {}", self.active_cmd);
    match self.active_cmd.command_code {
      TofCommandCode::DataRunStop => {
        let payload = self.active_cmd.pack().to_bytestream();
        match self.cmd_sender.send(&payload, 0) {
          Err(err) => {
            error!("Unable to send command! {err}");
          },
          Ok(_) => {
            println!("=> Calibration  initialized!");
          }
        }  

      }     
      _ => ()
    }
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {

    let main_lo = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
          [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
      )
      .split(*main_window);
       let par_title_string = String::from("Select Command");
       let (first, rest) = par_title_string.split_at(1);
    let par_title = Line::from(vec![
         Span::styled(
             first,
             Style::default()
                 .fg(self.theme.hc)
                 .add_modifier(Modifier::UNDERLINED),
         ),
         Span::styled(rest, self.theme.style()),
       ]);
    let cmds = Block::default()
      .borders(Borders::ALL)
      .style(self.theme.style())
      .title(par_title)
      .border_type(BorderType::Plain);
    let cmd_select_list = List::new(self.cmdl_items.clone()).block(cmds)
      .highlight_style(self.theme.highlight().add_modifier(Modifier::BOLD))
      .highlight_symbol(">>")
      .repeat_highlight_symbol(true);
    match self.cmdl_state.selected() {
      None    => {
        self.cmdl_selector = 0;
      },
      Some(cmd_id) => {
        // entry 0 is for all paddles
        let selector =  cmd_id;
        if self.cmdl_selector != selector {
          //self.paddle_changed = true;
          //self.init_histos();
          self.cmdl_selector = selector;
        } else {
          //self.paddle_changed = false;
        }
      },
    }
    if self.allow_commands {
      frame.render_stateful_widget(cmd_select_list, main_lo[0], &mut self.cmdl_state );
    }
  }
} // end impl
    
//     
//    
//    let cmd_block = Block::default()
//    .borders(Borders::ALL)
//    .style(Style::default().fg(Color::White))
//    .title("Commands")
//    .border_type(BorderType::Plain);
//
//    // FIXME
//    //let cmd_list = vec![
//    let mut cmd_list = Vec::<TofCommand>::new();
//    cmd_list.push(  TofCommand::PowerOn               (0)); 
//    cmd_list.push(  TofCommand::PowerOff              (0));    
//    cmd_list.push(  TofCommand::PowerCycle            (0));   
//    cmd_list.push(  TofCommand::RBSetup               (0));      
//    cmd_list.push(  TofCommand::SetThresholds         (0));   
//    //cmd_list.push(  TofCommand::SetMtConfig           (0));   
//    cmd_list.push(  TofCommand::StartValidationRun    (9));    
//    //cmd_list.push(  TofCommand::RequestWaveforms      (0));     
//    cmd_list.push(  TofCommand::UnspoolEventCache     (0));      
//    cmd_list.push(  TofCommand::StreamAnyEvent        (0));     
//    cmd_list.push(  TofCommand::StreamOnlyRequested   (0));     
//    cmd_list.push(  TofCommand::DataRunStart          (0));    
//    //cmd_list.push(  TofCommand::DataRunEnd            (0));       
//    cmd_list.push(  TofCommand::VoltageCalibration    (0));       
//    cmd_list.push(  TofCommand::TimingCalibration     (0));       
//    cmd_list.push(  TofCommand::CreateCalibrationFile (0));          
//    //cmd_list.push(  TofCommand::RequestEvent          (0));        
//    cmd_list.push(  TofCommand::RequestMoni           (0));   
//    //];
//
//    let mut items = Vec::<ListItem>::new();
//    for n in 0..cmd_list.len() {
//      items.push(
//        ListItem::new(Spans::from(vec![Span::styled(
//          cmd_list[n].to_string().clone().replace("<TofCommand", "").replace(">",""),
//          Style::default())]))
//        );
//    }
//    let selected_cmd = cmd_list[0]
//     // .get(
//     //   rb_list_state
//     //     .selected()
//     //     .expect("there is always a selected pet"),
//     // )
//     // .expect("exists")
//     .clone();
//
//    let list_widget = List::new(items).block(cmd_block).highlight_style(
//      Style::default()
//        .bg(Color::Blue)
//        .fg(Color::Black)
//        .add_modifier(Modifier::BOLD),
//    );
//    
//    let stream =  Paragraph::new("")
//    .style(Style::default().fg(Color::LightCyan))
//    .alignment(Alignment::Left)
//    //.scroll((5, 10))
//    .block(
//      Block::default()
//        .borders(Borders::ALL)
//        .style(Style::default().fg(Color::White))
//        .title("Stream")
//        .border_type(BorderType::Plain),
//    );
//
//    let tof_resp = Paragraph::new("")
//    .style(Style::default().fg(Color::LightCyan))
//    .alignment(Alignment::Center)
//    .block(
//      Block::default()
//        .borders(Borders::ALL)
//        .style(Style::default().fg(Color::White))
//        .title("TofResponse")
//        .border_type(BorderType::Plain),
//    );
//
//    let list_rect    = cmds_chunks[0];
//    let rsp_rect     = resp_chunks[0];
//    let stream_rect  = resp_chunks[1];
//
//    let mut ct = CommandTab {
//      stream   ,
//      tof_resp ,
//      cmd_list,
//      list_widget,
//      list_rect,
//      rsp_rect,
//      stream_rect,
//      message_queue : VecDeque::<String>::new()
//    };
//    let response = Vec::<Option<TofResponse>>::new();
//    ct.update(packets, &response);
//    ct
//  }
//
//  /// Create a displayable string for the ui from 
//  /// a tof packet
//  pub fn get_pk_repr(pk : &TofPacket) -> String {
//    let mut pk_repr = String::from("<Unknown TP>");
//    info!("Got package of type {:?}", pk.packet_type);
//    let now = Utc::now().to_rfc2822();  
//    match &pk.packet_type {
//      PacketType::Unknown   => {
//      },
//      PacketType::RBEvent   => {
//        //let eventid = RBEventMemoryView::decode_event_id_from_stream(&pk.payload).unwrap();
//        let eventid = 42u32;
//        pk_repr = String::from("\u{2728} <") + &now + " " + &eventid.to_string() + " - RBEvent >";
//      },
//      PacketType::RBMoni   => {
//        pk_repr = String::from("\u{1f4c8} <") + &now + " - RBMoniData >";
//      },
//      PacketType::HeartBeat => {
//      },
//      _         => (),
//    }
//    pk_repr
//  }
//
//  /// Split the init routine in two parts:
//  /// 1) ::new() will init the message cache
//  /// 2) init_draw will inti the drawing
//  /// on the frame
//  ///
//  /// In this way, we can maintain 
//  /// a single event cache 
//  pub fn init_draw(&mut self, main_window : Rect) {
//     let cmds_chunks = Layout::default()
//       .direction(Direction::Horizontal)
//       .constraints(
//           [Constraint::Percentage(10), Constraint::Percentage(90)].as_ref(),
//       )
//       .split(main_window);
//     
//     let resp_chunks = Layout::default()
//       .direction(Direction::Vertical)
//       .constraints(
//           [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
//       )
//       .split(cmds_chunks[1]);
//  }
//
//
//  /// Update the tab 
//  ///
//  /// Use in the render loop. Will add current stream 
//  /// information as well as the last response
//  /// from the tof system.
//  pub fn update(&mut self,
//                packets : &VecDeque<String>,
//                response : &Vec<Option<TofResponse>>) {
//
//    //
//    //let foo = packets.pop().unwrap();
//    //let foo = CommandTab::<'_>::get_pk_repr(&foo);
//    let mut spans = Vec::<Spans>::new();
//    for n in 0..packets.len() {
//        spans.push(Spans::from(vec![Span::styled(
//            //String::from("PowerOn"),
//            packets[n].clone(),
//            Style::default())])
//        );
//    }
//       
//    let mut spans_resp = Vec::<Spans>::new();
//    for n in 0..response.len() {
//        if response[n].is_none() {
//          continue;
//        }
//        spans_resp.push(Spans::from(vec![Span::styled(
//            String::from("FIXME"),
//            //response[n].unwrap().string_repr().clone(),
//            Style::default())])
//        );
//    }
//    self.tof_resp = Paragraph::new(spans_resp)
//    .style(Style::default().fg(Color::LightCyan))
//    .alignment(Alignment::Center)
//    .block(
//      Block::default()
//        .borders(Borders::ALL)
//        .style(Style::default().fg(Color::White))
//        .title("TofResponse")
//        .border_type(BorderType::Plain),
//    );
//
//    self.stream =  Paragraph::new(spans)
//    .style(Style::default().fg(Color::LightCyan))
//    .alignment(Alignment::Left)
//    //.scroll((5,10))
//    .block(
//      Block::default()
//        .borders(Borders::ALL)
//        .style(Style::default().fg(Color::White))
//        .title("Stream")
//        .border_type(BorderType::Plain),
//    );
//  }
//}
