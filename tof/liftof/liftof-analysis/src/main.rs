//! Basic analysis of taken data. Waveform visualisation
//!
//!
//!
//!

use std::path::PathBuf;
use tof_dataclasses::events::blob::RBEventHeader;
use tof_dataclasses::serialization::Serialization;

//use std::{thread, time};
//
//extern crate crossbeam_channel;
//use crossbeam_channel::{unbounded,
//                        Sender,
//                        Receiver};
//use local_ip_address::local_ip;
//
////use std::collections::HashMap;
//
//use liftof_rb::api::*;
//use liftof_rb::control::*;
//use liftof_rb::memory::{BlobBuffer,
//                    EVENT_SIZE,
//                    DATABUF_TOTAL_SIZE};
//
//use tof_dataclasses::threading::ThreadPool;
//use tof_dataclasses::packets::{TofPacket,
//                               PacketType};
//use tof_dataclasses::events::blob::RBEventPayload;
//use tof_dataclasses::commands::{TofCommand,
//                                TofResponse,
//                                TofOperationMode};
//use tof_dataclasses::commands as cmd;
//use tof_dataclasses::monitoring as moni;
////use liftof_lib::misc::*;
extern crate pretty_env_logger;
#[macro_use] extern crate log;
//
//use log::{info, LevelFilter};
////use std::io::Write;
//
//
//extern crate clap;
use clap::{arg,
           command,
           //value_parser,
           //ArgAction,
           //Command,
           Parser};

use liftof_lib::get_file_as_byte_vec;
use tof_dataclasses::events::blob::BlobData;

#[derive(Parser, Default, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// input file to analyze (with raw RBEvent dataa)
  raw_data: String,
  /// Show progress bars to indicate buffer fill values and number of acquired events
  #[arg(long, default_value_t = false)]
  show_progress: bool,
  /// Acquire this many events.
  /// If either --nevents or --run-forever options are given
  /// the board will not wait for a remote command, but start datataking as soon as 
  /// possible
  #[arg(short, long, default_value_t = 0)]
  nevents: u64,
  /// Cache size of the internal event cache in events
  #[arg(short, long, default_value_t = 10000)]
  cache_size: usize,
  /// If either --nevents or --run-forever options are given
  /// the board will not wait for a remote command, but start datataking as soon as 
  /// possible
  #[arg(long, default_value_t = false)]
  run_forever: bool,
  /// Activate the forced trigger. The value is the desired rate 
  #[arg(long, default_value_t = 0)]
  force_trigger: u32,
  /// Stream any eventy as soon as the software starts.
  /// Don't wait for command line.
  /// Behaviour can be controlled through `TofCommand` later
  #[arg(long, default_value_t = false)]
  stream_any : bool,
  /// Readoutboard testing with internal trigger
  #[arg(long, default_value_t = false)]
  rb_test_ext : bool,
  /// Readoutboard testing with softare trigger
  #[arg(long, default_value_t = false)]
  rb_test_sw : bool,
  ///// CnC server IP we should be listening to
  //#[arg(long, default_value_t = "10.0.1.1")]
  //cmd_server_ip : &'static str,
}

fn main() {

  //env_logger::Builder::new()
  //    .format(|buf, record| {
  //     writeln!(
  //     buf,
  //     "{}:{} {} [{}] - {}",
  //     record.file().unwrap_or("unknown"),
  //      record.line().unwrap_or(0),
  //     chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
  //          record.level(),
  //       record.args()
  //     )
  //                                })
  //.filter(Some("logger_example"), LevelFilter::Debug)
  //                        .init();
  pretty_env_logger::init();

  let kraken                = vec![240, 159, 144, 153];
  let fish                  = vec![240, 159, 144, 159];
  // We know these bytes are valid, so we'll use `unwrap()`.
  let kraken           = String::from_utf8(kraken).unwrap();
  let fish             = String::from_utf8(fish).unwrap();

  // welcome banner!
  println!("-----------------------------------------------");
  println!(" ** Welcome to liftof-analysis \u{1F680} \u{1F388} *****");
  println!(" .. liftof if a software suite for the time-of-flight detector ");
  println!(" .. for the GAPS experiment \u{1F496}");
  println!(" .. this part or the suite is meant to analyze offline data!");
  println!(" .. see the gitlab repository for documentation and submitting issues at" );
  println!(" **https://uhhepvcs.phys.hawaii.edu/Achim/gaps-online-software/-/tree/main/tof/liftof**");
  println!("-----------------------------------------------");

  // deal with command line arguments
  let args = Args::parse();
  info!("Got input file {}", args.raw_data);
  let bytestream = get_file_as_byte_vec(&args.raw_data);
  let mut pos              = 0usize;
  let mut n_events_decoded = 0usize;
  let mut event = BlobData::new();
  let mut n_errors = 0usize;
  let mut decoded_evids = Vec::<u32>::new();
  let mut header = RBEventHeader::new();
  while pos + BlobData::SERIALIZED_SIZE < bytestream.len() {
    //match event.from_bytestream(&bytestream, pos, false) {
    //  Err(_) => {
    //    n_errors += 1;
    //    pos += BlobData::SERIALIZED_SIZE;
    //    continue;
    //  },
    //  Ok(_)  => ()
    //}
    header = RBEventHeader::from_bytestream(&bytestream, pos).unwrap();
    pos = event.from_bytestream(&bytestream, pos, true);
    n_events_decoded += 1;
    decoded_evids.push(event.event_id);
    //println!("{}",event.event_id);
    //println!("{pos}");
    //pos += BlobData::SERIALIZED_SIZE;
  }
  //println!("{:?} decoded event ids", decoded_evids);
  println!("We decoded {n_events_decoded} and had {n_errors} corrupt events!");
} // end main

