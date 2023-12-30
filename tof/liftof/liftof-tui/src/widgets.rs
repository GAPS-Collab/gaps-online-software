use std::collections::VecDeque;

use ratatui::symbols;
use ratatui::text::Span;
use ratatui::terminal::Frame;
use ratatui::layout::Rect;
use ratatui::style::{
    Color,
    Style,
};
use ratatui::widgets::{
    Axis,
    Block,
    BorderType,
    GraphType,
    Dataset,
    Chart,
    Borders,
    Cell,
    List,
    ListItem,
    ListState,
    Paragraph,
    Row,
    Table,
    Tabs,
};


use crate::colors::ColorTheme2;

// FIXME - make some smart decisions 
// about palette
pub fn timeseries<'a>(data        : &'a mut VecDeque<(f64,f64)>,
                      t_min       : u64,
                      t_max       : u64,
                      t_labels    : &'a Vec<String>,
                      ds_name     : String,
                      xlabel      : String,
                      theme       : &'a ColorTheme2) -> Chart<'a> {
  let y_only : Vec::<i64> = data.iter().map(|z| z.1.round() as i64).collect();
  let y_max = *y_only.iter().max().unwrap_or(&0) + 5;
  let y_min = *y_only.iter().min().unwrap_or(&0) - 5;
  let y_spacing = (y_max - y_min)/5;
  let y_labels = vec![y_min.to_string(),
                       (y_min + y_spacing).to_string(),
                       (y_min + 2*y_spacing).to_string(),
                       (y_min + 3*y_spacing).to_string(),
                       (y_min + 4*y_spacing).to_string(),
                       (y_min + 5*y_spacing).to_string()];
  let dataset = vec![Dataset::default()
      .name(ds_name)
      .marker(symbols::Marker::Braille)
      .graph_type(GraphType::Line)
      .style(theme.style())
      .data(data.make_contiguous())];
  let chart = Chart::new(dataset)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .style(theme.style())
        .title(xlabel )
        .border_type(BorderType::Double),
    )
    .x_axis(Axis::default()
      .title(Span::styled("MET [s]", Style::default().fg(Color::White)))
      .style(theme.style())
      .bounds([t_min as f64, t_max as f64])
      .labels(t_labels.clone().iter().cloned().map(Span::from).collect()))
    .y_axis(Axis::default()
      //.title(Span::styled("T [\u{00B0}C]", Style::default().fg(Color::White)))
      .style(theme.style())
      .bounds([y_min as f64, y_max as f64])
      .labels(y_labels.clone().iter().cloned().map(Span::from).collect()))
    .style(theme.style());
    chart
}
