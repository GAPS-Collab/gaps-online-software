use std::collections::{
    //HashMap,
    VecDeque,
};

use crossbeam_channel::{
    Receiver,
    //Sender,
};

use ratatui::prelude::*;
use ratatui::widgets::{
    Block,
    BorderType,
    Borders,
    Paragraph,
    //BarChart,
    List,
    ListItem,
    ListState,
};

use ratatui::terminal::Frame;
use ratatui::layout::Rect;

use ndhistogram::{
    ndhistogram,
    Histogram,
    Hist1D,
};

use ndhistogram::axis::{
    Uniform,
};

use tof_dataclasses::events::{
    //RBEvent,
    //TofEvent,
    TofHit,
    //TofEventHeader,
    //MasterTriggerEvent,
};

use tof_dataclasses::errors::SerializationError;

use crate::colors::ColorTheme;
use crate::widgets::{
    //clean_data,
    prep_data,
    create_labels,
    histogram,
};

#[derive(Debug, Clone)]
pub enum TofHitView {
  Hits,
  Pulses,
  Paddles,
  SelectPaddle,
}

#[derive(Debug, Clone)]
pub struct TofHitTab<'a> {
  pub theme           : ColorTheme,
  pub th_recv         : Receiver<TofHit>,
  pub hit_queue       : VecDeque<TofHit>,
  pub queue_size      : usize,
  // pulse height
  pub pha_histo       : Hist1D<Uniform<f32>>,
  // pulse time
  pub pta_histo       : Hist1D<Uniform<f32>>,
  // pulse charge
  pub pca_histo       : Hist1D<Uniform<f32>>,
  // pulse height
  pub phb_histo       : Hist1D<Uniform<f32>>,
  // pulse time
  pub ptb_histo       : Hist1D<Uniform<f32>>,
  // pulse charge
  pub pcb_histo       : Hist1D<Uniform<f32>>,
  // pos acrross
  pub pa_histo        : Hist1D<Uniform<f32>>,
  // tzero           
  pub t0_histo        : Hist1D<Uniform<f32>>,
  pub edep_histo      : Hist1D<Uniform<f32>>,
  // paddle id
  pub pid_histo       : Hist1D<Uniform<f32>>,
  pub view            : TofHitView,
  pub paddle_selector : u8,
  pub paddle_changed  : bool,
  
  // list for the rb selector
  pub pl_state       : ListState,
  pub pl_items       : Vec::<ListItem<'a>>,
  pub pl_active      : bool,
}

