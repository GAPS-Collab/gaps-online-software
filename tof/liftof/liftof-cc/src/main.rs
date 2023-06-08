//! LIFTOF-CC - Main C&C (command and control) server application for 
//! tof datataking and control.
//!
//!
//!
//!

extern crate pretty_env_logger;
#[macro_use] extern crate log;

extern crate clap;
extern crate json;
#[cfg(feature = "diagnostics")]
extern crate hdf5;
#[cfg(feature = "diagnostics")]
extern crate ndarray;
extern crate ctrlc;
extern crate zmq;
extern crate tof_dataclasses;
extern crate local_ip_address;
extern crate crossbeam_channel;

extern crate colored;
extern crate liftof_lib;
extern crate liftof_cc;

use log::{LevelFilter, Level};
use pretty_env_logger::formatted_builder;
use colored::{Colorize, ColoredString};
use std::io::Write;
use std::{thread,
          time};

use std::path::Path;

use clap::{arg,
           command,
           //value_parser,
           //ArgAction,
           //Command,
           Parser};

//use crossbeam_channel::{unbounded,
//                        Sender,
//                        Receiver};
use crossbeam_channel as cbc; 
use tof_dataclasses::events::{MasterTriggerEvent,
                              MasterTriggerMapping};
use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::packets::paddle_packet::PaddlePacket;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::manifest::{LocalTriggerBoard,
                                ReadoutBoard,
                                get_ltbs_from_sqlite,
                                get_rbs_from_sqlite};
use tof_dataclasses::commands::{TofCommand, TofResponse};
use liftof_lib::master_trigger;

use liftof_cc::readoutboard_comm::readoutboard_communicator;
use liftof_cc::event_builder::{event_builder,
                           TofEventBuilderSettings};
                           //event_builder_no_master};
use liftof_cc::api::{commander,
                     tofcmp_and_mtb_moni};
use liftof_cc::paddle_packet_cache::paddle_packet_cache;
use liftof_cc::flight_comms::global_data_sink;

use std::process::exit;

/*************************************/

#[derive(Parser, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// Write the raw data from the readoutboards,
  /// one file per readoutboard
  #[arg(short, long, default_value_t = false)]
  write_blob: bool,
  /// Dump the entire TofPacket Stream to a file
  #[arg(long, default_value_t = false)]
  write_stream: bool,
  #[arg(short, long, default_value_t = false)]
  use_master_trigger: bool,
  /// Disable monitoring features
  #[arg(short, long, default_value_t = false)]
  no_monitoring: bool,
  /// Enhance output to console
  #[arg(short, long, default_value_t = false)]
  verbose: bool,
  /// A json config file with detector information
  #[arg(short, long)]
  json_config: Option<std::path::PathBuf>,
}

/*************************************/

fn color_log(level : &Level) -> ColoredString {
  match level {
    Error    => String::from("ERROR!").red(),
    Warn     => String::from("WARN").yellow(),
    Info     => String::from("Info").green(),
    Debug    => String::from("debug").blue(),
    Trace    => String::from("trace").cyan(),
  }
}

/*************************************/

