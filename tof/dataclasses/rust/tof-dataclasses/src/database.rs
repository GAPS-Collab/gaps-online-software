//! Database access & entities of the TOF
//!
//! A local .sqlite database is shipped with 
//! this packet and allows to access all
//! mapping relevant TOF information, e.g. 
//! paddle connection to LTBs anr RBs,
//! paddle information, paddle cordinates,
//! panel ids and so on.
//!

use std::fmt;
use std::collections::HashMap;

use glob::glob;
use regex::Regex;
use chrono::{
    DateTime,
    Utc,
};

//use rusqlite::Connection;
use diesel::prelude::*;
mod schema;
    
use schema::tof_db_rat::dsl::*;
use schema::tof_db_dsicard::dsl::*;

use crate::calibrations::RBCalibrations;
//use crate::constants::HUMAN_TIMESTAMP_FORMAT;
use crate::DsiLtbRBMapping;

// FIXME - probably we should make this nicer
pub type DsiJChPidMapping = DsiLtbRBMapping; 

/// Universal function to connect to the database
pub fn connect_to_db(database_url : String) -> Result<diesel::SqliteConnection, ConnectionError>  {
    //let database_url = "database.sqlite3";
    SqliteConnection::establish(&database_url)
}

/// Create a mapping of mtb link ids to rb ids
pub fn get_linkid_rbid_map(rbs : &Vec<ReadoutBoard>) -> HashMap<u8, u8>{
  let mut mapping = HashMap::<u8, u8>::new();
  for rb in rbs {
    mapping.insert(rb.mtb_link_id, rb.rb_id);
  }
  mapping
}

/// Create a mapping of rb id to mtb link ids
pub fn get_rbid_linkid_map(rbs : &Vec<ReadoutBoard>) -> HashMap<u8, u8> {
  let mut mapping = HashMap::<u8, u8>::new();
  for rb in rbs {
    mapping.insert(rb.rb_id, rb.mtb_link_id);
  }
  mapping
}

pub fn get_dsi_j_ch_pid_map(paddles : &Vec<Paddle>) -> DsiJChPidMapping {
  let mut mapping = DsiJChPidMapping::new();
  for dsi in 1..6 {
    let mut jmap = HashMap::<u8, HashMap<u8, (u8, u8)>>::new();
    for j in 1..6 {
      let mut rbidch_map : HashMap<u8, (u8,u8)> = HashMap::new();
      for ch in 1..17 {
        let rbidch = (0,0);
        rbidch_map.insert(ch,rbidch);
        //map[dsi] = 
      }
      jmap.insert(j,rbidch_map);
    }
    mapping.insert(dsi,jmap);
  }
  for pdl in paddles {
    let dsi  = pdl.dsi as u8;
    let   j  = pdl.j_ltb   as u8;
    let ch_b = pdl.ltb_chA as u8;
    let ch_a = pdl.ltb_chB as u8;
    let pid  = pdl.paddle_id as u8;
    let panel_id = pdl.panel_id as u8;
    mapping.get_mut(&dsi).unwrap().get_mut(&j).unwrap().insert(ch_a,(pid, panel_id));
    mapping.get_mut(&dsi).unwrap().get_mut(&j).unwrap().insert(ch_b,(pid, panel_id));
  }
  return mapping;
}

/// A representation of a run 
#[derive(Debug, Clone, Queryable,Insertable, Selectable, serde::Serialize, serde::Deserialize)]
#[diesel(table_name = schema::tof_db_run)]
#[diesel(primary_key(run_id))]
pub struct Run {
  pub run_id                    : i64,
  pub runtime_secs              : Option<i64>,
  pub calib_before              : Option<bool>,
  pub shifter                   : Option<i16>,
  pub run_type                  : Option<i16>,
  pub run_path                  : Option<String>,
}

impl Run {
  pub fn new() -> Self {
    Self {
      run_id        : 0, 
      runtime_secs  : Some(0), 
      calib_before  : Some(true), 
      shifter       : Some(0), 
      run_type      : Some(0), 
      run_path      : Some(String::from("")), 
    }
  }

  pub fn get_last_run(conn: &mut SqliteConnection) -> Option<u32> {
    use schema::tof_db_run::dsl::*;
    match tof_db_run.load::<Run>(conn) {
      Err(err) => {
        error!("Unable to load DSICards from db! {err}");
        return None;
      }
      Ok(_runs) => {
        //return Some(runs);
      }
    }
    let _results = tof_db_run
      //.filter(published.eq(true))
      .limit(1)
      //.select(Run::as_select())
      .load::<Run>(conn)
      .expect("Error loading posts");
    None
  }
}

impl fmt::Display for Run {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<Run");
    repr += &(format!("\n  RunID         : {}", self.run_id));                   
    repr += &(format!("\n  - auto cali   : {}", self.calib_before.unwrap_or(false)));
    repr += &(format!("\n  runtime [sec] : {}", self.runtime_secs.unwrap_or(-1)));
    repr += &(format!("\n  shifter       : {}", self.shifter.unwrap_or(-1)));
    repr += &(format!("\n  run_type      : {}", self.run_type.unwrap_or(-1)));
    repr += &(format!("\n  run_path      : {}", self.run_path.clone().unwrap_or(String::from(""))));
    write!(f, "{}", repr)
  }
}

/// Representation of a local trigger board.
/// 
/// The individual LTB channels do not map directly to PaddleEnds. Rather two of them
/// map to a paddle and then the whole paddle should get read out.
/// To be more specific about this. The LTB has 16 channels, but we treat them as 8.
/// Each 2 LTB channels get "married" internally in the board and will then continue
/// on as 1 LTB channel, visible to the outside. The information about which end of 
/// the Paddle crossed which threshhold is lost.
/// How it works is that the two channels will be combined by the trigger logic:
/// - There are 4 states (2 bits)
///   - 0 - no hit
///   - 1 - Hit
///   - 2 - Beta
///   - 3 - Veto
/// 
/// Each defining an individual threshold. If that is crossed, the whole paddle
/// (ends A+B) will be read out by the ReadoutBoard
/// 
/// The LTB channels here are labeled 1-8. This is as it is in the TOF spreadsheet.
/// Also dsi is labeled as in the spreadsheet and will start from one.
/// 
/// It is NOT clear from this which ch on the rb is connected to which side, for that
/// the paddle/RB tables need to be consulted.
/// Again: rb_ch0 does NOT necessarily correspond to the A side!
/// 
#[derive(Debug,Queryable, Selectable, serde::Serialize, serde::Deserialize)]
#[diesel(table_name = schema::tof_db_rat)]
#[diesel(primary_key(rat_id))]
pub struct RAT {
  pub rat_id                    : i16, 
  pub pb_id                     : i16, 
  pub rb1_id                    : i16, 
  pub rb2_id                    : i16, 
  pub ltb_id                    : i16, 
  pub ltb_harting_cable_length  : i16, 
}

impl RAT {
  pub fn new() -> Self {
    Self {
      rat_id                    : 0, 
      pb_id                     : 0, 
      rb1_id                    : 0, 
      rb2_id                    : 0, 
      ltb_id                    : 0, 
      ltb_harting_cable_length  : 0, 
    }
  }
  
  /// Get the RAT where rb2id matched the argument
  pub fn where_rb2id(conn: &mut SqliteConnection, rb2id : u8) -> Option<Vec<RAT>> {
    let mut result = Vec::<RAT>::new();
    match RAT::all(conn) {
      Some(rats) => {
        for rat in rats {
          if rat.rb2_id == rb2id as i16 {
            result.push(rat);
          }
        }
        return Some(result);
      }
      None => ()
    }
    Some(result)
  }

