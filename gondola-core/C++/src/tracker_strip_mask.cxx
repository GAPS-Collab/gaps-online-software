// This file is part of gaps-online-software and published 
// under the GPLv3 license

#ifdef BUILD_CXX_DB

#include <format>
//#include <iostream>

#include "spdlog/spdlog.h"
#include "database.h"

using namespace sqlite_orm;
namespace g = gondola;

auto g::TrackerStripMask::to_string() const -> std::string {
  std::string repr = "<TrackerStripMask:";
  repr += std::format("\n strip id        : {}",  strip_id );
  repr += std::format("\n volume id       : {}",  volume_id);
  repr += std::format("\n Timestamp (UTC) start/stop : {} {}",  utc_timestamp_start, utc_timestamp_stop);
  repr += std::format("\n mask name       : {}",  name); 
  repr += std::format("\n active          : {}>", active    ); 
  return repr;
}

//-----------------------------------------------------------------------

auto g::get_trackerstripmasks(std::string mask_name) -> g::TrkStripMaskMap {
  g::TrkStripMaskMap mask_map;
  auto db_path = std::getenv("GONDOLA_DB_URL");
  if (db_path == nullptr) {
    spdlog::error("Unable to retrieve database! The GONDOLA_DB_URL shell variable is not set. Did you load the setup-env.sh shell?");
    return mask_map;
  } 
  std::string dbname(db_path);
  auto storage = make_storage(dbname,
    make_table("tof_db_trackerstripmask",
      make_column("data_id"             , &g::TrackerStripMask::strip_id, primary_key()),
      make_column("strip_id"             , &g::TrackerStripMask::strip_id),
      make_column("volume_id"            , &g::TrackerStripMask::volume_id),  
      make_column("utc_timestamp_start"  , &g::TrackerStripMask::utc_timestamp_start),
      make_column("utc_timestamp_stop"   , &g::TrackerStripMask::utc_timestamp_stop),
      make_column("name"                 , &g::TrackerStripMask::name),
      make_column("active"               , &g::TrackerStripMask::active)));  
  
  auto masks = storage.get_all<g::TrackerStripMask>();
  for (auto const &m : masks) {
    if (mask_name != "") {
      if (m.name != mask_name) {
        continue;
      }
    }
    mask_map.insert({m.strip_id, m.active});
  }  
  return mask_map;
}

//-----------------------------------------------------------------------

// FIXME - this needs urgent refactoring - introduce a get_all method first
auto g::get_trackerstripmasks_by_volumeid(std::string mask_name) -> TrkStripMaskMap {
  g::TrkStripMaskMap mask_map;
  auto db_path = std::getenv("GONDOLA_DB_URL");
  if (db_path == nullptr) {
    spdlog::error("Unable to retrieve database! The GONDOLA_DB_URL shell variable is not set. Did you load the setup-env.sh shell?");
    return mask_map;
  } 
  std::string dbname(db_path);
  auto storage = make_storage(dbname,
    make_table("tof_db_trackerstripmask",
      make_column("data_id"             , &g::TrackerStripMask::strip_id, primary_key()),
      make_column("strip_id"             , &g::TrackerStripMask::strip_id),
      make_column("volume_id"            , &g::TrackerStripMask::volume_id),  
      make_column("utc_timestamp_start"  , &g::TrackerStripMask::utc_timestamp_start),
      make_column("utc_timestamp_stop"   , &g::TrackerStripMask::utc_timestamp_stop),
      make_column("name"                 , &g::TrackerStripMask::name),
      make_column("active"               , &g::TrackerStripMask::active)));  
  
  auto masks = storage.get_all<g::TrackerStripMask>();
  usize n_masks = 0;
  for (auto const &m : masks) {
    if (mask_name != "") {
      if (m.name != mask_name) {
        continue;
      }
    }
    mask_map.insert({m.volume_id, m.active});
    n_masks++;
  }  
  spdlog::info("Found {} masks for {}. Return map of size {} ", n_masks, mask_name, mask_map.size());
  return mask_map;
}

#endif
