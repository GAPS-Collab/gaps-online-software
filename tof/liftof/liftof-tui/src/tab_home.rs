use std::collections::{
    HashMap,
    VecDeque,
};
use std::sync::{
    Arc,
    Mutex,
};
use ratatui::prelude::*;

use ratatui::symbols;
use ratatui::text::Span;
use ratatui::terminal::Frame;
use ratatui::layout::Rect;
use ratatui::style::{
    Color,
    Style,
};
use ratatui::widgets::{
    Block,
    BorderType,
    Borders,
    Paragraph,
};

use crate::colors::ColorTheme2;

const logo : &str = "
                                  ___                         ___           ___     
                                 /\\__\\                       /\\  \\         /\\__\\    
                    ___         /:/ _/_         ___         /::\\  \\       /:/ _/_   
                   /\\__\\       /:/ /\\__\\       /\\__\\       /:/\\:\\  \\     /:/ /\\__\\  
    ___     ___   /:/__/      /:/ /:/  /      /:/  /      /:/  \\:\\  \\   /:/ /:/  /  
   /\\  \\   /\\__\\ /::\\  \\     /:/_/:/  /      /:/__/      /:/__/ \\:\\__\\ /:/_/:/  /   
   \\:\\  \\ /:/  / \\/\\:\\  \\__  \\:\\/:/  /      /::\\  \\      \\:\\  \\ /:/  / \\:\\/:/  /    
    \\:\\  /:/  /   ~~\\:\\/\\__\\  \\::/__/      /:/\\:\\  \\      \\:\\  /:/  /   \\::/__/     
     \\:\\/:/  /       \\::/  /   \\:\\  \\      \\/__\\:\\  \\      \\:\\/:/  /     \\:\\  \\     
      \\::/  /        /:/  /     \\:\\__\\          \\:\\__\\      \\::/  /       \\:\\__\\    
       \\/__/         \\/__/       \\/__/           \\/__/       \\/__/         \\/__/    

          (LIFTOF - liftof is for tof, Version 0.8 'NIUHI', Dec 2023)

          * Documentation
          ==> GitHub   https://github.com/GAPS-Collab/gaps-online-software/tree/NIUHI-0.8
          ==> API docs https://gaps-collab.github.io/gaps-online-software/

";

#[derive(Debug, Clone)]
pub struct HomeTab {
  pub theme      : ColorTheme2,
  pub streamer   : Arc<Mutex<VecDeque<String>>>,
  pub pack_stat  : Arc<Mutex<HashMap<String, usize>>>,
  pub stream     : String,
  pub stream_max : usize, 
}

impl HomeTab {
  pub fn new(theme     : ColorTheme2,
             streamer  : Arc<Mutex<VecDeque<String>>>,
             pack_stat : Arc<Mutex<HashMap<String,usize>>>) -> HomeTab {
    HomeTab {
      theme,
      streamer, 
      pack_stat,
      stream     : String::from(""),
      stream_max : 30,
    }
  }

  // Color::Blue was nice for background
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
    
    let mut stat_string_render = String::from("\n\n");
    let mut sum_pack = 0;
    match self.pack_stat.lock() {
      Err(_err) => (),
      Ok(mut _stat) =>  {
        for k in _stat.keys() {
          //stat_string_render += "  -- -- -- -- -- -- -- -- -- --\n";
          if _stat[k] != 0 {
            let line = format!("  {} \t=> [{}]\n",  _stat[k],k);
            stat_string_render += &line;
            sum_pack += _stat[k];
          }
        }
      }, 
    }
    stat_string_render += "  == == == == == == ==\n";
    let sum_string      = format!("  {} [total]", sum_pack); 
    stat_string_render += &sum_string; 

    let statistics_view = Paragraph::new(stat_string_render)
    .style(self.theme.style())
    .alignment(Alignment::Left)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Received Packets")
    );
        

    let main_view = Paragraph::new(logo)
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
    frame.render_widget(statistics_view, upper_chunks[1])
  }
}

