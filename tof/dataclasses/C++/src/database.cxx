#include <format>
#include <iostream>
#include "database.h"


using namespace sqlite_orm;
//using namespace Gaps;

std::string Gaps::TofPaddle::to_string() const {
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
  repr += std::format("\n  dsi                 : {} ", dsi              );         
  repr += std::format("\n  j_rb                : {} ", j_rb             );         
  repr += std::format("\n  j_ltb               : {} ", j_ltb            );         
  repr += std::format("\n  height              : {} ", height           );         
  repr += std::format("\n  width               : {} ", width            );         
  repr += std::format("\n  length              : {} ", length           );         
  repr += std::format("\n  global_pos_x_l0     : {} ", global_pos_x_l0  );         
  repr += std::format("\n  global_pos_y_l0     : {} ", global_pos_y_l0  );         
  repr += std::format("\n  global_pos_z_l0     : {} ", global_pos_z_l0  );         
  repr += std::format("\n  global_pos_x_l0_A   : {} ", global_pos_x_l0_A);          
  repr += std::format("\n  global_pos_y_l0_A   : {} ", global_pos_y_l0_A);          
  repr += std::format("\n  global_pos_z_l0_A   : {}>", global_pos_z_l0_A);         
  return repr;
}

std::map<u8, Gaps::TofPaddle> Gaps::get_tofpaddles(std::string dbname) {
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
      make_column("global_pos_x_l0"  , &Gaps::TofPaddle::global_pos_x_l0  ),         
      make_column("global_pos_y_l0"  , &Gaps::TofPaddle::global_pos_y_l0  ),         
      make_column("global_pos_z_l0"  , &Gaps::TofPaddle::global_pos_z_l0  ),         
      make_column("global_pos_x_l0_A", &Gaps::TofPaddle::global_pos_x_l0_A),          
      make_column("global_pos_y_l0_A", &Gaps::TofPaddle::global_pos_y_l0_A),          
      make_column("global_pos_z_l0_A", &Gaps::TofPaddle::global_pos_z_l0_A)));          
  
  auto paddles = storage.get_all<Gaps::TofPaddle>();
  auto paddle_map = std::map<u8, Gaps::TofPaddle>();
  for (auto p : paddles) {
    paddle_map.insert({p.paddle_id, p});
  }  
  return paddle_map;
}

std::ostream& operator<<(std::ostream& os, const Gaps::TofPaddle& tp) {
  os << tp.to_string();
  return os;
}

