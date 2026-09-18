#ifdef BUILD_CXX_DB

#include <format>
#include "spdlog/spdlog.h"
#include "sqlite_orm.h"

#include "database/tracker_strip_gain.hpp"
    
namespace g = gondola;
using namespace sqlite_orm;
    
auto g::TrackerStripGain::to_string() const -> std::string {
  std::string repr = "<TrackerStripGain:";
  repr += std::format("\n strip id              : {}",  strip_id );
  repr += std::format("\n volume id             : {}",  volume_id);
  repr += std::format("\n Timestamp Start (UTC) : {}",  utc_timestamp_start);
  repr += std::format("\n Timestamp Stop  (UTC) : {}",  utc_timestamp_start);

  repr += std::format("\n name                  : {}",  name); 
  repr += std::format("\n gain                  : {}",  gain); 
  repr += std::format("\n gain_is_men           : {}>", gain_is_mean); 
  return repr;
}

auto g::get_trackerstripgains(std::string name) -> g::TrkStripGainMap {
  g::TrkStripGainMap gain_map;
  auto db_path = std::getenv("GONDOLA_DB_URL");
  if (db_path == nullptr) {
    spdlog::error("Unable to retrieve database! The GONDOLA_DB_URL shell variable is not set. Did you load the setup-env.sh shell?");
    return gain_map;
  } 
  std::string dbname(db_path);
  auto storage = make_storage(dbname,
    make_table("tof_db_trackerstripgain",
      make_column("data_id"              , &g::TrackerStripGain::data_id, primary_key()),
      make_column("strip_id"             , &g::TrackerStripGain::strip_id),
      make_column("volume_id"            , &g::TrackerStripGain::volume_id),  
      make_column("utc_timestamp_start"  , &g::TrackerStripGain::utc_timestamp_start),
      make_column("utc_timestamp_stop"   , &g::TrackerStripGain::utc_timestamp_stop),
      make_column("name"                 , &g::TrackerStripGain::name),
      make_column("gain"                 , &g::TrackerStripGain::gain),
      make_column("gain_is_mean"         , &g::TrackerStripGain::gain_is_mean)));  
  
  auto gains = storage.get_all<g::TrackerStripGain>();
  for (auto const &g : gains) {
    if (name != "") {
      if (g.name != name) {
        continue;
      }
    }
    gain_map.insert({g.strip_id, g.gain});
  }  
  return gain_map;
}

namespace gondola {
  std::ostream& operator<<(std::ostream& os, const TrackerStripGain& ts_gain) {
    return os << ts_gain.to_string();
  }
}

#endif
