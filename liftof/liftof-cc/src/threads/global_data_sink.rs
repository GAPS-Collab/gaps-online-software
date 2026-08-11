//! Global data sink - a 'funnel' for all packets
//! generated through the liftof system.
//!
//! Each thread of liftof-cc can connect to the 
//! data sink through a channel and it will 
//! forward the tof packets to the designated
//! zmq socket.
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

use crossbeam_channel::Receiver; 

use gondola_core::prelude::*;

/// Manages "outgoing" 0MQ PUB socket and writing
/// data to disk
///
/// All received packets will be either forwarded
/// over zmq or saved to disk
///
/// # Arguments:
///   * incoming       : incoming connection for TofPackets
///                      from any source
///   * thread_control : inter-thread communications,
///                      start/stop signals.
///                      Keeps global settings.
pub fn global_data_sink(incoming       : &Receiver<TofPacket>,
                        incoming_ev    : &Receiver<TofEvent>,
                        thread_control : Arc<Mutex<ThreadControl>>) {
  // when the thread starts, we need to wait a bit
  // till thread_control becomes usable
  sleep(Duration::from_secs(10));
  let mut flight_address           = String::from("");
  let mut mbytes_per_file          = 420usize;
  let mut write_stream_path        = String::from("");
  let mut send_tof_summary_packets = false;
  let mut send_rbwaveform_packets  = false;
  //let mut send_mtb_event_packets   = false;
  let mut write_stream             = false;
  let mut send_rbwf_every_x_event  = 1;
  // fixme - smaller hb interfal
  let mut hb_interval              = Duration::from_secs(20u64);
  let mut only_save_interesting    = false;
  //let mut thr_n_hits_umb           = 0u8;         
  let mut thr_n_hits_cbe           = 0u8;
  let mut thr_n_hits_outer         = 0u8;
  //let mut thr_n_hits_cor           = 0u8;
  //let mut thr_tot_edep_umb         = 0.0f32;
  let mut thr_tot_edep_cbe         = 0.0f32;
  let mut thr_tot_edep_outer       = 0.0f32;
  //let mut thr_tot_edep_cor         = 0.0f32;
  match thread_control.lock() {
    Ok(mut tc) => {
      tc.thread_data_sink_active = true; 
      flight_address             = tc.liftof_settings.data_publisher_settings.fc_pub_address.clone();
      mbytes_per_file            = tc.liftof_settings.data_publisher_settings.mbytes_per_file; 
      write_stream_path          = tc.liftof_settings.data_publisher_settings.data_dir.clone();
      write_stream               = tc.write_data_to_disk;
      send_tof_summary_packets   = tc.liftof_settings.data_publisher_settings.send_tof_summary_packets;
      send_rbwaveform_packets    = tc.liftof_settings.data_publisher_settings.send_rbwaveform_packets;
      send_rbwf_every_x_event    = tc.liftof_settings.data_publisher_settings.send_rbwf_every_x_event;
      only_save_interesting      = tc.liftof_settings.event_builder_settings.only_save_interesting;
      //thr_n_hits_umb             = tc.liftof_settings.event_builder_settings.thr_n_hits_umb  ;         
      thr_n_hits_cbe             = tc.liftof_settings.event_builder_settings.thr_n_hits_cbe.unwrap_or(0)  ;
      thr_n_hits_outer           = tc.liftof_settings.event_builder_settings.thr_n_hits_outer.unwrap_or(0);
      //thr_n_hits_cor             = tc.liftof_settings.event_builder_settings.thr_n_hits_cor  ;
      //thr_tot_edep_umb           = tc.liftof_settings.event_builder_settings.thr_tot_edep_umb;
      thr_tot_edep_outer         = tc.liftof_settings.event_builder_settings.thr_tot_edep_outer.unwrap_or(0.0);
      thr_tot_edep_cbe           = tc.liftof_settings.event_builder_settings.thr_tot_edep_cbe.unwrap_or(0.0);
      //thr_tot_edep_cor           = tc.liftof_settings.event_builder_settings.thr_tot_edep_cor;
      hb_interval                = Duration::from_secs(tc.liftof_settings.data_publisher_settings.hb_send_interval as u64);
    },
    Err(err) => {
      error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
    },
  }
  
  if send_rbwf_every_x_event == 0 {
    error!("0 is not a reasonable value for send_rbwf_every_x_event!. We will switch of the sending of RBWaveforms instead!");
    send_rbwaveform_packets = false;
  }

  let mut evid_check        = Vec::<u32>::new();

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

  // for debugging/profiling
  let mut timer                = Instant::now();
  let mut check_settings_timer = Instant::now();

  // run settings 
  let mut writer : Option<TofPacketWriter> = None;
  let mut runid : u32   = 0;
  let mut new_run_start = false;
  let mut retire        = false;
  let mut heartbeat     = DataSinkHB::new();
  let mut hb_timer      = Instant::now(); 
  let mut rbwf_ctr      = 0u32;
  loop {
    if retire {
      // take a long nap to give other threads 
      // a chance to finish first
      warn!("Will end data sink thread in 25 seconds!");
      println!("= =>Will end data sink thread in 25 seconds!");
      sleep(Duration::from_secs(25));
      break;
    }
    // even though this is called kill timer, check
    // the settings in general, since they might have
    // changed due to remote access.
    if check_settings_timer.elapsed().as_secs_f32() > 1.5 {
      match thread_control.try_lock() {
        Ok(mut tc) => {
          send_tof_summary_packets = tc.liftof_settings.data_publisher_settings.send_tof_summary_packets;
          send_rbwaveform_packets  = tc.liftof_settings.data_publisher_settings.send_rbwaveform_packets; 
          if tc.stop_flag {
            tc.thread_data_sink_active = false;
            // we want to make sure that data sink ends the latest
            retire = true;
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
          error!("Can't acquire lock for ThreadControl! {err}");
        },
      }
      check_settings_timer = Instant::now();
    }
    if write_stream && new_run_start {
      let file_type = FileType::RunFile(runid as u32);
      //println!("==> Writing stream to file with prefix {}", streamfile_name);
      writer = Some(TofPacketWriter::new(write_stream_path.clone(), file_type));
      writer.as_mut().unwrap().mbytes_per_file = mbytes_per_file as usize;
      new_run_start = false;
    } else if !write_stream {
      writer = None;
    }

    match incoming.try_recv() {
      Err(err) => trace!("No new packet, err {err}"),
      Ok(pack) => {
        debug!("Got new tof packet {}", pack.packet_type);
        // if it is not some event type, just write it to disk
        if writer.is_some() {
          if !pack.no_write_to_disk {
            writer.as_mut().unwrap().add_tof_packet(&pack);
            heartbeat.n_pack_write_disk += 1;
            heartbeat.n_bytes_written += pack.payload.len() as u64;   
          }
        }
        match data_socket.send(pack.to_bytestream(),0) {
          Err(err) => error !("Not able to send packet over 0MQ PUB! {err}"),
          Ok(_)    => {
            trace!("TofPacket sent");
            heartbeat.n_packets_sent += 1;
          }
        } // end match
      } 
    } 
    match incoming_ev.recv() {
      Err(err) => trace!("Unable to receive event packet! {err}"),
      Ok(mut ev_)  => { 
        // FIXME - we need to work on the system to 
        // select how to send waveforms or not
        ev_.calc_gcu_variables();
        let mut write_to_disk = true; 
        if only_save_interesting {
          //if ev_.n_hits_umb   < thr_n_hits_umb 
          if ev_.n_hits_cbe     < thr_n_hits_cbe
          && ev_.n_hits_cor + ev_.n_hits_umb   < thr_n_hits_outer
          //&& ev_.n_hits_cor   < thr_n_hits_cor
          //&& ev_.tot_edep_umb < thr_tot_edep_umb
          && ev_.tot_edep_cbe   < thr_tot_edep_cbe
          && ev_.tot_edep_cor + ev_.tot_edep_umb  < thr_tot_edep_outer { 
          //&& ev_.tot_edep_cor < thr_tot_edep_cor {
            write_to_disk = false;
          }
        }
        let mut pack = ev_.pack();
        if writer.is_some() && write_to_disk {
          writer.as_mut().unwrap().add_tof_packet(&pack);
          heartbeat.n_pack_write_disk += 1;
          heartbeat.n_bytes_written += pack.payload.len() as u64;   
        }

        if send_rbwaveform_packets {
          if rbwf_ctr % send_rbwf_every_x_event == 0 {
            for wf in ev_.get_waveforms() { 
              match data_socket.send(wf.pack().to_bytestream(),0) {
                Err(err) => error !("Not able to send packet over 0MQ PUB! {err}"),
                Ok(_)    => {
                  trace!("TofPacket sent");
                  heartbeat.n_packets_sent += 1;
                }
              } // end match
            }
          }
          rbwf_ctr += 1;
        }

        // Now move hits, strip waveforms and calculate gcu variables 
        ev_.move_hits();
        
        //--------------------------------
        // DEBUG - have the cake and eat it too!
        // This will not send waveforms over 
        // the gcu (because of protocolversion :: V3
        // but also not strip the RBEvents
        //ev_.strip_rbevents();
        //ev_.version = ProtocolVersion::V3;
        //--------------------------------
       
        // this is that the events go over the gcu
        ev_.strip_rbevents();
        ev_.version = ProtocolVersion::V1;
        pack = ev_.pack();
        if send_tof_summary_packets {
          match data_socket.send(pack.to_bytestream(),0) {
            Err(err) => error !("Not able to send packet over 0MQ PUB! {err}"),
            Ok(_)    => {
              trace!("TofPacket sent");
              heartbeat.n_packets_sent += 1;
            }
          } // end match
        }
      } // end if pk == event packet
    } // end incoming.recv

    let evid_check_len = evid_check.len();
    if timer.elapsed().as_secs() > 10 {
      // FIXME - might be too slow?
      if evid_check_len > 0 {
        let mut evid = evid_check[0];
        for _ in 0..evid_check_len {
          if !evid_check.contains(&evid) {
            heartbeat.n_evid_missing += 1;
            heartbeat.n_evid_chunksize = evid_check_len as u64;
          }
          evid += 1;
        }
      }
      timer = Instant::now();
    }
    if hb_timer.elapsed() >= hb_interval {
      heartbeat.met += hb_timer.elapsed().as_secs();
      
      match data_socket.send(heartbeat.pack().to_bytestream(),0) {
        Err(err) => error!("Not able to send heartbeat over 0MQ PUB! {err}"),
        Ok(_)    => {
          trace!("Heartbeat sent");
        }
      } 
      evid_check.clear();
      hb_timer = Instant::now();
    }
  } //end loop
} //end function
