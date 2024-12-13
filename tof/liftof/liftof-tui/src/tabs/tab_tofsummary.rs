use std::collections::VecDeque;

use crossbeam_channel::Receiver;

//extern crate ndhistogram;
use ndhistogram::{
  ndhistogram,
  Histogram,
  Hist1D,
};

use ndhistogram::axis::{
  Uniform,
};

use ratatui::{
  //symbols,
  layout::{
      Alignment,
      Constraint,
      Direction,
      Layout,
      Rect
  },
  style::{
      //Color,
      //Modifier,
      Style
  },
  //text::Span,
  //terminal::Frame,
  Frame,
  widgets::{
      Block,
      //Dataset,
      //Axis,
      //GraphType,
      BorderType,
      //Chart,
      //BarChart,
      Sparkline,
      Borders,
      Paragraph
  },
};

//use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::events::TofEventSummary;
use tof_dataclasses::errors::SerializationError;
//use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::database::DsiJChPidMapping;

use crate::colors::{
    ColorTheme,
};

use crate::widgets::{
    //clean_data,
    prep_data,
    create_labels,
    histogram,
    gauge,
};

#[derive(Debug, Clone)]
pub struct TofSummaryTab {
  pub ts_receiver     : Receiver<TofEventSummary>,
  pub summary_queue   : VecDeque<TofEventSummary>,
  pub queue_size      : usize,
  pub n_trg_pdl_histo : Hist1D<Uniform<f32>>, 
  pub theme           : ColorTheme,

  // missing event analysis
  pub event_id_test   : Vec<u32>,
  pub evid_test_info  : String,
  pub evid_test_len   : usize,
  pub n_evid_test     : usize,
  pub evid_test_chnks : VecDeque<u64>,

  // missing HG hit analysis
  pub miss_hg_hits    : Hist1D<Uniform<f32>>,
  pub pid_map         : DsiJChPidMapping,
}

impl TofSummaryTab {
  pub fn new(ts_receiver  : Receiver<TofEventSummary>,
             dsijchpidmap : &DsiJChPidMapping,
             theme        : ColorTheme) -> Self {
    
    let bins          = Uniform::new(25, 0.0, 25.0).unwrap();
    let mhg_bins      = Uniform::new(160, 0.0, 160.0).unwrap();
    Self {
      ts_receiver     : ts_receiver,
      summary_queue   : VecDeque::<TofEventSummary>::new(),
      queue_size      : 10000,
      n_trg_pdl_histo : ndhistogram!(bins),
      theme           : theme,
      event_id_test   : Vec::<u32>::with_capacity(100000),
      evid_test_info  : String::from("Missing event id analysis"),
      evid_test_len   : 20000,
      n_evid_test     : 0,
      evid_test_chnks : VecDeque::<u64>::new(),
      miss_hg_hits    : ndhistogram!(mhg_bins),
      pid_map         : dsijchpidmap.clone(),
    }
  }