  pub fn all(conn: &mut SqliteConnection) -> Option<Vec<RAT>> {
    match tof_db_rat.load::<RAT>(conn) {
      Err(err) => {
        error!("Unable to load RATs from db! {err}");
        return None;
      }
      Ok(rats) => {
        return Some(rats);
      }
    }
  }

}

impl fmt::Display for RAT {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<RAT");
    repr += &(format!("\n  ID                : {}", self.rat_id));                   
    repr += &(format!("\n  PB                : {} ", self.pb_id));                    
    repr += &(format!("\n  RB1               : {}", self.rb1_id));                   
    repr += &(format!("\n  RB2               : {}", self.rb2_id));                   
    repr += &(format!("\n  LTB               : {}", self.ltb_id));                   
    repr += &(format!("\n  H. cable len [cm] : {}>", self.ltb_harting_cable_length)); 
    write!(f, "{}", repr)
  }
}


/// A DSI card which is plugged into one of five slots on the MTB
/// The DSI card provides the connection to RBs and LTBs and has 
/// a subdivision, which is called 'j'
#[derive(Queryable, Selectable)]
#[diesel(primary_key(dsi_id))]
#[diesel(table_name = schema::tof_db_dsicard)]
pub struct DSICard { 
  pub dsi_id    : i16,
  pub j1_rat_id : Option<i16>,
  pub j2_rat_id : Option<i16>,
  pub j3_rat_id : Option<i16>,
  pub j4_rat_id : Option<i16>,
  pub j5_rat_id : Option<i16>,
}
 

impl DSICard {
  pub fn new() -> Self {
    Self {
      dsi_id    : 0,
      j1_rat_id : None,
      j2_rat_id : None,
      j3_rat_id : None,
      j4_rat_id : None,
      j5_rat_id : None,
    }
  }
  
  /// True if this RAT box is plugged in to any of the j 
  /// connectors on this specific DSI card
  pub fn has_rat(&self, r_id : u8) -> bool {
    if let Some(rid) = self.j1_rat_id {
      if rid as u8 == r_id {
        return true;
      }
    }
    if let Some(rid) = self.j2_rat_id {
      if rid as u8 == r_id {
        return true;
      }
    }
    if let Some(rid) = self.j3_rat_id {
      if rid as u8 == r_id {
        return true;
      }
    }
    if let Some(rid) = self.j4_rat_id {
      if rid as u8 == r_id {
        return true;
      }
    }
    if let Some(rid) = self.j5_rat_id {
      if rid as u8 == r_id {
        return true;
      }
    }
    return false;
  }

  /// Get the j connetor for this specific RAT
  /// Raises ValueError if the RAT is not connected
  pub fn get_j(&self, r_id : u8) -> Option<u8> {
    if !self.has_rat(r_id) {
      return None;
    }
    if let Some(rid) = self.j1_rat_id {
      if rid as u8 == r_id {
        let _j = self.j1_rat_id.unwrap() as u8;
        return Some(_j);
      }
    }
    if let Some(rid) = self.j2_rat_id {
      if rid as u8 == r_id {
        let _j = self.j2_rat_id.unwrap() as u8;
        return Some(_j);
      }
    }
    if let Some(rid) = self.j3_rat_id {
      if rid as u8 == r_id {
        let _j = self.j3_rat_id.unwrap() as u8;
        return Some(_j);
      }
    }
    if let Some(rid) = self.j4_rat_id {
      if rid as u8 == r_id {
        let _j = self.j4_rat_id.unwrap() as u8;
        return Some(_j);
      }
    }
    if let Some(rid) = self.j5_rat_id {
      if rid as u8 == r_id {
        let _j = self.j5_rat_id.unwrap() as u8;
        return Some(_j);
      }
    }
  None
  }
  
  pub fn all(conn: &mut SqliteConnection) -> Option<Vec<DSICard>> {
    match tof_db_dsicard.load::<DSICard>(conn) {
      Err(err) => {
        error!("Unable to load DSICards from db! {err}");
        return None;
      }
      Ok(dsis) => {
        return Some(dsis);
      }
    }
  }
}

impl fmt::Display for DSICard {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr  = String::from("<DSI Card:");
    repr += &(format!("\n  ID     : {}", self.dsi_id));     
    repr += "\n  -- -- -- --";
    if let Some(_j) = self.j1_rat_id {
        repr += &(format!("\n  J1 RAT : {}",_j));
    } else {
        repr += "\n  J1 RAT : Not connected";
    }
    if let Some(_j) = self.j2_rat_id {
        repr += &(format!("\n  J2 RAT : {}",_j));
    } else {
        repr += "\n  J2 RAT : Not connected";
    }
    if let Some(_j) = self.j3_rat_id {
        repr += &(format!("\n  J3 RAT : {}",_j));
    } else {
        repr += "\n  J3 RAT : Not connected";
    }
    if let Some(_j) = self.j4_rat_id {
        repr += &(format!("\n  J4 RAT : {}",_j));
    } else {
        repr += "\n  J4 RAT : Not connected";
    }
    if let Some(_j) = self.j5_rat_id {
        repr += &(format!("\n  J5 RAT : {}>",_j));
    } else {
        repr += "\n  J5 RAT : Not connected>";
    }
    write!(f, "{}", repr)
  }
}

/// A single TOF paddle with 2 ends 
/// comnected
#[derive(Debug,PartialEq, Clone,Queryable, Selectable, serde::Serialize, serde::Deserialize)]
#[diesel(table_name = schema::tof_db_paddle)]
#[diesel(primary_key(paddle_id))]
#[allow(non_snake_case)]
pub struct Paddle {
  pub paddle_id         : i16, 
  pub volume_id         : i64, 
  pub panel_id          : i16, 
  pub mtb_link_id       : i16, 
  pub rb_id             : i16, 
  pub rb_chA            : i16, 
  pub rb_chB            : i16, 
  pub ltb_id            : i16, 
  pub ltb_chA           : i16, 
  pub ltb_chB           : i16, 
  pub pb_id             : i16, 
  pub pb_chA            : i16, 
  pub pb_chB            : i16, 
  pub cable_len         : f32, 
  pub dsi               : i16, 
  pub j_rb              : i16, 
  pub j_ltb             : i16, 
  pub height            : f32, 
  pub width             : f32, 
  pub length            : f32, 
  pub global_pos_x_l0   : f32, 
  pub global_pos_y_l0   : f32, 
  pub global_pos_z_l0   : f32, 
  pub global_pos_x_l0_A : f32, 
  pub global_pos_y_l0_A : f32, 
  pub global_pos_z_l0_A : f32, 
}

impl Paddle {
  pub fn new() -> Self {
    Self {
      paddle_id         : 0, 
      volume_id         : 0, 
      panel_id          : 0, 
      mtb_link_id       : 0, 
      rb_id             : 0, 
      rb_chA            : 0, 
      rb_chB            : 0, 
      ltb_id            : 0, 
      ltb_chA           : 0, 
      ltb_chB           : 0, 
      pb_id             : 0, 
      pb_chA            : 0, 
      pb_chB            : 0, 
      cable_len         : 0.0, 
      dsi               : 0, 
      j_rb              : 0, 
      j_ltb             : 0, 
      height            : 0.0, 
      width             : 0.0, 
      length            : 0.0, 
      global_pos_x_l0   : 0.0, 
      global_pos_y_l0   : 0.0, 
      global_pos_z_l0   : 0.0, 
      global_pos_x_l0_A : 0.0, 
      global_pos_y_l0_A : 0.0, 
      global_pos_z_l0_A : 0.0, 
    }
  }

