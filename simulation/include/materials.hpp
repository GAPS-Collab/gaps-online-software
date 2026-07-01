#ifndef MATERIALS_H_INCLUDED
#define MATERIALS_H_INCLUDED

#include <string>
#include <memory>

#include "G4Material.hh"

namespace gondola {
  //typedef std::shared_ptr<G4Material> G4MaterialPtr;
  typedef G4Material* G4MaterialPtr;
  auto GetMaterial(std::string name) -> G4MaterialPtr;
 
  auto InitMaterials() -> void; 

}

#endif