  pub fn receive_packet(&mut self) -> Result<(), SerializationError> {
    //let mut ts = TofEventSummary::new();
    match self.ts_receiver.try_recv() {
      Err(_err)  => {
        trace!("Unable to receive new TofEventSummary!");
      },
      Ok(ts)    => {
        // triggerd paddles histogram
        self.n_trg_pdl_histo.fill(&(ts.n_trigger_paddles as f32));
        // missing hg hits for paddles histogram
        let missing_hg = ts.get_missing_paddles_hg(&self.pid_map);
        for pid in &missing_hg {
            self.miss_hg_hits.fill(&(*pid as f32));
        }
        if self.event_id_test.len() != self.evid_test_len {
          self.event_id_test.push(ts.event_id);
        } else {
          //let mut miss_pos = Vec::<usize>::new();
          let mut missing = 0usize;
          let mut evid = self.event_id_test[0];
          for _ in 0..self.event_id_test.len() {
            if !self.event_id_test.contains(&evid) {
              missing += 1;
              //miss_pos.push(k);
            }
            evid += 1;
          }
          self.n_evid_test += 1;
          self.evid_test_chnks.push_back(missing as u64);
          if self.evid_test_chnks.len() > 100 {
            self.evid_test_chnks.pop_front();
          }
          self.evid_test_info  = format!("Missing event ID search [{}]", self.n_evid_test);
          self.evid_test_info += &(format!("\n-- in a chunk of {} event ids", self.evid_test_len)); 
          self.evid_test_info += &(format!("\n-- we found {} event ids missing ({}%)", missing, 100.0*(missing as f64)/self.event_id_test.len() as f64));
          self.evid_test_info += &(format!("\n-- -- previous: {:?}", self.evid_test_chnks));
          self.event_id_test.clear();
        }
        self.summary_queue.push_back(ts);
        if self.summary_queue.len() > self.queue_size {
          self.summary_queue.pop_front();
        }
      }
    }
    Ok(())
  }
  
  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    let layout = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
          [Constraint::Percentage(30),
           Constraint::Percentage(70)].as_ref(),
      )
      .split(*main_window);
   
    let histo_view = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
          [Constraint::Percentage(33),
           Constraint::Percentage(33),
           Constraint::Percentage(34)].as_ref(),
      )  
      .split(layout[1]);

    let evid_test_view = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
        [Constraint::Percentage(70),
         Constraint::Percentage(30)].as_ref(),
      )
      .split(histo_view[2]);
    
    let evid_test_view_0 = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
        [Constraint::Percentage(30),
         Constraint::Percentage(70)].as_ref(),
      )
      .split(evid_test_view[0]);

    let last_ts = self.summary_queue.back();
    let view_string : String;
    match last_ts {
      Some(ts) => { 
        view_string = ts.to_string();
      }, 
      None => {
        view_string = String::from("TofEventSummary QUEUE EMPTY!");
      }
    }
    let event_view = Paragraph::new(view_string)
      .style(Style::default().fg(self.theme.fg0))
      .alignment(Alignment::Left)
      //.scroll((5, 10))
      .block(
        Block::default()
          .borders(Borders::ALL)
          .style(self.theme.style())
          .title("Last TofEventSummary")
          .border_type(BorderType::Rounded),
      );
    frame.render_widget(event_view, layout[0]);
     
    // histograms
    let th_labels  = create_labels(&self.n_trg_pdl_histo);
    let th_data    = prep_data(&self.n_trg_pdl_histo, &th_labels, 5, true); 
    let th_chart   = histogram(th_data, String::from("N Trig Paddles"), 2, 0, &self.theme);
    frame.render_widget(th_chart, histo_view[0]); 
   
    let mhg_labels = create_labels(&self.miss_hg_hits);
    let mhg_data   = prep_data(&self.miss_hg_hits, &mhg_labels, 10, true);
    let mhg_chart  = histogram(mhg_data, String::from("Missing HG hits"), 1, 0, &self.theme);
    frame.render_widget(mhg_chart, histo_view[1]);

    let evid_test_data = Paragraph::new(self.evid_test_info.clone())
      .style(Style::default().fg(self.theme.fg0))
      .alignment(Alignment::Left)
      //.scroll((5, 10))
      .block(
        Block::default()
          .borders(Borders::ALL)
          .style(self.theme.style())
          .title("Missing event ID test")
          .border_type(BorderType::Rounded),
      );
    let mut spl_data  = Vec::<u64>::new();
    spl_data.extend_from_slice(self.evid_test_chnks.make_contiguous());
    // that the sparkline does something, it can't be zero. 
    // There is no axis marker, so we just add 1 to every bin
    for k in 0..spl_data.len() {
      spl_data[k] += 1;
    }
    let sparkline = Sparkline::default()
      .style(self.theme.style())
      //.direction(RenderDirection::LeftToRight)
      //.data(self.evid_test_chnks.make_contiguous())
      .data(&spl_data)
      .block(
        Block::default()
        .borders(Borders::ALL)
        .style(self.theme.style())
        .title("Missing event IDs in chunks")
      );

    frame.render_widget(evid_test_data, evid_test_view_0[0]);
    frame.render_widget(sparkline, evid_test_view_0[1]);
    let ratio = self.event_id_test.len() as f64 / self.evid_test_len as f64;
    let test_gauge = gauge(String::from("Missing event ID check"), String::from("Gathering data"), ratio, &self.theme);
    frame.render_widget(test_gauge, evid_test_view[1]);
  }
}
