use std::time::{Instant};
use std::{thread, time};
use crossbeam_channel::{Sender,
                        Receiver};
use std::sync::{
    Arc,
    Mutex,
};

use indicatif::{
    MultiProgress,
    ProgressBar,
    ProgressStyle
};

use crate::control::*;
use crate::memory::*;
use crate::api::*;

use tof_dataclasses::events::{DataType};
use tof_dataclasses::commands::{TofOperationMode};
use tof_dataclasses::commands::config::{
  RunConfig
};
//use tof_dataclasses::io::RBEventMemoryStreamer;
//use tof_dataclasses::packets::TofPacket;
//use tof_dataclasses::threading::ThreadControl;

use liftof_lib::settings::{
    RBSettings,
    RBBufferStrategy
};

use liftof_lib::thread_control::ThreadControl;


/// Shutdown a run within the runner thread
fn termination_seqeunce(prog_ev       : &ProgressBar,
                        prog_a        : &ProgressBar,
                        prog_b        : &ProgressBar,
                        show_progress : bool,
                        bs_sender     : &Sender<Vec<u8>>) {
  info!("Calling termination sequence, will end current run!");
  // just to be sure we set the self trigger rate to 0 
  // this is for the poisson trigger)
  match set_self_trig_rate(0) {
    Err(err) => error!("Resetting self trigger rate to 0Hz failed! Err {err}"),
    Ok(_)    => ()
  }
  match disable_trigger() {
    Err(err) => error!("Can not disable triggers, error {err}"),
    Ok(_)    => info!("Disabling triggers! Stopping current run!")
  }
  if show_progress {
    prog_ev.finish();
    prog_a.finish();
    prog_b.finish();
  }
  match ram_buffer_handler(1,
                           &bs_sender) { 
    Err(err)   => {
      error!("Can not deal with RAM buffers {err}");
    },
    Ok(_) => ()
  }
  info!("Termination sequence complete!");
}


