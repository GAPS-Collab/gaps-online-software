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
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::events::{
  //RBEvent,
  //TofEvent,
  TofEventSummary,
  //TofHit,
  //TofEventHeader,
  //MasterTriggerEvent,
  RBWaveform,
};
use tof_dataclasses::calibrations::RBCalibrations;
use tof_dataclasses::database::Paddle;

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
  pub te_receiver        : Receiver<TofEventSummary>,
  pub event_queue        : VecDeque<TofEventSummary>,
  pub wf_receiver        : Receiver<TofPacket>,
  pub queue_size         : usize,
  pub menu               : PaddleMenu<'a>,
  pub wf                 : HashMap<u8, RBWaveform>,
  pub last_wf_ch_a       : HashMap<u8, VecDeque<(f64, f64)>>,
  pub last_wf_ch_b       : HashMap<u8, VecDeque<(f64, f64)>>,
  pub wf_label_a         : String,
  pub wf_label_b         : String,
  // baseline histograms
  pub calibrations       : HashMap<u8, RBCalibrations>,
  pub baseline_ch_a      : HashMap<u8, Hist1D<Uniform<f32>>>,
  pub baseline_ch_b      : HashMap<u8, Hist1D<Uniform<f32>>>,
  pub baseline_rms_ch_a  : HashMap<u8, Hist1D<Uniform<f32>>>,
  pub baseline_rms_ch_b  : HashMap<u8, Hist1D<Uniform<f32>>>,

  // energy depostion & relative position histograms
  pub h_edep     : HashMap<u8, Hist1D<Uniform<f32>>>,
  pub h_rel_pos  : HashMap<u8, Hist1D<Uniform<f32>>>,

  pub pca_histo  : HashMap<u8, Hist1D<Uniform<f32>>>,
  pub pcb_histo  : HashMap<u8, Hist1D<Uniform<f32>>>,
  pub pha_histo  : HashMap<u8, Hist1D<Uniform<f32>>>,
  pub phb_histo  : HashMap<u8, Hist1D<Uniform<f32>>>,
  pub pta_histo  : HashMap<u8, Hist1D<Uniform<f32>>>,
  pub ptb_histo  : HashMap<u8, Hist1D<Uniform<f32>>>,

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
  pub fn new(te_receiver : Receiver<TofEventSummary>,
             wf_receiver : Receiver<TofPacket>,
             all_paddles : HashMap<u8, Paddle>,
             calibrations: Arc<Mutex<HashMap<u8, RBCalibrations>>>,
             theme       : ColorTheme) -> Self {
    let theme_c = theme.clone();
    let mut pd_select_items = Vec::<ListItem>::new();
    for k in 1..161 {
      let this_item = format!("  Paddle{:0>3}", k);
      pd_select_items.push(ListItem::new(Line::from(this_item)));
    }
    // get calibrations
    let mut calibrations_cloned = HashMap::<u8, RBCalibrations>::new();
    match calibrations.lock() {
      Err(_err) => error!("Unable to get lock on rbcalibrations!"),
      Ok(cali) => {
        calibrations_cloned = cali.clone();
      }
    }
    let mut charge_a   = HashMap::<u8, VecDeque<f64>>::new();
    let mut charge_b   = HashMap::<u8, VecDeque<f64>>::new();
    let mut wf         = HashMap::<u8, RBWaveform>::new();
    let mut bl_ch_a    = HashMap::<u8, Hist1D<Uniform<f32>>>::new();
    let mut bl_ch_b    = HashMap::<u8, Hist1D<Uniform<f32>>>::new();
    let mut blrms_ch_a = HashMap::<u8, Hist1D<Uniform<f32>>>::new();
    let mut blrms_ch_b = HashMap::<u8, Hist1D<Uniform<f32>>>::new();
    let mut h_edep_    = HashMap::<u8, Hist1D<Uniform<f32>>>::new();
    let mut h_rel_pos_ = HashMap::<u8, Hist1D<Uniform<f32>>>::new();
    let mut pca_histo  = HashMap::<u8, Hist1D<Uniform<f32>>>::new();
    let mut pcb_histo  = HashMap::<u8, Hist1D<Uniform<f32>>>::new();
    let mut pha_histo  = HashMap::<u8, Hist1D<Uniform<f32>>>::new();
    let mut phb_histo  = HashMap::<u8, Hist1D<Uniform<f32>>>::new();
    let mut pta_histo  = HashMap::<u8, Hist1D<Uniform<f32>>>::new();
    let mut ptb_histo  = HashMap::<u8, Hist1D<Uniform<f32>>>::new();
    let bins_bl        = Uniform::new(20, -2.0, 2.0).unwrap();
    let bins_bl_rms    = Uniform::new(20, 0.0,  2.0).unwrap(); 
    let bins_edep      = Uniform::new(50, 0.0, 25.0).unwrap();
    let bins_relpos    = Uniform::new(50, 0.0, 1.2).unwrap(); 
    let bins_ph        = Uniform::new(50, 0.0, 200.0).unwrap();
    let bins_pt        = Uniform::new(50, 25.0, 350.0).unwrap();
    let bins_pc        = Uniform::new(30, 0.0, 40.0).unwrap();
    let mut lwf_ch_a   = HashMap::<u8, VecDeque<(f64, f64)>>::new();
    let mut lwf_ch_b   = HashMap::<u8, VecDeque<(f64, f64)>>::new();
    for pid in 1..161 {
      charge_a.insert(pid, VecDeque::<f64>::new());
      charge_b.insert(pid, VecDeque::<f64>::new());
      //wf.insert(pid, VecDeque::<RBWaveform>::new());
      wf.insert(pid, RBWaveform::new());
      bl_ch_a.insert(pid, ndhistogram!(bins_bl.clone()));
      bl_ch_b.insert(pid, ndhistogram!(bins_bl.clone()));
      blrms_ch_a.insert(pid, ndhistogram!(bins_bl_rms.clone()));
      blrms_ch_b.insert(pid, ndhistogram!(bins_bl_rms.clone()));
      h_edep_.insert(pid, ndhistogram!(bins_edep.clone()));
      h_rel_pos_.insert(pid, ndhistogram!(bins_relpos.clone()));
      pca_histo.insert(pid, ndhistogram!(bins_pc .clone()));
      pcb_histo.insert(pid, ndhistogram!(bins_pc .clone()));
      pha_histo.insert(pid, ndhistogram!(bins_ph .clone()));
      phb_histo.insert(pid, ndhistogram!(bins_ph .clone()));
      pta_histo.insert(pid, ndhistogram!(bins_pt .clone()));
      ptb_histo.insert(pid, ndhistogram!(bins_pt .clone()));
      lwf_ch_a.insert(pid, VecDeque::<(f64,f64)>::new());
      lwf_ch_b.insert(pid, VecDeque::<(f64,f64)>::new());
    }

    Self {
      theme,
      te_receiver,
      wf_receiver,
      event_queue       : VecDeque::<TofEventSummary>::new(),
      queue_size        : 10000, // enough points for histograms! 
                                 
      menu              : PaddleMenu::new(theme_c),
      wf                : wf,
      wf_label_a        : String::from("A"),
      wf_label_b        : String::from("B"),
      last_wf_ch_a      : lwf_ch_a,
      last_wf_ch_b      : lwf_ch_b,
      calibrations      : calibrations_cloned,
      baseline_ch_a     : bl_ch_a,
      baseline_ch_b     : bl_ch_b,
      baseline_rms_ch_a : blrms_ch_a,
      baseline_rms_ch_b : blrms_ch_b,
      h_edep            : h_edep_,
      h_rel_pos         : h_rel_pos_,
      pca_histo,
      pcb_histo,
      pha_histo,
      phb_histo,
      pta_histo,
      ptb_histo,
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
    match self.wf_receiver.try_recv() {
      Err(_err) => {
      }
      Ok(wf_pack)    => {
        let mut wf : RBWaveform = wf_pack.unpack()?;
        match self.calibrations.get(&wf.rb_id) {
          None => error!("RBCalibrations for board {} not available!", wf.rb_id),
          Some(rbcal) => {
            match wf.calibrate(rbcal) {
              Err(err) => error!("Calibration error! {err}"),
              Ok(_) => ()
            }
          }
        }
        if wf.paddle_id == self.current_paddle.paddle_id as u8 {
          let rb_channel_a = wf.rb_channel_a + 1;
          let rb_channel_b = wf.rb_channel_b + 1;
          if (rb_channel_a != self.current_paddle.rb_chA as u8 ) 
             || (rb_channel_b != self.current_paddle.rb_chB as u8 ) {
            error!("Inconsistent paddle RB channels! Maybe A and B are switched!");
          }
        }
        if wf.paddle_id == 0 {
          error!("Got waveform with padle id 0!");
        } else if wf.paddle_id > 160 {
          error!("Got paddle id which is too large! {}", wf.paddle_id);
        } else {
          let pid = wf.paddle_id as u8;
          *self.wf.get_mut(&pid).unwrap() = wf;
        }
      }
    }
    match self.te_receiver.try_recv() {
      Err(_err) => {
        return Ok(());
      },
      Ok(mut ev)    => {
        //let hits = ev.get_hits();
        // FIXME - get baselines from hits
        for h in &mut ev.hits {
          self.h_edep.get_mut(&(h.paddle_id as u8)).unwrap().fill(&h.get_edep());
          h.set_paddle(&self.current_paddle);
          let rel_pos = h.get_pos()/(self.current_paddle.length*10.0);
          self.h_rel_pos.get_mut(&h.paddle_id).unwrap().fill(&rel_pos);
          self.pha_histo.get_mut(&h.paddle_id).unwrap().fill(&h.get_peak_a());
          self.phb_histo.get_mut(&h.paddle_id).unwrap().fill(&h.get_peak_b());
          self.pca_histo.get_mut(&h.paddle_id).unwrap().fill(&h.get_charge_a());
          self.pcb_histo.get_mut(&h.paddle_id).unwrap().fill(&h.get_charge_b());
          self.pta_histo.get_mut(&h.paddle_id).unwrap().fill(&h.get_time_a());
          self.ptb_histo.get_mut(&h.paddle_id).unwrap().fill(&h.get_time_a());

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
        match self.wf.get_mut(&(self.current_paddle.paddle_id as u8)) {
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
            //*self.last_wf_ch_a.get_mut(&wf.paddle_id).unwrap() = wf_data_a;
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
            //*self.last_wf_ch_b.get_mut(&wf.paddle_id).unwrap() = wf_data_b;
          }
        }
        
        let wf_chart_a = timeseries(&mut wf_data_a,
                                    self.wf_label_a.clone(),
                                    self.wf_label_a.clone(),
                                    &wf_theme);
        let wf_chart_b = timeseries(&mut wf_data_b,
                                    self.wf_label_b.clone(),
                                    self.wf_label_b.clone(),
                                    &wf_theme);
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
        frame.render_widget(charge_plot, ch_lo[0]);
         
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
      }
      UIMenuItem::RecoVars => {
        let main_lo = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(60), Constraint::Percentage(40)].as_ref(),
          )
          .split(*main_window);
        let col_left = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
          )
          .split(main_lo[0]);
        let rows_right = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(33),
               Constraint::Percentage(33),
               Constraint::Percentage(34)].as_ref(),
          )
          .split(main_lo[1]);
        let rows_left_left = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(33),
               Constraint::Percentage(33),
               Constraint::Percentage(34)].as_ref(),
          )
          .split(col_left[0]);
        let rows_left_right = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(33),
               Constraint::Percentage(33),
               Constraint::Percentage(34)].as_ref(),
          )
          .split(col_left[1]);
        
        // histograms
        let pid        = self.current_paddle.paddle_id as u8;

        let ph_labels  = create_labels(&self.pha_histo.get(&pid).unwrap());
        let pha_data   = prep_data(&self.pha_histo.get(&pid).unwrap(), &ph_labels, 5, false); 
        let pha_chart  = histogram(pha_data, String::from("Pulse height SideA [mV]"), 2, 0, &self.theme);
        frame.render_widget(pha_chart, rows_left_left[0]);
        let phb_data   = prep_data(&self.phb_histo.get(&pid).unwrap(), &ph_labels, 5, false); 
        let phb_chart  = histogram(phb_data, String::from("Pulse height SideB [mV]"), 2, 0, &self.theme);
        frame.render_widget(phb_chart, rows_left_right[0]);
        
        let pt_labels  = create_labels(&self.pta_histo.get(&pid).unwrap());
        let pta_data   = prep_data(&self.pta_histo.get(&pid).unwrap(), &pt_labels, 5, false); 
        let pta_chart  = histogram(pta_data, String::from("Pulse time SideA [a.u.]"), 2, 0, &self.theme);
        frame.render_widget(pta_chart, rows_left_left[1]);

        let ptb_data   = prep_data(&self.ptb_histo.get(&pid).unwrap(), &pt_labels, 5, false); 
        let ptb_chart  = histogram(ptb_data, String::from("Pulse time SideB [a.u.]"), 2, 0, &self.theme);
        frame.render_widget(ptb_chart, rows_left_right[1]);
        
        let pc_labels  = create_labels(&self.pca_histo.get(&pid).unwrap());
        let pca_data   = prep_data(&self.pca_histo.get(&pid).unwrap(), &pc_labels, 5, false); 
        let pca_chart  = histogram(pca_data, String::from("Pulse charge SideA [mC]"), 2, 0, &self.theme);
        frame.render_widget(pca_chart, rows_left_left[2]);
        
        let pcb_data   = prep_data(&self.pcb_histo.get(&pid).unwrap(), &pc_labels, 5, false); 
        let pcb_chart  = histogram(pcb_data, String::from("Pulse charge SideB [mC]"), 2, 0, &self.theme);
        frame.render_widget(pcb_chart, rows_left_right[2]);

        // edep hist
        let edep_labels       = create_labels(&self.h_edep.get(&(self.current_paddle.paddle_id as u8)).unwrap());
        let edep_data         = prep_data(&self.h_edep.get    (&(self.current_paddle.paddle_id as u8)).unwrap(), &edep_labels, 1, false);
        let edep_chart        = histogram(edep_data, String::from("Reco. Energy Deposition [minI]"), 1, 0, &self.theme);
        frame.render_widget(edep_chart, rows_right[0]);
        // norm_pos
        let relpos_labels     = create_labels(&self.h_rel_pos.get(&(self.current_paddle.paddle_id as u8)).unwrap());
        let relpos_data       = prep_data(&self.h_rel_pos.get    (&(self.current_paddle.paddle_id as u8)).unwrap(), &relpos_labels, 1, false);
        let relpos_chart      = histogram(relpos_data, String::from("Rel. pos. (1 = B side)"), 1, 0, &self.theme);
        frame.render_widget(relpos_chart, rows_right[1]);
      }
      _ => ()
    } // end match
  } 
}
