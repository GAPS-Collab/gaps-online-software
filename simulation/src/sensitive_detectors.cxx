#include "G4ProcessType.hh"

#include "sensitive_detectors.hpp" 

namespace go = gondola;

go::SensitiveDetector::SensitiveDetector(G4String sdname)
  :G4VSensitiveDetector(sdname){
  collectionName.insert("Collection");
}

go::SensitiveDetector::~SensitiveDetector() {
}

auto go::SensitiveDetector::Initialize(G4HCofThisEvent* HCE) -> void {
  mchit_collection_ = new McHitCollection(SensitiveDetectorName, collectionName[0]);
  if (HCE != nullptr) {
    static G4int HCID = -1;
    if (HCID < 0) {
       HCID = G4SDManager::GetSDMpointer()->GetCollectionID(collectionName[0]);
    }
    HCE->AddHitsCollection(HCID, mchit_collection_);
  }
}

auto go::SensitiveDetector::ProcessHits(G4Step* step, G4TouchableHistory* history) -> G4bool {
  if (step->GetTrack() != nullptr) {
    if (step->GetTrack()->GetVolume() != nullptr) {
      //std::cout << "step " << step->GetTrack()->GetVolume()->GetCopyNo() << std::endl;
      auto hit   = new McHit();
      auto track = step->GetTrack();
      auto beta  = track->GetDynamicParticle()->GetBeta();          
      hit->beta         = beta;
      hit->volume_id    = track->GetVolume()->GetCopyNo();
      hit->parent_id    = track->GetParentID();
      hit->track_id     = track->GetTrackID();
      hit->kin_E        = track->GetKineticEnergy();
      hit->glob_time    = track->GetGlobalTime();
      auto pos          = track->GetPosition();
      hit->pos_x        = pos.getX();
      hit->pos_y        = pos.getY();
      hit->pos_z        = pos.getZ();
      auto vpos         = track->GetVertexPosition();
      hit->vertex_pos_x = vpos.getX();
      hit->vertex_pos_y = vpos.getY();
      hit->vertex_pos_z = vpos.getZ();
      hit->vertex_kin_E = track->GetVertexKineticEnergy();
      auto mom          = track->GetMomentumDirection();
      hit->mom_x        = mom.getX();
      hit->mom_y        = mom.getY();
      hit->mom_z        = mom.getZ();
      mom               = track->GetVertexMomentumDirection();
      hit->vertex_mom_x = mom.getX();
      hit->vertex_mom_y = mom.getY();
      hit->vertex_mom_z = mom.getZ();
      hit->step_len     = step->GetStepLength();
      hit->step_edep    = step->GetTotalEnergyDeposit();
      auto pre_step_pnt = step->GetPreStepPoint();
      mom               = pre_step_pnt->GetMomentumDirection();
      hit->pre_mom_x    = mom.getX(); 
      hit->pre_mom_y    = mom.getY(); 
      hit->pre_mom_z    = mom.getZ(); 
      hit->pre_kin_E    = pre_step_pnt->GetKineticEnergy();
      hit->pdg                  = step->GetTrack()->GetDefinition()->GetPDGEncoding(); 
      hit->pre_step_status      = step->GetPreStepPoint()->GetStepStatus();
      hit->post_step_status     = step->GetPostStepPoint()->GetStepStatus();
      hit->is_first_step_in_vol = step->IsFirstStepInVolume();
      hit->is_last_step_in_vol  = step->IsLastStepInVolume();
      auto p_type_nd            = G4ProcessType::fNotDefined;
      auto process              = step->GetTrack()->GetCreatorProcess();
      if (process != nullptr) {
        hit->process_type       = process->GetProcessType();
      }
      mchit_collection_->insert(hit);
    } 
  } else {
    std::cout << "NoTrack" << std::endl;
  }
  return true;
}

auto go::SensitiveDetector::EndOfEvent(G4HCofThisEvent*) -> void {

}

