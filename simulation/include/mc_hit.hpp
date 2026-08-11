#pragma once

#include "G4VHit.hh" 
#include "G4THitsCollection.hh"
#include "G4Allocator.hh"
#include "gondola.hpp"

namespace gondola {
  /// McHit is inspired by SimplDet's CHit and combines
  /// information from G4Track and G4Step for further 
  /// processing
  struct McHit : public G4VHit {
    u32 volume_id    = 0;
    u32 hw_id        = 0;
    u32 parent_id    = 0;
    u32 track_id     = 0;
    f32 kin_E        = 0;
    f32 glob_time    = 0;
    f32 pos_x        = 0;
    f32 pos_y        = 0;
    f32 pos_z        = 0;
    f32 vertex_pos_x = 0;
    f32 vertex_pos_y = 0;
    f32 vertex_pos_z = 0;
    f32 vertex_kin_E = 0;
    f32 mom_x        = 0;
    f32 mom_y        = 0;
    f32 mom_z        = 0;
    f32 vertex_mom_x = 0;
    f32 vertex_mom_y = 0;
    f32 vertex_mom_z = 0;
    f32 step_len     = 0;
    /// The total energy deposition of the step
    f32 step_edep    = 0;
    // Pre (when enterting the stepping action)  
    f32 pre_mom_x    = 0;
    f32 pre_mom_y    = 0;
    f32 pre_mom_z    = 0;
    f32 pre_kin_E    = 0;
    // FIXME - not yet serialized
    // pdg 
    i32 pdg                   = 0;
    i32 pre_step_status       = 0;
    i32 post_step_status      = 0;
    u32 vertex_vol_id         = 0;
    u32 vertex_hw_id          = 0;
    bool is_first_step_in_vol = false;
    bool is_last_step_in_vol  = false;
    u8  process_type          = 0;
    // new - not used yet, save beta of the track
    f32 beta                  = 0.0;

    auto to_bytestream() const -> Vec<u8>;
   
  };
}

typedef G4THitsCollection<gondola::McHit> McHitCollection;
extern G4Allocator<gondola::McHit> McHitAllocator;