  pub fn all(conn: &mut SqliteConnection) -> Option<Vec<Paddle>> {
    use schema::tof_db_paddle::dsl::*;
    match tof_db_paddle.load::<Paddle>(conn) {
      Err(err) => {
        error!("Unable to load paddles from db! {err}");
        return None;
      }
      Ok(pdls) => {
        return Some(pdls);
      }
    }
  }
}

impl fmt::Display for Paddle {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<Paddle:");
    repr += "\n** identifiers **";
    repr += &(format!("\n   pid                : {}", self.paddle_id));     
    repr += &(format!("\n   vid                : {}", self.volume_id));
    repr += &(format!("\n   panel id           : {}", self.panel_id));
    repr += "\n  ** connedtions **";
    repr += &(format!("\n   DSI/J/CH (LG) [A]  : {}  | {} | {:02}", self.dsi, self.j_ltb, self.ltb_chA));
    repr += &(format!("\n   DSI/J/CH (HG) [A]  : {}  | {} | {:02}", self.dsi, self.j_rb, self.rb_chA));
    repr += &(format!("\n   DSI/J/CH (LG) [B]  : {}  | {} | {:02}", self.dsi, self.j_ltb, self.ltb_chB));
    repr += &(format!("\n   DSI/J/CH (HG) [B]  : {}  | {} | {:02}", self.dsi, self.j_rb, self.rb_chB));
    repr += &(format!("\n   RB/CH         [A]  : {:02} | {}", self.rb_id, self.rb_chA));
    repr += &(format!("\n   RB/CH         [B]  : {:02} | {}", self.rb_id, self.rb_chB));
    repr += &(format!("\n   LTB/CH        [A]  : {:02} | {}", self.ltb_id, self.ltb_chA));
    repr += &(format!("\n   LTB/CH        [B]  : {:02} | {}", self.ltb_id, self.ltb_chB));
    repr += &(format!("\n   PB/CH         [A]  : {:02} | {}", self.pb_id, self.pb_chA));
    repr += &(format!("\n   PB/CH         [B]  : {:02} | {}", self.pb_id, self.pb_chB));
    repr += &(format!("\n   MTB Link ID        : {:02}", self.mtb_link_id));
    repr += "\n   cable len [cm] :";
    repr += &(format!("\n    \u{21B3} {:.2}", self.cable_len));
    repr += "\n    (Harting -> RB)";
    repr += "\n  ** Coordinates (L0) & dimensions **";
    repr += "\n   length, width, height [mm]";
    repr += &(format!("\n    \u{21B3} [{:.2}, {:.2}, {:.2}]", self.length, self.width, self.height));
    repr += "\n   center [mm]:";
    repr += &(format!("\n    \u{21B3} [{:.2}, {:.2}, {:.2}]", self.global_pos_x_l0, self.global_pos_y_l0, self.global_pos_z_l0));
    repr += "\n   A-side [mm]:";
    repr += &(format!("\n    \u{21B3} [{:.2}, {:.2}, {:.2}]>", self.global_pos_x_l0_A, self.global_pos_y_l0_A, self.global_pos_z_l0_A));
    write!(f, "{}", repr)
  }
}
    
// Summary of DSI/J/LTBCH (0-319)
// This is not "official" but provides a way of indexing all
// the individual channels
#[derive(Debug,PartialEq,Queryable, Selectable)]
#[diesel(table_name = schema::tof_db_mtbchannel)]
#[diesel(primary_key(mtb_ch))]
#[allow(non_snake_case)]
pub struct MTBChannel {
  pub mtb_ch      : i64,         
  pub dsi         : Option<i16>, 
  pub j           : Option<i16>, 
  pub ltb_id      : Option<i16>, 
  pub ltb_ch      : Option<i16>, 
  pub rb_id       : Option<i16>, 
  pub rb_ch       : Option<i16>, 
  pub mtb_link_id : Option<i16>, 
  pub paddle_id   : Option<i16>, 
  pub paddle_isA  : Option<bool>,
  pub hg_ch       : Option<i16>, 
  pub lg_ch       : Option<i16>, 
}

impl MTBChannel {

  pub fn new() -> Self {
    Self {
      mtb_ch      : -1,         
      dsi         : None, 
      j           : None, 
      ltb_id      : None, 
      ltb_ch      : None, 
      rb_id       : None, 
      rb_ch       : None, 
      mtb_link_id : None, 
      paddle_id   : None, 
      paddle_isA  : None,
      hg_ch       : None, 
      lg_ch       : None, 
    }
  }
  
  pub fn all(conn: &mut SqliteConnection) -> Option<Vec<MTBChannel>> {
    use schema::tof_db_mtbchannel::dsl::*;
    match tof_db_mtbchannel.load::<MTBChannel>(conn) {
      Err(err) => {
        error!("Unable to load RATs from db! {err}");
        return None;
      }
      Ok(mtbch) => {
        return Some(mtbch);
      }
    }
  }
}


impl fmt::Display for MTBChannel {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<MTBChannel");
    repr += &(format!("\n  Channel ID : {}", self.mtb_ch));
    repr += &(format!("\n  DSI/J/     : {}/{}", self.dsi.unwrap_or(-1), self.j.unwrap_or(-1)));
    repr += "\n  LTB ID/CH => RB ID/CH";
    repr += &(format!("\n   |-> {}/{} => {}/{}", self.ltb_id.unwrap_or(-1), self.ltb_ch.unwrap_or(-1), self.rb_id.unwrap_or(-1), self.rb_ch.unwrap_or(-1)));
    repr += &(format!("\n  MTB Link ID [RB] : {}", self.mtb_link_id.unwrap_or(-1)));
    repr += "\n  LG CH => HG CH";
    repr += &(format!("\n   |-> {} => {}", self.lg_ch.unwrap_or(-1), self.hg_ch.unwrap_or(-1)));
    repr += &(format!("\n  Paddle Id: {}", self.paddle_id.unwrap_or(-1)));
    let mut pend = "None";
    if !self.paddle_isA.is_none() {
      if self.paddle_isA.unwrap() {
          pend = "A";
      } else {
          pend = "B";
      }
    }
    repr += &(format!("\n  Paddle End: {}>", pend));
    write!(f, "{}", repr)
  }
}


///////////////////////////////////////////////////
//
// The following models exceed a bit the capabilities
// of Diesel, or my Diesel skill.
// These models contain multiple ForeignKeys, in all
// cases these link to the paddle table. 
//
// For each of LocalTriggerBoard, ReadoutBoard, Panel
// we have 2 structs:
// One called DB<entity> and the other <entity>. The
// first does have the ForeignKeys as SmallInt, and 
// the latter looks them up and fills in the blanks
//
//
//

/// The DB wrapper for the LocalTriggerBoard, for 
/// easy implementation there are no joins, we do 
/// them manually in the public implementation 
/// of the LocaltriggerBoard
#[derive(Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = schema::tof_db_localtriggerboard)]
#[diesel(primary_key(board_id))]
#[diesel(belongs_to(Paddle, foreign_key=paddle1_id))]
pub struct DBLocalTriggerBoard {
    pub board_id      : i16,    
    pub dsi           : Option<i16>,
    pub j             : Option<i16>,
    pub rat           : Option<i16>,
    pub ltb_id        : Option<i16>, 
    pub cable_len     : f32,
    pub paddle1_id    : Option<i16>,
    pub paddle2_id    : Option<i16>,
    pub paddle3_id    : Option<i16>,
    pub paddle4_id    : Option<i16>,
    pub paddle5_id    : Option<i16>,
    pub paddle6_id    : Option<i16>,
    pub paddle7_id    : Option<i16>,
    pub paddle8_id    : Option<i16>,
}

impl DBLocalTriggerBoard {
  
