//! API for liftof-cc, these are basically the individual threads
//!
//!

//use std::time::Instant;
//
//extern crate sensors;
//use self::sensors::Sensors;
//
//extern crate crossbeam_channel;
//use crossbeam_channel::Sender;
//
//use tof_dataclasses::monitoring::{TofCmpMoniData,
//                                  MtbMoniData};
//use tof_dataclasses::packets::TofPacket;
//use liftof_lib:: monitor_mtb;

///// Temperature monitoring for the tof computer. 
///// This works only on that machine. Unfortunatly, nothing smart seems
///// to work.
/////
///// Can't fail. Will return (0,0) when broken.
///// FIXME
///// # Returns
/////
///// (CORE 1 TEMP, CORE 2 TEMP, PCH TEMP)
/////
//pub fn read_cpu_temperature() -> (f64,f64,f64) {
//
//  let mut c1_t  = 0f64;
//  let mut c2_t  = 0f64;
//  let mut pch_t = 0f64;
//  let sensors = Sensors::new();
//  let sensors_c = Sensors::new();
//  for chip_c in sensors {
//    if chip_c.get_name().unwrap() == "pch_skylake-virtual-0" {
//      for feature in chip_c {
//        for subfeature in feature {
//          if subfeature.name() == "temp1_input" {
//            pch_t = subfeature.get_value().unwrap();
//          }
//        }
//      }
//    }
//  }
//  for chip in sensors_c {
//    if chip.get_name().unwrap() == "coretemp-isa-0000" {
//      for feature in chip {
//        for subfeature in feature {
//          //println!("{}", subfeature.name());
//          if subfeature.name() == "temp2_input" {
//            c1_t = subfeature.get_value().unwrap();
//          }
//          if subfeature.name() == "temp3_input" {
//            c2_t = subfeature.get_value().unwrap();
//          }
//          //println!( "    - {} = {}", subfeature.name(), subfeature.get_value().unwrap());
//        }
//      }
//    }
//    //println!( "{} (on {})",
//    //   chip.get_name().unwrap(),
//    //   chip.bus().get_adapter_name().unwrap()
//    //);
//    //for feature in chip {
//    //  println!("  - {}", feature.get_label().unwrap());
//    //  for subfeature in feature {
//    //    println!( "    - {} = {}", subfeature.name(), subfeature.get_value().unwrap()
//    //    );
//    //  }
//    //}
//  }
//  info!("=> Tof computer CPU Temps - Core 1 [C] {}, Core 2 [C] {}, PCH [C] {}", c1_t, c2_t, pch_t);
//  (c1_t, c2_t, pch_t)
//}



///// Do "global" monitoring tasks, that is monitor cpu temp
///// and usage of the tof computer itself and the MTB
/////
///// # Arguments
/////
///// * tp_to_sink    : The moni data will be wrapped in tof packets
/////                   Send them to the global data sink for 
/////                   further distribution/saving on disk
///// * mtb_ip        : if the MTB is used, this is the supposed ip 
/////                   of the MTB
///// * mtb_port      : if the MTB is used, listen to this port.
///// * moni_interval : in seconds - read new moni data
///// * verbose       : print moni information to console
//pub fn tofcmp_and_mtb_moni(tp_to_sink    : &Sender<TofPacket>,
//                           mtb_ip        : &str,
//                           mtb_port      : usize,
//                           moni_interval : u64,
//                           verbose       : bool) {
//  let use_mtb = mtb_ip != "";
//  let mut timer   = Instant::now();
//  //let mut socket  : io::Result::<UdpSocket>; 
//  let mut mtb_moni    = MtbMoniData::new();
//  let mut tofcmp_moni = TofCmpMoniData::new();
//  let mut tp : TofPacket;
//  let mtb_address = mtb_ip.to_owned() + ":" + &mtb_port.to_string();
//  loop {
//    // reconnect to MTB
//    if timer.elapsed().as_secs() > moni_interval {
//      if use_mtb {
//        //monitor_mtb(&mtb_address, &mut mtb_moni);
//      }
//
//      let (c1, c2, pch) = read_cpu_temperature();
//      tofcmp_moni.core1_tmp = c1 as u8;
//      tofcmp_moni.core2_tmp = c2 as u8;
//      tofcmp_moni.pch_tmp   = pch as u8;
//      if verbose {
//        println!("{}", tofcmp_moni);
//        if use_mtb {
//          println!("{}", mtb_moni);
//        }
//      }
//      tp = TofPacket::from(&tofcmp_moni);
//      match tp_to_sink.send(tp) {
//        Err(err) => error!("Tof computer moni data packet sending failed! Err {}", err),
//        Ok(_)    => ()
//      }
//      if use_mtb {
//        tp = TofPacket::from(&mtb_moni);
//        match tp_to_sink.send(tp) {
//          Err(err) => error!("MTB moni data packet sending failed! Err {}", err),
//          Ok(_)    => () 
//        }
//      }
//      timer = Instant::now();
//    }
//  }
//}



//#[test]
//fn test_read_cpu_temperature() {
//  // Call the function to get the CPU temperature
//  let cpu_temp = read_cpu_temperature();
//  println!("Got cpu temp of {:?}", cpu_temp);
//  assert!(cpu_temp.0 <= 100.0, "CPU temperature should be within a reasonable range");
//  assert!(cpu_temp.1 <= 100.0, "CPU temperature should be within a reasonable range");
//  assert!(cpu_temp.2 <= 100.0, "CPU temperature should be within a reasonable range");
//}

