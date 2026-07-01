#include <filesystem>

#include "G4SDManager.hh"

#include "event_action.hpp"
#include "mc_hit.hpp"
#include "mc_event.hpp"
#include "io/parsers.h"

namespace g  = gondola;
namespace fs = std::filesystem;

g::EventAction::EventAction(std::string fname){
  auto path = fs::current_path().string();
  std::cout << "Will write to " << path << std::endl;
  writer = std::shared_ptr<CRWriter>(new CRWriter(path, fname, 0, Some(0), None));
}

g::EventAction::~EventAction(){}

auto g::EventAction::BeginOfEventAction(const G4Event* event) -> void {
  // we can actually install filters here, to only select events with 
  // specific conditions on the primary

}

//------------------------------------------------------------------------

auto g::EventAction::EndOfEventAction(const G4Event* event)   -> void {
  McHitCollection* mchit_collection = nullptr;
  G4SDManager* SDman = G4SDManager::GetSDMpointer();
  G4HCofThisEvent* HCE = event->GetHCofThisEvent(); 
  auto mc_event = McEvent();
  if (HCE != nullptr) {
    G4int Id           = SDman->GetCollectionID("Collection");
    mchit_collection = (McHitCollection*)HCE->GetHC(Id);
    g::McHit* hit;
    for(u32 k = 0;k<mchit_collection->entries();k++) {
       hit = (*mchit_collection)[k];

       //std::cout << "mchit ====================" << std::endl; 
       //std::cout << "vol id       : " << hit->volume_id    << std::endl;
       //std::cout << "hw id        : " << hit->hw_id        << std::endl;
       //std::cout << "parent id    : " << hit->parent_id    << std::endl;
       //std::cout << "track id     : " << hit->track_id     << std::endl;
       //std::cout << "kinE id      : " << hit->kin_E        << std::endl;
       //std::cout << "glob time id : " << hit->glob_time    << std::endl;
       //std::cout << "x            : " << hit->pos_x        << std::endl;
       //std::cout << "y            : " << hit->pos_y        << std::endl;
       //std::cout << "z            : " << hit->pos_z        << std::endl;
       //std::cout << "vx           : " << hit->vertex_pos_x << std::endl;
       //std::cout << "vy           : " << hit->vertex_pos_y << std::endl;
       //std::cout << "vz           : " << hit->vertex_pos_z << std::endl;
       //std::cout << "vkinE        : " << hit->vertex_kin_E << std::endl;
       //std::cout << "mom x        : " << hit->mom_x        << std::endl;
       //std::cout << "mom y        : " << hit->mom_y        << std::endl;
       //std::cout << "mom z        : " << hit->mom_z        << std::endl;
       //std::cout << "mom vx       : " << hit->vertex_mom_x << std::endl;
       //std::cout << "mom vy       : " << hit->vertex_mom_y << std::endl;
       //std::cout << "mom vz       : " << hit->vertex_mom_z << std::endl;
       //std::cout << "step len     : " << hit->step_len     << std::endl;
       //std::cout << "pre step x   : " << hit->pre_mom_x    << std::endl;
       //std::cout << "pre step x   : " << hit->pre_mom_y    << std::endl;
       //std::cout << "pre step x   : " << hit->pre_mom_z    << std::endl;
       //std::cout << "pre kin E    : " << hit->pre_kin_E    << std::endl;
       //std::cout << "step len     : " << hit->step_len     << std::endl;
       //std::cout << "=========================" << std::endl;
       mc_event.hits.push_back(*hit); 
       //exit(1);
    }
  }
  mc_event.event_id = event->GetEventID();

  // fill the primary information 
  if (event->GetNumberOfPrimaryVertex() != 0) {
    auto p_vertex   = event->GetPrimaryVertex(0);
    auto time       = p_vertex->GetT0();
    auto p_particle = p_vertex->GetPrimary(0);
    auto pdg        = p_particle->GetPDGcode();
    auto p_kin_e    = p_particle->GetKineticEnergy();
    auto p_mom_dir  = p_particle->GetMomentumDirection();
    auto vertex     = std::make_shared<g::RecoHit>();
    vertex->x       = p_vertex->GetX0();
    vertex->y       = p_vertex->GetY0();
    vertex->z       = p_vertex->GetZ0(); 
    vertex->time    = time;
    vertex->energy  = p_kin_e;
    auto primary_track = g::Tracklet(vertex);
    primary_track.vertex_mom_x = p_mom_dir.getX();
    primary_track.vertex_mom_y = p_mom_dir.getY();
    primary_track.vertex_mom_z = p_mom_dir.getZ();
    primary_track.pdg          = p_particle->GetPDGcode(); 

    mc_event.primary   = std::move(primary_track);
    //auto p_mom_tot  = p_particle->GetP(); // Magnitude of momentum
    //auto  momentumVector = primaryParticle->GetMomentum(); // 3D vector (pX, pY, pZ)


  } else {
    std::cout << "ERROR - no primary in event!" << std::endl;
  }
  //std::cout << event->GetEventID() << std::endl;
  //std::cout << event->GetPrimaryVertex()->GetX0() << std::endl; 
  //std::cout << event->GetPrimaryVertex()->GetY0() << std::endl; 
  //std::cout << event->GetPrimaryVertex()->GetZ0() << std::endl; 
  //std::cout << "===================================" << std::endl;
  auto f_obj    = CRFrameObject();
  f_obj.version = 0;
  f_obj.ftype   = CRFrameObjectType::McTree;
  f_obj.payload = mc_event.to_bytestream();
  auto frame = CRFrame();
  frame.put_fobject(f_obj, "McEvent");
  writer->add_frame(frame); 
}

