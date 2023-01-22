//! Commands tab
//! 
//! Display available commands as a list, if issued show 
//! the corresponding response recieved over the command 
//! channel and show the incoming data stream as well
//!
//! The layout is somewhat like this
//!
//! -----------------------------------------
//! | Menu  | .. | .. |                     |
//! -----------------------------------------
//! | Command List | | Response: <Success>  |
//! | ..           | | ----------------------
//! | <StartRun>   | | Stream               |
//! |              | |   -> ...             |
//! -----------------------------------------
//! | Logs                                  |

use tof_dataclasses::events::blob::BlobData;

use chrono::Utc;

use tui::{
    symbols,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{
        Block, Dataset, Axis, GraphType, BorderType, Chart, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,    },
    Terminal,
};

use std::collections::VecDeque;

use tof_dataclasses::packets::{TofPacket, PacketType};
use tof_dataclasses::commands::{TofCommand,
                                TofResponse};

use crossbeam_channel::{unbounded,
                        Sender,
                        Receiver};

#[derive(Debug, Clone)]
pub struct CommandTab<'a> {

  pub stream      : Paragraph<'a>,
  pub tof_resp    : Paragraph<'a>,
  cmd_list        : Vec::<TofCommand>,
  pub list_widget : List<'a>,
  pub list_rect   : Rect,
  pub rsp_rect    : Rect,
  pub stream_rect : Rect,

  message_queue   : VecDeque<String>
  
}

