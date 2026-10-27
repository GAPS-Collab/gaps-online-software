#pragma once 

#include <memory>
#include "G4UserEventAction.hh"
//#include "G4ThreeVector.hh"
#include "G4Event.hh"

#include "gondola.hpp"
#include "caraspace.hpp"
#include "digitizer.hpp"

#include "gun.hpp"

namespace gondola {

    class EventAction : public G4UserEventAction {
      public:
        EventAction(std::string fname, const SimConfig& cfg);
        ~EventAction();

        auto BeginOfEventAction(const G4Event*) -> void;
        auto EndOfEventAction(const G4Event*)   -> void;
        // think about why these are pointers?
        std::shared_ptr<CRWriter>      writer;
        std::shared_ptr<GapsDigitizer> digitizer;
                
        // debug counter
        u64 processed_events = 0;
        PrimaryGeneratorAction* gun;
  };
}

