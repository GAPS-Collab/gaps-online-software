use std::collections::{
  HashMap,
  VecDeque,
};

use std::time::Instant;

//use std::sync::{
//  Arc,
//  Mutex,
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
  Table,
  Row,
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

use telemetry_dataclasses::packets::TelemetryPacketType;

use crate::colors::ColorTheme;
use crate::telly_packet_counter;

#[derive(Debug, Copy, Clone)]
pub enum TelemetryTabView {
  Stream,
  MergedEvents
}

// no clone ore debug implemented for 
// zmq socket
pub struct TelemetryTab<'a> {
  pub theme         : ColorTheme, 
  pub tele_recv     : Receiver<TelemetryPacket>,
  pub queue_size    : usize, //8
  pub merged_queue  : VecDeque<MergedEvent>,
  pub header_queue  : VecDeque<TelemetryHeader>,
  // when we decide to use ONLY the telemetry stream,
  // we have to pass on the decoded TOF packets
  pub tp_sender     : Option<Sender<TofPacket>>,
  pub view          : TelemetryTabView, 
  pub pack_map      : HashMap<&'a str, usize>,
  start_time        : Instant,
}

impl TelemetryTab<'_> {
  pub fn new(tp_sender   : Option<Sender<TofPacket>>,
             tele_recv   : Receiver<TelemetryPacket>,
             theme       : ColorTheme) -> Self {
    Self {
      theme,
      tele_recv    : tele_recv,
      queue_size   : 20000,
      merged_queue : VecDeque::<MergedEvent>::new(),
      header_queue : VecDeque::<TelemetryHeader>::new(),
      tp_sender    : tp_sender,
      view         : TelemetryTabView::Stream, 
      pack_map     : HashMap::<&str, usize>::new(),
      start_time   : Instant::now()
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
        let telly_ptype = TelemetryPacketType::from(packet.header.ptype);
        telly_packet_counter(&mut self.pack_map, &telly_ptype);

        self.header_queue.push_back(packet.header.clone());
        if self.header_queue.len() > self.queue_size {
          let _ = self.header_queue.pop_front();
        }
        if packet.header.ptype == 92 {
          match TofPacket::from_bytestream(&packet.payload, &mut 0) {
            Err(err) => {
              error!("Unable to decode AnyHKpacket! {err}");
            }
            Ok(tpack) => {
              if let Some(sender) = &self.tp_sender {
                if let Err(err) = sender.send(tpack) {
                  error!("Unable to send TP over channel! {err}");
                }
              }
            }
          }
        }
        
        if packet.header.ptype == 90 || packet.header.ptype == 191  {
          let expected_size = (packet.header.length as usize) - TelemetryHeader::SIZE;
          if expected_size > packet.payload.len() {
            error!("Unable to decode MergedEvent Telemetry packet of type {}! The expected size is {}, but the payload len is {}", packet.header.ptype, expected_size - TelemetryHeader::SIZE, packet.payload.len());
            return Err(SerializationError::StreamTooShort);
          }
          match MergedEvent::from_bytestream(&packet.payload, &mut 0) {
            Err(err) => {
              error!("Unable to decode MergedEvent Telemetry packet of type {}! The expected size is {}, but the payload len is {}! {err}", packet.header.ptype, expected_size - TelemetryHeader::SIZE, packet.payload.len());
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

  pub fn render(&mut self, main_window: &Rect, frame: &mut Frame) {
    match self.view {
      TelemetryTabView::Stream => {
        // Layout first
        let main_lo = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [Constraint::Percentage(33),
                 Constraint::Percentage(33),
                 Constraint::Percentage(34)].as_ref(),
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
  
        // packet overview table, similar to home tab 
        let mut rows   = Vec::<Row>::new();
        let mut sum_pack = 0;
        let passed_time = self.start_time.elapsed().as_secs_f64();
        for k in self.pack_map.keys() {
          //stat_string_render += "  -- -- -- -- -- -- -- -- -- --\n";
          if self.pack_map[k] != 0 {
            sum_pack += self.pack_map[k];
            if k.contains("Heart"){
              rows.push(Row::new(vec![format!("  \u{1f493} {:.1}", self.pack_map[k]),
                                      format!("{:.1}", (self.pack_map[k] as f64)/passed_time,),
                                      format!("[{}]", k)]));
            } else {
              rows.push(Row::new(vec![format!("  \u{279f} {:.1}", self.pack_map[k]),
                                      format!("{:.1}", (self.pack_map[k] as f64)/passed_time,),
                                      format!("[{}]", k)]));
            }
          }
        }
        rows.push(Row::new(vec!["  \u{FE4C}\u{FE4C}\u{FE4C}","\u{FE4C}\u{FE4C}","\u{FE4C}\u{FE4C}\u{FE4C}\u{FE4C}\u{FE4C}\u{FE4C}"])); 
        rows.push(Row::new(vec![format!("  \u{279f}{}", sum_pack),
                           format!("{:.1}/s", (sum_pack as f64)/passed_time),
                           format!("[TOTAL]")]));
        
        let widths = [Constraint::Percentage(30),
                      Constraint::Percentage(20),
                      Constraint::Percentage(50)];
        let table  = Table::new(rows, widths)
          .column_spacing(1)
          .header(
            Row::new(vec!["  N", "\u{1f4e6}/s", "Type"])
            .bottom_margin(1)
            .top_margin(1)
            .style(Style::new().add_modifier(Modifier::UNDERLINED))
          )
          .block(Block::new()
                 .title("Telemetry Packet summary \u{1f4e6}")
                 .borders(Borders::ALL)
                 //.border_type(BorderType::Rounded)
                 )
          .style(self.theme.style());
        frame.render_widget(table, main_lo[2])
      }
      TelemetryTabView::MergedEvents => {
      }
    }
  }
}
