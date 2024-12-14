//! Heartbeats are a special kind of monitoring
//! which monitor the individual main threads of 
//! liftof-cc and are sent in regular intervals
//! 
//! The main threads are for the Master Trigger,
//! Event Builder and the Data sender.

//use chrono::Utc;
use std::collections::VecDeque; 

use crossbeam_channel::Receiver;

use ratatui::prelude::*;
use ratatui::widgets::{
  Block,
  BorderType,
  Borders,
  Paragraph,
  //BarChart,
};

use tof_dataclasses::packets::{
  TofPacket,
  PacketType
};

use tof_dataclasses::serialization::{
  SerializationError,
};

use tof_dataclasses::heartbeats::{
  EVTBLDRHeartbeat,
  HeartBeatDataSink,
  MTBHeartbeat,
};

use crate::colors::ColorTheme;
use crate::widgets::timeseries;

pub enum HeartBeatView {
  EventBuilder,
  MTB,
  DataSink
}

pub struct HeartBeatTab {
  pub theme      : ColorTheme,
  // FIXME - we don't seemt to need this queues, 
  // apparently 
  pub evb_queue  : VecDeque<EVTBLDRHeartbeat>,
  pub mtb_queue  : VecDeque<MTBHeartbeat>,
  pub gds_queue  : VecDeque<HeartBeatDataSink>,
  pub pkt_recv   : Receiver<TofPacket>,
  last_evb       : Option<EVTBLDRHeartbeat>,
  last_mtb       : Option<MTBHeartbeat>,
  last_gds       : Option<HeartBeatDataSink>,
  pub view       : HeartBeatView,
  pub queue_size : usize,

  // containers for a bunch of nice looking plots
  pub ev_c_q     : VecDeque<(f64,f64)>,
  pub to_q       : VecDeque<(f64,f64)>,
  pub mangl_q    : VecDeque<(f64,f64)>,
  pub rb_disc_q  : VecDeque<(f64,f64)>,
  pub lhit_fr_q  : VecDeque<(f64,f64)>,
  pub ch_len_mte : VecDeque<(f64,f64)>,
  pub ch_len_rbe : VecDeque<(f64,f64)>,
}

impl HeartBeatTab{

  pub fn new(pkt_recv     : Receiver<TofPacket>,
             theme        : ColorTheme) -> HeartBeatTab{  

    HeartBeatTab {
      theme   ,
      evb_queue  : VecDeque::<EVTBLDRHeartbeat>::new(),
      mtb_queue  : VecDeque::<MTBHeartbeat>::new(),
      gds_queue  : VecDeque::<HeartBeatDataSink>::new(),
      last_evb   : None,
      last_mtb   : None,
      last_gds   : None,
      pkt_recv   : pkt_recv,
      view       : HeartBeatView::EventBuilder,
      queue_size : 1000,
  
      ev_c_q     : VecDeque::<(f64,f64)>::with_capacity(1000),
      to_q       : VecDeque::<(f64,f64)>::with_capacity(1000),
      mangl_q    : VecDeque::<(f64,f64)>::with_capacity(1000),
      rb_disc_q  : VecDeque::<(f64,f64)>::with_capacity(1000),
      lhit_fr_q  : VecDeque::<(f64,f64)>::with_capacity(1000),
      ch_len_mte : VecDeque::<(f64,f64)>::with_capacity(1000),
      ch_len_rbe : VecDeque::<(f64,f64)>::with_capacity(1000),
    }
  }
  
