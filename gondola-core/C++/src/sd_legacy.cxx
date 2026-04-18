#ifdef BUILD_CXX_WITH_ROOT

#include <iostream>

#include "TChain.h"

#include "sd_legacy.hpp"
#include "telemetry_dataclasses.hpp"
#include "events.h"
#include "database.h"

namespace cb = Crane::Calibration;
namespace tf = Crane::Reconstruction::TrackFit;
namespace  g = gondola;

ClassImp(GRecoHit);
ClassImp(CEventBase);
ClassImp(CTrackBase);
ClassImp(CEventRec);
ClassImp(CTrackRec);
ClassImp(cb::CRawHeader);
ClassImp(cb::CRawTrk);
ClassImp(cb::CRawTofHits);
ClassImp(cb::CRawTrigger);
ClassImp(cb::CRawTofWFs);
ClassImp(cb::CRawTof);
//ClassImp(tr::Plane);
ClassImp(tr::GDataEvent);


void gondola::read_sd_legacy_example(std::string filename) {
  auto hid_vid_map    = gondola::get_hid_vid_map();
  TChain * input_tree = new TChain("TreeRec");
  input_tree->Add(filename.c_str());
  auto input_event = new CEventRec();
  input_tree->SetBranchAddress("Rec", &input_event);
  u64 max_events = input_tree->GetEntries();
  for (u64 evid=0; evid<max_events; evid++) {
    input_tree->GetEntry(evid);
    //std::cout << input_event->runNumber_ << std::endl; 
    //std::cout << input_event->run_number << std::endl; 
    //std::cout << input_event->eventId_ << std::endl; 
    //std::cout << input_event->to_telemetry(hid_vid_map).to_string() << std::endl;
    std::cout << input_event->pretty_print() << std::endl;
    break;
  }
}

//------------------------------------------------------------------------

gondola::SDRootReader::SDRootReader(std::string fname) {
  filename            = fname;
  tchain              = new TChain("TreeRec");
  tchain->Add(fname.c_str());
  event               = new CEventRec();
  tchain->SetBranchAddress("Rec", &event);
  nevents_total       = tchain->GetEntries();
}

gondola::SDRootReader::~SDRootReader() {
  delete event;
  delete tchain;
}

auto gondola::SDRootReader::get_event(u64 event_idx) -> void {
  tchain->GetEntry(event_idx); 
  std::cout << event->pretty_print() << std::endl;
}

//------------------------------------------------------------------------

auto GRecoHit::GetVolId() const -> u32 {
  return volume_id_;
}

auto GRecoHit::GetEDep()  const -> f64 {
  return energydep_;
}

auto GRecoHit::GetPos()   const -> TVector3 {
  return hit_position_;
}

auto GRecoHit::GetTime()  const -> f64 {
  return hit_time_;
}

auto GRecoHit::GetIdx()   const -> i32 {
  return index_;
}

auto GRecoHit::pretty_print() const -> std::string {
  auto vid  = volume_id_;
  auto edep = energydep_;
  auto x    = hit_position_.X();
  auto y    = hit_position_.Y();
  auto z    = hit_position_.Z();
  auto t    = hit_time_; 
  auto idx  = index_;
  std::string repr = std::format("<GRecoHit : Vid {} Edep {:.3e} X {:.2f} Y {:.2f} Z {:.2f} Time {:.2f} Idx {}>", vid, edep, x, y, z, t, idx);
  return repr; 
}

//------------------------------------------------------------------------

