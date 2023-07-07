//! Read readoutboard binary files of the different levels
//!

#[macro_use] extern crate log;
extern crate pretty_env_logger;

extern crate json;
extern crate glob;
extern crate regex;

use glob::glob;
use regex::Regex;

use clap::Parser;
use std::io::BufReader;
use std::path::PathBuf;
use std::path::Path;
use std::io;
use std::io::Read;
use std::fs::File;
use std::collections::HashMap;

use tof_dataclasses::packets::PacketType;
use tof_dataclasses::events::{RBEvent,
                              MasterTriggerMapping,
                              MasterTriggerEvent};
use tof_dataclasses::manifest::{LocalTriggerBoard,
                                ReadoutBoard,
                                get_ltbs_from_sqlite,
                                get_rbs_from_sqlite};
use tof_dataclasses::serialization::Serialization;

use liftof_lib::{RobinReader,
                 TofPacketReader};


#[derive(Parser, Default, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// input folder with files to analyze (with RBEvent dataa)
  robin_data: String,
  /// Open a stream file to get access to TofPackets, e.g. MasterTriggerEvents
  /// or monitoring data
  #[arg(long)]
  stream: Option<PathBuf>,
  /// A json config file with detector information
  #[arg(short, long)]
  json_config: Option<PathBuf>,
}


fn main() {
  
  pretty_env_logger::init();

  let args = Args::parse();
  
  let json_content  : String;
  let config        : json::JsonValue;

  match args.json_config {
    None => panic!("No .json config file provided! Please provide a config file with --json-config or -j flag!"),
    Some(_) => {
      json_content = std::fs::read_to_string(args.json_config.as_ref().unwrap()).expect("Can not open json file");
      config = json::parse(&json_content).expect("Unable to parse json file");
    } // end Some
  } // end match
  let db_path               = Path::new(config["db_path"].as_str().unwrap());
  let db_path_c             = db_path.clone();
  let mut ltb_list          = get_ltbs_from_sqlite(db_path);
  let mut rb_list           = get_rbs_from_sqlite(db_path_c);
  let mapping = MasterTriggerMapping::new(ltb_list, rb_list);

  let mut packet_reader = TofPacketReader::default();
  let mut has_stream = false;
  if args.stream.is_some() {
    let stream_filename = args.stream.unwrap().into_os_string().into_string().expect("Not able to read input stream file!");
    packet_reader = TofPacketReader::new(stream_filename);
    has_stream = true;
  }
  //info!("Found start position in stream {}", start_pos);
  info!("Got input directory {}", args.robin_data);
  let mut robin_readers = HashMap::<u8, RobinReader>::new();
  let pattern = r#"/RB(\d{1,2})"#; 
  let re = Regex::new(pattern).unwrap();
  
  if let Ok(entries) = glob(&args.robin_data) { 
    for entry in entries {
      if let Ok(path) = entry {
        println!("Matched file: {:?}", path.display());
        let filename = path.into_os_string().into_string().unwrap();
        if let Some(mat) = re.captures(&filename) {
          let rb_id = mat.get(1).unwrap().as_str().parse::<u8>().unwrap();
          robin_readers.insert(rb_id, RobinReader::new(filename));
          //println!("First one or two-digit number: {}", mat.get(1).unwrap().as_str());
        } else {
          error!("Can not recognize pattern!!");
        }
      } else {
        println!("Error globbing files");
      }
    }
  }


  let mut reader = robin_readers.get_mut(&22).unwrap();
  //reader.print_index();
  let n_events = reader.count_packets();
  //let chunk = 1000usize;
  //reader.precache_events(chunk);

  let mut is_done = false;

  let mut r_events   = Vec::<RBEvent>::new();
  let mut seen_evids = Vec::<u32>::new(); 
  
  // FIXME - this can be optimized. For now, cache 
  // everything in memory
  reader.cache_all_events();
  println!("=> Cached {} events!", reader.get_cache_size());
  if has_stream {
    for packet in packet_reader {
      //println!("{}", packet);
      match packet.packet_type {
        PacketType::MasterTrigger =>  {
          //println!("{:?}", packet.payload);
          let mt_packet = MasterTriggerEvent::from_bytestream(&packet.payload, &mut 0); 
          let mut rb_event = RBEvent::new();
          if let Ok(mtp) = mt_packet {
            println!("MTE: rbids {:?}", mapping.get_rb_ids(&mtp));
            match reader.get_from_cache(&mtp.event_id) {
              None     => continue,
              Some(rbevent) => {
                r_events.push(rbevent);
                seen_evids.push(mtp.event_id);
              }
            }
            // this is if the reader has no double events
            //if reader.is_indexed(&mtp.event_id) {
            //  println!("--> Found {} in robin data!", mtp.event_id);
            //  let ev = reader.get_in_order(&mtp.event_id);
            //  match ev {
            //    None          => {
            //      error!("Can not get {}", mtp.event_id);
            //      if seen_evids.contains(&mtp.event_id) {
            //        reader.rewind();
            //        continue;
            //      } else {
            //        break;
            //      }
            //    }
            //    Some(rbevent) => {
            //      println!("{}", rbevent);
            //      r_events.push(rbevent);
            //      seen_evids.push(mtp.event_id);
            //    },
            //  }
            //}
          } else {
            error!("Error decoding MasterTriggerPacket!");
          }
        }
        _ => ()
      }
    }
  }
  println!("=> Extracted {} events from {} where we have corresponding MTB information", r_events.len(), reader.filename);
  for k in 0..r_events.len() {
    if r_events[k].header.channel_mask != 255 {
      println!("{}", r_events[k].header.channel_mask);
    }
  }
  //for event in reader {
  //  println!("{}", event);
  //}
}
