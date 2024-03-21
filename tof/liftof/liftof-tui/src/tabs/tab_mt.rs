//! Master Trigger tab
//! 
//! Show current data from the master trigger

use std::collections::VecDeque;

use std::time::{
    Instant
};

use ratatui::{
    symbols,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    terminal::Frame,
    widgets::{
        Block,
        Dataset,
        Axis,
        GraphType,
        BorderType,
        Chart,
        BarChart,
        Borders,
        Paragraph
    },
};

extern crate crossbeam_channel;
use crossbeam_channel::Receiver;

extern crate ndhistogram;
use ndhistogram::{
    ndhistogram,
    Histogram,
    Hist1D,
};
use ndhistogram::axis::{
    Uniform,
};

use tof_dataclasses::packets::{TofPacket, PacketType};
use tof_dataclasses::events::MasterTriggerEvent;
use tof_dataclasses::monitoring::MtbMoniData;
use tof_dataclasses::errors::SerializationError;
use tof_dataclasses::serialization::Serialization;

use crate::colors::{
    ColorTheme2,
};


pub const HIST_LABELS : [&str;100]  = ["0",   "1", "2",  "3",  "4",  "5",  "6",  "7",  "8",  "9",
                                       "10", "11", "12", "13", "14", "15", "16", "17", "18", "19",
                                       "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", 
                                       "30", "31", "32", "33", "34", "35", "36", "37", "38", "39",
                                       "40", "41", "42", "43", "44", "45", "46", "47", "48", "49",
                                       "50", "51", "52", "53", "54", "55", "56", "57", "58", "59",
                                       "60", "61", "62", "63", "64", "65", "66", "67", "68", "69",
                                       "70", "71", "72", "73", "74", "75", "76", "77", "78", "79",
                                       "80", "81", "82", "83", "84", "85", "86", "87", "88", "89",
                                       "90", "91", "92", "93", "94", "95", "96", "97", "98", "99",
                                       ];


#[derive(Debug, Clone)]
pub struct MTTab {
  pub main_layout    : Vec<Rect>,
  pub info_layout    : Vec<Rect>,
  pub detail_layout  : Vec<Rect>,
  pub view_layout    : Vec<Rect>,

  pub event_queue    : VecDeque<MasterTriggerEvent>,
  pub moni_queue     : VecDeque<MtbMoniData>,
  pub met_queue      : VecDeque<f64>,
  pub rate_queue     : VecDeque<(f64,f64)>,
  pub fpgatmp_queue  : VecDeque<(f64,f64)>,
  pub tp_receiver    : Receiver<TofPacket>,
  pub mte_receiver   : Receiver<MasterTriggerEvent>,
  pub queue_size     : usize,
 
  pub n_events       : usize,
  pub n_moni         : usize,
  pub miss_evid      : usize,
  pub last_evid      : u32,
  pub nch_histo      : Hist1D<Uniform<f32>>,
  pub theme          : ColorTheme2,

  timer              : Instant,

}

impl MTTab {

  pub fn new(tp_receiver  : Receiver<TofPacket>,
             mte_receiver : Receiver<MasterTriggerEvent>,
             theme        : ColorTheme2) -> MTTab {
    let bins = Uniform::new(50, -0.5, 49.5);
    Self {
      main_layout    : Vec::<Rect>::new(),
      info_layout    : Vec::<Rect>::new(),
      detail_layout  : Vec::<Rect>::new(),
      view_layout    : Vec::<Rect>::new(),
      event_queue    : VecDeque::<MasterTriggerEvent>::with_capacity(1000),
      moni_queue     : VecDeque::<MtbMoniData>::with_capacity(1000),
      met_queue      : VecDeque::<f64>::with_capacity(1000),
      rate_queue     : VecDeque::<(f64,f64)>::with_capacity(1000),
      fpgatmp_queue  : VecDeque::<(f64,f64)>::with_capacity(1000),
      tp_receiver    : tp_receiver,
      mte_receiver   : mte_receiver,
      queue_size     : 1000, // random
      n_events       : 0,
      n_moni         : 0,
      miss_evid      : 0,
      last_evid      : 0,
      nch_histo      : ndhistogram!(bins),
      theme          : theme,
      timer          : Instant::now(),
    }
  }

