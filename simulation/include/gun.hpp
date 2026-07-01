#ifndef PRIMARY_GENERATOR_ACTION_HH
#define PRIMARY_GENERATOR_ACTION_HH

#include <random>

#include "G4VUserPrimaryGeneratorAction.hh"
#include "G4ParticleGun.hh"
#include "G4GeneralParticleSource.hh"
#include "G4ParticleTable.hh"
#include "G4ThreeVector.hh"
#include "G4SystemOfUnits.hh"

#include "gondola.hpp"
#include "sim_config.hpp"

class PrimaryGeneratorAction : public G4VUserPrimaryGeneratorAction {
  public:
    PrimaryGeneratorAction(const SimConfig& cfg);
    ~PrimaryGeneratorAction() override;

    auto GeneratePrimaries(G4Event* event) -> void override;

    auto set_gun_position(f32 x, f32 y, f32 z) -> void; 
    /// The original SimpleDet Cube solver
    auto random_sample_isotropic() -> void;
    f32 pos_offset_x = 0.0;
    f32 pos_offset_y = 0.0;
    f32 pos_offset_z = 0.0;

  private:
    G4ParticleGun* fParticleGun; // Or use G4GeneralParticleSource if preferred
    SimConfig sim_config;
    std::mt19937 random_gen;  
    std::uniform_real_distribution<double> primary_energy; 
};

#endif
