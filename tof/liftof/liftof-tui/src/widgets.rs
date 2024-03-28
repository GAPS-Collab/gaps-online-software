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

//extern crate ndhistogram;
use ndhistogram::{
    ndhistogram,
    Histogram,
    Hist1D,
};
use ndhistogram::axis::{
    Uniform,
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

/// Create the labels for a certain histogram
/// for rendering
pub fn create_labels(histo : &Hist1D<Uniform<f32>>) -> Vec<String> {
  let mut labels = Vec::<String>::new();
  for bin in histo.iter() {
    labels.push(format!("{}",bin.bin.start().unwrap_or(0.0) as u64));
  }
  labels
}

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

// FIXME - merge this with clean data
pub fn prep_data<'a>(histo      : &'a Hist1D<Uniform<f32>>, 
                     labels     : &'a Vec<String>,
                     spacing    : usize) -> Vec<(&'a str,u64)> {
  let mut data = Vec::<(&str, u64)>::new();
  let mut cnt = 0usize;
  for bin in histo.iter() {
    if cnt % spacing != 0 {
      data.push(("", *bin.value as u64));
    } else {
      data.push((&labels[cnt], *bin.value as u64));
    }
    cnt += 1;
  }
  data
}

pub fn histogram(nbin : usize, bin_low : f32,
                 bin_high : f32,
                 data : Vec<f32>) -> Hist1D<Uniform<f32>> {
  let bins = Uniform::new(nbin, bin_low, bin_high);
  let mut histo = ndhistogram!(bins);
  for k in data {
    histo.fill(&k);
  }
  histo
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