  pub fn receive_packet(&mut self) -> Result<(), SerializationError> {
    let mut mte = MasterTriggerEvent::new(0,0);
    let met     = self.timer.elapsed().as_secs_f64();
    match self.tp_receiver.try_recv() {
      Err(_err)   => {
        match self.mte_receiver.try_recv() {
          Err(_err) => (),
          Ok(_ev)   => {
            mte = _ev;
          }
        }
      },
      Ok(pack)    => {
        info!("Got next packet {}!", pack);
        match pack.packet_type {
          PacketType::MasterTrigger => {
            mte = MasterTriggerEvent::from_bytestream(&pack.payload, &mut 0)?;
          },
          PacketType::MonitorMtb   => {
            info!("Got new MtbMoniData!");
            let moni = MtbMoniData::from_bytestream(&pack.payload, &mut 0)?;
            self.n_moni += 1;
            self.moni_queue.push_back(moni);
            if self.moni_queue.len() > self.queue_size {
              self.moni_queue.pop_front();
            }
            self.rate_queue.push_back((met, moni.rate as f64));
            if self.rate_queue.len() > self.queue_size {
              self.rate_queue.pop_front();
            }
            self.fpgatmp_queue.push_back((met, moni.get_fpga_temp() as f64));
            if self.fpgatmp_queue.len() > self.queue_size {
              self.fpgatmp_queue.pop_front();
            }
            self.met_queue.push_back(met);
            if self.met_queue.len() > self.queue_size {
              self.met_queue.pop_front();
            }
            return Ok(());
          },
          _ => (),
        }
      } // end Ok
    } // end match
    if mte.event_id != 0 {
      let hits = mte.get_dsi_j_ch_for_triggered_ltbs();
      self.nch_histo.fill(&(hits.len() as f32));
      self.n_events += 1;
      self.event_queue.push_back(mte);
      if self.event_queue.len() > self.queue_size {
        self.event_queue.pop_front();
      }
      if self.last_evid != 0 {
        if mte.event_id - self.last_evid != 1 {
          self.miss_evid += (mte.event_id - self.last_evid) as usize;
        }
      }
      self.last_evid = mte.event_id;
    }
    Ok(())
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    let main_chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
          [Constraint::Percentage(70), Constraint::Percentage(30)].as_ref(),
      )
      .split(*main_window);
    
