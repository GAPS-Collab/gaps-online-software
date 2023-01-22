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

fn clone_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Clone,
{
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}

#[derive(Debug, Clone)]
pub struct StatusTab<'a> {

  pub detail       : Paragraph<'a>,
  rb_list          : Vec::<ReadoutBoard>,
  pub list_widget  : List<'a>,
  pub list_rect    : Rect,
  pub detail_rect  : Rect,
  pub ch_rect      : Vec<Rect>,
  //message_queue    : VecDeque<String>, 
  //rb_list_state    : ListState,
  //pub ch_datasets  : Vec<Dataset<'a>>,
  //pub ch_charts    : Vec<Chart<'a>>
}

impl StatusTab<'_> {

  //pub fn update(&mut self, list_state : ListState) {
  //  self.rb_list_state = list_state;
  //}

  //pub fn update_datasets<'a>(&mut self,
  //                           ch_times : [
  //                           ch0_data : [(f64,f64)],
  //                           ch1_data : [(f64,f64)],
  //                           ch2_data : [(f64,f64)],
  //                           ch3_data : [(f64,f64)],
  //                           ch4_data : [(f64,f64)],
  //                           ch5_data : [(f64,f64)],
  //                           ch6_data : [(f64,f64)],
  //                           ch7_data : [(f64,f64)],
  //                           ch8_data : [(f64,f64)]) {

  //                           //data : &'a Vec<Vec<(f64, f64)>>) {
  //  
  //  let xlabels = vec!["0", "200", "400", "600", "800", "1000"];
  //  let ylabels = vec!["0","50", "100"];
  //  //let cdata = data.clone();

  //  let datasets = vec![
  //    Dataset::default()
  //      .name("Ch0")
  //      .marker(symbols::Marker::Dot)
  //      .graph_type(GraphType::Scatter)
  //      .style(Style::default().fg(Color::Cyan))
  //      .data(ch0_data),
  //    Dataset::default()
  //      .name("Ch1")
  //      .marker(symbols::Marker::Braille)
  //      .graph_type(GraphType::Line)
  //      .style(Style::default().fg(Color::Magenta))
  //      .data(ch1_data),
  //    Dataset::default()
  //      .name("Ch2")
  //      .marker(symbols::Marker::Braille)
  //      .graph_type(GraphType::Line)
  //      .style(Style::default().fg(Color::Magenta))
  //      .data(ch2_data),
  //    Dataset::default()
  //      .name("Ch3")
  //      .marker(symbols::Marker::Braille)
  //      .graph_type(GraphType::Line)
  //      .style(Style::default().fg(Color::Magenta))
  //      .data(ch3_data),
  //    Dataset::default()
  //      .name("Ch4")
  //      .marker(symbols::Marker::Braille)
  //      .graph_type(GraphType::Line)
  //      .style(Style::default().fg(Color::Magenta))
  //      .data(ch4_data),
  //    Dataset::default()
  //      .name("Ch5")
  //      .marker(symbols::Marker::Braille)
  //      .graph_type(GraphType::Line)
  //      .style(Style::default().fg(Color::Magenta))
  //      .data(ch5_data),
  //    Dataset::default()
  //      .name("Ch6")
  //      .marker(symbols::Marker::Braille)
  //      .graph_type(GraphType::Line)
  //      .style(Style::default().fg(Color::Magenta))
  //      .data(ch6_data),
  //    Dataset::default()
  //      .name("Ch7")
  //      .marker(symbols::Marker::Braille)
  //      .graph_type(GraphType::Line)
  //      .style(Style::default().fg(Color::Magenta))
  //      .data(ch7_data),
  //    Dataset::default()
  //      .name("Ch8 ('Ninth')")
  //      .marker(symbols::Marker::Braille)
  //      .graph_type(GraphType::Line)
  //      .style(Style::default().fg(Color::Magenta))
  //      .data(ch8_data),
  //  ];
  //  
  //  let mut charts  = Vec::<Chart>::new();
  //  for n in 0..datasets.len() {
  //    let this_chart_dataset = vec![datasets[n].clone()];
  //    let chart = Chart::new(this_chart_dataset)
  //    .block(
  //      Block::default()
  //        .borders(Borders::ALL)
  //        .style(Style::default().fg(Color::White))
  //        .title("Ch ".to_owned() + &n.to_string() )
  //        .border_type(BorderType::Plain),
  //    )
  //    .x_axis(Axis::default()
  //      .title(Span::styled("bin", Style::default().fg(Color::White)))
  //      .style(Style::default().fg(Color::White))
  //      .bounds([0.0, 1024.0])
  //      .labels(xlabels.clone().iter().cloned().map(Span::from).collect()))
  //    .y_axis(Axis::default()
  //      .title(Span::styled("ADC", Style::default().fg(Color::White)))
  //      .style(Style::default().fg(Color::White))
  //      .bounds([0.0, 100.0])
  //      .labels(ylabels.clone().iter().cloned().map(Span::from).collect()));
  //    charts.push(chart);
  //  }
  //  //return charts;
  //  self.ch_charts = charts;
  //}

  pub fn new<'a> (main_window   : Rect,
                  rb_list       : &Vec<ReadoutBoard>,
                  rb_list_state : ListState)
    -> StatusTab<'a> {
    let empty_data = vec![(0.0,0.0);1024]; 
    let data = vec![empty_data;9];
    //let data     = Vec::<Vec<(f64,f64)>>::new();
    let charts = Vec::<Chart>::new();
    //let data0 = data[0].clone();
    //let data1 = data[1].clone();
    //let data2 = data[2].clone();
    //let data3 = data[3].clone();
    //let data4 = data[4].clone();
    //let data5 = data[5].clone();
    //let data6 = data[6].clone();
    //let data7 = data[7].clone();
    //let data8 = data[8].clone();
    //let ch0_data = clone_into_array(data0);//.as_slice().clone();
    //let ch1_data = clone_into_array(data1);//.as_slice().clone();
    //let ch2_data = clone_into_array(data2);//.as_slice().clone();
    //let ch3_data = clone_into_array(data3);//.as_slice().clone();
    //let ch4_data = clone_into_array(data4);//.as_slice().clone();
    //let ch5_data = clone_into_array(data5);//.as_slice().clone();
    //let ch6_data = clone_into_array(data6);//.as_slice().clone();
    //let ch7_data = clone_into_array(data7);//.as_slice().clone();
    //let ch8_data = clone_into_array(data8);//.as_slice().clone();
    //let charts = StatusTab::update_datasets(&data);
    let chart_list = charts.clone();

    // set up general layout
    let status_chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
          [Constraint::Percentage(10), Constraint::Percentage(20), Constraint::Percentage(70)].as_ref(),
      )
      .split(main_window);
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
      detail_rect      : status_chunks[1],
      ch_rect          : ch_chunks,
      //ch_charts        : chart_list,
    };
    //st.update_datasets(
    //  ch0_data,
    //  ch1_data,
    //  ch2_data,
    //  ch3_data,
    //  ch0_data,
    //  ch5_data,
    //  ch6_data,
    //  ch7_data,
    //  ch8_data,
    //);
    st
    
  } // end new
}

