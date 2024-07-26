//! Menus for the main application and 
//! individual tabs

use ratatui::widgets::Tabs;
use ratatui::text::{Span, Line};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::terminal::Frame;
use ratatui::layout::Rect;

use crate::colors::{
    ColorTheme
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ActiveMenu {
  MainMenu,
  RBMenu,
  Paddles,
  Trigger,
  Events,
  Monitoring,
  Heartbeats,
}

#[derive(Copy, Clone, Debug, PartialEq)]
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
  Telemetry,
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
      MenuItem::Telemetry     => 8,
      MenuItem::Settings      => 9,
      MenuItem::Quit          => 10,
    }   
  }
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UIMenuItem {
  Unknown,
  // main menu
  Home,
  Back,
  Events,
  ReadoutBoards,
  Paddles,
  Trigger,
  Monitoring,
  Telemetry,
  Commands,
  Settings,
  Quit,
  // rb menu
  Waveforms,
  RBMoniData,
  PAMoniData,
  PBMoniData,
  LTBMoniData,
  SelectBoard,
  // event menu
  TofSummary,
  TofEvents,
  TofHits,
  RBWaveform,
  // moni menu
  PreampBias,
  PreampTemp,
  LTBThresholds,
  // paddle menu
  Signal,
  RecoVars,
  // heartbeats
  Heartbeats,
  EventBuilderHB,
  TriggerHB,
  DataSenderHB,
}

impl UIMenuItem {

  pub fn get_title(&self, theme : ColorTheme) -> String {
    match self {
      UIMenuItem::Unknown        => String::from("Unknown"       ),
      UIMenuItem::Home           => String::from("Home"          ),
      UIMenuItem::Back           => String::from("Back"          ),
      UIMenuItem::Events         => String::from("Events"        ),
      UIMenuItem::ReadoutBoards  => String::from("ReadoutBoards" ),
      UIMenuItem::Trigger        => String::from("Trigger"       ),
      UIMenuItem::Monitoring     => String::from("Monitoring"    ),
      UIMenuItem::Telemetry      => String::from("Telemetry"     ),
      UIMenuItem::Settings       => String::from("Settings"      ),
      UIMenuItem::Commands       => String::from("Commands"      ),
      UIMenuItem::Quit           => String::from("Quit"          ),
      UIMenuItem::Waveforms      => String::from("Waveforms"     ),
      UIMenuItem::RBMoniData     => String::from("RBMoniData"    ),
      UIMenuItem::PAMoniData     => String::from("PAMoniData"    ),
      UIMenuItem::PBMoniData     => String::from("PBMoniData"    ),
      UIMenuItem::LTBMoniData    => String::from("LTBMoniData"   ),
      UIMenuItem::SelectBoard    => String::from("SelectBoard"   ),
      UIMenuItem::TofSummary     => String::from("TofSummary"    ),
      UIMenuItem::TofEvents      => String::from("TofEvents"     ),
      UIMenuItem::TofHits        => String::from("TofHits"       ),
      UIMenuItem::RBWaveform     => String::from("RBWaveform"    ),
      UIMenuItem::PreampBias     => String::from("Preamp Bias Voltages"   ),
      UIMenuItem::PreampTemp     => String::from("Preamp Temps"),
      UIMenuItem::LTBThresholds  => String::from("LTBThresholds"),
      UIMenuItem::Quit           => String::from("Quit"),
      UIMenuItem::Paddles        => String::from("Paddles"),
      UIMenuItem::Signal         => String::from("Wf & Charge"),
      UIMenuItem::RecoVars       => String::from("Reco Vars"),
      UIMenuItem::Heartbeats     => String::from("Heartbeats"),
      UIMenuItem::EventBuilderHB => String::from("EventBuilderHB"),
      UIMenuItem::TriggerHB      => String::from("TriggerHB"),
      UIMenuItem::DataSenderHB   => String::from("DataSenderHB"),
      _ => String::from("Unknown"),
    } 
  }
}


