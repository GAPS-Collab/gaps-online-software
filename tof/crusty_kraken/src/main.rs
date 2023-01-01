mod readoutboard_comm;
mod reduced_tofevent;
mod constants;
mod waveform;
mod errors;
mod commands;
mod master_trigger;
mod monitoring;
mod event_builder;
mod paddle_packet_cache;

// this is a list of tests
// FIXME - this should follow
// the "official" structure
// for now, let's just keep it here
mod test_blobdata;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

extern crate clap;
extern crate json;
#[cfg(feature = "diagnostics")]
extern crate hdf5;
#[cfg(feature = "diagnostics")]
extern crate ndarray;

#[cfg(feature="random")]
extern crate rand;

extern crate tof_dataclasses;

use std::{thread,
          time,
          path::Path,
          sync::mpsc::Sender,
          sync::mpsc::Receiver,
          sync::mpsc::channel};

use clap::{arg,
           command,
           //value_parser,
           //ArgAction,
           //Command,
           Parser};

use crate::readoutboard_comm::readoutboard_communicator;
use crate::master_trigger::{master_trigger,
                            MasterTriggerEvent};
use crate::event_builder::{event_builder,
                           event_builder_no_master};
use tof_dataclasses::threading::ThreadPool;
//use crate::reduced_tofevent::PaddlePacket;

use tof_dataclasses::packets::paddle_packet::PaddlePacket;
use crate::paddle_packet_cache::paddle_packet_cache;

/*************************************/