  //pub fn new() -> Self {
  //  Self {
  //    board_id      : 0,    
  //    dsi           : None,
  //    j             : None,
  //    rat           : None,
  //    ltb_id        : None, 
  //    cable_len     : 0.0,
  //    paddle1_id    : None,
  //    paddle2_id    : None,
  //    paddle3_id    : None,
  //    paddle4_id    : None,
  //    paddle5_id    : None,
  //    paddle6_id    : None,
  //    paddle7_id    : None,
  //    paddle8_id    : None,
  //  }
  //}

  /// True if sane dsi and j values are 
  /// assigned to this board
  pub fn connected(&self) -> bool {
    self.dsi != None && self.j != None
  }

  /// True if all fields are filled with 
  /// reasonable values and not the default
  pub fn valid(&self) -> bool {
    self.board_id      > 0 &&    
    self.dsi       .is_some() && 
    self.j         .is_some() && 
    self.rat       .is_some() && 
    // right now, we explicitly don't care
    // about the ltb_id
    //self.ltb_id    .is_some() &&  
    self.cable_len     > 0.0  &&
    self.paddle1_id.is_some() &&
    self.paddle2_id.is_some() &&
    self.paddle3_id.is_some() &&
    self.paddle4_id.is_some() &&
    self.paddle5_id.is_some() &&
    self.paddle6_id.is_some() &&
    self.paddle7_id.is_some() &&
    self.paddle8_id.is_some()
  }
  
  pub fn all(conn: &mut SqliteConnection) -> Option<Vec<DBLocalTriggerBoard>> {
    use schema::tof_db_localtriggerboard::dsl::*;
    match tof_db_localtriggerboard
        //.inner_join(tof_db_localtriggerboard.on(schema::tof_db_paddle::dsl::paddle_id.eq(schema::tof_db_localtriggerboard::dsl::paddle1_id)))
        .load::<DBLocalTriggerBoard>(conn) {
      Err(err) => {
        error!("Unable to load LocalTriggerBoards from db! {err}");
        return None;
      }
      Ok(ltbs) => {
        return Some(ltbs);
      }
    }
  }
}

impl fmt::Display for DBLocalTriggerBoard {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr : String;
    if !self.connected() {
      repr = format!("<DBLocalTriggerBoard: ID {}  - UNCONNECTED>", self.board_id);
    } else {
      repr = String::from("<DBLocalTriggerBoard:");
      repr += &(format!("\n  LTB ID  : {}", self.board_id));             
    }
    repr += &(format!("\n  DSI/J   : {}/{}", self.dsi.unwrap(), self.j.unwrap()));     
    repr += &(format!("\n  RAT ID  : {}", self.rat.unwrap()));
    repr += "\n  H. cable len (MTB connection):";
    repr += &(format!("\n    ->      {}", self.cable_len));
    repr += "\n  -- -- -- -- -- -- -- -- -- -- -- -- -- --";
    repr += "\n  Paddle IDs:";
    repr += &(format!("\n    {:02}", self.paddle1_id.unwrap_or(-1))); 
    repr += &(format!("\n    {:02}", self.paddle2_id.unwrap_or(-1)));  
    repr += &(format!("\n    {:02}", self.paddle3_id.unwrap_or(-1)));  
    repr += &(format!("\n    {:02}", self.paddle4_id.unwrap_or(-1)));  
    repr += &(format!("\n    {:02}", self.paddle5_id.unwrap_or(-1))); 
    repr += &(format!("\n    {:02}", self.paddle6_id.unwrap_or(-1))); 
    repr += &(format!("\n    {:02}", self.paddle7_id.unwrap_or(-1))); 
    repr += &(format!("\n    {:02}", self.paddle8_id.unwrap_or(-1))); 
    write!(f, "{}", repr)
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LocalTriggerBoard {
    pub board_id      : u8,    
    pub dsi           : u8,
    pub j             : u8,
    pub rat           : u8,
    pub ltb_id        : u8, 
    pub cable_len     : f32,
    pub paddle1       : Paddle,
    pub paddle2       : Paddle,
    pub paddle3       : Paddle,
    pub paddle4       : Paddle,
    pub paddle5       : Paddle,
    pub paddle6       : Paddle,
    pub paddle7       : Paddle,
    pub paddle8       : Paddle,
}

impl LocalTriggerBoard {
  
  pub fn new() -> Self {
    Self {
      board_id      : 0,    
      dsi           : 0,
      j             : 0,
      rat           : 0,
      ltb_id        : 0, 
      cable_len     : 0.0,
      paddle1       : Paddle::new(),
      paddle2       : Paddle::new(),
      paddle3       : Paddle::new(),
      paddle4       : Paddle::new(),
      paddle5       : Paddle::new(),
      paddle6       : Paddle::new(),
      paddle7       : Paddle::new(),
      paddle8       : Paddle::new(),
    }
  }
  
  pub fn all(conn: &mut SqliteConnection) -> Option<Vec<LocalTriggerBoard>> {
    use schema::tof_db_localtriggerboard::dsl::*;
    let db_ltbs : Vec<DBLocalTriggerBoard>;
    match tof_db_localtriggerboard
        //.inner_join(tof_db_localtriggerboard.on(schema::tof_db_paddle::dsl::paddle_id.eq(schema::tof_db_localtriggerboard::dsl::paddle1_id)))
        .load::<DBLocalTriggerBoard>(conn) {
      Err(err) => {
        error!("Unable to load LocalTriggerBoards from db! {err}");
        return None;
      }
      Ok(ltbs) => {
        db_ltbs = ltbs;
      }
    }
    let paddles_op = Paddle::all(conn);
    match paddles_op {
      None => {
        return None;
      }
      Some(_) => ()
    }
    let paddles = paddles_op.unwrap();
    // This is not the best and fastest, but since our diesel skills 
    // are a merely 3, we can't do it right now.
    let mut ltbs = Vec::<LocalTriggerBoard>::new();
    //println!("Iterating over {} ltbs in the DB!", db_ltbs.len());
    for dbltb in db_ltbs {
      let mut ltb  = LocalTriggerBoard::new();
      for pdl in paddles.iter() {
        // this call ensures that the following unwraps
        // go through
        if !dbltb.valid() {
          error!("Got unpopulated LTB from DB for LTB {}", dbltb);
          continue;
        }
        if pdl.paddle_id == dbltb.paddle1_id.unwrap() {
          ltb.board_id  = dbltb.board_id as u8;        
          ltb.dsi       = dbltb.dsi.unwrap_or(0) as u8;
          ltb.j         = dbltb.j.unwrap_or(0) as u8;     
          ltb.rat       = dbltb.rat.unwrap_or(0) as u8;     
          ltb.ltb_id    = dbltb.ltb_id.unwrap_or(0) as u8;    
          ltb.cable_len = dbltb.cable_len;    
          ltb.paddle1   = pdl.clone();
        }
        if pdl.paddle_id == dbltb.paddle2_id.unwrap() {
          ltb.paddle2   = pdl.clone();
        }
        if pdl.paddle_id == dbltb.paddle3_id.unwrap() {
          ltb.paddle3   = pdl.clone();
        }
        if pdl.paddle_id == dbltb.paddle4_id.unwrap() {
          ltb.paddle4   = pdl.clone();
        }
        if pdl.paddle_id == dbltb.paddle5_id.unwrap() {
          ltb.paddle5   = pdl.clone();
        }
        if pdl.paddle_id == dbltb.paddle6_id.unwrap() {
          ltb.paddle6   = pdl.clone();
        }
        if pdl.paddle_id == dbltb.paddle7_id.unwrap() {
          ltb.paddle7   = pdl.clone();
        }
        if pdl.paddle_id == dbltb.paddle8_id.unwrap() {
          ltb.paddle8   = pdl.clone();
        }
      }
      ltbs.push(ltb);
    }
    Some(ltbs)
  }
}

impl fmt::Display for LocalTriggerBoard {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr : String;
    repr = String::from("<LocalTriggerBoard:");
    repr += &(format!("\n  LTB ID  : {}", self.board_id));             
    repr += &(format!("\n  DSI/J   : {}/{}", self.dsi, self.j));     
    repr += &(format!("\n  RAT ID  : {}", self.rat));
    repr += "\n  H. cable len (MTB connection):";
    repr += &(format!("\n    ->      {}", self.cable_len));
    repr += "\n  -- -- -- -- -- -- -- -- -- -- -- -- -- --";
    repr += "\n  LTB Ch -> RB Id, RB chn, Pdl ID, Pan ID:";
    repr += &(format!("\n            {:02}   |   {},{} |  {:03} | {:02}",  self.paddle1.rb_id, self.paddle1.rb_chA, self.paddle1.rb_chB, self.paddle1.paddle_id, self.paddle1.panel_id)); 
    repr += &(format!("\n            {:02}   |   {},{} |  {:03} | {:02}",  self.paddle2.rb_id, self.paddle2.rb_chA, self.paddle2.rb_chB, self.paddle2.paddle_id, self.paddle2.panel_id));  
    repr += &(format!("\n            {:02}   |   {},{} |  {:03} | {:02}",  self.paddle3.rb_id, self.paddle3.rb_chA, self.paddle3.rb_chB, self.paddle3.paddle_id, self.paddle3.panel_id));  
    repr += &(format!("\n            {:02}   |   {},{} |  {:03} | {:02}",  self.paddle4.rb_id, self.paddle4.rb_chA, self.paddle4.rb_chB, self.paddle4.paddle_id, self.paddle4.panel_id));  
    repr += &(format!("\n            {:02}   |   {},{} |  {:03} | {:02}",  self.paddle5.rb_id, self.paddle5.rb_chA, self.paddle5.rb_chB, self.paddle5.paddle_id, self.paddle5.panel_id)); 
    repr += &(format!("\n            {:02}   |   {},{} |  {:03} | {:02}",  self.paddle6.rb_id, self.paddle6.rb_chA, self.paddle6.rb_chB, self.paddle6.paddle_id, self.paddle6.panel_id)); 
    repr += &(format!("\n            {:02}   |   {},{} |  {:03} | {:02}",  self.paddle7.rb_id, self.paddle7.rb_chA, self.paddle7.rb_chB, self.paddle7.paddle_id, self.paddle7.panel_id)); 
    repr += &(format!("\n            {:02}   |   {},{} |  {:03} | {:02}>", self.paddle8.rb_id, self.paddle8.rb_chA, self.paddle8.rb_chB, self.paddle8.paddle_id, self.paddle8.panel_id)); 
    write!(f, "{}", repr)
  }
}