auto CTrackRec::pretty_print() const -> std::string {
  // FIXME - provide pointers
  auto vid_hid_map    = gondola::get_vid_hid_map();
  i32 pid = -1;
  if (vid_hid_map.contains(VertexVolumeId)) {
    pid  = vid_hid_map.at(VertexVolumeId);
  }
  auto vpos = VertexPosition;
  auto lpos = LastPosition;
  std::string repr = "<CTrackRec (cpt layer)";
  repr += std::format("\n   Primary        : {}", Primary);
  repr += std::format("\n   TrackId        : {}", TrackId);
  repr += std::format("\n   VertexVolumeId : {}/[{}]", VertexVolumeId, pid);
  repr += std::format("\n   VertexPos X {:.2f} Y {:.2f} Z {:.2f}", vpos.X(), vpos.Y(), vpos.Z());
  if (vid_hid_map.contains(LastVolumeId)) {
    pid = vid_hid_map.at(LastVolumeId);
  } else {
    pid = -1;
  }
  repr += std::format("\n   LastVolumeId   ; {}/[{}]", LastVolumeId, pid);
  repr += std::format("\n   LastPos   X {:.2f} Y {:.2f} Z {:.2f}", lpos.X(), lpos.Y(), lpos.Z());
  repr += std::format("\n   ColumnDensity  : {}", ColumnDensity); 
  repr += std::format("\n   bool Used      : {}", Used); 
  repr += std::format("\n   Associated     : {}", Associated);
  repr += std::format("\n   Chi2           : {}", Chi2);
  repr += std::format("\n   Ndof           : {}", Ndof);
  repr += std::format("\n   FitStatus      : {}>", FitStatus); 
  
  repr += "\n -- Vector size checks";
  repr += std::format("\n -- -- EnergyDeposition       {}", EnergyDeposition      .size()) ;
  repr += std::format("\n -- -- GlobalTime             {}", GlobalTime            .size()) ;
  repr += std::format("\n -- -- StepLength             {}", StepLength            .size()) ;
  repr += std::format("\n -- -- VolumeId               {}", VolumeId              .size()) ;
  repr += std::format("\n -- -- Position               {}", Position              .size()) ;
  repr += std::format("\n -- -- PositionResidual       {}", PositionResidual      .size()) ;
  repr += std::format("\n -- -- MomentumDirection      {}", MomentumDirection     .size()) ;
  repr += std::format("\n -- -- Depth                  {}", Depth                 .size()) ; 
  repr += std::format("\n -- -- ColumnDensityUntilStep {}", ColumnDensityUntilStep.size()) ; 
  repr += "\n -- Hits --";
  for (usize k=0; k<EnergyDeposition.size(); k++) {
    auto pos     = Position.at(k);
    auto pos_res = PositionResidual.at(k);
    auto mom     = MomentumDirection.at(k);
    repr += std::format("\n -- EnergyDepositon  {}", EnergyDeposition.at(k));
    repr += std::format("\n -- GlobalTime       {}", GlobalTime.at(k));        
    repr += std::format("\n -- StepLength       {}", StepLength.at(k));
    if (vid_hid_map.contains(VolumeId.at(k))) {
      pid = vid_hid_map.at(VolumeId.at(k));
    } else {
      pid = -1;
    }
    repr += std::format("\n -- VolumeId         {}/[{}]", VolumeId.at(k), pid);
    repr += std::format("\n -- Position         X {:.2f} Y {:.2f} Z {:.2f}",pos.X(), pos.Y(), pos.Z());
    repr += std::format("\n -- PoistionResidual X {:.2f} Y {:.2f} Z {:.2f}",pos_res.X(), pos_res.Y(), pos_res.Z());
    repr += std::format("\n -- MomentumDir      X {:.2f} Y {:.2f} Z {:.2f}",mom.X(), mom.Y(), mom.Z()); 
    //repr += std::format("\n -- Depth            {}", Depth.at(k));
    repr += std::format("\n -- ColDensUntStep   {}", ColumnDensityUntilStep.at(k));
    repr += "\n -- -- -- -- -- --";
  }
  repr += "\n"; 
  return repr;
}

//------------------------------------------------------------------------

auto CEventRec::from_telemetry(g::TelemetryEvent const &event) -> CEventRec {
  auto sd_event = CEventRec();
  return sd_event;
}

//------------------------------------------------------------------------

auto CEventRec::GetGPSTime() const -> f64 {
  u64 gps_time48 = 0x273000000000000 | (u64) gps_time_upper_ << 32 | (u64) gps_time_lower_;
  // this timestamp has 10ns precision 
  return (f64)gps_time48 * 1e-8;
}

//------------------------------------------------------------------------

auto CEventRec::to_telemetry(HashMap<u32, u32> const &vid_hid_map) -> g::TelemetryEvent {
  auto event = g::TelemetryEvent();
  event.event_id = eventId_;
  for (auto const &hit : hitseries_) {
    // check if tracker or tof hit 
    if (hit.GetVolId() > 200000000) {
      g::TrkHit trk_hit;
      u32 hw_id = vid_hid_map.at((u32)hit.GetVolId());
      auto lrms = g::TrkHit::decode_id(hw_id);
      // some fields are just lost
      trk_hit.layer           = lrms[0];
      trk_hit.row             = lrms[1];
      trk_hit.module          = lrms[2];
      trk_hit.channel         = lrms[3];
      //trk_hit.adc             {-1};
      //trk_hit.oscillator      {-1};
      trk_hit.energy = hit.GetEDep();
      //trk_hit.asic_event_code {0};
      event.trk_hits.push_back(trk_hit);
    } else {
      g::TofHit tofhit;
      tofhit.event_t0  = hit.GetTime();  
      //tofhit.
      tofhit.paddle_id = vid_hid_map.at(hit.GetVolId());  
      event.tof_event.hits.push_back(tofhit);
    }
  }  
  return event;
}

