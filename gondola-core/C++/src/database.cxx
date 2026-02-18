#ifdef BUILD_CXX_DB
#include <cmath>
#include <cstdlib>
#include <format>
#include <iostream>

#include "spdlog/spdlog.h"
#include "database.h"

using namespace sqlite_orm;
namespace g = gondola;

auto g::TofPaddle::to_string() const -> std::string {
  auto repr = std::string("<TofPaddle: ");
  repr += std::format("\n  paddle_id           : {} ", paddle_id        );
  repr += std::format("\n  volume_id           : {} ", volume_id        );  
  repr += std::format("\n  panel_id            : {} ", panel_id         ); 
  repr += std::format("\n  mtb_link_id         : {} ", mtb_link_id      ); 
  repr += std::format("\n  rb_id               : {} ", rb_id            ); 
  repr += std::format("\n  rb_chA              : {} ", rb_chA           ); 
  repr += std::format("\n  rb_chB              : {} ", rb_chB           ); 
  repr += std::format("\n  ltb_id              : {} ", ltb_id           );         
  repr += std::format("\n  ltb_chA             : {} ", ltb_chA          );         
  repr += std::format("\n  ltb_chB             : {} ", ltb_chB          );         
  repr += std::format("\n  pb_id               : {} ", pb_id            );         
  repr += std::format("\n  pb_chA              : {} ", pb_chA           );         
  repr += std::format("\n  pb_chB              : {} ", pb_chB           );         
  repr += std::format("\n  cable_len           : {} ", cable_len        );         
  repr += std::format("\n  coax cbl time [ns]  : {} ", coax_cable_time  );         
  repr += std::format("\n  hart. cbl time [ns] : {} ", harting_cable_time);         
  repr += std::format("\n  dsi                 : {} ", dsi              );         
  repr += std::format("\n  j_rb                : {} ", j_rb             );         
  repr += std::format("\n  j_ltb               : {} ", j_ltb            );         
  repr += std::format("\n  height              : {} ", height           );         
  repr += std::format("\n  width               : {} ", width            );         
  repr += std::format("\n  length              : {} ", length           );         
  repr += std::format("\n  normal_x            : {} ", normal_x         );         
  repr += std::format("\n  normal_y            : {} ", normal_y         );         
  repr += std::format("\n  normal_z            : {} ", normal_z         );         
  repr += std::format("\n  global_pos_x_l0     : {} ", global_pos_x_l0  );         
  repr += std::format("\n  global_pos_y_l0     : {} ", global_pos_y_l0  );         
  repr += std::format("\n  global_pos_z_l0     : {} ", global_pos_z_l0  );         
  repr += std::format("\n  global_pos_x_l0_A   : {} ", global_pos_x_l0_A);          
  repr += std::format("\n  global_pos_y_l0_A   : {} ", global_pos_y_l0_A);          
  repr += std::format("\n  global_pos_z_l0_A   : {}>", global_pos_z_l0_A);         
  repr += std::format("\n  global_pos_x_l0_B   : {} ", global_pos_x_l0_B);          
  repr += std::format("\n  global_pos_y_l0_B   : {} ", global_pos_y_l0_B);          
  repr += std::format("\n  global_pos_z_l0_B   : {}>", global_pos_z_l0_B);         
  return repr;
}

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

