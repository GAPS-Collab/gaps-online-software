use std::collections::VecDeque;

use ratatui::symbols;
use ratatui::symbols::line::*;
use ratatui::text::Span;
use ratatui::style::{
    Modifier,
    Color,
    Style,
};
use ratatui::widgets::{
    Axis,
    Block,
    BorderType,
    BarChart,
    GraphType,
    Dataset,
    Chart,
    LineGauge,
    Borders,
};

//extern crate ndhistogram;
use ndhistogram::{
    //ndhistogram,
    Histogram,
    Hist1D,
};
use ndhistogram::axis::{
    Uniform,
};

pub const LG_LINE_HORIZONTAL : &str = "░";
pub const LG_LINE: Set = Set {
        vertical         : THICK_VERTICAL,
        //horizontal       : THICK_HORIZONTAL,
        horizontal       : LG_LINE_HORIZONTAL,
        top_right        : THICK_TOP_RIGHT,
        top_left         : THICK_TOP_LEFT,
        bottom_right     : THICK_BOTTOM_RIGHT,
        bottom_left      : THICK_BOTTOM_LEFT,
        vertical_left    : THICK_VERTICAL_LEFT,
        vertical_right   : THICK_VERTICAL_RIGHT,
        horizontal_down  : THICK_HORIZONTAL_DOWN,
        horizontal_up    : THICK_HORIZONTAL_UP,
        cross            : THICK_CROSS,
};

//extern crate num_traits;
//use num_traits::Num;

use crate::colors::{
  ColorTheme,
};


//#[derive(Debug, Clone)]
//struct HistoDisplay {
//  pub nbin     : usize,
//  pub bin_low  : f32,
//  pub bin_high : f32,
//  pub histo    : Hist1D<Uniform<f32>>,
//}
//
//impl HistoDisplay {
//  pub fn new(nbin : usize, bin_low : f32, bin_high : f32) -> Self {
//    let bins = Uniform::new(nbin, bin_low, bin_high);  
//    Self {
//      nbin     : nbin,
//      bin_low  : bin_low,
//      bin_high : bin_high,
//      histo    : ndhistogram!(bins), 
//    }
//  }
//}


/// Adapt the bins of the histogram for the 
/// bar chart which will get rendered.
/// Always show a minimum number of bins, 
/// but if the max y-bin is "too far to the left"
/// then shorten the range for a better visualization
///
/// # Arguments
///
/// * labels       : bin labels for rendering
/// * clean_from   : leave bins below this 
///                  untouched
pub fn clean_data<'a>(histo      : &'a Hist1D<Uniform<f32>>, 
                      labels     : &'a Vec<String>, 
                      clean_from : usize) -> Vec<(&'a str,u64)> {
  let mut max_pop_bin = 0;
  let mut vec_index   = 0;
  let mut bins = Vec::<(u64, u64)>::new();
  for bin in histo.iter() {
    let bin_value = *bin.value as u64;
    bins.push((bin.index as u64, bin_value));
    // always show the first x bins, but if 
    // the bins with index > clean_from are not 
    // populated, discard them
    if bin_value > 0 && bin.index > clean_from {
      max_pop_bin = vec_index;
    }
    vec_index += 1;
  }
  bins.retain(|&(x,_)| x <= max_pop_bin);
  let mut clean_data = Vec::<(&str, u64)>::new();
  for n in bins.iter() {
    clean_data.push((&labels[n.0 as usize], n.1));
  }
  clean_data
}

/// Create the labels for a certain histogram
/// for rendering
pub fn create_labels(histo : &Hist1D<Uniform<f32>>) -> Vec<String> {
  let mut labels = Vec::<String>::new();
  for bin in histo.iter() {
    match bin.bin.start() {
      None => {
        labels.push(String::from("x"));
      },
      Some(value) => {
        labels.push(format!("{}", value as u64));
      }
    }
  }
  labels
}

