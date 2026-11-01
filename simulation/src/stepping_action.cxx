#include <limits>
#include <cmath>

#include "stepping_action.hpp"


namespace gon = gondola;

auto gon::SteppingAction::UserSteppingAction(const G4Step* step) -> void {
  u32 vid = 0;
  if (step->GetTrack()->GetTrackID() == 1) {
    vid = step->GetTrack()->GetVolume()->GetCopyNo();
    if (step->GetPostStepPoint()->GetKineticEnergy() == 0) {
      // Workaround - in case the particle does not stop, these values 
      // remain at the ones where it actually stopped.
      // Work around by checking for vid == 0 (for some reason the bug/feature soeem 
      // not to affect the volume id.
      // FIXME
      if (vid == 0) { // particle did not stop, or stops in the "world"
        primary_stopping_x      = std::numeric_limits<f64>::quiet_NaN();
        primary_stopping_y      = std::numeric_limits<f64>::quiet_NaN();
        primary_stopping_z      = std::numeric_limits<f64>::quiet_NaN();
        primary_stopping_t      = std::numeric_limits<f64>::quiet_NaN();
        primary_stopping_kinE   = std::numeric_limits<f64>::quiet_NaN(); 
      } else {
        auto stop_pos = step->GetTrack()->GetPosition();
        primary_stopping_t    = step->GetTrack()->GetGlobalTime();
        primary_stopping_x    = stop_pos.getX();    
        primary_stopping_y    = stop_pos.getY();    
        primary_stopping_z    = stop_pos.getZ();    
        primary_stopping_kinE = step->GetPreStepPoint()->GetKineticEnergy();
      }
    } else { // be paranoid. Not sure if they ever stop
             // omit the volume id here, since that seems to be fine
      primary_stopping_t    = std::numeric_limits<f64>::quiet_NaN();
      primary_stopping_x    = std::numeric_limits<f64>::quiet_NaN();    
      primary_stopping_y    = std::numeric_limits<f64>::quiet_NaN();    
      primary_stopping_z    = std::numeric_limits<f64>::quiet_NaN();    
      primary_stopping_kinE = std::numeric_limits<f64>::quiet_NaN();
    }
  }
}