#[derive(Parser, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// Increase output for debugging
  #[arg(short, long, default_value_t = false)]
  debug: bool,
  #[arg(short, long, default_value_t = false)]
  write_blob: bool,
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
  let kraken                = vec![240, 159, 144, 153];
  //let satelite_antenna      = vec![240, 159, 147, 161];
  let fish                  = vec![240, 159, 144, 159];

  // We know these bytes are valid, so we'll use `unwrap()`.
  let sparkle_heart    = String::from_utf8(sparkle_heart).unwrap();
  let kraken           = String::from_utf8(kraken).unwrap();
  //let satelite_antenna = String::from_utf8(satelite_antenna).unwrap();
  let fish             = String::from_utf8(fish).unwrap();
  // welcome banner!
  //
  println!("-----------------------------------------------");
  println!(" ** Welcome to crusty_kraken {} *****", kraken);
  println!(" .. TOF C&C and data acquistion suite");
  println!(" .. for the GAPS experiment {}", sparkle_heart);
  println!("-----------------------------------------------");
  println!("");

  // deal with command line arguments
  let args = Args::parse();
  
  let write_blob = args.write_blob;
  if write_blob {
    info!("Will write blob data to file!");
  }
  
  let json_content  : String;
  let config        : json::JsonValue;
  
  let nboards       : usize;

  let use_master_trigger = args.use_master_trigger;
  let mut master_trigger_ip   = String::from("");
  let mut master_trigger_port = 0usize;

  match args.json_config {
    None => panic!("No .json config file provided! Please provide a config file with --json-config or -j flag!"),
    Some(_) => {
      if !args.json_config.as_ref().unwrap().exists() {
          panic!("The file {} does not exist!", args.json_config.as_ref().unwrap().display());
      }
      info!("Found config file {}", args.json_config.as_ref().unwrap().display());
      json_content = std::fs::read_to_string(args.json_config.as_ref().unwrap()).unwrap();
      config = json::parse(&json_content).unwrap();
      //println!(" .. .. using config:");
      //println!("  {}", config.pretty(2));
      nboards = config["readout_boards"].len();
      info!("Found configuration for {} readout boards!", nboards);
      //for n in 0..config["readout_boards"].len() {
      //   println!(" {}", config["readout_boards"][n].pretty(2));
    } // end Some
  } // end match
  
  if use_master_trigger {
   master_trigger_ip   = config["master_trigger"]["ip"].as_str().unwrap().to_owned();
   master_trigger_port = config["master_trigger"]["port"].as_usize().unwrap();
   info!("Will connect to the master trigger board at {}:{}", master_trigger_ip, master_trigger_port);
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
  
  // FIXME - port and address need to be 
  // configurable
  let mut port       : usize;
  
  let mut address : String;
  let mut cali_file_name : String;
  let mut cali_file_path : &Path;
  let mut board_config   : &json::JsonValue;
  let mut rb_id          : usize;

  // prepare channels for inter thread communications
  
  // master thread -> event builder ocmmuncations
  let (master_ev_send, master_ev_rec): (Sender<MasterTriggerEvent>, Receiver<MasterTriggerEvent>) = channel(); 
  // event builder  <-> paddle cache communications
  let (pp_send, pp_rec) : (Sender<Option<PaddlePacket>>, Receiver<Option<PaddlePacket>>) = channel(); 
  // readout boards <-> paddle cache communications 
  let (rb_send, rb_rec) : (Sender<PaddlePacket>, Receiver<PaddlePacket>) = channel();
  // paddle cache <-> event builder communications
  let (id_send, id_rec) : (Sender<Option<u32>>, Receiver<Option<u32>>) = channel();
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
  let mut nthreads = nboards + 2; // 
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

  // open a zmq context
  println!("==> Seting up zmq context and opening socket for event builder!");
  let ctx = zmq::Context::new();
  let evb_comm_socket = ctx.socket(zmq::PUB).unwrap();
  let mut address_ip = String::from("tcp://");
  address_ip += config["flight_computer"]["ip_address"].as_str().unwrap();
  port        = config["flight_computer"]["port"].as_usize().unwrap();
  address = address_ip.to_owned() + ":" + &port.to_string();
  info!("Will bind to port for flight comp comm at {}", address);
  let evb_comm_ok = evb_comm_socket.bind(&address);
  match evb_comm_ok {
      Ok(_)    => info!("Bound socket to {}", address),
      Err(err) => panic!("Can not communicate with rb at address {}. Maybe you want to check your .json configuration file?, error {}",address, err)
  }

  println!("==> Starting event builder and master trigger threads...");
  if use_master_trigger {
    // start the event builder thread
    worker_threads.execute(move || {
                           event_builder(&master_ev_rec,
                                         &id_send,
                                         &pp_rec,
                                         &evb_comm_socket);
    });
    // master trigger
    worker_threads.execute(move || {
                           master_trigger(&master_trigger_ip, 
                                          master_trigger_port,
                                          &master_ev_send);
    });
  } else {
    // we start the event builder without 
    // depending on the master trigger
    worker_threads.execute(move || {
                           event_builder_no_master(&id_send,
                                                   &pp_rec,
                                                   &evb_comm_socket);
    });
  }
  println!("==> Will now start rb threads..");

  for n in 0..nboards {
    board_config   = &config["readout_boards"][n];
    address_ip = String::from("tcp://");
    let rb_comm_socket = ctx.socket(zmq::REP).unwrap();
    rb_id = board_config["id"].as_usize().unwrap();
    address_ip += board_config["ip_address"].as_str().unwrap();
    port        = board_config["port"].as_usize().unwrap();
    address = address_ip.to_owned() + ":" + &port.to_string();
    info!("Will bind to port for rb comm at {}", address);
    cali_file_name = "".to_owned() + board_config["calibration_file"].as_str().unwrap();
    cali_file_path = Path::new(&cali_file_name);
    if !cali_file_path.exists() {
      panic!("The desired configuration file {} does not exist!", cali_file_name);
    }

    let result = rb_comm_socket.bind(&address);
    match result {
        Ok(_)    => info!("Bound socket to {}", address),
        Err(err) => panic!("Can not communicate with rb at address {}. Maybe you want to check your .json configuration file?, error {}",address, err)
    }
    let this_rb_pp_sender = rb_send.clone();
    worker_threads.execute(move || {
      readoutboard_communicator(&rb_comm_socket,
                                this_rb_pp_sender,
                                rb_id,
                                write_blob,
                                &cali_file_name); 
    });
  } // end for loop over nboards
  let one_minute = time::Duration::from_millis(60000);
  let one_second = time::Duration::from_millis(1000);
  
  // now as we have the readoutboard threads started, 
  // give them some time to fire up and then let the 
  // event builder and finally the master trigger 
  // thread start
  
  //println!("==> Sleeping a bit to give the rb's a chance to fire up..");
  //thread::sleep(10*one_second);

  thread::sleep(10*one_minute);
  println!("Program terminating after specified runtime! So long and thanks for all the {}", fish); 
}