auto g::get_tofpaddles() -> std::map<u8, g::TofPaddle> {
  // FIXME - find a better name for the database variable
  //         env name
  auto paddle_map = std::map<u8, g::TofPaddle>();
  auto db_path = std::getenv("GONDOLA_DB_URL");
  if (db_path == nullptr) {
    spdlog::error("Unable to retrieve database! The GONDOLA_DB_URL shell variable is not set. Did you load the setup-env.sh shell?");
    return paddle_map;
  } 
  std::string dbname(db_path);
  auto storage = make_storage(dbname,
    make_table("tof_db_paddle",
      make_column("paddle_id"        , &g::TofPaddle::paddle_id, primary_key()        ),
      make_column("volume_id"        , &g::TofPaddle::volume_id        ),  
      make_column("panel_id"         , &g::TofPaddle::panel_id         ), 
      make_column("mtb_link_id"      , &g::TofPaddle::mtb_link_id      ), 
      make_column("rb_id"            , &g::TofPaddle::rb_id            ), 
      make_column("rb_chA"           , &g::TofPaddle::rb_chA           ), 
      make_column("rb_chB"           , &g::TofPaddle::rb_chB           ), 
      make_column("ltb_id"           , &g::TofPaddle::ltb_id           ),         
      make_column("ltb_chA"          , &g::TofPaddle::ltb_chA          ),         
      make_column("ltb_chB"          , &g::TofPaddle::ltb_chB          ),         
      make_column("pb_id"            , &g::TofPaddle::pb_id            ),         
      make_column("pb_chA"           , &g::TofPaddle::pb_chA           ),         
      make_column("pb_chB"           , &g::TofPaddle::pb_chB           ),         
      make_column("cable_len"        , &g::TofPaddle::cable_len        ),         
      make_column("dsi"              , &g::TofPaddle::dsi              ),         
      make_column("j_rb"             , &g::TofPaddle::j_rb             ),         
      make_column("j_ltb"            , &g::TofPaddle::j_ltb            ),         
      make_column("height"           , &g::TofPaddle::height           ),         
      make_column("width"            , &g::TofPaddle::width            ),         
      make_column("length"           , &g::TofPaddle::length           ),         
      make_column("normal_x"         , &g::TofPaddle::normal_x         ),         
      make_column("normal_y"         , &g::TofPaddle::normal_y         ),         
      make_column("normal_z"         , &g::TofPaddle::normal_z         ),         
      make_column("global_pos_x_l0"  , &g::TofPaddle::global_pos_x_l0  ),         
      make_column("global_pos_y_l0"  , &g::TofPaddle::global_pos_y_l0  ),         
      make_column("global_pos_z_l0"  , &g::TofPaddle::global_pos_z_l0  ),         
      make_column("global_pos_x_l0_A", &g::TofPaddle::global_pos_x_l0_A),          
      make_column("global_pos_y_l0_A", &g::TofPaddle::global_pos_y_l0_A),          
      make_column("global_pos_z_l0_A", &g::TofPaddle::global_pos_z_l0_A),          
      make_column("global_pos_x_l0_B", &g::TofPaddle::global_pos_x_l0_B),          
      make_column("global_pos_y_l0_B", &g::TofPaddle::global_pos_y_l0_B),          
      make_column("global_pos_z_l0_B", &g::TofPaddle::global_pos_z_l0_B),          
      make_column("coax_cable_time"  , &g::TofPaddle::coax_cable_time),          
      make_column("harting_cable_time", &g::TofPaddle::harting_cable_time)));          
  auto paddles = storage.get_all<g::TofPaddle>();
  for (auto p : paddles) {
    paddle_map.insert({p.paddle_id, p});
  }  
  return paddle_map;
}

auto g::get_rb_id_paddles() -> RbIdChannelPaddleIdMap {
  RbIdChannelPaddleIdMap map;
  for (u8 rb_id=1; rb_id<50; rb_id++) {
    auto ch_map = std::map<u8, std::tuple<u8, TofPaddleEnd>>();
    map.insert(std::make_pair(rb_id, ch_map));
  }
  auto paddles = get_tofpaddles();
  for (auto const &pdl : paddles) {
    map[pdl.second.rb_id].insert(std::make_pair(pdl.second.rb_chA, std::make_tuple(pdl.second.paddle_id, g::TofPaddleEnd::A)));
    map[pdl.second.rb_id].insert(std::make_pair(pdl.second.rb_chB, std::make_tuple(pdl.second.paddle_id, g::TofPaddleEnd::B)));
  }
  return map;
};