/// Thread which controls run start/stop, deals with 
/// runconfigs and dis/enable triggers accordingly
///
///
///  # Arguments
///
///  * run_config     : A channel over which we can pass a RunConfig.
///                     This will either initialize data taking or 
///                     stop it.
/// 
///  * prog_op_ev     : An option for a progress bar which
///                     is helpful for debugging
///  * force_trigger  : Run in forced trigger mode
///
///
pub fn runner(run_config              : &Receiver<RunConfig>,
              bs_sender               : &Sender<Vec<u8>>,
              dtf_to_evproc           : &Sender<DataType>,
              opmode_to_cache         : &Sender<TofOperationMode>,
              show_progress           : bool,
              settings                : &RBSettings, 
              thread_control          : Arc<Mutex<ThreadControl>>) { 
  
  let one_milli        = time::Duration::from_millis(1);
  let one_sec          = time::Duration::from_secs(1);
  let mut first_iter   = true; 
  let mut last_evt_cnt : u32 = 0;
  let mut evt_cnt      = 0u32;
  let mut delta_events : u64;
  let mut n_events     : u64 = 0;
 
  // trigger settings. Per default, we latch to the 
  let mut latch_to_mtb = true;

  let mut timer               = Instant::now();
  // do we have to manually trigger at the desired 
  // time inberval? Then we set force_trigger.
  // The Poisson trigger triggers automatically.
  let mut force_trigger = false;
  let mut time_between_events : Option<f32> = None;
  let met = time::Instant::now();

  // run start/stop conditions
  let mut terminate             = false;
  let mut is_running            = false;
  let mut listen_for_new_config = false;
  let mut rc = RunConfig::new();
  
  // this are all settings for the progress bar
  let mut template_bar_n_ev : &str;
  let mut sty_ev     : ProgressStyle;
  let mut multi_prog : MultiProgress;
  let mut prog_a  = ProgressBar::hidden();
  let mut prog_b  = ProgressBar::hidden();
  let mut prog_ev = ProgressBar::hidden();
  let template_bar_a   : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.blue/grey} {bytes:>7}/{total_bytes:7} ";
  let template_bar_b   : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.green/grey} {bytes:>7}/{total_bytes:7} ";
  let label_a   = String::from("Buff A");
  let label_b   = String::from("Buff B");
  let sty_a = ProgressStyle::with_template(template_bar_a)
  .unwrap();
  let sty_b = ProgressStyle::with_template(template_bar_b)
  .unwrap();
  prog_a.set_position(0);
  prog_b.set_position(0);
  prog_ev.set_position(0);

  let mut which_buff  : RamBuffer;
  let mut buff_size   : usize;
  // set a default of 2000 events in the cache, 
  // but this will be defined in the run params
  let mut buffer_trip : usize = 2000*EVENT_SIZE;
  let mut uio1_total_size = DATABUF_TOTAL_SIZE;
  let mut uio2_total_size = DATABUF_TOTAL_SIZE;
  loop {
    match thread_control.lock() {
      Ok(tc) => {
        if tc.stop_flag {
          info!("Received stop signal. Will stop thread!");
          termination_seqeunce(&prog_ev     ,
                               &prog_a      ,
                               &prog_b      ,
                               show_progress,
                               &bs_sender   );
          break;
        }
      },
      Err(err) => {
        trace!("Can't acquire lock! {err}");
      },
    }
    match run_config.try_recv() {
      Err(err) => {
        trace!("Did not receive a new RunConfig! Err {err}");
        // in this case, we just wait until we get a new run config!
        if listen_for_new_config {
          thread::sleep(one_sec);
          continue;
        }
      }
      Ok(new_config) => {
        // we got a new config. We will proceed with our loop,
        // except the config says run_active = false.
        // In that case, we will end and listen for the next
        // config
        listen_for_new_config = false;
        println!("==> Received a new set of RunConfig\n {}!", new_config);

        // reset some variables for the loop
        first_iter   = true; 
        last_evt_cnt = 0;
        evt_cnt      = 0;
        //delta_events = 0;
        n_events     = 0;
        rc           = new_config;
        // first of all, check if the new run config is active. 
        // if not, stop all triggers
        if !rc.is_active {
          listen_for_new_config = true;
          termination_seqeunce(&prog_ev     ,
                               &prog_a      ,
                               &prog_b      ,
                               show_progress,
                               &bs_sender   );
          continue;
        }
        // we have an active run, initializing
        terminate = false;

        // deal with the individual settings:
        // first buffer size
        info!("Setting buffer trip value to {}", buffer_trip);
        buffer_trip = (rc.rb_buff_size as usize)*EVENT_SIZE; 
        if (buffer_trip > DATABUF_TOTAL_SIZE) 
        || (buffer_trip > DATABUF_TOTAL_SIZE) {
          error!("Tripsize of {buffer_trip} exceeds buffer sizes of A : {uio1_total_size} or B : {uio2_total_size}. The EVENT_SIZE is {EVENT_SIZE}");
          warn!("Will set buffer_trip to {DATABUF_TOTAL_SIZE}");
          buffer_trip = DATABUF_TOTAL_SIZE;
        } else {
          uio1_total_size = buffer_trip;
          uio2_total_size = buffer_trip;
        }
        
        match opmode_to_cache.send(rc.tof_op_mode.clone()) {
          Err(err) => {
            error!("Unable to send TofOperationMode to the event cache! Err {err}");
          }
          Ok(_)    => {
            info!("Send TofOperationMode {} to event processing thread!", rc.tof_op_mode);
          }
        }

        let dt_c = rc.data_type.clone();
        match dtf_to_evproc.send(dt_c) {
          Err(err) => {
            error!("Unable to send dataformat to event processing thread! {err}");
          }
          Ok(_) => {
            info!("Sent dataformat {} to event processing thread!", rc.data_type);
          }
        }

        // data type
        match rc.data_type {
          DataType::VoltageCalibration | 
          DataType::TimingCalibration  | 
          DataType::Noi                |
          DataType::RBTriggerPoisson   | 
          DataType::RBTriggerPeriodic =>  {
            latch_to_mtb = false;
          },
          _ => ()
        }
        if rc.trigger_poisson_rate > 0 {
          latch_to_mtb = false;
          // we also activate the poisson trigger
          //enable_poisson_self_trigger(rc.trigger_poisson_rate as f32);
        }
        if rc.trigger_fixed_rate>0 {
          force_trigger = true;
          time_between_events = Some(1.0/(rc.trigger_fixed_rate as f32));
          warn!("Will run in forced trigger mode with a rate of {} Hz!", rc.trigger_fixed_rate);
          debug!("Will call trigger() every {} seconds...", time_between_events.unwrap());
          latch_to_mtb = false;
        }
        match disable_trigger() {
          Err(err) => error!("Can not disable triggers! {err}"),
          Ok(_)    => info!("Triggers disabled awaiting SOFT_RESET!"),
        }
        info!("Attempting soft reset...");
        match soft_reset_board() {
          Err(err) => error!("Unable to reset board! {err}"),
          Ok(_)    => info!("SOFT_RESET succesful!"),
        }
        // preparations done, let's gooo
        //reset_dma_and_buffers();

        if latch_to_mtb {
          match set_master_trigger_mode() {
            Err(err) => error!("Can not initialize master trigger mode, Err {err}"),
            Ok(_)    => info!("Latching to MasterTrigger")
          }
        } else {
          match disable_master_trigger_mode() {
            Err(err) => error!("Can not disable master trigger mode, Err {err}"),
            Ok(_)    => info!("Master trigger mode didsabled!")
          }
        }
        
        // this basically signals "RUNSTART"
        match enable_trigger() {
          Err(err) => error!("Can not enable triggers! Err {err}"),
          Ok(_)    => info!("Triggers enabled - Run start!")
        }
        if rc.trigger_poisson_rate > 0 {
          enable_poisson_self_trigger(rc.trigger_poisson_rate as f32);
        }
        // FIXME - only if above call Ok()
        is_running = true;

        if !force_trigger {
          // we relax and let the system go 
          // for a short bit
          thread::sleep(one_sec);
          match get_trigger_rate() {
            Err(err) => error!("Unable to obtain trigger rate! Err {err}"),
            Ok(rate) => info!("Seing MTB trigger rate of {rate} Hz")
          }
        }
        if show_progress {
          if rc.nevents == 0 {
            template_bar_n_ev = "[{elapsed_precise}] {prefix} {msg} {spinner} ";
          } else {
            template_bar_n_ev = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.red/grey} {pos:>7}/{len:7}";
          }
          sty_ev = ProgressStyle::with_template(template_bar_n_ev)
          .unwrap();
          multi_prog = MultiProgress::new();
          prog_a  = multi_prog
                    .add(ProgressBar::new(uio1_total_size as u64)); 
          prog_b  = multi_prog
                    .insert_after(&prog_a, ProgressBar::new(uio2_total_size as u64)); 
          prog_ev = multi_prog
                    .insert_after(&prog_b, ProgressBar::new(rc.nevents as u64)); 
          prog_a.set_message (label_a.clone());
          prog_a.set_prefix  ("\u{1F4BE}");
          prog_a.set_style   (sty_a.clone());
          prog_a.set_position(0);
          prog_b.set_message (label_b.clone());
          prog_b.set_prefix  ("\u{1F4BE}");
          prog_b.set_style   (sty_b.clone());
          prog_b.set_position(0);
          prog_ev.set_style  (sty_ev.clone());
          prog_ev.set_prefix ("\u{2728}");
          prog_ev.set_message("EVENTS");
          prog_ev.set_position(0);
          info!("Preparations complete. Run start should be imminent.");
        }
        continue; // start loop again
      } // end Ok(RunConfig) 
    } // end run_params.try_recv()

    if is_running {
      if terminate {
        is_running = false;
        termination_seqeunce(&prog_ev     ,
                             &prog_a      ,
                             &prog_b      ,
                             show_progress,
                             &bs_sender   );
        info!("Run stopped! The runner has processed {n_events} events!");
        continue;
      } // end if terminate
      
      // We did not terminate the run,
      // that means we are still going!
      if force_trigger {
        //println!("Forcing trigger!");
        //println!("Time between events {}", time_between_events.unwrap());
        let elapsed = timer.elapsed().as_secs_f32();
        //println!("Elapsed {}", elapsed);
        trace!("Forced trigger mode, {} seconds since last trigger", elapsed);
        // decide if we have to issue the trigger signal NOW!
        if elapsed > time_between_events.unwrap() {
          timer = Instant::now(); 
          match trigger() {
            Err(err) => error!("Error when triggering! {err}"),
            Ok(_)    => trace!("Firing trigger!")
          }
        } else { // not enough time has yet passed for the next trigger signal
          // FIXME - we could sleep here for a bit!
          continue;
        }
      }    

      // calculate current event count
      if !force_trigger {
        // this checks if we have seen a new event
        //match get_event_count_mt() {
        match get_event_count() {
          Err (err) => {
            error!("Can not obtain event count! Err {:?}", err);
            continue;
          }
          Ok (cnt) => {
            evt_cnt = cnt;
            if first_iter {
              last_evt_cnt = evt_cnt;
              first_iter = false;
              continue;
            }
            if evt_cnt == last_evt_cnt {
              thread::sleep(one_milli);
              trace!("We didn't get an updated event count!");
              continue; // only continue after we see a new event!
            }
          } // end ok
        } // end match
      } // end force trigger

      // AT THIS POINT WE KNOW WE HAVE SEEN SOMETHING!!!
      // THIS IS IMPORTANT
      match ram_buffer_handler(buffer_trip,
                               &bs_sender) { 
        Err(err)   => {
          error!("Can not deal with RAM buffers {err}");
          continue;
        }
        Ok(result) => {
          which_buff = result.0;
          buff_size  = result.1;
          if result.2 { // buffer has tripped
            // in case we chose a dynamic buffer strategy, 
            // adapt the buffer size for the next time
            match settings.rb_buff_strategy {
              // first case we have a fixed buffer size
              RBBufferStrategy::NEvents(_) => (),
              RBBufferStrategy::AdaptToRate(n_secs) => {
                match get_trigger_rate() {
                  Err(err) => {
                    error!("Unable to obtain trigger rate! {err}");
                    // FIXME - Reasonable default?
                    buffer_trip = 50;
                  },
                  Ok(rate) => {
                    buffer_trip = rate as usize*n_secs as usize*EVENT_SIZE ;
                    trace!("Dynamic setting of buffer trip size for rate {} and n_secs {}! Setting buffer trip size to {}",rate, n_secs, buffer_trip);
                  }
                }
              }
            }
            // check again if buffer trip exceeds total size
            if (buffer_trip > DATABUF_TOTAL_SIZE) 
            || (buffer_trip > DATABUF_TOTAL_SIZE) {
              error!("Tripsize of {buffer_trip} exceeds buffer sizes of A : {uio1_total_size} or B : {uio2_total_size}. The EVENT_SIZE is {EVENT_SIZE}");
              warn!("Will set buffer_trip to {DATABUF_TOTAL_SIZE}");
              buffer_trip = DATABUF_TOTAL_SIZE;
            } else {
              uio1_total_size = buffer_trip;
              uio2_total_size = buffer_trip;
            }
            if show_progress {
              prog_a.set_length(uio1_total_size as u64);
              prog_b.set_length(uio2_total_size as u64);
            }
          }
        }
      }
      if force_trigger {
          n_events += 1;
      } else {
        delta_events = (evt_cnt - last_evt_cnt) as u64;
        n_events    += delta_events;
        last_evt_cnt = evt_cnt;
      }
      if show_progress {
        match which_buff {
          RamBuffer::A => {
            prog_a.set_position(buff_size as u64);
            prog_b.set_position(0);
          }
          RamBuffer::B => {
            prog_b.set_position(buff_size as u64);
            prog_a.set_position(0);
          }
        }
        prog_ev.set_position(n_events);
      }
    } // end is_running
    
    // from here on, check termination 
    // conditions
    if rc.nseconds > 0 {
      if met.elapsed().as_secs() > rc.nseconds  as u64{
        terminate = true;
      }
    }
    
    // FIXME
    if !rc.nevents == 0 {
      if rc.nevents != 0 {
        if n_events > rc.nevents as u64{
          terminate = true;
        }
      }
      
      if rc.nseconds > 0 {
          if met.elapsed().as_secs() > rc.nseconds  as u64{
            terminate = true;
          }
        }

      // reduce cpu load
      if !terminate {
        if !force_trigger { 
          thread::sleep(100*one_milli);
        }
      }
    }
  } // end loop
}



