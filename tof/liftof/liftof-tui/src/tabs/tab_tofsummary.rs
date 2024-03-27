use std::collections::VecDeque;

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
        Modifier,
        Style
    },
    //text::Span,
    terminal::Frame,
    widgets::{
        Block,
        //Dataset,
        //Axis,
        //GraphType,
        BorderType,
        //Chart,
        BarChart,
        Borders,
        Paragraph
    },
};

use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::events::TofEventSummary;
use tof_dataclasses::errors::SerializationError;
use tof_dataclasses::serialization::Serialization;

use crate::colors::{
    ColorTheme,
};

use crate::widgets::{
    //clean_data,
    prep_data,
    create_labels,
};

#[derive(Debug, Clone)]
pub struct TofSummaryTab {
  pub ts_receiver     : Receiver<TofEventSummary>,
  pub summary_queue   : VecDeque<TofEventSummary>,
  pub queue_size      : usize,
  pub n_trg_pdl_histo : Hist1D<Uniform<f32>>, 
  pub theme           : ColorTheme
}

impl TofSummaryTab {
  pub fn new(ts_receiver : Receiver<TofEventSummary>,
             theme       : ColorTheme) -> Self {
    
    let bins          = Uniform::new(25, 0.0, 24.0);
    Self {
        ts_receiver     : ts_receiver,
        summary_queue   : VecDeque::<TofEventSummary>::new(),
        queue_size      : 10000,
        n_trg_pdl_histo : ndhistogram!(bins),
        theme           : theme
    }
  }

  pub fn receive_packet(&mut self) -> Result<(), SerializationError> {
    //let mut ts = TofEventSummary::new();
    match self.ts_receiver.try_recv() {
      Err(_err)  => {
        trace!("Unable to receive new TofEventSummary!");
      },
      Ok(ts)    => {
        //let ts = TofEventSummary::from_tofpacket(&tp)?;
        self.n_trg_pdl_histo.fill(&(ts.n_trigger_paddles as f32));
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
    let th_data    = prep_data(&self.n_trg_pdl_histo, &th_labels, 5); 
    let th_chart   = BarChart::default()
      .block(Block::default().title("N Trig Paddles").borders(Borders::ALL))
      .data(th_data.as_slice())
      .bar_width(2)
      .bar_gap(0)
      .bar_style(self.theme.highlight_fg())
      .value_style(
        self.theme.highlight_fg()
        .add_modifier(Modifier::BOLD),
      )
      .style(self.theme.background());
    frame.render_widget(th_chart, histo_view[0]); 
  }
}
