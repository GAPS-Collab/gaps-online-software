#pragma once

#include "G4Step.hh"
#include "G4VSensitiveDetector.hh"
#include "G4SDManager.hh"
#include "G4VProcess.hh"
#include "G4HCofThisEvent.hh"

#include "mc_hit.hpp"


namespace gondola {
  class SensitiveDetector : public G4VSensitiveDetector {
    public: 
      SensitiveDetector(G4String);
      ~SensitiveDetector();
     
      auto Initialize(G4HCofThisEvent*) -> void;
      auto ProcessHits(G4Step*, G4TouchableHistory*) -> G4bool;
      auto EndOfEvent(G4HCofThisEvent*) -> void;
    private:
      McHitCollection* mchit_collection_;
  };
}
