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

use crate::colors::{
    ColorTheme,
    ColorTheme2,
    ColorSet,
    COLORSETOMILU,
    COLORSETBW,
    COLORSETDUNE,
    COLORSETGMAPS,
    COLORSETLD,
    COLORSETNIUHI,
    COLORSETMATRIX,
};


#[derive(Debug, Clone)]
pub struct SettingsTab<'a> {
  pub theme      : ColorTheme2,
  pub ctl_state  : ListState,
  pub ctl_items  : Vec::<ListItem<'a>>,
  pub ctl_active : bool,
}

impl SettingsTab<'_> {
  pub fn new(theme : ColorTheme2) -> SettingsTab<'static> {
    let ct_list_items = vec![ListItem::new(Line::from("Black&White")),
                             ListItem::new(Line::from("Omiliu")),
                             ListItem::new(Line::from("Dune")),
                             ListItem::new(Line::from("GMaps")),
                             ListItem::new(Line::from("LowerDecks")),
                             ListItem::new(Line::from("Niuhi")),
                             ListItem::new(Line::from("Matrix")),
                             ];
    SettingsTab {
      theme,
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
          1 => Some(COLORSETOMILU),
          2 => Some(COLORSETDUNE),
          3 => Some(COLORSETGMAPS),
          4 => Some(COLORSETLD),
          5 => Some(COLORSETNIUHI),
          6 => Some(COLORSETMATRIX),
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

    let main_cols1 = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [Constraint::Percentage(50), 
             Constraint::Percentage(50)
            ].as_ref()
        )
        .split(main_rows[1]);
//      let items: Vec<_> = rb_list
//      .iter()
//      .map(|rb| {
//        ListItem::new(Line::from(vec![Span::styled(
//          "RB ".to_owned() + &rb.rb_id.to_string(),
//          Style::default(),
//        )]))
//      })
//      .collect();
      let par_title_string = String::from("Apply Color Theme");
      let (first, rest) = par_title_string.split_at(1);
      let par_title = Line::from(vec![
        Span::styled(
            first,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::UNDERLINED),
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
        //.direction(ListDirection::BottomToTop);  
//
//    let rb_detail =  Paragraph::new(selected_rb.to_string())
//     .style(Style::default().fg(Color::LightCyan))
//     .alignment(Alignment::Left)
//     //.scroll((5, 10))
//     //.text(rb_list[0].to_string())
//     .block(
//       Block::default()
//         .borders(Borders::ALL)
//         .style(Style::default().fg(Color::White))
//         .title("Detail")
//         .border_type(BorderType::Double),
//    );


    let content = "Settings (WIP)"; 
    let main_view = Paragraph::new(content)
    .style(self.theme.style())
    .alignment(Alignment::Center)
    .block(
      Block::default()
        .borders(Borders::NONE)
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
    frame.render_stateful_widget(color_theme_list, main_cols0[1], &mut self.ctl_state );
  }
}


