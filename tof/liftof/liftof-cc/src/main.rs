
extern crate pretty_env_logger;
#[macro_use] extern crate log;

extern crate clap;
extern crate json;
#[cfg(feature = "diagnostics")]
extern crate hdf5;
#[cfg(feature = "diagnostics")]
extern crate ndarray;

extern crate local_ip_address;

extern crate liftof_lib;
use liftof_lib::{LocalTriggerBoard,
                 ReadoutBoard,
                 master_trigger,
                 get_tof_manifest};
                 //rb_manifest_from_json,
                 //get_rb_manifest};

#[cfg(feature="random")]
extern crate rand;

extern crate ctrlc;

extern crate zmq;

extern crate tof_dataclasses;

use std::{thread,
          time};

use clap::{arg,
           command,
           //value_parser,
           //ArgAction,
           //Command,
           Parser};

extern crate crossbeam_channel;
//use crossbeam_channel::{unbounded,
//                        Sender,
//                        Receiver};
use crossbeam_channel as cbc; 
use tof_dataclasses::events::MasterTriggerEvent;
//                            MasterTriggerEvent};
use tof_dataclasses::threading::ThreadPool;
use tof_dataclasses::packets::paddle_packet::PaddlePacket;
use tof_dataclasses::packets::TofPacket;

extern crate liftof_cc;

use liftof_cc::readoutboard_comm::readoutboard_communicator;
use liftof_cc::event_builder::{event_builder,
                           TofEventBuilderSettings};
                           //event_builder_no_master};
use liftof_cc::api::commander;
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
  /// A json config file with detector information
  #[arg(short, long)]
  json_config: Option<std::path::PathBuf>,
}



/*************************************/

