//! API for liftof-cc, these are basically the individual threads
//!
//!

use std::net::{IpAddr, Ipv4Addr};
use std::net::{UdpSocket, SocketAddr};
use std::io;
use std::time::{Duration, Instant};
use zmq;

extern crate crossbeam_channel;
use sensors::Sensors;

use crossbeam_channel::{Receiver, Sender};

use tof_dataclasses::manifest::ReadoutBoard;
use tof_dataclasses::commands::TofCommand;
use tof_dataclasses::monitoring::{TofCmpMoniData,
                                  MtbMoniData};
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::events::master_trigger::{read_rate,
                                              read_lost_rate,
                                              read_adc_temp_and_vccint,
                                              read_adc_vccaux_and_vccbram};

use liftof_lib::{connect_to_mtb, 
                 MT_MAX_PACKSIZE};

pub const DATAPORT : u32 = 42000;

/// Temperature monitoring for the tof computer. 
/// This works only on that machine. Unfortunatly, nothing smart seems
/// to work.
///
/// Can't fail. Will return (0,0) when broken.
/// FIXME
/// # Returns
///
/// (CORE 1 TEMP, CORE 2 TEMP, PCH TEMP)
///
pub fn read_cpu_temperature() -> (f64,f64,f64) {

  let mut c1_t  = 0f64;
  let mut c2_t  = 0f64;
  let mut pch_t = 0f64;
  let sensors = Sensors::new();
  let sensors_c = Sensors::new();
  for chip_c in sensors {
    if chip_c.get_name().unwrap() == "pch_skylake-virtual-0" {
      for feature in chip_c {
        for subfeature in feature {
          if subfeature.name() == "temp1_input" {
            pch_t = subfeature.get_value().unwrap();
          }
        }
      }
    }
  }
  for chip in sensors_c {
    if chip.get_name().unwrap() == "coretemp-isa-0000" {
      for feature in chip {
        for subfeature in feature {
          //println!("{}", subfeature.name());
          if subfeature.name() == "temp2_input" {
            c1_t = subfeature.get_value().unwrap();
          }
          if subfeature.name() == "temp3_input" {
            c2_t = subfeature.get_value().unwrap();
          }
          //println!( "    - {} = {}", subfeature.name(), subfeature.get_value().unwrap());
        }
      }
    }
    //println!( "{} (on {})",
    //   chip.get_name().unwrap(),
    //   chip.bus().get_adapter_name().unwrap()
    //);
    //for feature in chip {
    //  println!("  - {}", feature.get_label().unwrap());
    //  for subfeature in feature {
    //    println!( "    - {} = {}", subfeature.name(), subfeature.get_value().unwrap()
    //    );
    //  }
    //}
  }
  info!("=> Tof computer CPU Temps - Core 1 [C] {}, Core 2 [C] {}, PCH [C] {}", c1_t, c2_t, pch_t);
  (c1_t, c2_t, pch_t)
}

/// Do "global" monitoring tasks, that is monitor cpu temp
/// and usage of the tof computer itself and the MTB
///
/// # Arguments
///
/// * tp_to_sink    : The moni data will be wrapped in tof packets
///                   Send them to the global data sink for 
///                   further distribution/saving on disk
/// * mtb_ip        : if the MTB is used, this is the supposed ip 
///                   of the MTB
/// * mtb_port      : if the MTB is used, listen to this port.
/// * moni_interval : in seconds - read new moni data
/// * verbose       : print moni information to console
pub fn tofcmp_and_mtb_moni(tp_to_sink    : &Sender<TofPacket>,
                           mtb_ip        : &str,
                           mtb_port      : usize,
                           moni_interval : u64,
                           verbose       : bool) {
  let use_mtb = mtb_ip != "";
  let mut mtb_address = String::from("");
  let mut timer   = Instant::now();
  let mut socket  : io::Result::<UdpSocket>; 
  let mut buffer = [0u8;MT_MAX_PACKSIZE];  
  let mut mtb_moni    = MtbMoniData::new();
  let mut tofcmp_moni = TofCmpMoniData::new();
  let mut tp = TofPacket::new();
  loop {
    // reconnect to MTB
    if timer.elapsed().as_secs() > moni_interval {
      if use_mtb {
        // connect to the mtb and get the rate
        mtb_address = mtb_ip.to_owned() + ":" + &mtb_port.to_string();
        socket = connect_to_mtb(&mtb_address); 
        if let Ok(sock) = socket {
          match read_rate(&sock, &mtb_address, &mut buffer) {
            Err(err) => {
              error!("Unable to obtain MT rate information! error {err}");
            }
            Ok(rate) => {
              info!("Got MTB rate of {rate}");
              mtb_moni.rate = rate as u16;
            }
          } // end match
          match read_adc_vccaux_and_vccbram(&sock, &mtb_address, &mut buffer) {
            Err(err) => {
              error!("Unable to obtain MT VCCAUX and VCCBRAM! error {err}");
            }
            Ok(values) => {
              mtb_moni.fpga_vccaux  = values.0;
              mtb_moni.fpga_vccbram = values.1; 
            }
          }
          match read_adc_temp_and_vccint(&sock, &mtb_address, &mut buffer) {
            Err(err) => {
              error!("Unable to obtain MT VCCAUX and VCCBRAM! error {err}");
            }
            Ok(values) => {
              mtb_moni.fpga_temp    = values.0;
              mtb_moni.fpga_vccint  = values.1; 
            }
          }

        } else {
          error!("Can not connect to MTB at {}", mtb_address);
        }
      }
      let (c1, c2, pch) = read_cpu_temperature();
      tofcmp_moni.core1_tmp = c1 as u8;
      tofcmp_moni.core2_tmp = c2 as u8;
      tofcmp_moni.pch_tmp   = pch as u8;
      if verbose {
        println!("{}", tofcmp_moni);
        if use_mtb {
          println!("{}", mtb_moni);
        }
      }
      tp = TofPacket::from(&tofcmp_moni);
      match tp_to_sink.send(tp) {
        Err(err) => error!("Tof computer moni data packet sending failed! Err {}", err),
        Ok(_)    => ()
      }
      if use_mtb {
        tp = TofPacket::from(&mtb_moni);
        match tp_to_sink.send(tp) {
          Err(err) => error!("MTB moni data packet sending failed! Err {}", err),
          Ok(_)    => () 
        }
      }
      timer = Instant::now();
    }
  }
}


