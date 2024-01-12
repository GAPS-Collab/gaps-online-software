use std::collections::VecDeque;

use ratatui::symbols;
use ratatui::text::Span;
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
};


use crate::colors::ColorTheme2;

pub fn timeseries<'a>(data        : &'a mut VecDeque<(f64,f64)>,
                      ds_name     : String,
                      xlabel      : String,
                      theme       : &'a ColorTheme2) -> Chart<'a> {
  let x_only : Vec::<f64> = data.iter().map(|z| z.0).collect();
  // get timing axis
  let t_min : u64;
  let mut t_max : u64;
  if x_only.len() == 0 {
    t_min = 0;
    t_max = 0;
  } else {   
    t_min = x_only[0] as u64;
    t_max = x_only[x_only.len() -1] as u64;
  }
  t_max += (0.05*t_max as f64).round() as u64;
  let t_spacing = (t_max - t_min)/10;
  let mut t_labels = Vec::<String>::new();
  for k in 0..10 {
    let _label = format!("{}", (t_min + t_spacing * k as u64));
    t_labels.push(_label);
  }

  let y_only : Vec::<f64> = data.iter().map(|z| z.1).collect();
  let mut y_min = f64::MAX;
  let mut y_max = f64::MIN;
  if y_only.len() == 0 {
    y_max = 0.0;
    y_min = 0.0;
  }
  for y in y_only {
    if y < y_min {
      y_min = y;
    }
    if y > y_max {
      y_max = y;
    }
  }
  y_max += f64::abs(y_max)*0.05;
  y_min -= f64::abs(y_min)*0.05;
  let y_spacing = f64::abs(y_max - y_min)/5.0;
  let mut y_labels = Vec::<String>::new() ;
  let mut precision = 0u8;
  if f64::abs(y_max - y_min) <= 10.0 {
    precision = 1;
  }
  if f64::abs(y_max - y_min) <= 1.0 {
    precision = 2;
  }
  for k in 0..5 {
    match precision {
      0 => {
        let _label = format!("{}", (y_min + y_spacing * k as f64).round() as i64);
        y_labels.push(_label);
      },
      1 => {
        let _label = format!("{:.1}", (y_min + y_spacing * k as f64));
        y_labels.push(_label);
      },
      2 => {
        let _label = format!("{:.2}", (y_min + y_spacing * k as f64));
        y_labels.push(_label);
      },
      _ => (),
    }
  }

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