fn main() {
  pretty_env_logger::init();

  // some bytes, in a vector
  let sparkle_heart         = vec![240, 159, 146, 150];
  //let kraken                = vec![240, 159, 144, 153];
  //let satelite_antenna      = vec![240, 159, 147, 161];

  // We know these bytes are valid, so we'll use `unwrap()`.
  let sparkle_heart    = String::from_utf8(sparkle_heart).unwrap();
  //let kraken           = String::from_utf8(kraken).unwrap();
  //let satelite_antenna = String::from_utf8(satelite_antenna).unwrap();
  // welcome banner!
  //
  println!("-----------------------------------------------");
  println!(" ** Welcome to liftof-cc \u{1F680} \u{1F388} *****");
  println!(" .. liftof if a software suite for the time-of-flight detector ");
  println!(" .. for the GAPS experiment {}", sparkle_heart);
  println!(" .. This is the Command&Control server which connects to the MasterTriggerBoard and the ReadoutBoards");
  println!(" .. see the gitlab repository for documentation and submitting issues at" );
  println!(" **https://uhhepvcs.phys.hawaii.edu/Achim/gaps-online-software/-/tree/main/tof/liftof**");

  ctrlc::set_handler(move || {
    println!("received Ctrl+C! We will stop triggers and end the run!");
    println!("So long and thanks for all the \u{1F41F}"); 
    exit(0);
  })
  .expect("Error setting Ctrl-C handler");



  // deal with command line arguments
  let args = Args::parse();
 

  let write_blob = args.write_blob;
  if write_blob {
    info!("Will write blob data to file!");
  }
 
  let write_stream = args.write_stream;
  if write_stream {
    info!("Will write the entire stream to files");
  }
  let json_content  : String;
  let config        : json::JsonValue;
  
  let nboards       : usize;

  let use_master_trigger      = args.use_master_trigger;
  let mut master_trigger_ip   = String::from("");
  let mut master_trigger_port = 0usize;

  // Have all the readoutboard related information in this list
  let rb_list      : Vec::<ReadoutBoard>;
  let manifest : (Vec::<LocalTriggerBoard>, Vec::<ReadoutBoard>);
  match args.json_config {
    None => panic!("No .json config file provided! Please provide a config file with --json-config or -j flag!"),
    Some(_) => {
      //if !args.json_config.as_ref().unwrap().exists() {
      //    panic!("The file {} does not exist!", args.json_config.as_ref().unwrap().display());
      //}
      //info!("Found config file {}", args.json_config.as_ref().unwrap().display());
      json_content = std::fs::read_to_string(args.json_config.as_ref().unwrap()).unwrap();
      config = json::parse(&json_content).unwrap();
      manifest = get_tof_manifest(args.json_config.unwrap());
      println!("==> Tof Manifest following:");
      println!("{:?}", manifest);
      println!("***************************");
      rb_list = manifest.1;
      nboards = rb_list.len();
      //panic!("That's it");
      //println!(" .. .. using config:");
      //println!("  {}", config.pretty(2));
      //nboards = config["readout_boards"].len();
      //info!("Found configuration for {} readout boards!", nboards);
      //for n in 0..config["readout_boards"].len() {
      //   println!(" {}", config["readout_boards"][n].pretty(2));
    } // end Some
  } // end match
 
  //if autodiscover_rbs {
  //  println!("==> Autodiscovering ReadoutBoards!...");
  //  rb_list = get_rb_manifest();
  //  nboards = rb_list.len();
  //}
  //for rb in rb_list.iter() {
  //  println!("{}", rb);
  //}

  if use_master_trigger {
   master_trigger_ip   = config["master_trigger"]["ip"].as_str().unwrap().to_owned();
   master_trigger_port = config["master_trigger"]["port"].as_usize().unwrap();
   info!("Will connect to the master trigger board at {}:{}", master_trigger_ip, master_trigger_port);
  }

  let storage_savepath = config["raw_storage_savepath"].as_str().unwrap().to_owned();
  let events_per_file  = config["events_per_file"].as_usize().unwrap(); 
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
  let mut nthreads = 20;
  if use_master_trigger { 
    nthreads += 1;
  }

  let worker_threads = ThreadPool::new(nthreads);
 
  println!("==> Starting paddle cache trhead...");
  worker_threads.execute(move || {
                         paddle_packet_cache(&id_rec,
                                             &rb_rec,
                                             &pp_send);
  });
  
  worker_threads.execute(move || {
                         global_data_sink(&tp_from_client,
                                          write_stream);
  });

  // open a zmq context
  println!("==> Seting up zmq context and opening socket for event builder!");
  //let ctx = zmq::Context::new();
  //let evb_comm_socket = ctx.socket(zmq::PUB).unwrap();
  //let mut address_ip = String::from("tcp://");
  //address_ip += config["flight_computer"]["ip_address"].as_str().unwrap();
  //port        = config["flight_computer"]["port"].as_usize().unwrap();
  //address = address_ip.to_owned() + ":" + &port.to_string();
  //info!("Will bind to port for flight comp comm at {}", address);
  //let evb_comm_ok = evb_comm_socket.bind(&address);
  //match evb_comm_ok {
  //    Ok(_)    => info!("Bound socket to {}", address),
  //    Err(err) => panic!("Can not communicate with rb at address {}. Maybe you want to check your .json configuration file?, error {}",address, err)
  //}

  println!("==> Starting event builder and master trigger threads...");
  let tp_to_sink_c = tp_to_sink.clone();
  if use_master_trigger {
    // start the event builder thread
    worker_threads.execute(move || {
                           event_builder(&master_ev_rec,
                                         &id_send,
                                         &pp_rec,
                                         &ebs_from_cmdr,
                                         &tp_to_sink);
                                         //&evb_comm_socket);
    });
    // master trigger
    worker_threads.execute(move || {
                           master_trigger(&master_trigger_ip, 
                                          master_trigger_port,
                                          &tp_to_sink_c,
                                          &rate_to_main,
                                          &master_ev_send,
                                          true);
    });
  } else {
    // we start the event builder without 
    // depending on the master trigger
    //worker_threads.execute(move || {
    //                       event_builder_no_master(&id_send,
    //                                               &pp_rec,
    //                                               &evb_comm_socket);
    //});
  }
  println!("==> Will now start rb threads..");

  for n in 0..nboards {
    let this_rb_pp_sender = rb_send.clone();
    let this_rb = rb_list[n].clone();
    let this_path = storage_savepath.clone();
    worker_threads.execute(move || {
      readoutboard_communicator(this_rb_pp_sender,
                                write_blob,
                                &this_path,
                                &events_per_file,
                                &this_rb);
    });
  } // end for loop over nboards
  // lastly start the commander thread 
  // wait a bit before, so the boards have
  // time to come up
  let one_second = time::Duration::from_millis(1000);
  thread::sleep(20*one_second);
  worker_threads.execute(move || {
    commander(&rb_list);
  });
  info!("All threads started!");
  let one_minute = time::Duration::from_millis(60000);
  
  //println!("==> Sleeping a bit to give the rb's a chance to fire up..");
  //thread::sleep(10*one_second);
  loop{
    thread::sleep(1*one_minute);
    println!("...");
  }
  //println!("Program terminating after specified runtime! So long and thanks for all the {}", fish); 
}
