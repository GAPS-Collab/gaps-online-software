//! Read readoutboard binary files of the different levels
//!

#[macro_use] extern crate log;
extern crate pretty_env_logger;

extern crate json;
extern crate glob;
extern crate regex;
extern crate textplots;

use glob::glob;
use regex::Regex;
use textplots::{Chart, Plot, Shape};
use indicatif::{ProgressBar,
                ProgressStyle};


use clap::Parser;
use std::path::PathBuf;
use std::path::Path;
use std::collections::HashMap;

use std::process::exit;

use tof_dataclasses::packets::{PacketType,
                               TofPacket};
use tof_dataclasses::events::{TofEvent,
                              RBEvent,
                              MasterTriggerMapping,
                              MasterTriggerEvent};
use tof_dataclasses::manifest::{get_ltbs_from_sqlite,
                                get_rbs_from_sqlite};
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::calibrations::RBCalibrations;

use liftof_lib::{RobinReader,
                 TofPacketWriter,
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
  /// Readoutboard callibration textfiles
  #[arg(short, long)]
  calibrations: Option<PathBuf>,
  #[arg(long)]
  no_missing_hits : bool,
}


fn main() {
  
  pretty_env_logger::init();

  let args = Args::parse();
  
  let json_content  : String;
  let config        : json::JsonValue;

  let use_calibrations : bool;
  let mut calibrations = HashMap::<u8, RBCalibrations>::new();
  match args.calibrations {
    None => {
      use_calibrations = false;
    },
    Some(directory) => {
      use_calibrations = true;
      println!("=> Using calibrations from directory {}", directory.display());
      let mut calib_dir = directory;
      calib_dir.push("rb*_cal.txt");
      println!("=> Looklng for files with pattern {}", calib_dir.display());
      if let Ok(entries) = glob(calib_dir.to_str().unwrap()) { 
        for entry in entries {
          if let Ok(path) = entry {
            let rb_calib = RBCalibrations::from(path.as_path());            
            println!("=> Loaded RB calibration: {} from file {:?}", rb_calib, path.display());
            calibrations.insert(rb_calib.rb_id, rb_calib);
            //let filename = path.into_os_string().into_string().unwrap();
            //if let Some(mat) = re.captures(&filename) {
            //  let rb_id = mat.get(1).unwrap().as_str().parse::<u8>().unwrap();
            //  let mut this_reader = RobinReader::new(filename);
            //  this_reader.cache_all_events();
            //  robin_readers.insert(rb_id, this_reader);
            //  //println!("First one or two-digit number: {}", mat.get(1).unwrap().as_str());
            //  available_rbs.push(rb_id);
            //} else {
            //  error!("Can not recognize pattern!!");
            //}
          } else {
            println!("Error globbing files");
          }
        }
      } else {
        error!("No calibration files found in path!");
      }
    }
  }
  match args.json_config {
    None => panic!("No .json config file provided! Please provide a config file with --json-config or -j flag!"),
    Some(_) => {
      json_content = std::fs::read_to_string(args.json_config.as_ref().unwrap()).expect("Can not open json file");
      config = json::parse(&json_content).expect("Unable to parse json file");
    } // end Some
  } // end match
  let db_path               = Path::new(config["db_path"].as_str().unwrap());
  let db_path_c             = db_path.clone();
  let ltb_list              = get_ltbs_from_sqlite(db_path);
  let rb_list               = get_rbs_from_sqlite(db_path_c);
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
 
  let mut available_rbs = Vec::<u8>::new();
  if let Ok(entries) = glob(&args.robin_data) { 
    for entry in entries {
      if let Ok(path) = entry {
        println!("Matched file: {:?}", path.display());
        let filename = path.into_os_string().into_string().unwrap();
        if let Some(mat) = re.captures(&filename) {
          let rb_id = mat.get(1).unwrap().as_str().parse::<u8>().unwrap();
          let mut this_reader = RobinReader::new(filename);
          this_reader.cache_all_events();
          robin_readers.insert(rb_id, this_reader);
          //println!("First one or two-digit number: {}", mat.get(1).unwrap().as_str());
          available_rbs.push(rb_id);
        } else {
          error!("Can not recognize pattern!!");
        }
      } else {
        println!("Error globbing files");
      }
    }
  }

  //let mut reader = robin_readers.get_mut(&22).unwrap();
  //reader.cache_all_events();
  //reader.print_index();
  //let n_events = reader.count_packets();
  //let chunk = 1000usize;
  //reader.precache_events(chunk);
  // FIXME - this can be optimized. For now, cache 
  // everything in memory
  //println!("=> Cached {} events!", reader.get_cache_size());


  let r_events   = Vec::<RBEvent>::new();
  let mut seen_evids = Vec::<u32>::new(); 
  
  let mut ev_with_missing = 0;
  let mut mtp_events_tot  = 0;

  let mut writer = TofPacketWriter::new(String::from("combined"));

  if has_stream {
    for packet in packet_reader {
      //println!("{}", packet);
      match packet.packet_type {
        PacketType::MasterTrigger =>  {
          //println!("{:?}", packet.payload);
          let mt_packet = MasterTriggerEvent::from_bytestream(&packet.payload, &mut 0); 
          if let Ok(mtp) = mt_packet {
            if seen_evids.contains(&mtp.event_id) {
              continue;
            }
            let mut master_tof_event = TofEvent::new();
            let rb_ids_debug = mapping.get_rb_ids_debug(&mtp, false);
            mtp_events_tot += 1;
            if args.no_missing_hits {
              if rb_ids_debug.1.len() > 0 {
                ev_with_missing += 1;
                continue
              }
            }
            // we are done with the mtb_event and push it to the event
            master_tof_event.mt_event = mtp;

            println!("MTE: rbids {:?}", rb_ids_debug);
            println!("available_rbs {:?}", available_rbs);
            //println!("MTE: ltbids {:?}", mapping.get_ltb_ids(&mtp));
            for k in rb_ids_debug.0 {
              let this_ev_rbid = k.0;
              let this_ev_rbch = k.1;
              if !available_rbs.contains(&this_ev_rbid) {
                if this_ev_rbid != 0 {
                  println!("Requesting to read from RB {}, but we don't have data for that!", this_ev_rbid);
                  //panic!("Requesting to read from RB {}, but we don't have data for that!", this_ev_rbid);
                  continue;
                }
              }
              println!("Getting RB {}", this_ev_rbid);
              let reader = robin_readers.get_mut(&this_ev_rbid).unwrap();
              //panic!("{}", reader.get_cache_size());
              match reader.get_from_cache(&mtp.event_id) {
                None     => {
                  //reader.print_index();
                  //println!("Events: {:?}", reader.event_ids_in_cache());
                  //println!("Reader has {} events", reader.event_ids_in_cache().len());
                  error!("We do not have that event {}!", mtp.event_id);
                  continue;
                }
                Some(rbevent) => {
                  //println!("{:?}", rbevent.adc);
                  if !rbevent.is_over_adc_threshold(this_ev_rbch, 8000) {
                    continue;
                  }
                  if use_calibrations {
                    //let mut channel_data : [f32;1024] = [0.0;1024];
                    let mut channel_data = vec![0.0f32;1024];
                    let channel_adc = rbevent.get_adc_ch(this_ev_rbch);
                    //let mut channel_adc  : [u16;1024] = rbevent.get_adc_ch(this_ev_rbch).try_into().expect("Waveform does not have expected len of 1024!");
                    calibrations[&this_ev_rbid].voltages(this_ev_rbch as usize,
                                                         rbevent.header.stop_cell as usize,
                                                         &channel_adc,
                                                         &mut channel_data);
                    let mut data = Vec::<(f32, f32)>::with_capacity(1024);
                    for k in 0..800 {
                      data.push((k as f32, channel_data[k]));
                    }
                    //let data = [(0.0, 0.0), (1.0, 1.0), (2.0, 0.5), (3.0, 1.5)];
                    //Chart::new(240, 60, 0.0, 1024.0)
                    //  .lineplot(&Shape::Lines(data.as_slice())).display();
                  } else {
                    let channel_data = rbevent.get_adc_ch(this_ev_rbch);
                    //println!("ch {}", this_ev_rbch);
                    //println!("{:?}", channel_data);
                    //println!("{}", channel_data.len());
                    if channel_data.len() == 0 {
                      error!("There is no channel data for ch {}!", this_ev_rbch);
                    } else {
                      let mut data = Vec::<(f32, f32)>::with_capacity(1024);
                      if channel_data.len() < 1024 {
                        error!("Corrupt channel data!");
                        continue;
                      }
                      for k in 0..800 {
                        data.push((k as f32, channel_data[k] as f32));
                      }
                      //let data = [(0.0, 0.0), (1.0, 1.0), (2.0, 0.5), (3.0, 1.5)];
                      Chart::new(240, 60, 0.0, 1024.0)
                        .lineplot(&Shape::Lines(data.as_slice())).display();
                      //panic!("e basta!");
                    }
                  }
                  //r_events.push(rbevent);
                  master_tof_event.rb_events.push(rbevent);
                  //master_tof_event.missing_hits = rb_ids_debug.1;
                  for k in 0..rb_ids_debug.1.len() {
                    master_tof_event.missing_hits.push(rb_ids_debug.1[k]);
                  }
                  seen_evids.push(mtp.event_id);
                }
              }
            } 
            //match reader.get_from_cache(&mtp.event_id) {
            //  None     => continue,
            //  Some(rbevent) => {
            //    r_events.push(rbevent);
            //    seen_evids.push(mtp.event_id);
            //  }
            //}
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
            //let bss = master_tof_event.to_bytestream();
            println!("{}", master_tof_event);
            //panic!("Size {}", bss.len());
            
            let tof_packet = TofPacket::from(&master_tof_event);
            writer.add_tof_packet(&tof_packet);
            exit(0);
          } else {
            error!("Error decoding MasterTriggerPacket!");
          }
        }
        _ => ()
      }
    }
  } else {
    println!("=> Will merge events without master trigger information!");
    let mut alleventids = Vec::<u32>::new();
    //let mut tofevents   = Vec::<TofEvent>::new();
    for rb in &available_rbs {
      //println!("RB {}", rb);
      alleventids.extend(robin_readers.get(&rb).expect("We do not have a file for RB").event_ids_in_cache());
    }
    alleventids.sort();
    alleventids.dedup();
    let template_bar   : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.blue/grey} {human_pos:>7}/{human_len:7} ";
    let bar                   = ProgressBar::new(alleventids.len() as u64);
    let sty_bar               = ProgressStyle::with_template(template_bar).unwrap();
    bar.set_message("Merging events...");
    bar.set_prefix("");
    bar.set_style(sty_bar);
    let mut n_processed = 0u64;
    let mut n_tofpackets = 0u64;
    for ev in alleventids {
      let mut t_event   = TofEvent::new();
      let mut mt_event  = MasterTriggerEvent::default();
      mt_event.event_id = ev;
      t_event.mt_event = mt_event;
      for rb in &available_rbs {
        match robin_readers.get_mut(&rb).expect("We do not have a file for this RB").get_event_by_id(&ev) {
          None => (),
          Some(rbevent) => {
            t_event.rb_events.push(rbevent);
          }
        }
      }
      n_processed += 1;
      bar.set_position(n_processed);
      //tofevents.push(t_event);  

      let tof_packet = TofPacket::from(&t_event);
      writer.add_tof_packet(&tof_packet);
      n_tofpackets += 1;
      //if n_tofpackets == 10 {break;}
    }
    println!("=> We have written {} TofPackets!", n_tofpackets);
    bar.finish();
  }
  println!("=> In total, we saw {} events recorded by the MTB", mtp_events_tot);
  println!("=> Extracted {} events where we have corresponding MTB information", r_events.len());
  if args.no_missing_hits {
    println!("=> We did found {ev_with_missing} events which had missing hits, these events were discarded!");
  }

  let data = [(0.0, 0.0), (1.0, 1.0), (2.0, 0.5), (3.0, 1.5)];
  Chart::new(120, 40, 0.0, 3.0)
              .lineplot(&Shape::Lines(&data)).display();
  //plot.display();

  //println!("{:?}", plot);

  //for k in 0..r_events.len() {
  //  if r_events[k].header.channel_mask != 255 {
  //    println!("{}", r_events[k].header.channel_mask);
  //  }
  //}
  //for event in reader {
  //  println!("{}", event);
  //}
}
