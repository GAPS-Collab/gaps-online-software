//! Paddle overview - waveforms, 2d charge plot 
//! as well as baselines and baseline rms from
//! TofHits
//!

use std::collections::{
  HashMap,
  VecDeque,
};
use std::sync::{
  Arc,
  Mutex,
};

use crossbeam_channel::{
  Receiver,
  //Sender,
};

use ratatui::prelude::*;
use ratatui::symbols::Marker;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{
  Block,
  BorderType,
  Borders,
  Paragraph,
  List,
  ListItem,
  ListState,
  canvas::{
      Canvas,
      //Circle,
      //Rectangle,
      Points},
};

use ndhistogram::{
  ndhistogram,
  Histogram,
  Hist1D,
};
use ndhistogram::axis::{
    Uniform,
};

//use tof_dataclasses::serialization::Serialization;
use tof_dataclasses::errors::SerializationError;
//use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::events::{
  //RBEvent,
  TofEvent,
  //TofHit,
  //TofEventHeader,
  //MasterTriggerEvent,
  RBWaveform,
};
use tof_dataclasses::calibrations::RBCalibrations;
use tof_dataclasses::database::Paddle;
use tof_dataclasses::analysis::calculate_pedestal;

use crate::colors::ColorTheme;
use crate::menu::{
  PaddleMenu,
  UIMenu,
  UIMenuItem
};

use crate::widgets::{
  prep_data,
  create_labels,
  histogram,
  timeseries,
};

#[derive(Debug, Clone)]
pub struct PaddleTab<'a> {
  pub theme              : ColorTheme,
  pub te_receiver        : Receiver<TofEvent>,
  pub event_queue        : VecDeque<TofEvent>,
  pub queue_size         : usize,
  pub menu               : PaddleMenu<'a>,
  pub wf                 : HashMap<u8, VecDeque<RBWaveform>>,
  pub last_wf_ch_a       : VecDeque<(f64, f64)>,
  pub last_wf_ch_b       : VecDeque<(f64, f64)>,
  pub wf_label_a         : String,
  pub wf_label_b         : String,
  // baseline histograms
  pub calibrations       : Arc<Mutex<HashMap<u8, RBCalibrations>>>,
  pub baseline_ch_a      : HashMap<u8, Hist1D<Uniform<f32>>>,
  pub baseline_ch_b      : HashMap<u8, Hist1D<Uniform<f32>>>,
  pub baseline_rms_ch_a  : HashMap<u8, Hist1D<Uniform<f32>>>,
  pub baseline_rms_ch_b  : HashMap<u8, Hist1D<Uniform<f32>>>,

  // charges
  pub charge_a           : HashMap<u8, VecDeque<f64>>,
  pub charge_b           : HashMap<u8, VecDeque<f64>>, 

  // list for the paddle selector
  pub all_paddles        : HashMap<u8, Paddle>,
  pub pdl_state          : ListState,
  pub current_paddle     : Paddle,
  pub pdl_items          : Vec::<ListItem<'a>>,
  pub pdl_active         : bool,
  pub pdl_selector       : usize,
  pub pdl_changed        : bool,
}

