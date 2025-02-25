//! Interactive display for the tof system for the 
//! GAPS experiment
//!
//!
//!
//!

use std::sync::{
    Arc,
    Mutex,
};

use std::thread;
use std::time::{
    Duration,
    Instant
};
use std::io;
use std::collections::{
    VecDeque,
    HashMap
};
#[macro_use] extern crate log;

// third party widgets
use tui_popup::Popup;

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

//extern crate crossbeam_channel;
use crossbeam_channel::{unbounded,
                        Sender,
                        Receiver};


use ratatui::prelude::Alignment;
use ratatui::{
  backend::CrosstermBackend,
  //terminal::Frame,
  Frame,
  //style::{
  //    Color,
  //    Style,
  //},
  widgets::{
      Paragraph,
      Block,
      Borders,
  },
  Terminal,
};

use tof_dataclasses::packets::{
    TofPacket,
    //PacketType
};

use tof_dataclasses::database::{
  connect_to_db,
  get_dsi_j_ch_pid_map,
  get_linkid_rbid_map,
  ReadoutBoard,
  Paddle,
};

//use tof_dataclasses::serialization::{
//  //Serialization,
//  //Packable,
//};
use tof_dataclasses::events::{
  MasterTriggerEvent,
  RBEvent,
  TofEvent,
  TofHit,
  TofEventSummary,
  //RBWaveform,
};

use tof_dataclasses::io::{
  TofPacketWriter,
  FileType
};

use tof_dataclasses::calibrations::RBCalibrations;
use tof_dataclasses::alerts::{
  TofAlert,
  TofAlertManifest,
  load_alerts
};
use telemetry_dataclasses::packets::TelemetryPacket;


use liftof_lib::settings::LiftofSettings;

use liftof_tui::menu::{
  UIMenuItem,
  MenuItem,
  MainMenu,
  TriggerMenu,
  EventMenu,
  MoniMenu,
  ActiveMenu,
  RBMenu2,
  UIMenu,
  PAMoniMenu,
  SettingsMenu,
  THMenu,
  TSMenu,
  RWMenu,
  HBMenu,
  TelemetryMenu,
};

use liftof_tui::colors::{
  ColorSet,
  ColorTheme,
  COLORSETBW, // current default (valkyrie)
  //COLORSETOMILU
};

use liftof_tui::*;// {
//  MainLayout,
//  EventTab,
//  HomeTab,
//  SettingsTab,
//  TofHitTab,
//  RBTab,
//  RBTabView,
//  MTTab,
//  CPUTab,
//  //RBWaveformTab,
//  TofSummaryTab, 
//  TelemetryTab,
//  TelemetryTabView,
//  AlertTab,
//  CommandTab,
//  PaddleTab,
//  HeartBeatTab,
//  HeartBeatView,
//  //packet_sorter,
//  packet_distributor,
//  socket_wrap_tofstream
//};


//extern crate clap;
use clap::{arg,
           command,
           //value_parser,
           //ArgAction,
           //Command,
           Parser
};


#[derive(Parser, Debug)]
#[command(author = "J.A.Stoessl", version, about, long_about = None)]
struct Args {
  /// Adjust the rendering rate for the application in Hz
  /// The higher the rate, the more strenous on the 
  /// system, but the more responsive it gets.
  /// If screen flickering appears, try to change
  /// this parameter. 
  /// Default value is 100 Hz
  #[arg(short, long, default_value_t = 10.0)]
  refresh_rate: f32,
  /// Get the TofData not from the TOF stream, 
  /// but extract it from the telemetry data instead
  /// THIS NEEDS THAT THE CODE HAS BEEN COMPILED WITH 
  /// --features=telemetry
  #[arg(short, long, default_value_t = false)]
  from_telemetry : bool,
  /// Capture the packets and save them in .tof.gaps files
  #[arg(long, default_value_t = false)]
  capture        : bool,
  /// Allow to control the liftof-cc server with commands
  /// WARNING - this is an expert feature. Only use if 
  /// you are know what you are doing
  #[arg(long, default_value_t = false)]
  allow_commands : bool,
  /// generic liftof config file. If not given, we 
  /// assume liftof-config.toml in this directory
  #[arg(short, long)]
  config: Option<String>,
  /// The alert manifest allows to configure alerts
  /// and subscribe to pages
  #[arg(long)]
  alert_manifest: Option<String>,
}

enum Event<I> {
    Input(I),
    Tick,
}




// make a "holder" for all the tabs and menus, 
// so that it can be put in an Arc(Mutex), so
// we can multithread it
pub struct TabbedInterface<'a> {
  pub ui_menu       :  MainMenu<'a>,
  pub rb_menu       :  RBMenu2<'a>,
  pub mt_menu       :  TriggerMenu<'a>,
  pub st_menu       :  SettingsMenu,
  pub th_menu       :  THMenu,
  pub ts_menu       :  TSMenu,
  pub rw_menu       :  RWMenu,
  pub pa_menu       :  PAMoniMenu,
  pub te_menu       :  EventMenu<'a>,
  pub mo_menu       :  MoniMenu<'a>,
  pub hb_menu       :  HBMenu<'a>,
  pub tl_menu       :  TelemetryMenu<'a>,
  pub active_menu   :  ActiveMenu,

  // The tabs
  pub mt_tab        : MTTab<'a>,
  pub cpu_tab       : CPUTab<'a>,
  // waifu tab
  pub wf_tab        : RBTab<'a>,
  pub settings_tab  : SettingsTab<'a>,
  pub home_tab      : HomeTab<'a>,
  pub event_tab     : EventTab,
  pub cmd_tab       : CommandTab<'a>,

  pub th_tab        : TofHitTab<'a>,
  pub ts_tab        : TofSummaryTab,
  
  // telemetry 
  pub te_tab        : TelemetryTab<'a>,

  pub al_tab        : AlertTab<'a>,
  // paddles 
  pub pd_tab        : PaddleTab<'a>,

  pub hb_tab        : HeartBeatTab,

  // latest color set
  pub color_set     : ColorSet,

  pub quit_request  : bool,
} 