impl TofHitTab<'_> {
  pub fn new(th_recv : Receiver<TofHit>, theme : ColorTheme) -> TofHitTab<'static> {
    let bins_ph    = Uniform::new(50, 0.0, 200.0);
    let bins_pt    = Uniform::new(50, 0.0, 400.0);
    let bins_pc    = Uniform::new(30, 0.0, 30.0);
    let bins_pa    = Uniform::new(90, 0.0, 1800.0);
    let bins_t0    = Uniform::new(50, 0.0, 200.0);
    let bins_pid   = Uniform::new(160,1.0, 161.0);
    let bins_edep  = Uniform::new(50,0.0,100.0);
    let mut paddle_select_items = Vec::<ListItem>::new();
    let all_pdl = String::from("  All paddles");
    paddle_select_items.push(ListItem::new(Line::from(all_pdl)));
    for k in 1..161 {
      let this_item = format!("  Paddle {:0>3}", k);
      paddle_select_items.push(ListItem::new(Line::from(this_item)));
    }
    TofHitTab {
      th_recv         : th_recv,
      hit_queue       : VecDeque::<TofHit>::new(),
      queue_size      : 3000,
      theme           : theme,
      pha_histo       : ndhistogram!(bins_ph.clone()),
      pta_histo       : ndhistogram!(bins_pt.clone()),
      pca_histo       : ndhistogram!(bins_pc.clone()),
      phb_histo       : ndhistogram!(bins_ph),
      ptb_histo       : ndhistogram!(bins_pt),
      pcb_histo       : ndhistogram!(bins_pc),
      pa_histo        : ndhistogram!(bins_pa),
      t0_histo        : ndhistogram!(bins_t0),
      edep_histo      : ndhistogram!(bins_edep),
      pid_histo       : ndhistogram!(bins_pid),
      view            : TofHitView::Pulses,
      paddle_selector : 0,
      paddle_changed  : false,
      
      pl_state        : ListState::default(),
      pl_items        : paddle_select_items,
      pl_active       : false,
    }
  }

  pub fn init_histos(&mut self) {
    let bins_ph     = Uniform::new(50, 0.0, 200.0);
    let bins_pt     = Uniform::new(50, 0.0, 400.0);
    let bins_pc     = Uniform::new(30, 0.0, 30.0);
    let bins_pa     = Uniform::new(90, 0.0, 1800.0);
    let bins_t0     = Uniform::new(50, 0.0, 200.0);
    let bins_edep   = Uniform::new(50,0.0,100.0);
    self.pha_histo  = ndhistogram!(bins_ph.clone());
    self.pta_histo  = ndhistogram!(bins_pt.clone());
    self.pca_histo  = ndhistogram!(bins_pc.clone());
    self.phb_histo  = ndhistogram!(bins_ph);
    self.ptb_histo  = ndhistogram!(bins_pt);
    self.pcb_histo  = ndhistogram!(bins_pc);
    self.pa_histo   = ndhistogram!(bins_pa);
    self.t0_histo   = ndhistogram!(bins_t0);
    self.edep_histo = ndhistogram!(bins_edep);
  }

  pub fn receive_packet(&mut self) -> Result<(), SerializationError> {  
    // FIXME - can/should this block or not?
    match self.th_recv.try_recv() {
      Err(_err) => {
        return Ok(());
      },
      Ok(hit)    => {
        self.hit_queue.push_back(hit);
        if self.hit_queue.len() > self.queue_size {
          self.hit_queue.pop_front();
        }
        // never filter pid histogram
        self.pid_histo.fill(&(hit.paddle_id as f32));
        if self.paddle_selector != 0 {
          if hit.paddle_id != self.paddle_selector {
            return Ok(());
          }
        }
        self.pha_histo.fill(&hit.get_peak_a());
        self.phb_histo.fill(&hit.get_peak_b());
        self.pta_histo.fill(&hit.get_time_a());
        self.ptb_histo.fill(&hit.get_time_b());
        self.pca_histo.fill(&hit.get_charge_a());
        self.pcb_histo.fill(&hit.get_charge_b());
        self.t0_histo.fill(&(hit.get_t0()));
        self.pa_histo.fill(&(hit.get_pos()));
        self.edep_histo.fill(&(hit.get_edep()));
        return Ok(());
      }
    }
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
   
    match self.view {
      TofHitView::Pulses => {
        // as usual, layout first
        let chunks = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(50),
               Constraint::Percentage(50)].as_ref(),
          )
          .split(*main_window);
        let plots_a = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(33),
               Constraint::Percentage(33),
               Constraint::Percentage(34),
              ].as_ref(),
          )
          .split(chunks[0]);
        let plots_b = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(33),
               Constraint::Percentage(33),
               Constraint::Percentage(34),
              ].as_ref(),
          )
          .split(chunks[1]);

        // histograms
        let ph_labels  = create_labels(&self.pha_histo);
        let pha_data   = prep_data(&self.pha_histo, &ph_labels, 5, false); 
        let pha_chart  = histogram(pha_data, String::from("Pulse height SideA [mV]"), 2, 0, &self.theme);
        frame.render_widget(pha_chart, plots_a[0]);
        let phb_data   = prep_data(&self.phb_histo, &ph_labels, 5, false); 
        let phb_chart  = histogram(phb_data, String::from("Pulse height SideB [mV]"), 2, 0, &self.theme);
        frame.render_widget(phb_chart, plots_b[0]);
        
        let pt_labels  = create_labels(&self.pta_histo);
        let pta_data   = prep_data(&self.pta_histo, &pt_labels, 5, false); 
        let pta_chart  = histogram(pta_data, String::from("Pulse time SideA [a.u.]"), 2, 0, &self.theme);
        frame.render_widget(pta_chart, plots_a[1]);

        let ptb_data   = prep_data(&self.ptb_histo, &pt_labels, 5, false); 
        let ptb_chart  = histogram(ptb_data, String::from("Pulse time SideB [a.u.]"), 2, 0, &self.theme);
        frame.render_widget(ptb_chart, plots_b[1]);
        
        let pc_labels  = create_labels(&self.pca_histo);
        let pca_data   = prep_data(&self.pca_histo, &pc_labels, 5, false); 
        let pca_chart  = histogram(pca_data, String::from("Pulse charge SideA [mC]"), 2, 0, &self.theme);
        frame.render_widget(pca_chart, plots_a[2]);

        let pcb_data   = prep_data(&self.pcb_histo, &pc_labels, 5, false); 
        let pcb_chart  = histogram(pcb_data, String::from("Pulse charge SideB [mC]"), 2, 0, &self.theme);
        frame.render_widget(pcb_chart, plots_b[2]);
        
      },
      TofHitView::Hits => {
        let chunks = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(60), Constraint::Percentage(40)].as_ref(),
          )
          .split(*main_window);
        let mut hit_string = String::from("No HIT");
        match self.hit_queue.back() {
          None => (),
          Some(hit)   => {
            hit_string = hit.to_string();
          }
        }
        let hit_view   = Paragraph::new(hit_string)
          .style(self.theme.style())
          .alignment(Alignment::Left)
          .block(
            Block::default()
              .borders(Borders::ALL)
              .border_type(BorderType::Rounded)
              .title("Last TofHit")
          );
        frame.render_widget(hit_view, chunks[0]);
      },
      TofHitView::Paddles => {
        let chunks = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(33),
               Constraint::Percentage(33),
               Constraint::Percentage(34)].as_ref(),
          )
          .split(*main_window);
        let plots = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(50),
               Constraint::Percentage(50)].as_ref(),
          )
          .split(chunks[0]);
        
        // histograms
        let t0_labels  = create_labels(&self.t0_histo);
        let t0_data    = prep_data(&self.t0_histo, &t0_labels, 10, false); 
        let t0_chart   = histogram(t0_data, String::from("Reco. T0"), 2, 0, &self.theme);
        frame.render_widget(t0_chart, plots[0]);

        let edep_labels  = create_labels(&self.edep_histo);
        let edep_data    = prep_data(&self.edep_histo, &edep_labels, 5, false); 
        let edep_chart   = histogram(edep_data, String::from("Reco. EDep"), 2, 0, &self.theme);
        frame.render_widget(edep_chart, plots[1]);
        
        // position across paddle
        let pa_labels = create_labels(&self.pa_histo);
        let pa_data   = prep_data(&self.pa_histo, &pa_labels, 20, false); 
        let pa_chart  = histogram(pa_data, String::from("Position accross paddle"), 2, 0, &self.theme);
        frame.render_widget(pa_chart, chunks[1]);
        
        let pid_labels = create_labels(&self.pid_histo);
        let pid_data   = prep_data(&self.pid_histo, &pid_labels, 10, true); 
        let pid_chart  = histogram(pid_data, String::from("Paddle ID"), 3, 0, &self.theme);
        frame.render_widget(pid_chart, chunks[2]);
      },
      TofHitView::SelectPaddle => {
        let list_chunks = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
          )
          .split(*main_window);
        let par_title_string = String::from("Select Paddle ID");
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
        let paddles = Block::default()
          .borders(Borders::ALL)
          .style(self.theme.style())
          .title(par_title)
          .border_type(BorderType::Plain);
        let paddle_select_list = List::new(self.pl_items.clone()).block(paddles)
          .highlight_style(self.theme.highlight().add_modifier(Modifier::BOLD))
          .highlight_symbol(">>")
          .repeat_highlight_symbol(true);
        match self.pl_state.selected() {
          None    => {
            self.paddle_selector = 0;
            //let selector =  1;
            //if self.paddle_selector != selector {
            //  self.paddle_changed = true;
            //  self.paddle_selector = selector;
            //} else {
            //  self.paddle_changed = false;
            //}
          },
          Some(pid) => {
            // entry 0 is for all paddles
            let selector =  pid as u8;
            if self.paddle_selector != selector {
              self.paddle_changed = true;
              self.init_histos();
              self.paddle_selector = selector;
            } else {
              self.paddle_changed = false;
            }
          },
        }
        frame.render_stateful_widget(paddle_select_list, list_chunks[0], &mut self.pl_state );
      }
    }
  }
}
