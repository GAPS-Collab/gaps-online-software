//! Create RBCalibration in binary format from a bunch of 
//! files
//!
//! THIS IS FOR NTS DATA. 
//!
//! Hopefully can be deprecated soon, or changed in 
//! purpose

#[macro_use] extern crate log;

extern crate glob;
extern crate regex;

use glob::glob;
use regex::Regex;
use indicatif::{ProgressBar,
                ProgressStyle};


use clap::Parser;
use std::path::PathBuf;
use std::path::Path;
use std::collections::HashMap;

use std::process::exit;

use tof_dataclasses::packets::{PacketType,
                               TofPacket};
use tof_dataclasses::events::{RBEvent,
                              MasterTriggerEvent};
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::calibrations::RBCalibrations;

use liftof_lib::{RobinReader,
                 TofPacketWriter};


#[derive(Parser, Default, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// input folder with noi, vcal and tcal files
  cali_data: PathBuf,
}


fn main() {
  
  let args      = Args::parse();
  let noi_pattern   = String::from(".noi");
  let vcal_pattern  = String::from(".vcal");
  let tcal_pattern  = String::from(".tcal");
  let mut board_ids      = Vec::<u8>::new(); 
  let mut calibrations   = HashMap::<u8, RBCalibrations>::new();
  let board_events_noi   = HashMap::<u8, Vec<RBEvent>>::new();
  let board_events_tcal  = HashMap::<u8, Vec<RBEvent>>::new();
  let board_events_vcal  = HashMap::<u8, Vec<RBEvent>>::new();
  let rb_pattern = r#"rb(\d{1,2})"#; 
  let rb_regex   = Regex::new(rb_pattern).unwrap();
  let template_bar   : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.blue/grey} {human_pos:>7}/{human_len:7} ";
  let mut files = Vec::<String>::new();
  if let Ok(entries) = glob(&(args.cali_data.to_str().unwrap().to_owned() + "/*")){
    for entry in entries {
      let filename = entry.as_ref().unwrap().to_str().unwrap();
      files.push(filename.to_string());
    }
  } 
  let bar                   = ProgressBar::new(files.len() as u64);
  let sty_bar               = ProgressStyle::with_template(template_bar).unwrap();
  bar.set_message("Processing calibrations...");
  bar.set_prefix("");
  bar.set_style(sty_bar);
  bar.set_position(0);
  let mut n_processed = 0;
  for filename in files.iter() {
    bar.println("Checking ".to_owned() + filename + " ..");
    if let Some(mat) = rb_regex.captures(&filename) {
      let rb_id = mat.get(1).unwrap().as_str().parse::<u8>().unwrap();
      board_ids.push(rb_id);
      let mut reader = RobinReader::new((&filename).to_string());
      reader.cache_all_events();
      if !calibrations.contains_key(&rb_id) {
        calibrations.insert(rb_id, RBCalibrations::new(rb_id));
      }
      let mut cali   = calibrations.get_mut(&rb_id).unwrap();
      if filename.to_string().ends_with(".vcal") {
        cali.vcal_data = reader.get_events();
      } else if filename.to_string().ends_with(".tcal") {
        cali.tcal_data = reader.get_events();
      } else if filename.to_string().ends_with(".noi") {
        cali.noi_data = reader.get_events();
      } else {
        println!("=> Unable to identify file type of {}", filename);
      }
      //values().cloned().collect()
      n_processed += 1;
      bar.set_position(n_processed);
    } else {
      warn!("Can't process {}", filename);
      //panic!("=> Unable to find calibration files in {}", args.cali_data.display());
    }
    bar.finish();
  }
  // remove tripled board ids
  board_ids.dedup();
  for rb in board_ids.iter() {
    calibrations.get_mut(&rb).unwrap().calibrate();
    calibrations.get_mut(&rb).unwrap().serialize_event_data = true;
    //cali.calibrate();
    let tp   = TofPacket::from(&calibrations[&rb]);
    let cali_filename = format!("rb{:02}.cali", rb);
    let mut writer = TofPacketWriter::new(cali_filename);
    writer.add_tof_packet(&tp);
  }
}

