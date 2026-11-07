#pragma once 

#include <memory>
#include "G4UserEventAction.hh"
//#include "G4ThreeVector.hh"
#include "G4Event.hh"

#include "gondola.hpp"
#include "caraspace.hpp"
#include "digitizer.hpp"
#include "stepping_action.hpp"
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
        /// write the primary of events which do not 
        /// pass the trigger (digi_write_untriggered_primary)
        bool write_untriggered_primary = false;
        // keep a reference to the gun, so 
        // tat at the time of writing the file, 
        // priimary porperties can be accessed
        PrimaryGeneratorAction* gun;
        SteppingAction        * stepping_action;
  };
}

