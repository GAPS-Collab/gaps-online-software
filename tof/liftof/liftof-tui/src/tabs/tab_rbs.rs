//! ReadoutBoard Status tab
//!
//! Find connected ReadoutBoards and show their 
//! details as well as the last waveforms
//!

use std::time::Instant;
use std::fs;
use std::collections::{
    VecDeque,
    HashMap,
};

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
    //terminal::Frame,
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{
        Modifier,
        //Style
    },
    text::{
        //Span,
        Line
    },
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

use crossbeam_channel::{
    Receiver
};


use tof_dataclasses::packets::{
    TofPacket,
    PacketType
};
use tof_dataclasses::calibrations::RBCalibrations;
use tof_dataclasses::errors::SerializationError;
use tof_dataclasses::events::RBEvent;
//use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::monitoring::{
    RBMoniData,
    LTBMoniData,
    PAMoniData,
    RBMoniDataSeries,
    LTBMoniDataSeries,
    PAMoniDataSeries,
};
use tof_dataclasses::series::MoniSeries;
use tof_dataclasses::io::RBEventMemoryStreamer;
use tof_dataclasses::database::ReadoutBoard;

use crate::widgets::{
    timeseries,
    //histogram,
};
use crate::colors::{
  ColorTheme,
};

#[derive(Debug, Copy, Clone)]
pub enum RBLTBListFocus {
  RBList,
  LTBList,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RBTabView {
  Info,
  Waveform,
  RBMoniData,
  PAMoniData,
  PBMoniData,
  LTBMoniData,
  SelectRB,
}

#[derive(Debug, Clone)]
pub struct RBTab<'a>  {
  pub tp_receiver        : Receiver<TofPacket>,
  pub rb_receiver        : Receiver<RBEvent>,
  pub rb_selector        : u8,
  pub ltb_selector       : u8,
  pub rb_changed         : bool,
  pub rb_calibration     : RBCalibrations,
  pub cali_loaded        : bool,
  pub event_queue        : VecDeque<RBEvent>,
  pub moni_queue         : RBMoniDataSeries,
  pub ltb_moni_queue     : LTBMoniDataSeries,  
  pub pa_show_biases     : bool,
  pub pa_moni_queue      : PAMoniDataSeries,
  pub met_queue          : VecDeque<f64>,
  pub met_queue_moni     : HashMap<u8,VecDeque<f64>>,
  pub met_queue_ltb_moni : HashMap<u8,VecDeque<f64>>,
  pub met_queue_pa_moni  : HashMap<u8,VecDeque<f64>>,
  /// Holds waveform data
  pub ch_data            : Vec<Vec<(f64,f64)>>,
  /// Holds the monitoring qunatities
  pub fpgatmp_queue      : VecDeque<(f64,f64)>,
  pub fpgatmp_fr_moni    : bool,

  pub queue_size         : usize,
  
  pub n_events           : usize,
  pub n_moni             : usize,
  pub miss_evid          : usize,
  pub last_evid          : u32,
  pub nch_histo          : Hist1D<Uniform<f32>>,
  timer                  : Instant,

  pub theme              : ColorTheme,
  pub view               : RBTabView,

  pub rbs                : HashMap<u8, ReadoutBoard>,

  pub list_focus         : RBLTBListFocus,
  // list for the rb selector
  pub rbl_state          : ListState,
  pub rbl_items          : Vec::<ListItem<'a>>,
  pub rbl_active         : bool,
  // list for the ltb selector
  pub ltbl_state         : ListState,
  pub ltbl_items         : Vec::<ListItem<'a>>,
  pub ltbl_active        : bool,
}

