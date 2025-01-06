//! Overview over precarious conditions
//! - Alert system

use std::time::{
  Instant,
  //Duration,
};

use std::sync::{
  Arc,
  Mutex,
};
use std::collections::HashMap;

use ratatui::layout::Rect;
use ratatui::Frame;
use ratatui::prelude::*;
use ratatui::style::{
  Style,
  Modifier
};
use ratatui::widgets::{
  Block, 
  Borders,
  BorderType,
  Cell,
  //HighlightSpacing,
  Paragraph,
  Row, 
  //Scrollbar,
  //ScrollbarOrientation,
  //ScrollbarState,
  Table, 
  TableState,
};


use tof_dataclasses::alerts::TofAlert;

use crate::colors::ColorTheme;

#[derive(Debug, Clone)]
pub struct AlertTab<'a> {
  pub theme       : ColorTheme,
  pub alerts      : Arc<Mutex<HashMap<&'a str, TofAlert<'a>>>>,
  //pub active_al   : Vec<TofAlert<'a>>,
  met             : Instant,
  active_alerts   : Vec<TofAlert<'a>>,
  table_state     : TableState,
  check_interval  : Instant,
  check_every     : u32,
}

impl AlertTab<'_> {
  pub fn new<'a>(theme  : ColorTheme,
                 alerts : Arc<Mutex<HashMap<&'a str,TofAlert<'a>>>>) -> AlertTab<'a> {
  
    AlertTab {
      theme,
      alerts,
      //active_al    :  Vec::<TofAlert<'a>>::new(),
      met            : Instant::now(),
      table_state    : TableState::default(),
      active_alerts  : Vec::<TofAlert<'a>>::new(),
      check_interval : Instant::now(),
      check_every    : 10,
    }
  }

  pub fn check_alert_state(&mut self) {
    let no_alarm_before  = 120u64;
    if self.check_interval.elapsed().as_secs() > self.check_every as u64 {
      match self.alerts.lock() {
        Ok(mut al) => {
          let mut triggered = Vec::<&str>::new();
          for k in al.keys() {
            if al[k].has_triggered() {
              triggered.push(k);
            }
          }
          if self.met.elapsed().as_secs() < no_alarm_before {
            for k in triggered {
              al.get_mut(k).unwrap().acknowledge();
            }
          } else {
            for k in triggered {
              if !self.active_alerts.contains(&al[k]) {
                self.active_alerts.push(al[k].clone());
                if al.get(k).unwrap().n_paged == 0 {
                  al.get_mut(k).unwrap().page();
                }
              }
            }
          }
        }
        Err(err) => {
          error!("Unable to lock alert mutex! Alert state unknown! {err}");
        }
      }
      self.check_interval = Instant::now();
    }
  }

  pub fn next_row(&mut self) {
    info!("Selecting next row!");
    let i = match self.table_state.selected() {
      Some(i) => {
        if i >= self.active_alerts.len() - 1 {
          0
        } else {
          i + 1
        }
      }
      None => 0,
    };
    self.table_state.select(Some(i));
    //self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
  }

  pub fn previous_row(&mut self) {
    info!("Selecting previous row!");
    let i = match self.table_state.selected() {
      Some(i) => {
        if i == 0 {
          self.active_alerts.len() - 1
        } else {
          i - 1
        }
      }
      None => 0,
    };
    self.table_state.select(Some(i));
    //self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {

     let selected_row_style = Style::default()
       .add_modifier(Modifier::REVERSED)
       .fg(self.theme.hc);
    
    let main_lo = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
          [Constraint::Percentage(30),
           Constraint::Percentage(70)].as_ref(),
      )
      .split(*main_window);
    let right_col = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
          [Constraint::Percentage(40),
           Constraint::Percentage(60)].as_ref(),
      )
      .split(main_lo[1]);

    // suppress alarms at program start
    let no_alarm_before  = 120u64;
    // keep a copy of active alerts
    let mut fallback     = String::from("You're lucky! Everything is Awesome!\n (... or we haven't yet had the chance to implement this features yet, ore we are not catching the fact that something is misbehaving ... )");
    if self.met.elapsed().as_secs() < no_alarm_before {
      // delete all alarms!
      fallback = format!("Alarms will only be available 2mins after program start since it might take a bit until everything is settled!\n So far {}s have passed!", self.met.elapsed().as_secs());
    }

    let table_title = format!("Current active alerts \u{26A0} ({})", self.active_alerts.len());
    if self.active_alerts.is_empty() {
      let main_view = Paragraph::new(fallback)
      .style(self.theme.style())
      .alignment(Alignment::Center)
      .block(
        Block::default()
          .borders(Borders::ALL)
          .border_type(BorderType::Thick)
      );
      frame.render_widget(main_view, *main_window);
      return;
    }


    //let header_style = Style::default();
        //.fg(self.colors.header_fg)
        //.bg(self.colors.header_bg);
    //let selected_row_style = Style::default();
        //.add_modifier(Modifier::REVERSED)
        //.fg(self.colors.selected_row_style_fg);
    //let selected_col_style = Style::default().fg(self.colors.selected_column_style_fg);
    
    //let selected_cell_style = Style::default()
    //    .add_modifier(Modifier::REVERSED)
    //    .fg(self.colors.selected_cell_style_fg);

    //let header = ["What", "Variable", "Condition", "Description", "Required Action?"]
    //  .into_iter()
    //  .map(Cell::from)
    //  .collect::<Row>()
    //  //.style(header_style)
    //  .style(self.theme.style())
    //  .height(1);
    //let rows = alerts.iter().enumerate().map(|(i, data)| {
    let mut rows   = Vec::<Row>::new();
    for ale in &self.active_alerts {
      rows.push(Row::new(vec![Cell::from(Text::from(format!("{}", ale)))]));
    }
    let widths = [Constraint::Percentage(100)];
    let table  = Table::new(rows, widths)
      .column_spacing(1)
      .row_highlight_style(selected_row_style)
      .header(
        Row::new(vec![" ALERT "])
        .bottom_margin(1)
        .top_margin(1)
        .style(Style::new().add_modifier(Modifier::UNDERLINED))
      )
      .block(Block::new()
        .title(table_title)
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        )
      .style(self.theme.style());
    frame.render_stateful_widget(table, main_lo[0], &mut self.table_state);
   
    match self.table_state.selected() {
      Some(trow) => {
        let descr_view = Paragraph::new(self.active_alerts[trow].descr)
          .style(self.theme.style())
          .alignment(Alignment::Left)
          .block(
            Block::default()
              .borders(Borders::ALL)
              .border_type(BorderType::Thick)
        );
        frame.render_widget(descr_view, right_col[0]);
        let mut message = String::from("");
        for k in &self.active_alerts[trow].whattodo {
          message += k;
        }
        let whattodo_view = Paragraph::new(message)
          .style(self.theme.style())
          .alignment(Alignment::Left)
          .block(
            Block::default()
              .borders(Borders::ALL)
              .border_type(BorderType::Thick)
        );
        frame.render_widget(whattodo_view, right_col[1]);
      }
      None => ()
    }
    //let whattodo_view = Paragraph::new(self.active_alerts[self.table_state as usize].whattodo)
    //  .style(self.theme.style())
    //  .alignment(Alignment::Center)
    //  .block(
    //    Block::default()
    //      .borders(Borders::ALL)
    //      .border_type(BorderType::Thick)
    //);
    //frame.render_widget(whattodo_view, right_col[1]);
    
    //    let color = match i % 2 {
    //        0 => self.colors.normal_row_color,
    //        _ => self.colors.alt_row_color,
    //    };
    //    let item = data.ref_array();
    //    item.into_iter()
    //        .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
    //        .collect::<Row>()
    //        .style(Style::new().fg(self.colors.row_fg).bg(color))
    //        .height(4)
    //});
    //let bar = " █ ";
    //let t = Table::new(
    //    rows,
    //    [
    //        // + 1 is for padding.
    //        Constraint::Length(self.longest_item_lens.0 + 1),
    //        Constraint::Min(self.longest_item_lens.1 + 1),
    //        Constraint::Min(self.longest_item_lens.2),
    //    ],
    //)
    //.header(header)
    //.row_highlight_style(selected_row_style)
    //.column_highlight_style(selected_col_style)
    //.cell_highlight_style(selected_cell_style)
    //.highlight_symbol(Text::from(vec![
    //    "".into(),
    //    bar.into(),
    //    bar.into(),
    //    "".into(),
    //]))
    //.bg(self.colors.buffer_bg)
    //.highlight_spacing(HighlightSpacing::Always);
    //frame.render_stateful_widget(t, main_window, &mut self.state);
  }
}
