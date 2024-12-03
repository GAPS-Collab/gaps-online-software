use std::time::Instant;
use std::collections::VecDeque;

use crossbeam_channel::Receiver;

use ratatui::symbols::line::*;
use ratatui::{
    //backend::CrosstermBackend,
    //symbols, 
    Frame,
    layout::{
        Alignment,
        Constraint,
        Direction,
        Layout,
        Rect
    },
    //widgets::Paragraph,
    style::{Modifier, Color, Style},
    //text::{Span, Line},
    widgets::{
        Block, BorderType, Borders, LineGauge,
        //List, ListItem, ListState,
        Paragraph},
};

use tof_dataclasses::packets::{
    TofPacket,
    PacketType,
};
use tof_dataclasses::monitoring::CPUMoniData;
use tof_dataclasses::errors::SerializationError;
//use tof_dataclasses::serialization::{
//    Serialization,
//    Packable
//};

use crate::colors::ColorTheme;
use crate::widgets::timeseries;

//pub const LG_LINE_HORIZONTAL : &str = "◉";
//pub const LG_LINE_HORIZONTAL : &str = "▥";
pub const LG_LINE_HORIZONTAL : &str = "░";
pub const LG_LINE: Set = Set {
        vertical         : THICK_VERTICAL,
        //horizontal       : THICK_HORIZONTAL,
        horizontal       : LG_LINE_HORIZONTAL,
        top_right        : THICK_TOP_RIGHT,
        top_left         : THICK_TOP_LEFT,
        bottom_right     : THICK_BOTTOM_RIGHT,
        bottom_left      : THICK_BOTTOM_LEFT,
        vertical_left    : THICK_VERTICAL_LEFT,
        vertical_right   : THICK_VERTICAL_RIGHT,
        horizontal_down  : THICK_HORIZONTAL_DOWN,
        horizontal_up    : THICK_HORIZONTAL_UP,
        cross            : THICK_CROSS,
};


#[derive(Debug, Clone)]
pub struct CPUTab {
  pub theme      : ColorTheme,
  pub freq_queue  : Vec<VecDeque<(f64,f64)>>,
  pub temp_queue  : Vec<VecDeque<(f64,f64)>>,
  pub disk_usage : u8, // disk usage in per cent
  pub tp_recv    : Receiver<TofPacket>,
  timer          : Instant,
  queue_size     : usize,
  pub last_moni  : CPUMoniData,
}

impl CPUTab {

  pub fn new(tp_recv : Receiver<TofPacket>,
             theme : ColorTheme) -> Self {
    let queue_size    = 1000usize;
    let mut freq_queue = Vec::<VecDeque::<(f64,f64)>>::with_capacity(4);
    let mut temp_queue = Vec::<VecDeque::<(f64,f64)>>::with_capacity(4);

    for _core in 0..4 {
      let core_queue  = VecDeque::<(f64,f64)>::with_capacity(queue_size);
      freq_queue.push(core_queue);
    }
    for _core in 0..4 {
      let core_queue  = VecDeque::<(f64,f64)>::with_capacity(queue_size);
      temp_queue.push(core_queue);
    }


    CPUTab {
      theme      : theme,
      timer      : Instant::now(),
      freq_queue  : freq_queue,
      temp_queue  : temp_queue,
      disk_usage : 0u8,
      tp_recv    : tp_recv,
      queue_size : 1000usize,
      last_moni  : CPUMoniData::new(),
    }
  }
  