/// This is listening to commands from the flight computer 
/// and relays them to the RadoutBoards
/// 
/// # Arguments 
///
/// * rbs 
/// * rp_to_main
pub fn commander(rbs : &Vec<ReadoutBoard>,
                 cmd : Receiver<TofCommand>){
                 //rp_to_main : &Sender<RunParams>) {
             
  info!("Initialiized");
  let ctx = zmq::Context::new();
  //let mut sockets = Vec::<zmq::Socket>::new();

  let mut address_ip = String::from("tcp://");
  //let this_board_ip = local_ip().expect("Unable to obtainl local board IP. Something is messed up!");
  let data_port    = DATAPORT;
  let this_board_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 1, 1));

  match this_board_ip {
    IpAddr::V4(ip) => address_ip += &ip.to_string(),
    IpAddr::V6(_) => panic!("Currently, we do not support IPV6!")
  }
  let data_address : String = address_ip.clone() + ":" + &data_port.to_string();
  let data_socket = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  data_socket.bind(&data_address).expect("Unable to bind to data (PUB) socket {data_adress}");
  println!("0MQ PUB socket bound to address {data_address}");
  //let init_run = TofCommand::DataRunStart(100000);
  //let mut payload_cmd  = init_run.to_bytestream();
  //let mut payload  = String::from("BRCT").into_bytes();
  //payload.append(&mut payload_cmd);

  println!("Starting cmd receiver loop!");
  loop {
    // check if we get a command from the main 
    // thread
    match cmd.try_recv() {
      Err(err) => trace!("Did not receive a new command, error {err}"),
      Ok(new_command) => {
        info!("Received new command!");
        let mut payload  = String::from("BRCT").into_bytes();
        let mut payload_cmd = new_command.to_bytestream();
        payload.append(&mut payload_cmd);
        println!("{:?}", payload);
        match data_socket.send(&payload,0) {
          Err(err) => error!("Can send command! Error {err}"),
          Ok(_)    => info!("BRCT command sent!")
        }
      }
    }
  }
  //for rb in rbs.iter() {
  //  let sock = ctx.socket(zmq::REQ).expect("Unable to create socket!");
  //  let address = "tcp://".to_owned()
  //            + &rb.ip_address.expect("No IP known for this board!").to_string()
  //            + ":"
  //            +  &rb.cmd_port.expect("No CMD port known for this board!").to_string();
  //  sock.connect(&address);
  //  sockets.push(sock);
  //}
  //let init_run = TofCommand::DataRunStart(100000);
  ////let init_run = RunParams::new();
  //for s in sockets.iter() {
  //  match s.send(init_run.to_bytestream(), 0) {
  //    Err(err) => warn!("Could not initalize run, err {err}"),
  //    Ok(_)    => info!("Initialized run!")
  //  }
  //}
}

#[test]
fn test_read_cpu_temperature() {
  // Call the function to get the CPU temperature
  let cpu_temp = read_cpu_temperature();
  println!("Got cpu temp of {:?}", cpu_temp);
  assert!(cpu_temp.0 <= 100.0, "CPU temperature should be within a reasonable range");
  assert!(cpu_temp.1 <= 100.0, "CPU temperature should be within a reasonable range");
  assert!(cpu_temp.2 <= 100.0, "CPU temperature should be within a reasonable range");
}

