//! Database access & entities for gaps-online-software
//!
//! A local .sqlite database is shipped with gaps-online-software,
//! pre-populated with relevant meta data for the GAPS experiment.
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

use crate::database::schema;
use diesel::prelude::*;

/// The db insert companion to TrackerStripMask
#[derive(Debug,PartialEq, Clone, Insertable)]
#[diesel(table_name = schema::tof_db_trackerstripmask)]
#[allow(non_snake_case)]
#[cfg_attr(feature="pybindings", pyclass)]
struct NewTrackerStripMask {
  pub strip_id            : i32,    
  pub volume_id           : i64,    
  pub utc_timestamp_start : i64,
  pub utc_timestamp_stop  : i64,
  pub name                : Option<String>, 
  pub active              : bool,   
}

impl NewTrackerStripMask {
  pub fn from(mask : &TrackerStripMask) -> Self {
    Self {
      strip_id            : mask.strip_id             ,    
      volume_id           : mask.volume_id            ,    
      utc_timestamp_start : mask.utc_timestamp_start  ,
      utc_timestamp_stop  : mask.utc_timestamp_stop   ,
      name                : mask.name.clone()         , 
      active              : mask.active               ,   
    }
  }
}

#[cfg_attr(feature="pybindings", pyfunction)]
pub fn create_trk_mask_table( db_path: &str, masks: Vec<TrackerStripMask>) { 
  use schema::tof_db_trackerstripmask::dsl::*;
  //use schema::tof_db_trackerstripmask;
  let mut conn = SqliteConnection::establish(db_path).ok().unwrap();
  
  let mut _query_result = diesel::sql_query("
      CREATE TABLE IF NOT EXISTS tof_db_trackerstripmask (
          data_id INTEGER PRIMARY KEY AUTOINCREMENT,
          strip_id INTEGER NOT NULL,
          volume_id BIGINT NOT NULL,
          utc_timestamp_start BIGINT NOT NULL,
          utc_timestamp_stop BIGINT NOT NULL,
          name TEXT,
          active BOOLEAN NOT NULL
      )
  ").execute(&mut conn);
  let mut new_masks = Vec::<NewTrackerStripMask>::new();
  for m in masks {
    let nm = NewTrackerStripMask::from(&m);
    new_masks.push(nm);
  }
  _query_result = diesel::insert_into(tof_db_trackerstripmask)
    .values(&new_masks)
    .execute(&mut conn);
}

