use std::collections::{
    VecDeque,
    HashMap,
};

use crossbeam_channel::{
    Receiver,
};

use ndhistogram::{
    ndhistogram,
    Histogram,
    Hist1D,
};

use ndhistogram::axis::{
    Uniform,
};

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::{
    Block,
    BorderType,
    Borders,
    Paragraph,
    //BarChart,
    //List,
    //ListItem,
    //ListState,
};

use tof_dataclasses::packets::{
    TofPacket,
};
use tof_dataclasses::events::{
    RBWaveform,
};

use tof_dataclasses::errors::SerializationError;
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::database::ReadoutBoard;

use crate::colors::ColorTheme;
use crate::widgets::{
    //clean_data,
    prep_data,
    timeseries,
    histogram,
    create_labels,
};

#[derive(Debug, Clone)]
pub struct RBWaveformTab {
  pub theme          : ColorTheme,
  pub rbw_recv       : Receiver<TofPacket>,
  pub rbw_queue      : VecDeque<RBWaveform>,
  pub queue_size     : usize,
  pub rb_histo       : Hist1D<Uniform<f32>>,
  cali_loaded        : bool,
  pub exclude_ch9    : bool,
  pub rbs            : HashMap<u8, ReadoutBoard>,

  voltages           : Vec<f32>,
  nanoseconds        : Vec<f32>,
}

impl RBWaveformTab {

  // FIXME - eventually share rbs with the other tabs,
  // so we want to have a pointer or a static reference
  pub fn new(rbw_recv : Receiver<TofPacket>,
             rbs      : HashMap<u8, ReadoutBoard>,
             theme    : ColorTheme) -> Self {
    let bins_rb  = Uniform::new(50, 1.0, 51.0).unwrap();
    // FIXME check if there are calibrations
    let mut cali_loaded = false;
    if rbs.len() > 0 {
      cali_loaded = true;
    }
    Self {
      theme   ,
      rbw_recv,
      rbw_queue   : VecDeque::<RBWaveform>::new(),
      queue_size  : 100, // no need for long queue, since we are displaying 
                        // only the last anyway
      rb_histo    : ndhistogram!(bins_rb),
      cali_loaded,
      exclude_ch9 : true,
      rbs         : rbs,

      voltages    : vec![0.0;1024],
      nanoseconds : vec![0.0;1024]
    }
  }
  
  pub fn receive_packet(&mut self) -> Result<(), SerializationError> {  
    // FIXME - can/should this block or not?
    match self.rbw_recv.try_recv() {
      Err(_err) => {
        return Ok(());
      },
      Ok(tp)    => {
        match RBWaveform::from_bytestream(&tp.payload, &mut 0) {
          Ok(wf) => {
            self.rb_histo.fill(&(wf.rb_id as f32));
            self.rbw_queue.push_back(wf);
            if self.rbw_queue.len() > self.queue_size {
              self.rbw_queue.pop_front();
            }
          }
          Err(_err) => {
            return Ok(())
          }
        }
      }
    } // end match
    Ok(())
  }
  
  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
          [Constraint::Percentage(50),
           Constraint::Percentage(50)].as_ref(),
      )
      .split(*main_window);
    let info = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
          [Constraint::Percentage(30),
           Constraint::Percentage(70)].as_ref(),
      )
      .split(chunks[1]);
    
    let mut wf_string = String::from("No RBWaveform");
    //let mut wf        = RBWaveform::new();
    let mut wf        : RBWaveform;
    loop {
      match self.rbw_queue.pop_front() {
        Some(_wf) => {
          wf = _wf;
          if self.exclude_ch9 {
            if wf.rb_channel != 8 {
              break
            }
          }
          wf_string = format!("{}", wf);
        },
        None => {
          return;
        }
      }
    }
    let label       = format!("RBWaveform RB {}-{}", wf.rb_id, wf.rb_channel);
    let wf_theme    = self.theme.clone();
    let mut wf_data = VecDeque::<(f64, f64)>::new();    
    if self.cali_loaded {
      if wf.rb_channel != 0 {
        self.rbs[&wf.rb_id].calibration.voltages(wf.rb_channel as usize,
                                                 wf.stop_cell as usize,
                                                 &wf.adc,
                                                 &mut self.voltages); 
        self.rbs[&wf.rb_id].calibration.nanoseconds(wf.rb_channel as usize,
                                                    wf.stop_cell as usize,
                                                    &mut self.nanoseconds);
        for k in 0..self.nanoseconds.len() {
          wf_data.push_back((self.nanoseconds[k] as f64, self.voltages[k] as f64));
        }
      }
    } else {
      for (i,k) in wf.adc.iter().enumerate() {
        wf_data.push_back((i as f64, *k as f64));
      }
    }
    let wf_chart = timeseries(&mut wf_data,
                              label.clone(),
                              label.clone(),
                              &wf_theme  );
    frame.render_widget(wf_chart, chunks[0]);

    let wf_info = Paragraph::new(wf_string)
      .style(self.theme.style())
      .alignment(Alignment::Left)
      .block(
      Block::default()
        .borders(Borders::ALL)
        .style(self.theme.style())
        .title("RBWaveform")
        .border_type(BorderType::Rounded),
        );
    frame.render_widget(wf_info, info[0]);
    
    let rbhist_labels  = create_labels(&self.rb_histo);
    let rbhist_data    = prep_data(&self.rb_histo, &rbhist_labels, 5, true); 
    let rbhist_chart   = histogram(rbhist_data, String::from("ReadoutBoards"), 3, 0, &self.theme);
    //let rbhist_chart   = BarChart::default()
    //  .block(Block::default().title("ReadoutBoards").borders(Borders::ALL))
    //  .data(rbhist_data.as_slice())
    //  .bar_width(3)
    //  .bar_gap(0)
    //  //.bar_style(Style::default().fg(Color::Blue))
    //  .bar_style(self.theme.highlight_fg())
    //  .value_style(
    //    self.theme.highlight_fg()
    //    //Style::default()
    //    //.bg(Color::Blue)
    //    .add_modifier(Modifier::BOLD),
    //  )
    //  .style(self.theme.background());
    frame.render_widget(rbhist_chart, info[1]);
     
  }
}

