use std::process::exit;
use clap::Parser;
#[macro_use] extern crate log;
extern crate env_logger;
use std::io::Write;


use liftof_lib::{
    color_log,
};

use tof_dataclasses::calibrations::RBCalibrations;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::io::{
    TofPacketReader,
    TofPacketWriter,
};
#[derive(Parser, Default, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// Recalibrate an existing calibration (re-calculate the calibration constants)
  #[arg(long, default_value_t=String::from(""))]
  recalibrate: String,
  /// File with No-input data
  noi_file: Option<String>,
  /// File with vcal data
  vcal_file: Option<String>,
  /// File with tcal data
  tcal_file: Option<String>,
}

fn main() {
  
  env_logger::builder()
    .format(|buf, record| {
    writeln!( buf, "[{level}][{module_path}:{line}] {args}",
      level = color_log(&record.level()),
      module_path = record.module_path().unwrap_or("<unknown>"),
      //target = record.target(),
      line = record.line().unwrap_or(0),
      args = record.args()
      )
    }).init();
 
  let args = Args::parse(); 
  if args.recalibrate != String::from("") {
    println!("=> Will recalibrate {}", args.recalibrate);
    let mut reader = TofPacketReader::new(args.recalibrate);
    let mut cali   : RBCalibrations;
    match reader.next() {
      None => {
        error!("Can not read from inputfile!");
      },
      Some(tp) => {
        match RBCalibrations::from_bytestream(&tp.payload, &mut 0) {
          Err(err) => error!("Can not decode RBCalibration from input file! {err}"),
          Ok(_cali) => {
            cali = _cali;
            match cali.calibrate() {
              Err(err) => {
                error!("Calibration failed, exiting! {err}");
                exit(1);
              },
              Ok(_) => ()
            }
            let rb_id = cali.rb_id;
            let mut outfile = String::from("rb");
            if rb_id < 10 {
              outfile += "0";
            }
            outfile += &rb_id.to_string();
            outfile += ".cali";
            let mut writer  = TofPacketWriter::new(outfile);
            let pack        = TofPacket::from(&cali);
            writer.add_tof_packet(&pack);
            println!("=> Done, exciting!");
            exit(0); 
          }
        }
      }
    }
  }

  let mut cali = RBCalibrations::new(27);
  let mut inputfiles : Vec<String> = Vec::<String>::new();
  inputfiles.push(args.noi_file. expect("Please specify noi data path").clone());
  inputfiles.push(args.vcal_file.expect("Please specify vcal data path").clone());
  inputfiles.push(args.tcal_file.expect("Please specify tcal data path").clone());
  for f in inputfiles.iter().enumerate() {
    info!("Processing {}", f.1);
    let mut reader = TofPacketReader::new(f.1.to_string());
    loop {
      match reader.next() {
        None => {
          break;
        }
        Some(tp) => {
          match tp.unpack_rbevent() {
            Err(err) => {
              error!("Issues with packet {err}");
            }
            Ok(event) => { 
              if f.0 == 0 {
                cali.noi_data.push(event);
                continue;
              }
              if f.0 == 1 {
                cali.vcal_data.push(event);
                continue;
              }
              if f.0 == 2 {
                cali.tcal_data.push(event);
                continue;
              }
            }
          }
        }
      }
    }
  }
  match cali.calibrate() {
    Err(err) => {
      error!("Calibration failed, exiting! {err}");
      exit(1);
    },
    Ok(_) => ()
  }
  let mut writer = TofPacketWriter::new("rb27.cali.tof.gaps".to_string());
  let pack       = TofPacket::from(&cali);
  writer.add_tof_packet(&pack);
}
