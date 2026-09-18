#ifdef BUILD_CXX_DB
#include <format>
#include <iostream>

#include "sqlite_orm.h"
#include "spdlog/spdlog.h"
#include "database/tracker_strip.hpp"

using namespace sqlite_orm;
namespace g = gondola;

auto g::TrackerStrip::to_string() const -> std::string {
  auto repr = std::string("<TrackerStrip: ");
  repr += std::format("\n  StripID            : {}", strip_id   );
  repr += std::format("\n  VolumeID           : {}", volume_id  );  
  repr += std::format("\n  Row                : {}", row);                    
  repr += std::format("\n  Module             : {}", module);                   
  repr += std::format("\n  Channel            : {}", channel);                  
  repr += std::format("\n  Volume ID          : {}", volume_id);  
  repr += std::format("\n  -- str pos. (from sim) --");
  repr += std::format("\n  X: {} Y: {} Z: {}", global_pos_x_l0, global_pos_y_l0, global_pos_z_l0);                 
  repr += std::format("\n  -- det pos. (from sim) --");
  repr += std::format("\n  X: {} Y: {} Z: {}", global_pos_x_det_l0, global_pos_y_det_l0, global_pos_z_det_l0);               
  repr += std::format("\n  -- principal dir (from sim) --");
  repr += std::format("\n  X: {} Y: {} Z: {}>", principal_x, principal_y, principal_z);                 
  return repr;
}

auto g::TrackerStrip::create_id() const -> u32 {
  return g::TrackerStrip::create_id(layer, row, module, channel);
}; 

auto g::TrackerStrip::create_id(u32 layer, u32 row, u32 module, u32 channel) -> u32 {
  return channel + module*100 + row*10000 + layer*100000;
};

//------------------------------------------------------------------

auto g::get_trackerstrips() -> std::map<u32, g::TrackerStrip> {
  // FIXME - find a better name for the database variable
  //         env name
  auto strip_map = std::map<u32, g::TrackerStrip>();
  auto db_path = std::getenv("GONDOLA_DB_URL");
  if (db_path == nullptr) {
    spdlog::error("Unable to retrieve database! The GONDOLA_DB_URL shell variable is not set. Did you load the setup-env.sh shell?");
    return strip_map;
  } 
  std::string dbname(db_path);
  auto storage = make_storage(dbname,
    make_table("tof_db_trackerstrip",
      make_column("strip_id"           , &g::TrackerStrip::strip_id, primary_key()),
      make_column("layer"              , &g::TrackerStrip::layer), 
      make_column("row"                , &g::TrackerStrip::row), 
      make_column("module"             , &g::TrackerStrip::module), 
      make_column("channel"            , &g::TrackerStrip::channel),  
      make_column("global_pos_x_l0"    , &g::TrackerStrip::global_pos_x_l0),
      make_column("global_pos_y_l0"    , &g::TrackerStrip::global_pos_y_l0),
      make_column("global_pos_z_l0"    , &g::TrackerStrip::global_pos_z_l0),
      make_column("global_pos_x_det_l0", &g::TrackerStrip::global_pos_x_det_l0),
      make_column("global_pos_y_det_l0", &g::TrackerStrip::global_pos_y_det_l0),
      make_column("global_pos_z_det_l0", &g::TrackerStrip::global_pos_z_det_l0),
      make_column("principal_x"        , &g::TrackerStrip::principal_x),
      make_column("principal_y"        , &g::TrackerStrip::principal_y),
      make_column("principal_z"        , &g::TrackerStrip::principal_z),
      make_column("volume_id"          , &g::TrackerStrip::volume_id)));  
  
  auto strips = storage.get_all<g::TrackerStrip>();
  for (auto const &strip : strips) {
    strip_map.insert({strip.strip_id, strip});
  }  
  return strip_map;
}

//------------------------------------------------------------------

auto g::get_module_position(u8 layer, u8 row, u8 mod, const g::TrkStripMap& strips) -> Vec<f32> {
  auto det_0  = strips.at(TrackerStrip::create_id(layer, row, mod, 0));
  auto det_1  = strips.at(TrackerStrip::create_id(layer, row, mod, 8));
  auto det_2  = strips.at(TrackerStrip::create_id(layer, row, mod, 16));
  auto det_3  = strips.at(TrackerStrip::create_id(layer, row, mod, 24));
  auto mod_x  = det_0.global_pos_x_det_l0 + det_1.global_pos_x_det_l0
              + det_2.global_pos_x_det_l0 + det_3.global_pos_x_det_l0;
  mod_x = mod_x / 4;
  auto mod_y  = det_0.global_pos_y_det_l0 + det_1.global_pos_y_det_l0
              + det_2.global_pos_y_det_l0 + det_3.global_pos_y_det_l0;
  mod_y = mod_y / 4;
  auto mod_z  = det_0.global_pos_z_det_l0 + det_1.global_pos_z_det_l0
              + det_2.global_pos_z_det_l0 + det_3.global_pos_z_det_l0;
  mod_z = mod_z / 4;
  Vec<f32> result = {mod_x, mod_y, mod_z};
  //std::cout << std::format("X {} Y {} Z {}", mod_x, mod_y, mod_z) << std::endl;
  return result;
}

//------------------------------------------------------------------

namespace gondola {
  std::ostream& operator<<(std::ostream& os, const g::TrackerStrip& ts) {
    os << ts.to_string();
    return os;
  }
}

#endif 