/// A Readoutboard with paddles connected
/// 
#[derive(Debug,PartialEq, Clone,Queryable, Selectable, serde::Serialize, serde::Deserialize)]
#[diesel(table_name = schema::tof_db_readoutboard)]
#[diesel(primary_key(rb_id_id))]
#[allow(non_snake_case)]
pub struct DBReadoutBoard {
  // FIXME - this HAS TO BE (MUST!) the same order
  // as in schema.rs !!
  pub rb_id        : i16, 
  pub dsi          : i16, 
  pub j            : i16, 
  pub mtb_link_id  : i16, 
  pub paddle12_chA : Option<i16>,
  pub paddle34_chA : Option<i16>,
  pub paddle56_chA : Option<i16>,
  pub paddle78_chA : Option<i16>,
  pub paddle12_id  : Option<i16>,
  pub paddle34_id  : Option<i16>,
  pub paddle56_id  : Option<i16>,
  pub paddle78_id  : Option<i16>,
}

impl DBReadoutBoard {
  pub fn all(conn: &mut SqliteConnection) -> Option<Vec<DBReadoutBoard>> {
    use schema::tof_db_readoutboard::dsl::*;
    match tof_db_readoutboard
        //.inner_join(tof_db_localtriggerboard.on(schema::tof_db_paddle::dsl::paddle_id.eq(schema::tof_db_localtriggerboard::dsl::paddle1_id)))
        .load::<DBReadoutBoard>(conn) {
      Err(err) => {
        error!("Unable to load ReadoutBoards from db! {err}");
        return None;
      }
      Ok(rbs) => {
        return Some(rbs);
      }
    }
  }
}

impl fmt::Display for DBReadoutBoard {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr  = String::from("<ReadoutBoard:");
    repr += &(format!("\n  Board id    : {}",self.rb_id));            
    repr += &(format!("\n  MTB Link ID : {}",self.mtb_link_id));
    repr += &(format!("\n  DSI/J       : {}/{}",self.dsi,self.j));
    repr += "\n **Connected paddles**";
    repr += &(format!("\n  Ch0/1(1/2)  : {}", self.paddle12_id.unwrap_or(-1)));         
    repr += &(format!("\n  Ch1/2(2/3)  : {}", self.paddle34_id.unwrap_or(-1)));         
    repr += &(format!("\n  Ch2/3(3/4)  : {}", self.paddle56_id.unwrap_or(-1)));         
    repr += &(format!("\n  Ch3/4(4/5)  : {}>",self.paddle78_id.unwrap_or(-1)));         
    write!(f, "{}", repr)
  }
}

/// A Readoutboard with paddles connected
#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct ReadoutBoard {
  pub rb_id           : u8, 
  pub dsi             : u8, 
  pub j               : u8, 
  pub mtb_link_id     : u8, 
  pub paddle12        : Paddle,
  pub paddle12_chA    : u8,
  pub paddle34        : Paddle,
  pub paddle34_chA    : u8,
  pub paddle56        : Paddle,
  pub paddle56_chA    : u8,
  pub paddle78        : Paddle,
  pub paddle78_chA    : u8,
  // extra stuff, not from the db
  // or maybe in the future?
  pub calib_file_path : String,
  pub calibration     : RBCalibrations,       
}

impl ReadoutBoard {

  pub fn new() -> Self {
    Self {
      rb_id           : 0, 
      dsi             : 0, 
      j               : 0, 
      mtb_link_id     : 0, 
      paddle12        : Paddle::new(),
      paddle12_chA    : 0,
      paddle34        : Paddle::new(),
      paddle34_chA    : 0,
      paddle56        : Paddle::new(),
      paddle56_chA    : 0,
      paddle78        : Paddle::new(),
      paddle78_chA    : 0,
      calib_file_path : String::from(""),
      calibration     : RBCalibrations::new(0),
    }
  }

  /// Returns the ip address following a convention
  ///
  /// This does NOT GUARANTEE that the address is correct!
  pub fn guess_address(&self) -> String {
    format!("tcp://10.0.1.1{:02}:42000", self.rb_id)
  }
 
  pub fn get_paddle_ids(&self) -> [u8;4] {
    let pid0 = self.paddle12.paddle_id as u8;
    let pid1 = self.paddle34.paddle_id as u8;
    let pid2 = self.paddle56.paddle_id as u8;
    let pid3 = self.paddle78.paddle_id as u8;
    [pid0, pid1, pid2, pid3]
  }

  #[allow(non_snake_case)]
  pub fn get_A_sides(&self) -> [u8;4] {
    let pa_0 = self.paddle12_chA;
    let pa_1 = self.paddle34_chA;
    let pa_2 = self.paddle56_chA;
    let pa_3 = self.paddle78_chA;
    [pa_0, pa_1, pa_2, pa_3]
  }

