//! Communication with the flight computer
//!
//! Using two dedicated 0MQ wires - one for 
//! data, the other for commands
//!
//!

use std::time::{
    Instant,
    //Duration,
};

use std::sync::{
    Arc,
    Mutex,
};


extern crate crossbeam_channel;
use crossbeam_channel::Receiver; 

use colored::Colorize;

use tof_dataclasses::packets::{
    TofPacket,
    PacketType
};

use tof_dataclasses::threading::{
    ThreadControl,
};

use tof_dataclasses::monitoring::{
    RBMoniData,
    MtbMoniData,
    CPUMoniData
};

//use tof_dataclasses::events::TofEvent;
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::io::{
    TofPacketWriter,
    FileType
};
use tof_dataclasses::events::TofEvent;


use liftof_lib::settings::DataPublisherSettings;

/// Manages "outgoing" 0MQ PUB socket
///
/// Everything should send to here, and 
/// then it gets passed on over the 
/// connection to the flight computer
///
/// # Arguments
///
/// * flight_address   : The address the flight computer
///                      (or whomever) wants to listen.
///                      A 0MQ PUB socket witll be bound 
///                      to this address.
/// * write_npack_file : Write this many TofPackets to a 
///                      single file before starting a 
///                      new one.
pub fn global_data_sink(incoming           : &Receiver<TofPacket>,
                        write_stream       : bool,
                        runid              : usize,
                        settings           : &DataPublisherSettings,
                        print_moni_packets : bool,
                        thread_control     : Arc<Mutex<ThreadControl>>) {
  let flight_address      = settings.fc_pub_address.clone();
  let write_stream_path   = settings.data_dir.clone(); 
  let write_npack_file    = settings.packs_per_file;
  let mut met_time_secs   = 0f32; // mission elapsed time

  let ctx = zmq::Context::new();
  // FIXME - should we just move to another socket if that one is not working?
  let data_socket = ctx.socket(zmq::PUB).expect("Can not create socket!");
  let unlim : i32 = 1000000;
  data_socket.set_sndhwm(unlim).unwrap();
  match data_socket.bind(&flight_address) {
    Err(err) => panic!("Can not bind to address {}! {}", flight_address, err),
    Ok(_)    => ()
  }
  info!("ZMQ PUB Socket for global data sink bound to {flight_address}");

  let mut writer : Option<TofPacketWriter> = None;
  if write_stream {
    //let mut streamfile_name = write_stream_path + "/run_";
    //streamfile_name += &runid.to_string();
    let file_type = FileType::RunFile(runid as u32);
    //println!("==> Writing stream to file with prefix {}", streamfile_name);
    writer = Some(TofPacketWriter::new(write_stream_path.clone(), file_type));
    writer.as_mut().unwrap().pkts_per_file = write_npack_file;
  }
  //let mut event_cache = Vec::<TofPacket>::with_capacity(100); 

  let mut n_pack_sent = 0;
  //let mut last_evid   = 0u32;

  // for debugging/profiling
  let mut timer = Instant::now();
  loop {
    match incoming.recv() {
      Err(err) => trace!("No new packet, err {err}"),
      Ok(pack) => {
        debug!("Got new tof packet {}", pack.packet_type);
        if writer.is_some() {
          writer.as_mut().unwrap().add_tof_packet(&pack);
        }
        // yeah, this is it. 
        // catch RBCalibration packets here,
        // they will end up automatically in 
        // the stream, but if we catch them 
        // and expose them to an arc mutex,
        // the waveform processing could 
        // access them directly. 
        match pack.packet_type {
          PacketType::RBCalibration => {
            let cali_rb_id = pack.payload[2]; 
            debug!("Received RBCalibration packet for board {}!", cali_rb_id);
            // we notify the other threads that we got this specific packet, 
            // so we know how long we still have to wait
            match thread_control.lock() {
              Ok(mut tc) => {
                // FIXME - unwrap (for bad packets)
                *tc.finished_calibrations.get_mut(&cali_rb_id).unwrap() = true; 
              },
              Err(err) => {
                error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
              },
            }
            
            // See RBCalibration reference
            let file_type  = FileType::CalibrationFile(cali_rb_id);
            //println!("==> Writing stream to file with prefix {}", streamfile_name);
            let mut cali_writer = TofPacketWriter::new(write_stream_path.clone(), file_type);
            cali_writer.add_tof_packet(&pack);
            drop(cali_writer);
          }
          _ => ()
        }
        if print_moni_packets {
          let mut pos = 0;
          // some output to the console
          match pack.packet_type {
            PacketType::RBMoni => {
              let moni = RBMoniData::from_bytestream(&pack.payload, &mut pos);
              match moni {
                Ok(data) => {
                  debug!("Sending RBMoniData {}", data);
                },
                Err(err) => error!("Can not unpack RBMoniData! {err}")}
              }, 
            PacketType::CPUMoniData => {
              let moni = CPUMoniData::from_bytestream(&pack.payload, &mut pos);
              match moni {
                Ok(data) => {println!("{}", data);},
                Err(err) => error!("Can not unpack TofCmpData! {err}")}
              },
            PacketType::MonitorMtb => {
              let moni = MtbMoniData::from_bytestream(&pack.payload, &mut pos);
              match moni {
                Ok(data) => {println!("{}", data);},
                Err(err) => error!("Can not unpack MtbMoniData! {err}")}
              },
            _ => ()
          } // end match 
        }
        //if pack.packet_type == PacketType::TofEvent {
        //  if event_cache.len() != 100 {
        //    event_cache.push(pack);
        //    continue;
        //  } else {
        //    if n_pack_sent % 1000 == 0 && n_pack_sent != 0 {
        //      println!("=> [SINK] Sent {n_pack_sent}, last evid {last_evid} ===");
        //    }
        //    // sort the cache
        //    // FIXME - at this step, we should have checked if the 
        //    // packets are broken.
        //    event_cache.sort_by(| a, b|  TofEvent::extract_event_id_from_stream(&a.payload).unwrap().cmp(
        //                                &TofEvent::extract_event_id_from_stream(&b.payload).unwrap()));
        //   
        //    for ev in event_cache.iter() {
        //      last_evid = TofEvent::extract_event_id_from_stream(&ev.payload).unwrap();
        //      match data_socket.send(&ev.to_bytestream(),0) {
        //        Err(err) => error!("Not able to send packet over 0MQ PUB, {err}"),
        //        Ok(_)    => { 
        //          trace!("TofPacket sent");
        //          n_pack_sent += 1;
        //        }
        //      }
        //    }
        //    event_cache.clear();
        //  }

        //} else {
        // FIXME - disentangle network and disk I/O?
        if pack.packet_type == PacketType::TofEvent {
          if settings.send_flight_packets {
            let mut pos = 0;
            // unfortunatly we have to do this unnecessary step
            // I have to think about fast tracking these.
            // maybe sending TofEvents over the channel instead
            // of TofPackets?
            let ev_to_send : TofEvent;
            match TofEvent::from_bytestream(&pack.payload, &mut pos) {
              Err(err) => {
                error!("Unable to unpack TofEvent! {err}");
                continue;
              },
              Ok(_ev_to_send) => {
                ev_to_send = _ev_to_send;
              }
            }
            let te_summary = ev_to_send.get_summary();
            let pack = TofPacket::from(&te_summary);
            match data_socket.send(pack.to_bytestream(),0) {
              Err(err) => {
                error!("Packet sending failed! {err}");
              }
              Ok(_)    => {
                //trace!("Event Summary for event id {} send!", evid);
                n_pack_sent += 1;
              }
            }
            if settings.send_rbwaveforms {
              for rbwave in ev_to_send.get_rbwaveforms() {
                let pack = TofPacket::from(&rbwave);
                match data_socket.send(pack.to_bytestream(),0) {
                  Err(err) => {
                    error!("Packet sending failed! {err}");
                  }
                  Ok(_)    => {
                    //trace!("RB waveform for event id {} send!", evid);
                    n_pack_sent += 1;
                  }
                }
              }
            }
            if settings.send_mtb_event_packets {
              let pack = TofPacket::from(&ev_to_send.mt_event);
              match data_socket.send(pack.to_bytestream(),0) {
                Err(err) => {
                  error!("Packet sending failed! {err}");
                }
                Ok(_)    => {
                  //trace!("RB waveform for event id {} send!", evid);
                  n_pack_sent += 1;
                }
              }
            }
          } else {
            match data_socket.send(pack.to_bytestream(),0) {
              Err(err) => error !("Not able to send packet over 0MQ PUB! {err}"),
              Ok(_)    => {
                trace!("TofPacket sent");
                n_pack_sent += 1;
              }
            } // end match
          }
        // FIXME else branching not optimal
        } else {
          match data_socket.send(pack.to_bytestream(),0) {
            Err(err) => error !("Not able to send packet over 0MQ PUB! {err}"),
            Ok(_)    => {
              trace!("TofPacket sent");
              n_pack_sent += 1;
            }
          } // end match
        } // end else
      } // end if pk == event packet
    } // end incoming.recv
    if timer.elapsed().as_secs() > 60 {
      met_time_secs += timer.elapsed().as_secs_f32();
      let packet_rate = n_pack_sent as f32 /met_time_secs;
      println!("  {:<60}", ">> == == == == ==  DATA SINK HEARTBEAT   == == == == == <<".bright_cyan().bold());
      println!("  {:<60} <<", format!(">> ==> Sent \t{} TofPackets! (packet rate {:.2}/s)", n_pack_sent ,packet_rate).bright_cyan());
      println!("  {:<60} <<", format!(">> ==> Incoming cb channel len {}", incoming.len()).bright_cyan());

      println!("  {:<60}", ">> == == == == ==  == == == == == == ==  == == == == == <<".bright_cyan().bold());
      timer = Instant::now();
    }
  }
}
