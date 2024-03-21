//! ReadoutBoard Status tab
//!
//! Find connected ReadoutBoards and show their 
//! details as well as the last waveforms
//!

use std::time::Instant;
use std::fs;
use std::collections::VecDeque;

//extern crate histo;
//use histo::Histogram;
use ndhistogram::{
    Histogram,
    Hist1D,
    ndhistogram,
};
use ndhistogram::axis::{
    Uniform,
};

use ratatui::{
    //backend::CrosstermBackend,
    terminal::Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Line},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

use crossbeam_channel::{
    Receiver
};


use tof_dataclasses::packets::{TofPacket, PacketType};
use tof_dataclasses::calibrations::RBCalibrations;
use tof_dataclasses::errors::SerializationError;
use tof_dataclasses::events::RBEvent;
use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::monitoring::RBMoniData;
use tof_dataclasses::io::RBEventMemoryStreamer;
use crate::widgets::{
    timeseries,
    //histogram,
};
use crate::colors::{
  ColorTheme2,
};

#[derive(Debug, Copy, Clone)]
pub enum RBTabView {
  Info,
  Waveform,
  RBMoniData,
  SelectRB,
}

#[derive(Debug, Clone)]
pub struct RBTab<'a>  {
  pub tp_receiver    : Receiver<TofPacket>,
  pub rb_receiver    : Receiver<RBEvent>,
  pub rb_selector    : u8,
  pub rb_changed     : bool,
  pub rb_calibration : RBCalibrations,
  pub cali_loaded    : bool,
  pub event_queue    : VecDeque<RBEvent>,
  pub moni_queue     : VecDeque<RBMoniData>,
  pub met_queue      : VecDeque<f64>,
  pub met_queue_moni : VecDeque<f64>,
  /// Holds waveform data
  pub ch_data        : Vec<Vec<(f64,f64)>>,
  /// Holds the monitoring qunatities
  pub rate_queue     : VecDeque<(f64,f64)>,
  pub fpgatmp_queue  : VecDeque<(f64,f64)>,
  
  pub pressure       : VecDeque<(f64,f64)>,
  pub humidity       : VecDeque<(f64,f64)>,
  
  pub mag_x          : VecDeque<(f64,f64)>,
  pub mag_y          : VecDeque<(f64,f64)>,
  pub mag_z          : VecDeque<(f64,f64)>,
  pub mag_tot        : VecDeque<(f64,f64)>,

  pub queue_size     : usize,
  
  pub n_events       : usize,
  pub n_moni         : usize,
  pub miss_evid      : usize,
  pub last_evid      : u32,
  pub nch_histo      : Hist1D<Uniform<f32>>,
  timer              : Instant,

  pub theme          : ColorTheme2,
  pub view           : RBTabView,

  // list for the rb selector
  pub rbl_state      : ListState,
  pub rbl_items      : Vec::<ListItem<'a>>,
  pub rbl_active     : bool,
}