impl<'a> TabbedInterface<'a> {
  pub fn new(ui_menu      : MainMenu<'a>,
             rb_menu      : RBMenu2<'a>,
             mt_menu      : TriggerMenu<'a>,
             st_menu      : SettingsMenu,
             th_menu      : THMenu,
             ts_menu      : TSMenu,
             rw_menu      : RWMenu,
             pa_menu      : PAMoniMenu,
             te_menu      : EventMenu<'a>,
             mo_menu      : MoniMenu<'a>,
             hb_menu      : HBMenu<'a>,
             tl_menu      : TelemetryMenu<'a>,
             active_menu  : ActiveMenu,
             mt_tab       : MTTab<'a>,
             cpu_tab      : CPUTab<'a>,
             wf_tab       : RBTab<'a>,
             settings_tab : SettingsTab<'a>,
             home_tab     : HomeTab<'a>,
             event_tab    : EventTab,
             th_tab       : TofHitTab<'a>,
             //rbwf_tab     : RBWaveformTab,
             ts_tab       : TofSummaryTab,
             te_tab       : TelemetryTab<'a>,
             al_tab       : AlertTab<'a>,
             cmd_tab      : CommandTab<'a>,
             hb_tab       : HeartBeatTab,
             pd_tab       : PaddleTab<'a>) -> Self {
    Self {
      ui_menu     ,
      rb_menu     , 
      mt_menu     , 
      st_menu     ,
      th_menu     ,
      ts_menu     ,
      rw_menu     ,
      pa_menu     ,
      te_menu     ,
      mo_menu     ,
      hb_menu     ,
      tl_menu     ,
      active_menu ,
      mt_tab      , 
      cpu_tab     , 
      wf_tab      , 
      settings_tab,
      home_tab    , 
      event_tab   , 
      th_tab      ,
      //rbwf_tab    ,
      ts_tab      ,
      te_tab      ,
      al_tab      ,
      cmd_tab     ,
      pd_tab      , 
      hb_tab      ,
      color_set   : COLORSETBW,
      quit_request: false,
    }
  }

  pub fn get_colorset(&self) -> ColorSet {
    self.color_set.clone()
  }

