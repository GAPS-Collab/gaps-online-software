

use ratatui::widgets::Tabs;
use ratatui::text::{Span, Line};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::terminal::Frame;
use ratatui::layout::Rect;

use crate::colors::ColorTheme2;

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
  Home,
  //Status,
  //Alerts,
  //Commands,
  //Dashboard,
  TofEvents,
  ReadoutBoards,
  MasterTrigger,
  TOFCpu, 
  Settings,
  Quit,
}


impl From<MenuItem> for usize {
  fn from(input: MenuItem) -> usize {
    match input {
      MenuItem::Home          => 0,
      MenuItem::TofEvents     => 1,
      MenuItem::ReadoutBoards => 2,
      //MenuItem::Alerts        => 2,
      //MenuItem::Commands      => 3,
      //MenuItem::Dashboard     => 4,
      MenuItem::MasterTrigger => 3,
      MenuItem::TOFCpu        => 4,
      MenuItem::Settings      => 5,
      MenuItem::Quit          => 6,
    }   
  }
}

#[derive(Debug, Clone)]
pub struct MainMenu {
  pub theme : ColorTheme2,
  pub active_menu_item : MenuItem,
}

impl MainMenu {

  pub fn new(theme : ColorTheme2) -> MainMenu {
    MainMenu {
      theme,
      active_menu_item : MenuItem::Home,
    }
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    let menu_titles  = vec!["Home", "TofEvents", "ReadoutBoards", "MasterTrigger", "CPUMoni" , "Settings", "Quit" ];
    let menu : Vec<Line> = menu_titles
               .iter()
               .map(|t| {
                 let (first, rest) = t.split_at(1);
                 Line::from(vec![
                   Span::styled(
                       first,
                       Style::default()
                           .fg(Color::Yellow)
                           .add_modifier(Modifier::UNDERLINED),
                   ),
                   Span::styled(rest, self.theme.style()),
                 ])
               })
               .collect();

    let tabs = Tabs::new(menu)
        .select(self.active_menu_item.into())
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(self.theme.style())
        .highlight_style(self.theme.highlight())
        .divider(Span::raw("|"));
    frame.render_widget(tabs, *main_window);
  }
}

///////////////////////////////////////////

#[derive(Debug, Copy, Clone)]
pub enum MTMenuItem {
  Home,
  Quit
}

impl From<MTMenuItem> for usize {
  fn from(input: MTMenuItem) -> usize {
    match input {
      MTMenuItem::Home          => 0,
      MTMenuItem::Quit          => 1,
    }   
  }
}

#[derive(Debug, Clone)]
pub struct MTMenu {
  pub theme : ColorTheme2,
  pub active_menu_item : MTMenuItem,
}

impl MTMenu {

  pub fn new(theme : ColorTheme2) -> MTMenu  {
    MTMenu {
      theme,
      active_menu_item : MTMenuItem::Home,
    }
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    let menu_titles : Vec<&str> = vec!["Home", "Quit" ];
    let menu : Vec<Line> = menu_titles
               .iter()
               .map(|t| {
                 let (first, rest) = t.split_at(1);
                 Line::from(vec![
                   Span::styled(
                       first,
                       Style::default()
                           .fg(Color::Yellow)
                           .add_modifier(Modifier::UNDERLINED),
                   ),
                   Span::styled(rest, self.theme.style()),
                 ])
               })
               .collect();

    let tabs = Tabs::new(menu)
        .select(self.active_menu_item.into())
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(self.theme.style())
        .highlight_style(self.theme.highlight())
        .divider(Span::raw("|"));
    frame.render_widget(tabs, *main_window);
  }
}

///////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RBMenuItem {
  Home,
  Info,
  Waveforms,
  RBMoniData,
  SelectRB,
  Quit,
}

impl From<RBMenuItem> for usize {
  fn from(input: RBMenuItem) -> usize {
    match input {
      RBMenuItem::Home          => 0,
      RBMenuItem::Info          => 1,
      RBMenuItem::Waveforms     => 2,
      RBMenuItem::RBMoniData    => 3,
      RBMenuItem::SelectRB      => 4,
      RBMenuItem::Quit          => 5,
    }   
  }
}

#[derive(Debug, Clone)]
pub struct RBMenu  {
  pub theme : ColorTheme2,
  pub active_menu_item : RBMenuItem,
}

impl  RBMenu {

  pub fn new(theme : ColorTheme2) -> RBMenu {
    RBMenu {
      theme,
      active_menu_item : RBMenuItem::Home,
    }
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    let menu_titles = vec!["Home", "Info", "Waveforms", "RBMoniData", "SelectRB", "Quit" ];
    let menu : Vec<Line> = menu_titles
               .iter()
               .map(|t| {
                 let (first, rest) = t.split_at(1);
                 Line::from(vec![
                   Span::styled(
                       first,
                       Style::default()
                           .fg(Color::Yellow)
                           .add_modifier(Modifier::UNDERLINED),
                   ),
                   Span::styled(rest, self.theme.style()),
                 ])
               })
               .collect();

    let tabs = Tabs::new(menu)
        .select(self.active_menu_item.into())
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(self.theme.style())
        .highlight_style(self.theme.highlight())
        .divider(Span::raw("|"));
    frame.render_widget(tabs, *main_window);
  }
}

///////////////////////////////////////////

#[derive(Debug, Copy, Clone)]
pub enum SettingsMenuItem {
  Home,
  Quit
}

impl From<SettingsMenuItem> for usize {
  fn from(input: SettingsMenuItem) -> usize {
    match input {
      SettingsMenuItem::Home          => 0,
      SettingsMenuItem::Quit          => 1,
    }   
  }
}

#[derive(Debug, Clone)]
pub struct SettingsMenu {
  pub theme : ColorTheme2,
  pub active_menu_item : SettingsMenuItem,
}

impl  SettingsMenu {

  pub fn new(theme : ColorTheme2) -> SettingsMenu {
    SettingsMenu {
      theme,
      active_menu_item : SettingsMenuItem::Home,
    }
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    let menu_titles : Vec<&str> = vec!["Home", "Quit" ];
    let menu : Vec<Line> = menu_titles
               .iter()
               .map(|t| {
                 let (first, rest) = t.split_at(1);
                 Line::from(vec![
                   Span::styled(
                       first,
                       Style::default()
                           .fg(Color::Yellow)
                           .add_modifier(Modifier::UNDERLINED),
                   ),
                   Span::styled(rest, self.theme.style()),
                 ])
               })
               .collect();

    let tabs = Tabs::new(menu)
        .select(self.active_menu_item.into())
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(self.theme.style())
        .highlight_style(self.theme.highlight())
        .divider(Span::raw("|"));
    frame.render_widget(tabs, *main_window);
  }
}