pub trait UIMenu<'a> {

  fn get_max_idx() -> usize {
    Self::get_items().len() - 1
  }

  fn get_items() -> Vec<UIMenuItem>;

  fn get_active_menu_item(&self) -> UIMenuItem;

  fn set_active_menu_item(&mut self, item : UIMenuItem);

  fn get_active_idx(&self) -> usize;

  fn set_active_idx(&mut self, idx : usize);
 
  fn get_theme(&self) -> ColorTheme;

  fn get_titles(theme : ColorTheme) -> Vec<Line<'a>> {
    let mut titles = Vec::<Line>::new();
    for item in Self::get_items().clone() {
      let ti = item.get_title(theme).clone();
      let line =  Line::from(vec![Span::styled(ti, theme.style()),]);
      titles.push(line);
    }
    titles
  }

  fn next(&mut self) {
    if self.get_active_idx() + 1 > Self::get_max_idx() {
      self.set_active_menu_item(Self::get_items()[0]);
      self.set_active_idx(0);
    } else {
      self.set_active_menu_item(Self::get_items()[self.get_active_idx() + 1]);
      self.set_active_idx(self.get_active_idx() + 1);
    }
  }

  fn prev(&mut self) {
    if self.get_active_idx() == 0 {
      self.set_active_menu_item(Self::get_items()[Self::get_max_idx()]);
      self.set_active_idx(Self::get_max_idx());
    } else {
      self.set_active_menu_item(Self::get_items()[self.get_active_idx() -1]);
      self.set_active_idx(self.get_active_idx() - 1);
    }
  }
  
  fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    let theme = self.get_theme();
    let tabs = Tabs::new(Self::get_titles(theme))
      .select(self.get_active_idx())
      .block(Block::default().title("Menu").borders(Borders::ALL))
      .style(self.get_theme().style())
      .highlight_style(self.get_theme().highlight())
      .divider(Span::raw("|"));
    frame.render_widget(tabs, *main_window);
  }
}


#[derive(Debug, Clone)]
pub struct MainMenu2<'a> {
  pub theme        : ColorTheme,
  pub active_index : usize,
  pub titles       : Vec<Line<'a>>,
  pub active_menu_item : MenuItem,
  pub active_menu_item2 : UIMenuItem,
}

impl UIMenu<'_> for MainMenu2<'_> {
  fn get_items() -> Vec<UIMenuItem> {
    let items = vec![UIMenuItem::Home,
                     UIMenuItem::Events,
                     UIMenuItem::ReadoutBoards,
                     UIMenuItem::Paddles,
                     UIMenuItem::Trigger,
                     UIMenuItem::Monitoring,
                     UIMenuItem::Heartbeats,
                     UIMenuItem::Telemetry,
                     UIMenuItem::Commands,
                     UIMenuItem::Settings,
                     UIMenuItem::Quit];
    items
  }

  fn get_theme(&self) -> ColorTheme {
    self.theme.clone()
  }

  fn set_active_menu_item(&mut self, item : UIMenuItem) {
    self.active_menu_item2 = item;
  }

  fn set_active_idx(&mut self, idx : usize) {
    self.active_index = idx;
  }

  fn get_active_idx(&self) -> usize {
    self.active_index
  }
  
  //fn get_titles(&self) -> Vec<Line> {
  //  self.titles.clone()
  //}
  fn get_active_menu_item(&self) -> UIMenuItem {
    self.active_menu_item2
  }
}

impl MainMenu2<'_> {
  pub fn new(theme : ColorTheme) -> Self {
    let titles = Self::get_titles(theme);
    let theme_cl = theme.clone();
    Self {
      theme : theme_cl,
      active_index : 0,
      titles,
      active_menu_item : MenuItem::Home,
      active_menu_item2 : UIMenuItem::Home,
    }
  }
}

//======================================

#[derive(Debug, Clone)]
pub struct RBMenu2<'a>  {
  pub theme             : ColorTheme,
  pub active_menu_item  : RBMenuItem,
  pub active_menu_item2 : UIMenuItem,
  pub active_index      : usize, 
  pub titles            : Vec<Line<'a>>,
}