  pub fn receive_packet(&mut self) {
    match self.mt_tab.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for MTTab! {err}"),
      Ok(_)    => ()
    }
    match self.pd_tab.receive_packet() {
      Err(err) => error!("Can not receive TofEvent for PaddleTab! {err}"),
      Ok(_)    => ()
    }
    match self.wf_tab.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for WfTab! {err}"),
      Ok(_)    => ()
    }
    match self.event_tab.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for EventTab! {err}"),
      Ok(_)    => ()
    }
    match self.cpu_tab.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for CPUTab! {err}"),
      Ok(_)    => ()
    }
    match self.th_tab.receive_packet() {
      Err(err) => error!("Can not receive TofPackets for TofHitTab! {err}"),
      Ok(_)    => ()
    }
    //match self.rbwf_tab.receive_packet() {
    //  Err(err) => error!("Can not receive RBWaveforms! {err}"),
    //  Ok(_)    => ()
    //}
    match self.ts_tab.receive_packet() {
      Err(err) => error!("Can not receive TofEventSummaries for TofEventSummaryTab! {err}"),
      Ok(_)    => ()
    }
    match self.hb_tab.receive_packet() {
      Err(err) => error!("Can not receive Heartbeats for HeartBeatTab! {err}"),
      Ok(_)    => ()
    }
    // the same though, so maybe for now it is fine
    match self.te_tab.receive_packet() {
      Err(err) => error!("Can not receive a new packet from the Telemetry Stream! {err}"),
      Ok(_)    => ()
    }
    // check for alerts
    self.al_tab.check_alert_state();
  }

  fn update_color_theme(&mut self, cs : ColorSet) {
    self.st_menu.theme.update(&cs);
    self.ui_menu.theme.update(&cs);
    self.rb_menu.theme.update(&cs);
    self.mt_menu.theme.update(&cs);
    self.rw_menu.theme.update(&cs);
    self.ts_menu.theme.update(&cs);
    self.th_menu.theme.update(&cs);
    self.te_menu.theme.update(&cs);
    self.hb_menu.theme.update(&cs);
    self.tl_menu.theme.update(&cs);
    self.home_tab    .theme.update(&cs);
    self.event_tab   .theme.update(&cs);
    self.wf_tab      .theme.update(&cs);
    self.mt_tab      .theme.update(&cs);
    self.settings_tab.theme.update(&cs);
    self.cpu_tab     .theme.update(&cs);
    self.th_tab      .theme.update(&cs);
    //self.rbwf_tab    .theme.update(&cs);
    self.ts_tab      .theme.update(&cs);
    self.te_tab      .theme.update(&cs);
    self.cmd_tab     .theme.update(&cs);
    self.pd_tab      .theme.update(&cs);
    self.hb_tab      .theme.update(&cs);
    self.color_set = cs;
  }
  
  pub fn render_home(&mut self, main_lo : &mut MainLayout, frame : &mut Frame) {
    self.ui_menu.render (&main_lo.menu, frame);
    self.home_tab.render(&main_lo.main, frame);
  }
  
  pub fn render_alerts(&mut self, main_lo : &mut MainLayout, frame : &mut Frame) {
    self.ui_menu.render (&main_lo.menu, frame);
    self.al_tab.render(&main_lo.main, frame);
  }

  pub fn render_events(&mut self, main_lo : &mut MainLayout, frame : &mut Frame) {
    self.te_menu.render  (&main_lo.menu, frame);
    self.event_tab.render(&main_lo.main, frame);
  }

  pub fn render_monitoring(&mut self, main_lo : &mut MainLayout, frame : &mut Frame) {
    self.mo_menu.render(&main_lo.menu, frame);
    self.home_tab.render(&main_lo.main, frame);
  }

  pub fn render_cpu(&mut self, master_lo : &mut MainLayout, frame : &mut Frame) {
    self.ui_menu.render(&master_lo.menu, frame);
    self.cpu_tab.render(&master_lo.main, frame);
  }
  
  pub fn render_mt(&mut self, main_lo : &mut MainLayout, frame : &mut Frame) {
    self.ui_menu.render(&main_lo.menu, frame);
    self.mt_tab.render (&main_lo.main, frame);
  }
  
  pub fn render_rbs(&mut self, main_lo : &mut MainLayout, frame : &mut Frame) {
    match self.active_menu {
      ActiveMenu::RBMenu => {
        self.rb_menu.render(&main_lo.menu, frame);
        self.wf_tab.render (&main_lo.main, frame);
      }
      _ => {
        self.ui_menu.render(&main_lo.menu, frame);
        self.wf_tab.render (&main_lo.main, frame);
      }
    }
  }
  
  pub fn render_paddles(&mut self, main_lo : &mut MainLayout, frame : &mut Frame) {
    match self.active_menu {
      ActiveMenu::Paddles => {
        self.pd_tab.menu.render(&main_lo.menu, frame);
        self.pd_tab.render (&main_lo.main, frame);
      }
      _ => {
        self.ui_menu.render(&main_lo.menu, frame);
        self.pd_tab.render (&main_lo.main, frame);
      }
    }
  }

  pub fn render_heartbeats(&mut self, main_lo : &mut MainLayout, frame : &mut Frame) {
    match self.active_menu {
      ActiveMenu::Heartbeats => {
        self.hb_menu.render(&main_lo.menu, frame);
      }
      _ => {
        self.ui_menu.render(&main_lo.menu, frame);
      }
    }
    self.hb_tab.render(&main_lo.main, frame);
  }

  pub fn render_commands(&mut self, main_lo : &mut MainLayout, frame : &mut Frame) {
    self.ui_menu.render(&main_lo.menu, frame);
    self.cmd_tab.render(&main_lo.main, frame);
  }

  pub fn render_settings(&mut self, main_lo : &mut MainLayout, frame : &mut Frame) {
    self.ui_menu.render     (&main_lo.menu, frame);
    self.settings_tab.render(&main_lo.main, frame);
  }
   
  pub fn render_quit(&mut self, main_lo : &mut MainLayout, frame : &mut Frame) {
    self.ui_menu.render(&main_lo.menu, frame);
  }

  pub fn render_tofhittab(&mut self, main_lo : &mut MainLayout, frame : &mut Frame) {
    self.th_menu.render(&main_lo.menu, frame);
    self.th_tab.render(&main_lo.main, frame);
  }

  pub fn render_tofsummarytab(&mut self, main_lo : &mut MainLayout, frame : &mut Frame) {
    self.te_menu.render(&main_lo.menu, frame);
    self.ts_tab.render(&main_lo.main, frame);
  }
  
  //pub fn render_rbwaveformtab(&mut self, master_lo : &mut MainLayout, frame : &mut Frame) {
  //  self.te_menu.render(&master_lo.rect[0], frame);
  //  self.rbwf_tab.render(&master_lo.rect[1], frame);
  //}

  //pub fn render_pamonidatatab(&mut self, master_lo : &mut MainLayout, frame : &mut Frame) {
  //  self.pa_menu.render(&master_lo.rect[0], frame);
  //  self.wf_tab.render(&master_lo.rect[1], frame);
  //}
  
  pub fn render_telemetrytab(&mut self, main_lo : &mut MainLayout, frame : &mut Frame) {
    match self.active_menu {
      ActiveMenu::Telemetry => {
        self.tl_menu.render(&main_lo.menu, frame);
      }
      _ => {
        self.ui_menu.render(&main_lo.menu, frame);
      }
    }
    self.te_tab.render(&main_lo.main, frame);
  }

  pub fn render(&mut self,
                main_lo : &mut MainLayout,
                frame   : &mut Frame) {
    match self.active_menu {
      ActiveMenu::MainMenu => {
        match self.ui_menu.get_active_menu_item() {
          UIMenuItem::Home => {
            self.render_home(main_lo, frame);
          }
          UIMenuItem::Events => {
            self.render_home(main_lo, frame);
          },
          UIMenuItem::ReadoutBoards => {
            self.wf_tab.view = RBTabView::SelectRB;
            self.render_rbs(main_lo, frame);
          }
          UIMenuItem::Trigger => {
            self.render_mt(main_lo, frame);
          }
          UIMenuItem::Monitoring => {
            self.render_cpu(main_lo, frame);
            //self.render_home(main_lo, frame);
          }
          UIMenuItem::Telemetry => {
            self.render_telemetrytab(main_lo, frame);
          }
          UIMenuItem::Commands => {
            self.render_commands(main_lo, frame);
          }
          UIMenuItem::Settings => {
            self.render_settings(main_lo, frame);
          }
          UIMenuItem::Paddles => {
            self.render_paddles(main_lo, frame);
          }
          UIMenuItem::Heartbeats => {
            self.render_heartbeats(main_lo, frame);
          }
          UIMenuItem::Alerts => {
            self.render_alerts(main_lo, frame);
          }
          UIMenuItem::Quit => {
            self.render_quit(main_lo, frame);
          }
          _ => ()
        }
      }
      ActiveMenu::RBMenu => {
        self.render_rbs(main_lo, frame);
      }
      ActiveMenu::Paddles => {
        self.render_paddles(main_lo, frame);
      }
      ActiveMenu::Heartbeats => {
        self.render_heartbeats(main_lo, frame);
      }
      ActiveMenu::Telemetry => {
        self.render_telemetrytab(main_lo, frame);
      }
      ActiveMenu::Events => {
        match self.te_menu.active_menu_item {
          UIMenuItem::TofSummary => {
            self.render_tofsummarytab(main_lo, frame);
          }
          UIMenuItem::TofHits => {
            self.render_tofhittab(main_lo, frame);
          }
          UIMenuItem::Back => {
            self.render_events(main_lo, frame);
          }
          _ => ()
        }
      }
      ActiveMenu::Monitoring => {
        match self.mo_menu.active_menu_item {
          UIMenuItem::Back => {
            self.render_monitoring(main_lo, frame);
          }
          UIMenuItem::PreampBias => {
          }
          UIMenuItem::PreampTemp => {
          }
          UIMenuItem::LTBThresholds => {
          }
          _ => ()
        }
      }
      _ => ()
    }
    if self.quit_request {
      let popup = Popup::new("Quit liftof-tui?")
        .title("Press Y to confirm, any key to abort")
        .style(self.home_tab.theme.style());
      frame.render_widget(&popup, frame.area());
    }
  }

  /// Perform actions depending on the input.
  ///
  /// Returns a flag indicating if we should 
  /// close the app
  ///
  /// # Returns:
  ///
  /// * (bool, bool) - quit and tab_changed. Indicator if the app should quit or if a tab
  ///   has changed
  pub fn digest_input(&mut self, key_code : KeyCode)
  -> (bool, bool) {
    let mut tab_changed = false;
    if self.quit_request {
      match key_code {
        KeyCode::Char('Y') => {
          return (true, tab_changed);
        }
        _ => {
          self.quit_request = false;
        }
      }
    }
    if self.settings_tab.colortheme_popup {
      self.settings_tab.colortheme_popup = false;
    }

    match key_code {
      KeyCode::Char('q') | KeyCode::Char('Q') 
      => {
        self.quit_request = true;
      }
      KeyCode::Char('a') => {
        if self.ui_menu.get_active_menu_item() == UIMenuItem::Settings {
          self.settings_tab.ctl_active = true;
          //self.settings_tab.ctl_state.select(Some(0));
        }
      }
      KeyCode::Enter => {
        //info!("{:?}", self.ui_menu.get_active_menu_item());
        self.settings_tab.ctl_active = false;
        match self.active_menu {
          ActiveMenu::Events  => {
            match self.te_menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.ui_menu.set_active_menu_item(UIMenuItem::Home);
                self.ui_menu.active_menu_item = MenuItem::Home;
                self.active_menu = ActiveMenu::MainMenu;
              }
              _ => ()
            }
          }
          //ActiveMenu::Trigger => {
          //  match self.mt_menu.get_active_menu_item() {
          //    UIMenuItem::Back => {
          //      self.ui_menu.set_active_menu_item(UIMenuItem::Home);
          //      self.ui_menu.active_menu_item = MenuItem::Home;
          //      self.active_menu = ActiveMenu::MainMenu;
          //    }
          //    _ => ()
          //  }
          //}
          ActiveMenu::Paddles => {
            match self.pd_tab.menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.ui_menu.set_active_menu_item(UIMenuItem::Paddles);
                //self.ui_menu.active_menu_item = MenuItem::Home;
                self.active_menu = ActiveMenu::MainMenu;
              }
              _ => ()
            }
          }
          ActiveMenu::Monitoring => {
            match self.mo_menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.ui_menu.set_active_menu_item(UIMenuItem::Home);
                self.ui_menu.active_menu_item = MenuItem::Home;
                self.active_menu = ActiveMenu::MainMenu;
              }
              _ => ()
            }
          }
          ActiveMenu::Heartbeats => {
            match self.hb_menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.ui_menu.set_active_menu_item(UIMenuItem::Home);
                self.ui_menu.active_menu_item = MenuItem::Home;
                self.active_menu = ActiveMenu::MainMenu;
              }
              _ => ()
            }
          }
          ActiveMenu::RBMenu => {
            match self.rb_menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.ui_menu.set_active_menu_item(UIMenuItem::ReadoutBoards);
                self.ui_menu.active_menu_item = MenuItem::ReadoutBoards;
                self.active_menu = ActiveMenu::MainMenu;
              }
              _ => ()
            }
          }
          ActiveMenu::Telemetry => {
            match self.tl_menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.ui_menu.set_active_menu_item(UIMenuItem::Telemetry);
                self.ui_menu.active_menu_item = MenuItem::ReadoutBoards;
                self.active_menu = ActiveMenu::MainMenu;
              }
              _ => ()
            }
          }
          ActiveMenu::MainMenu => {
            match self.ui_menu.get_active_menu_item() {
              UIMenuItem::ReadoutBoards => {
                //self.rb_menu.set_active_menu_item(UIMenuItem::Back);
                info!("Setting active menu to RBMenu!");
                self.active_menu = ActiveMenu::RBMenu;
              }
              UIMenuItem::Trigger => {
                self.active_menu = ActiveMenu::Trigger;
              }
              UIMenuItem::Events => {
                self.active_menu = ActiveMenu::Events;
              }
              UIMenuItem::Monitoring => {
                self.active_menu = ActiveMenu::Monitoring;
              }
              UIMenuItem::Paddles => {
                self.active_menu = ActiveMenu::Paddles;
              }
              UIMenuItem::Heartbeats => {
                self.active_menu = ActiveMenu::Heartbeats;
              }
              UIMenuItem::Telemetry => {
                self.active_menu = ActiveMenu::Telemetry;
              }
              UIMenuItem::Commands => {
                self.cmd_tab.send_command(); 
                //self.active_menu = ActiveMenu::Paddles;
              }
              UIMenuItem::Settings => {
                self.settings_tab.ctl_active       = true;
                self.settings_tab.colortheme_popup = true;
              }
              UIMenuItem::Quit => {
                info!("Feeling a desire of the user to quit this application...");
                self.quit_request = true;
              }
              _ => ()
            }
          }
          _ => ()
        }
      }
      KeyCode::Right => {
        tab_changed = true;
        self.settings_tab.ctl_active = false;
        match self.active_menu {
          ActiveMenu::MainMenu => {
            self.ui_menu.next();
          }
          ActiveMenu::RBMenu => {
            self.rb_menu.next();
            match self.rb_menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.wf_tab.view = RBTabView::SelectRB; 
              }
              UIMenuItem::Waveforms => {
                self.wf_tab.view = RBTabView::Waveform; 
              }
              UIMenuItem::RBMoniData => {
                self.wf_tab.view = RBTabView::RBMoniData;
              }
              UIMenuItem::PAMoniData => {
                self.wf_tab.view = RBTabView::PAMoniData;
              }
              UIMenuItem::PBMoniData => {
                self.wf_tab.view = RBTabView::PBMoniData;
              }
              UIMenuItem::LTBMoniData => {
                self.wf_tab.view = RBTabView::LTBMoniData; 
              }
              UIMenuItem::GlobalRates => {
                self.wf_tab.view = RBTabView::GlobalRates;
              }
              _ => ()
            }
          }
          ActiveMenu::Paddles => {
            self.pd_tab.menu.next();
          }
          ActiveMenu::Heartbeats => {
            self.hb_menu.next();
            match self.hb_menu.get_active_menu_item() {
              UIMenuItem::EventBuilderHB => {
                self.hb_tab.view = HeartBeatView::EventBuilder; 
              }
              UIMenuItem::TriggerHB => {
                self.hb_tab.view = HeartBeatView::MTB; 
              }
              UIMenuItem::DataSenderHB => {
                self.hb_tab.view = HeartBeatView::DataSink; 
              }
              _ => ()
            }
          }
          ActiveMenu::Telemetry => {
            self.tl_menu.next();
            match self.tl_menu.get_active_menu_item() {
              UIMenuItem::Stream => {
                self.te_tab.view = TelemetryTabView::Stream; 
              }
              UIMenuItem::MergedEvents => {
                self.te_tab.view = TelemetryTabView::MergedEvents; 
              }
              _ => ()
            }
          }
          //ActiveMenu::Trigger => {
          //  self.mt_menu.next();
          //}
          ActiveMenu::Events => {
            self.te_menu.next();
          }
          ActiveMenu::Monitoring => {
            self.mo_menu.next();
          }
          _ => ()
        }
      }
      KeyCode::Left => {
        tab_changed = true;
        self.settings_tab.ctl_active = false;
        match self.active_menu {
          ActiveMenu::MainMenu => {
            self.ui_menu.prev();
          }
          ActiveMenu::RBMenu => {
            self.rb_menu.prev();
            match self.rb_menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.wf_tab.view = RBTabView::SelectRB; 
              }
              UIMenuItem::Waveforms => {
                self.wf_tab.view = RBTabView::Waveform; 
              }
              UIMenuItem::RBMoniData => {
                self.wf_tab.view = RBTabView::RBMoniData;
              }
              UIMenuItem::PAMoniData => {
                self.wf_tab.view = RBTabView::PAMoniData;
              }
              UIMenuItem::PBMoniData => {
                self.wf_tab.view = RBTabView::PBMoniData;
              }
              UIMenuItem::LTBMoniData => {
                self.wf_tab.view = RBTabView::LTBMoniData; 
              }
              UIMenuItem::GlobalRates => {
                self.wf_tab.view = RBTabView::GlobalRates;
              }
              _ => ()
            }
          }
          //ActiveMenu::Trigger => {
          //  self.mt_menu.prev();
          //}
          ActiveMenu::Paddles => {
            self.pd_tab.menu.prev();
          }
          ActiveMenu::Events => {
            self.te_menu.prev();
          }
          ActiveMenu::Monitoring => {
            self.mo_menu.prev();
          }
          ActiveMenu::Telemetry => {
            self.tl_menu.prev();
            match self.tl_menu.get_active_menu_item() {
              UIMenuItem::Stream => {
                self.te_tab.view = TelemetryTabView::Stream; 
              }
              UIMenuItem::MergedEvents => {
                self.te_tab.view = TelemetryTabView::MergedEvents; 
              }
              _ => ()
            }
          }
          ActiveMenu::Heartbeats => {
            self.hb_menu.prev();
            match self.hb_menu.get_active_menu_item() {
              UIMenuItem::Back => {
                self.hb_tab.view = HeartBeatView::EventBuilder; 
              }
              UIMenuItem::EventBuilderHB => {
                self.hb_tab.view = HeartBeatView::EventBuilder; 
              }
              UIMenuItem::TriggerHB => {
                self.hb_tab.view = HeartBeatView::MTB; 
              }
              UIMenuItem::DataSenderHB => {
                self.hb_tab.view = HeartBeatView::DataSink; 
              }
              _ => ()
            }
          }
          _ => ()
        }
      }
      KeyCode::Down => {
        if self.ui_menu.get_active_menu_item() == UIMenuItem::Alerts {
          self.al_tab.next_row();
        }
        if self.settings_tab.ctl_active {
          self.settings_tab.next_ct();
          match self.settings_tab.get_colorset() {
            None => info!("Did not get a new colorset!"),
            Some(cs) => {
              self.update_color_theme(cs);
            }
          }
        }
        // Paddle lsit
        if self.active_menu == ActiveMenu::MainMenu && self.ui_menu.get_active_menu_item() == UIMenuItem::Paddles {
          self.pd_tab.next_pd();
          //info!("selected rb {}", self.wf_tab.rb_selector);
        }
        if self.active_menu == ActiveMenu::Paddles && self.pd_tab.menu.get_active_menu_item() == UIMenuItem::Back {
          self.pd_tab.next_pd();
        }
        // RB list
        if self.active_menu == ActiveMenu::MainMenu && self.ui_menu.get_active_menu_item() == UIMenuItem::ReadoutBoards {
          self.wf_tab.next_rb();
          //info!("selected rb {}", self.wf_tab.rb_selector);
        }
        if self.active_menu == ActiveMenu::RBMenu && self.rb_menu.get_active_menu_item() == UIMenuItem::Back {
          self.wf_tab.next_rb();
        }
        if self.active_menu == ActiveMenu::MainMenu && self.ui_menu.get_active_menu_item() == UIMenuItem::Commands {
          self.cmd_tab.next_cmd();
        }
      }
      KeyCode::Up => {
        if self.ui_menu.get_active_menu_item() == UIMenuItem::Alerts {
          self.al_tab.previous_row();
        }
        if self.settings_tab.ctl_active {
          self.settings_tab.previous_ct();
          match self.settings_tab.get_colorset() {
            None => info!("Did not get a new colorset!"),
            Some(cs) => {
              self.update_color_theme(cs);
            }
          }
        }
        // Paddle lsit
        if self.active_menu == ActiveMenu::MainMenu && self.ui_menu.get_active_menu_item() == UIMenuItem::Paddles {
          self.pd_tab.prev_pd();
          //info!("selected rb {}", self.wf_tab.rb_selector);
        }
        if self.active_menu == ActiveMenu::Paddles && self.pd_tab.menu.get_active_menu_item() == UIMenuItem::Back {
          self.pd_tab.prev_pd();
        }
        if self.active_menu == ActiveMenu::MainMenu && self.ui_menu.get_active_menu_item() == UIMenuItem::ReadoutBoards {
          self.wf_tab.previous_rb();
        }
        if self.active_menu == ActiveMenu::RBMenu && self.rb_menu.get_active_menu_item() == UIMenuItem::Back {
          self.wf_tab.previous_rb();
        }
        if self.active_menu == ActiveMenu::MainMenu && self.ui_menu.get_active_menu_item() == UIMenuItem::Commands {
          self.cmd_tab.prev_cmd();
        }
      }
      _ => {
        self.settings_tab.ctl_active = false;
      }
    }
    debug!("No exit command received, continuing");
    (false, tab_changed) // if we arrive here, we don't
                         // want to exit the app
  }
}