//------------------------------------------------------------------------

auto CEventRec::pretty_print() const -> std::string {
  std::string repr = "<CEventRec (compat layer):";
  repr += "\n ---- CEventBase ----";
  repr += std::format("\n Run Number     {}",  runNumber_);
  repr += std::format("\n Subrun Number  {}", subRunNumber_);
  repr += std::format("\n Event Number   {}", eventNumber_);
  repr += std::format("\n Event Time     {}", eventTime_);
  repr += std::format("\n Event Id       {}", eventId_);
  
  repr += std::format("\n Prim. Beta Gen {}", primaryBetaGenerated_);  
  auto pmg = primaryMomentumDirectionGenerated_;
  repr += std::format("\n Prim. Mom Gen X {:.2f} Y {:.2f} Z {:.2f}", pmg.X(), pmg.Y(), pmg.Z());
  repr += std::format("\n Prim. KinE Gen {}", primaryKineticEnergyGenerated_);  
  repr += "\n ---- CEventRec ----";
  repr += "\n -- registered recos:";
  for (auto const &k : registeredRecos_) {
    repr += std::format("\n -- -- {}", k);
  } 
  repr += std::format("\n active reco    {}", activeReco_); 
  repr += std::format("\n GPS Time       {}", GetGPSTime());
  repr += std::format("\n Packet Type    {}", PacketType);
  repr += "\n -- event qualities:";
  for (auto const &k : event_quality) {
    repr += std::format("\n -- -- {}", k);
  }
  repr += "\n -- trigger sources:";
  for (auto const &k : trigger_sources) {
    repr += std::format("\n -- -- {}", k);
  }
  repr += "\n -- trigger volume ids:";
  for (auto const &k : trigger_vids) {
    repr += std::format("\n -- -- {}", k);
  }
  repr += "\n -- GRecoHitSeries:";
  usize n_hit = 0;
  for (auto const &k : hitseries_) {
    repr += std::format("\n -- {} : {}", n_hit, k.pretty_print());
    n_hit += 1;
  }
  repr += std::format("\n == => In total {} RecoHits!\n", n_hit);
  if (registeredRecos_.size() > 0) {
    repr += "\n -- Primary reconstruction values:";
    for (auto const &k : registeredRecos_) {
      auto pstop = primaryStoppingPosition_.at(k);
      repr += std::format("\n Prim. Stop Pos  ({})  X {:.2f} Y {:.2f} Z {:.2f}", k, pstop.X(), pstop.Y(), pstop.Z());
      repr += std::format("\n Prim. Stop Vol  ({})  {}", k, primaryStoppingVolume_.at(k));
      repr += std::format("\n Prim. Stop Time ({})  {}", k, primaryStoppingTime_.at(k));
      repr += std::format("\n Prim. Beta      ({})  {}", k, primaryBeta_.at(k));
      repr += std::format("\n Prim. Beta Err  ({})  {}", k, primaryBetaError_.at(k));
      repr += std::format("\n N Reco Tracks   ({})  {}", k, Tracks.at(k).size()); 
    
      auto pmom = primaryMomentumDirection_.at(k);
      repr += std::format("\n Prim. Direct.   ({})  X {:.2f} Y {:.2f} Z {:.2f}", k, pmom.X(), pmom.Y(), pmom.Z());
      repr += std::format("\n Chi2            ({})  {}", k, Chi2.at(k));
      repr += std::format("\n Ndof            ({})  {}", k, Ndof.at(k));
      repr += std::format("\n FitStatus       ({})  {}", k, FitStatus.at(k));
      repr += std::format("\n SdChi2          ({})  {}", k, SdFitChi2.at(k));
      repr += std::format("\n SdNdof          ({})  {}", k, SdFitNdof.at(k));
      
      //std::map<std::string,Vec<f64>>         primaryEnergyDepositions_; 
      //std::map<std::string,Vec<i32>>         HitTrackIndex; ///< track index of the associated track
      //std::map<std::string,Vec<f64>>         ParCov; ///< covariance matrix of vertex fit parameters

      //std::map<std::string,Vec<f64>>         SdFitPar; ///< slowdown fit parameters {range, Ekin}
      //std::map<std::string,Vec<f64>>         SdFitErr; ///< slowdown fit errors
      repr += "\n -- -- Reco Tracks -- -";
      for (auto const &t : Tracks.at(k)) {
        repr += "\n";
        repr += t->pretty_print();
      }
      repr += "\n -- -- -- -- ";
    }
  }
  return repr;
}

#endif