impl UIMenu<'_> for RBMenu2<'_> {
  
  fn get_items() -> Vec<UIMenuItem> {
    let items = vec![UIMenuItem::Back,
                     UIMenuItem::Waveforms,
                     UIMenuItem::RBMoniData,
                     UIMenuItem::PBMoniData,
                     UIMenuItem::PAMoniData,
                     UIMenuItem::LTBMoniData,
                     //UIMenuItem::SelectBoard,
                     UIMenuItem::Quit];
    items
  }

  fn get_theme(&self) -> ColorTheme {
    self.theme
  }

  fn set_active_idx(&mut self, idx : usize) {
    self.active_index = idx;
  }

  fn get_active_idx(&self) -> usize {
    self.active_index
  }
  
  fn set_active_menu_item(&mut self, item : UIMenuItem) {
    self.active_menu_item2 = item;
  }
  
  //fn get_titles(&self) -> Vec<Line> {
  //  self.titles.clone()
  //}
  
  fn get_active_menu_item(&self) -> UIMenuItem {
    match self.active_index {
      0 => UIMenuItem::Back,
      1 => UIMenuItem::Waveforms,
      2 => UIMenuItem::RBMoniData,
      3 => UIMenuItem::PAMoniData,
      4 => UIMenuItem::PBMoniData,
      5 => UIMenuItem::LTBMoniData,
      6 => UIMenuItem::SelectBoard,
      7 => UIMenuItem::Quit,
      _ => UIMenuItem::Unknown
    }
  }
}

impl  RBMenu2<'_> {

  pub fn new(theme : ColorTheme) -> Self {
    let title_str  =  vec!["Back", "Waveforms",
                           "RBMoniData", 
                           "PBMoniData", "PAMoniData",
                           "LTBMoniData",
                           "SelectBoards [LTB & RB]",
                           "Quit"];

    let titles : Vec<Line> = title_str
                .iter()
                .map(|t| {
                   Line::from(vec![
                     Span::styled(*t, theme.style()),
                   ])
                })
                .collect();
 
    let n_titles = titles.len();
    Self {
      theme,
      active_index : 0,
      titles,
      active_menu_item : RBMenuItem::Home,
      active_menu_item2 : UIMenuItem::Home,
    }
  }
}

#[derive(Debug, Clone)]
pub struct TriggerMenu<'a>  {
  pub theme            : ColorTheme,
  pub active_menu_item : UIMenuItem,
  pub active_index     : usize, 
  pub titles           : Vec<Line<'a>>,
}

impl UIMenu<'_> for TriggerMenu<'_> {
  
  fn get_items() -> Vec<UIMenuItem> {
    let items = vec![UIMenuItem::Back,
                     UIMenuItem::Quit];
    items
  }

  fn get_theme(&self) -> ColorTheme {
    self.theme
  }

  fn set_active_idx(&mut self, idx : usize) {
    self.active_index = idx;
  }

  fn get_active_idx(&self) -> usize {
    self.active_index
  }
  
  fn set_active_menu_item(&mut self, item : UIMenuItem) {
    self.active_menu_item = item;
  }
  
  fn get_active_menu_item(&self) -> UIMenuItem {
    self.active_menu_item
  }
}

impl  TriggerMenu<'_> {

  pub fn new(theme : ColorTheme) -> Self {
    let theme_c = theme.clone();
    let titles  = Self::get_titles(theme_c);
    let n_titles = titles.len();
    Self {
      theme,
      active_index : 0,
      titles,
      active_menu_item : UIMenuItem::Back,
    }
  }
}

//============================================

#[derive(Debug, Clone)]
pub struct EventMenu<'a>  {
  pub theme            : ColorTheme,
  pub active_menu_item : UIMenuItem,
  pub active_index     : usize, 
  pub titles           : Vec<Line<'a>>,
}

