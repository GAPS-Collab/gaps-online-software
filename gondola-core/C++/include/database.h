#ifndef GOS_DB_HEADER_INCLUDED
#define GOS_DB_HEADER_INCLUDED

#include <memory> 

#include "gondola_typedefs.hpp"
#include "sqlite_orm.h"

#include <map>

#include "database/tof_paddle.hpp"
#include "database/tracker_strip.hpp"

namespace gondola {
  
  enum class TofPaddleEnd : i16 {
    Unknown                = 0,
    A                      = -1,
    B                      = 1,
  };


  /// A map of paddle id -> TofPaddle
  typedef std::map<u8,  TofPaddle> TofPaddleMap;
  /// Shared ptr to TofPaddleMap 
  typedef std::shared_ptr<TofPaddleMap> TofPaddleMapPtr;
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

  /// The mapping of volume id to hardware id, in this case, strip id
  auto get_hid_vid_map_tracker() -> HashMap<u32, u32>;

  /// The mapping of hardwer id, in 
  /// this case, strip id to volume id
  auto get_vid_hid_map_tracker() -> HashMap<u32, u32>; 
  
  auto get_hid_vid_map_tof() -> HashMap<u32, u32>;

  auto get_vid_hid_map_tof() -> HashMap<u32, u32>; 
  
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

  std::ostream& operator<<(std::ostream& os, const gondola::TofPaddle& paddle);
  
  
  std::ostream& operator<<(std::ostream& os, const gondola::TrackerStripMask& strip);
  
  std::ostream& operator<<(std::ostream& os, const gondola::TrackerStripPedestal& strip);
  
  std::ostream& operator<<(std::ostream& os, const gondola::TofPaddleTimingConstant& paddle);

#endif