  #[allow(non_snake_case)]
  pub fn get_pid_rbchA(&self, pid : u8) -> Option<u8> {
    if self.paddle12.paddle_id as u8 == pid {
      let rv = self.paddle12.rb_chA as u8;
      return Some(rv);
    } else if self.paddle34.paddle_id as u8 == pid {
      let rv = self.paddle34.rb_chA as u8;
      return Some(rv);
    } else if self.paddle56.paddle_id as u8 == pid {
      let rv = self.paddle56.rb_chA as u8;
      return Some(rv);
    } else if self.paddle78.paddle_id as u8== pid {
      let rv = self.paddle78.rb_chA as u8;
      return Some(rv);
    } else {
      return None;
    }
  }
  
  #[allow(non_snake_case)]
  pub fn get_pid_rbchB(&self, pid : u8) -> Option<u8> {
    if self.paddle12.paddle_id as u8 == pid {
      let rv = self.paddle12.rb_chB as u8;
      return Some(rv);
    } else if self.paddle34.paddle_id as u8== pid {
      let rv = self.paddle34.rb_chB as u8;
      return Some(rv);
    } else if self.paddle56.paddle_id as u8== pid {
      let rv = self.paddle56.rb_chB as u8;
      return Some(rv);
    } else if self.paddle78.paddle_id as u8 == pid {
      let rv = self.paddle78.rb_chB as u8;
      return Some(rv);
    } else {
      return None;
    }
  }

  pub fn get_paddle_length(&self, pid : u8) -> Option<f32> {
    if self.paddle12.paddle_id as u8 == pid {
      let rv = self.paddle12.length;
      return Some(rv);
    } else if self.paddle34.paddle_id as u8== pid {
      let rv = self.paddle34.length;
      return Some(rv);
    } else if self.paddle56.paddle_id as u8== pid {
      let rv = self.paddle56.length;
      return Some(rv);
    } else if self.paddle78.paddle_id as u8 == pid {
      let rv = self.paddle78.length;
      return Some(rv);
    } else {
      return None;
    }
  }

  pub fn all(conn: &mut SqliteConnection) -> Option<Vec<ReadoutBoard>> {
    use schema::tof_db_readoutboard::dsl::*;
    let db_rbs : Vec<DBReadoutBoard>;
    match tof_db_readoutboard
        //.inner_join(tof_db_localtriggerboard.on(schema::tof_db_paddle::dsl::paddle_id.eq(schema::tof_db_localtriggerboard::dsl::paddle1_id)))
        .load::<DBReadoutBoard>(conn) {
      Err(err) => {
        error!("Unable to load ReadoutBoards from db! {err}");
        return None;
      }
      Ok(rbs) => {
        db_rbs = rbs;
      }
    }
    let paddles_op = Paddle::all(conn);
    match paddles_op {
      None => {
        return None;
      }
      Some(_) => ()
    }
    let paddles = paddles_op.unwrap();
    // This is not the best and fastest, but since our diesel skills 
    // are a merely 3, we can't do it right now.
    let mut rbs = Vec::<ReadoutBoard>::new();
    //println!("Iterating over {} rbs in the DB!", db_rbs.len());
    for dbrb in db_rbs {
      let mut rb  = ReadoutBoard::new();
      rb.rb_id        = dbrb.rb_id as u8;        
      rb.dsi          = dbrb.dsi as u8;
      rb.j            = dbrb.j  as u8;     
      rb.mtb_link_id  = dbrb.mtb_link_id  as u8;    
      rb.paddle12_chA = dbrb.paddle12_chA.unwrap() as u8;
      rb.paddle34_chA = dbrb.paddle34_chA.unwrap() as u8;
      rb.paddle56_chA = dbrb.paddle56_chA.unwrap() as u8;
      rb.paddle78_chA = dbrb.paddle78_chA.unwrap() as u8;
      for pdl in paddles.iter() {
        // this call ensures that the following unwraps
        // go through
        //if !dbltb.valid() {
        //  error!("Got unpopulated LTB from DB for LTB {}", dbltb);
        //  continue;
        //}
        if pdl.paddle_id == dbrb.paddle12_id.unwrap() {
          rb.paddle12     = pdl.clone();
        }
        if pdl.paddle_id == dbrb.paddle34_id.unwrap() {
          rb.paddle34   = pdl.clone();
        }
        if pdl.paddle_id == dbrb.paddle56_id.unwrap() {
          rb.paddle56   = pdl.clone();
        }
        if pdl.paddle_id == dbrb.paddle78_id.unwrap() {
          rb.paddle78   = pdl.clone();
        }
      }
      rbs.push(rb);
    }
    Some(rbs)
  }
  
  // FIXME - better query
  pub fn where_rbid(conn: &mut SqliteConnection, rb_id : u8) -> Option<ReadoutBoard> {
    let all = ReadoutBoard::all(conn)?;
    for rb in all {
      if rb.rb_id == rb_id {
        return Some(rb);
      }
    }
    None
  }

  pub fn to_summary_str(&self) -> String {
    let mut repr  = String::from("<ReadoutBoard:");
    repr += &(format!("\n  Board id    : {}",self.rb_id));            
    repr += &(format!("\n  MTB Link ID : {}",self.mtb_link_id));
    repr += &(format!("\n  RAT         : {}",self.paddle12.ltb_id));
    repr += &(format!("\n  DSI/J       : {}/{}",self.dsi,self.j));
    repr += "\n **Connected paddles**";
    repr += &(format!("\n  Channel 1/2 : {:02} (panel {:01})", self.paddle12.paddle_id, self.paddle12.panel_id));
    repr += &(format!("\n  Channel 3/4 : {:02} (panel {:01})", self.paddle34.paddle_id, self.paddle34.panel_id));
    repr += &(format!("\n  Channel 5/6 : {:02} (panel {:01})", self.paddle56.paddle_id, self.paddle56.panel_id));
    repr += &(format!("\n  Channel 7/8 : {:02} (panel {:01})", self.paddle78.paddle_id, self.paddle78.panel_id));
    repr
  }

  /// Load the newest calibration from the calibration file path
  pub fn load_latest_calibration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    //  files look like RB20_2024_01_26-08_15_54.cali.tof.gaps
    //let re = Regex::new(r"(\d{4}_\d{2}_\d{2}-\d{2}_\d{2}_\d{2})")?;
    let re = Regex::new(r"(\d{6}_\d{6})")?;
    // Define your file pattern (e.g., "logs/*.log" for all .log files in the logs directory)
    let pattern = format!("{}/RB{:02}_*", self.calib_file_path, self.rb_id); // Adjust this pattern to your files' naming convention
    let timestamp = DateTime::<Utc>::from_timestamp(0,0).unwrap(); // I am not sure what to do here
                                                                   // otherwise than unwrap. How is
                                                                   // this allowed to fail?
    //let mut newest_file = (String::from(""), NaiveDateTime::from_timestamp(0, 0));
    let mut newest_file = (String::from(""), timestamp);

    // Iterate over files that match the pattern
    let mut filename : String;
    for entry in glob(&pattern)? {
      if let Ok(path) = entry {
        // Get the filename as a string
        //let cpath = path.clone();
        match path.file_name() {
          None => continue,
          Some(fname) => {
              // the expect might be ok, since this is something done during initialization
              filename = fname.to_os_string().into_string().expect("Unwrapping filename failed!");
          }
        }
        if let Some(caps) = re.captures(&filename) {
          if let Some(timestamp_str) = caps.get(0).map(|m| m.as_str()) {
            //println!("timestamp_str {}, {}",timestamp_str, HUMAN_TIMESTAMP_FORMAT);
            //let timestamp = NaiveDateTime::parse_from_str(timestamp_str, "%Y_%m_%d-%H_%M_%S")?;
            //let timestamp = DateTime::<Utc>::parse_from_str(timestamp_str, "%Y_%m_%d-%H_%M_%S")?;
            let footzstring = format!("{}+0000", timestamp_str);
            let timestamp = DateTime::parse_from_str(&footzstring, "%y%m%d_%H%M%S%z")?;
            //let timestamp = DateTime::parse_from_str(&footzstring, HUMAN_TIMESTAMP_FORMAT)?;
            //println!("parse successful");
            //let _timestamp = DateTime
            if timestamp > newest_file.1 {
              // FIXME - into might panic?
              newest_file.1 = timestamp.into();
              newest_file.0 = filename.clone();
            }
          }
        }
      }
    }
    
