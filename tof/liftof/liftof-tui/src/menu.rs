

use tui::widgets::Tabs;
use tui::text::{Span, Spans};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders};


#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
  Home,
  Status,
  Alerts,
  Commands,
  Dashboard,
  Logs
}


impl From<MenuItem> for usize {
  fn from(input: MenuItem) -> usize {
    match input {
      MenuItem::Home      => 0,
      MenuItem::Status    => 1,
      MenuItem::Alerts    => 2,
      MenuItem::Commands  => 3,
      MenuItem::Dashboard => 4,
      MenuItem::Logs      => 5
    }   
  }
}


#[derive(Debug, Clone)]
pub struct Menu<'a> {
  // FIXME - array
  pub menu_titles : Vec::<&'static str>,
  pub active_menu_item : MenuItem,
  pub tabs : Tabs<'a>
}

impl Menu<'_> {

  pub fn new<'a>() -> Menu<'a> {
    let menu_titles = vec!["Home", "RBStatus", "Commands", "Alerts", "Dashboard", "Logs" ];
    let menu = menu_titles
    .iter()
    .map(|t| {
      let (first, rest) = t.split_at(1);
      Spans::from(vec![
        Span::styled(
            first,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::UNDERLINED),
        ),
        Span::styled(rest, Style::default().fg(Color::White)),
      ])
    })
    .collect();

    let active_menu_item = MenuItem::Home;
    let tabs = Tabs::new(menu)
        .select(active_menu_item.into())
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider(Span::raw("|"));
    Menu { 
      menu_titles,
      active_menu_item,
      tabs 
    }
  }




}
