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

use telemetry_dataclasses::packets::{
  TelemetryHeader,
  TelemetryPacket,
  MergedEvent,
//  TrackerPacket,
};

use ratatui::prelude::*;

//use ratatui::terminal::Frame;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{
    Block,
    BorderType,
    Borders,
    Paragraph,
};

use tof_dataclasses::serialization::{
    Serialization,
//    search_for_u16
};
use tof_dataclasses::errors::SerializationError;
use tof_dataclasses::packets::{
    TofPacket,
//    PacketType,
};

//use tof_dataclasses::events::{
//    RBEvent,
//    TofEvent,
//    TofHit,
//    TofEventHeader,
//    TofEventSummary,
//    MasterTriggerEvent,
//};

use crate::colors::ColorTheme;

// no clone ore debug implemented for 
// zmq socket
pub struct TelemetryTab {
  pub theme         : ColorTheme, 
  //pub tp_receiver   : Receiver<TofPacket>,
  //pub event_queue   : VecDeque<TofEvent>,
  //pub zmq_socket    : zmq::Socket,
  pub tele_recv     : Receiver<TelemetryPacket>,
  pub queue_size    : usize, //8
  pub merged_queue  : VecDeque<MergedEvent>,
  pub header_queue  : VecDeque<TelemetryHeader>,
  // when we decide to use ONLY the telemetry stream,
  // we have to pass on the decoded TOF packets
  pub tp_sender     : Option<Sender<TofPacket>>,
  
  //pub mte_sender    : Sender<MasterTriggerEvent>,
  //pub rbe_sender    : Sender<RBEvent>,
  //pub th_sender     : Sender<TofHit>,
  //pub streamer   : Arc<Mutex<VecDeque<String>>>,
  //pub pack_stat  : Arc<Mutex<HashMap<String, usize>>>,
  //pub stream     : String,
  //pub stream_max : usize, 
}

impl TelemetryTab {
  pub fn new(//tp_receiver : Receiver<TofPacket>,
             //mte_sender  : Sender<MasterTriggerEvent>,
             //rbe_sender  : Sender<RBEvent>,
             tp_sender   : Option<Sender<TofPacket>>,
             tele_recv   : Receiver<TelemetryPacket>,
             theme       : ColorTheme) -> Self {
             //streamer  : Arc<Mutex<VecDeque<String>>>,
             //pack_stat : Arc<Mutex<HashMap<String,usize>>>) -> HomeTab<T> {
    Self {
      theme,
      tele_recv    : tele_recv,
      queue_size   : 20000,
      merged_queue : VecDeque::<MergedEvent>::new(),
      header_queue : VecDeque::<TelemetryHeader>::new(),
      tp_sender    : tp_sender,
    }
  }
 