    let info_chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
          [Constraint::Percentage(32),
           Constraint::Percentage(32),
           Constraint::Percentage(32),
          ].as_ref(),
      )
      .split(main_chunks[1]);
       
    let detail_chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
          [Constraint::Percentage(60),
           Constraint::Percentage(40),
          ].as_ref(),
      )
      .split(main_chunks[0]);

    let view_chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
          [Constraint::Percentage(50),
           Constraint::Percentage(50),
          ].as_ref(),
      )
      .split(detail_chunks[0]);
    self.main_layout   = main_chunks.to_vec();
    self.info_layout   = info_chunks.to_vec();
    self.detail_layout = detail_chunks.to_vec();
    self.view_layout   = view_chunks.to_vec();

    let t_min = *self.met_queue.front().unwrap_or(&0.0) as u64;
    let t_max = *self.met_queue.back().unwrap_or(&0.0)  as u64;
    let t_spacing = (t_max - t_min)/5;

    let t_labels = vec![t_min.to_string(),
                       (t_min + t_spacing).to_string(),
                       (t_min + 2*t_spacing).to_string(),
                       (t_min + 3*t_spacing).to_string(),
                       (t_min + 4*t_spacing).to_string(),
                       (t_min + 5*t_spacing).to_string()];
    
     let rate_only : Vec::<i64> = self.rate_queue.iter().map(|z| z.1.round() as i64).collect();
     let r_max = *rate_only.iter().max().unwrap_or(&0) + 5;
     let r_min = *rate_only.iter().min().unwrap_or(&0) - 5;
     let rate_spacing = (r_max - r_min)/5;
     let rate_labels = vec![r_min.to_string(),
                            (r_min + rate_spacing).to_string(),
                            (r_min + 2*rate_spacing).to_string(),
                            (r_min + 3*rate_spacing).to_string(),
                            (r_min + 4*rate_spacing).to_string(),
                            (r_min + 5*rate_spacing).to_string()];
     
     let rate_dataset = vec![Dataset::default()
         .name("MTB Rate")
         .marker(symbols::Marker::Braille)
         .graph_type(GraphType::Line)
         //.style(Style::default().fg(pl.get_fg_light()).bg(pl.get_bg_dark()))
         .style(self.theme.style())
         .data(self.rate_queue.make_contiguous())];

    let rate_chart = Chart::new(rate_dataset)
      .block(
        Block::default()
          .borders(Borders::ALL)
          .style(Style::default().patch(self.theme.style()))
          .title("MT rate ".to_owned() )
          .border_type(BorderType::Double),
      )
      .x_axis(Axis::default()
        .title(Span::styled("MET [s]", Style::default().patch(self.theme.style())))
        .style(Style::default().patch(self.theme.style()))
        .bounds([t_min as f64, t_max as f64])
        //.bounds([0.0, 1000.0])
        .labels(t_labels.clone().iter().cloned().map(Span::from).collect()))
      .y_axis(Axis::default()
        .title(Span::styled("Hz", Style::default().patch(self.theme.style())))
        .style(Style::default().patch(self.theme.style()))
        .bounds([r_min as f64, r_max as f64])
        //.bounds([0.0,1000.0])
        .labels(rate_labels.clone().iter().cloned().map(Span::from).collect()))
      .style(self.theme.style()); 
     
    // NChannel distribution
    let mut max_pop_bin = 0;
    let mut vec_index   = 0;
    let mut bins = Vec::<(u64, u64)>::new();
    
    for bin in self.nch_histo.iter() {
      let bin_value = *bin.value as u64;
      bins.push((bin.index as u64, bin_value));
      // always show the first 10 bins, but if 
      // the bins with index > 10 are not 
      // populated, discard them
      if bin_value > 0 && bin.index > 10 {
        max_pop_bin = vec_index;
      }
      vec_index += 1;
    }
    bins.retain(|&(x,_)| x <= max_pop_bin);
    let mut bins_for_bc = Vec::<(&str, u64)>::new();
    debug!("bins: {:?}", bins);
    for n in bins.iter() {
      bins_for_bc.push((HIST_LABELS[n.0 as usize], n.1));
      //bins_for_bc.push((foo, n.1));
      //n_iter += 1;
    }
    let nch_chart = BarChart::default()
      .block(Block::default().title("N Hits (N CH)").borders(Borders::ALL))
      .data(bins_for_bc.as_slice())
      .bar_width(1)
      .bar_gap(1)
      //.bar_style(Style::default().fg(Color::Blue))
      .bar_style(self.theme.highlight_fg())
      .value_style(
        self.theme.highlight_fg()
        //Style::default()
        //.bg(Color::Blue)
        .add_modifier(Modifier::BOLD),
      )
      .style(self.theme.background());


    // FPGA temperature
    let tmp_only : Vec::<i64> = self.fpgatmp_queue.iter().map(|z| z.1.round() as i64).collect();
    let tmp_max = *tmp_only.iter().max().unwrap_or(&0) + 5;
    let tmp_min = *tmp_only.iter().min().unwrap_or(&0) - 5;
    let tmp_spacing = (tmp_max - tmp_min)/5;
    let tmp_labels = vec![tmp_min.to_string(),
                         (tmp_min + tmp_spacing).to_string(),
                         (tmp_min + 2*tmp_spacing).to_string(),
                         (tmp_min + 3*tmp_spacing).to_string(),
                         (tmp_min + 4*tmp_spacing).to_string(),
                         (tmp_min + 5*tmp_spacing).to_string()];
    let fpga_temp_dataset = vec![Dataset::default()
        .name("FPGA T")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().patch(self.theme.style()))
        .data(self.fpgatmp_queue.make_contiguous())];
    let fpga_temp_chart = Chart::new(fpga_temp_dataset)
      .block(
        Block::default()
          .borders(Borders::ALL)
          .style(Style::default().patch(self.theme.style()))
          .title("FPGA T [\u{00B0}C] ".to_owned() )
          .border_type(BorderType::Double),
      )
      .x_axis(Axis::default()
        .title(Span::styled("MET [s]", Style::default().fg(Color::White)))
        .style(Style::default().patch(self.theme.style()))
        .bounds([t_min as f64, t_max as f64])
        .labels(t_labels.clone().iter().cloned().map(Span::from).collect()))
      .y_axis(Axis::default()
        //.title(Span::styled("T [\u{00B0}C]", Style::default().fg(Color::White)))
        .style(self.theme.style())
        .bounds([tmp_min as f64, tmp_max as f64])
        .labels(tmp_labels.clone().iter().cloned().map(Span::from).collect()))
      .style(self.theme.style());
    
    let last_event = self.event_queue.back();
    let view_string : String;
    match last_event {
      Some(event) => { 
        view_string = event.to_string();
      }, 
      None => {
        view_string = String::from("EVT QUEUE EMPTY!");
      }
    }
    let event_view = Paragraph::new(view_string)
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    //.scroll((5, 10))
    .block(
      Block::default()
        .borders(Borders::ALL)
        .style(self.theme.style())
        .title("Last MasterTriggerEvent")
        .border_type(BorderType::Rounded),
    );

    let last_moni = self.moni_queue.back();
    let view_moni : String;
    match last_moni {
      Some(moni) => { 
        view_moni = moni.to_string();
      }, 
      None => {
        view_moni = String::from("MTBMONI QUEUE EMPTY!");
      }
    }
    
    let moni_view = Paragraph::new(view_moni)
    .style(self.theme.style())
    .alignment(Alignment::Left)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .style(self.theme.style())
        .title("Last MtbMoniData")
        .border_type(BorderType::Rounded),
    );
    
    let met = self.met_queue.back().unwrap_or(&0.0);
    let view_summary = format!("Summary Statistics:
  N_Events                         : {}
  N_Moni                           : {}
  Mission Elapsed Time (MET) [sec] : {:.3}
  N EventID Missed                 : {}",
                              self.n_events,
                              self.n_moni,
                              met,
                              self.miss_evid);
    let summary_view = Paragraph::new(view_summary)
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .style(self.theme.style())
        .title("Overview")
        .border_type(BorderType::Rounded),
    );


    // render everything
    frame.render_widget(rate_chart,      self.info_layout[0]); 
    frame.render_widget(nch_chart,       self.info_layout[1]);
    frame.render_widget(fpga_temp_chart, self.info_layout[2]);
    frame.render_widget(event_view,      self.view_layout[0]);
    frame.render_widget(moni_view,       self.view_layout[1]);
    frame.render_widget(summary_view,    self.detail_layout[1]);
  }
}
    
//    let network = Sparkline::default()
//    .block(
//      Block::default()
//        .borders(Borders::ALL)
//        .style(Style::default().fg(Color::White))
//        .title("Network I/O")
//        .border_type(BorderType::Double),
//    ) // or THREE_LEVELS
//    .bar_set(tui::symbols::bar::NINE_LEVELS)
//    .data(&[0, 2, 3, 4, 1, 4, 10])
//    .max(5)
//    .style(Style::default().fg(Color::Blue).bg(Color::Black));