auto g::get_dsi_j_paddles() -> DsiJChnPaddleIdMap {
  DsiJChnPaddleIdMap map;
  for (u8 dsi=1; dsi<6; dsi++) {
    //auto j_map = TofPaddleMap();
    map.insert(std::make_pair(dsi, std::map<u8, std::map<u8, u8>>()));
    for (u8 j=1; j<6; j++) {
      map[dsi].insert(std::make_pair(j, std::map<u8, u8>()));
      for (u8 ch=1; ch<17; ch++) {
        map[dsi][j].insert(std::make_pair(ch, 0));
      }
    }
  }
  auto paddles = get_tofpaddles();
  for (auto const &pdl : paddles) {
    map[pdl.second.dsi][pdl.second.j_ltb][pdl.second.ltb_chA] = pdl.second.paddle_id;
    map[pdl.second.dsi][pdl.second.j_ltb][pdl.second.ltb_chB] = pdl.second.paddle_id;
  }
  return map;
};


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

auto g::TofPaddle::get_principal() const -> Vec<f32> {
  Vec<f32> pr(3,0);
  pr[0] = global_pos_x_l0_B - global_pos_x_l0_A;
  pr[1] = global_pos_y_l0_B - global_pos_y_l0_A;
  pr[2] = global_pos_z_l0_B - global_pos_z_l0_A;
  f32 length = std::sqrt((std::pow(pr[0],2) + std::pow(pr[1],2) + std::pow(pr[2],2)));
  if (length > 0) {
    pr = {pr[0]/length, pr[1]/length, pr[2]/length};
  } else {
    pr = {0,0,0};
  }
  return pr; 
}
  
auto g::TrackerStripMask::to_string() const -> std::string {
  std::string repr = "<TrackerStripMask:";
  repr += std::format("\n strip id        : {}",  strip_id );
  repr += std::format("\n volume id       : {}",  volume_id);
  repr += std::format("\n Timestamp (UTC) : {}",  utc_timestamp);
  repr += std::format("\n mask name       : {}",  mask_name); 
  repr += std::format("\n active          : {}>", active    ); 
  return repr;
}

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
      make_column("strip_id"             , &g::TrackerStripMask::strip_id, primary_key()),
      make_column("volume_id"            , &g::TrackerStripMask::volume_id),  
      make_column("utc_timestamp"        , &g::TrackerStripMask::utc_timestamp),
      make_column("mask_name"            , &g::TrackerStripMask::mask_name),
      make_column("active"               , &g::TrackerStripMask::active)));  
  
  auto masks = storage.get_all<g::TrackerStripMask>();
  for (auto const &m : masks) {
    if (mask_name != "") {
      if (m.mask_name != mask_name) {
        continue;
      }
    }
    mask_map.insert({m.strip_id, m.active});
  }  
  return mask_map;
}

auto g::TrackerStripPedestal::to_string() const -> std::string {
  std::string repr = "<TrackerStripPedestal:";
  repr += std::format("\n strip id        : {}",  strip_id );
  repr += std::format("\n volume id       : {}",  volume_id);
  repr += std::format("\n Timestamp (UTC) : {}",  utc_timestamp);
  repr += std::format("\n Pedestal Mean   : {}",  pedestal_mean);
  repr += std::format("\n Pedestal Sigma  : {}",  pedestal_sigma);
  repr += std::format("\n IsMeanValue     : {}",  is_mean_value);
  return repr;
}

auto g::get_trackerstrippedestals() -> g::TrkStripPedMap {
  g::TrkStripPedMap ped_map;
  auto db_path = std::getenv("GONDOLA_DB_URL");
  if (db_path == nullptr) {
    spdlog::error("Unable to retrieve database! The GONDOLA_DB_URL shell variable is not set. Did you load the setup-env.sh shell?");
    return ped_map;
  } 
  std::string dbname(db_path);
  auto storage = make_storage(dbname,
    make_table("tof_db_trackerstrippedestal",
      make_column("strip_id"             , &g::TrackerStripPedestal::strip_id, primary_key()),
      make_column("volume_id"            , &g::TrackerStripPedestal::volume_id),  
      make_column("utc_timestamp"        , &g::TrackerStripPedestal::utc_timestamp),
      make_column("pedestal_mean"        , &g::TrackerStripPedestal::pedestal_mean),
      make_column("pedestal_sigma"       , &g::TrackerStripPedestal::pedestal_sigma),
      make_column("is_mean_value"        , &g::TrackerStripPedestal::is_mean_value)));  
  
  auto pedestals = storage.get_all<g::TrackerStripPedestal>();
  for (auto const &m : pedestals) {
    ped_map.insert({m.strip_id, m});
  }  
  return ped_map;
}

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

