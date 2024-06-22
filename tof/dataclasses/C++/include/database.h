#ifndef GOS_DB_HEADER_INCLUDED
#define GOS_DB_HEADER_INCLUDED

#include "tof_typedefs.h"
#include "sqlite_orm.h"

#include <map>

namespace Gaps {

  struct TofPaddle {
    u8  paddle_id         ; 
    u64 volume_id         ; 
    u8  panel_id          ; 
    u8  mtb_link_id       ; 
    u8  rb_id             ; 
    u8  rb_chA            ; 
    u8  rb_chB            ; 
    /// LTB ID equals RAT ID - for confusion, there is another LTB id, which is 
    /// only hardware
    u8  ltb_id            ;         
    u8  ltb_chA           ;         
    u8  ltb_chB           ;         
    u8  pb_id             ;         
    u8  pb_chA            ;         
    u8  pb_chB            ;         
    f32 cable_len         ;         
    u8  dsi               ;         
    u8  j_rb              ;         
    u8  j_ltb             ;         
    f32 height            ;         
    f32 width             ;         
    f32 length            ;         
    f32 global_pos_x_l0   ;         
    f32 global_pos_y_l0   ;         
    f32 global_pos_z_l0   ;         
    f32 global_pos_x_l0_A ;         
    f32 global_pos_y_l0_A ;         
    f32 global_pos_z_l0_A ;         
  
    std::string to_string() const;
  };



  /// Get a paddle from the database
  std::map<u8, TofPaddle> get_tofpaddles(std::string dbname);
}

std::ostream& operator<<(std::ostream& os, const Gaps::TofPaddle& paddle);


#endif
