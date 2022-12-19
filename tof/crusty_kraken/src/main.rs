mod readoutboard_blob;
mod calibrations;
mod readoutboard_comm;
mod threading;
mod reduced_tofevent;
mod constants;
mod waveform;
mod errors;
mod commands;
mod master_trigger;
mod monitoring;

// this is a list of tests
// FIXME - this should follow
// the "official" structure
// for now, let's just keep it here
mod test_blobdata;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

extern crate clap;
extern crate json;
extern crate hdf5;
extern crate ndarray;

use std::{thread,
          time,
          path::Path};

use clap::{arg,
           command,
           //value_parser,
           //ArgAction,
           //Command,
           Parser};

use crate::constants::{MAX_NBOARDS, NCHN};
use crate::readoutboard_comm::readoutboard_communicator;
use crate::master_trigger::master_and_commander;
use crate::threading::ThreadPool;

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
  master_trigger: bool,
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
  let satelite_antenna      = vec![240, 159, 147, 161];
  // We know these bytes are valid, so we'll use `unwrap()`.
  let sparkle_heart    = String::from_utf8(sparkle_heart).unwrap();
  let kraken           = String::from_utf8(kraken).unwrap();
  let satelite_antenna = String::from_utf8(satelite_antenna).unwrap();

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
  
  let mut json_content  : String;
  let mut config        : json::JsonValue;
  
  let mut nboards       = 0usize;

  let master_trigger = args.master_trigger;
  let mut master_trigger_ip   = "";
  let mut master_trigger_port = 0usize;

  match args.json_config {
    None => panic!("No .json config file provided! Please provide a config file with --json-config or -j flag!"),
    Some(ref json_file_path) => {
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
  
  if master_trigger {
   master_trigger_ip = config["master_trigger"]["ip"].as_str().unwrap();
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
  let mut board_config : &json::JsonValue;
  let mut rb_id = 0usize;
 
  // for debugging, if master trigger only 
  // run master trigger thread
  if master_trigger {
    master_and_commander(master_trigger_ip, 
                         master_trigger_port);
    panic!("Done with the master trigger debugging!");
  }

  // each readoutboard gets its own worker
  let rbcomm_workers = ThreadPool::new(nboards);
  
  // open a zmq context
  let ctx = zmq::Context::new();
  for n in 0..nboards {
    board_config   = &config["readout_boards"][n];
    let mut address_ip = String::from("tcp://");
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
    rbcomm_workers.execute(move || {
      readoutboard_communicator(&rb_comm_socket,
                                rb_id,
                                write_blob,
                                &cali_file_name); 
    });
  }
  
  let one_minute = time::Duration::from_millis(60000);
  //let now = time::Instant::now();
  
  thread::sleep(2*one_minute);
}
