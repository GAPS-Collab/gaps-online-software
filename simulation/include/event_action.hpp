#pragma once 

#include <memory>
#include "G4UserEventAction.hh"
//#include "G4ThreeVector.hh"
#include "G4Event.hh"

#include "gondola.hpp"
#include "caraspace.hpp"

namespace gondola {

    class EventAction : public G4UserEventAction {
      public:
        EventAction(std::string fname);
        ~EventAction();

        auto BeginOfEventAction(const G4Event*) -> void;
        auto EndOfEventAction(const G4Event*)   -> void;
        std::shared_ptr<CRWriter> writer;
  };
}