  pub fn receive_packet(&mut self) -> Result<(), SerializationError> {
    let moni : CPUMoniData;// CPUMoniData::new();
    let met      = self.timer.elapsed().as_secs_f64();
    match self.tp_recv.try_recv() {
      Err(err)   => {
        trace!("Can't receive packet! {err}");
        return Ok(())
      }
      Ok(pack)    => {
        trace!("Got next packet {}!", pack);
        match pack.packet_type {
          PacketType::CPUMoniData => {
            //moni = CPUMoniData::from_bytestream(&pack.payload, &mut 0)?;
            // new API!
            moni = pack.unpack()?;
          }
          _ => {
            return Ok(());
          },
        }
      } 
    }
    if moni.disk_usage == u8::MAX {
      error!("CPUInfo packet only contains error vals!");
      return Ok(());
    }

    let temps = moni.get_temps();
    for core in 0..4 {
      self.freq_queue[core].push_back((met, moni.cpu_freq[core] as f64));
      if self.freq_queue[core].len() > self.queue_size {
        self.freq_queue[core].pop_front();
      }
      
      self.temp_queue[core].push_back((met, temps[core] as f64));
      if self.temp_queue[core].len() > self.queue_size {
        self.temp_queue[core].pop_front();
      }
    }
    self.disk_usage = moni.disk_usage;
    self.last_moni  = moni;
    Ok(())
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {
        let main_chunks = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(30), Constraint::Percentage(70)].as_ref(),
          )
          .split(*main_window);
        let main_cols0 = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(90),
               Constraint::Percentage(10)].as_ref(),
          )
          .split(main_chunks[0]);
      
        let graph_chunks = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
              [Constraint::Percentage(50),
               Constraint::Percentage(50)].as_ref(),
          )
          .split(main_chunks[1]).to_vec();

        
        let freq_chunks = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(25),
               Constraint::Percentage(25),
               Constraint::Percentage(26),
               Constraint::Percentage(25)].as_ref(),
          )
          .split(graph_chunks[0]).to_vec();
        
        let temp_chunks = Layout::default()
          .direction(Direction::Vertical)
          .constraints(
              [Constraint::Percentage(25),
               Constraint::Percentage(25),
               Constraint::Percentage(26),
               Constraint::Percentage(25)].as_ref(),
          )
          .split(graph_chunks[1]).to_vec();


        let info_view_str = format!("{}", self.last_moni);

        let info_view = Paragraph::new(info_view_str)
        .style(self.theme.style())
        .alignment(Alignment::Left)
        .block(
          Block::default()
            .borders(Borders::ALL)
            .style(self.theme.style())
            .title("Info")
            .border_type(BorderType::Rounded),
        );
        //let foo_par3 = Paragraph::new(foo3)
        //.style(self.theme.style())
        //.alignment(Alignment::Left)
        //.block(
        //  Block::default()
        //    .borders(Borders::ALL)
        //    .style(self.theme.style())
        //    .title("Overview")
        //    .border_type(BorderType::Rounded),
        //);
        
        // disc usage gauge
        //self.disk_usage = 42;
        
        let mut ratio = self.disk_usage as f64/100.0;
        if ratio > 1.00 {
          error!("TOF CPU disk filled to more than 100%");
          ratio = 0.0;
        }
        let fg_color  : Color;
        if self.disk_usage > 80 {
          fg_color   = Color::Red; // this should be an 
                                      // alert color
        } else {
          fg_color = self.theme.hc;
        }

        let label_str = format!("Disc usage {} %", self.disk_usage);
        let du_gauge = LineGauge::default()
          .block(
            Block::default()
            .borders(Borders::ALL)
            .style(self.theme.style())
            .title("Disk usage (/tpool)")
            .border_type(BorderType::Rounded)
          )
          .filled_style(
            Style::default()
              .fg(fg_color)
              .bg(self.theme.bg1)
              .add_modifier(Modifier::BOLD)
          )
          //.use_unicode(true)
          .label(label_str)
          //.line_set(symbols::line::THICK)  // THICK
          .line_set(LG_LINE)
          //.percent(self.disk_usage as u16);
          .ratio(ratio);
        //let foo_par4 = Paragraph::new(foo4)
        //.style(self.theme.style())
        //.alignment(Alignment::Left)
        //.block(
        //  Block::default()
        //    .borders(Borders::ALL)
        //    .style(self.theme.style())
        //    .title("Overview")
        //    .border_type(BorderType::Rounded),
        //);
        for core in 0..4 {
          let label            = format!("Core{} freq. [GHz]", core);
          let core_theme       = self.theme.clone();
          let mut freq_ts_data = VecDeque::from(self.freq_queue[core].clone());
          let freq_ts = timeseries(&mut freq_ts_data,
                                   label.clone(),
                                   label.clone(),
                                   &core_theme  );
          frame.render_widget(freq_ts,freq_chunks[core]);
        }
       
        let temp_labels = vec!["Core0 T [\u{00B0}C]", 
                               "Core1 T [\u{00B0}C]",
                               "CPU   T [\u{00B0}C]",
                               "MB    T [\u{00B0}C]"];

        for core in 0..4 {
          let label            = temp_labels[core].to_string();
          let core_theme       = self.theme.clone();
          let mut temp_ts_data = VecDeque::from(self.temp_queue[core].clone());
          let temp_ts = timeseries(&mut temp_ts_data,
                                   label.clone(),
                                   label.clone(),
                                   &core_theme  );
          frame.render_widget(temp_ts,temp_chunks[core]);
        }
        frame.render_widget(info_view, main_cols0[0]);
        frame.render_widget(du_gauge, main_cols0[1]);

        //let mut ch_chunks = Layout::default()
        //  .direction(Direction::Vertical)
        //  .constraints(
        //      [Constraint::Percentage(25),
        //       Constraint::Percentage(25),
        //       Constraint::Percentage(26),
        //       Constraint::Percentage(25)].as_ref(),
        //  )
        //  .split(wf_chunks[0]).to_vec();
  }
}