fn main() {
  //formatted_builder()
  //  .format(|buf, record| {
  //  writeln!( buf, "[{level}][{module_path}:{line}] {args}",
  //    level = color_log(&record.level()),
  //    module_path = record.module_path().unwrap_or("<unknown>"),
  //    line = record.line().unwrap_or(0),
  //    args = record.args()
  //    )
  //  }).filter(None, LevelFilter::Error)
  //  .format(|buf, record| {
  //  writeln!( buf, "[{level}][{module_path}:{line}] {args}",
  //    level = color_log(&record.level()),
  //    module_path = record.module_path().unwrap_or("<unknown>"),
  //    line = record.line().unwrap_or(0),
  //    args = record.args()
  //    )
  //  }).filter(None, LevelFilter::Warn)
  //  .format(|buf, record| {
  //  writeln!( buf, "[{level}][{module_path}:{line}] {args}",
  //    level = color_log(&record.level()),
  //    module_path = record.module_path().unwrap_or("<unknown>"),
  //    line = record.line().unwrap_or(0),
  //    args = record.args()
  //    )
  //  }).filter(None, LevelFilter::Info)
  //  .format(|buf, record| {
  //  writeln!( buf, "[{level}][{module_path}:{line}] {args}",
  //    level = color_log(&record.level()),
  //    module_path = record.module_path().unwrap_or("<unknown>"),
  //    line = record.line().unwrap_or(0),
  //    args = record.args()
  //    )
  //  }).filter(None, LevelFilter::Debug)
  //  .format(|buf, record| {
  //  writeln!( buf, "[{level}][{module_path}:{line}] {args}",
  //    level = color_log(&record.level()),
  //    module_path = record.module_path().unwrap_or("<unknown>"),
  //    line = record.line().unwrap_or(0),
  //    args = record.args()
  //    )
  //  }).filter(None, LevelFilter::Trace).init();
  pretty_env_logger::init();

  // welcome banner!
  println!("-----------------------------------------------");
  println!(" ** Welcome to liftof-cc \u{1F680} \u{1F388} *****");
  println!(" .. liftof if a software suite for the time-of-flight detector ");
  println!(" .. for the GAPS experiment \u{1F496}");
  println!(" .. This is the Command&Control server which connects to the MasterTriggerBoard and the ReadoutBoards");
  println!(" .. see the gitlab repository for documentation and submitting issues at" );
  println!(" **https://uhhepvcs.phys.hawaii.edu/Achim/gaps-online-software/-/tree/main/tof/liftof**");


  // deal with command line arguments
  let args = Args::parse();

  let verbose = args.verbose;

  let write_blob = args.write_blob;
  if write_blob {
    info!("Will write blob data to file!");
  }
 
  let write_stream = args.write_stream;
  if write_stream {
    info!("Will write the entire stream to files");
  }
  let no_monitoring = args.no_monitoring;
  if no_monitoring {
    warn!("All monitoring features disabled!");
  }

  let json_content  : String;
  let config        : json::JsonValue;
  
  let nboards       : usize;

  let use_master_trigger        = args.use_master_trigger;
  let mut master_trigger_ip     = String::from("");
  let mut master_trigger_port   = 0usize;
  // create copies, since we need this information
  // for 2 threads at least (moni and event)
  let mut master_trigger_ip_c   = String::from("");
  let mut master_trigger_port_c = 0usize;
  
  match args.json_config {
    None => panic!("No .json config file provided! Please provide a config file with --json-config or -j flag!"),
    Some(_) => {
      //if !args.json_config.as_ref().unwrap().exists() {
      //    panic!("The file {} does not exist!", args.json_config.as_ref().unwrap().display());
      //}
      //info!("Found config file {}", args.json_config.as_ref().unwrap().display());
      json_content = std::fs::read_to_string(args.json_config.as_ref().unwrap()).expect("Can not open json file");
      config = json::parse(&json_content).expect("Unable to parse json file");
    } // end Some
  } // end match
 

  if use_master_trigger {
    master_trigger_ip   = config["master_trigger"]["ip"].as_str().unwrap().to_owned();
    master_trigger_port = config["master_trigger"]["port"].as_usize().unwrap();
    master_trigger_ip_c = master_trigger_ip.clone();
    master_trigger_port_c = master_trigger_port.clone();
    info!("Will connect to the master trigger board at {}:{}", master_trigger_ip, master_trigger_port);
  } else {
    println!("==> Will NOT connect to the MTB, since -u has not been provided in the commandlline!");
  }

  let storage_savepath = config["raw_storage_savepath"].as_str().unwrap().to_owned();
  let events_per_file  = config["events_per_file"].as_usize().unwrap(); 
  let calib_file_path  = config["calibration_file_path"].as_str().unwrap().to_owned();
  let db_path          = Path::new(config["db_path"].as_str().unwrap());
  let db_path_c        = db_path.clone();
  let ltb_list = get_ltbs_from_sqlite(db_path);

  let rb_ignorelist =  &config["rb_ignorelist"];
  //exit(0);
  let mut rb_list  = get_rbs_from_sqlite(db_path_c);
  for k in 0..rb_ignorelist.len() {
    println!("=> We will remove RB {} due to it being marked as IGNORE in the config file!", rb_ignorelist[k]);
    let bad_rb = rb_ignorelist[k].as_u8().unwrap();
    rb_list.retain(|x| x.rb_id != bad_rb);
  }
  nboards = rb_list.len();
  println!("=> We will use the following tof manifest:");
  println!("== ==> LTBs [{}]:", ltb_list.len());
  for ltb in &ltb_list {
    println!("\t {}", ltb);
  }
  println!("== ==> RBs [{}]:", rb_list.len());
  for rb in &rb_list {
    println!("\t {}", rb);
  }
  //let matches = command!() // requires `cargo` feature
  //     //.arg(arg!([name] "Optional name to operate on"))
  //     .arg(
  //         arg!(
  //             -c --json-config <FILE> "Sets a custom config file"
  //         )
  //         // We don't have syntax yet for optional options, so manually calling `required`
  //         .required(true)
  //         .value_parser(value_parser!(std::path::PathBuf)),
  //     )
  //     .arg(arg!(
  //         -d --debug ... "Turn debugging information on"
  //     ))
  //     //.subcommand(
  //     //    Command::new("test")
  //     //        .about("does testing things")
  //     //        .arg(arg!(-l --list "lists test values").action(ArgAction::SetTrue)),
  //     //)
  //     .get_matches();

  // // You can check the value provided by positional arguments, or option arguments
  // //if let Some(name) = matches.get_one::<String>("name") {
  // //    println!("Value for name: {}", name);
  // //}

  // if let Some(config_path) = matches.get_one::<std::path::PathBuf>("json-config") {
  //     println!("Value for config: {}", config_path.display());
  // }

  // // You can see how many times a particular flag or argument occurred
  // // Note, only flags can have multiple occurrences
  // match matches
  //     .get_one::<u8>("debug")
  //     .expect("Count's are defaulted")
  // {
  //     0 => println!("Debug mode is off"),
  //     1 => println!("Debug mode is kind of on"),
  //     2 => println!("Debug mode is on"),
  //     _ => println!("Don't be crazy"),
  // }

  //// You can check for the existence of subcommands, and if found use their
  //// matches just as you would the top level cmd
  //if let Some(matches) = matches.subcommand_matches("test") {
  //    // "$ myapp test" was run
  //    if *matches.get_one::<bool>("list").expect("defaulted by clap") {
  //        // "$ myapp test -l" was run
  //        println!("Printing testing lists...");
  //    } else {
  //        println!("Not printing testing lists...");
  //    }
  //}

  println!(" .. .. .. .. .. .. .. ..");
  

  // prepare channels for inter thread communications
 
  let (tp_to_sink, tp_from_client) : (cbc::Sender<TofPacket>, cbc::Receiver<TofPacket>) = cbc::unbounded();

  // set the parameters for the event builder
  let (ebs_to_eb, ebs_from_cmdr)   : (cbc::Sender<TofEventBuilderSettings>,cbc::Receiver<TofEventBuilderSettings>) = cbc::unbounded();

  // send the rate from the master trigger to the main thread
  let (rate_to_main, rate_from_mt) : (cbc::Sender<u32>, cbc::Receiver<u32>) = cbc::unbounded();
  // master thread -> event builder ocmmuncations
  let (master_ev_send, master_ev_rec): (cbc::Sender<MasterTriggerEvent>, cbc::Receiver<MasterTriggerEvent>) = cbc::unbounded(); 
  // event builder  <-> paddle cache communications
  let (pp_send, pp_rec) : (cbc::Sender<Option<PaddlePacket>>, cbc::Receiver<Option<PaddlePacket>>) = cbc::unbounded(); 
  // readout boards <-> paddle cache communications 
  let (rb_send, rb_rec) : (cbc::Sender<PaddlePacket>, cbc::Receiver<PaddlePacket>) = cbc::unbounded();
  // paddle cache <-> event builder communications
  let (id_send, id_rec) : (cbc::Sender<Option<u32>>, cbc::Receiver<Option<u32>>) = cbc::unbounded();
  let (cmd_sender, cmd_receiver) : (cbc::Sender<TofCommand>, cbc::Receiver<TofCommand>) = cbc::unbounded();

  let (resp_sender, resp_receiver) : (cbc::Sender<TofResponse>, cbc::Receiver<TofResponse>) = cbc::unbounded();

  // prepare a thread pool. Currently we have
  // 1 thread per rb, 1 master trigger thread
  // and 1 event builder thread.
  // Also, the paddle cache is its separate 
  // thread.
  // There might
  // be a monitoring thread, too.
  // The number of threads should be fixed at 
  // runtime, but it should be possible to 
  // respawn them
  //let mut nthreads = nboards + 2; // 
  let mut nthreads = 60;
  if use_master_trigger { 
    nthreads += 1;
  }

  let worker_threads = ThreadPool::new(nthreads);

  if !no_monitoring {
    println!("==> Starting main monitoring thread...");
    let tp_to_sink_c = tp_to_sink.clone();
    let moni_interval = 10u64; // in seconds
    worker_threads.execute(move || {
                           tofcmp_and_mtb_moni(&tp_to_sink_c,
                                               &master_trigger_ip_c,
                                               master_trigger_port_c,
                                               moni_interval,
                                               false);
    });
  }

  println!("==> Starting paddle cache thread...");
  worker_threads.execute(move || {
                         paddle_packet_cache(&id_rec,
                                             &rb_rec,
                                             &pp_send);
  });
  println!("==> paddle cache thread started!");
  println!("==> Starting data sink thread!");
  worker_threads.execute(move || {
                         global_data_sink(&tp_from_client,
                                          write_stream,
                                          verbose);
  });
  println!("==> data sink thread started!");
  println!("==> Will now start rb threads..");

  for n in 0..nboards {
    let this_rb_pp_sender = rb_send.clone();
    let mut this_rb = rb_list[n].clone();
    let this_tp_to_sink_clone = tp_to_sink.clone();
    this_rb.infer_ip_address();
    this_rb.calib_file = calib_file_path.clone() + "/" + "rb";
    if this_rb.rb_id < 10 {
      this_rb.calib_file += "0";
    }
    this_rb.calib_file += &(this_rb.rb_id).to_string();
    this_rb.calib_file += "_cal.txt";
    println!("==> Starting RB thread for {}", this_rb);
    let resp_sender_c = resp_sender.clone();
    let this_path = storage_savepath.clone();
    worker_threads.execute(move || {
      readoutboard_communicator(this_rb_pp_sender,
                                resp_sender_c,
                                this_tp_to_sink_clone,
                                write_blob,
                                &this_path,
                                &events_per_file,
                                &this_rb,
                                verbose);
    });
  } // end for loop over nboards
  println!("==> All RB threads started!");
  
  let one_second = time::Duration::from_millis(1000);
  let rb_list_cc = rb_list.clone();
  worker_threads.execute(move || {
    commander(&rb_list_cc,
              cmd_receiver);
  });
  if use_master_trigger {
    // start the event builder thread
    println!("==> Starting event builder and master trigger threads...");
    let cmd_sender_c = cmd_sender.clone();
    let rb_list_c    = rb_list.clone();
    let ltb_list_c   = ltb_list.clone();
    let mapping = MasterTriggerMapping::new(ltb_list_c, rb_list_c);
    println!("{:?}", mapping.ltb_mapping);
    //exit(0);
    worker_threads.execute(move || {
                           event_builder(&master_ev_rec,
                                         mapping,
                                         &id_send,
                                         &pp_rec,
                                         &ebs_from_cmdr,
                                         &tp_to_sink,
                                         &cmd_sender_c);
                                         //&evb_comm_socket);
    });
    // master trigger
    worker_threads.execute(move || {
                           master_trigger(&master_trigger_ip, 
                                          master_trigger_port,
                                          &rate_to_main,
                                          &master_ev_send,
                                          true);
    });
  } else {
    println!("=> {}", "NOT using the MTB! This means that currently we can only save the blobfiles directly and NO EVENT data will be passed on to the flight computer!".red().bold());
    println!("=> {}", "This mode is still useful for calibration runs or to save RBBinary data locally!".italic());
  }

  let one_minute = time::Duration::from_millis(60000);
  
  println!("==> Sleeping 10 seconds to give the rb's a chance to fire up..");
  thread::sleep(10*one_second);
  println!("==> Sleeping done!");

  // set the handler for SIGINT
  let cmd_sender_c = cmd_sender.clone();
  ctrlc::set_handler(move || {
    println!("==> \u{1F6D1} received Ctrl+C! We will stop triggers and end the run!");
    let end_run = TofCommand::DataRunEnd(42);
    match cmd_sender_c.send(end_run) {
     Err(err) => error!("Can not send end run command! {err}"),
     Ok(_)    => ()
    }
    thread::sleep(one_second);
    println!("So long and thanks for all the \u{1F41F}"); 
    exit(0);
  })
  .expect("Error setting Ctrl-C handler");
  
  // start a new data run 
  let start_run = TofCommand::DataRunStart(1000);
  match cmd_sender.send(start_run) {
    Err(err) => error!("Unable to send command, error{err}"),
    Ok(_)    => ()
  }

  println!("==> All threads initialized!");
  loop{
    // first we issue start commands until we receive
    // at least 1 positive
    //cmd_sender.send(start_run);
    thread::sleep(1*one_second); 
    thread::sleep(1*one_minute);
    println!("...");
  }
  //println!("Program terminating after specified runtime! So long and thanks for all the {}", fish); 
}
