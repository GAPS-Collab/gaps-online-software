//! Basic analysis of taken data. Waveform visualisation
//!
//!
//!
//!

use tof_dataclasses::serialization::Serialization;
use std::path::{PathBuf};

extern crate pretty_env_logger;
#[macro_use] extern crate log;
use clap::{arg,
           command,
           //value_parser,
           //ArgAction,
           //Command,
           Parser};

use tof_dataclasses::io::read_file;
use tof_dataclasses::events::RBEventMemoryView;

#[derive(Parser, Default, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// input file to analyze (with raw RBEvent dataa)
  raw_data: PathBuf,
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
  //let bytestream = read_file(&args.raw_data);
  let bytestream : Vec<u8>;
  if let Ok(bs) = read_file(&args.raw_data) {
    info!("Got input file {}", args.raw_data.display());
    bytestream = bs;
  } else {
    panic!("Unable to read file {}", args.raw_data.display());
  }
  let mut pos              = 0usize;
  let mut n_events_decoded = 0usize;
  let mut event = RBEventMemoryView::new();
  let mut n_errors = 0usize;
  let mut decoded_evids = Vec::<u32>::new();
  //let mut header = RBEventHeader::new();
  while pos + RBEventMemoryView::SIZE < bytestream.len() {
    //match event.from_bytestream(&bytestream, pos, false) {
    //  Err(_) => {
    //    n_errors += 1;
    //    pos += RBEventMemoryView::SERIALIZED_SIZE;
    //    continue;
    //  },
    //  Ok(_)  => ()
    //}
    //header = RBEventHeader::from_bytestream(&bytestream, pos).unwrap();
    match RBEventMemoryView::from_bytestream(&bytestream, &mut pos) {
      Err(err) => {
        error!("Unable to decode RBEventMemoryView! Err {err}");
        n_errors += 1;
      }
      Ok(ev) => {
        event = ev;
      }
    }
    n_events_decoded += 1;

    decoded_evids.push(event.event_id);
    //println!("{}",event.event_id);
    //println!("{pos}");

    //println!("{}",event);
    //println!("{}",event.head);
    //println!("{}",event.status);
    //println!("{}",event.len);
    //println!("{}",event.roi);
    //println!("{}",event.dna);
    //println!("{}",event.fw_hash);
    //println!("{}",event.id);
    //println!("{}",event.ch_mask);
    //println!("{}",event.dna);
    //println!("{pos} : pos");
    //pos += RBEventMemoryView::SERIALIZED_SIZE;
  }
  println!("{:?} decoded event ids", decoded_evids);
  println!("We decoded {n_events_decoded} and had {n_errors} corrupt events!");
} // end main

