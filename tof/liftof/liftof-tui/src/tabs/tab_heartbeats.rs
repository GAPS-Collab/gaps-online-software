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
    List,
    ListItem,
    ListState,
};
//use tui::{
//    symbols,
//    backend::CrosstermBackend,
//    layout::{Alignment, Constraint, Direction, Layout, Rect},
//    style::{Color, Modifier, Style},
//    text::{Span, Spans, Text},
//    widgets::{
//        Block, Dataset, Axis, GraphType, BorderType, Chart, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,    },
//    Terminal,
//};


use tof_dataclasses::packets::{
  TofPacket,
  PacketType
};

use tof_dataclasses::serialization::{
    SerializationError,
    Serialization,
    Packable
};

use tof_dataclasses::heartbeats::{
  EVTBLDRHeartbeat,
  HeartBeatDataSink,
  MTBHeartbeat,
};

use crate::colors::ColorTheme;

pub enum HeartBeatView {
  EventBuilder,
  MTB,
  DataSink
}

pub struct HeartBeatTab {
  pub theme      : ColorTheme,
  pub evb_queue  : VecDeque<EVTBLDRHeartbeat>,
  pub mtb_queue  : VecDeque<MTBHeartbeat>,
  pub gds_queue  : VecDeque<HeartBeatDataSink>,
  pub pkt_recv   : Receiver<TofPacket>,
  last_evb       : Option<EVTBLDRHeartbeat>,
  last_mtb       : Option<MTBHeartbeat>,
  last_gds       : Option<HeartBeatDataSink>,
  pub view       : HeartBeatView,
  pub queue_size : usize,
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
      .direction(Direction::Horizontal)
      .constraints(
          [Constraint::Percentage(100)].as_ref(),
      )
      .split(*main_window);
    

    let last_evbhb = self.evb_queue.back();
    let last_mtbhb = self.mtb_queue.back();
    let last_dshb  = self.gds_queue.back();

    let mut view_string = String::from("HB QUEUE EMPTY!");
    let mut evb_is_empty = false;
    let mut mtb_is_empty = false;
    let mut gds_is_empty = false;
    self.last_evb = self.evb_queue.back().copied();
    self.last_mtb = self.mtb_queue.back().copied();
    self.last_gds = self.gds_queue.back().copied();

    match self.view {
      HeartBeatView::EventBuilder => {
        if self.last_evb.is_some() {
          view_string = self.last_evb.unwrap().to_string();
        }
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
    
