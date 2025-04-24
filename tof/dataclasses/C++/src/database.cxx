#ifdef BUILD_CXXDB
#include <cmath>
#include <cstdlib>
#include <format>
#include <iostream>

#include "spdlog/spdlog.h"
#include "database.h"

using namespace sqlite_orm;

auto Gaps::TofPaddle::to_string() const -> std::string {
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

auto Gaps::TrackerStrip::to_string() const -> std::string {
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

auto Gaps::TrackerStrip::create_id() const -> u32 {
  return Gaps::TrackerStrip::create_id(layer, row, module, channel);
}; 

auto Gaps::TrackerStrip::create_id(u32 layer, u32 row, u32 module, u32 channel) -> u32 {
  return channel + module*100 + row*10000 + layer*100000;
};

auto Gaps::get_tofpaddles() -> std::map<u8, Gaps::TofPaddle> {
  // FIXME - find a better name for the database variable
  //         env name
  auto paddle_map = std::map<u8, Gaps::TofPaddle>();
  auto db_path = std::getenv("DATABASE_URL");
  if (db_path == nullptr) {
    spdlog::error("Unable to retrieve database! The DATABASE_URL shell variable is not set. Did you load the setup-env.sh shell?");
    return paddle_map;
  } 
  std::string dbname(db_path);
  auto storage = make_storage(dbname,
    make_table("tof_db_paddle",
      make_column("paddle_id"        , &Gaps::TofPaddle::paddle_id, primary_key()        ),
      make_column("volume_id"        , &Gaps::TofPaddle::volume_id        ),  
      make_column("panel_id"         , &Gaps::TofPaddle::panel_id         ), 
      make_column("mtb_link_id"      , &Gaps::TofPaddle::mtb_link_id      ), 
      make_column("rb_id"            , &Gaps::TofPaddle::rb_id            ), 
      make_column("rb_chA"           , &Gaps::TofPaddle::rb_chA           ), 
      make_column("rb_chB"           , &Gaps::TofPaddle::rb_chB           ), 
      make_column("ltb_id"           , &Gaps::TofPaddle::ltb_id           ),         
      make_column("ltb_chA"          , &Gaps::TofPaddle::ltb_chA          ),         
      make_column("ltb_chB"          , &Gaps::TofPaddle::ltb_chB          ),         
      make_column("pb_id"            , &Gaps::TofPaddle::pb_id            ),         
      make_column("pb_chA"           , &Gaps::TofPaddle::pb_chA           ),         
      make_column("pb_chB"           , &Gaps::TofPaddle::pb_chB           ),         
      make_column("cable_len"        , &Gaps::TofPaddle::cable_len        ),         
      make_column("dsi"              , &Gaps::TofPaddle::dsi              ),         
      make_column("j_rb"             , &Gaps::TofPaddle::j_rb             ),         
      make_column("j_ltb"            , &Gaps::TofPaddle::j_ltb            ),         
      make_column("height"           , &Gaps::TofPaddle::height           ),         
      make_column("width"            , &Gaps::TofPaddle::width            ),         
      make_column("length"           , &Gaps::TofPaddle::length           ),         
      make_column("normal_x"         , &Gaps::TofPaddle::normal_x         ),         
      make_column("normal_y"         , &Gaps::TofPaddle::normal_y         ),         
      make_column("normal_z"         , &Gaps::TofPaddle::normal_z         ),         
      make_column("global_pos_x_l0"  , &Gaps::TofPaddle::global_pos_x_l0  ),         
      make_column("global_pos_y_l0"  , &Gaps::TofPaddle::global_pos_y_l0  ),         
      make_column("global_pos_z_l0"  , &Gaps::TofPaddle::global_pos_z_l0  ),         
      make_column("global_pos_x_l0_A", &Gaps::TofPaddle::global_pos_x_l0_A),          
      make_column("global_pos_y_l0_A", &Gaps::TofPaddle::global_pos_y_l0_A),          
      make_column("global_pos_z_l0_A", &Gaps::TofPaddle::global_pos_z_l0_A),          
      make_column("global_pos_x_l0_B", &Gaps::TofPaddle::global_pos_x_l0_B),          
      make_column("global_pos_y_l0_B", &Gaps::TofPaddle::global_pos_y_l0_B),          
      make_column("global_pos_z_l0_B", &Gaps::TofPaddle::global_pos_z_l0_B),          
      make_column("coax_cable_time"  , &Gaps::TofPaddle::coax_cable_time),          
      make_column("harting_cable_time", &Gaps::TofPaddle::harting_cable_time)));          
  auto paddles = storage.get_all<Gaps::TofPaddle>();
  for (auto p : paddles) {
    paddle_map.insert({p.paddle_id, p});
  }  
  return paddle_map;
}

auto Gaps::get_rb_id_paddles() -> RbIdChannelPaddleIdMap {
  RbIdChannelPaddleIdMap map;
  for (u8 rb_id=1; rb_id<50; rb_id++) {
    auto ch_map = std::map<u8, std::tuple<u8, TofPaddleEnd>>();
    map.insert(std::make_pair(rb_id, ch_map));
  }
  auto paddles = get_tofpaddles();
  for (auto const &pdl : paddles) {
    map[pdl.second.rb_id].insert(std::make_pair(pdl.second.rb_chA, std::make_tuple(pdl.second.paddle_id, Gaps::TofPaddleEnd::A)));
    map[pdl.second.rb_id].insert(std::make_pair(pdl.second.rb_chB, std::make_tuple(pdl.second.paddle_id, Gaps::TofPaddleEnd::B)));
  }
  return map;
};

auto Gaps::get_dsi_j_paddles() -> DsiJChnPaddleIdMap {
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


auto Gaps::get_trackerstrips() -> std::map<u32, Gaps::TrackerStrip> {
  // FIXME - find a better name for the database variable
  //         env name
  auto strip_map = std::map<u32, Gaps::TrackerStrip>();
  auto db_path = std::getenv("DATABASE_URL");
  if (db_path == nullptr) {
    spdlog::error("Unable to retrieve database! The DATABASE_URL shell variable is not set. Did you load the setup-env.sh shell?");
    return strip_map;
  } 
  std::string dbname(db_path);
  auto storage = make_storage(dbname,
    make_table("tof_db_trackerstrip",
      make_column("strip_id"           , &Gaps::TrackerStrip::strip_id, primary_key()),
      make_column("layer"              , &Gaps::TrackerStrip::layer), 
      make_column("row"                , &Gaps::TrackerStrip::row), 
      make_column("module"             , &Gaps::TrackerStrip::module), 
      make_column("channel"            , &Gaps::TrackerStrip::channel),  
      make_column("global_pos_x_l0"    , &Gaps::TrackerStrip::global_pos_x_l0),
      make_column("global_pos_y_l0"    , &Gaps::TrackerStrip::global_pos_y_l0),
      make_column("global_pos_z_l0"    , &Gaps::TrackerStrip::global_pos_z_l0),
      make_column("global_pos_x_det_l0", &Gaps::TrackerStrip::global_pos_x_det_l0),
      make_column("global_pos_y_det_l0", &Gaps::TrackerStrip::global_pos_y_det_l0),
      make_column("global_pos_z_det_l0", &Gaps::TrackerStrip::global_pos_z_det_l0),
      make_column("principal_x"        , &Gaps::TrackerStrip::principal_x),
      make_column("principal_y"        , &Gaps::TrackerStrip::principal_y),
      make_column("principal_z"        , &Gaps::TrackerStrip::principal_z),
      make_column("volume_id"          , &Gaps::TrackerStrip::volume_id)));  
  
  auto strips = storage.get_all<Gaps::TrackerStrip>();
  for (auto const &strip : strips) {
    strip_map.insert({strip.strip_id, strip});
  }  
  return strip_map;
}

auto Gaps::TofPaddle::get_principal() const -> Vec<f32> {
  Vec<f32> pr(3,0);
  pr[0] = global_pos_x_l0_A - global_pos_x_l0;
  pr[1] = global_pos_y_l0_A - global_pos_y_l0;
  pr[2] = global_pos_z_l0_A - global_pos_z_l0;
  f32 length = std::sqrt((std::pow(pr[0],2) + std::pow(pr[1],2) + std::pow(pr[2],2)));
  if (length > 0) {
    pr = {pr[0]/length, pr[1]/length, pr[2]/length};
  } else {
    pr = {0,0,0};
  }
  return pr; 
}
  
auto Gaps::TrackerStripMask::to_string() const -> std::string {
  std::string repr = "<TrackerStripMask:";
  repr += std::format("\n strip id        : {}",  strip_id );
  repr += std::format("\n volume id       : {}",  volume_id);
  repr += std::format("\n Timestamp (UTC) : {}",  utc_timestamp);
  repr += std::format("\n mask name       : {}",  mask_name); 
  repr += std::format("\n active          : {}>", active    ); 
  return repr;
}

auto Gaps::get_trackerstripmasks(std::string mask_name) -> Gaps::TrkStripMaskMap {
  Gaps::TrkStripMaskMap mask_map;
  auto db_path = std::getenv("DATABASE_URL");
  if (db_path == nullptr) {
    spdlog::error("Unable to retrieve database! The DATABASE_URL shell variable is not set. Did you load the setup-env.sh shell?");
    return mask_map;
  } 
  std::string dbname(db_path);
  auto storage = make_storage(dbname,
    make_table("tof_db_trackerstripmask",
      make_column("strip_id"             , &Gaps::TrackerStripMask::strip_id, primary_key()),
      make_column("volume_id"            , &Gaps::TrackerStripMask::volume_id),  
      make_column("utc_timestamp"        , &Gaps::TrackerStripMask::utc_timestamp),
      make_column("mask_name"            , &Gaps::TrackerStripMask::mask_name),
      make_column("active"               , &Gaps::TrackerStripMask::active)));  
  
  auto masks = storage.get_all<Gaps::TrackerStripMask>();
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

auto Gaps::TrackerStripPedestal::to_string() const -> std::string {
  std::string repr = "<TrackerStripPedestal:";
  repr += std::format("\n strip id        : {}",  strip_id );
  repr += std::format("\n volume id       : {}",  volume_id);
  repr += std::format("\n Timestamp (UTC) : {}",  utc_timestamp);
  repr += std::format("\n Pedestal Mean   : {}",  pedestal_mean);
  repr += std::format("\n Pedestal Sigma  : {}",  pedestal_sigma);
  repr += std::format("\n IsMeanValue     : {}",  is_mean_value);
  return repr;
}

auto Gaps::get_trackerstrippedestals() -> Gaps::TrkStripPedMap {
  Gaps::TrkStripPedMap ped_map;
  auto db_path = std::getenv("DATABASE_URL");
  if (db_path == nullptr) {
    spdlog::error("Unable to retrieve database! The DATABASE_URL shell variable is not set. Did you load the setup-env.sh shell?");
    return ped_map;
  } 
  std::string dbname(db_path);
  auto storage = make_storage(dbname,
    make_table("tof_db_trackerstrippedestal",
      make_column("strip_id"             , &Gaps::TrackerStripPedestal::strip_id, primary_key()),
      make_column("volume_id"            , &Gaps::TrackerStripPedestal::volume_id),  
      make_column("utc_timestamp"        , &Gaps::TrackerStripPedestal::utc_timestamp),
      make_column("pedestal_mean"        , &Gaps::TrackerStripPedestal::pedestal_mean),
      make_column("pedestal_sigma"       , &Gaps::TrackerStripPedestal::pedestal_sigma),
      make_column("is_mean_value"        , &Gaps::TrackerStripPedestal::is_mean_value)));  
  
  auto pedestals = storage.get_all<Gaps::TrackerStripPedestal>();
  for (auto const &m : pedestals) {
    ped_map.insert({m.strip_id, m});
  }  
  return ped_map;
}

std::ostream& operator<<(std::ostream& os, const Gaps::TofPaddle& tp) {
  os << tp.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const Gaps::TrackerStrip& ts) {
  os << ts.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const Gaps::TrackerStripMask& ts) {
  os << ts.to_string();
  return os;
}

std::ostream& operator<<(std::ostream& os, const Gaps::TrackerStripPedestal& ts) {
  os << ts.to_string();
  return os;
}

#endif
