//! Commands tab
//! 
//use chrono::Utc;

use ratatui::prelude::*;
use ratatui::widgets::{
    Block,
    BorderType,
    Borders,
    //Paragraph,
    //BarChart,
    List,
    ListItem,
    ListState,
};

use std::collections::VecDeque;
use crossbeam_channel::{
    //unbounded,
    //Sender,
    Receiver
};

use tof_dataclasses::packets::{
  TofPacket,
  PacketType
};
use tof_dataclasses::commands::{
  TofCommandV2,
  TofCommandCode,
  TofResponse
};

use tof_dataclasses::serialization::{
    Serialization,
    SerializationError,
    Packable
};

use crate::colors::ColorTheme;

pub struct CommandTab<'a> {
  pub resp_rc            : Receiver<TofPacket>,
  pub theme              : ColorTheme,
  pub cmd_sender         : zmq::Socket,
  // list for the command selector
  pub cmdl_state         : ListState,
  pub cmdl_items         : Vec::<ListItem<'a>>,
  pub cmdl_active        : bool,
  pub cmdl_selector      : usize,
  pub allow_commands     : bool,
  pub active_cmd         : TofCommandV2,
  pub commands           : Vec<TofCommandV2>,
  pub queue_size         : usize,
  pub tr_queue           : VecDeque<TofResponse>,
}


impl CommandTab<'_> {

  pub fn new<'a>(resp_rc        : Receiver<TofPacket>,
                 cmd_pub_addr   : String,
                 theme          : ColorTheme,
                 allow_commands : bool) -> CommandTab<'a> {  
    let mut ping_cmd        = TofCommandV2::new();
    ping_cmd.command_code   = TofCommandCode::Ping;
    let mut start_cmd       = TofCommandV2::new();
    start_cmd.command_code  = TofCommandCode::DataRunStart;
    let mut stop_cmd        = TofCommandV2::new();
    stop_cmd.command_code   = TofCommandCode::DataRunStop;
    let mut cali_cmd        = TofCommandV2::new();
    cali_cmd.command_code   = TofCommandCode::RBCalibration;
    
    let mut send_te         = TofCommandV2::new();
    send_te.command_code    = TofCommandCode::SendTofEvents;
    let mut no_send_te      = TofCommandV2::new();
    no_send_te.command_code = TofCommandCode::NoSendTofEvents;
    
    let mut send_rw         = TofCommandV2::new();
    send_rw.command_code    = TofCommandCode::SendRBWaveforms;
    let mut no_send_rw      = TofCommandV2::new();
    no_send_rw.command_code = TofCommandCode::NoSendRBWaveforms;

    let mut kill_cmd        = TofCommandV2::new();
    kill_cmd.command_code   = TofCommandCode::Kill;


    let commands = vec![ping_cmd,
                        start_cmd,
                        stop_cmd,
                        cali_cmd,
                        send_te,
                        no_send_te,
                        send_rw,
                        no_send_rw,
                        kill_cmd];

    let mut cmd_select_items = Vec::<ListItem>::new();
    for k in &commands {
      let this_item = format!("{:?}", k.command_code);
      cmd_select_items.push(ListItem::new(Line::from(this_item)));
    } 
    let ctx = zmq::Context::new();
    let cmd_sender = ctx.socket(zmq::PUB).expect("Can not create 0MQ PUB socket!"); 
    if allow_commands {
      cmd_sender.bind(&cmd_pub_addr).expect("Unable to bind to (PUB) socket!");
    }
    //thread::sleep(10*one_second);

    CommandTab {
      theme          ,
      resp_rc        ,
      cmd_sender     ,
      cmdl_state     : ListState::default(),
      cmdl_items     : cmd_select_items,
      cmdl_active    : false,
      cmdl_selector  : 0,
      allow_commands : allow_commands,
      active_cmd     : TofCommandV2::new(),
      commands       : commands,
      queue_size     : 1000,
      tr_queue       : VecDeque::<TofResponse>::new(),
    }
  }
  
  pub fn next_cmd(&mut self) {
    let i = match self.cmdl_state.selected() {
      Some(i) => {
        if i >= self.cmdl_items.len() - 1 {
          self.cmdl_items.len() - 1
        } else {
          i + 1
        }
      }
      None => 0,
    };
    self.cmdl_state.select(Some(i));
    self.active_cmd = self.commands.get(i).unwrap().clone();
    //info!("Selecting {}", i);
  }

  pub fn prev_cmd(&mut self) {
    let i = match self.cmdl_state.selected() {
      Some(i) => {
        if i == 0 {
          0 
        } else {
          i - 1
        }
      }
      None => 0,
    };
    self.cmdl_state.select(Some(i));
    self.active_cmd = self.commands.get(i).unwrap().clone();
  }
 
  /// send the selected dommand
  pub fn send_command(&self) {
    if !self.allow_commands {
      error!("To send commands, run program with --send-commands!");
      return;
    }
    let payload = self.active_cmd.pack().to_bytestream();
    match self.cmd_sender.send(&payload, 0) {
      Err(err) => {
        error!("Unable to send command! {err}");
      },
      Ok(_) => {
        info!("TOF cmd {} sent!", self.active_cmd);
      }
    }  
  }

  /// Get the responses from the main program packet distributor
  pub fn receive_packet(&mut self) -> Result<(), SerializationError> {
    match self.resp_rc.try_recv() {
      Err(_err)   => {
        debug!("Unable to receive ACK TofPacket!");  
      }
      Ok(pack)    => {
        //println!("Received {}", pack);
        match pack.packet_type {
          PacketType::TofResponse => {
            let tr : TofResponse = pack.unpack()?;
            self.tr_queue.push_back(tr);
            if self.tr_queue.len() > self.queue_size {
              self.tr_queue.pop_front(); 
            }
          }
          _ => () // we don't care
        }
      }
    }
    Ok(())
  }

  pub fn render(&mut self, main_window : &Rect, frame : &mut Frame) {

    let main_lo = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(
          [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
      )
      .split(*main_window);
       let par_title_string = String::from("Select Command");
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

    let cmds = Block::default()
      .borders(Borders::ALL)
      .style(self.theme.style())
      .title(par_title)
      .border_type(BorderType::Plain);
    
    let cmd_select_list = List::new(self.cmdl_items.clone()).block(cmds)
      .highlight_style(self.theme.highlight().add_modifier(Modifier::BOLD))
      .highlight_symbol(">>")
      .repeat_highlight_symbol(true);
    
    match self.cmdl_state.selected() {
      None    => {
        self.cmdl_selector = 0;
      },
      Some(cmd_id) => {
        // entry 0 is for all paddles
        let selector =  cmd_id;
        if self.cmdl_selector != selector {
          //self.paddle_changed = true;
          //self.init_histos();
          self.cmdl_selector = selector;
        } else {
          //self.paddle_changed = false;
        }
      },
    }
    if self.allow_commands {
      frame.render_stateful_widget(cmd_select_list, main_lo[0], &mut self.cmdl_state );
    }
  }
} // end impl
    