impl UIMenu<'_> for EventMenu<'_> {
  
  fn get_items() -> Vec<UIMenuItem> {
    let items = vec![UIMenuItem::Back,
                     UIMenuItem::TofSummary,
                     UIMenuItem::TofHits,
                     UIMenuItem::RBWaveform,
                     UIMenuItem::Quit];
    items
  }

  fn get_theme(&self) -> ColorTheme {
    self.theme
  }

  fn set_active_idx(&mut self, idx : usize) {
    self.active_index = idx;
  }

  fn get_active_idx(&self) -> usize {
    self.active_index
  }
  
  fn set_active_menu_item(&mut self, item : UIMenuItem) {
    self.active_menu_item = item;
  }
  
  fn get_active_menu_item(&self) -> UIMenuItem {
    self.active_menu_item
  }
}

impl  EventMenu<'_> {

  pub fn new(theme : ColorTheme) -> Self {
    let theme_c = theme.clone();
    let titles  = Self::get_titles(theme_c);
    let n_titles = titles.len();
    Self {
      theme,
      active_index : 0,
      titles,
      active_menu_item : UIMenuItem::Back,
    }
  }
}

//============================================

#[derive(Debug, Clone)]
pub struct PaddleMenu<'a>  {
  pub theme            : ColorTheme,
  pub active_menu_item : UIMenuItem,
  pub active_index     : usize, 
  pub titles           : Vec<Line<'a>>,
}

impl UIMenu<'_> for PaddleMenu<'_> {
  
  fn get_items() -> Vec<UIMenuItem> {
    let items = vec![UIMenuItem::Back,
                     UIMenuItem::Signal,
                     UIMenuItem::RecoVars,
                     UIMenuItem::Quit];
    items
  }

  fn get_theme(&self) -> ColorTheme {
    self.theme
  }

  fn set_active_idx(&mut self, idx : usize) {
    self.active_index = idx;
  }

  fn get_active_idx(&self) -> usize {
    self.active_index
  }
  
  fn set_active_menu_item(&mut self, item : UIMenuItem) {
    self.active_menu_item = item;
  }
  
  fn get_active_menu_item(&self) -> UIMenuItem {
    self.active_menu_item
  }
}

impl PaddleMenu<'_> {

  pub fn new(theme : ColorTheme) -> Self {
    let theme_c = theme.clone();
    let titles  = Self::get_titles(theme_c);
    let n_titles = titles.len();
    Self {
      theme,
      active_index : 0,
      titles,
      active_menu_item : UIMenuItem::Back,
    }
  }
}

//============================================

#[derive(Debug, Clone)]
pub struct HBMenu<'a>  {
  pub theme            : ColorTheme,
  pub active_menu_item : UIMenuItem,
  pub active_index     : usize, 
  pub titles           : Vec<Line<'a>>,
}

impl UIMenu<'_> for HBMenu<'_> {
  
  fn get_items() -> Vec<UIMenuItem> {
    let items = vec![UIMenuItem::Back,
                     UIMenuItem::EventBuilderHB,
                     UIMenuItem::TriggerHB,
                     UIMenuItem::DataSenderHB];
    items
  }

  fn get_theme(&self) -> ColorTheme {
    self.theme
  }

  fn set_active_idx(&mut self, idx : usize) {
    self.active_index = idx;
  }

  fn get_active_idx(&self) -> usize {
    self.active_index
  }
  
  fn set_active_menu_item(&mut self, item : UIMenuItem) {
    self.active_menu_item = item;
  }
  
  fn get_active_menu_item(&self) -> UIMenuItem {
    self.active_menu_item
  }
}

impl HBMenu<'_> {

  pub fn new(theme : ColorTheme) -> Self {
    let theme_c = theme.clone();
    let titles  = Self::get_titles(theme_c);
    let n_titles = titles.len();
    Self {
      theme,
      active_index : 0,
      titles,
      active_menu_item : UIMenuItem::Back,
    }
  }
}

//============================================