impl CommandTab<'_> {

  pub fn new<'a>(main_window : Rect,
                 packets : &VecDeque<String>,
                 recv_resp     : Receiver<Vec<Option<TofResponse>>>,
                 send_cmd      : Sender<TofCommand>) -> CommandTab<'a> {

     let cmds_chunks = Layout::default()
       .direction(Direction::Horizontal)
       .constraints(
           [Constraint::Percentage(10), Constraint::Percentage(90)].as_ref(),
       )
       .split(main_window);
     
     let resp_chunks = Layout::default()
       .direction(Direction::Vertical)
       .constraints(
           [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
       )
       .split(cmds_chunks[1]);
        
    
    let cmd_block = Block::default()
    .borders(Borders::ALL)
    .style(Style::default().fg(Color::White))
    .title("Commands")
    .border_type(BorderType::Plain);

    // FIXME
    //let cmd_list = vec![
    let mut cmd_list = Vec::<TofCommand>::new();
    cmd_list.push(  TofCommand::PowerOn               (0)); 
    cmd_list.push(  TofCommand::PowerOff              (0));    
    cmd_list.push(  TofCommand::PowerCycle            (0));   
    cmd_list.push(  TofCommand::RBSetup               (0));      
    cmd_list.push(  TofCommand::SetThresholds         (0));   
    cmd_list.push(  TofCommand::SetMtConfig           (0));   
    cmd_list.push(  TofCommand::StartValidationRun    (9));    
    cmd_list.push(  TofCommand::RequestWaveforms      (0));     
    cmd_list.push(  TofCommand::UnspoolEventCache     (0));      
    cmd_list.push(  TofCommand::StreamAnyEvent        (0));     
    cmd_list.push(  TofCommand::StreamOnlyRequested   (0));     
    cmd_list.push(  TofCommand::DataRunStart          (0));    
    cmd_list.push(  TofCommand::DataRunEnd            (0));       
    cmd_list.push(  TofCommand::VoltageCalibration    (0));       
    cmd_list.push(  TofCommand::TimingCalibration     (0));       
    cmd_list.push(  TofCommand::CreateCalibrationFile (0));          
    cmd_list.push(  TofCommand::RequestEvent          (0));        
    cmd_list.push(  TofCommand::RequestMoni           (0));   
    //];

    let mut items = Vec::<ListItem>::new();
    for n in 0..cmd_list.len() {
      items.push(
        ListItem::new(Spans::from(vec![Span::styled(
          cmd_list[n].to_string().clone(),
          Style::default())]))
        );
    }
    let selected_cmd = cmd_list[0]
     // .get(
     //   rb_list_state
     //     .selected()
     //     .expect("there is always a selected pet"),
     // )
     // .expect("exists")
     .clone();

    let list_widget = List::new(items).block(cmd_block).highlight_style(
      Style::default()
        .bg(Color::Blue)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD),
    );
    
    let stream =  Paragraph::new("")
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    //.scroll((5, 10))
    .block(
      Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Stream")
        .border_type(BorderType::Plain),
    );

    let tof_resp = Paragraph::new("")
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Center)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("TofResponse")
        .border_type(BorderType::Plain),
    );

    let list_rect    = cmds_chunks[0];
    let rsp_rect     = resp_chunks[0];
    let stream_rect  = resp_chunks[1];

    let mut ct = CommandTab {
      stream   ,
      tof_resp ,
      cmd_list,
      list_widget,
      list_rect,
      rsp_rect,
      stream_rect,
      message_queue : VecDeque::<String>::new()
    };
    let response = Vec::<Option<TofResponse>>::new();
    ct.update(packets, &response);
    ct
  }

  /// Create a displayable string for the ui from 
  /// a tof packet
  pub fn get_pk_repr(pk : &TofPacket) -> String {
    let mut pk_repr = String::from("<Unknown TP>");
    info!("Got package of type {:?}", pk.packet_type);
    let now = Utc::now().to_rfc2822();  
    match &pk.packet_type {
      PacketType::Unknown   => {
      },
      PacketType::Command   => {
      },
      PacketType::RBEvent   => {
        let eventid = BlobData::decode_event_id(pk.payload.as_slice());
        pk_repr = String::from("\u{2728} <") + &now + " " + &eventid.to_string() + " - RBEvent >";
      },
      PacketType::Monitor   => {
        pk_repr = String::from("\u{1f4c8} <") + &now + " - RBMoni >";
      },
      PacketType::HeartBeat => {
      },
      PacketType::Scalar    => {
      },
      _         => (),
    }
    pk_repr
  }

  /// Split the init routine in two parts:
  /// 1) ::new() will init the message cache
  /// 2) init_draw will inti the drawing
  /// on the frame
  ///
  /// In this way, we can maintain 
  /// a single event cache 
  pub fn init_draw(&mut self, main_window : Rect) {
     let cmds_chunks = Layout::default()
       .direction(Direction::Horizontal)
       .constraints(
           [Constraint::Percentage(10), Constraint::Percentage(90)].as_ref(),
       )
       .split(main_window);
     
     let resp_chunks = Layout::default()
       .direction(Direction::Vertical)
       .constraints(
           [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
       )
       .split(cmds_chunks[1]);
  }


  /// Update the tab 
  ///
  /// Use in the render loop. Will add current stream 
  /// information as well as the last response
  /// from the tof system.
  pub fn update(&mut self,
                packets : &VecDeque<String>,
                response : &Vec<Option<TofResponse>>) {

    //
    //let foo = packets.pop().unwrap();
    //let foo = CommandTab::<'_>::get_pk_repr(&foo);
    let mut spans = Vec::<Spans>::new();
    for n in 0..packets.len() {
        spans.push(Spans::from(vec![Span::styled(
            //String::from("PowerOn"),
            packets[n].clone(),
            Style::default())])
        );
    }
       
    let mut spans_resp = Vec::<Spans>::new();
    for n in 0..response.len() {
        if response[n].is_none() {
          continue;
        }
        spans_resp.push(Spans::from(vec![Span::styled(
            //String::from("PowerOn"),
            response[n].unwrap().string_repr().clone(),
            Style::default())])
        );
    }
    self.tof_resp = Paragraph::new(spans_resp)
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Center)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("TofResponse")
        .border_type(BorderType::Plain),
    );

    self.stream =  Paragraph::new(spans)
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    //.scroll((5,10))
    .block(
      Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Stream")
        .border_type(BorderType::Plain),
    );
  }
}
