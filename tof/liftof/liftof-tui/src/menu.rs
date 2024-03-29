

use ratatui::widgets::Tabs;
use ratatui::text::{Span, Line};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::terminal::Frame;
use ratatui::layout::Rect;

use crate::colors::{
    ColorTheme
};

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
  Home,
  //Status,
  //Alerts,
  //Commands,
  //Dashboard,
  TofEvents,
  TofSummary,
  TofHits,
  RBWaveform,
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
      MenuItem::TofSummary    => 3,
      MenuItem::RBWaveform    => 4,
      MenuItem::TofHits       => 5,
      //MenuItem::Alerts        => 2,
      //MenuItem::Commands      => 3,
      //MenuItem::Dashboard     => 4,
      MenuItem::MasterTrigger => 6,
      MenuItem::TOFCpu        => 7,
      MenuItem::Settings      => 8,
      MenuItem::Quit          => 9,
    }   
  }
}

#[derive(Debug, Clone)]
pub struct MainMenu {
  pub theme : ColorTheme,
  pub active_menu_item : MenuItem,
}

impl MainMenu {

  pub fn new(theme : ColorTheme) -> MainMenu {
    MainMenu {
      theme,
      active_menu_item : MenuItem::Home,
    }
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    let menu_titles  = vec!["Home", "TofEvents", "TofSummary", "RBWaveform", "TofHits",  "ReadoutBoards", "MasterTrigger", "CPUMoni" , "Settings", "Quit" ];
    let menu : Vec<Line> = menu_titles
               .iter()
               .map(|t| {
                 if t == &"TofSummary" {
                   // none of these handpicked strings has 0 len
                   let (rest, last) = t.split_at(t.len() - 1);
                   Line::from(vec![
                     Span::styled(rest, self.theme.style()),
                     Span::styled(
                         last,
                         Style::default()
                             .fg(self.theme.hc)
                             .add_modifier(Modifier::UNDERLINED),
                     ),
                   ])
                 } else if t == &"RBWaveform" {
                   // none of these handpicked strings has 0 len
                   let (a, b) = t.split_at(2);
                   let (highlight, c) = b.split_at(1);
                   Line::from(vec![
                     Span::styled(a, self.theme.style()),
                     Span::styled(
                         highlight,
                         Style::default()
                             .fg(self.theme.hc)
                             .add_modifier(Modifier::UNDERLINED),
                     ),
                     Span::styled(c, self.theme.style()),
                   ])
                 } else if t == &"TofHits" {
                   // none of these handpicked strings has 0 len
                   let (a, b) = t.split_at(2);
                   let (highlight, c) = b.split_at(1);
                   Line::from(vec![
                     Span::styled(a, self.theme.style()),
                     Span::styled(
                         highlight,
                         Style::default()
                             .fg(self.theme.hc)
                             .add_modifier(Modifier::UNDERLINED),
                     ),
                     Span::styled(c, self.theme.style()),
                   ])
                 } else {
                   let (first, rest) = t.split_at(1);
                   Line::from(vec![
                     Span::styled(
                         first,
                         Style::default()
                             .fg(self.theme.hc)
                             .add_modifier(Modifier::UNDERLINED),
                     ),
                     Span::styled(rest, self.theme.style()),
                   ])
                 }
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
  pub theme : ColorTheme,
  pub active_menu_item : MTMenuItem,
}

impl MTMenu {

  pub fn new(theme : ColorTheme) -> MTMenu  {
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
                           .fg(self.theme.hc)
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
  PAMoniData,
  PBMoniData,
  LTBMoniData,
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
      RBMenuItem::PAMoniData    => 4,
      RBMenuItem::PBMoniData    => 5,
      RBMenuItem::LTBMoniData   => 6,
      RBMenuItem::SelectRB      => 7,
      RBMenuItem::Quit          => 8,
    }   
  }
}

#[derive(Debug, Clone)]
pub struct RBMenu  {
  pub theme : ColorTheme,
  pub active_menu_item : RBMenuItem,
}

impl  RBMenu {

  pub fn new(theme : ColorTheme) -> RBMenu {
    RBMenu {
      theme,
      active_menu_item : RBMenuItem::Home,
    }
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    let menu_titles = vec!["Home", "Info", "Waveforms", "RBMoniData", "PAMoniData", "PBMoniData", "LTBMoniData", "SelectRB", "Quit" ];
    let menu : Vec<Line> = menu_titles
               .iter()
               .map(|t| {
                 if t == &"Paddles" || t == &"Hits" {
                   let (second, rest) = t.split_at(2);
                   Line::from(vec![
                     Span::styled(
                       second,
                       Style::default()
                         .fg(self.theme.hc)
                         .add_modifier(Modifier::UNDERLINED),
                     ),
                     Span::styled(rest, self.theme.style()),
                   ])
                 } else {
                   let (first, rest) = t.split_at(1);
                   Line::from(vec![
                     Span::styled(
                         first,
                         Style::default()
                             .fg(self.theme.hc)
                             .add_modifier(Modifier::UNDERLINED),
                     ),
                     Span::styled(rest, self.theme.style()),
                   ])
                 }
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
  pub theme : ColorTheme,
  pub active_menu_item : SettingsMenuItem,
}

impl  SettingsMenu {

  pub fn new(theme : ColorTheme) -> SettingsMenu {
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

///////////////////////////////////////////

#[derive(Debug, Copy, Clone)]
pub enum TSMenuItem {
  Home,
  Quit
}

impl From<TSMenuItem> for usize {
  fn from(input: TSMenuItem) -> usize {
    match input {
      TSMenuItem::Home          => 0,
      TSMenuItem::Quit          => 1,
    }   
  }
}

#[derive(Debug, Clone)]
pub struct TSMenu {
  pub theme : ColorTheme,
  pub active_menu_item : TSMenuItem,
}

impl TSMenu {

  pub fn new(theme : ColorTheme) -> TSMenu  {
    TSMenu {
      theme,
      active_menu_item : TSMenuItem::Home,
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
                           .fg(self.theme.hc)
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
pub enum RWMenuItem {
  Home,
  Quit
}

impl From<RWMenuItem> for usize {
  fn from(input: RWMenuItem) -> usize {
    match input {
      RWMenuItem::Home          => 0,
      RWMenuItem::Quit          => 1,
    }   
  }
}

#[derive(Debug, Clone)]
pub struct RWMenu {
  pub theme : ColorTheme,
  pub active_menu_item : RWMenuItem,
}

impl RWMenu {

  pub fn new(theme : ColorTheme) -> RWMenu  {
    RWMenu {
      theme,
      active_menu_item : RWMenuItem::Home,
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
                           .fg(self.theme.hc)
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
pub enum THMenuItem {
  Home,
  Hits,
  Pulses,
  Paddles,
  SelectPaddle,
  Quit
}

impl From<THMenuItem> for usize {
  fn from(input: THMenuItem) -> usize {
    match input {
      THMenuItem::Home          => 0,
      THMenuItem::Hits          => 1,
      THMenuItem::Pulses        => 2,
      THMenuItem::Paddles       => 3,
      THMenuItem::SelectPaddle  => 4,
      THMenuItem::Quit          => 5,
    }   
  }
}

#[derive(Debug, Clone)]
pub struct THMenu {
  pub theme : ColorTheme,
  pub active_menu_item : THMenuItem,
}

impl THMenu {

  pub fn new(theme : ColorTheme) -> THMenu  {
    THMenu {
      theme,
      active_menu_item : THMenuItem::Home,
    }
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    let menu_titles : Vec<&str> = vec!["Home", "Hits", "Pulses", "Paddles", "SelectPaddle", "Quit" ];
    let menu : Vec<Line> = menu_titles
               .iter()
               .map(|t| {
                 if t == &"Paddles" || t == &"Hits" {
                   let (second, rest) = t.split_at(2);
                   Line::from(vec![
                     Span::styled(
                         second,
                         Style::default()
                             .fg(self.theme.hc)
                             .add_modifier(Modifier::UNDERLINED),
                     ),
                     Span::styled(rest, self.theme.style()),
                   ])
                 } else {
                   let (first, rest) = t.split_at(1);
                   Line::from(vec![
                     Span::styled(
                         first,
                         Style::default()
                             .fg(self.theme.hc)
                             .add_modifier(Modifier::UNDERLINED),
                     ),
                     Span::styled(rest, self.theme.style()),
                   ])
                 }
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


