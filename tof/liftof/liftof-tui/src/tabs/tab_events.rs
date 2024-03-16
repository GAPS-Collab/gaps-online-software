use std::collections::{
    //HashMap,
    VecDeque,
};
//use std::sync::{
//    Arc,
//    Mutex,
//};

use crossbeam_channel::{
    Receiver,
    Sender,
};

use ratatui::prelude::*;

use ratatui::terminal::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{
    Block,
    BorderType,
    Borders,
    Paragraph,
};

use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::errors::SerializationError;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::events::{
    RBEvent,
    TofEvent,
    TofHit,
    TofEventHeader,
    MasterTriggerEvent,
};

use crate::colors::ColorTheme2;

#[derive(Debug, Clone)]
pub struct EventTab {
  pub theme         : ColorTheme2,
  pub tp_receiver   : Receiver<TofPacket>,
  pub event_queue   : VecDeque<TofEvent>,
  pub queue_size    : usize,
  pub mte_sender    : Sender<MasterTriggerEvent>,
  pub rbe_sender    : Sender<RBEvent>,
  //pub streamer   : Arc<Mutex<VecDeque<String>>>,
  //pub pack_stat  : Arc<Mutex<HashMap<String, usize>>>,
  //pub stream     : String,
  //pub stream_max : usize, 
}

impl EventTab {
  pub fn new(tp_receiver : Receiver<TofPacket>,
             mte_sender  : Sender<MasterTriggerEvent>,
             rbe_sender  : Sender<RBEvent>,
             theme       : ColorTheme2) -> EventTab {
             //streamer  : Arc<Mutex<VecDeque<String>>>,
             //pack_stat : Arc<Mutex<HashMap<String,usize>>>) -> HomeTab<T> {
    EventTab {
      theme,
      tp_receiver,
      event_queue : VecDeque::<TofEvent>::new(),
      queue_size  : 1000,
      mte_sender  : mte_sender,
      rbe_sender  : rbe_sender,
      //streamer, 
      //pack_stat,
      //stream     : String::from(""),
      //stream_max : 30,
    }
  }
 
  pub fn receive_packet(&mut self) -> Result<(), SerializationError> {  
    match self.tp_receiver.try_recv() {
      Err(_err) => {
        return Ok(());
      },
      Ok(pack)    => {
        let ev = TofEvent::from_bytestream(&pack.payload, &mut 0)?;
        match self.mte_sender.send(ev.mt_event.clone()) {
          Err(err) => error!("Can send MasterTriggerEvent! {err}"),
          Ok(_)    => ()
        }
        for k in ev.rb_events.iter() {
          match self.rbe_sender.send(k.clone()) {
            Err(err) => error!("Can not send RB Event! {err}"),
            Ok(_)    => {}
          }
        }
        self.event_queue.push_back(ev);
        if self.event_queue.len() > self.queue_size {
          self.event_queue.pop_front();
        }
        return Ok(());
      }
    }
  }

  // Color::Blue was nice for background
  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    
    // as usual, layout first
    let status_chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
          [Constraint::Percentage(30), Constraint::Percentage(70)].as_ref(),
      )
      .split(*main_window);
  
    let hit_chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
          [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
      )
      .split(status_chunks[1]);

    let header      = TofEventHeader::new();
    let hit         = TofHit::new();
    let mut header_string = header.to_string();
    let mut hit_string    = hit.to_string(); 
    match self.event_queue.back() {
      None => (),
      Some(ev)   => {
        header_string = ev.header.to_string();
        let info_field = format!("\n --> NRBs {}\n --> NMissingHit {}\n Quality: {}\n CompressionLevel {}",
                                 ev.rb_events.len(), ev.missing_hits.len(), ev.quality, ev.compression_level);
        header_string += &info_field;
        if ev.rb_events.len() > 0 {
          if ev.rb_events[0].hits.len() > 0 {
            hit_string = ev.rb_events[0].hits[0].to_string();
          } else {
            hit_string = String::from("NO HIT IN RB!"); 
          }
        } else {
          hit_string = String::from("NO HIT!");
        }
      }
    }
    let header_view = Paragraph::new(header_string)
      .style(self.theme.style())
      .alignment(Alignment::Left)
      .block(
        Block::default()
          .borders(Borders::ALL)
          .border_type(BorderType::Rounded)
          .title("Last TofEvent")
      );
    frame.render_widget(header_view, status_chunks[0]);
    let hit_view   = Paragraph::new(hit_string)
      .style(self.theme.style())
      .alignment(Alignment::Left)
      .block(
        Block::default()
          .borders(Borders::ALL)
          .border_type(BorderType::Rounded)
          .title("Last TofHit")
      );
    frame.render_widget(hit_view, hit_chunks[0]);

  }
}