    if newest_file.0.is_empty() {
      error!("No matching calibration available for board {}!", self.rb_id);
    } else {
      let file_to_load = format!("{}/{}", self.calib_file_path, newest_file.0);
      info!("Loading calibration from file: {}", file_to_load);
      self.calibration = RBCalibrations::from_file(file_to_load, true)?;
    }
    Ok(())
  }
}

impl fmt::Display for ReadoutBoard {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr  = String::from("<ReadoutBoard:");
    repr += &(format!("\n  Board id    : {}",self.rb_id));            
    repr += &(format!("\n  MTB Link ID : {}",self.mtb_link_id));
    repr += &(format!("\n  DSI/J       : {}/{}",self.dsi,self.j));
    repr += "\n **Connected paddles**";
    repr += &(format!("\n  Ch0/1(1/2)  : {}",self.paddle12)); 
    repr += &(format!("\n    A-side    : {}", self.paddle12_chA));
    repr += &(format!("\n  Ch1/2(2/3)  : {}",self.paddle34));         
    repr += &(format!("\n    A-side    : {}", self.paddle34_chA));
    repr += &(format!("\n  Ch2/3(3/4)  : {}",self.paddle56));         
    repr += &(format!("\n    A-side    : {}", self.paddle56_chA));
    repr += &(format!("\n  Ch3/4(4/5)  : {}>",self.paddle78));         
    repr += &(format!("\n    A-side    : {}", self.paddle78_chA));
    repr += "** calibration will be loaded from this path:";
    repr += &(format!("\n      \u{021B3} {}", self.calib_file_path));
    repr += &(format!("\n  calibration : {}>", self.calibration));
    write!(f, "{}", repr)
  }
}


/// A TOF Panel is a larger unit of paddles next to each other
///
/// TOF faces (e.g. Umbrella) can have multiple Panels
#[derive(Debug, Clone,Queryable, Selectable)]
#[diesel(table_name = schema::tof_db_panel)]
#[diesel(primary_key(panel_id))]
pub struct DBPanel {
  // ORDER OF THESE FIELDS HAS TO BE THE SAME AS IN schema.rs!!
  pub  panel_id    : i16        ,   
  pub  description : String     ,   
  pub  normal_x    : i16        ,   
  pub  normal_y    : i16        ,   
  pub  normal_z    : i16        ,   
  pub  dw_paddle   : Option<i16>,   
  pub  dh_paddle   : Option<i16>,   
  pub  paddle0_id  : Option<i16>,   
  pub  paddle1_id  : Option<i16>,   
  pub  paddle10_id : Option<i16>,   
  pub  paddle11_id : Option<i16>,   
  pub  paddle2_id  : Option<i16>,   
  pub  paddle3_id  : Option<i16>,   
  pub  paddle4_id  : Option<i16>,   
  pub  paddle5_id  : Option<i16>,   
  pub  paddle6_id  : Option<i16>,   
  pub  paddle7_id  : Option<i16>,   
  pub  paddle8_id  : Option<i16>,   
  pub  paddle9_id  : Option<i16>,   
}

impl DBPanel {

  pub fn valid(&self) -> bool {
    self.panel_id    > 0 &&    
    self.description != String::from("") &&   
    self.paddle0_id.is_some()   
  }

  pub fn all(conn: &mut SqliteConnection) -> Option<Vec<DBPanel>> {
    use schema::tof_db_panel::dsl::*;
    match tof_db_panel
        //.inner_join(tof_db_localtriggerboard.on(schema::tof_db_paddle::dsl::paddle_id.eq(schema::tof_db_localtriggerboard::dsl::paddle1_id)))
        .load::<DBPanel>(conn) {
      Err(err) => {
        error!("Unable to load Panels from db! {err}");
        return None;
      }
      // dirty mind check
      Ok(pnls) => {
        return Some(pnls);
      }
    }
  }
  
  pub fn get_npaddles(&self) -> u8 {
    let mut npaddles = 0u8;
    if self.paddle0_id.is_some() {
      npaddles += 1;
    }
    if self.paddle1_id.is_some() {
      npaddles += 1;
    }
    if self.paddle2_id.is_some() {
      npaddles += 1;
    }
    if self.paddle3_id.is_some() {
      npaddles += 1;
    }
    if self.paddle4_id.is_some() {
      npaddles += 1;
    }
    if self.paddle5_id.is_some() {
      npaddles += 1;
    }
    if self.paddle6_id.is_some() {
      npaddles += 1;
    }
    if self.paddle7_id.is_some() {
      npaddles += 1;
    }
    if self.paddle8_id.is_some() {
      npaddles += 1;
    }
    if self.paddle9_id.is_some() {
      npaddles += 1;
    }
    if self.paddle10_id.is_some() {
      npaddles += 1;
    }
    if self.paddle11_id.is_some() {
      npaddles += 1;
    }
    npaddles
  }
}

impl fmt::Display for DBPanel {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<DBPanel");
    repr += &(format!("\n  id    : {}",self.panel_id));
    repr += &(format!("\n  descr : {}",self.description));
    repr += "\n  orientation:";
    repr += &(format!("\n   [{},{},{}]", self.normal_x, self.normal_y, self.normal_z));
    repr += &(format!("\n  paddle list ({}) paddles)", self.get_npaddles()));
    if self.paddle0_id.is_some() {
      repr += &(format!("\n   {}",self.paddle0_id.unwrap()));
    }
    if self.paddle1_id.is_some() {
      repr += &(format!("\n   {}",self.paddle1_id.unwrap()));
    }
    if self.paddle2_id.is_some() { 
      repr += &(format!("\n   {}",self.paddle2_id.unwrap()));
    }
    if self.paddle3_id.is_some() { 
      repr += &(format!("\n   {}",self.paddle3_id.unwrap()));
    }
    if self.paddle4_id.is_some() {
      repr += &(format!("\n   {}",self.paddle4_id.unwrap()));
    }
    if self.paddle5_id.is_some() {
      repr += &(format!("\n   {}",self.paddle5_id.unwrap()));
    }
    if self.paddle6_id.is_some()  {
      repr += &(format!("\n   {}",self.paddle6_id.unwrap()));
    }
    if self.paddle7_id.is_some() {
      repr += &(format!("\n   {}",self.paddle7_id.unwrap()));
    }
    if self.paddle8_id.is_some() {
      repr += &(format!("\n   {}",self.paddle8_id.unwrap()));
    }
    if self.paddle9_id.is_some() {
      repr += &(format!("\n   {}",self.paddle9_id.unwrap()));
    }
    if self.paddle10_id.is_some() {
      repr += &(format!("\n   {}",self.paddle10_id.unwrap()));
    }
    if self.paddle11_id.is_some() {
      repr += &(format!("\n   {}",self.paddle11_id.unwrap()));
    }
    repr += ">";
    write!(f, "{}", repr)
  }
}

