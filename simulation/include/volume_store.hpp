#ifndef VOLUME_STORE_H_INCLUDED
#define VOLUME_STORE_H_INCLUDED

#include <map>

#include "G4LogicalVolume.hh"
#include "G4LogicalVolumeStore.hh"
#include "G4TessellatedSolid.hh"
#include "G4GDMLParser.hh"
#include "G4Transform3D.hh"

#include "tof_typedefs.h"
#include "materials.hpp"
#include "sim_config.hpp"

namespace gondola {
  typedef G4LogicalVolume* G4LogicalVolumePtr;
  typedef G4VPhysicalVolume* G4VPhysicalVolumePtr;
  typedef G4VSolid* G4VSolidPtr;
  typedef G4TessellatedSolid* G4TessellatedSolidPtr;

  auto InitLVolumes(const SimConfig& cfg) -> void;

  auto GetLogicalVolumeByName(const G4String& name)              -> G4LogicalVolumePtr; 
  auto GetTessSolidFromGdml(const G4String& name, bool validate) -> G4TessellatedSolidPtr;
  auto GetAssemblyFromGdml(const G4String& name, bool validate) -> Vec<G4VPhysicalVolumePtr>;
}

#endif 