impl PaddleTab<'_> {
  pub fn new(te_receiver : Receiver<TofEvent>,
             all_paddles : HashMap<u8, Paddle>,
             calibrations: Arc<Mutex<HashMap<u8, RBCalibrations>>>,
             theme       : ColorTheme) -> Self {
    let theme_c = theme.clone();
    let mut pd_select_items = Vec::<ListItem>::new();
    for k in 1..161 {
      let this_item = format!("  Paddle{:0>3}", k);
      pd_select_items.push(ListItem::new(Line::from(this_item)));
    }
    let mut charge_a   = HashMap::<u8, VecDeque<f64>>::new();
    let mut charge_b   = HashMap::<u8, VecDeque<f64>>::new();
    let mut wf         = HashMap::<u8, VecDeque<RBWaveform>>::new();
    let mut bl_ch_a    = HashMap::<u8, Hist1D<Uniform<f32>>>::new();
    let mut bl_ch_b    = HashMap::<u8, Hist1D<Uniform<f32>>>::new();
    let mut blrms_ch_a = HashMap::<u8, Hist1D<Uniform<f32>>>::new();
    let mut blrms_ch_b = HashMap::<u8, Hist1D<Uniform<f32>>>::new();
    let bins_bl        = Uniform::new(20, -2.0, 2.0).unwrap();
    let bins_bl_rms    = Uniform::new(20, 0.0, 2.0).unwrap(); 
    for pid in 1..161 {
      charge_a.insert(pid, VecDeque::<f64>::new());
      charge_b.insert(pid, VecDeque::<f64>::new());
      wf.insert(pid, VecDeque::<RBWaveform>::new());
      bl_ch_a.insert(pid, ndhistogram!(bins_bl.clone()));
      bl_ch_b.insert(pid, ndhistogram!(bins_bl.clone()));
      blrms_ch_a.insert(pid, ndhistogram!(bins_bl_rms.clone()));
      blrms_ch_b.insert(pid, ndhistogram!(bins_bl_rms.clone()));
    }

    Self {
      theme,
      te_receiver,
      event_queue       : VecDeque::<TofEvent>::new(),
      queue_size        : 100, // short queue for waveforms so that we don't see 
                               // all the old stuff
      menu              : PaddleMenu::new(theme_c),
      wf                : wf,
      wf_label_a        : String::from("A"),
      wf_label_b        : String::from("B"),
      last_wf_ch_a      : VecDeque::<(f64, f64)>::new(),
      last_wf_ch_b      : VecDeque::<(f64, f64)>::new(),
      calibrations,
      baseline_ch_a     : bl_ch_a,
      baseline_ch_b     : bl_ch_b,
      baseline_rms_ch_a : blrms_ch_a,
      baseline_rms_ch_b : blrms_ch_b,
      charge_a          ,
      charge_b          ,
      all_paddles,
      pdl_items         : pd_select_items,
      pdl_state         : ListState::default(),
      current_paddle    : Paddle::new(),
      pdl_active        : false,
      pdl_selector      : 1,
      pdl_changed       : false,
    }
  }
  
  pub fn next_pd(&mut self) {
    let i = match self.pdl_state.selected() {
      Some(i) => {
        if i >= self.pdl_items.len() - 1 {
          self.pdl_items.len() - 1
        } else {
          i + 1
        }
      }
      None => 0,
    };
    self.pdl_state.select(Some(i));
  }

  pub fn prev_pd(&mut self) {
    let i = match self.pdl_state.selected() {
      Some(i) => {
        if i == 0 {
          0 
        } else {
          i - 1
        }
      }
      None => 0,
    };
    self.pdl_state.select(Some(i));
  }

  pub fn unselect_pdl(&mut self) {
    self.pdl_state.select(None);
  }
 
  pub fn receive_packet(&mut self) -> Result<(), SerializationError> {  
    match self.te_receiver.try_recv() {
      Err(_err) => {
        return Ok(());
      },
      Ok(ev)    => {
        let hits = ev.get_hits();
        // FIXME - get baselines from hits
        for h in hits {
          self.charge_a.get_mut(&(h.paddle_id as u8)).unwrap().push_back(h.get_charge_a() as f64);
          self.charge_b.get_mut(&(h.paddle_id as u8)).unwrap().push_back(h.get_charge_b() as f64);
          if self.charge_a.get_mut(&(h.paddle_id as u8)).unwrap().len() > self.queue_size {
            self.charge_a.get_mut(&(h.paddle_id as u8)).unwrap().pop_front();
          }
          if self.charge_b.get_mut(&(h.paddle_id as u8)).unwrap().len() > self.queue_size {
            self.charge_b.get_mut(&(h.paddle_id as u8)).unwrap().pop_front();
          }
          let ch_a_bl = h.get_bl_a();
          let ch_b_bl = h.get_bl_b();
          //let ch_a_bl_rms = h.get_bl_a_rms();
          //let ch_b_bl_rms = h.get_bl_b_rms();
          // cut on the range
          if -2.0 < ch_a_bl && ch_b_bl < 2.0 {
            self.baseline_ch_a.get_mut(&(h.paddle_id as u8)).unwrap().fill(&ch_a_bl);
          }
          if -2.0 < ch_b_bl && ch_b_bl < 2.0 {
            self.baseline_ch_b.get_mut(&(h.paddle_id as u8)).unwrap().fill(&ch_b_bl);
          }
          //self.baseline_ch_a.get_mut(&(h.paddle_id as u8)).unwrap().fill(&h.get_bl_a());
          self.baseline_rms_ch_a.get_mut(&(h.paddle_id as u8)).unwrap().fill(&h.get_bl_a_rms());
          //self.baseline_ch_b.get_mut(&(h.paddle_id as u8)).unwrap().fill(&h.get_bl_b());
          self.baseline_rms_ch_b.get_mut(&(h.paddle_id as u8)).unwrap().fill(&h.get_bl_b_rms());
        }
        let wfs  = ev.get_rbwaveforms();
        let mut bl_a  : f32;
        let mut bl_b  : f32;
        let rms_a = 0f32;
        let rms_b = 0f32;
        for mut wf in wfs {
          // FIXME - this is an incpmplete cosistency check and 
          // should be removed soon
          if wf.paddle_id == self.current_paddle.paddle_id as u8 {
            let rb_channel_a = wf.rb_channel_a + 1;
            let rb_channel_b = wf.rb_channel_b + 1;
            if (rb_channel_a != self.current_paddle.rb_chA as u8 ) 
               || (rb_channel_b != self.current_paddle.rb_chB as u8 ) {
              error!("Inconsistent paddle RB channels! Maybe A and B are switched!");
            }
          match self.calibrations.lock() {
            Err(_err) => error!("Unable to get lock on rbcalibrations!"),
            Ok(cali) => {
              match cali.get(&wf.rb_id) {
                None => error!("RBCalibrations for board {} not available!", wf.rb_id),
                Some(rbcal) => {
                  match wf.calibrate(rbcal) {
                    Err(err) => error!("Calibration error! {err}"),
                    Ok(_) => ()
                  }
                }
              }
            }
          }
          self.wf.get_mut(&(wf.paddle_id as u8)).unwrap().push_back(wf.clone());
          if self.wf.get_mut(&(wf.paddle_id as u8)).unwrap().len() > self.queue_size {
            self.wf.get_mut(&(wf.paddle_id as u8)).unwrap().pop_front();
          }
        }
        return Ok(());
      }
    }
  }

  // Color::Blue was nice for background
  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
   
    match self.menu.get_active_menu_item() {
      UIMenuItem::Back => {
        // as usual, layout first
        let main_lo = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(15), Constraint::Percentage(85)].as_ref(),
          )
          .split(*main_window);
        let pdl = Block::default()
          .borders(Borders::ALL)
          .style(self.theme.style())
          .title("Select Paddle")
          .border_type(BorderType::Plain);
        let pd_select_list = List::new(self.pdl_items.clone()).block(pdl)
          .highlight_style(self.theme.highlight().add_modifier(Modifier::BOLD))
          .highlight_symbol(">>")
          .repeat_highlight_symbol(true);
        match self.pdl_state.selected() {
          None    => {
            let selector =  1;
            if self.pdl_selector != selector {
              self.pdl_changed = true;
              self.pdl_selector = selector;
            } else {
              self.pdl_changed = false;
            }
          },
          Some(_pid) => {
            let selector =  _pid + 1;
            if self.pdl_selector != selector {
              self.pdl_changed = true;
              self.pdl_selector = selector;
            } else {
              self.pdl_changed = false;
            }
          }
        }
        let view_string : String;
        match self.all_paddles.get(&(self.pdl_selector as u8)) {
          Some(_pd) => {
            view_string = format!("{}", _pd);
            self.current_paddle = _pd.clone();
          }
          None => {
            view_string = format!("No information for Paddle {} in DB or DB not available!", self.pdl_selector);
          }
        }
        let pd_view = Paragraph::new(view_string)
          .style(self.theme.style())
          .alignment(Alignment::Left)
          //.scroll((5, 10))
          .block(
          Block::default()
            .borders(Borders::ALL)
            .style(self.theme.style())
            .title("Paddle")
            .border_type(BorderType::Rounded),
        );
        frame.render_stateful_widget(pd_select_list,  main_lo[0], &mut self.pdl_state );
        frame.render_widget(pd_view, main_lo[1]);
      }
      UIMenuItem::Signal => {
        let main_lo = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(50),
               Constraint::Percentage(50)].as_ref(),
          )
          .split(*main_window);
        let wf_lo = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
            [Constraint::Percentage(40),
             Constraint::Percentage(40),
             Constraint::Percentage(20)].as_ref(),
          )
          .split(main_lo[0]);
        let ch_lo = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
            [Constraint::Percentage(80),
             Constraint::Percentage(20)].as_ref(),
          )
          .split(main_lo[1]);
        let bla_lo = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
            [Constraint::Percentage(50),
             Constraint::Percentage(50)].as_ref(),
          )
          .split(wf_lo[2]);
        let blb_lo = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
            [Constraint::Percentage(50),
             Constraint::Percentage(50)].as_ref(),
          )
          .split(ch_lo[1]);

        let mut wf_data_a = VecDeque::<(f64, f64)>::new();    
        let mut wf_data_b = VecDeque::<(f64, f64)>::new();  
        //let mut label_a   = String::from("");
        //let mut label_b   = String::from("");
        let wf_theme      = self.theme.clone();
        match self.wf.get_mut(&(self.current_paddle.paddle_id as u8)).unwrap().pop_front() {
          None => {
            //for (i,k) in wf.adc.iter().enumerate() {
            //  wf_data_a.push_back((i as f64, *k as f64));
            //}
          }
          Some(wf) => {
            //label_a  = format!("Paddle {}A, RB {}-{}",self.current_paddle.paddle_id, wf.rb_id, wf.rb_channel_a + 1);
            if wf.voltages_a.len() == 0 {
              for (i,k) in wf.adc_a.iter().enumerate() {
                wf_data_a.push_back((i as f64, *k as f64));
              }
            } else {
              for k in 0..wf.nanoseconds_a.len() {
                wf_data_a.push_back((wf.nanoseconds_a[k] as f64, wf.voltages_a[k] as f64));
              }
            }
            self.last_wf_ch_a = wf_data_a.clone();
            //label_b  = format!("Paddle {}B, RB {}-{}",self.current_paddle.paddle_id, wf.rb_id, wf.rb_channel_b + 1);
            if wf.voltages_b.len() == 0 {
              for (i,k) in wf.adc_b.iter().enumerate() {
                wf_data_b.push_back((i as f64, *k as f64));
              }
            } else {
              for k in 0..wf.nanoseconds_b.len() {
                wf_data_b.push_back((wf.nanoseconds_b[k] as f64, wf.voltages_b[k] as f64));
              }
            }
            self.last_wf_ch_b = wf_data_b.clone();
          }
        }
        
        if wf_data_a.len() == 0 {
          wf_data_a = self.last_wf_ch_a.clone();
        }
        let wf_theme_a = wf_theme.clone();
        let wf_chart_a = timeseries(&mut wf_data_a,
                                    self.wf_label_a.clone(),
                                    self.wf_label_a.clone(),
                                    &wf_theme_a);
        if wf_data_b.len() == 0 {
          wf_data_b = self.last_wf_ch_b.clone();
        }
        let wf_theme_b = wf_theme.clone();
        let wf_chart_b = timeseries(&mut wf_data_b,
                                    self.wf_label_b.clone(),
                                    self.wf_label_b.clone(),
                                    &wf_theme_b);
        frame.render_widget(wf_chart_a, wf_lo[0]);
        frame.render_widget(wf_chart_b, wf_lo[1]);
        
        // 2d charge plot
        let mut ch2d_points = Vec::<(f64, f64)>::new();
        for k in 0..self.charge_a.get(&(self.current_paddle.paddle_id as u8)).unwrap().len() {
          ch2d_points.push((self.charge_a.get(&(self.current_paddle.paddle_id as u8)).unwrap()[k],
                            self.charge_b.get(&(self.current_paddle.paddle_id as u8)).unwrap()[k]));
        }

        let charge_plot = Canvas::default()
          .block(Block::bordered().title("Charge AvsB"))
          .marker(Marker::Braille)
          .paint(|ctx| {
            // let xaxis  = canvas::Line {
            //   x1 : 0.0,
            //   x2 : 30.0,
            //   y1 : 0.0,
            //   y2 : 0.0,
            //   color : self.theme.fg0
            // };
            // let yaxis  = canvas::Line {
            //   x1 : 0.0,
            //   x2 : 0.0,
            //   y1 : 0.0,
            //   y2 : 30.0,
            //   color : self.theme.fg0
            // };
            let points = Points {
              coords : &ch2d_points.as_slice(),
              color  : self.theme.hc,
            };
            ctx.draw(&points);
            //ctx.draw(&xaxis);
            //ctx.draw(&yaxis);
          })
          .x_bounds([0.0, 200.0])
          .y_bounds([0.0, 200.0]);
       
      // baseline histos
      //println!("{:?}", self.baseline_ch_a.get(&(self.current_paddle.paddle_id as u8)).unwrap());
      let bl_a_labels     = create_labels(&self.baseline_ch_a.get(&(self.current_paddle.paddle_id as u8)).unwrap());
      let bl_a_data       = prep_data(&self.baseline_ch_a.get(&(self.current_paddle.paddle_id as u8)).unwrap(), &bl_a_labels, 1, false); 
      let bl_a_chart      = histogram(bl_a_data, String::from("Baseline Side A [mV]"), 2, 0, &self.theme);
      frame.render_widget(bl_a_chart, bla_lo[0]);
      
      let bl_a_rms_data   = prep_data(&self.baseline_rms_ch_a.get(&(self.current_paddle.paddle_id as u8)).unwrap(), &bl_a_labels, 1, false); 
      let bl_a_rms_chart  = histogram(bl_a_rms_data, String::from("Baseline RMS Side A [mV]"), 2, 0, &self.theme);
      frame.render_widget(bl_a_rms_chart, bla_lo[1]);
      
      // B side
      // let bl_b_labels = create_labels(&self.baseline_ch_b.get(&(self.current_paddle.paddle_id as u8)).unwrap());
      let bl_b_data   = prep_data(&self.baseline_ch_b.get(&(self.current_paddle.paddle_id as u8)).unwrap(), &bl_a_labels, 1, false); 
      let bl_b_chart  = histogram(bl_b_data, String::from("Baseline Side B [mV]"), 2, 0, &self.theme);
      frame.render_widget(bl_b_chart, blb_lo[0]);
      
      // let bl_b_rms_labels = create_labels(&self.baseline_rms_ch_b.get(&(self.current_paddle.paddle_id as u8)).unwrap());
      let bl_b_rms_data   = prep_data(&self.baseline_rms_ch_b.get(&(self.current_paddle.paddle_id as u8)).unwrap(), &bl_a_labels, 1, false); 
      let bl_b_rms_chart  = histogram(bl_b_rms_data, String::from("Baseline RMS Side B [mV]"), 2, 0, &self.theme);
      frame.render_widget(bl_b_rms_chart, blb_lo[1]);

      
      //charge_plot.
      //  .marker(Marker::Dot)
      //  .paint(|ctx| {
      //    let mut points = Points {
      //      coords : &[(10.0, 10.0)],
      //      color  : self.theme.hc,
      //    };
      //    ctx.draw(&points);
      //  }
      //);
      frame.render_widget(charge_plot, ch_lo[0]);
      }
      _ => ()
    } // end match
  } 
}