auto g::get_hid_vid_map() -> HashMap<u32, u32> {
  auto map = HashMap<u32, u32>();
  auto paddles = g::get_tofpaddles();
  auto strips  = g::get_trackerstrips();
  for (const auto& p : paddles) {
    map.insert(std::make_pair(p.first, p.second.volume_id));  
  }
  for (const auto& s : strips) {
    map.insert(std::make_pair(s.first, s.second.volume_id));  
  }
  return map;
}

auto g::get_vid_hid_map() -> HashMap<u32, u32> {
  auto map = HashMap<u32, u32>();
  auto paddles = g::get_tofpaddles();
  auto strips  = g::get_trackerstrips();
  for (const auto& p : paddles) {
    map.insert(std::make_pair(p.second.volume_id, p.first));  
  }
  for (const auto& s : strips) {
    map.insert(std::make_pair(s.second.volume_id, s.first));   
  }
  return map;
}

//--------------------------------------------------------------

auto g::TofPaddleTimingConstant::to_string() const -> std::string {
  std::string repr = "<TofPaddleTimingConstant:";
  repr += std::format("\n paddle id             : {}",  paddle_id );
  repr += std::format("\n volume id             : {}",  volume_id);
  repr += std::format("\n Timestamp (UTC) start : {}",  utc_timestamp_start);
  repr += std::format("\n Timestamp (UTC) stop  : {}",  utc_timestamp_stop);
  repr += std::format("\n name                  : {}",  name); 
  repr += std::format("\n version               : {}",  version); 
  repr += std::format("\n timing_constant       : {}>", timing_constant    ); 
  return repr;
}

//--------------------------------------------------------------

auto g::get_tofpaddletimingconstants(std::string name) -> g::TofPaddleTimingConstantMap {
  g::TofPaddleTimingConstantMap tmg_map;
  auto db_path = std::getenv("GONDOLA_DB_URL");
  if (db_path == nullptr) {
    spdlog::error("Unable to retrieve database! The GONDOLA_DB_URL shell variable is not set. Did you load the setup-env.sh shell?");
    return tmg_map;
  } 
  std::string dbname(db_path);
  auto storage = make_storage(dbname,
    make_table("tof_db_tofpaddletimingconstant",
      make_column("data_id"              , &g::TofPaddleTimingConstant::data_id, primary_key()),
      make_column("paddle_id"            , &g::TofPaddleTimingConstant::paddle_id),
      make_column("volume_id"            , &g::TofPaddleTimingConstant::volume_id),  
      make_column("utc_timestamp_start"  , &g::TofPaddleTimingConstant::utc_timestamp_start),
      make_column("utc_timestamp_stop"   , &g::TofPaddleTimingConstant::utc_timestamp_stop),
      make_column("name"                 , &g::TofPaddleTimingConstant::name),
      make_column("version"              , &g::TofPaddleTimingConstant::version),
      make_column("timing_constant"      , &g::TofPaddleTimingConstant::timing_constant)));  
  
  auto tcs = storage.get_all<g::TofPaddleTimingConstant>();
  for (auto const &tc : tcs) {
    if (name != "") {
      if (tc.name != name) {
        continue;
      }
    }
    tmg_map.insert({tc.paddle_id, tc.timing_constant});
  }  
  return tmg_map;
}

//--------------------------------------------------------------

std::ostream& operator<<(std::ostream& os, const g::TofPaddle& tp) {
  os << tp.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const g::TrackerStrip& ts) {
  os << ts.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const g::TrackerStripMask& ts) {
  os << ts.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const g::TrackerStripPedestal& ts) {
  os << ts.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const g::TofPaddleTimingConstant& tc) {
  os << tc.to_string();
  return os;
}

#endif