#[derive(Debug, Clone)]
pub struct MoniMenu<'a>  {
  pub theme            : ColorTheme,
  pub active_menu_item : UIMenuItem,
  pub active_index     : usize, 
  pub titles           : Vec<Line<'a>>,
}

impl UIMenu<'_> for MoniMenu<'_> {
  
  fn get_items() -> Vec<UIMenuItem> {
    let items = vec![UIMenuItem::Back,
                     UIMenuItem::PreampBias,
                     UIMenuItem::PreampTemp,
                     UIMenuItem::LTBThresholds,
                     UIMenuItem::Quit];
    items
  }

  fn get_theme(&self) -> ColorTheme {
    self.theme
  }

  fn set_active_idx(&mut self, idx : usize) {
    self.active_index = idx;
  }

  fn get_active_idx(&self) -> usize {
    self.active_index
  }
  
  fn set_active_menu_item(&mut self, item : UIMenuItem) {
    self.active_menu_item = item;
  }
  
  fn get_active_menu_item(&self) -> UIMenuItem {
    self.active_menu_item
  }
}

impl  MoniMenu<'_> {

  pub fn new(theme : ColorTheme) -> Self {
    let theme_c = theme.clone();
    let titles  = Self::get_titles(theme_c);
    let n_titles = titles.len();
    Self {
      theme,
      active_index : 0,
      titles,
      active_menu_item : UIMenuItem::Back,
    }
  }
}

//============================================

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
    let menu_titles  = vec!["Home", "TofEvents", "TofSummary", "RBWaveform", "TofHits",  "ReadoutBoards", "MasterTrigger", "CPUMoniData", "Telemetry" , "Settings", "Quit" ];
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
                 } else if t == &"Telemetry" {
                   // none of these handpicked strings has 0 len
                   let (a, b) = t.split_at(1);
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

  pub fn new(theme : ColorTheme) -> Self  {
    Self {
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

/// Telemetry menu
#[derive(Debug, Copy, Clone)]
pub enum TEMenuItem {
  Home,
  Quit
}

impl From<TEMenuItem> for usize {
  fn from(input: TEMenuItem) -> usize {
    match input {
      TEMenuItem::Home          => 0,
      TEMenuItem::Quit          => 1,
    }   
  }
}

#[derive(Debug, Clone)]
pub struct TEMenu {
  pub theme : ColorTheme,
  pub active_menu_item : TEMenuItem,
}

impl TEMenu {

  pub fn new(theme : ColorTheme) -> Self  {
    Self {
      theme,
      active_menu_item : TEMenuItem::Home,
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PAMoniMenuItem {
  Back,
  Temperatures,
  Biases,
  Quit,
}

impl From<PAMoniMenuItem> for usize {
  fn from(input: PAMoniMenuItem) -> usize {
    match input {
      PAMoniMenuItem::Back          => 0,
      PAMoniMenuItem::Temperatures  => 1,
      PAMoniMenuItem::Biases        => 2,
      PAMoniMenuItem::Quit          => 3
    }
  }
}

#[derive(Debug, Clone)]
pub struct PAMoniMenu {
  pub theme : ColorTheme,
  pub active_menu_item : PAMoniMenuItem,
}

impl PAMoniMenu {
  pub fn new(theme : ColorTheme) -> Self {
    Self {
      theme,
      active_menu_item : PAMoniMenuItem::Temperatures,
    }
  }
  
  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    let menu_titles = vec!["Back", "Temperatures", "Biases", "Quit"];
    let menu : Vec<Line> = menu_titles
               .iter()
               .map(|t| {
                 if t == &"Biases" {
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
    let menu_titles = vec!["Home", "Info", "Waveforms", "RBMoniData", "PAMoniData", "PBMoniData", "LTBMoniData", "SelectBoards [LTB & RB]", "Quit" ];
    let menu : Vec<Line> = menu_titles
               .iter()
               .map(|t| {
                 if t == &"PBMoniData" || t == &"Hits" {
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