impl RBTab<'_>  {

  pub fn new(tp_receiver  : Receiver<TofPacket>,
             rb_receiver  : Receiver<RBEvent>,
             rbs          : HashMap<u8, ReadoutBoard>,
             theme        : ColorTheme) -> RBTab<'static>  {
    let mut rb_select_items = Vec::<ListItem>::new();
    for k in 1..51 {
      let this_item = format!("  RB{:0>2}", k);
      rb_select_items.push(ListItem::new(Line::from(this_item)));
    }
    let mut ltb_select_items = Vec::<ListItem>::new();
    for k in 1..21 {
      let this_item = format!("  LTB{:0>2}", k);
      ltb_select_items.push(ListItem::new(Line::from(this_item)));
    }

    let queue_size = 1000usize;
    let mut ch_data    = Vec::<Vec::<(f64,f64)>>::with_capacity(1024);
    for _channel in 0..9 {
      let tmp_vec = vec![(0.0f64,0.0f64);1024];
      //ch_data.push(Vec::<(f64,f64)>::new());
      ch_data.push(tmp_vec);
    }
    let bins = Uniform::new(50,-0.5,49.5).unwrap();
    let mut rbl_state    = ListState::default();
    rbl_state.select(Some(1));
    let mut ltbl_state   = ListState::default();
    ltbl_state.select(Some(1));
    RBTab {
      tp_receiver        : tp_receiver,
      rb_receiver        : rb_receiver,
      rb_selector        : 0,
      ltb_selector       : 9,
      rb_changed         : false,
      rb_calibration     : RBCalibrations::new(0),
      cali_loaded        : false,
      event_queue        : VecDeque::<RBEvent>::with_capacity(queue_size),
      //moni_queue         : VecDeque::<RBMoniData>::with_capacity(queue_size),
      moni_queue         : RBMoniDataSeries::new(),
      met_queue          : VecDeque::<f64>::with_capacity(queue_size),
      met_queue_moni     : HashMap::<u8, VecDeque<f64>>::new(),
      ltb_moni_queue     : LTBMoniDataSeries::new(),
      met_queue_ltb_moni : HashMap::<u8, VecDeque<f64>>::new(),
      pa_show_biases     : false,
      pa_moni_queue      : PAMoniDataSeries::new(),
      met_queue_pa_moni  : HashMap::<u8, VecDeque<f64>>::new(),
      fpgatmp_queue      : VecDeque::<(f64,f64)>::with_capacity(queue_size),
      fpgatmp_fr_moni    : true,

      ch_data            : ch_data,

      queue_size         : queue_size,
      
      n_events           : 0,
      n_moni             : 0,
      miss_evid          : 0,
      last_evid          : 0,
      nch_histo          : ndhistogram!(bins),
      timer              : Instant::now(),
  
      theme              : theme,
      view               : RBTabView::Waveform,
   
      rbs                : rbs,

      list_focus         : RBLTBListFocus::RBList,

      rbl_state          : rbl_state,
      rbl_items          : rb_select_items,
      rbl_active         : false,
      
      ltbl_state         : ltbl_state,
      ltbl_items         : ltb_select_items,
      ltbl_active        : false,
    }
  }
  
  pub fn receive_packet(&mut self) -> Result<(), SerializationError> {
    let met    = self.timer.elapsed().as_secs_f64();
    let mut ev = RBEvent::new();
    let bins   = Uniform::new(50,-0.5,49.5).unwrap();
    //info!("Receive packet!"); 
    if self.rb_changed {
      info!("RB change detectod!");
      // currently, only one RB at a time is supported
      //self.moni_queue.clear();
      self.event_queue.clear();
      self.met_queue.clear();
      //self.met_queue_moni.clear();
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
            match RBCalibrations::from_file(cali_path.clone(), true) {
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
      self.rb_changed = false;
      info!("RB changed!");
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
            PacketType::PAMoniData => {
              trace!("Received new PAMoniData!");
              let moni : PAMoniData = pack.unpack()?;
              self.pa_moni_queue.add(moni);
              if !self.met_queue_pa_moni.contains_key(&moni.board_id) {
                self.met_queue_pa_moni.insert(moni.board_id, VecDeque::<f64>::with_capacity(1000));
              } else {
                self.met_queue_pa_moni.get_mut(&moni.board_id).unwrap().push_back(met);
                if self.met_queue_pa_moni.get(&moni.board_id).unwrap().len() > self.queue_size {
                  self.met_queue_pa_moni.get_mut(&moni.board_id).unwrap().pop_front();
                }
              }
              return Ok(());
            }
            PacketType::LTBMoniData => {
              trace!("Received new LTBMoniData!");
              let moni : LTBMoniData = pack.unpack()?;
              self.ltb_moni_queue.add(moni);
              if !self.met_queue_ltb_moni.contains_key(&moni.board_id) {
                self.met_queue_ltb_moni.insert(moni.board_id, VecDeque::<f64>::with_capacity(1000));
              } else {
                self.met_queue_ltb_moni.get_mut(&moni.board_id).unwrap().push_back(met);
                if self.met_queue_ltb_moni.get(&moni.board_id).unwrap().len() > self.queue_size {
                  self.met_queue_ltb_moni.get_mut(&moni.board_id).unwrap().pop_front();
                }
              }
              return Ok(());
            },
            PacketType::RBMoniData   => {
              trace!("Received new RBMoniData!");
              let moni : RBMoniData = pack.unpack()?;
              self.moni_queue.add(moni);
              self.n_moni += 1;
              if !self.met_queue_moni.contains_key(&moni.board_id) {
                // FIXME - make the 1000 (which is queue size) a member
                self.met_queue_moni.insert(moni.board_id, VecDeque::<f64>::with_capacity(1000));
              } else {
                self.met_queue_moni.get_mut(&moni.board_id).unwrap().push_back(met);
                if self.met_queue_moni.get(&moni.board_id).unwrap().len() > self.queue_size {
                  self.met_queue_moni.get_mut(&moni.board_id).unwrap().pop_front();
                }
              }
              return Ok(());
            },
            PacketType::RBEvent => {
              ev = pack.unpack()?;
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
      self.fpgatmp_fr_moni = false; // choose this as source
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
    //info!("Selecting {}", i);
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
  
  pub fn next_ltb(&mut self) {
    let i = match self.ltbl_state.selected() {
      Some(i) => {
        if i >= self.ltbl_items.len() - 1 {
          self.ltbl_items.len() - 1
        } else {
          i + 1
        }
      }
      None => 0,
    };
    self.ltbl_state.select(Some(i));
  }

  pub fn previous_ltb(&mut self) {
    let i = match self.ltbl_state.selected() {
      Some(i) => {
        if i == 0 {
          0 
        } else {
          i - 1
        }
      }
      None => 0,
    };
    self.ltbl_state.select(Some(i));
  }

  pub fn unselect_ltbl(&mut self) {
    self.ltbl_state.select(None);
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
    match self.view {
      RBTabView::SelectRB => {
        let main_lo = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(10),
               //Constraint::Percentage(20),
               Constraint::Percentage(90)].as_ref(),
          )
          .split(*main_window);
        //let par_rb_title_string = String::from("Select ReadoutBoard (RB)");
        //let (first, rest) = par_title_string.split_at(1);
        //let par_title = Line::from(vec![
        //  Span::styled(
        //      first,
        //      Style::default()
        //          .fg(self.theme.hc)
        //          .add_modifier(Modifier::UNDERLINED),
        //  ),
        //  Span::styled(rest, self.theme.style()),
        //]);
        let rbs = Block::default()
          .borders(Borders::ALL)
          .style(self.theme.style())
          .title("Select ReadoutBoard (RB)")
          .border_type(BorderType::Plain);
        let rb_select_list = List::new(self.rbl_items.clone()).block(rbs)
          .highlight_style(self.theme.highlight().add_modifier(Modifier::BOLD))
          .highlight_symbol(">>")
          .repeat_highlight_symbol(true);
        // No ltb selection right now
        //let ltbs = Block::default()
        //  .borders(Borders::ALL)
        //  .style(self.theme.style())
        //  .title("Select LocalTriggerBoard (LTB)")
        //  .border_type(BorderType::Plain);
        //let ltb_select_list = List::new(self.ltbl_items.clone()).block(ltbs)
        //  .highlight_style(self.theme.highlight().add_modifier(Modifier::BOLD))
        //  .highlight_symbol(">>")
        //  .repeat_highlight_symbol(true);
        match self.list_focus {
          RBLTBListFocus::RBList => {
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
          },
          _ => ()
          // no ltb selection right now
          //RBLTBListFocus::LTBList => {
          //  match self.ltbl_state.selected() {
          //    None    => {
          //      let selector =  1;
          //      if self.ltb_selector != selector {
          //        self.ltb_selector = selector;
          //      } 
          //    },
          //    Some(_ltbid) => {
          //      let selector =  _ltbid as u8 + 1;
          //      if self.ltb_selector != selector {
          //        self.ltb_selector = selector;
          //      }
          //    },
          //  }
          //}
        }
        let view_string : String;
        match self.rbs.get(&self.rb_selector) {
          Some(_rb) => {
            view_string = format!("{}", _rb.to_summary_str());
          }
          None => {
            view_string = format!("No information for RB {} in DBor DB not available!", self.rb_selector);
          }
        }
        let rb_view = Paragraph::new(view_string)
          .style(self.theme.style())
          .alignment(Alignment::Left)
          //.scroll((5, 10))
          .block(
          Block::default()
            .borders(Borders::ALL)
            .style(self.theme.style())
            .title("RB")
            .border_type(BorderType::Rounded),
        );

        frame.render_stateful_widget(rb_select_list,  main_lo[0], &mut self.rbl_state );
        frame.render_widget(rb_view, main_lo[1]);
        //frame.render_stateful_widget(ltb_select_list, list_chunks[1], &mut self.ltbl_state );
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
          let label          = format!("Ch{}", ch + 1);
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
        // Have 4 columns a 4 plots each. We use col(0,0) for the RBMoniData string,
        // so this leaves us with 13 plots
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
        let col0 = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(75),
               Constraint::Percentage(25)].as_ref()
          )
          .split(columns[0]);
        let col1 = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(25),
               Constraint::Percentage(25),
               Constraint::Percentage(25),
               Constraint::Percentage(25),
              ].as_ref(),
          )
          .split(columns[1]);
        let col2 = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(25),
               Constraint::Percentage(25),
               Constraint::Percentage(25),
               Constraint::Percentage(25),
              ].as_ref(),
          )
          .split(columns[2]);
        let col3 = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(25),
               Constraint::Percentage(25),
               Constraint::Percentage(25),
               Constraint::Percentage(25),
              ].as_ref(),
          )
          .split(columns[3]);

        let last_moni = self.moni_queue.get_last_moni(self.rb_selector);
        let view_string : String;
        match last_moni {
          Some(_moni) => { 
            view_string = _moni.to_string();
          }, 
          None => {
            view_string = format!("No RBMoniData for board {} avaiable", self.rb_selector);
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
        frame.render_widget(moni_view, col0[0]);
       
        let rate_ds_name   = String::from("Rate");
        let rate_ds_title  = String::from("RB Rate [Hz]");
        let rate_data      = self.moni_queue.get_var_for_board("rate", &self.rb_selector);
        let mut rate_ts    = VecDeque::<(f64, f64)>::new(); 
        match rate_data {
          None => {
            error!("No rate data available for board {}", self.rb_selector);
          },
          Some(rdata) => {
            if rdata.len() != 0 {
              for (k, time) in self.met_queue_moni.get(&self.rb_selector).unwrap().iter().enumerate() {
                rate_ts.push_back((*time, rdata[k] as f64));
              }
            }
          }
        }
        let rate_tc = timeseries(&mut rate_ts,
                                 rate_ds_name,
                                 rate_ds_title,
                                 &self.theme);
        frame.render_widget(rate_tc, col0[1]);

        // ambience
        let mag_tot_ds_name   = String::from("Magnetic Field");
        let mag_tot_ds_title  = String::from("Tot mag field [Gauss]");
        let mag_tot_data      = self.moni_queue.get_var_for_board("mag_tot", &self.rb_selector);
        let mut mag_tot_ts    = VecDeque::<(f64, f64)>::new(); 
        match mag_tot_data  {
          None => {
            error!("No mag_tot data available for board {}", self.rb_selector);
          },
          Some(data) => {
            if data.len() != 0 {
              for (k, time) in self.met_queue_moni.get(&self.rb_selector).unwrap().iter().enumerate() {
                mag_tot_ts.push_back((*time, data[k] as f64));
              }
            }
          }
        }
        let mag_tot_tc = timeseries(&mut mag_tot_ts,
                                 mag_tot_ds_name,
                                 mag_tot_ds_title,
                                 &self.theme);
        frame.render_widget(mag_tot_tc, col1[0]);

        let pres_ds_name   = String::from("Atmospheric pressure");
        let pres_ds_title  = String::from("Atmospheric pressure [hPa]");
        let pres_data      = self.moni_queue.get_var_for_board("pressure", &self.rb_selector);
        let mut pres_ts    = VecDeque::<(f64, f64)>::new(); 
        match pres_data {
          None => {
            error!("No atmos pressure data available for board {}", self.rb_selector);
          },
          Some(data) => {
            if data.len() != 0 {
              for (k, time) in self.met_queue_moni.get(&self.rb_selector).unwrap().iter().enumerate() {
                pres_ts.push_back((*time, data[k] as f64));
              }
            }
          }
        }
        
        let pres_tc = timeseries(&mut pres_ts,
                                 pres_ds_name,
                                 pres_ds_title,
                                 &self.theme);
        frame.render_widget(pres_tc, col1[1]);
        
        let humi_ds_name   = String::from("Ambient humidity");
        let humi_ds_title  = String::from("Humidity [%]");
        let humi_data      = self.moni_queue.get_var_for_board("humidity", &self.rb_selector);
        let mut humi_ts    = VecDeque::<(f64, f64)>::new(); 
        match humi_data {
          None => {
            error!("No humidity data available for board {}", self.rb_selector);
          },
          Some(data) => {
            if data.len() != 0 {
              for (k, time) in self.met_queue_moni.get(&self.rb_selector).unwrap().iter().enumerate() {
                humi_ts.push_back((*time, data[k] as f64));
              }
            }
          }
        }
        let humi_tc = timeseries(&mut humi_ts,
                                 humi_ds_name,
                                 humi_ds_title,
                                 &self.theme);
        frame.render_widget(humi_tc, col1[2]);

        // Temperatures (one is missing because of display constraints)
        let fpga_ds_name   = String::from("FPGA (DRS) Temperature");
        let fpga_ds_title  = String::from("DRS Temp [\u{00B0}C]");
        // in case we are receiving RBEvents, these have the FPGA temperature as well
        // since this is more fine-grained (once every event) we will use that instead
        if !self.fpgatmp_fr_moni {
          let fpga_tc = timeseries(&mut self.fpgatmp_queue,
                                   fpga_ds_name,
                                   fpga_ds_title,
                                   &self.theme  );
          frame.render_widget(fpga_tc, col1[3]);
        } else {
          // we will get it from the RBMoniData
          let fpga_data      = self.moni_queue.get_var_for_board("tmp_drs", &self.rb_selector);
          let mut fpga_ts    = VecDeque::<(f64, f64)>::new(); 
          match fpga_data {
            None => {
              error!("No DRS4 temperature data available for board {}", self.rb_selector);
            },
            Some(data) => {
              if data.len() != 0 {
                for (k, time) in self.met_queue_moni.get(&self.rb_selector).unwrap().iter().enumerate() {
                  fpga_ts.push_back((*time, data[k] as f64));
                }
              }
            }
          }
          let fpga_tc = timeseries(&mut fpga_ts,
                                   fpga_ds_name,
                                   fpga_ds_title,
                                   &self.theme);
          frame.render_widget(fpga_tc, col1[3]);
        }

        let tmp_clk_ds_name   = String::from("CLK Temperature");
        let tmp_clk_ds_title  = String::from("CLK Temp. [\u{00B0}C]");
        let tmp_clk_data      = self.moni_queue.get_var_for_board("tmp_clk", &self.rb_selector);
        let mut tmp_clk_ts    = VecDeque::<(f64, f64)>::new(); 
        match tmp_clk_data {
          None => {
            error!("No CLK temperature data available for board {}", self.rb_selector);
          },
          Some(data) => {
            if data.len() != 0 {
              for (k, time) in self.met_queue_moni.get(&self.rb_selector).unwrap().iter().enumerate() {
                tmp_clk_ts.push_back((*time, data[k] as f64));
              }
            }
          }
        }
        let tmp_clk_tc = timeseries(&mut tmp_clk_ts,
                                    tmp_clk_ds_name,
                                    tmp_clk_ds_title,
                                    &self.theme);
        frame.render_widget(tmp_clk_tc, col2[0]);
        
        let tmp_adc_ds_name   = String::from("ADC Temperature");
        let tmp_adc_ds_title  = String::from("ADC Temp. [\u{00B0}C]");
        let tmp_adc_data      = self.moni_queue.get_var_for_board("tmp_adc", &self.rb_selector);
        let mut tmp_adc_ts    = VecDeque::<(f64, f64)>::new(); 
        match tmp_adc_data {
          None => {
            error!("No ADC temperature data available for board {}", self.rb_selector);
          },
          Some(data) => {
            if data.len() != 0 {
              for (k, time) in self.met_queue_moni.get(&self.rb_selector).unwrap().iter().enumerate() {
                tmp_adc_ts.push_back((*time, data[k] as f64));
              }
            }
          }
        }
        let tmp_adc_tc = timeseries(&mut tmp_adc_ts,
                                    tmp_adc_ds_name,
                                    tmp_adc_ds_title,
                                    &self.theme);
        frame.render_widget(tmp_adc_tc, col2[1]);
        
        let tmp_zynq_ds_name   = String::from("ZYNQ Temperature");
        let tmp_zynq_ds_title  = String::from("ZYNQ Temp. [\u{00B0}C]");
        let tmp_zynq_data      = self.moni_queue.get_var_for_board("tmp_zynq", &self.rb_selector);
        let mut tmp_zynq_ts    = VecDeque::<(f64, f64)>::new(); 
        match tmp_zynq_data {
          None => {
            error!("No ZYNQ temperature data available for board {}", self.rb_selector);
          },
          Some(data) => {
            if data.len() != 0 {
              for (k, time) in self.met_queue_moni.get(&self.rb_selector).unwrap().iter().enumerate() {
                tmp_zynq_ts.push_back((*time, data[k] as f64));
              }
            }
          }
        }
        let tmp_zynq_tc = timeseries(&mut tmp_zynq_ts,
                                     tmp_zynq_ds_name,
                                     tmp_zynq_ds_title,
                                     &self.theme);
        frame.render_widget(tmp_zynq_tc, col2[2]);
        
        let tmp_bm280_name   = String::from("BM280 Temperature");
        let tmp_bm280_title  = String::from("BM280 Temp. [\u{00B0}C]");
        let tmp_bm280_data      = self.moni_queue.get_var_for_board("tmp_bm280", &self.rb_selector);
        let mut tmp_bm280_ts    = VecDeque::<(f64, f64)>::new(); 
        match tmp_bm280_data {
          None => {
            error!("No BM280 temperature data available for board {}", self.rb_selector);
          },
          Some(data) => {
            if data.len() != 0 {
              for (k, time) in self.met_queue_moni.get(&self.rb_selector).unwrap().iter().enumerate() {
                tmp_bm280_ts.push_back((*time, data[k] as f64));
              }
            }
          }
        }
        let tmp_bm280_tc = timeseries(&mut tmp_bm280_ts,
                                      tmp_bm280_name,
                                      tmp_bm280_title,
                                      &self.theme);
        frame.render_widget(tmp_bm280_tc, col2[3]);
       
        // Currents 
        let drs_c_name   = String::from("DRS Current");
        let drs_c_title  = String::from("DRS Curr. [mA]");
        let drs_c_data   = self.moni_queue.get_var_for_board("drs_dvdd_current", &self.rb_selector);
        let mut drs_c_ts = VecDeque::<(f64, f64)>::new(); 
        match drs_c_data {
          None => {
            error!("No DRS4 current data available for board {}", self.rb_selector);
          },
          Some(data) => {
            if data.len() != 0 {
              for (k, time) in self.met_queue_moni.get(&self.rb_selector).unwrap().iter().enumerate() {
                drs_c_ts.push_back((*time, data[k] as f64));
              }
            }
          }
        }
        let drs_c_tc = timeseries(&mut drs_c_ts,
                                  drs_c_name,
                                  drs_c_title,
                                  &self.theme);
        frame.render_widget(drs_c_tc, col3[0]);

        let zynq_c_name   = String::from("Zynq Current");
        let zynq_c_title  = String::from("Zynq Curr. [mA]");
        let zynq_c_data   = self.moni_queue.get_var_for_board("zynq_current", &self.rb_selector);
        let mut zynq_c_ts = VecDeque::<(f64, f64)>::new(); 
        match zynq_c_data {
          None => {
            error!("No ZYNQ current data available for board {}", self.rb_selector);
          },
          Some(data) => {
            if data.len() != 0 {
              for (k, time) in self.met_queue_moni.get(&self.rb_selector).unwrap().iter().enumerate() {
                zynq_c_ts.push_back((*time, data[k] as f64));
              }
            }
          }
        }
        let zynq_c_tc = timeseries(&mut zynq_c_ts,
                                   zynq_c_name,
                                   zynq_c_title,
                                   &self.theme);
        frame.render_widget(zynq_c_tc, col3[1]);
        
        let p3v3_c_name   = String::from("P3V3 Current");
        let p3v3_c_title  = String::from("P3V3 Curr. [mA]");
        let p3v3_c_data   = self.moni_queue.get_var_for_board("p3v3_current", &self.rb_selector);
        let mut p3v3_c_ts = VecDeque::<(f64, f64)>::new(); 
        match p3v3_c_data {
          None => {
            error!("No P3V3 current data available for board {}", self.rb_selector);
          },
          Some(data) => {
            if data.len() != 0 {
              for (k, time) in self.met_queue_moni.get(&self.rb_selector).unwrap().iter().enumerate() {
                p3v3_c_ts.push_back((*time, data[k] as f64));
              }
            }
          }
        }
        let p3v3_c_tc = timeseries(&mut p3v3_c_ts,
                                   p3v3_c_name,
                                   p3v3_c_title,
                                   &self.theme);
        frame.render_widget(p3v3_c_tc, col3[2]);
        
        let p3v5_c_name   = String::from("P3V5 Current");
        let p3v5_c_title  = String::from("P3V5 Curr. [mA]");
        let p3v5_c_data   = self.moni_queue.get_var_for_board("p3v5_current", &self.rb_selector);
        let mut p3v5_c_ts = VecDeque::<(f64, f64)>::new(); 
        match p3v5_c_data {
          None => {
            error!("No P3V5 current data available for board {}", self.rb_selector);
          },
          Some(data) => {
            if data.len() != 0 {
              for (k, time) in self.met_queue_moni.get(&self.rb_selector).unwrap().iter().enumerate() {
                p3v5_c_ts.push_back((*time, data[k] as f64));
              }
            }
          }
        }
        let p3v5_c_tc = timeseries(&mut p3v5_c_ts,
                                   p3v5_c_name,
                                   p3v5_c_title,
                                   &self.theme);
        frame.render_widget(p3v5_c_tc, col3[3]);
      },
      RBTabView::LTBMoniData => {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [Constraint::Percentage(30),
                 Constraint::Percentage(70)].as_ref(),
            )
            .split(*main_window);
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [Constraint::Percentage(30),
                 Constraint::Percentage(30),
                 Constraint::Percentage(40)].as_ref(),
            )
            .split(columns[1]);
        
        let last_ltb_moni = self.ltb_moni_queue.get_last_moni(self.ltb_selector);
        let mut ltb_moni_str = format!("No data for board {}!", self.ltb_selector);
        let mut ltb_thr_str  = format!("No data for board {}!", self.ltb_selector);
        match last_ltb_moni {
          None => (),
          Some(mon) => {
            ltb_moni_str = format!("{}", mon);
            ltb_thr_str  = format!("  Hit  : {:.3} [mV]", mon.thresh[0]);
            ltb_thr_str += &(format!("\n  Beta : {:.3} [mV]", mon.thresh[1]));
            ltb_thr_str += &(format!("\n  Veto : {:.3} [mV]", mon.thresh[2]));
          }
        }
        let moni_view = Paragraph::new(ltb_moni_str)
          .style(self.theme.style())
          .alignment(Alignment::Left)
          //.scroll((5, 10))
          .block(
          Block::default()
            .borders(Borders::ALL)
            .style(self.theme.style())
            .title("Last LTBMoniData")
            .border_type(BorderType::Rounded),
        );
        frame.render_widget(moni_view, columns[0]);
        let thr_view = Paragraph::new(ltb_thr_str)
          .style(self.theme.style())
          .alignment(Alignment::Left)
          //.scroll((5, 10))
          .block(
          Block::default()
            .borders(Borders::ALL)
            .style(self.theme.style())
            .title("Thresholds")
            .border_type(BorderType::Rounded),
        );
        frame.render_widget(thr_view, rows[2]);

        let tt_name   = String::from("Trenz Temp");
        let tt_title  = String::from("Trenz Temp [\u{00B0}C]");
        let tt_data   = self.ltb_moni_queue.get_var_for_board("trenz_temp", &self.ltb_selector);
        let mut tt_ts = VecDeque::<(f64, f64)>::new(); 
        match tt_data {
          None => {
            error!("No trenz temp data available for board {}", self.ltb_selector);
          },
          Some(data) => {
            if data.len() != 0 {
              for (k, time) in self.met_queue_ltb_moni.get(&self.ltb_selector).unwrap().iter().enumerate() {
                tt_ts.push_back((*time, data[k] as f64));
              }
            }
          }
        }
        let tt_tc = timeseries(&mut tt_ts,
                               tt_name,
                               tt_title,
                               &self.theme);
        frame.render_widget(tt_tc, rows[0]);

        let lt_name   = String::from("LTB Temperature");
        let lt_title  = String::from("LTB Temp. [\u{00B0}C]");
        let lt_data      = self.ltb_moni_queue.get_var_for_board("ltb_temp", &self.ltb_selector);
        let mut lt_ts    = VecDeque::<(f64, f64)>::new(); 
        match lt_data {
          None => {
            error!("No LTB temp data available for board {}", self.ltb_selector);
          },
          Some(data) => {
            if data.len() != 0 {
              for (k, time) in self.met_queue_ltb_moni.get(&self.ltb_selector).unwrap().iter().enumerate() {
                lt_ts.push_back((*time, data[k] as f64));
              }
            }
          }
        }
        let lt_tc = timeseries(&mut lt_ts,
                               lt_name,
                               lt_title,
                               &self.theme);
        frame.render_widget(lt_tc, rows[1]);

      }
      RBTabView::PAMoniData => {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [Constraint::Percentage(8),
                 Constraint::Percentage(92)].as_ref(),
            )
            .split(*main_window);
          let columns = Layout::default()
              .direction(Direction::Horizontal)
              .constraints(
                  [Constraint::Percentage(25),
                   Constraint::Percentage(25),
                   Constraint::Percentage(25),
                   Constraint::Percentage(25)].as_ref(),
              )
              .split(rows[1]);
          let col0 = Layout::default()
              .direction(Direction::Vertical)
              .constraints(
                  [Constraint::Percentage(24),
                   Constraint::Percentage(24),
                   Constraint::Percentage(24),
                   Constraint::Percentage(24)].as_ref(),
              )
              .split(columns[0]);
          let col1 = Layout::default()
              .direction(Direction::Vertical)
              .constraints(
                  [Constraint::Percentage(24),
                   Constraint::Percentage(24),
                   Constraint::Percentage(24),
                   Constraint::Percentage(24)].as_ref(),
              )
              .split(columns[1]);
          let col2 = Layout::default()
              .direction(Direction::Vertical)
              .constraints(
                  [Constraint::Percentage(24),
                   Constraint::Percentage(24),
                   Constraint::Percentage(24),
                   Constraint::Percentage(24)].as_ref(),
              )
              .split(columns[2]);
          let col3 = Layout::default()
              .direction(Direction::Vertical)
              .constraints(
                  [Constraint::Percentage(24),
                   Constraint::Percentage(24),
                   Constraint::Percentage(24),
                   Constraint::Percentage(24)].as_ref(),
              )
              .split(columns[3]);
        // the preamps don't have their own board, the board id refers to the RB
        let pa_str  = format!("PreampMoniData for ReadoutBoard {}", self.rb_selector);
        let pa_view = Paragraph::new(pa_str)
          .style(self.theme.style())
          .alignment(Alignment::Left)
          .block(
          Block::default()
            .borders(Borders::ALL)
            .style(self.theme.style())
            //.title("Thresholds")
            .border_type(BorderType::Rounded),
        );
        frame.render_widget(pa_view, rows[0]);
        
        if self.pa_show_biases {
          //let mut moni_str  = String::from("No PAMoniData avaiable!");
          //match self.pa_moni_queue.get_last_moni(self.rb_selector) {
          //  None => (),
          //  Some(moni) => {
          //    moni_str = format!("{}", moni);
          //  }
          //}
          //let moni_view = Paragraph::new(moni_str)
          //  .style(self.theme.style())
          //  .alignment(Alignment::Left)
          //  .block(
          //  Block::default()
          //    .borders(Borders::ALL)
          //    .style(self.theme.style())
          //    //.title("Thresholds")
          //    .border_type(BorderType::Rounded),
          //);
          //frame.render_widget(moni_view, rows[1]);
          // the temperature plots
          for k in 0..16 {
            let identifier = format!("bias_{}", k+1);
            let name   = format!("Ch {} Bias Voltage", k+1);
            let title  = format!("Ch {} Bias [V]", k+1);
            let c_data = self.pa_moni_queue.get_var_for_board(&identifier, &self.rb_selector);
            let mut ts = VecDeque::<(f64, f64)>::new(); 
            match c_data {
              None => {
                error!("No {} data available for board {}", identifier, self.rb_selector);
              },
              Some(data) => {
                for (k, time) in self.met_queue_pa_moni.get(&self.rb_selector).unwrap().iter().enumerate() {
                  ts.push_back((*time, data[k] as f64));
                }
              }
            }
            let tc = timeseries(&mut ts,
                                name,
                                title,
                                &self.theme);
            if k < 4 {
              frame.render_widget(tc, col0[k]);
            } else if k < 8 {
              frame.render_widget(tc, col1[k-4]);
            } else if k < 12 {
              frame.render_widget(tc, col2[k-8]);
            } else if k < 16 {
              frame.render_widget(tc, col3[k-12]);
            }
          }
        } else {
          // the temperature plots
          for k in 0..16 {
            let identifier = format!("temp_{}", k+1);
            let name   = format!("Ch {} Temperature", k+1);
            let title  = format!("Ch {} Temp [\u{00B0}C]", k+1);
            let c_data = self.pa_moni_queue.get_var_for_board(&identifier, &self.rb_selector);
            let mut ts = VecDeque::<(f64, f64)>::new(); 
            match c_data {
              None => {
                error!("No {} data available for board {}", identifier, self.rb_selector);
              },
              Some(data) => {
                if data.len() != 0 {
                  for (k, time) in self.met_queue_pa_moni.get(&self.rb_selector).unwrap().iter().enumerate() {
                    ts.push_back((*time, data[k] as f64));
                  }
                }
              }
            }
            let tc = timeseries(&mut ts,
                                name,
                                title,
                                &self.theme);
            if k < 4 {
              frame.render_widget(tc, col0[k]);
            } else if k < 8 {
              frame.render_widget(tc, col1[k-4]);
            } else if k < 12 {
              frame.render_widget(tc, col2[k-8]);
            } else if k < 16 {
              frame.render_widget(tc, col3[k-12]);
            }
          }

        }

      }
      RBTabView::PBMoniData => {
      }
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

