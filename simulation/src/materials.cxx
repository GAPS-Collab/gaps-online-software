#include <string>

#include "G4NistManager.hh"
#include "G4SystemOfUnits.hh"

#include "materials.hpp"
#include "sili_params.hpp"

//using CLHEP::cm3;
//using CLHEP::perCent;
//using CLHEP::g;
//using CLHEP::mg;

namespace go = gondola;

auto go::GetMaterial(std::string name) -> go::G4MaterialPtr {
  //std::cout << "go::GetMaterial" << std::endl;
  auto nistMan = G4NistManager::Instance();
  G4Material* material_from_table = nullptr;
  material_from_table = G4Material::GetMaterial(name, false);
  if (!material_from_table) {
    if (!name.starts_with("G4_")) {
      // most likely an element where we forgot 
      // the G4
      name = "G4_" + name;
    }
    material_from_table = nistMan->FindOrBuildMaterial(name);
    if (material_from_table == nullptr) {
      std::cout << "We can not find a material or element with the name " << name << "! Remember, if giving an element, please use the chemical symbol!" << std::endl;
      //log_fatal("We can not find a material or element with the name " << name << "! Remember, if giving an element, please use the chemical symbol!");
      exit(EXIT_FAILURE);  
    }
  } 
  auto material = G4MaterialPtr(material_from_table);
  return material;
}

auto go::InitMaterials() -> void {
  // G4 will manage all that we create here with new
  // internally, so we can't use shared pointers
  auto nistMan = G4NistManager::Instance();

  auto elH  = nistMan->FindOrBuildElement("H");
  auto elHe = nistMan->FindOrBuildElement("He");
  auto elC  = nistMan->FindOrBuildElement("C");
  auto elN  = nistMan->FindOrBuildElement("N");
  auto elO  = nistMan->FindOrBuildElement("O");
  auto elAr = nistMan->FindOrBuildElement("Ar");
  auto elSi = nistMan->FindOrBuildElement("Si");
  auto elNi = nistMan->FindOrBuildElement("Ni");
  auto elAu = nistMan->FindOrBuildElement("Au");
  


  G4MaterialPtr pvt = new G4Material("PVT", 1.032*g/cm3, 2);
  pvt->AddElement(elC, 9);
  pvt->AddElement(elH, 10);
  
  G4MaterialPtr air = new G4Material("Air", 0.0057*mg/cm3, 5);
  air->AddElement(elN, 75.5569*perCent);
  air->AddElement(elO, 23.1542*perCent);
  air->AddElement(elAr, 1.28881*perCent);
  air->AddElement(elHe, 7.24467e-05*perCent);
  air->AddElement(elH,  0*perCent);
  
  G4Material* ethafoam = new G4Material("Ethafoam",  0.035*g/cm3, 2);
  ethafoam->AddElement(elC,2);
  ethafoam->AddElement(elH, 4);
  
  G4Material* polyimide = new G4Material("Polyimide",1.42*g/cm3,4);
  polyimide-> AddElement(elC, 35);
  polyimide-> AddElement(elH, 26);
  polyimide-> AddElement(elO, 8);
  polyimide-> AddElement(elN, 4);

  auto sp = go::SiLiParams();
  G4Material* sili_top = new G4Material("SiLiTop", sp.density_toplayer(), 4);
  sili_top->AddElement(elSi, sp.mass_fraction_si_top());
  sili_top->AddElement(elNi, sp.mass_fraction_ni_top());
  sili_top->AddElement(elAu, sp.mass_fraction_au_top());
  sili_top->AddMaterial(go::GetMaterial("Polyimide"), sp.mass_fraction_pi());
  
  G4Material* sili_bot = new G4Material("SiLiBottom",sp.density_botlayer(),3);
  sili_bot->AddElement(elSi, sp.mass_fraction_si_bot());
  sili_bot->AddElement(elNi, sp.mass_fraction_ni_bot());
  sili_bot->AddElement(elAu, sp.mass_fraction_au_bot());

  std::cout << "Materials loaded!" << std::endl;
}

