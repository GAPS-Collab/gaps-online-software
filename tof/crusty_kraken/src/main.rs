mod readoutboard_blob;
mod calibrations;
mod readoutboard_comm;
mod threading;
mod reduced_tofevent;
mod constants;
mod waveform;
mod errors;


// this is a list of tests
// FIXME - this should follow
// the "official" structure
// for now, let's just keep it here
mod test_blobdata;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

extern crate clap;

extern crate hdf5;
extern crate ndarray;

use crate::calibrations::{Calibrations, read_calibration_file};

use crate::constants::{NBOARDS, NCHN};

use crate::readoutboard_comm::readoutboard_communicator;

use crate::threading::ThreadPool;

use std::{thread,
          time,
          path::Path};


use clap::Parser;
use clap::{arg, command, value_parser, ArgAction, Command};


/*************************************/

#[derive(Parser, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
    /// Increase output for debugging
    #[arg(short, long, default_value_t = false)]
    debug: bool,
    #[arg(short, long, default_value_t = false)]
    write_blob: bool,
    /// A json config file with detector information
    #[arg(short, long)]
    json_config: Option<std::path::PathBuf>,
}



/*************************************/

fn main() {
   pretty_env_logger::init();
   let args = Args::parse();

   // some bytes, in a vector
   let sparkle_heart         = vec![240, 159, 146, 150];
   let kraken                = vec![240, 159, 144, 153];
   let sattelite_antenna     = vec![240, 159, 147, 161];
   // We know these bytes are valid, so we'll use `unwrap()`.
   let sparkle_heart = String::from_utf8(sparkle_heart).unwrap();
   let kraken        = String::from_utf8(kraken).unwrap();
   let sattelite_antenna = String::from_utf8(sattelite_antenna).unwrap();

   // welcome banner!
   println!("-----------------------------------------------");
   println!(" ** Welcome to crusty_kraken {} *****", kraken);
   println!(" .. TOF C&C and data acquistion suite");
   println!(" .. for the GAPS experiment {}", sparkle_heart);
   println!("-----------------------------------------------");
   println!("");

   let write_blob = args.write_blob;
   if write_blob {
     info!("Will write blob data to file!");
   }

   match args.json_config {
     None => warn!("No config file provided!"),
     Some(ref json_file_path) => {
       if !args.json_config.as_ref().unwrap().exists() {
           panic!("The file {} does not exist!", args.json_config.as_ref().unwrap().display());
       }
       info!("Found config file {}", args.json_config.as_ref().unwrap().display());
     }
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

    // Continued program logic goes here...


  // read calibration data
  let mut calibrations = [[Calibrations {..Default::default()}; NCHN]; NBOARDS];
  let mut rb_id = 0usize;
  for n in 0..NBOARDS {
      rb_id = n + 1;
      let file_name = "/srv/gaps/gfp-data/gaps-gfp/TOFsoftware/server/datafiles/rb".to_owned() + &rb_id.to_string() + "_cal.txt";
      info!("Reading calibrations from file {}", file_name);
      let file_path = Path::new(&file_name);
      calibrations[n] = read_calibration_file(file_path); 
  }
  
  // each readoutboard gets its own worker
  let rbcomm_workers = ThreadPool::new(NBOARDS);
  
  // open a zmq context
  let ctx = zmq::Context::new();
  // FIXME - port and address need to be 
  // configurable
  let mut port = 38830usize;
  let address_ip = "tcp://127.0.0.1";
  
  let mut address : String;
  for n in 0..NBOARDS {
    let rb_comm_socket = ctx.socket(zmq::REP).unwrap();
    address = address_ip.to_owned() + ":" + &port.to_string();
    info!("Will bind to port for rb comm at {}", address);
    let result = rb_comm_socket.bind(&address);
    match result {
        Ok(_)    => info!("Bound socket to {}", address),
        Err(err) => panic!("Can not communicate with rb at address {}, error {}",address, err)
    }
    rbcomm_workers.execute(move || {
        readoutboard_communicator(&rb_comm_socket,
                                  n + 1,
                                  write_blob); 
    });
    port += 1;
  }
  
  let one_minute = time::Duration::from_millis(60000);
  //let now = time::Instant::now();
  
  thread::sleep(2*one_minute);



}