pub struct Panel {
  pub  panel_id    : u8        ,   
  pub  description : String    ,   
  pub  normal_x    : u8        ,   
  pub  normal_y    : u8        ,   
  pub  normal_z    : u8        ,   
  pub  paddle0  : Paddle,   
  pub  paddle1  : Option<Paddle>,   
  pub  paddle2  : Option<Paddle>,   
  pub  paddle3  : Option<Paddle>,   
  pub  paddle4  : Option<Paddle>,   
  pub  paddle5  : Option<Paddle>,   
  pub  paddle6  : Option<Paddle>,   
  pub  paddle7  : Option<Paddle>,   
  pub  paddle8  : Option<Paddle>,   
  pub  paddle9  : Option<Paddle>,   
  pub  paddle10 : Option<Paddle>,   
  pub  paddle11 : Option<Paddle>,   
  // FIXME - these are for the future 
  // when we are buiding the geometry 
  // from the database
  //pub  dh_paddle   : Option<>,   
  //pub  dw_paddle   : Option<>,   
}

impl Panel {
 
  pub fn new() -> Self {
    Self {
      panel_id    : 0        ,   
      description : String::from(""),   
      normal_x    : 0        ,   
      normal_y    : 0        ,   
      normal_z    : 0        ,   
      paddle0     : Paddle::new(),   
      paddle1     : None,   
      paddle2     : None,   
      paddle3     : None,   
      paddle4     : None,   
      paddle5     : None,   
      paddle6     : None,   
      paddle7     : None,   
      paddle8     : None,   
      paddle9     : None,   
      paddle10    : None,   
      paddle11    : None,   
    }
  }


  pub fn get_npaddles(&self) -> u8 {
    let mut npaddles = 1u8;
    if self.paddle1.is_some() {
      npaddles += 1;
    }
    if self.paddle2.is_some() {
      npaddles += 1;
    }
    if self.paddle3.is_some() {
      npaddles += 1;
    }
    if self.paddle4.is_some() {
      npaddles += 1;
    }
    if self.paddle5.is_some() {
      npaddles += 1;
    }
    if self.paddle6.is_some() {
      npaddles += 1;
    }
    if self.paddle7.is_some() {
      npaddles += 1;
    }
    if self.paddle8.is_some() {
      npaddles += 1;
    }
    if self.paddle9.is_some() {
      npaddles += 1;
    }
    if self.paddle10.is_some() {
      npaddles += 1;
    }
    if self.paddle11.is_some() {
      npaddles += 1;
    }
    npaddles
  }
  
  pub fn all(conn: &mut SqliteConnection) -> Option<Vec<Panel>> {
    use schema::tof_db_panel::dsl::*;
    let db_panels : Vec<DBPanel>;
    match tof_db_panel
        //.inner_join(tof_db_localtriggerboard.on(schema::tof_db_paddle::dsl::paddle_id.eq(schema::tof_db_localtriggerboard::dsl::paddle1_id)))
        .load::<DBPanel>(conn) {
      Err(err) => {
        error!("Unable to load Panels from db! {err}");
        return None;
      }
      Ok(pnls) => {
        db_panels = pnls;
      }
    }
    let paddles_op = Paddle::all(conn);
    match paddles_op {
      None => {
        return None;
      }
      Some(_) => ()
    }
    let paddles = paddles_op.unwrap();
    // This is not the best and fastest, but since our diesel skills 
    // are a merely 3, we can't do it right now.
    let mut panels = Vec::<Panel>::new();
    println!("Iterating over {} panels in the DB!", db_panels.len());
    for dbpanel in db_panels {
      let mut pnl  = Panel::new();
      for pdl in paddles.iter() {
        // this call ensures that the following unwraps
        // go through
        if !dbpanel.valid() {
          error!("Got unpopulated Panel from DB for Panel {}", dbpanel);
          continue;
        }
        if pdl.paddle_id == dbpanel.paddle0_id.unwrap() {
          pnl.panel_id     = dbpanel.panel_id as u8;        
          pnl.description  = dbpanel.description.clone();
          pnl.normal_x     = dbpanel.normal_x as u8;     
          pnl.normal_y     = dbpanel.normal_y as u8;     
          pnl.normal_z     = dbpanel.normal_z as u8;    
          pnl.paddle0      = pdl.clone();
        }
        if pdl.paddle_id == dbpanel.paddle1_id.unwrap() {
          pnl.paddle1   = Some(pdl.clone());
        }
        if pdl.paddle_id == dbpanel.paddle2_id.unwrap() {
          pnl.paddle2   = Some(pdl.clone());
        }
        if pdl.paddle_id == dbpanel.paddle3_id.unwrap() {
          pnl.paddle3   = Some(pdl.clone());
        }
        if pdl.paddle_id == dbpanel.paddle4_id.unwrap() {
          pnl.paddle4   = Some(pdl.clone());
        }
        if pdl.paddle_id == dbpanel.paddle5_id.unwrap() {
          pnl.paddle5   = Some(pdl.clone());
        }
        if pdl.paddle_id == dbpanel.paddle6_id.unwrap() {
          pnl.paddle6   = Some(pdl.clone());
        }
        if pdl.paddle_id == dbpanel.paddle7_id.unwrap() {
          pnl.paddle7   = Some(pdl.clone());
        }
        if pdl.paddle_id == dbpanel.paddle8_id.unwrap() {
          pnl.paddle8   = Some(pdl.clone());
        }
        if pdl.paddle_id == dbpanel.paddle9_id.unwrap() {
          pnl.paddle9   = Some(pdl.clone());
        }
        if pdl.paddle_id == dbpanel.paddle10_id.unwrap() {
          pnl.paddle10   = Some(pdl.clone());
        }
        if pdl.paddle_id == dbpanel.paddle11_id.unwrap() {
          pnl.paddle11   = Some(pdl.clone());
        }
      }
      panels.push(pnl);
    }
    Some(panels)
  }
}

impl fmt::Display for Panel {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<Panel");
    repr += &(format!("\n  id    : {}",self.panel_id));
    repr += &(format!("\n  descr : {}",self.description));
    repr += "\n  orientation:";
    repr += &(format!("\n   [{},{},{}]", self.normal_x, self.normal_y, self.normal_z));
    repr += &(format!("\n  paddle list ({}) paddles)", self.get_npaddles()));
    repr += &(format!("\n   {}",self.paddle0));
    if self.paddle1.is_some() {
      repr += &(format!("\n   {}",self.paddle1.as_ref().unwrap()));
    }
    if self.paddle2.is_some() { 
      repr += &(format!("\n   {}",self.paddle2.as_ref().unwrap()));
    }
    if self.paddle3.is_some() { 
      repr += &(format!("\n   {}",self.paddle3.as_ref().unwrap()));
    }
    if self.paddle4.is_some() {
      repr += &(format!("\n   {}",self.paddle4.as_ref().unwrap()));
    }
    if self.paddle5.is_some() {
      repr += &(format!("\n   {}",self.paddle5.as_ref().unwrap()));
    }
    if self.paddle6.is_some()  {
      repr += &(format!("\n   {}",self.paddle6.as_ref().unwrap()));
    }
    if self.paddle7.is_some() {
      repr += &(format!("\n   {}",self.paddle7.as_ref().unwrap()));
    }
    if self.paddle8.is_some() {
      repr += &(format!("\n   {}",self.paddle8.as_ref().unwrap()));
    }
    if self.paddle9.is_some() {
      repr += &(format!("\n   {}",self.paddle9.as_ref().unwrap()));
    }
    if self.paddle10.is_some() {
      repr += &(format!("\n   {}",self.paddle10.as_ref().unwrap()));
    }
    if self.paddle11.is_some() {
      repr += &(format!("\n   {}",self.paddle11.as_ref().unwrap()));
    }
    repr += ">";
    write!(f, "{}", repr)
  }
}




