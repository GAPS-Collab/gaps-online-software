#include <numbers> 

#include "G4Event.hh"
#include "G4ParticleDefinition.hh"
#include "G4ParticleTable.hh"
#include "gun.hpp"

#include "database.h" 

PrimaryGeneratorAction::PrimaryGeneratorAction(const SimConfig& cfg) {
  fParticleGun = new G4ParticleGun(1); // shoot 1 particle per event

  G4ParticleDefinition* particle = G4ParticleTable::GetParticleTable()->FindParticle(cfg.gun_particle_type);
  fParticleGun->SetParticleDefinition(particle);
  // straight down for now
  fParticleGun->SetParticleMomentumDirection(G4ThreeVector(0., 0., -1.));
  if (cfg.gun_fixed_energy) {
    fParticleGun->SetParticleEnergy(cfg.gun_energy * MeV);
  }
  if (cfg.gun_uniform_energy) {
    primary_energy = std::uniform_real_distribution<double>((f32)cfg.gun_min_e_per_n * MeV,
                                                            (f32)cfg.gun_max_e_per_n * MeV); 
  }
  //std::cout << "FIXEDPOS    " <<  cfg.gun_fixed_pos << std::endl;
  //std::cout << "FIXEDPOS    " <<  cfg.gun_fixed_pos_x << std::endl;
  //std::cout << "FIXEDPOS    " <<  cfg.gun_fixed_pos_y << std::endl;
  //std::cout << "FIXEDPOS    " <<  cfg.gun_fixed_pos_z << std::endl;
  if (cfg.gun_fixed_pos) {
    fParticleGun->SetParticlePosition(G4ThreeVector(cfg.gun_fixed_pos_x,
                                                    cfg.gun_fixed_pos_y,
                                                    cfg.gun_fixed_pos_z));
  }
  if (cfg.gun_center_around_pid != 0) {
    u8 pid = cfg.gun_center_around_pid;
    auto paddle = gondola::get_tofpaddles()[pid];
    //std::cout << paddle << std::endl;
    pos_offset_x =  paddle.global_pos_x_l0*10   ;
    pos_offset_y =  paddle.global_pos_y_l0*10   ;
    pos_offset_z =  paddle.global_pos_z_l0*10   ;     
    //std::cout << "POS OFFSET X " << pos_offset_x << std::endl;
    //std::cout << "POS OFFSET Y " << pos_offset_y << std::endl;
    //std::cout << "POS OFFSET Z " << pos_offset_z << std::endl;
    //exit(1);
  }
  
  sim_config = cfg;
  // set up random number generator 
  std::random_device rd; 
  //std::mt19937 gen(rd());  
  random_gen = std::mt19937(rd());  
}

PrimaryGeneratorAction::~PrimaryGeneratorAction() {
  delete fParticleGun;
}

auto PrimaryGeneratorAction::set_gun_position(f32 x, f32 y, f32 z) -> void {
    fParticleGun->SetParticlePosition(G4ThreeVector(x, y, z));
}

auto PrimaryGeneratorAction::random_sample_isotropic() -> void {
  // setting up random number generation 
  // 2. Initialize a generator (Mersenne Twister is standard for most uses)
  //double Limit = 220*cm;  
  double Limit = sim_config.gun_sample_isotropic_box_len*mm;
  std::uniform_real_distribution<double> dis_limit(-Limit, Limit); 
  std::uniform_real_distribution<double> dis_phi(0, 2*std::numbers::pi); 
  std::uniform_real_distribution<double> dis_theta(0, 1.0); 

  double s1 = 1e100, s2 = 1e100, s3 = 1e100;
  double n1 = 1e100, n2 = 1e100, n3 = 1e100;
  
  // sample until we get something pointing down
  // This will only produce downgoing primaries!
  while (n3 > 0) {
    double phi = dis_phi(random_gen); 
    int plane = (rand()%6)+1;
    double Theta = acos(sqrt(dis_theta(random_gen)));
    switch(plane) {
      case 1: 
        s1 = dis_limit(random_gen);
        s2 = dis_limit(random_gen);
        s3 = Limit;
  
        n1 = sin(Theta)*cos(phi);
        n2 = sin(Theta)*sin(phi);
        n3 = -cos(Theta);
        break;
      case 2:
        s1 = dis_limit(random_gen);
        s2 = dis_limit(random_gen);
        s3 = -Limit;
  
        n1 = -sin(Theta)*cos(phi);
        n2 = sin(Theta)*sin(phi);
        n3 = cos(Theta);
        break;
      case 3:
        s1 = dis_limit(random_gen);
        s2 = Limit;
        s3 = dis_limit(random_gen);
  
        n1 = sin(Theta)*cos(phi);
        n2 = -cos(Theta);
        n3 = -sin(Theta)*sin(phi);
        break;
      case 4:
        s1 = dis_limit(random_gen);
        s2 = -Limit;
        s3 = dis_limit(random_gen);
  
        n1 = sin(Theta)*cos(phi);
        n2 = cos(Theta);
        n3 = sin(Theta)*sin(phi);
        break;
      case 5:
        s1 = Limit;
        s2 = dis_limit(random_gen);
        s3 = dis_limit(random_gen);
  
        n1 = -cos(Theta);
        n2 = sin(Theta)*sin(phi);
        n3 = -sin(Theta)*cos(phi);
        break;
      case 6:
        s1 = -Limit;
        s2 = dis_limit(random_gen);
        s3 = dis_limit(random_gen);
  
        n1 = cos(Theta);
        n2 = sin(Theta)*sin(phi);
        n3 = sin(Theta)*cos(phi);
        break;
     } // end switch
  }
  
  //------------------------
  set_gun_position(pos_offset_x + s1,pos_offset_y + s2, pos_offset_z + s3);
  //particleGun->GetCurrentSource()->GetAngDist()->SetParticleMomentumDirection(G4ThreeVector(n1,n2,n3));
  fParticleGun->SetParticleMomentumDirection(G4ThreeVector(n1, n2, n3));
}

auto PrimaryGeneratorAction::GeneratePrimaries(G4Event* event) -> void{
  if (sim_config.gun_sample_isotropic_box) {
    random_sample_isotropic();
  }
  if (sim_config.gun_uniform_energy) {
    auto p_en = primary_energy(random_gen);
    //std::cout << "-> Primary with energy " << p_en << std::endl;
    fParticleGun->SetParticleEnergy(p_en);
  }
  fParticleGun->GeneratePrimaryVertex(event);
  //event->GetPrimaryVertex()->Print();
}

