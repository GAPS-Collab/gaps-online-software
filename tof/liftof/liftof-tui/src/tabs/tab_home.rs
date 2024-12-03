//! The front page of the app, including a view of received packets

use std::collections::{
    HashMap,
    VecDeque,
};

use std::time::{
  Instant,
  //Duration
};

use std::sync::{
  Arc,
  Mutex,
};
use ratatui::prelude::*;

//use ratatui::terminal::Frame;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{
  Block,
  BorderType,
  Borders,
  Paragraph,
  Table,
  Row,
};

use liftof_lib::LIFTOF_LOGO_SHOW;
use crate::colors::ColorTheme;


#[derive(Debug, Clone)]
pub struct HomeTab {
  pub theme      : ColorTheme,
  pub streamer   : Arc<Mutex<VecDeque<String>>>,
  pub pack_stat  : Arc<Mutex<HashMap<String, usize>>>,
  pub stream     : String,
  pub stream_max : usize, 
  start_time     : Instant,
}

impl HomeTab {
  pub fn new(theme     : ColorTheme,
             streamer  : Arc<Mutex<VecDeque<String>>>,
             pack_stat : Arc<Mutex<HashMap<String,usize>>>) -> HomeTab {
    HomeTab {
      theme,
      streamer, 
      pack_stat,
      stream     : String::from(""),
      stream_max : 30,
      start_time : Instant::now(),
    }
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    let main_chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
          [Constraint::Percentage(70),
           Constraint::Percentage(30)].as_ref(),
      )
      .split(*main_window);
    
    let upper_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [Constraint::Percentage(70),
            Constraint::Percentage(30)].as_ref(),
        )
        .split(main_chunks[0]);
 
    let mut rows   = Vec::<Row>::new();
    let mut sum_pack = 0;
    let passed_time = self.start_time.elapsed().as_secs_f64();
    match self.pack_stat.lock() {
      Err(_err) => (),
      Ok(mut _stat) =>  {
        for k in _stat.keys() {
          //stat_string_render += "  -- -- -- -- -- -- -- -- -- --\n";
          if _stat[k] != 0 {
            sum_pack += _stat[k];
            if k.contains("Heart"){
              rows.push(Row::new(vec![format!("  \u{1f493} {:.1}", _stat[k]),
                                      format!("{:.1}", (_stat[k] as f64)/passed_time,),
                                      format!("[{}]", k)]));
            } else {
              rows.push(Row::new(vec![format!("  \u{279f} {:.1}", _stat[k]),
                                      format!("{:.1}", (_stat[k] as f64)/passed_time,),
                                      format!("[{}]", k)]));
            }
          }
        }
      } 
    }
    rows.push(Row::new(vec!["  \u{FE4C}\u{FE4C}\u{FE4C}","\u{FE4C}\u{FE4C}","\u{FE4C}\u{FE4C}\u{FE4C}\u{FE4C}\u{FE4C}\u{FE4C}"])); 
    rows.push(Row::new(vec![format!("  \u{279f}{}", sum_pack),
                       format!("{:.1}/s", (sum_pack as f64)/passed_time),
                       format!("[TOTAL]")]));
    
    let widths = [Constraint::Percentage(30),
                  Constraint::Percentage(20),
                  Constraint::Percentage(50)];
    let table  = Table::new(rows, widths)
      .column_spacing(1)
      .header(
        Row::new(vec!["  N", "\u{1f4e6}/s", "Type"])
        .bottom_margin(1)
        .top_margin(1)
        .style(Style::new().add_modifier(Modifier::UNDERLINED))
      )
      .block(Block::new()
             .title("Packet summary \u{1f4e6}")
             .borders(Borders::ALL)
             .border_type(BorderType::Rounded)
             )
      .style(self.theme.style());

    let main_view = Paragraph::new(LIFTOF_LOGO_SHOW)
    .style(self.theme.style())
    .alignment(Alignment::Center)
    .block(
      Block::default()
        .borders(Borders::NONE)
    );
    
    match self.streamer.lock() {
      Err(_err) => (),
      Ok(mut _vecdeque) =>  {
        self.stream = _vecdeque
            .iter()
            .cloned() // Clone each string to avoid moving ownership
            .collect::<Vec<String>>()
            .join("\n");
        //if _vecdeque.len() > self.stream_max {
        while _vecdeque.len() > self.stream_max {
          _vecdeque.pop_front();
        }
      }, 
    }
    //let stream : String = String::from("");
    let side_view = Paragraph::new(self.stream.clone())
    .style(self.theme.style())
    .alignment(Alignment::Left)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Stream")
    );
    frame.render_widget(main_view,       upper_chunks[0]);
    frame.render_widget(side_view,       main_chunks[1]);
    frame.render_widget(table, upper_chunks[1])
  }
}