fn main () -> Result<(), Box<dyn std::error::Error>>{
  
  let args = Args::parse();                   
  let allow_commands = args.allow_commands;
  
  let config          : LiftofSettings;
  match args.config {
    None => {
      match LiftofSettings::from_toml("liftof-config.toml") {
        Err(err) => {
          error!("CRITICAL! Unable to parse .toml settings file! {}", err);
          panic!("Unable to parse config file!");
        }
        Ok(_cfg) => {
          config = _cfg;
        }
      }
    }
    Some(cfg_file) => {
      //cfg_file_str = cfg_file.clone();
      match LiftofSettings::from_toml(&cfg_file) {
        Err(err) => {
          error!("CRITICAL! Unable to parse .toml settings file! {}", err);
          panic!("Unable to parse config file!");
        }
        Ok(_cfg) => {
          config = _cfg;
        }
      }
    } // end Some
  } // end match
 
  // alerts for everybody! (yay I guess..)
  let global_alerts      = Arc::new(Mutex::new(HashMap::<&'static str, TofAlert<'static>>::new())); 
  match args.alert_manifest {
    Some(mani_file) => {
      match TofAlertManifest::from_toml(&mani_file) {
        Err(err) => {
          panic!("CRITICAL! Unable to parse alert manifest! {}", err);
        }
        Ok(mani) => {
          let glob_al = load_alerts(mani);
          match global_alerts.lock() {
            Ok(mut gal) => {
              *gal = glob_al;
            }
            Err(err) => error!("Unable to lock global alerts! {err}"),
          }
        }
      }
    }
    None => {
      warn!("Not loading an alert manifest! Alert feature not available!");
    }
  } // end match

  let home_stream_wd_cnt : Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
  let home_streamer      = home_stream_wd_cnt.clone();


  // calibrations for everybody!
  let rbcalibrations : Arc<Mutex<HashMap<u8, RBCalibrations>>> = Arc::new(Mutex::new(HashMap::<u8, RBCalibrations>::new()));
  // prepare calibrations
  let mut readoutboards = HashMap::<u8, ReadoutBoard>::new();
  let mut rb_conn = connect_to_db(String::from("gaps_flight2.db")).expect("Will need database access. Make sure gaps_flight2.db is installed!");
  let rbs     = ReadoutBoard::all(&mut rb_conn).expect("Will need database access. Make sure gaps_flight2.db is installed!");
  let mtlink_rb_map = get_linkid_rbid_map(&rbs);
  for mut rb in rbs {
    rb.calib_file_path = String::from("calibrations");
    match rb.load_latest_calibration() {
      Err(err) => error!("Unable to load calibration for {}! {}", rb, err),
      Ok(_) => {
        match rbcalibrations.lock() {
          Err(_err)  => error!("Unable to lock rbcalibrations mutex!"),
          Ok(mut rbcal) => {
            rbcal.insert(rb.rb_id as u8, rb.calibration.clone()); 
          }
        }
        readoutboards.insert(rb.rb_id as u8, rb);
      }
    }
  }
  let paddles = Paddle::all(&mut rb_conn).expect("Database corrupt!");
  let dsijch_paddle_map = get_dsi_j_ch_pid_map(&paddles);
  let mut paddle_map = HashMap::<u8, Paddle>::new();
  for pdl in paddles {
    paddle_map.insert(pdl.paddle_id as u8, pdl.clone());
  }

  // map for counting TofPackets
  let pm = HashMap::<&str, usize>::new();
  let packet_map : Arc<Mutex<HashMap<&str, usize>>> = Arc::new(Mutex::new(pm));
  let packet_map_home = packet_map.clone();

  // this determines the source of all TofPackets
  let (tp_to_distrib, tp_from_sock)     : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();

  // sender receiver combo to subscribe to tofpackets
  let (mt_pack_send, mt_pack_recv)      : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  let (rb_pack_send, rb_pack_recv)      : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  let (ev_pack_send, ev_pack_recv)      : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  let (cp_pack_send, cp_pack_recv)      : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  let (rbwf_pack_send, rbwf_pack_recv)  : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  let (tr_pack_send, tr_pack_recv)      : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  #[cfg(feature = "telemetry")]
  let (te_pack_send,  te_pack_recv)     : (Sender<TelemetryPacket>, Receiver<TelemetryPacket>) = unbounded();

  // sender receiver for inter thread communication with decoded packets
  let (mte_send, mte_recv)         : (Sender<MasterTriggerEvent>, Receiver<MasterTriggerEvent>) = unbounded();
  let (rbe_send, rbe_recv)         : (Sender<RBEvent>, Receiver<RBEvent>)                       = unbounded();
  let (th_send, th_recv)           : (Sender<TofHit>, Receiver<TofHit>)                         = unbounded();
  let (ts_send, ts_recv)           : (Sender<TofEventSummary>, Receiver<TofEventSummary>)       = unbounded();
  // channel to send TofEventSummary from tofevent to paddle tab
  let (ts_send_pdl, ts_recv_pdl)   : (Sender<TofEventSummary>, Receiver<TofEventSummary>)       = unbounded();
  let (te_send, te_recv)           : (Sender<TofEvent>, Receiver<TofEvent>)                     = unbounded();

  // send tof packets containing heartbeats
  let (hb_pack_send, hb_pack_recv)      : (Sender<TofPacket>, Receiver<TofPacket>) = unbounded();
  

  // depending on the switch in the 
  // commandline args, we either 
  // connect to the telemetry stream 
  // or directly to the tof data strea
  // ROUTING: In this case we reroute the 
  // packets through the TelemetryTab, 
  // which unpacks them and then it will 
  // funnel in the packet-distributor 
  // thread
  // This also means, that in case where we 
  // don't want the packets from telemetry,
  // we have to spawn an additional thread which 
  // receives the packets from the TOF stream
  // instead
  //if args.from_telemetry {
  if !args.from_telemetry {
    let _tp_to_distrib = tp_to_distrib.clone();
    let tofcpu_address : &str = "tcp://192.168.37.20:42000";
    let _packet_recv_thread = thread::Builder::new()
      .name("socket-wrap-tofstream".into())
      .spawn(move || {
        socket_wrap_tofstream(tofcpu_address,
                              _tp_to_distrib);
      }).expect("Unable to spawn socket-wrap-tofstream thread!");
  }
  //  let _packet_recv_thread = thread::Builder::new()
  //    .name("packet-receiver".into())
  //    .spawn(move || {
  //      let telemetry_address : &str = "tcp://127.0.0.1:55555";
  //      socket_wrap_telemetry(telemetry_address, 
  //                            tp_to_distrib );
  //    })
  //  .expect("Failed to spawn mt packet receiver thread!");


  // FIXME - spawn a new thread per each tab!
  let th_sender_c = th_send.clone();
  let mut writer : Option<TofPacketWriter> = None;
  if args.capture{
    info!("Capturing packets and write them to disk!");
    let file_type = FileType::RunFile(0);
    // FIXME - create "captured" directory
    writer = Some(TofPacketWriter::new(String::from("."), file_type));
    // FIXME - use value from config file
    writer.as_mut().unwrap().mbytes_per_file = 420;
  }
  let _packet_dist_thread = thread::Builder::new()
    .name("packet-distributor".into())
    .spawn(move || {
      packet_distributor(tp_from_sock,
                         mt_pack_send, 
                         rb_pack_send,
                         ev_pack_send,
                         cp_pack_send,
                         tr_pack_send,
                         rbwf_pack_send,
                         ts_send,
                         th_sender_c,
                         hb_pack_send,
                         home_stream_wd_cnt,
                         packet_map,
                         writer);
    }).expect("Failed to spawn mt packet receiver thread!");
  
  // spawn the telemetry receiver thread
  cfg_if::cfg_if! {
    if #[cfg(feature = "telemetry")] {
      let telemetry_address : &str = "tcp://127.0.0.1:55555";
      let _telemetry_receiver_thread = thread::Builder::new()
        .name("socket-wrap-telemetry".into())
        .spawn(move || {
          socket_wrap_telemetry(
            telemetry_address,
            32000, // FIXME - can be up to u16::MAX
            te_pack_send
          );
        })
        .expect("Failed to spawn mt packet receiver thread!");
    }
  }
  // Set max_log_level to Trace
  match tui_logger::init_logger(log::LevelFilter::Info) {
    Err(err) => panic!("Something bad just happened {err}"),
    Ok(_)    => (),
  }
  // Set default level for unknown targets to Trace ("Trace"/"Info") 
  tui_logger::set_default_level(log::LevelFilter::Info);
  
  
  // set up the terminal
  enable_raw_mode().expect("Unable to enter raw mode");
  let stdout       = io::stdout();
  let backend      = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;
  terminal.clear()?;
  
  //let (tx, rx) = mpsc::channel();
  let (tx, rx) = unbounded();

  // heartbeat, keeps it going
  let _heartbeat_thread = thread::Builder::new()
    .name("heartbeat".into())
    .spawn(move || {
                     // change this to make it more/less 
                     // responsive
                     let refresh_perioad = 1000.0/args.refresh_rate as f32;
                     let tick_rate = Duration::from_millis(refresh_perioad.round() as u64);
                     let mut last_tick = Instant::now();
                     loop {
                       let timeout = tick_rate
                         .checked_sub(last_tick.elapsed())
                         .unwrap_or_else(|| Duration::from_secs(0));
                       if event::poll(timeout).expect("poll works") {
                         if let CEvent::Key(key) = event::read().expect("can read events") {
                           tx.send(Event::Input(key)).expect("can send events");
                         }
                       }
                       if last_tick.elapsed() >= tick_rate {
                         if let Ok(_) = tx.send(Event::Tick) {
                           //info!("ticker : {}", last_tick.elapsed().as_micros());
                           last_tick = Instant::now();
                         }
                       }
                     }
                   }
    ).expect("Failed to spawn heartbeat thread!");


  // A color theme, can be changed later
  let mut color_theme     = ColorTheme::new();
  color_theme.update(&COLORSETBW);
  //let mut color_set_bw    = 
  
  // The menus
  let ui_menu         = MainMenu::new(color_theme.clone());
  let rb_menu         = RBMenu2::new(color_theme.clone());
  let mt_menu         = TriggerMenu::new(color_theme.clone());
  let st_menu         = SettingsMenu::new(color_theme.clone());
  let th_menu         = THMenu::new(color_theme.clone());
  let ts_menu         = TSMenu::new(color_theme.clone());
  let rw_menu         = RWMenu::new(color_theme.clone());
  let pa_menu         = PAMoniMenu::new(color_theme.clone());
  let te_menu         = EventMenu::new(color_theme.clone());
  let mo_menu         = MoniMenu::new(color_theme.clone());
  let hb_menu         = HBMenu::new(color_theme.clone());
  let tl_menu         = TelemetryMenu::new(color_theme.clone());
  // The tabs
  let ts_tab          = TofSummaryTab::new(ts_recv,
                                           ts_send_pdl,
                                           &dsijch_paddle_map,
                                           color_theme.clone());
  let mt_tab          = MTTab::new(mt_pack_recv,
                                   mte_recv,
                                   dsijch_paddle_map,
                                   mtlink_rb_map,
                                   global_alerts.clone(),
                                   color_theme.clone());
 
  let cpu_tab         = CPUTab::new(cp_pack_recv,
                                    global_alerts.clone(),
                                    color_theme.clone());
  // waifu tab
  let wf_tab          = RBTab::new(rb_pack_recv,
                                   rbe_recv,
                                   readoutboards.clone(),
                                   global_alerts.clone(),
                                   color_theme.clone());
  let settings_tab    = SettingsTab::new(color_theme.clone());
  let home_tab        = HomeTab::new(color_theme.clone(), home_streamer, packet_map_home);
  let event_tab       = EventTab::new(ev_pack_recv, mte_send, rbe_send, th_send, te_send, color_theme);
  let hit_tab         = TofHitTab::new(th_recv,color_theme.clone());
  //let rbwf_tab        = RBWaveformTab::new(rbwf_pack_recv,
  //                                         readoutboards,
  //                                         color_theme.clone());
  let te_tab          : TelemetryTab;
  if args.from_telemetry {
    te_tab            = TelemetryTab::new(Some(tp_to_distrib),
                                          te_pack_recv,
                                          color_theme.clone());
  } else {
    te_tab      = TelemetryTab::new(None,
                                    te_pack_recv, 
                                    color_theme.clone());
  
  
  }
  let cmd_sender_addr = String::from("tcp://192.168.37.5:42000");
  let cmd_tab         = CommandTab::new(tr_pack_recv, cmd_sender_addr, color_theme.clone(), allow_commands);
  let pd_tab          = PaddleTab::new(ts_recv_pdl, rbwf_pack_recv, paddle_map, rbcalibrations, color_theme.clone());
  let hb_tab          = HeartBeatTab::new(hb_pack_recv, color_theme.clone());
  let al_tab          = AlertTab::new(color_theme.clone(),global_alerts.clone()); 
  let active_menu     = ActiveMenu::MainMenu;
  let tabs            = TabbedInterface::new(ui_menu,
                                             rb_menu,
                                             mt_menu,
                                             st_menu,
                                             th_menu,
                                             ts_menu,
                                             rw_menu,
                                             pa_menu,
                                             te_menu,
                                             mo_menu,
                                             hb_menu,
                                             tl_menu,
                                             active_menu,
                                             mt_tab,
                                             cpu_tab,
                                             wf_tab,
                                             settings_tab,
                                             home_tab,
                                             event_tab,
                                             hit_tab,
                                             //rbwf_tab,
                                             ts_tab,
                                             te_tab,
                                             al_tab,
                                             cmd_tab,
                                             hb_tab,
                                             pd_tab);

  let shared_tabs : Arc<Mutex<TabbedInterface>> = Arc::new(Mutex::new(tabs));
  let shared_tabs_c = shared_tabs.clone();
  let _update_thread = thread::Builder::new()
    .name("tab-packet-receiver".into())
    .spawn(move || {
      loop {
        match shared_tabs_c.lock() {
          Err(err) => error!("Can't get lock on shared tabs! {err}"),
          Ok(mut tabs) => {
            tabs.receive_packet();
          }
        }
      }
    }).expect("Failed to spawn tab-packet-receiver thread!");

  let mut quit_app = false;
  loop {
    match rx.recv() {
      Err(err) => trace!("Err - no update! {err}"),
      Ok(event) => {
        //info!("next event!");
        match event {
          Event::Input(ev) => {
            //info!("input");
            match shared_tabs.lock() {
              Err(err) => error!("Unable to get lock on shared tabbed interface! {err}"),
              Ok(mut tabs) => {
                //tabs.receive_packet();
                let (want_quit, tab_changed) = tabs.digest_input(ev.code);
                quit_app = want_quit;
                if tab_changed {
                  let _ = terminal.clear();
                }
                let cs = tabs.get_colorset();
                color_theme.update(&cs);
              }
            }
          }, 
          Event::Tick => {
            //info!("tick");
            match shared_tabs.lock() {
              Err(err) => error!("Unable to get lock on shared tabbed interface! {err}"),
              Ok(mut tabs) => {
                match terminal.draw(|frame| {
                  let size           = frame.area();
                  let mut main_lo    = MainLayout::new(size); 
                  let w_logs         = render_logs(color_theme.clone());
                  let help_text = "Navigate with \u{2190} \u{2191} \u{2192} \u{2193}\n 'Enter' to confirm \n 'q' to quit";
                  let help_view = Paragraph::new(help_text)
                    .style(color_theme.style())
                    .alignment(Alignment::Center)
                    .block(
                     Block::default()
                       .borders(Borders::ALL)
                  );
                  frame.render_widget(w_logs, main_lo.log);
                  tabs.render(&mut main_lo, frame);
                  frame.render_widget(help_view, main_lo.help);
                }) {
                  Err(err) => error!("Can't render terminal! {err}"),
                  Ok(_)    => () ,
                } // end terminal.draw
              }
            }
          } // end Event::Tick
        }
      }
    }
    if quit_app {
      disable_raw_mode()?;
      terminal.clear()?;
      terminal.show_cursor()?;
      break;
    }
  } // end loop;
  Ok(())
}
