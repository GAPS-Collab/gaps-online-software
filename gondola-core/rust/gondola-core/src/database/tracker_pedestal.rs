//! Database access & entities for gaps-online-software
//!
//! A local .sqlite database is shipped with gaps-online-software,
//! pre-populated with relevant meta data for the GAPS experiment.
// This file is part of gaps-online-software and published 
// under the GPLv3 license

use crate::prelude::*;

use crate::database::schema;
use diesel::prelude::*;

/// The db insert companion to TrackerStripPedestal
#[derive(Debug,PartialEq, Clone, Insertable)]
#[diesel(table_name = schema::tof_db_trackerstrippedestal)]
#[allow(non_snake_case)]
#[cfg_attr(feature="pybindings", pyclass)]
struct NewTrackerStripPedestal {
  pub strip_id            : i32,    
  pub volume_id           : i64,    
  pub utc_timestamp_start : i64,
  pub utc_timestamp_stop  : i64,
  pub name                : Option<String>,
  pub pedestal_mean       : f32, 
  pub pedestal_sigma      : f32, 
  pub is_mean_value       : bool,
}

impl NewTrackerStripPedestal {
  pub fn from(ped : &TrackerStripPedestal) -> Self {
    Self {
      strip_id            : ped.strip_id            ,    
      volume_id           : ped.volume_id           ,    
      utc_timestamp_start : ped.utc_timestamp_start ,
      utc_timestamp_stop  : ped.utc_timestamp_stop  ,
      name                : ped.name.clone()        ,
      pedestal_mean       : ped.pedestal_mean       , 
      pedestal_sigma      : ped.pedestal_sigma      , 
      is_mean_value       : ped.is_mean_value       ,
    }
  }
}

#[cfg_attr(feature="pybindings", pyfunction)]
pub fn create_trk_pedestal_table( db_path: &str, pedestals: Vec<TrackerStripPedestal>) { 
  use schema::tof_db_trackerstrippedestal::dsl::*;
  let mut conn = SqliteConnection::establish(db_path).ok().unwrap(); 
  let mut _query_result = diesel::sql_query("
      CREATE TABLE IF NOT EXISTS tof_db_trackerstrippedestal (
          data_id INTEGER PRIMARY KEY AUTOINCREMENT,
          strip_id INTEGER NOT NULL,
          volume_id BIGINT NOT NULL,
          utc_timestamp_start BIGINT NOT NULL,
          utc_timestamp_stop BIGINT NOT NULL,
          name TEXT,
          pedestal_mean  FLOAT NOT NULL, 
          pedestal_sigma FLOAT NOT NULL, 
          is_mean_value BOOLEAN NOT NULL 
      )
  ").execute(&mut conn);
  let mut new_peds = Vec::<NewTrackerStripPedestal>::new();
  for p in pedestals {
    let np = NewTrackerStripPedestal::from(&p);
    new_peds.push(np);
  }
  _query_result = diesel::insert_into(tof_db_trackerstrippedestal)
    .values(&new_peds)
    .execute(&mut conn);
}



