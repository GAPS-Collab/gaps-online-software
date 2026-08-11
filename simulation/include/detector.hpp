#ifndef DETECTOR_H_INCLUDED
#define DETECTOR_H_INCLUDED

#include "G4VUserDetectorConstruction.hh"
#include "G4VPhysicalVolume.hh"
#include "G4PVPlacement.hh"
#include "G4Box.hh"

#include "tof_typedefs.h"

#include "materials.hpp"
#include "volume_store.hpp"
#include "sim_config.hpp"

namespace gondola {
  typedef G4PVPlacement* G4PVPlacementPtr;

  struct GapsDetector : G4VUserDetectorConstruction {
    u8                                      version;
    bool                                    check_overlap;
    G4PVPlacementPtr                        world;
    /// Keep references to the active volumes, so that when 
    /// we save the geometry, we can add the information 
    /// about them being active to the gdml file through 
    /// the parser
    Vec<G4LogicalVolumePtr>                 active_vols = {};
    //auto construct_detector() -> G4VPhysicalVolumePtr;
    GapsDetector(const SimConfig& cfg);
    auto Construct()                     -> G4VPhysicalVolume* override;
    auto ConstructSDandField()           -> void override;
    auto SaveGeometry(std::string fname) -> void;
    // constructors, destructors
    virtual ~GapsDetector(){};
    private:
      SimConfig sim_config;
  };
}

#endif
