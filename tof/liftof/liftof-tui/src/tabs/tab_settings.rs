use ratatui::prelude::*;

//use ratatui::symbols;
//use ratatui::text::Span;
//use ratatui::terminal::Frame;
//use ratatui::layout::Rect;
//use ratatui::style::{
//    Color,
//    Style,
//};
use ratatui::widgets::{
//    Axis,
    Block,
    BorderType,
//    GraphType,
//    Dataset,
//    Chart,
    Borders,
//    Cell,
    List,
    ListState,
    ListItem,
    //ListDirection,
//    ListState,
    Paragraph,
//    Row,
//    Table,
//    Tabs,
};

use tui_popup::Popup;

use crate::colors::{
    ColorTheme,
    ColorSet,
    COLORSETOMILU,
    COLORSETBW,
    COLORSETDUNE,
    COLORSETLD,
    COLORSETNIUHI,
    COLORSETMATRIX,
    COLORSETSTARFIELD,
    COLORSETGAPS,
    COLORSETPRINCESSPEACH
};


#[derive(Debug, Clone)]
pub struct SettingsTab<'a> {
  pub theme      : ColorTheme,
  pub colortheme_popup : bool,
  pub ctl_state  : ListState,
  pub ctl_items  : Vec::<ListItem<'a>>,
  pub ctl_active : bool,
}

impl SettingsTab<'_> {
  pub fn new(theme : ColorTheme) -> SettingsTab<'static> {
    let ct_list_items = vec![ListItem::new(Line::from("Black&White")),
                             ListItem::new(Line::from("StarField")),
                             ListItem::new(Line::from("Omiliu")),
                             ListItem::new(Line::from("GAPS")),
                             ListItem::new(Line::from("Niuhi")),
                             ListItem::new(Line::from("Dune")),
                             ListItem::new(Line::from("LowerDecks")),
                             ListItem::new(Line::from("Matrix")),
                             ListItem::new(Line::from("PrincessPeach")),
    ];
    SettingsTab {
      theme,
      colortheme_popup : false,
      ctl_state  : ListState::default(),
      ctl_items  : ct_list_items,
      ctl_active : false,
    }
  }

  pub fn next_ct(&mut self) {
    let i = match self.ctl_state.selected() {
      Some(i) => {
        if i >= self.ctl_items.len() - 1 {
          0
        } else {
          i + 1
        }
      }
      None => 0,
    };
    self.ctl_state.select(Some(i));
  }

  pub fn previous_ct(&mut self) {
    let i = match self.ctl_state.selected() {
      Some(i) => {
        if i == 0 {
          self.ctl_items.len() - 1
        } else {
          i - 1
        }
      }
      None => 0,
    };
    self.ctl_state.select(Some(i));
  }

  pub fn unselect_ctl(&mut self) {
    self.ctl_state.select(None);
  }

  pub fn get_colorset(&self) -> Option<ColorSet> {
    let cs = match self.ctl_state.selected() {
      Some(i) => {
        match i {
          0 => Some(COLORSETBW),
          1 => Some(COLORSETSTARFIELD),
          2 => Some(COLORSETOMILU),
          3 => Some(COLORSETGAPS),
          4 => Some(COLORSETNIUHI),
          5 => Some(COLORSETDUNE),
          6 => Some(COLORSETLD),
          7 => Some(COLORSETMATRIX),
          8 => Some(COLORSETPRINCESSPEACH),
          _ => None,
        }
      }
      None => None,
    };
    cs
  }

  // Color::Blue was nice for background
  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    let main_rows = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
          [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
      )
      .split(*main_window);

    let main_cols0 = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [Constraint::Percentage(50), 
             Constraint::Percentage(50)
            ].as_ref()
        )
        .split(main_rows[0]);

    let sub_rows0 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [Constraint::Percentage(50),
             Constraint::Percentage(50)
            ].as_ref()
        )
        .split(main_cols0[1]);

    let main_cols1 = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [Constraint::Percentage(50), 
             Constraint::Percentage(50)
            ].as_ref()
        )
        .split(main_rows[1]);
    let sub_rows1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [Constraint::Percentage(50),
             Constraint::Percentage(50)
            ].as_ref()
        )
        .split(main_cols1[1]);
      let par_title_string = String::from("Apply Color Theme");
      let (first, rest) = par_title_string.split_at(1);
      let par_title = Line::from(vec![
        Span::styled(
            first,
            Style::default()
                .fg(self.theme.hc)
                .add_modifier(Modifier::UNDERLINED)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(rest, self.theme.style()),
      ]);

      let rbs = Block::default()
        .borders(Borders::ALL)
        .style(self.theme.style())
        .title(par_title)
        .border_type(BorderType::Plain);

     let color_theme_list = List::new(self.ctl_items.clone()).block(rbs)
        .highlight_style(self.theme.highlight().add_modifier(Modifier::BOLD))
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true);
      match self.ctl_state.selected() {
        None => self.ctl_state.select(Some(1)),
        Some(_) => (),
      }
    let content = "Settings (WIP)"; 
    let main_view = Paragraph::new(content)
    .style(self.theme.style())
    .alignment(Alignment::Center)
    .block(
      Block::default()
        .borders(Borders::NONE)
    );

    let par_refresh_string = String::from("Refresh rate");
    let (first, rest) = par_refresh_string.split_at(1);
    let par_refresh_title = Line::from(vec![
      Span::styled(
          first,
          Style::default()
              .fg(self.theme.hc)
              .add_modifier(Modifier::UNDERLINED)
              .add_modifier(Modifier::BOLD),
      ),
      Span::styled(rest, self.theme.style()),
    ]);

    let refresh_view = Paragraph::new(content)
    .style(self.theme.style())
    .alignment(Alignment::Center)
    .block(
      Block::default()
        .title(par_refresh_title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded),
    );
  
    let par_wf_fixed_y_title = String::from("Fix Waveform y-scale");
    let wf_fixed_y_view = Paragraph::new(content)
    .style(self.theme.style())
    .alignment(Alignment::Center)
    .block(
      Block::default()
        .title(par_wf_fixed_y_title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded),
    );

    let stream : String = String::from("");
    let side_view = Paragraph::new(stream)
    .style(self.theme.style())
    .alignment(Alignment::Left)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        //.title("Stream")
    );
    frame.render_widget(main_view, main_cols1[1]);
    frame.render_widget(side_view, main_cols0[0]);
    frame.render_widget(refresh_view, sub_rows1[0]);
    frame.render_stateful_widget(color_theme_list, sub_rows0[0], &mut self.ctl_state );
    frame.render_widget(wf_fixed_y_view, sub_rows1[1]);
    if self.colortheme_popup {
      let popup = Popup::new("Any key to continue!")
        .title("New color theme selected!")
        .style(self.theme.style());
      frame.render_widget(&popup, frame.area());
    }
  }
}


