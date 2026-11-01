#pragma once

#include "G4Step.hh"
#include "G4UserSteppingAction.hh"
#include "gondola_typedefs.hpp"

namespace gondola { 
  struct SteppingAction : public G4UserSteppingAction {
    u32 primary_stopping_volume = 0;
    f64 primary_stopping_x      = 0.0;
    f64 primary_stopping_y      = 0.0;
    f64 primary_stopping_z      = 0.0;
    f64 primary_stopping_t      = 0.0;
    f64 primary_stopping_kinE   = 0.0; 
  
    auto UserSteppingAction(const G4Step* step) -> void;
  };
}
