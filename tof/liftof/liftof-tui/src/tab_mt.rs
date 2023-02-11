//! Master Trigger tab
//! 
//! Show current data from the master trigger
//! The layout is somewhat like this
//!
//! -----------------------------------------
//! | Menu  | .. | .. |                     |
//! -----------------------------------------
//! | Rate           | Event Strean         |
//! | =====          |                      |
//! | Network        | <EVID 0>             |
//! | =====          | <EVID 1>             |
//! | Detail         | <EVID 2>             |
//! | =====          | <EVID 3>             |
//! | Commands       | <EVID 4>             |
//! -----------------------------------------
//! | Logs                                  |

use chrono::Utc;

use std::time::{Duration, Instant};

use tui::{
    symbols,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{
        Block, Dataset, Sparkline, Axis, GraphType, BorderType, Chart, BarChart, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,    },
    Terminal,
};

use std::collections::VecDeque;

use tof_dataclasses::packets::{TofPacket, PacketType};
use tof_dataclasses::commands::TofCommand;
use tof_dataclasses::events::MasterTriggerEvent;
/// Master trigger tab
/// 
/// Show information about the master trigger.
#[derive(Debug, Clone)]
pub struct MTTab<'a> {

  pub stream        : Paragraph<'a>,
  pub rate          : Sparkline<'a>,
  pub network_moni  : Sparkline<'a>,
  pub n_paddle_dist : BarChart<'a>, 
  pub detail        : Paragraph<'a>,
  cmd_list          : Vec::<TofCommand>,
  pub list_widget   : List<'a>,
  /// keep track of the passed time in seconds,
  /// to update only specific parts of the display
  pub list_rect     : Rect,
  pub stream_rect   : Rect,
  pub detail_rect   : Rect,
  pub nw_mon_rect   : Rect,
  pub rate_rect     : Rect,
  message_queue     : VecDeque<String> 
}

impl MTTab<'_> {

  pub fn new<'a>(main_window : Rect,
                 packets : &VecDeque<String>,
                 detail_string : Option<String>) -> MTTab<'a> {

    let message_queue = VecDeque::<String>::new();
    let main_chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
          [Constraint::Percentage(70), Constraint::Percentage(30)].as_ref(),
      )
      .split(main_window);
    
    let info_chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
          [Constraint::Percentage(40),
           Constraint::Percentage(40),
           Constraint::Percentage(20)
          ].as_ref(),
      )
      .split(main_chunks[1]);
       
    let detail_chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
          [Constraint::Percentage(60),
           Constraint::Percentage(40),
          ].as_ref(),
      )
      .split(main_chunks[0]);
   

    let cmd_block = Block::default()
    .borders(Borders::ALL)
    .style(Style::default().fg(Color::White))
    .title("Commands")
    .border_type(BorderType::Plain);

    let mut cmd_list = Vec::<TofCommand>::new();
    cmd_list.push(  TofCommand::DataRunStart          (0));    
    cmd_list.push(  TofCommand::DataRunEnd            (0));       
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
    let rate = Sparkline::default()
    .block(
      Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Rate")
        .border_type(BorderType::Double),
    ) // or THREE_LEVELS
    .bar_set(tui::symbols::bar::NINE_LEVELS)
    .data(&[0, 2, 3, 4, 1, 4, 10])
    .max(5)
    .style(Style::default().fg(Color::Blue).bg(Color::Black));

    let n_paddle_data = vec![ 
                        ( "0", 0),
                        ( "1", 0),
                        ( "2", 0),
                        ( "3", 0),
                        ( "4", 0),
                        ( "5", 0),
                        ( "6", 0),
                        ( "7", 0),
                        ];

    let n_paddle = BarChart::default()
        .block(Block::default().title("N Paddle").borders(Borders::ALL))
        .data(&[("0", 0)])
        .bar_width(5)
        .bar_gap(3)
        .bar_style(Style::default().fg(Color::Green))
        .value_style(
            Style::default()
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        );


    let network = Sparkline::default()
    .block(
      Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Network I/O")
        .border_type(BorderType::Double),
    ) // or THREE_LEVELS
    .bar_set(tui::symbols::bar::NINE_LEVELS)
    .data(&[0, 2, 3, 4, 1, 4, 10])
    .max(5)
    .style(Style::default().fg(Color::Blue).bg(Color::Black));
    

    let mut content = String::from("");
    if detail_string.is_some() {
      content = detail_string.unwrap();
    }
    let detail =  Paragraph::new(content)
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    //.scroll((5, 10))
    .block(
      Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Event Detail")
        .border_type(BorderType::Rounded),
    );

    let rate_rect    = info_chunks[0];
    let nw_mon_rect  = info_chunks[1]; 
    let list_rect    = info_chunks[2]; 
    let stream_rect  = detail_chunks[0];
    let detail_rect  = detail_chunks[1];

    let mut mt = MTTab {
      stream  ,
      rate    ,
      n_paddle_dist : n_paddle ,
      network_moni : network ,
      detail      ,
      cmd_list    ,
      list_widget ,
      list_rect   , 
      stream_rect ,
      detail_rect ,
      nw_mon_rect ,
      rate_rect   ,
      message_queue    : VecDeque::<String>::new() 
    };
    //mt.update(packets);
    mt
  }

  /// Update the tab 
  ///
  /// Use in the render loop. Will add current stream 
  /// information as well as the last response
  /// from the tof system.
  pub fn update(&mut self,
                mt_events : &VecDeque<MasterTriggerEvent>,
                update_detail : bool) -> Option<String> {

    //
    //let foo = packets.pop().unwrap();
    //let foo = CommandTab::<'_>::get_pk_repr(&foo);
    //
    let mut detail_string : Option<String> = None;
    let mut spans = Vec::<Spans>::new();
    for n in 0..mt_events.len() {
        let mut color = Color::White;
        if mt_events[n].is_broken() {
          color = Color::Red;
        }
        spans.push(Spans::from(vec![Span::styled(
            //mt_events[n].to_string().clone(),
            "<\u{2728}MTE : event id ".to_owned() + &mt_events[n].event_id.to_string() + " >",
            Style::default().fg(color))])
        );
    }
    let last_event = mt_events.back();
    if update_detail {
      info!("Updating detail field");
      if last_event.is_some() {
        detail_string = Some(last_event.unwrap().to_string());
        self.detail = Paragraph::new(last_event.unwrap().to_string())
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Left)
        //.scroll((5, 10))
        .block(
          Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Event Detail")
            .border_type(BorderType::Rounded),
        );
      }       
    }
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
    detail_string
  }


}
