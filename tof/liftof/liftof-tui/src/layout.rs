//! Main laoyt of the app

use ratatui::layout::{
  Constraint,
  Direction,
  Layout,
  Rect
};

#[derive(Debug, Clone)]
pub struct MainLayout {
  pub menu : Rect,
  pub main : Rect,
  pub log  : Rect,
  pub help : Rect
}

impl MainLayout {

  pub fn new(size : Rect) -> MainLayout {
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
    // logs and help
    let logs_n_help = Layout::default()
    .direction(Direction::Horizontal)
    .constraints(
      [Constraint::Percentage(80),
       Constraint::Percentage(20)]
       .as_ref(),
    )
    .split(chunks[2]);  
    MainLayout {
      //rect : chunks.to_vec()
      menu : chunks[0],
      main : chunks[1],
      log  : logs_n_help[0],
      help : logs_n_help[1],
    }
  }
}

