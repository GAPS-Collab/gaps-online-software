#[macro_use] extern crate log;
extern crate env_logger;
extern crate glob;

use std::io::Write;
use glob::glob;

use clap::Parser;

use liftof_lib::{
    color_log,
    TofPacketWriter,
    TofPacketReader
};


use tof_dataclasses::packets::{
    PacketType,
    TofPacket
};
use tof_dataclasses::events::{TofEvent,
                              RBEvent};
use tof_dataclasses::serialization::Serialization;


#[derive(Parser, Default, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// input folder with files to analyze (with RBEvent dataa)
  input_data: String,
}

fn main() {

  env_logger::builder()
    .format(|buf, record| {
    writeln!( buf, "[{level}][{module_path}:{line}] {args}",
      level = color_log(&record.level()),
      module_path = record.module_path().unwrap_or("<unknown>"),
      //target = record.target(),
      line = record.line().unwrap_or(0),
      args = record.args()
      )
    }).init();

  let args = Args::parse();
  let mut input_dir = args.input_data;
  input_dir += "*.tof.gaps";

  let mut events = Vec::<Vec<RBEvent>>::new();
  if let Ok(entries) = glob(input_dir.as_str()) { 
    for entry in entries.enumerate() {
      if let Ok(path) = entry.1 {
        events.push (Vec::<RBEvent>::new());
        println!("Found {}", path.display());
        let fname = path.into_os_string().into_string().expect("Not able to read input stream file!");
        let mut reader = TofPacketReader::new(fname);
        loop {
          match reader.next() {
            None => {
              break;
            },
            Some(tp) => {
              if tp.packet_type == PacketType::RBEvent {
                let event = RBEvent::from_bytestream(&tp.payload, &mut 0);
                match event {
                  Err(err) => error!("Can not unpack RBEvent! {err}"),
                  Ok(ev) => {
                    events[entry.0].push(ev);
                  }
                }
              }
            }
          }
        }
        println!("Found {} events for this file!", events[entry.0].len());
      }
    }
  }
  // here we have all our events
  let outfile    = String::from("merged.tof.gaps");
  let mut writer = TofPacketWriter::new(outfile);
  for ev in events[2].iter() {
    let mut tof_event = TofEvent::new();
    tof_event.rb_events.push(ev.clone());
    for ev0 in events[0].iter() {
      if ev0.header.event_id == ev.header.event_id {
        tof_event.rb_events.push(ev0.clone());
        break;
      }
    }
    for ev1 in events[1].iter() {
      if ev1.header.event_id == ev.header.event_id {
        tof_event.rb_events.push(ev1.clone());
        break;
      }
    }
    for ev3 in events[3].iter() {
      if ev3.header.event_id == ev.header.event_id {
        tof_event.rb_events.push(ev3.clone());
        break;
      }
    }
    let tp = TofPacket::from(&tof_event);
    writer.add_tof_packet(&tp);
  }


}
