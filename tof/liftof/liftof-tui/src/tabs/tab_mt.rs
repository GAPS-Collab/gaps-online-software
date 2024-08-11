//! Master Trigger tab
//! 
//! Show current data from the master trigger

use std::collections::{
    VecDeque,
    HashMap
};

use std::time::{
    Instant
};

use ratatui::{
    symbols,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{
        Color,
        //Modifier,
        Style},
    text::Span,
    terminal::Frame,
    widgets::{
        Block,
        Dataset,
        Axis,
        GraphType,
        BorderType,
        Chart,
        //BarChart,
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

use tof_dataclasses::packets::{
    TofPacket,
    PacketType
};
use tof_dataclasses::events::MasterTriggerEvent;
use tof_dataclasses::monitoring::MtbMoniData;
use tof_dataclasses::errors::SerializationError;
//use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::database::DsiJChPidMapping;
use tof_dataclasses::events::master_trigger::LTBThreshold;
use crate::colors::{
    ColorTheme,
};

use crate::widgets::{
    //clean_data,
    prep_data,
    create_labels,
    histogram,
    timeseries
};

#[derive(Debug, Clone)]
pub struct MTTab {
  pub event_queue    : VecDeque<MasterTriggerEvent>,
  pub moni_queue     : VecDeque<MtbMoniData>,
  pub met_queue      : VecDeque<f64>,
  pub rate_queue     : VecDeque<(f64,f64)>,
  pub lost_r_queue   : VecDeque<(f64,f64)>,
  pub fpgatmp_queue  : VecDeque<(f64,f64)>,
  pub tp_receiver    : Receiver<TofPacket>,
  pub mte_receiver   : Receiver<MasterTriggerEvent>,
  pub queue_size     : usize,
 
  pub n_events       : usize,
  pub n_moni         : usize,
  pub miss_evid      : usize,
  pub last_evid      : u32,
  pub nch_histo      : Hist1D<Uniform<f32>>,
  pub mtb_link_histo : Hist1D<Uniform<f32>>,
  pub panel_histo    : Hist1D<Uniform<f32>>,
  pub theme          : ColorTheme,

  pub mapping        : DsiJChPidMapping,
  pub mtlink_rb_map  : HashMap<u8,u8>,
  pub problem_hits   : Vec<(u8, u8, (u8, u8), LTBThreshold)>,
  timer              : Instant,
}

impl MTTab {

  pub fn new(tp_receiver  : Receiver<TofPacket>,
             mte_receiver : Receiver<MasterTriggerEvent>,
             mapping      : DsiJChPidMapping,
             mtlink_rb_map: HashMap<u8,u8>,
             theme        : ColorTheme) -> MTTab {
    let bins          = Uniform::new(50, 0.0, 50.0);
    let mtb_link_bins = Uniform::new(50, 0.0, 50.0);
    let panel_bins    = Uniform::new(22, 1.0, 22.0);
    Self {
      event_queue    : VecDeque::<MasterTriggerEvent>::with_capacity(1000),
      moni_queue     : VecDeque::<MtbMoniData>::with_capacity(1000),
      met_queue      : VecDeque::<f64>::with_capacity(1000),
      rate_queue     : VecDeque::<(f64,f64)>::with_capacity(1000),
      lost_r_queue   : VecDeque::<(f64,f64)>::with_capacity(1000),
      fpgatmp_queue  : VecDeque::<(f64,f64)>::with_capacity(1000),
      tp_receiver    : tp_receiver,
      mte_receiver   : mte_receiver,
      queue_size     : 1000, // random
      n_events       : 0,
      n_moni         : 0,
      miss_evid      : 0,
      last_evid      : 0,
      nch_histo      : ndhistogram!(bins),
      mtb_link_histo : ndhistogram!(mtb_link_bins),
      panel_histo    : ndhistogram!(panel_bins),
      theme          : theme,
      mapping        : mapping,
      mtlink_rb_map  : mtlink_rb_map,
      problem_hits   : Vec::<(u8, u8, (u8, u8), LTBThreshold)>::new(),
      timer          : Instant::now(),
    }
  }

  pub fn receive_packet(&mut self) -> Result<(), SerializationError> {
    let mut mte = MasterTriggerEvent::new();
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
        //error!("Got next packet {}!", pack);
        match pack.packet_type {
          PacketType::MasterTrigger => {
            match pack.unpack() {
              Ok(_ev) => {
                mte = _ev;
              },
              Err(err) => {
                error!("Unable to unpack MasterTriggerEvent! {err}");
                return Err(err);
              }
            }
          },
          PacketType::MonitorMtb   => {
            info!("Got new MtbMoniData!");
            let moni : MtbMoniData = pack.unpack()?;
            self.n_moni += 1;
            self.moni_queue.push_back(moni);
            if self.moni_queue.len() > self.queue_size {
              self.moni_queue.pop_front();
            }
            self.rate_queue.push_back((met, moni.rate as f64));
            if self.rate_queue.len() > self.queue_size {
              self.rate_queue.pop_front();
            }
            self.lost_r_queue.push_back((met, moni.lost_rate as f64));
            if self.lost_r_queue.len() > self.queue_size {
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
      let hits     = mte.get_trigger_hits();
      let rb_links = mte.get_rb_link_ids();
      for h in &hits {
        match self.mapping.get(&h.0) {
          None => {
            error!("Can't get mapping for hit {:?}", h);
            //self.problem_hits.push(*h);
          },
          Some(jmap) => {
            match jmap.get(&h.1) {
              None => {
                error!("Can't get mapping for hit {:?}", h);
                self.problem_hits.push(*h);
              },
              Some(chmap) => {
                // let's just consider one side of the paddle 
                // here. If the two sides are not connected to 
                // the same LTB we have bigger problems
                match chmap.get(&h.2.0) {
                  None => {
                    error!("Can't get mapping for hit {:?}", h);
                    self.problem_hits.push(*h);
                  },
                  Some((_,panel_id)) => {
                    self.panel_histo.fill(&(*panel_id as f32));
                  }
                }
              }
            }
          }
        }
        //self.panel_histo.fill(&(self.mapping[&h.0][&h.1][&h.2].1 as f32));
      }
      self.nch_histo.fill(&(hits.len() as f32));
      for k in rb_links {
        // FIXME unwrap
        let linked_rbid = self.mtlink_rb_map.get(&k).unwrap_or(&0);
        self.mtb_link_histo.fill(&(*linked_rbid as f32));
      }
      self.n_events += 1;
      self.event_queue.push_back(mte.clone());
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
          [Constraint::Percentage(70),
           Constraint::Percentage(30)].as_ref(),
      )
      .split(*main_window);
   
    // these are the 3 plots on the right side
    let info_chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
          [Constraint::Percentage(30),
           Constraint::Percentage(30),
           Constraint::Percentage(40),
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
          [Constraint::Percentage(33),
           Constraint::Percentage(33),
           Constraint::Percentage(34),
          ].as_ref(),
      )
      .split(detail_chunks[0]);
    let trig_pan_and_hits = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
          [Constraint::Percentage(50),
           Constraint::Percentage(50)].as_ref(),
      )
      .split(view_chunks[2]);
      
    let bottom_row = detail_chunks[1];

    //let main_layout   = main_chunks.to_vec();
    //let detail_layout = detail_chunks.to_vec();
    let info_layout   = info_chunks.to_vec();
    let view_layout   = view_chunks.to_vec();

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
          .border_type(BorderType::Rounded),
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
    let nch_labels  = create_labels(&self.nch_histo);
    let nch_data    = prep_data(&self.nch_histo, &nch_labels, 2, true); 
    let nch_chart   = histogram(nch_data, String::from("N Hits (N CH)"), 2, 0, &self.theme);

    // FPGA temperature (future)
    //let fpga_t_label    = String::from("FPGA T [\u{00B0}C] ");
    //let fpga_t_theme    = self.theme.clone();
    //let mut fpga_t_data = self.fpgatmp_queue.clone(); //.make_contiguous();
    //let mut fpga_t_ts   = timeseries(&mut fpga_t_data,
    //                                 fpga_t_label.clone(),
    //                                 fpga_t_label.clone(),
    //                                 &fpga_t_theme);
    //frame.render_widget(fpga_t_ts, info_layout[2]);

    // Lost Trigger rate
    let lost_t_label    = String::from("Lost Trigger Rate [Hz]");
    let lost_t_theme    = self.theme.clone();
    let mut lost_t_data = self.lost_r_queue.clone(); //.make_contiguous();
    let lost_t_ts       = timeseries(&mut lost_t_data,
                                     lost_t_label.clone(),
                                     lost_t_label.clone(),
                                     &lost_t_theme);

    
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
          .border_type(BorderType::Rounded),
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
    
    // histograms
    let ml_labels  = create_labels(&self.mtb_link_histo);
    let mlh_data   = prep_data(&self.mtb_link_histo, &ml_labels, 10, true); 
    // this actually now is the RB ID!
    let mlh_chart  = histogram(mlh_data, String::from("RB ID"), 3, 0, &self.theme);
    frame.render_widget(mlh_chart, bottom_row);   
    
    let tp_labels  = create_labels(&self.panel_histo);
    let tph_data   = prep_data(&self.panel_histo, &tp_labels, 2, true); 
    let tpc_chart  = histogram(tph_data, String::from("Triggered Panel ID"), 3, 0, &self.theme);
    //frame.render_widget(tpc_chart,      view_layout[2]);
    frame.render_widget(tpc_chart, trig_pan_and_hits[0]);
    frame.render_widget(nch_chart, trig_pan_and_hits[1]);

    // render everything
    frame.render_widget(rate_chart,      info_layout[0]); 
    frame.render_widget(lost_t_ts,       info_layout[1]);
    //frame.render_widget(nch_chart,       info_layout[1]);
    frame.render_widget(fpga_temp_chart, info_layout[2]);
    frame.render_widget(event_view,      view_layout[0]);
    frame.render_widget(moni_view,       view_layout[1]);
    //frame.render_widget(summary_view,    bottom_row[0]);
  }
}
    
