//! Communication with the flight computer
//!
//! Using two dedicated 0MQ wires - one for 
//! data, the other for commands
//!
//!

use std::time::{
    Instant,
    Duration,
};
use std::thread::sleep;
use std::sync::{
    Arc,
    Mutex,
};

use std::fs::create_dir_all;

extern crate crossbeam_channel;
use crossbeam_channel::Receiver; 

use colored::Colorize;

use tof_dataclasses::packets::{
    TofPacket,
    PacketType
};

//use tof_dataclasses::threading::{
//    ThreadControl,
//};

use tof_dataclasses::monitoring::{
    RBMoniData,
    MtbMoniData,
    CPUMoniData
};

//use tof_dataclasses::events::TofEvent;
use tof_dataclasses::serialization::{
    Serialization,
};

use tof_dataclasses::io::{
    TofPacketWriter,
    FileType,
    get_utc_timestamp
};

use tof_dataclasses::events::TofEvent;

#[cfg(features="debug")]
use tof_dataclasses::heartbeats::HeartBeatDataSink;

//use liftof_lib::settings::DataPublisherSettings;
use liftof_lib::thread_control::ThreadControl;

/// Manages "outgoing" 0MQ PUB socket and writing
/// data to disk
///
/// All received packets will be either forwarded
/// over zmq or saved to disk
///
/// # Arguments
///
/// * incoming           : incoming connection for TofPackets from 
///                        other threads
/// * print_moni_packets : print monitoring packets to the terminal
/// * thread_control     : start/stop thread, calibration information
pub fn global_data_sink(incoming           : &Receiver<TofPacket>,
                        print_moni_packets : bool,
                        thread_control     : Arc<Mutex<ThreadControl>>) {
  // when the thread starts, we need to wait a bit
  // till thread_control becomes usable
  sleep(Duration::from_secs(10));
  let mut flight_address           = String::from("");
  let mut mbytes_per_file          = 420usize;
  let mut write_stream_path        = String::from("");
  let mut cali_dir                 = String::from("");
  let mut send_tof_summary_packets = false;
  let mut send_rbwaveform_packets  = false;
  let mut send_mtb_event_packets   = false;
  let mut send_tof_event_packets   = false;
  match thread_control.lock() {
    Ok(mut tc) => {
      tc.thread_data_sink_active = true; 
      flight_address    = tc.liftof_settings.data_publisher_settings.fc_pub_address.clone();
      mbytes_per_file   = tc.liftof_settings.data_publisher_settings.mbytes_per_file; 
      write_stream_path = tc.liftof_settings.data_publisher_settings.data_dir.clone();
      cali_dir = tc.liftof_settings.data_publisher_settings.cali_dir.clone();
      send_tof_summary_packets = tc.liftof_settings.data_publisher_settings.send_tof_summary_packets;
      send_rbwaveform_packets  = tc.liftof_settings.data_publisher_settings.send_rbwaveform_packets;
      send_mtb_event_packets   = tc.liftof_settings.data_publisher_settings.send_mtb_event_packets;
      send_tof_event_packets   = tc.liftof_settings.data_publisher_settings.send_tof_event_packets;
    },
    Err(err) => {
      error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
    },
  }
  //let one_second          = Duration::from_millis(1000);
  //let flight_address      = settings.fc_pub_address.clone();
  //let write_stream_path   = settings.data_dir.clone(); 
  //let mut write_stream_path = String::from("");
  //let mbytes_per_file     = settings.mbytes_per_file;
  let mut met_time_secs   = 0f32; // mission elapsed time

  let mut evid_check      = Vec::<u32>::new();

  let ctx = zmq::Context::new();
  // FIXME - should we just move to another socket if that one is not working?
  let data_socket = ctx.socket(zmq::PUB).expect("Can not create socket!");
  let unlim : i32 = 1000000;
  data_socket.set_sndhwm(unlim).unwrap();
  //println!("==> Will bind zmq socket to address {}", flight_address);
  match data_socket.bind(&flight_address) {
    // FIXEM - this panic is no good! What we want to do instead is
    // 1) set the flag in thread_control that we are running 
    // to false, 
    // 2) enter an eternal loop where we try to restart it
    Err(err) => panic!("Can not bind to address {}! {}", flight_address, err),
    Ok(_)    => ()
  }
  info!("ZMQ PUB Socket for global data sink bound to {flight_address}");

  //let mut event_cache = Vec::<TofPacket>::with_capacity(100); 

  #[cfg(features="debug")]
  let mut heartbeat         = HeartBeatDataSink::new();
  let mut n_pack_sent       = 0;
  //let mut last_evid       = 0u32;
  let mut n_pack_write_disk = 0usize;
  let mut bytes_sec_disk    = 0f64;

  // for debugging/profiling
  let mut timer      = Instant::now();
  let mut kill_timer = Instant::now();

  let mut cali_expected    = 40;
  let mut cali_dir_created = false;
  let mut cali_output_dir  = String::from("");
  let mut cali_completed   = 0;
 
  // run settings 
  let mut writer : Option<TofPacketWriter> = None;
  let mut write_stream  = false;
  let mut runid : u32 = 0;
  let mut new_run_start = false;
  loop {
    // even though this is called kill timer, check
    // the settings in general, since they might have
    // changed due to remote access.
    if kill_timer.elapsed().as_secs_f32() > 0.11 {
      match thread_control.try_lock() {
        Ok(mut tc) => {
          //println!("== ==> [global_data_sink] tc lock acquired!");
          if tc.stop_flag {
            tc.thread_data_sink_active = false;
            break;
          } 
          if tc.new_run_start_flag {
            new_run_start         = true;
            write_stream          = tc.write_data_to_disk;
            write_stream_path     = tc.liftof_settings.data_publisher_settings.data_dir.clone(); 
            runid                 = tc.run_id;
            write_stream_path     += &(format!("/{}/", runid));
            tc.new_run_start_flag = false;
          }
        },
        Err(err) => {
          error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
        },
      }
      kill_timer = Instant::now();
    }
    if write_stream && new_run_start {
      //mut streamfile_name = write_stream_path + "/run_";
      //streamfile_name += &runid.to_string();
      let file_type = FileType::RunFile(runid as u32);
      //println!("==> Writing stream to file with prefix {}", streamfile_name);
      writer = Some(TofPacketWriter::new(write_stream_path.clone(), file_type));
      writer.as_mut().unwrap().mbytes_per_file = mbytes_per_file as usize;
      new_run_start = false;
    } else if !write_stream {
      writer = None;
    }
    match incoming.recv() {
      Err(err) => trace!("No new packet, err {err}"),
      Ok(pack) => {
        debug!("Got new tof packet {}", pack.packet_type);
        if writer.is_some() {
          writer.as_mut().unwrap().add_tof_packet(&pack);
          cfg_if::cfg_if!{
            if #[cfg(features="debug")] {
              heartbeat.n_pack_write_disk += 1;
              heartbeat.n_bytes_written += pack.payload.len();   
            }
          }
          n_pack_write_disk += 1;
          bytes_sec_disk    += pack.payload.len() as f64;
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
            info!("Received RBCalibration packet for board {}!", cali_rb_id);
            // we notify the other threads that we got this specific packet, 
            // so we know how long we still have to wait
            match thread_control.lock() {
              Ok(mut tc) => {
                // FIXME - unwrap (for bad packets)
                *tc.finished_calibrations.get_mut(&cali_rb_id).unwrap() = true; 
                cali_expected = tc.n_rbs;
                cali_completed += 1;
                println!("Changed tc {}", tc);
                info!("{} of {} calibrattions completed!", cali_completed, cali_expected);
              },
              Err(err) => {
                error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
              },
            }
            
            // See RBCalibration reference
            let file_type  = FileType::CalibrationFile(cali_rb_id);
            //println!("==> Writing stream to file with prefix {}", streamfile_name);
            //let mut cali_writer = TofPacketWriter::new(write_stream_path.clone(), file_type);
            if !cali_dir_created {
              let today           = get_utc_timestamp();
              cali_output_dir = format!("{}/{}", cali_dir.clone(), today);
              match create_dir_all(cali_output_dir.clone()) {
                Ok(_)    => info!("Created {} for calibration data!", cali_output_dir),
                Err(err) => error!("Unable to create {} for calibration data! {}", cali_output_dir, err)
              }
              cali_dir_created = true;
            }
            if cali_completed == cali_expected {
              cali_completed = 0;
              cali_dir_created = false;
            }
            let mut cali_writer = TofPacketWriter::new(cali_output_dir.clone(), file_type);
            cali_writer.add_tof_packet(&pack);
            drop(cali_writer);
          }
          _ => ()
        }
        if print_moni_packets {
          // some output to the console
          match pack.packet_type {
            PacketType::RBMoniData => {
              //let moni : Result<RBMoniData, SerializationError> = pack.unpack(); 
              match pack.unpack::<RBMoniData>() {
              //match moni {
                Ok(data) => {
                  debug!("Sending RBMoniData {}", data);
                },
                Err(err) => error!("Can not unpack RBMoniData! {err}")
              }
            }, 
            PacketType::CPUMoniData => {
              match pack.unpack::<CPUMoniData>() {
                Ok(data) => {println!("{}", data);},
                Err(err) => error!("Can not unpack TofCmpData! {err}")}
              },
            PacketType::MonitorMtb => {
              //let moni = MtbMoniData::from_bytestream(&pack.payload, &mut pos);
              match pack.unpack::<MtbMoniData>() {
                Ok(data) => {println!("{}", data);},
                Err(err) => error!("Can not unpack MtbMoniData! {err}")}
              },
            _ => ()
          } // end match 
        }
        
        // FIXME - disentangle network and disk I/O?
        if pack.packet_type == PacketType::TofEvent {
          if send_tof_summary_packets ||
             send_rbwaveform_packets {
            // unfortunatly we have to do this unnecessary step
            // I have to think about fast tracking these.
            // maybe sending TofEvents over the channel instead
            // of TofPackets?
            let ev_to_send : TofEvent;
            match pack.unpack::<TofEvent>() {
            //match TofEvent::from_bytestream(&pack.payload, &mut pos) {
              Err(err) => {
                error!("Unable to unpack TofEvent! {err}");
                continue;
              },
              Ok(_ev_to_send) => {
                ev_to_send = _ev_to_send;
              }
            }
            if send_tof_summary_packets {
              let te_summary = ev_to_send.get_summary();
              // debug
              if evid_check.len() < 20000 {
                evid_check.push(te_summary.event_id);
              }
              let pack = TofPacket::from(&te_summary);
              match data_socket.send(pack.to_bytestream(),0) {
                Err(err) => {
                  error!("Packet sending failed! {err}");
                }
                Ok(_)    => {
                  //trace!("Event Summary for event id {} send!", evid);
                  cfg_if::cfg_if!{
                    if #[cfg(features="debug")] {
                      heartbeat.n_pack_sent += 1;
                    }
                  }
                  n_pack_sent += 1;
                }
              }
            }
            if send_rbwaveform_packets {
              for rbwave in ev_to_send.get_rbwaveforms() {
                let pack = TofPacket::from(&rbwave);
                match data_socket.send(pack.to_bytestream(),0) {
                  Err(err) => {
                    error!("Packet sending failed! {err}");
                  }
                  Ok(_)    => {
                    //trace!("RB waveform for event id {} send!", evid);
                    cfg_if::cfg_if!{
                      if #[cfg(features="debug")] {
                        heartbeat.n_pack_sent += 1;
                      }
                    }
                    n_pack_sent += 1;
                  }
                }
              }
            }
            if send_mtb_event_packets {
              let pack = TofPacket::from(&ev_to_send.mt_event);
              match data_socket.send(pack.to_bytestream(),0) {
                Err(err) => {
                  error!("Packet sending failed! {err}");
                }
                Ok(_)    => {
                  //trace!("RB waveform for event id {} send!", evid);
                  cfg_if::cfg_if!{
                    if #[cfg(features="debug")] {
                      heartbeat.n_pack_sent += 1;
                    }
                  }
                  n_pack_sent += 1;
                }
              }
            }
          }
          if send_tof_event_packets {
            match data_socket.send(pack.to_bytestream(),0) {
              Err(err) => error !("Not able to send packet over 0MQ PUB! {err}"),
              Ok(_)    => {
                trace!("TofPacket sent");
                cfg_if::cfg_if!{
                  if #[cfg(features="debug")] {
                    heartbeat.n_pack_sent += 1;
                  }
                }
                n_pack_sent += 1;
              }
            } // end match
          }
        } else {
          match data_socket.send(pack.to_bytestream(),0) {
            Err(err) => error !("Not able to send packet over 0MQ PUB! {err}"),
            Ok(_)    => {
              trace!("TofPacket sent");
              cfg_if::cfg_if!{
                if #[cfg(features="debug")] {
                  heartbeat.n_pack_sent += 1;
                }
              }
              n_pack_sent += 1;
            }
          } // end match
          
        }
      } // end if pk == event packet
    } // end incoming.recv
    if timer.elapsed().as_secs() > 120 {
      let evid_check_len = evid_check.len();
      //println!("DEBUG .1.");
      //let mut evid_test_missing = 0usize;
      let mut evid_missing = 0;
      if evid_check_len > 0 {
        let mut evid = evid_check[0];
        //println!("DEBUG 1.5");
        //println!("len of evid_id_test {}", evid_id_test_len);
        for _ in 0..evid_check_len {
          if !evid_check.contains(&evid) {
            cfg_if::cfg_if!{
              if #[cfg(features="debug")] {
                heartbeat.n_evid_missing += 1;
                heartbeat.n_evid_chunksize = evid_check_len;
              }
            }
          }
          evid += 1;
        }
      }
      cfg_if::cfg_if!{
        if #[cfg(features="debug")] {
          heartbeat.incoming_ch_len = incoming.len();
          heartbeat.met += timer.elapsed().as_secs_u64();
          match data_socket.send(heartbeat.to_bytestream(),0) {
            Err(err) => error!("Not able to send heartbeat over 0MQ PUB! {err}"),
            Ok(_)    => {
              trace!("TofPacket sent");
            }
          } // end match
        }
      } 
      evid_check.clear();
      ////println!("DEBUG 2");
      //event_id_test.clear();
      //println!("DEBUG 3");

      met_time_secs += timer.elapsed().as_secs_f32();
      let packet_rate = n_pack_sent as f32 /met_time_secs;
      println!("  {:<75}", ">> == == == == == == DATA SINK HEARTBEAT  == == == == == == <<".bright_cyan().bold());
      println!("  {:<75} <<", format!(">> ==> Sent {} TofPackets! (packet rate {:.2}/s)", n_pack_sent ,packet_rate).bright_cyan());
      println!("  {:<75} <<", format!(">> ==> Incoming cb channel len {}", incoming.len()).bright_cyan());
      println!("  {:<75} <<", format!(">> ==> Writing events to disk: {} packets written, data write rate {:.2} MB/sec", n_pack_write_disk, bytes_sec_disk/(1e6*met_time_secs as f64)).bright_purple());
      println!("  {:<75} <<", format!(">> ==> Missing evid analysis:  {} of {} a chunk of events missing ({:.2}%)", evid_missing, evid_check_len, 100.0*(evid_missing as f64/evid_check_len as f64)).bright_purple());

      println!("  {:<75}", ">> == == == == == == == == == == == == == == == == == == == <<".bright_cyan().bold());
      timer = Instant::now();
    }
  }
}