impl RBTab<'_>  {

  pub fn new(tp_receiver : Receiver<TofPacket>,
             rb_receiver : Receiver<RBEvent>,
             theme       : ColorTheme2) -> RBTab<'static>  {
    let mut rb_select_items = Vec::<ListItem>::new();
    for k in 1..51 {
      let this_item = format!("  RB{:0>2}", k);
      rb_select_items.push(ListItem::new(Line::from(this_item)));
    }

    let queue_size = 1000usize;
    let mut ch_data    = Vec::<Vec::<(f64,f64)>>::with_capacity(1024);
    for _channel in 0..9 {
      let tmp_vec = vec![(0.0f64,0.0f64);1024];
      //ch_data.push(Vec::<(f64,f64)>::new());
      ch_data.push(tmp_vec);
    }
    let bins = Uniform::new(50,-0.5,49.5);
    RBTab {
      tp_receiver    : tp_receiver,
      rb_receiver    : rb_receiver,
      rb_selector    : 0,
      rb_changed     : false,
      rb_calibration : RBCalibrations::new(0),
      cali_loaded    : false,
      event_queue    : VecDeque::<RBEvent>::with_capacity(queue_size),
      moni_queue     : VecDeque::<RBMoniData>::with_capacity(queue_size),
      met_queue      : VecDeque::<f64>::with_capacity(queue_size),
      met_queue_moni : VecDeque::<f64>::with_capacity(queue_size),
      rate_queue     : VecDeque::<(f64,f64)>::with_capacity(queue_size),
      fpgatmp_queue  : VecDeque::<(f64,f64)>::with_capacity(queue_size),
      pressure       : VecDeque::<(f64,f64)>::with_capacity(queue_size),
      humidity       : VecDeque::<(f64,f64)>::with_capacity(queue_size),
      mag_x          : VecDeque::<(f64,f64)>::with_capacity(queue_size),
      mag_y          : VecDeque::<(f64,f64)>::with_capacity(queue_size),
      mag_z          : VecDeque::<(f64,f64)>::with_capacity(queue_size),
      mag_tot        : VecDeque::<(f64,f64)>::with_capacity(queue_size),

      ch_data        : ch_data,

      queue_size     : queue_size,
      
      n_events       : 0,
      n_moni         : 0,
      miss_evid      : 0,
      last_evid      : 0,
      nch_histo      : ndhistogram!(bins),
      timer          : Instant::now(),
  
      theme          : theme,
      view           : RBTabView::Waveform,
    
      rbl_state      : ListState::default(),
      rbl_items      : rb_select_items,
      rbl_active     : false,
    }
  }
  
  pub fn receive_packet(&mut self) -> Result<(), SerializationError> {
    let met    = self.timer.elapsed().as_secs_f64();
    let mut ev = RBEvent::new();
    let bins   = Uniform::new(50,-0.5,49.5);
    
    if self.rb_changed {
      // currently, only one RB at a time is supported
      self.moni_queue.clear();
      self.rate_queue.clear();
      self.pressure.clear();
      self.humidity.clear();
      self.mag_x.clear();
      self.mag_y.clear();
      self.mag_z.clear();
      self.mag_tot.clear();
      self.event_queue.clear();
      self.met_queue.clear();
      self.met_queue_moni.clear();
      self.fpgatmp_queue.clear();
      self.nch_histo = ndhistogram!(bins);
      // try to get a new calibration
      match self.rbl_state.selected() {
        None => {
          self.cali_loaded = false;
        }, 
        Some(_rb_id) => {
          let cali_path = format!("calibrations/rb_{:02}.cali.tof.gaps", _rb_id + 1);
          if fs::metadata(cali_path.clone()).is_ok() {
            match RBCalibrations::from_file(cali_path.clone()) {
              Err(err) => error!("Unable to load RBCalibration from file {}! {err}", cali_path),
              Ok(cali) => {
                self.rb_calibration = cali;
                self.cali_loaded    = true;
              }
            } 
          } else {
            self.cali_loaded = false;
          }
        }
      }
    }

    if !self.rb_receiver.is_empty() {
      match self.rb_receiver.try_recv() {
        Err(_) => (),
        Ok(_ev)   => {
          ev = _ev;
        }
      }
    }

    if !self.tp_receiver.is_empty() {
      match self.tp_receiver.try_recv() {
        Err(_err) => (),
        Ok(pack)    => {
          debug!("Got next packet {}!", pack);
          match pack.packet_type {
            PacketType::RBMoni   => {
              trace!("Got new RBMoniData!");
              let moni = RBMoniData::from_bytestream(&pack.payload, &mut 0)?;
              self.n_moni += 1;
              if moni.board_id == self.rb_selector {
                self.met_queue_moni.push_back(met); 
                if self.met_queue_moni.len() > self.queue_size {
                  self.met_queue_moni.pop_front();
                }
                self.moni_queue.push_back(moni);
                if self.moni_queue.len() > self.queue_size {
                  self.moni_queue.pop_front();
                }
                self.rate_queue.push_back((met, moni.rate as f64));
                if self.rate_queue.len() > self.queue_size {
                  self.rate_queue.pop_front();
                }
                
                self.pressure.push_back((met, moni.pressure as f64));
                if self.pressure.len() > self.queue_size {
                  self.pressure.pop_front();
                }
                self.humidity.push_back((met, moni.humidity as f64));
                if self.humidity.len() > self.queue_size {
                  self.humidity.pop_front();
                }
                self.mag_x.push_back((met, moni.mag_x as f64));
                if self.mag_x.len() > self.queue_size {
                  self.mag_x.pop_front();
                }
                self.mag_y.push_back((met, moni.mag_y as f64));
                if self.mag_y.len() > self.queue_size {
                  self.mag_y.pop_front();
                }
                self.mag_z.push_back((met, moni.mag_z as f64));
                if self.mag_z.len() > self.queue_size {
                  self.mag_z.pop_front();
                }
                self.mag_tot.push_back((met, moni.get_mag_tot() as f64));
                if self.mag_tot.len() > self.queue_size {
                  self.mag_tot.pop_front();
                }
              }
              return Ok(());
            },
            PacketType::RBEvent => {
              ev = RBEvent::from_bytestream(&pack.payload, &mut 0)?;
            },
            PacketType::RBEventMemoryView => {
              let mut streamer = RBEventMemoryStreamer::new();
              //println!("{:?}",&pack.payload[0..10]);
              streamer.add(&pack.payload, pack.payload.len());
              match streamer.get_event_at_pos_unchecked(None) {
                None => {
                  error!("Not able to obtain RBEvent from RBEventMemoryView packet!");
                  return Ok(());
                }
                Some(_ev) => {
                  ev = _ev;
                }
              }
            },
            _ => (),
          }
        }
      }
    }
   
    if ev.header.event_id != 0 && self.rb_selector == ev.header.rb_id {
      for ch in ev.header.get_channels() {
        if self.cali_loaded {
          let mut nanos = vec![0f32;1024];
          let mut volts = vec![0f32;1024];
          self.rb_calibration.nanoseconds(ch as usize + 1, ev.header.stop_cell as usize, 
                                          &mut nanos);
          self.rb_calibration.voltages(ch as usize + 1, ev.header.stop_cell as usize, 
                                       &ev.adc[ch as usize], &mut volts);
          //let 
          for k in 0..nanos.len() {
            let vals = (nanos[k] as f64, volts[k] as f64);
            self.ch_data[ch as usize][k] = vals;
          }
        } else {
          for k in 0..ev.adc[ch as usize].len() {
            let vals = (k as f64, ev.adc[ch as usize][k] as f64);
            self.ch_data[ch as usize][k] = vals;
          }
          //println!("{:?}", self.ch_data[ch as usize]);
        }
      }

      self.nch_histo.fill(&(ev.header.get_nchan() as f32));
      self.n_events += 1;
      if self.last_evid != 0 {
        if ev.header.event_id - self.last_evid != 1 {
          self.miss_evid += (ev.header.event_id - self.last_evid) as usize;
        }
      }
      self.last_evid = ev.header.event_id;
      self.fpgatmp_queue.push_back((met, ev.header.get_fpga_temp() as f64));
      if self.fpgatmp_queue.len() > self.queue_size {
        self.fpgatmp_queue.pop_front();
      }
      self.event_queue.push_back(ev);
      if self.event_queue.len() > self.queue_size {
        self.event_queue.pop_front();
      }
      self.met_queue.push_back(met);
      if self.met_queue.len() > self.queue_size {
        self.met_queue.pop_front();
      }
    }
    Ok(())
  }
  
  pub fn next_rb(&mut self) {
    let i = match self.rbl_state.selected() {
      Some(i) => {
        if i >= self.rbl_items.len() - 1 {
          self.rbl_items.len() - 1
        } else {
          i + 1
        }
      }
      None => 0,
    };
    self.rbl_state.select(Some(i));
  }

  pub fn previous_rb(&mut self) {
    let i = match self.rbl_state.selected() {
      Some(i) => {
        if i == 0 {
          0 
        } else {
          i - 1
        }
      }
      None => 0,
    };
    self.rbl_state.select(Some(i));
  }

  pub fn unselect_rbl(&mut self) {
    self.rbl_state.select(None);
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    match self.view {
      RBTabView::SelectRB => {
        let list_chunks = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
          )
          .split(*main_window);
        let par_title_string = String::from("Select ReadoutBoard (RB)");
        let (first, rest) = par_title_string.split_at(1);
        let par_title = Line::from(vec![
          Span::styled(
              first,
              Style::default()
                  .fg(self.theme.hc)
                  .add_modifier(Modifier::UNDERLINED),
          ),
          Span::styled(rest, self.theme.style()),
        ]);
        let rbs = Block::default()
          .borders(Borders::ALL)
          .style(self.theme.style())
          .title(par_title)
          .border_type(BorderType::Plain);
        let rb_select_list = List::new(self.rbl_items.clone()).block(rbs)
          .highlight_style(self.theme.highlight().add_modifier(Modifier::BOLD))
          .highlight_symbol(">>")
          .repeat_highlight_symbol(true);
        match self.rbl_state.selected() {
          None    => {
            let selector =  1;
            if self.rb_selector != selector {
              self.rb_changed = true;
              self.rb_selector = selector;
            } else {
              self.rb_changed = false;
            }
          },
          Some(_rbid) => {
            let selector =  _rbid as u8 + 1;
            if self.rb_selector != selector {
              self.rb_changed = true;
              self.rb_selector = selector;
            } else {
              self.rb_changed = false;
            }
          },
        }
        frame.render_stateful_widget(rb_select_list, list_chunks[0], &mut self.rbl_state );
      },
      RBTabView::Waveform => {
        // set up general layout
        let status_chunks = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(30), Constraint::Percentage(70)].as_ref(),
          )
          .split(*main_window);

        let detail_and_ch9_chunks = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(25),
               Constraint::Percentage(75)].as_ref(),
          )
          .split(status_chunks[0]);

        let wf_chunks = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(50),
               Constraint::Percentage(50)].as_ref(),
          )
          .split(status_chunks[1]);

        let mut ch_chunks = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(25),
               Constraint::Percentage(25),
               Constraint::Percentage(26),
               Constraint::Percentage(25)].as_ref(),
          )
          .split(wf_chunks[0]).to_vec();

        let mut ch_chunks_2 = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(25),
               Constraint::Percentage(25),
               Constraint::Percentage(26),
               Constraint::Percentage(25)].as_ref(),
          )
          .split(wf_chunks[1]).to_vec();

        ch_chunks.append(&mut ch_chunks_2);
        // the waveform plots
        for ch in 0..9 {
          let label          = format!("Ch{}", ch);
          let ch_tc_theme    = self.theme.clone();
          let mut ch_ts_data = VecDeque::from(self.ch_data[ch].clone());
          let ch_ts = timeseries(&mut ch_ts_data,
                                 label.clone(),
                                 label.clone(),
                                 &ch_tc_theme  );
          // render it!
          if ch == 8 {
            //frame.render_widget(chart, detail_and_ch9_chunks[0]);
            frame.render_widget(ch_ts,detail_and_ch9_chunks[0]);
          } else {
            frame.render_widget(ch_ts,ch_chunks[ch]);
            //frame.render_widget(chart, ch_chunks[ch]);
          }
        //charts.push(chart);
        } // end loop over channels

        let last_event = self.event_queue.back();
        let view_string : String;
        match last_event {
          Some(event) => { 
            view_string = event.to_string();
          }, 
          None => {
            view_string = String::from("EVT QUEUE EMPTY!");
          }
        }
        let event_view = Paragraph::new(view_string)
          .style(self.theme.style())
          .alignment(Alignment::Left)
          //.scroll((5, 10))
          .block(
          Block::default()
            .borders(Borders::ALL)
            .style(self.theme.style())
            .title("Last RBEvent")
            .border_type(BorderType::Rounded),
        );
        frame.render_widget(event_view, detail_and_ch9_chunks[1]);
      }, // end Waveform
      RBTabView::RBMoniData => {
        // Have 4 columns a 3 plots each (12 in total)
        let columns = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(25),
               Constraint::Percentage(25),
               Constraint::Percentage(25),
               Constraint::Percentage(25),
              ].as_ref(),
          )
          .split(*main_window);

        let col1 = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(34),
               Constraint::Percentage(33),
               Constraint::Percentage(33),
              ].as_ref(),
          )
          .split(columns[1]);
        let col2 = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(34),
               Constraint::Percentage(33),
               Constraint::Percentage(33),
              ].as_ref(),
          )
          .split(columns[2]);
        let col3 = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(34),
               Constraint::Percentage(33),
               Constraint::Percentage(33),
              ].as_ref(),
          )
          .split(columns[3]);

        let last_moni = self.moni_queue.back();
        let view_string : String;
        match last_moni {
          Some(_moni) => { 
            view_string = _moni.to_string();
          }, 
          None => {
            view_string = String::from("MONI QUEUE EMPTY!");
          }
        }
        
        let moni_view = Paragraph::new(view_string)
          .style(self.theme.style())
          .alignment(Alignment::Left)
          //.scroll((5, 10))
          .block(
          Block::default()
            .borders(Borders::ALL)
            .style(self.theme.style())
            .title("Last RBMoniData")
            .border_type(BorderType::Rounded),
        );
        frame.render_widget(moni_view, columns[0]);
       
        let fpga_ds_name   = String::from("FPGA T");
        let fpga_ds_title  = String::from("FPGA T [\u{00B0}C] ");
        let fpga_tc_theme  = self.theme.clone();
        let fpga_tc = timeseries(&mut self.fpgatmp_queue,
                                 fpga_ds_name,
                                 fpga_ds_title,
                                 &fpga_tc_theme  );
        frame.render_widget(fpga_tc, col1[0]);
        
        let humi_ds_name   = String::from("Humidity %");
        let humi_ds_title  = String::from("Humidity %");
        let humi_tc_theme  = self.theme.clone();
        let humi_tc = timeseries(&mut self.humidity,
                                 humi_ds_name,
                                 humi_ds_title,
                                 &humi_tc_theme);
        frame.render_widget(humi_tc, col2[0]);
        
        let pres_ds_name   = String::from("Pressure [hPa]");
        let pres_ds_title  = String::from("Pressure [hPa]");
        let pres_tc_theme  = self.theme.clone();
        let pres_tc = timeseries(&mut self.pressure,
                                 pres_ds_name,
                                 pres_ds_title,
                                 &pres_tc_theme);
        frame.render_widget(pres_tc, col2[1]);
        
        let mag_ds_x_name   = String::from("Magnetic x [G]");
        let mag_ds_x_title  = String::from("Magnetic x [G}");
        let mag_tc_x_theme  = self.theme.clone();
        let mag_tc_x = timeseries(&mut self.mag_x,
                                  mag_ds_x_name,
                                  mag_ds_x_title,
                                  &mag_tc_x_theme);
        frame.render_widget(mag_tc_x, col3[0]);
        
        let mag_ds_y_name   = String::from("Magnetic y [G]");
        let mag_ds_y_title  = String::from("Magnetic y [G}");
        let mag_tc_y_theme  = self.theme.clone();
        let mag_tc_y = timeseries(&mut self.mag_y,
                                  mag_ds_y_name,
                                  mag_ds_y_title,
                                  &mag_tc_y_theme);
        frame.render_widget(mag_tc_y, col3[1]);
        
        let mag_ds_z_name   = String::from("Magnetic z [G]");
        let mag_ds_z_title  = String::from("Magnetic z [G}");
        let mag_tc_z_theme  = self.theme.clone();
        let mag_tc_z = timeseries(&mut self.mag_z,
                                  mag_ds_z_name,
                                  mag_ds_z_title,
                                  &mag_tc_z_theme);
        frame.render_widget(mag_tc_z, col3[2]);
        
        let mag_ds_tot_name   = String::from("Magnetic TOT [G]");
        let mag_ds_tot_title  = String::from("Magnetic TOT [G}");
        let mag_tc_tot_theme  = self.theme.clone();
        let mag_tc_tot = timeseries(&mut self.mag_tot,
                                     mag_ds_tot_name,
                                     mag_ds_tot_title,
                                    &mag_tc_tot_theme);
        frame.render_widget(mag_tc_tot, col2[2]);

      },
      RBTabView::Info => {
        let main_view = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(30), Constraint::Percentage(70)].as_ref(),
          )
          .split(*main_window);
        let view_info = format!("Summary Statistics:
           N_Events                         : {}
           N_Moni                           : {}
           N EventID Missed                 : {}",
                              self.n_events,
                              self.n_moni,
                              self.miss_evid);
        let info_view = Paragraph::new(view_info)
        .style(self.theme.style())
        .alignment(Alignment::Left)
        .block(
          Block::default()
            .borders(Borders::ALL)
            .style(self.theme.style())
            .title("Overview")
            .border_type(BorderType::Rounded),
        );

        // render everything
        frame.render_widget(info_view, main_view[0]); 
      }
    } //end match 
  }
}

