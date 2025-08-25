// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

/// Waveform analysis engine - identify waveform variables
///
/// This will populate the TofHits in an RBEvent
///
/// TofHits contain information about peak location,
/// charge, timing.
///
/// FIXME - I think this should take a HashMap with 
/// algorithm settings, which we can load from a 
/// json file
///
/// # Arguments
///
/// * event       : current RBEvent with waveforms to 
///                 work on
/// * rb          : ReadoutBoard as loaded from the DB, 
///                 with latest calibration attached
/// * settings    : Parameters to configure the waveform
///                 analysis & peak finding
#[cfg(feature="database")]
pub fn waveform_analysis(event         : &mut RBEvent,
                         rb            : &ReadoutBoard,
                         settings      : AnalysisEngineSettings)
-> Result<(), AnalysisError> {
  // Don't do analysis for mangled events!
  if event.has_any_mangling_flag() {
    warn!("Event for RB {} has data mangling! Not doing analysis!", rb.rb_id);
    return Err(AnalysisError::DataMangling);
  }
  match event.self_check() {
    Err(_err) => {
      // Phlip want to ahve all hits even if they are broken
    },
    Ok(_)    => ()
  }
  let active_channels = event.header.get_channels();
  // will become a parameter
  let fit_sinus       = true;
  // allocate memory for the calbration results
  let mut voltages    : Vec<f32>= vec![0.0; NWORDS];
  let mut times       : Vec<f32>= vec![0.0; NWORDS];

  // Step 0 : If desired, fit sine
  let mut fit_result = (0.0f32, 0.0f32, 0.0f32);
  if fit_sinus {
    if !active_channels.contains(&8) {
      warn!("RB {} does not have ch9 data!", rb.rb_id);
      //println!("{}", event.header);
      return Err(AnalysisError::NoChannel9);
    }
    rb.calibration.voltages(9,
                            event.header.stop_cell as usize,
                            &event.adc[8],
                            &mut voltages);
    //warn!("We have to rework the spike cleaning!");
    //match RBCalibrations::spike_cleaning(&mut ch_voltages,
    //                                     event.header.stop_cell) {
    //  Err(err) => {
    //    error!("Spike cleaning failed! {err}");
    //  }
    //  Ok(_)    => ()
    //}
    rb.calibration.nanoseconds(9,
                               event.header.stop_cell as usize,
                               &mut times);
    fit_result                = fit_sine_simple(&voltages, &times);

    //println!("FIT RESULT = {:?}", fit_result);
    //event.header.set_sine_fit(fit_result);
  }

  // structure to store final result
  // extend with Vec<TofHit> in case
  // we want to have multiple hits
  let mut paddles    = HashMap::<u8, TofHit>::new();
  //println!("RBID {}, Paddles {:?}", rb.rb_id ,rb.get_paddle_ids());
  for pid in rb.get_paddle_ids() {
    // cant' fail by constructon of pid
    let ch_a = rb.get_pid_rbchA(pid).unwrap() as usize;
    let ch_b = rb.get_pid_rbchB(pid).unwrap() as usize;
    let mut hit = TofHit::new();
    hit.paddle_id = pid;
    //println!("{ch_a}, {ch_b}, active_channels {:?}", active_channels);
    for (k, ch) in [ch_a, ch_b].iter().enumerate() {
      // Step 1: Calibration
      //println!("Ch {}, event {}", ch, event);
      //println!("---------------------------");
      //println!("pid {}, active channels : {:?}, ch {}",pid, active_channels, ch);
      if !active_channels.contains(&(*ch as u8 -1)) {
        trace!("Skipping channel {} because it is not marked to be readout in the event header channel mask!", ch);
        continue;
      }
      //println!("Will do waveform analysis for ch {}", ch);
      rb.calibration.voltages(*ch,
                              event.header.stop_cell as usize,
                              &event.adc[*ch as usize -1],
                              &mut voltages);
      //FIXME - spike cleaning!
      //match RBCalibrations::spike_cleaning(&mut ch_voltages,
      //                                     event.header.stop_cell) {
      //  Err(err) => {
      //    error!("Spike cleaning failed! {err}");
      //  }
      //  Ok(_)    => ()
      //}
      rb.calibration.nanoseconds(*ch,
                                 event.header.stop_cell as usize,
                                 &mut times);
      // Step 2: Pedestal subtraction
      let (ped, ped_err) = calculate_pedestal(&voltages,
                                              settings.pedestal_thresh,
                                              settings.pedestal_begin_bin,
                                              settings.pedestal_win_bins);
      trace!("Calculated pedestal of {} +- {}", ped, ped_err);
      for n in 0..voltages.len() {
        voltages[n] -= ped;
      }
      let mut charge : f32 = 0.0;
      //let peaks : Vec::<(usize, usize)>;
      let mut cfd_times = Vec::<f32>::new();
      let mut max_volts = 0.0f32;
      // Step 4 : Find peaks
      // FIXME - what do we do for multiple peaks?
      // Currently we basically throw them away
      match find_peaks(&voltages ,
                       &times    ,
                       settings.find_pks_t_start , 
                       settings.find_pks_t_window,
                       settings.min_peak_size    ,
                       settings.find_pks_thresh  ,
                       settings.max_peaks      ) {
        Err(err) => {
          // FIXME - if this happens, most likely the channel is dead. 
          debug!("Unable to find peaks for RB{:02} ch {ch}! Ignoring this channel!", rb.rb_id);
          debug!("We won't be able to calculate timing information for this channel! Err {err}");
        },
        Ok(peaks)  => {
          //peaks = pks;
          // Step 5 : Find tdcs
          //println!("Found {} peaks for ch {}! {:?}", peaks.len(), raw_ch, peaks);
          for pk in peaks.iter() {
            match cfd_simple(&voltages,
                             &times,
                             settings.cfd_fraction,
                             pk.0, pk.1) {
              Err(err) => {
                debug!("Unable to calculate cfd for peak {} {}! {}", pk.0, pk.1, err);
              }
              Ok(cfd) => {
                cfd_times.push(cfd);
              }
            }
            let pk_height = voltages[pk.0..pk.1].iter().max_by(|a,b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less)).unwrap(); 
            max_volts = *pk_height;
            let max_index = voltages.iter().position(|element| *element == max_volts).unwrap();

            let (start_q_int, stop_q_int) = if max_index - 40 < 10 {
              (10, 210)
            } else {
              (max_index - 40, max_index + 160)
            };
          

            //debug!("Check impedance value! Just using 50 [Ohm]");
            // Step 3 : charge integration
            // FIXME - make impedance a settings parameter
            match integrate(&voltages,
                            &times,
                            //settings.integration_start,
                            //settings.integration_window,
                            //pk.0, 
                            //pk.1,
                            start_q_int,
                            stop_q_int,
                            50.0) {
              Err(err) => {
                error!("Integration failed! Err {err}");
              }
              Ok(chrg)   => {
                charge = chrg;
              }
            }
            // // just do the first peak for now
            // let pk_height = voltages[pk.0..pk.1].iter().max_by(|a,b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less)).unwrap(); 
            // max_volts = *pk_height; 
            // //debug!("Check impedance value! Just using 50 [Ohm]");
            // // Step 3 : charge integration
            // // FIXME - make impedance a settings parameter
            // match integrate(&voltages,
            //                 &times,
            //                 //settings.integration_start,
            //                 //settings.integration_window,
            //                 pk.0, 
            //                 pk.1,
            //                 50.0) {
            //   Err(err) => {
            //     error!("Integration failed! Err {err}");
            //   }
              
            break;
          }
        }// end OK
      } // end match find_peaks 
      let mut tdc : f32 = 0.0; 
      if cfd_times.len() > 0 {
        tdc = cfd_times[0];
      }
      //println!("Calucalated tdc {}, charge {}, max {} for ch {}!", tdc, charge, max_volts, ch); 
      //if rb.channel_to_paddle_end_id[*raw_ch as usize] > 2000 {
      if k == 0 {
        //hit.ftime_a      = tdc;
        //hit.fpeak_a      = max_volts;
        hit.set_time_a(tdc);
        hit.set_charge_a(charge);
        hit.set_peak_a(max_volts);
        hit.baseline_a     = f16::from_f32(ped);
        hit.baseline_a_rms = f16::from_f32(ped_err);
      } else {
        //hit.ftime_b = tdc;
        //hit.fpeak_b = max_volts;
        hit.set_time_b(tdc);
        hit.set_charge_b(charge);
        hit.set_peak_b(max_volts);
        hit.baseline_b     = f16::from_f32(ped);
        hit.baseline_b_rms = f16::from_f32(ped_err);
        // this is the seoond iteration,
        // we are done!
        hit.phase = f16::from_f32(fit_result.2);
        paddles.insert(pid, hit);
      }
    }
  }
  let result = paddles.into_values().collect();
  event.hits = result;
  //print ("EVENT {}", event);
  Ok(())
}

