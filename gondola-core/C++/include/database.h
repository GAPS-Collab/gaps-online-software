#ifndef GOS_DB_HEADER_INCLUDED
#define GOS_DB_HEADER_INCLUDED

#include "tof_typedefs.h"
#include "sqlite_orm.h"

#include <map>

namespace Gaps {
  
  enum class TofPaddleEnd : i16 {
    Unknown                = 0,
    A                      = -1,
    B                      = 1,
  };


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
    f32 normal_x          ;
    f32 normal_y          ;
    f32 normal_z          ;
    f32 global_pos_x_l0   ;         
    f32 global_pos_y_l0   ;         
    f32 global_pos_z_l0   ;         
    f32 global_pos_x_l0_A ;         
    f32 global_pos_y_l0_A ;         
    f32 global_pos_z_l0_A ;         
    f32 global_pos_x_l0_B ;         
    f32 global_pos_y_l0_B ;         
    f32 global_pos_z_l0_B ;         
    f32 coax_cable_time   ;
    f32 harting_cable_time;
  
    auto to_string() const -> std::string;
  
    /// Vector along the longest axis
    auto get_principal() const -> Vec<f32>;
  };
  /// A map of paddle id -> TofPaddle
  typedef std::map<u8,  TofPaddle> TofPaddleMap;
  /// A map of RBID, RBCh -> TofPaddle
  typedef std::map<u8, std::map<u8, std::tuple<u8, TofPaddleEnd>>> RbIdChannelPaddleIdMap;
  /// A map of DSI,J -> TofPaddle
  typedef std::map<u8, std::map<u8, std::map<u8, u8>>> DsiJChnPaddleIdMap;

  /// Get a paddle from the database
  auto get_tofpaddles() -> TofPaddleMap;        
 
  /// Get a paddle if the rb id and channel is known (HG)
  auto get_rb_id_paddles() -> RbIdChannelPaddleIdMap;

  /// Get a paddle if the dsi,j connection of a paddle is known (LTB, LG)
  auto get_dsi_j_paddles() -> DsiJChnPaddleIdMap;

  struct TrackerStrip {
    u32 strip_id           ;
    i32 layer              ; 
    i32 row                ; 
    i32 module             ; 
    i32 channel            ;  
    f32 global_pos_x_l0    ;
    f32 global_pos_y_l0    ;
    f32 global_pos_z_l0    ;
    f32 global_pos_x_det_l0;
    f32 global_pos_y_det_l0;
    f32 global_pos_z_det_l0;
    f32 principal_x        ;
    f32 principal_y        ;
    f32 principal_z        ;
    u64 volume_id          ;
  
    auto to_string() const -> std::string;
    auto create_id() const -> u32; 
    static auto create_id(u32 layer, u32 row, u32 module, u32 channel) -> u32;
    /// Vector along the longest axis
    auto get_principal() const -> Vec<f32>;
  };

  /// A map of strip identifier (layer-row-module-channel -> Tracker strip
  typedef std::map<u32, TrackerStrip> TrkStripMap;
  
  /// Retrieve all tracker strips from the database
  auto get_trackerstrips() -> TrkStripMap;        
  
  /// Get the position of a module - returns in cm
  auto get_module_position(u8 layer, u8 row, u8 mod, const TrkStripMap&) -> Vec<f32>;

  /// Each module can have a mask, which allows to disable
  /// trcker strips. The mask is typically a 32bit number
  struct TrackerStripMask {
    u32         strip_id ;
    u64         volume_id;
    u64         utc_timestamp;
    std::string mask_name; 
    bool        active     ; 
  
    auto to_string() const -> std::string;

  };

  typedef std::map<u32, bool> TrkStripMaskMap;

  auto get_trackerstripmasks(std::string mask_name = "") -> TrkStripMaskMap;

  struct TrackerStripPedestal {
    u32     strip_id;
    u64     volume_id;
    u64     utc_timestamp;
    f32     pedestal_mean;
    f32     pedestal_sigma;
    bool    is_mean_value;
  
    auto to_string() const -> std::string; 
  };
  
  typedef std::map<u32, TrackerStripPedestal> TrkStripPedMap;
  
  auto get_trackerstrippedestals() -> TrkStripPedMap;
}

// new items shall go directly into the new gondola namespace 
namespace gondola {
  /// The mapping of volume id to hardware id, which is either the strip
  /// identifier or the paddle id 
  auto get_hid_vid_map() -> HashMap<u32, u32>;

  /// The mapping of hardwer id (either paddle id or strip id to the 
  /// volume id
  auto get_vid_hid_map() -> HashMap<u32, u32>; 
  
  /// Arbitrary timing constant which is calibrated out 
  /// by requiring that overrlapping paddles should see 
  /// the same signal at the same time. Between panels, 
  /// the muon signal should be received at the known time  
  struct TofPaddleTimingConstant {
    u32         data_id; 
    u8          paddle_id ;
    u64         volume_id;
    u64         utc_timestamp_start;
    u64         utc_timestamp_stop;
    std::string name; 
    f32         version;
    f32         timing_constant; 
  
    auto to_string() const -> std::string;

  };

  typedef std::map<u32, f32> TofPaddleTimingConstantMap;

  auto get_tofpaddletimingconstants(std::string mask_name = "") -> TofPaddleTimingConstantMap;
}

std::ostream& operator<<(std::ostream& os, const Gaps::TofPaddle& paddle);

std::ostream& operator<<(std::ostream& os, const Gaps::TrackerStrip& strip);

std::ostream& operator<<(std::ostream& os, const Gaps::TrackerStripMask& strip);

std::ostream& operator<<(std::ostream& os, const Gaps::TrackerStripPedestal& strip);

std::ostream& operator<<(std::ostream& os, const gondola::TofPaddleTimingConstant& paddle);

#endif
