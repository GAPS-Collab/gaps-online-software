//! ReadoutBoard Status tab
//!
//! Find connected ReadoutBoards and show their 
//! details as well as the last waveforms
//!
//! -----------------------------------------
//! | Menu  | .. | .. |                     |
//! -----------------------------------------
//! | RBSelect | Details | WAVEFORMS        |
//! -----------------------------------------



use chrono::Utc;

use tui::{
    symbols,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{
        Block, Dataset, Sparkline, Axis, GraphType, BorderType, Chart, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,    },
    Terminal,
};

use crossbeam_channel::{unbounded,
                        Sender,
                        Receiver};

use std::collections::VecDeque;

use tof_dataclasses::packets::{TofPacket, PacketType};
use tof_dataclasses::commands::{TofCommand,
                                TofResponse};

use liftof_lib::{get_rb_manifest,
                 ReadoutBoard};

//fn clone_into_array<A, T>(slice: &[T]) -> A
//where
//    A: Default + AsMut<[T]>,
//    T: Clone,
//{
//    let mut a = A::default();
//    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
//    a
//}

#[derive(Debug, Clone)]
pub struct StatusTab<'a> {

  pub detail       : Paragraph<'a>,
  rb_list          : Vec::<ReadoutBoard>,
  pub list_widget  : List<'a>,
  pub list_rect    : Rect,
  pub detail_rect  : Rect,
  pub ch_rect      : Vec<Rect>,
  pub ch9_rect     : Rect,
  //message_queue    : VecDeque<String>, 
  //rb_list_state    : ListState,
  //pub ch_datasets  : Vec<Dataset<'a>>,
  //pub ch_charts    : Vec<Chart<'a>>
}

impl StatusTab<'_> {


  pub fn new<'a> (main_window   : Rect,
                  rb_list       : &Vec<ReadoutBoard>,
                  rb_list_state : ListState)
    -> StatusTab<'a> {
    let empty_data = vec![(0.0,0.0);1024]; 
    let data = vec![empty_data;9];
    //let data     = Vec::<Vec<(f64,f64)>>::new();
    let charts = Vec::<Chart>::new();
    let chart_list = charts.clone();

    // set up general layout
    let status_chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
          [Constraint::Percentage(10), Constraint::Percentage(20), Constraint::Percentage(70)].as_ref(),
      )
      .split(main_window);
    let detail_and_ch9_chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
          [Constraint::Percentage(50),
           Constraint::Percentage(50)].as_ref(),
      )
      .split(status_chunks[1]);
    let wf_chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
          [Constraint::Percentage(50),
           Constraint::Percentage(50)].as_ref(),
      )
      .split(status_chunks[2]);
    let mut ch_chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
          [Constraint::Percentage(25),
           Constraint::Percentage(25),
           Constraint::Percentage(26),
           Constraint::Percentage(25)].as_ref(),
      )
      .split(wf_chunks[0]);
    let mut ch_chunks_2 = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
          [Constraint::Percentage(25),
           Constraint::Percentage(25),
           Constraint::Percentage(26),
           Constraint::Percentage(25)].as_ref(),
      )
      .split(wf_chunks[1]);
    ch_chunks.append(&mut ch_chunks_2);
      let items: Vec<_> = rb_list
      .iter()
      .map(|rb| {
        ListItem::new(Spans::from(vec![Span::styled(
          "RB ".to_owned() + &rb.id.unwrap().to_string(),
          Style::default(),
        )]))
      })
      .collect();

    let selected_rb = rb_list[0]
    // .get(
    //   rb_list_state
    //     .selected()
    //     .expect("there is always a selected pet"),
    // )
    // .expect("exists")
     .clone();
    let rbs = Block::default()
    .borders(Borders::ALL)
    .style(Style::default().fg(Color::White))
    .title("ReadoutBoards")
    .border_type(BorderType::Plain);

    let list = List::new(items).block(rbs).highlight_style(
      Style::default()
        .bg(Color::Blue)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD),
    );

    let rb_detail =  Paragraph::new(selected_rb.to_string())
     .style(Style::default().fg(Color::LightCyan))
     .alignment(Alignment::Left)
     //.scroll((5, 10))
     //.text(rb_list[0].to_string())
     .block(
       Block::default()
         .borders(Borders::ALL)
         .style(Style::default().fg(Color::White))
         .title("Detail")
         .border_type(BorderType::Double),
    );
    let mut st = StatusTab {
      detail           : rb_detail,
      rb_list          : rb_list.clone(),
      list_widget      : list,
      list_rect        : status_chunks[0],
      //detail_rect      : status_chunks[1],
      detail_rect      : detail_and_ch9_chunks[0],
      ch_rect          : ch_chunks,
      ch9_rect         : detail_and_ch9_chunks[1]
      //ch_charts        : chart_list,
    };
    st
    
  } // end new
}