  pub fn receive_packet(&mut self) -> Result<(), SerializationError> {
    match self.pkt_recv.try_recv() {
      Err(_err)   => {
        debug!("Unable to receive heartbeat TofPacket!");  
      }
      Ok(pack)    => {
        //println!("Received {}", pack);
        match pack.packet_type {
          PacketType::MTBHeartbeat=> {
            let hb : MTBHeartbeat = pack.unpack()?;
            self.mtb_queue.push_back(hb);
            if self.mtb_queue.len() > self.queue_size {
              self.mtb_queue.pop_front(); 
            }
          },
          PacketType::EVTBLDRHeartbeat   => {
            let hb : EVTBLDRHeartbeat = pack.unpack()?;
            self.evb_queue.push_back(hb);
            if self.evb_queue.len() > self.queue_size {
              self.evb_queue.pop_front(); 
            }
            self.ev_c_q    .push_back((hb.met_seconds as f64,hb.event_cache_size as f64));
            if self.ev_c_q.len() > self.queue_size {
              self.ev_c_q.pop_front(); 
            }
            self.to_q      .push_back((hb.met_seconds as f64,hb.get_timed_out_frac()*100.0));
            if self.to_q.len() > self.queue_size {
              self.to_q.pop_front(); 
            }
            self.mangl_q   .push_back((hb.met_seconds as f64,hb.get_mangled_frac()*100.0));
            if self.mangl_q.len() > self.queue_size {
              self.mangl_q.pop_front(); 
            }
            self.rb_disc_q .push_back((hb.met_seconds as f64,hb.get_nrbe_discarded_frac()*100.0));
            if self.rb_disc_q.len() > self.queue_size {
              self.rb_disc_q.pop_front(); 
            }
            self.lhit_fr_q .push_back((hb.met_seconds as f64,hb.get_drs_lost_frac()*100.0));
            if self.lhit_fr_q.len() > self.queue_size {
              self.lhit_fr_q.pop_front(); 
            }
            self.ch_len_mte.push_back((hb.met_seconds as f64,hb.mte_receiver_cbc_len as f64));
            if self.ch_len_mte.len() > self.queue_size {
              self.ch_len_mte.pop_front(); 
            }
            self.ch_len_rbe.push_back((hb.met_seconds as f64,hb.rbe_receiver_cbc_len as f64));
            if self.ch_len_rbe.len() > self.queue_size {
              self.ch_len_rbe.pop_front(); 
            }
          }
          PacketType::HeartBeatDataSink => {
            let hb : HeartBeatDataSink = pack.unpack()?;
            self.gds_queue.push_back(hb);
            if self.gds_queue.len() > self.queue_size {
              self.gds_queue.pop_front(); 
            }
          }
          _ => () // we don't care
        }
      }
    }
    Ok(())
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {

    let main_lo = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
          [Constraint::Percentage(50),
           Constraint::Percentage(50)].as_ref(),
      )
      .split(*main_window);

    let mut view_string = String::from("HB QUEUE EMPTY!");
    self.last_evb = self.evb_queue.back().copied();
    self.last_mtb = self.mtb_queue.back().copied();
    self.last_gds = self.gds_queue.back().copied();

    match self.view {
      HeartBeatView::EventBuilder => {
        if self.last_evb.is_some() {
          view_string = self.last_evb.unwrap().to_string();
        }
        // A bunch of charts :
        //
        // event_cache   Lost hit frac 
        // time_out ev   Ch len MTE rec
        // data mangl    Ch len RBE rec
        // RBEv discar   XX
        //
        let hb_ev_cols = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(50),
               Constraint::Percentage(50)].as_ref(),
          )
          .split(main_lo[1]);
        let hb_ev_rows_left = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(25),
               Constraint::Percentage(25),
               Constraint::Percentage(25),
               Constraint::Percentage(25)].as_ref(),
          )
          .split(hb_ev_cols[0]);
        let hb_ev_rows_right = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(25),
               Constraint::Percentage(25),
               Constraint::Percentage(25),
               Constraint::Percentage(25)].as_ref(),
          )
          .split(hb_ev_cols[1]);
       
        // event cache graph 
        let mut ts_label   = String::from("Size of event cache [#evts]");
        let ts_ev_theme    = self.theme.clone();
        let mut ts_ev_data = self.ev_c_q.clone(); 
        let ts_ev          = timeseries(&mut ts_ev_data,
                                        ts_label.clone(),
                                        ts_label.clone(),
                                        &ts_ev_theme);
        frame.render_widget(ts_ev, hb_ev_rows_left[0]);
        
        // time out event graph 
        ts_label           = String::from("Fraction of timed out events [%]");
        let ts_to_theme    = self.theme.clone();
        let mut ts_to_data = self.to_q.clone(); 
        let ts_to          = timeseries(&mut ts_to_data,
                                        ts_label.clone(),
                                        ts_label.clone(),
                                        &ts_to_theme);
        frame.render_widget(ts_to, hb_ev_rows_left[1]);
        
        // data mangl graph 
        ts_label           = String::from("Fraction of mangled events [%]");
        let ts_dm_theme    = self.theme.clone();
        let mut ts_dm_data = self.mangl_q.clone(); 
        let ts_dm          = timeseries(&mut ts_dm_data,
                                        ts_label.clone(),
                                        ts_label.clone(),
                                        &ts_dm_theme);
        frame.render_widget(ts_dm, hb_ev_rows_left[2]);
        
        // rb events discarded graph
        ts_label               = String::from("Fraction of discarded RBEvents [%]");
        let ts_rbdisc_theme    = self.theme.clone();
        let mut ts_rbdisc_data = self.rb_disc_q.clone(); 
        let ts_rbdisc          = timeseries(&mut ts_rbdisc_data,
                                            ts_label.clone(),
                                            ts_label.clone(),
                                            &ts_rbdisc_theme);
        frame.render_widget(ts_rbdisc, hb_ev_rows_left[3]);
        
        // lost hit fraction
        ts_label           = String::from("Fraction of DRS4 dead hits [%]");
        let ts_lh_theme    = self.theme.clone();
        let mut ts_lh_data = self.lhit_fr_q.clone(); 
        let ts_lh          = timeseries(&mut ts_lh_data,
                                        ts_label.clone(),
                                        ts_label.clone(),
                                        &ts_lh_theme);
        frame.render_widget(ts_lh, hb_ev_rows_right[0]);
        
        // mte incoming ch len
        ts_label              = String::from("Incoming MTE buffer len [#ets]");
        let ts_mtech_theme    = self.theme.clone();
        let mut ts_mtech_data = self.ch_len_mte.clone(); 
        let ts_mtech          = timeseries(&mut ts_mtech_data,
                                           ts_label.clone(),
                                           ts_label.clone(),
                                           &ts_mtech_theme);
        frame.render_widget(ts_mtech, hb_ev_rows_right[1]);
        
        // rbe incoming ch len
        ts_label              = String::from("Incoming RBE buffer len [#ets]");
        let ts_rbech_theme    = self.theme.clone();
        let mut ts_rbech_data = self.ch_len_rbe.clone(); 
        let ts_rbech          = timeseries(&mut ts_rbech_data,
                                           ts_label.clone(),
                                           ts_label.clone(),
                                           &ts_rbech_theme);
        frame.render_widget(ts_rbech, hb_ev_rows_right[2]);
      
      }
      HeartBeatView::MTB => {
        if self.last_mtb.is_some() {
          view_string = self.last_mtb.unwrap().to_string();
        }
      }
      HeartBeatView::DataSink => {
        if self.last_gds.is_some() {
          view_string = self.last_gds.unwrap().to_string();
        }
      }
    }

    let hb_view = Paragraph::new(view_string)
      // FIXME color
      .style(Style::default().fg(Color::LightCyan))
      .alignment(Alignment::Left)
      //.scroll((5, 10))
      .block(
        Block::default()
          .borders(Borders::ALL)
          .style(self.theme.style())
          .title("Last Heartbeat \u{1f493}")
          .border_type(BorderType::Rounded),
      );
    frame.render_widget(hb_view, main_lo[0]);
  }
} // end impl
    
