use std::path::Path;
use tof_dataclasses::io::read_file;
use tof_dataclasses::serialization::{
  search_for_u16,
  Serialization
};
use crate::packets::{
  TelemetryHeader,
  MergedEvent,
  TrackerPacket,
  GapsEvent,
};
use tof_dataclasses::packets::{
  TofPacket,
  PacketType,
};
use tof_dataclasses::events::TofEventSummary;


/// Extract all merged events from a file and ignore all others
pub fn get_gaps_events(filename : String) -> Vec<GapsEvent> {
  let mut events = Vec::<GapsEvent>::new();
  let stream = read_file(Path::new(&filename)).expect("Unable to open input file!");
  let mut pos : usize = 0;
  //let mut npackets : usize = 0;
  let mut packet_types = Vec::<u8>::new();
  loop {
    match TelemetryHeader::from_bytestream(&stream, &mut pos) {
      Err(err) => {
        println!("Can not decode telemtry header! {err}");
        //for k in pos - 5 .. pos + 5 {
        //  println!("{}",stream[k]);
        //}
        match search_for_u16(0x90eb, &stream, pos) {
          Err(err) => {
            println!("Unable to find next header! {err}");
            break;
          }
          Ok(head_pos) => {
            pos = head_pos;
          }
        }
      }
      Ok(header) => {
        println!("HEADER {}", header);
        //for k in pos - 10 .. pos + 10 {
        //  println!("{}",stream[k]);
        //}
        if header.ptype == 80 {
          match TrackerPacket::from_bytestream(&stream, &mut pos) {
            Err(err) => {
              //for k in pos - 5 .. pos + 5 {
              //  println!("{}",stream[k]);
              //}
              println!("Unable to decode TrackerPacket! {err}");
            }
            Ok(mut tp) => {
              tp.telemetry_header = header;
              println!("{}", tp);
            }
          }
        }
        if header.ptype == 90 {
          match MergedEvent::from_bytestream(&stream, &mut pos) {
            Err(err) => {
              println!("Unable to decode MergedEvent! {err}");
            }
            Ok(mut me) => {
              me.header  = header;
              let mut g_event = GapsEvent::new();
              //println!("Event ID  : {}", me.event_id);
              //println!("Tof bytes : {:?}", me.tof_data);
              //println!("len tof bytes : {}", me.tof_data.len());
              match TofPacket::from_bytestream(&me.tof_data, &mut 0) {
                Err(err) => {
                  println!("Can't unpack TofPacket! {err}");
                }
                Ok(tp) => {
                  println!("{}", tp);
                  if tp.packet_type == PacketType::TofEventSummary {
                    match TofEventSummary::from_tofpacket(&tp) {
                      Err(err) => println!("Can't unpack TofEventSummary! {err}"),
                      Ok(ts) => {
                        println!("{}", ts);
                        g_event.tof = ts;
                      }
                    }
                  }
                }
              }
              g_event.tracker = me.tracker_events;
              events.push(g_event)
            }
          }
        }
        //npackets += 1;
        packet_types.push(header.ptype);
        match search_for_u16(0x90eb, &stream, pos) {
          Err(err) => {
            println!("Unable to find next header! {err}");
            break;
          }
          Ok(head_pos) => {
            pos = head_pos;
          }
        }
      }
    }
  }
  events
}