  pub fn receive_packet(&mut self) -> Result<(), SerializationError> {  
    match self.tele_recv.try_recv() {
        Err(crossbeam_channel::TryRecvError::Empty) => {
            trace!("No data available yet.");
            // Handle the empty case, possibly by doing nothing or logging.
        },
        Err(crossbeam_channel::TryRecvError::Disconnected) => {
            error!("Telemetry channel disconnected.");
            // Handle the disconnection, possibly by stopping processing or returning an error.
            return Err(SerializationError::Disconnected);
        },
        Ok(packet) => {
            // Process the received packet as before
            self.header_queue.push_back(packet.header.clone());
            if self.header_queue.len() > self.queue_size {
                let _ = self.header_queue.pop_front();
            }

            if packet.header.ptype == 90 {
                let expected_size = (packet.header.length as usize) - TelemetryHeader::SIZE;
                if expected_size > packet.payload.len() {
                    println!("Unable to decode MergedEvent Telemetry packet! The expected size is {}, but the payload len is {}", expected_size - TelemetryHeader::SIZE, packet.payload.len());
                    return Err(SerializationError::StreamTooShort);
                }
                match MergedEvent::from_bytestream(&packet.payload, &mut 0) {
                    Err(err) => {
                        error!("Unable to decode MergedEvent! {err}");
                    }
                    Ok(me) => {
                        if let Some(sender) = &self.tp_sender {
                            match TofPacket::from_bytestream(&me.tof_data, &mut 0) {
                                Err(err) => {
                                    error!("Can't unpack TofPacket! {err}");
                                }
                                Ok(tp) => {
                                    if let Err(err) = sender.send(tp) {
                                        error!("Unable to send TP over channel! {err}");
                                    }
                                }
                            }
                        }
                        self.merged_queue.push_back(me);
                        if self.merged_queue.len() > self.queue_size {
                            let _ = self.merged_queue.pop_front();
                        }
                    }
                }
            }
        }
    }
    Ok(())
  }
    //match self.tp_receiver.try_recv() {
    //  Err(_err) => {
    //    return Ok(());
    //  },
    //  Ok(pack)    => {
    //    let ev : TofEvent = pack.unpack()?;
    //    match self.mte_sender.send(ev.mt_event.clone()) {
    //      Err(err) => error!("Can send MasterTriggerEvent! {err}"),
    //      Ok(_)    => ()
    //    }
    //    for k in ev.rb_events.iter() {
    //      let rb_ev = k.clone();
    //      for h in rb_ev.hits.iter() {
    //        match self.th_sender.send(h.clone()) {
    //          Err(err) => error!("Can not send TofHit! {err}"),
    //          Ok(_)    => ()
    //        }
    //      }
    //      match self.rbe_sender.send(k.clone()) {
    //        Err(err) => error!("Can not send RBEvent! {err}"),
    //        Ok(_)    => ()
    //      }
    //    }
    //    self.event_queue.push_back(ev);
    //    if self.event_queue.len() > self.queue_size {
    //      self.event_queue.pop_front();
    //    }
    //    return Ok(());
    //  }
    //}

//   pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    
//     // as usual, layout first
//     let main_lo = Layout::default()
//       .direction(Direction::Horizontal)
//       .constraints(
//           [Constraint::Percentage(30), Constraint::Percentage(70)].as_ref(),
//       )
//       .split(*main_window);
    
//     let packet_lo = Layout::default()
//       .direction(Direction::Vertical)
//       .constraints(
//           [Constraint::Percentage(30), Constraint::Percentage(70)].as_ref(),
//       )
//       .split(main_lo[0]);


//     let mut header_string = String::from("");
//     if let Some(header) = self.header_queue.back() {
//       header_string = format!("{}", header);
//     }
//     let header_view = Paragraph::new(header_string)
//       .style(self.theme.style())
//       .alignment(Alignment::Left)
//       .block(
//         Block::default()
//           .borders(Borders::ALL)
//           .border_type(BorderType::Rounded)
//           .title("Last Header from Telemetry stream")
//       );

//     frame.render_widget(header_view, packet_lo[0]);
//     let mut merged_string = String::from("");
//     if let Some(ev) =  self.merged_queue.back() {
//       merged_string = format!("{}", ev);
//     }
//     let merged_view = Paragraph::new(merged_string)
//     //let merged_view = Paragraph::new(header_string)
//       .style(self.theme.style())
//       .alignment(Alignment::Left)
//       .block(
//         Block::default()
//           .borders(Borders::ALL)
//           .border_type(BorderType::Rounded)
//           .title("Last MergedEvent from Telemetry stream")
//       );
//     frame.render_widget(merged_view, packet_lo[1]);
//   }
// }

pub fn render(&mut self, main_window: &Rect, frame: &mut Frame) {
  // Layout first
  let main_lo = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
          [Constraint::Percentage(30), Constraint::Percentage(70)].as_ref(),
      )
      .split(*main_window);
  
  let packet_lo = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
          [Constraint::Percentage(30), Constraint::Percentage(70)].as_ref(),
      )
      .split(main_lo[0]);

  // Create header_string safely
  let header_string = if let Some(header) = self.header_queue.back() {
      format!("{}", header)
  } else {
      String::from("No header available")
  };
  
  let header_view = Paragraph::new(header_string)
      .style(self.theme.style())
      .alignment(Alignment::Left)
      .block(
          Block::default()
              .borders(Borders::ALL)
              .border_type(BorderType::Rounded)
              .title("Last Header from Telemetry stream")
      );

  frame.render_widget(header_view, packet_lo[0]);

  // Create merged_string safely
  let merged_string = if let Some(ev) = self.merged_queue.back() {
      format!("{}", ev)
  } else {
      String::from("No merged event available")
  };
  
  let merged_view = Paragraph::new(merged_string)
      .style(self.theme.style())
      .alignment(Alignment::Left)
      .block(
          Block::default()
              .borders(Borders::ALL)
              .border_type(BorderType::Rounded)
              .title("Last MergedEvent from Telemetry stream")
      );

  frame.render_widget(merged_view, packet_lo[1]);
  }
}
