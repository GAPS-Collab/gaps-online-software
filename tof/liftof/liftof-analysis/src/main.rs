//! Basic analysis of taken data. Waveform visualisation
//!
//! WIP - currently this is in limbo!
//!
//!

use std::path::{PathBuf};

extern crate pretty_env_logger;
use clap::{arg,
           command,
           //value_parser,
           //ArgAction,
           //Command,
           Parser};


#[derive(Parser, Default, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// input file to analyze (with raw RBEvent dataa)
  raw_data: PathBuf,
  /// Show progress bars to indicate buffer fill values and number of acquired events
  #[arg(long, default_value_t = false)]
  show_progress: bool,
  /// Restrict the number of events we are running over
  #[arg(short, long, default_value_t = 0)]
  nevents: u64,
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
  let _args = Args::parse();
} // end main