// FIXME - merge this with clean data
/// Prepare data for histogram widget
///
/// # Arguments:
///
/// * remove_uf   : Remove underflow bin
pub fn prep_data<'a>(histo      : &'a Hist1D<Uniform<f32>>, 
                     labels     : &'a Vec<String>,
                     spacing    : usize,
                     remove_uf  : bool) -> Vec<(&'a str,u64)> {
  let mut data = Vec::<(&str, u64)>::new();
  for (k,bin) in histo.iter().enumerate() {
    if k == 0 && remove_uf {
      continue;
    }
    if k == 1 && remove_uf {
      data.push((&labels[k], *bin.value as u64));
      continue;
    }
    // k+1 to account for underflow bin
    if k % spacing != 0 {
      data.push(("-", *bin.value as u64));
    } else {
      data.push((&labels[k], *bin.value as u64));
    }
  }
  data
}

pub fn histogram<'a>(hist_data : Vec<(&'a str, u64)>,
                     title     : String,
                     bar_width : u16,
                     bar_gap   : u16,
                     theme     : &ColorTheme) -> BarChart<'a> {
  //let bins = Uniform::new(nbin, bin_low, bin_high);
  //let mut histo = ndhistogram!(bins);
  //for k in data {
  //  histo.fill(&k);
  //}
  let chart  = BarChart::default()
    .block(Block::default().title(title).borders(Borders::ALL))
    .data(hist_data.as_slice())
    .bar_width(bar_width)
    .bar_gap(bar_gap)
    //.bar_style(Style::default().fg(Color::Blue))
    .bar_style(theme.style())
    .value_style(
      theme.style()
      //Style::default()
      //.bg(Color::Blue)
      .add_modifier(Modifier::BOLD),
    )
    .style(theme.background());
  chart
}

pub fn timeseries<'a>(data        : &'a mut VecDeque<(f64,f64)>,
                      ds_name     : String,
                      xlabel      : String,
                      theme       : &'a ColorTheme) -> Chart<'a> {
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
  let mut t_labels = Vec::<Span>::new();
  for k in 0..10 {
    let _label = format!("{}", (t_min + t_spacing * k as u64));
    t_labels.push(Span::from(_label));
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
  let mut y_labels = Vec::<Span>::new() ;
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
        y_labels.push(Span::from(_label));
      },
      1 => {
        let _label = format!("{:.1}", (y_min + y_spacing * k as f64));
        y_labels.push(Span::from(_label));
      },
      2 => {
        let _label = format!("{:.2}", (y_min + y_spacing * k as f64));
        y_labels.push(Span::from(_label));
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
        .border_type(BorderType::Rounded),
    )
    .x_axis(Axis::default()
      .title(Span::styled("MET [s]", Style::default().fg(Color::White)))
      .style(theme.style())
      .bounds([t_min as f64, t_max as f64])
      //.labels(t_labels.clone().iter().cloned().map(Span::from).collect()))
      .labels(t_labels.clone())
    )
    .y_axis(Axis::default()
      //.title(Span::styled("T [\u{00B0}C]", Style::default().fg(Color::White)))
      .style(theme.style())
      .bounds([y_min as f64, y_max as f64])
      //.labels(y_labels.clone().iter().cloned().map(Span::from).collect()))
      .labels(y_labels.clone())
    )
    .style(theme.style());
    chart
}


/// A simple line gauge, that is bacically a progress bar
pub fn gauge(title : String,
             label : String,
             ratio : f64,
             theme : &ColorTheme) -> LineGauge {
    let gauge = LineGauge::default()
      .block(
        Block::default()
        .borders(Borders::ALL)
        .style(theme.style())
        .title(title)
        .border_type(BorderType::Rounded)
      )
      .filled_style(
        Style::default()
          .fg(theme.hc)
          .bg(theme.bg1)
          .add_modifier(Modifier::BOLD)
      )
      //.use_unicode(true)
      .label(label)
      //.line_set(symbols::line::THICK)  // THICK
      .line_set(LG_LINE)
      //.percent(self.disk_usage as u16);
      .ratio(ratio);
    gauge
}

