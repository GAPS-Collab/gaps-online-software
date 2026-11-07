#ifdef BUILD_CXX_WITH_ROOT

#include <iostream>
#include <algorithm> 
#include <cmath>
#include <initializer_list>

#include "TChain.h"
#include "TFileMerger.h"

#include "sd_legacy.hpp"
#include "telemetry_dataclasses.hpp"
#include "events.h"
#include "database.h"
#include "events/mc_event.hpp"

namespace cb = Crane::Calibration;
namespace tf = Crane::Reconstruction::TrackFit;
namespace  g = gondola;

ClassImp(GRecoHit);
ClassImp(CEventBase);
ClassImp(CTrackBase);
ClassImp(CTrackMc);
ClassImp(CEventMc);
ClassImp(CEventRec);
ClassImp(CTrackRec);
ClassImp(cb::CRawHeader);
ClassImp(cb::CRawTrk);
ClassImp(cb::CRawTofHits);
ClassImp(cb::CRawTrigger);
ClassImp(cb::CRawTofWFs);
ClassImp(cb::CRawTof);
//ClassImp(tr::Plane);
ClassImp(tf::GDataEvent);

auto elena_cut(const g::TofHit& hit) -> bool {
  //---------------------------------------------------------
  //
  // paddle hit condition
  //
  //---------------------------------------------------------
  double peak_cut   = 10.;
  double charge_cut = 5.;
  double time_sat   = 490.;    

  bool hit_a = (hit.get_time_a() > 0.1 
                && hit.get_time_a() < time_sat
                && hit.get_peak_a() > peak_cut
                && hit.get_charge_a() > charge_cut);
  bool hit_b = (hit.get_time_b() > 0.1
                && hit.get_time_b() < time_sat
                && hit.get_peak_b() > peak_cut
                && hit.get_charge_b() > charge_cut);

  // currently not used (by Elena) 
  //double peak_sat   = 720.;
  //bool sat_a = (peak_a > peak_sat);
  //bool sat_b = (peak_b > peak_sat);
  // *****************************************************
  // NB!! FOR THE MOMENT KEEP ONLY PADDLE WITH DOUBLE HITS
  // *****************************************************
  return hit_a && hit_b;
}

void gondola::read_sd_legacy_example(std::string filename) {
  //auto hid_vid_map    = gondola::get_hid_vid_map();
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

//========================================================================

g::SDRootReader::SDRootReader(std::string fname, bool mc) {
  filename            = fname;
  if (!mc) {
    tchain              = new TChain("TreeRec");
    tchain->Add(fname.c_str());
    event               = new CEventRec();
    tchain->SetBranchAddress("Rec", &event);
    nevents_total       = tchain->GetEntries();
    rawtrk              = new Crane::Calibration::CRawTrk()   ;
    rawtof              = new Crane::Calibration::CRawTof()   ;
    rawhd               = new Crane::Calibration::CRawHeader();
    raw_tree            = new TChain("TreeRaw");
    raw_tree->Add(fname.c_str());
    raw_tree->SetBranchAddress("Trk", &rawtrk); 
    raw_tree->SetBranchAddress("Tof", &rawtof);
    paddle_vid_hid_map  = gondola::get_vid_hid_map_tof(); 
    is_mc               = false;
  } else {
    mc_tchain           = new TChain("TreeMc"); 
    mc_tchain->Add(fname.c_str());
    mc_event            = new CEventMc();
    mc_tchain->SetBranchAddress("Mc", &mc_event);
    nevents_total       = mc_tchain->GetEntries();
    is_mc               = true;
  }
}

//------------------------------------------------------------------------

g::SDRootReader::~SDRootReader() {
  if (is_mc) {
    delete mc_event;
    delete mc_tchain;
  } else {
    delete event;
    delete tchain;
  }
}

//------------------------------------------------------------------------

auto g::SDRootReader::get_event(u64 event_idx) -> void {
  if (is_mc) {
    mc_tchain->GetEntry(event_idx);
    std::cout << mc_event->pretty_print() << std::endl;
  } else {
    tchain->GetEntry(event_idx); 
    std::cout << event->pretty_print() << std::endl;
    if (raw_tree != nullptr) {
      raw_tree->GetEntry(event_idx);
      std::cout << rawtrk->pretty_print() << std::endl;
      std::cout << rawhd->pretty_print() << std::endl; 
      std::cout << rawtof->pretty_print() << std::endl;
    } else {
      std::cout << "-- There is no raw_tree --" << std::endl;
    }
  }
}

//------------------------------------------------------------------------

auto g::SDRootReader::get_event_tof_energies(u64 event_idx) -> Vec<f32> {
  tchain->GetEntry(event_idx); 
  return event->get_tof_energies();
  //std::cout << event->pretty_print() << std::endl;
}

//------------------------------------------------------------------------

auto g::SDRootReader::get_event_trk_energies(u64 event_idx) -> Vec<f32> {
  tchain->GetEntry(event_idx); 
  return event->get_trk_energies();
  //std::cout << event->pretty_print() << std::endl;
}

//------------------------------------------------------------------------

auto g::SDRootReader::get_simple_beta(u64 event_idx) -> f32 {
  tchain->GetEntry(event_idx);
  auto primary   = event->GetPrimaryTrack();
  if (primary == nullptr) {
    return 0;
  }
  auto pos           = primary->Position[0];
  auto dir           = primary->MomentumDirection[0];
  double first_outer = 1e10;
  double first_inner = 1e10;
  TVector3 first_outer_pos;
  TVector3 first_inner_pos;
  for (uint k=0; k<primary->EnergyDeposition.size();k++) { 
    auto vid = primary->VolumeId[k];    
    if (vid > 200000000) {
      continue;
    }
    auto hid = paddle_vid_hid_map.at(vid); 
    if (hid < 60) {
      if (primary->GlobalTime[k] < first_inner) {
        first_inner     = primary->GlobalTime[k];
        first_inner_pos = primary->Position[k];
      }
    } else {
      if (primary->GlobalTime[k] < first_outer) {
        first_outer     = primary->GlobalTime[k];
        first_outer_pos = primary->Position[k];
      }
    }  
  }
  auto beta = (first_inner_pos-first_outer_pos).Mag()/(first_inner-first_outer)*1e6/299792458;
  return (f32)beta;
}

//------------------------------------------------------------------------
    
auto g::SDRootReader::get_primary(u64 event_idx)            -> Option<g::Tracklet> {
  if (is_mc) {
    mc_tchain->GetEntry(event_idx);
    auto primary   = mc_event->get_primary();
    return Some(primary); 
  } else {
    tchain->GetEntry(event_idx);
    auto primary   = event->get_primary();
    return primary;
  }
}

//------------------------------------------------------------------------

auto g::SDRootReader::get_mc_tracks_edeps(u64 event_idx)     -> Vec<f32> {
  Vec<f32> edeps;
  if (is_mc) {
    mc_tchain->GetEntry(event_idx);
    for (auto const &t : mc_event->tracks_) {
      for (auto const &edep : t->EnergyDeposition) {
        edeps.push_back((f32)edep);
      } 
    }
    return edeps;
  } 
  return edeps;
}

//------------------------------------------------------------------------

auto g::SDRootReader::get_mc_tracks_edeps_tof(u64 event_id)     -> Vec<f32> {
  Vec<f32> edeps;
  if (is_mc) {
    mc_tchain->GetEntry(event_id);
    for (auto const &t : mc_event->tracks_) {
      for (usize idx = 0; idx<t->VolumeId.size(); idx++) {
        if (t->VolumeId[idx] < 200000000) { 
          edeps.push_back((f32)t->EnergyDeposition[idx]);
        }
      } 
    }
    return edeps;
  } 
  return edeps;
}

//------------------------------------------------------------------------

auto g::SDRootReader::get_mc_tracks_edeps_trk(u64 event_id)     -> Vec<f32> {
  Vec<f32> edeps;
  if (is_mc) {
    mc_tchain->GetEntry(event_id);
    for (auto const &t : mc_event->tracks_) {
      for (usize idx = 0; idx<t->VolumeId.size(); idx++) {
        if (t->VolumeId[idx] >= 200000000) { 
          edeps.push_back((f32)t->EnergyDeposition[idx]);
        }
      } 
    }
    return edeps;
  } 
  return edeps;
}

//========================================================================

g::SDRootWriter::SDRootWriter(std::string fname, std::string geo_file,  std::string file_mode, bool write_sim ) {
  // databases
  pmap     = g::get_tofpaddles();
  std::cout << "-> Loaded " << pmap.size() << " TofPaddles!" << std::endl;
  smap     = g::get_trackerstrips();
  //auto hgmap       = Gaps::get_rb_id_paddles();
  lgmap    = g::get_dsi_j_paddles();
  std::cout << "-> Loaded " << lgmap.size() << " Dsi/J -> Pid conversions!" << std::endl;
  // setup root ouput file
  filename            = fname;
  if (geo_file != "") {
    TFileMerger merger;
    merger.OutputFile(fname.c_str(), file_mode.c_str()); 
    merger.AddFile(geo_file.c_str());
    merger.Merge();
    file_mode = "UPDATE";
    output_file = new TFile(fname.c_str(), file_mode.c_str());
  } else {
    output_file = new TFile(fname.c_str(), file_mode.c_str());
  }
  //tchain              = new TChain("TreeRec");
  event               = new CEventRec();
  rawtrk              = new Crane::Calibration::CRawTrk();
  rawtof              = new Crane::Calibration::CRawTof();
  rawhd               = new Crane::Calibration::CRawHeader();
  reco_tree           = new TTree("TreeRec", "TreeRec");
  reco_tree->Branch("Rec", &event);
  raw_tree         = new TTree("TreeRaw", "TreeRaw");
  raw_tree->Branch("Header",&rawhd);
  raw_tree->Branch("Trk", &rawtrk);
  raw_tree->Branch("Tof", &rawtof);
  bool write_par_tree = true;
  if (write_par_tree) {
    par_tree          = new TTree("SimulationParameterTree", "SimulationParameterTree");
    par               = new GSimulationParameter();
    par_tree->Branch("SimulationParameter", &par);
  }
  if (write_sim) {
    mc_event = new CEventMc();
    // ROOT Tree, not a real mctree, to avoid 
    // further confusion, let's call it sim_tree 
    sim_tree  = new TTree("TreeMc", "TreeMc");
    sim_tree->Branch("Mc", &mc_event);
  }
}

gondola::SDRootWriter::~SDRootWriter() {
  // all entries are filled in add_event. Filling here again 
  // would duplicate the last event of each file
  reco_tree->Write();
  if (raw_tree != nullptr) {
    raw_tree->Write();
  }
  if (sim_tree != nullptr) {
    // all entries are filled in add_mc_event
    sim_tree->Write();
    mc_event->reset();
  }
  // one entry per file, with the primary histograms of all 
  // events added with add_sim_primary
  if (par_tree != nullptr) {
    par_tree->Fill();
    par_tree->Write();
  }
  output_file->Close();
  delete output_file;  
  // somehow cleaning up causes problems with python... 
  //if (output_file != nullptr) { 
  //  delete output_file;
  //}
  //if (reco_tree != nullptr) {
  //  delete reco_tree;
  //} 
  //if (event != nullptr) {
  //  delete event;
  //}
}

auto gondola::SDRootWriter::write_sdpar(u32 run_id, std::string hostname, std::string crane_version) -> void {
  // the tree is filled and written in the destructor
  par->runId_ = run_id;
  par->productionHostName_ = hostname;
  par->CraneVersion_.push_back(crane_version);
}

//------------------------------------------------------------------------

auto gondola::SDRootWriter::add_sim_primary(const McEvent& ev, bool triggered) -> void {
  const f64 beta    = ev.primary.beta;
  const f64 kin_e   = ev.primary.get_vertex()->energy;
  const f64 cos_zen = ev.primary.vertex_mom_z;
  par->fill_generated(beta, kin_e, cos_zen);
  if (triggered) {
    par->fill_triggered(beta, kin_e, cos_zen);
  }
  par->nEventsGeneratedPerFile_++;
}

//------------------------------------------------------------------------

auto gondola::SDRootWriter::set_generator(bool uniform_beta, f64 beta_low, f64 beta_high,
                                          bool uniform_log_e, f64 log_e_low, f64 log_e_high,
                                          bool isotropic, i32 up_down_all, f64 limit, i32 pdg,
                                          bool use_trigger, bool use_digitizer,
                                          std::string g4_version, std::string physics_list) -> void {
  par->UniformBeta               = uniform_beta;
  par->PrimaryBetaLow            = beta_low;
  par->PrimaryBetaHigh           = beta_high;
  par->UniformLogE               = uniform_log_e;
  par->PrimaryLogELow            = log_e_low;
  par->PrimaryLogEHigh           = log_e_high;
  par->PrimaryIsotropic          = isotropic;
  par->PrimaryIsotropicUpDownAll = up_down_all;
  par->PrimaryLimit              = limit;
  par->PrimaryPdg                = pdg;
  par->UseTrigger                = use_trigger;
  par->UseDigitizer              = use_digitizer;
  par->G4version_                = g4_version;
  par->physicsList_              = physics_list;
}


auto gondola::SDRootWriter::add_event(TelemetryEvent* ev, TelemetryEvent* ev_for_raw, u8 packet_type, f64 gcutime, bool use_corrected_edep, f32 tracker_cut) -> void {
   if (ev == nullptr) {
     std::cout << "Received nullptr for TelemetryEvent!" << std::endl;
     return;
   }
   // clear all fields
   *event  = std::move(CEventRec());
   *rawtrk = std::move(Crane::Calibration::CRawTrk());
   *rawtof = std::move(Crane::Calibration::CRawTof());
   *rawhd  = std::move(Crane::Calibration::CRawHeader());
   // first need to fill the raw tree, since it is referenced
   // later on 
   if (ev_for_raw != nullptr) {
     // the reason why this is relevant at all are the pulses
     // from the pulser, whcih midght be excluded from the 
     // other event (ev)
     rawtrk->fill_from_telemetry(ev_for_raw);
   } else {
     rawtrk->fill_from_telemetry(ev);
   }
   rawtof->fill_from_telemetry(ev);
   event ->fill_from_telemetry(ev, packet_type, gcutime, pmap,
                               smap, lgmap, rawtrk, rawtof, true,
                               tracker_cut, use_corrected_edep);
   //std::cout << rawtrk->pretty_print() << std::endl;
   reco_tree->Fill();  
   raw_tree->Fill();
   //reco_tree->Write();
   //raw_tree->Write();
   //std::cout << event->pretty_print() << std::endl;
}

//------------------------------------------------------------------------

auto gondola::SDRootWriter::add_mc_event(const McEvent& ev) -> void { 
  if (mc_event == nullptr) {
    std::cerr << "[ERROR] SDRootWriter has been created with write_sim = false, unable to write McEvent!" << std::endl;
    return;
  }
  constexpr f64 NaN = std::numeric_limits<f64>::quiet_NaN();
  mc_event->reset(); 
  // event and primary properties 
  // (crane only sets the event number, eventId_ and eventTime_
  // remain 0)
  mc_event->runNumber_   = ev.run_id;
  mc_event->eventNumber_ = ev.event_id;
  auto vertex            = ev.primary.get_vertex();
  mc_event->primaryPdg_  = ev.primary.pdg; 
  mc_event->primaryBetaGenerated_          = ev.primary.beta;
  mc_event->primaryKineticEnergyGenerated_ = vertex->energy;
  mc_event->primaryMomentumDirectionGenerated_ = TVector3(ev.primary.vertex_mom_x,
                                                          ev.primary.vertex_mom_y,
                                                          ev.primary.vertex_mom_z);
  mc_event->primaryPosition_ = TVector3(vertex->x, vertex->y, vertex->z);
  mc_event->primaryTime_     = vertex->time;
  // the simulation fills the stop also for primaries which
  // do not stop (with NaN position). Crane's stopping volume 
  // is only set if the primary stops, otherwise it is 
  // NANInt (= 0)
  auto stop = ev.primary.get_stop_raw();
  mc_event->primaryStoppingPosition_      = TVector3(stop->x, stop->y, stop->z);
  mc_event->primaryStoppingTime_          = stop->time;
  mc_event->primaryStoppingKineticEnergy_ = stop->energy;
  mc_event->primaryStoppingVolume_        = std::isnan(stop->x) ? 0 : stop->volume;
  // no per event seed in the simulation 
  mc_event->randomSeed_  = 0;

  // tracks - in the order of their first step, as in crane. 
  // The steps of a track keep their order.
  Vec<u32>          track_order = {};
  HashMap<u32,Vec<const McHit*>> track_hits = {};
  for (auto const &h : ev.hits) {
    if (!track_hits.contains(h.track_id)) {
      track_order.push_back(h.track_id);
      track_hits.insert({h.track_id, {}});
    }
    track_hits[h.track_id].push_back(&h);
  }
  bool found_primary = false;
  for (auto const &track_id : track_order) {
    auto sd_track   = new CTrackMc();
    auto const &thits = track_hits[track_id];
    for (auto const &h : thits) {
      sd_track->TrackId            = track_id; 
      sd_track->ParentId           = h->parent_id;
      sd_track->Pdg                = h->pdg; 
      sd_track->EnergyDeposition     .push_back(h->step_edep); 
      sd_track->StepLength           .push_back(h->step_len); 
      sd_track->PreKineticEnergy     .push_back(h->pre_kin_E);
      sd_track->KineticEnergy        .push_back(h->kin_E); 
      sd_track->GlobalTime           .push_back(h->glob_time); 
      sd_track->VolumeId             .push_back( h->volume_id); 
      sd_track->PreStepStatus        .push_back( h->pre_step_status);
      sd_track->PostStepStatus       .push_back( h->post_step_status);
      sd_track->Position             .push_back( TVector3(h->pos_x, h->pos_y, h->pos_z));
      sd_track->PositionResidual     .push_back( TVector3(0,0,0));
      sd_track->MomentumDirection    .push_back( TVector3(h->mom_x, h->mom_y, h->mom_z));
      sd_track->PreMomentumDirection .push_back( TVector3(h->pre_mom_x, h->pre_mom_y, h->pre_mom_z));
      sd_track->ColumnDensityUntilStep.push_back(NaN); 
      sd_track->VertexPosition      = TVector3(h->vertex_pos_x, h->vertex_pos_y, h->vertex_pos_z); 
      sd_track->VertexKineticEnergy = h->vertex_kin_E;
      sd_track->VertexVolumeId      = h->vertex_vol_id;
      // G4ProcessType counts 0,1,2,..., crane's CGeant4ProcessType
      // 0,10,20,... 
      if (h->process_type <= 12) {
        sd_track->ProcessType       = static_cast<CGeant4ProcessType>(10*h->process_type); 
      } else {
        sd_track->ProcessType       = CGeant4ProcessType::NOTDEFINED;
      }
    } 
    // as CEventMc::SetTrack
    if (sd_track->Pdg == mc_event->primaryPdg_ && sd_track->ParentId == 0) {
      sd_track->Primary = true;
      found_primary     = true;
    }
    mc_event->tracks_.push_back(sd_track);
  } 
  // crane adds a primary track without steps if the 
  // primary did not leave any steps (but only if there 
  // are any tracks)
  if (!found_primary && mc_event->tracks_.size() > 0) {
    auto sd_track                 = new CTrackMc();
    sd_track->Primary             = true;
    sd_track->TrackId             = 1;
    sd_track->VertexVolumeId      = 0;
    sd_track->VertexPosition      = mc_event->primaryPosition_;
    sd_track->LastVolumeId        = mc_event->primaryStoppingVolume_;
    sd_track->LastPosition        = mc_event->primaryStoppingPosition_;
    sd_track->ParentId            = 0;
    sd_track->Pdg                 = mc_event->primaryPdg_;
    sd_track->VertexKineticEnergy = mc_event->primaryKineticEnergyGenerated_;
    mc_event->tracks_.push_back(sd_track);
  }
  
  // the hits - one per volume, from all steps with energy 
  // deposition. Volumes are in the order of their first 
  // such step.
  // NOTE - crane's meanPosition_ is distorted due to a bug in 
  // SimpleDetEventAction::SaveRoot (see add_mc_event docs), 
  // here it is the energy weighted mean of the step positions.
  Vec<u32> volume_order = {};
  HashMap<u32,usize> volume_idx = {};
  Vec<f64>      edep_sum  = {};
  Vec<TVector3> pos_sum   = {};
  Vec<f64>      t_min     = {};
  Vec<f64>      t_max     = {};
  Vec<i32>      min_track = {};
  for (auto const &h : ev.hits) {
    if (!(h.step_edep > 0)) {
      continue;
    }
    if (!volume_idx.contains(h.volume_id)) {
      volume_idx.insert({h.volume_id, volume_order.size()});
      volume_order.push_back(h.volume_id);
      edep_sum .push_back(0);
      pos_sum  .push_back(TVector3(0,0,0));
      t_min    .push_back(std::numeric_limits<f64>::max());
      t_max    .push_back(std::numeric_limits<f64>::lowest());
      min_track.push_back(std::numeric_limits<i32>::max());
    }
    auto k        = volume_idx[h.volume_id];
    edep_sum[k]  += h.step_edep;
    pos_sum[k]   += h.step_edep*TVector3(h.pos_x, h.pos_y, h.pos_z);
    t_min[k]      = std::min(t_min[k], (f64)h.glob_time);
    t_max[k]      = std::max(t_max[k], (f64)h.glob_time);
    min_track[k]  = std::min(min_track[k], (i32)h.track_id);
  }
  for (usize k=0; k<volume_order.size(); k++) {
    mc_event->volumeId_             .push_back(volume_order[k]);
    mc_event->totalEnergyDeposition_.push_back(edep_sum[k]);
    mc_event->meanPosition_         .push_back(pos_sum[k]*(1.0/edep_sum[k]));
    mc_event->time_                 .push_back(0.5*(t_min[k] + t_max[k]));
    // as in crane, this is the smallest track id in the 
    // volume, not an index
    mc_event->hitTrackIndex_        .push_back(min_track[k]);
  }
  sim_tree->Fill();
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

auto CTrackMc::pretty_print() const -> std::string {
  std::string repr = "<CTrackMc>";
  return repr;
}

//------------------------------------------------------------------------

auto CTrackRec::pretty_print() const -> std::string {
  // FIXME - provide pointers
  auto vid_hid_map_tof    = gondola::get_vid_hid_map_tof();
  i32 pid = -1;
  if (vid_hid_map_tof.contains(VertexVolumeId)) {
    pid  = vid_hid_map_tof.at(VertexVolumeId);
  }
  auto vid_hid_map_tracker    = gondola::get_vid_hid_map_tracker();
  if (pid == -1) {
    if (vid_hid_map_tracker.contains(VertexVolumeId)) {
      pid  = vid_hid_map_tracker.at(VertexVolumeId);
    }
  }
  auto vpos = VertexPosition;
  auto lpos = LastPosition;
  std::string repr = "<CTrackRec (cpt layer)";
  repr += std::format("\n   Primary        : {}", Primary);
  repr += std::format("\n   TrackId        : {}", TrackId);
  repr += std::format("\n   VertexVolumeId : {}/[{}]", VertexVolumeId, pid);
  repr += std::format("\n   VertexPos X {:.2f} Y {:.2f} Z {:.2f}", vpos.X(), vpos.Y(), vpos.Z());
  if (vid_hid_map_tof.contains(LastVolumeId)) {
    pid = vid_hid_map_tof.at(LastVolumeId);
  } else {
    pid = vid_hid_map_tracker.at(LastVolumeId);
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
    if (vid_hid_map_tof.contains(VolumeId.at(k))) {
      pid = vid_hid_map_tof.at(VolumeId.at(k));
    } else {
      pid = vid_hid_map_tracker.at(VolumeId.at(k));
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

//========================================================================

auto CEventMc::pretty_print() const -> std::string {
  std::string repr = "<CEventMc";
  repr += std::format("\n n mc tracks : {}>", tracks_.size());
  return repr;
}

//------------------------------------------------------------------------

auto CEventMc::reset() -> void {
  for (auto t : tracks_) {
    delete t;
  }
  tracks_.clear();
  hitTrackIndex_        .clear();
  totalEnergyDeposition_.clear();
  meanPosition_         .clear();
  volumeId_             .clear();
  time_                 .clear();
  *this = CEventMc();
}

//------------------------------------------------------------------------

auto CEventMc::get_primary() const -> g::Tracklet {
  auto vertex          = std::make_shared<g::RecoHit>();
  vertex->x            = primaryPosition_.X();
  vertex->y            = primaryPosition_.Y();
  vertex->z            = primaryPosition_.Z(); 
  vertex->energy       = (f32)primaryKineticEnergyGenerated_; 
  g::Tracklet primary(vertex);
  primary.vertex_mom_x = primaryMomentumDirectionGenerated_.X(); 
  primary.vertex_mom_y = primaryMomentumDirectionGenerated_.Y(); 
  primary.vertex_mom_z = primaryMomentumDirectionGenerated_.Z();
  return primary;
}

//========================================================================

auto Crane::Calibration::CRawHeader::pretty_print() const -> std::string {
  std::string repr = "<CRawHeader (compat layer):";
  repr += std::format("\n  type      {}" ,type);
  repr += std::format("\n  timestamp {}" ,timestamp);
  repr += std::format("\n  counter   {}" ,counter);
  repr += std::format("\n  length    {}" ,length);
  repr += std::format("\n  systime   {}" ,systime);
  repr += std::format("\n  eventid   {}" ,eventid);      
  //      Vec<u64>  trk_eventtime;
  //      Vec<u64>  trk_eventid;
  //      Vec<u16>  trk_eventid_valid;
  //      Vec<u16>  trk_layer;
  //      Vec<u8>   tof_packettype;
  return repr;
} 

//------------------------------------------------------------------------

auto Crane::Calibration::CRawTrk::SetFlag(int bit, bool val ) -> void {
  std::bitset<8> bitset(flag);
  if(val) {
    bitset.set(bit);
  } else { 
    bitset.reset(bit);
  }
  flag = static_cast<unsigned char>(bitset.to_ulong());
}

auto Crane::Calibration::CRawTrk::fill_from_telemetry(gondola::TelemetryEvent* event) -> void {
  eventid  =  event->event_id; 
  // the event id valid flag is not included any more in merged event
  u8 eventid_valid = 0x3;// merged packets with not valid ID are rejected a priori
  SetFlag(1,(eventid_valid)&0x1);//event id valid flag
  SetFlag(2,(eventid_valid)&0x2);//event id valid flag
  SetFlag(3,true);//assert the merged flag
  SetFlag(6,true);//assert not-missing flag
  for (auto const &hit : event->trk_hits) {
    layer     .push_back(hit.layer);
    row       .push_back(hit.row);
    module    .push_back(hit.module);
    channel   .push_back(hit.channel);
    adcdata   .push_back((u16)hit.adc);
    eventtime .push_back(hit.oscillator);
    hindex    .push_back(-1);  
  }
}

auto Crane::Calibration::CRawTrk::pretty_print() const -> std::string {
  std::string repr = "<CRawTrk (gondola Compat Layer):";
  repr += std::format("\n  event id {}", eventid);
  repr += std::format("\n  flag     {}", flag);
  repr += std::format("\n -- {} TRK hits --", layer.size());
  for (usize k=0; k<layer.size();k++) {
    repr += std::format("\n -- layer {} // row {} // module {} // channel {}", layer.at(k), row.at(k), module.at(k), channel.at(k));
    repr += std::format("\n -- adc {} // osciallator {}", adcdata.at(k), eventtime.at(k));
    repr += std::format("\n -- hit index {}", hindex.at(k));
    repr += "\n .. .. .. .. .. ..";
  }
  return repr;
}

//------------------------------------------------------------------------

auto Crane::Calibration::CRawTof::fill_from_telemetry(gondola::TelemetryEvent* event) -> void {
  runid   = event->tof_event.run_id; 
  eventid = event->tof_event.event_id; 
  event_status = (u8)event->tof_event.status;
  hits    = CRawTofHits();

  for (auto const &h : event->tof_event.hits) {
    //trigger_th  .push_back(0);
    //timestamp48 .push_back(0);
    hits.paddle_id   .push_back(h.paddle_id);
    hits.base_a      .push_back(h.baseline_a);
    hits.base_b      .push_back(h.baseline_b);
    hits.base_a_rms  .push_back(h.baseline_a_rms);
    hits.base_b_rms  .push_back(h.baseline_b_rms);
    hits.phase       .push_back(h.get_phase_delay());
    hits.time_a      .push_back(h.get_time_a());
    hits.time_b      .push_back(h.get_time_b());
    hits.peak_a      .push_back(h.get_peak_a());
    hits.peak_b      .push_back(h.get_peak_b());
    hits.charge_a    .push_back(h.get_charge_a());
    hits.charge_b    .push_back(h.get_charge_b());
    //hits.charge_min_i.push_back(0);
    hits.x_0         .push_back(h.get_x_pos());
    hits.t_0         .push_back(h.event_t0);

    //hits.t_shift     .push_back(0);
    hits.hindex      .push_back(-1); // hit index inside CEventRec
  }
  trg = CRawTrigger();
  trg.mtb_link_ids   = event->tof_event.get_rb_link_ids();
  for (auto const &h : event->tof_event.get_trigger_hits() ) {
    trg.dsi            .push_back(std::get<0>(h));
    trg.j              .push_back(std::get<1>(h));
    trg.ch             .push_back(std::get<2>(h));
    trg.th             .push_back((int)std::get<3>(h));
  }
  for (auto const &t: event->tof_event.get_trigger_sources()) {
    trg.trigger_sources.push_back((int)t);
  }
  //trg.trigger_sources = event->tof_event.get_trigger_sources();
 //   trg.paddle_id      ;
 //   trigger_sources;
 // }
}

auto Crane::Calibration::CRawTof::pretty_print() const -> std::string {
  std::string repr = "<CRawTof (compat layer):";
  repr += std::format("\n  runid           {}", runid );                //MTB pkt
  repr += std::format("\n  eventid         {}", eventid );              //summary pkt
  repr += std::format("\n  event_status    {}", event_status );
  repr += std::format("\n  timestamp48     {}", timestamp48 );         //summary pkt
  repr += std::format("\n  timestamp       {}", timestamp );          //MTB pkt ???
  repr += std::format("\n  timestamp_gps48 {}", timestamp_gps48 );    //MTB pkt
  repr += std::format("\n  timestamp_abs48 {}>", timestamp_abs48 );    //MTB pkt
  repr += "\n-- -- -- -- Hits -- -- --\n";
  repr += hits.pretty_print(); 
  repr +=">";
  return repr;
}

//------------------------------------------------------------------------

auto Crane::Calibration::CRawTofHits::pretty_print() const -> std::string {
  std:: string repr = "<CRawTofHits (compat layer)";
  repr += std::format("\n Len trigger_th   : {}", trigger_th  .size()); 
  repr += std::format("\n Len timestamp48  : {}", timestamp48 .size()); 
  repr += std::format("\n Len paddle_id    : {}", paddle_id   .size()); 
  repr += std::format("\n Len base_a       : {}", base_a      .size()); 
  repr += std::format("\n Len base_b       : {}", base_b      .size()); 
  repr += std::format("\n Len base_a_rms   : {}", base_a_rms  .size()); 
  repr += std::format("\n Len base_b_rms   : {}", base_b_rms  .size()); 
  repr += std::format("\n Len phase        : {}", phase       .size()); 
  repr += std::format("\n Len time_a       : {}", time_a      .size()); 
  repr += std::format("\n Len time_b       : {}", time_b      .size()); 
  repr += std::format("\n Len peak_a       : {}", peak_a      .size()); 
  repr += std::format("\n Len peak_b       : {}", peak_b      .size()); 
  repr += std::format("\n Len charge_a     : {}", charge_a    .size()); 
  repr += std::format("\n Len charge_b     : {}", charge_b    .size()); 
  repr += std::format("\n Len charge_min_i : {}", charge_min_i.size()); 
  repr += std::format("\n Len x_0          : {}", x_0         .size()); 
  repr += std::format("\n Len t_0          : {}", t_0         .size()); 
  repr += std::format("\n Len t_shift      : {}", t_shift     .size()); 
  repr += std::format("\n Len hindex       : {}", hindex      .size()); 
  repr += "\n  -- -- values --";
  repr += "\n  hit index  : ";
  for (auto const &h : hindex) { 
    repr += std::format("{} ", h);
  }
  repr += ">";
  return repr;
}

//------------------------------------------------------------------------

auto CEventRec::get_tof_energies() const -> Vec<f32> {
  auto energies = Vec<f32>();
  for (auto const &h : hitseries_) {
    if (h.volume_id_ < 200000000) {
      energies.push_back(h.energydep_);
    }
  }
  return energies;
};

//------------------------------------------------------------------------

auto CEventRec::get_trk_energies() const -> Vec<f32> {
  auto energies = Vec<f32>();
  for (auto const &h : hitseries_) {
    if (h.volume_id_ >= 200000000) {
      energies.push_back(h.energydep_);
    }
  }
  return energies;
};

//------------------------------------------------------------------------

auto CEventRec::has_primary() const -> bool {
  if (registeredRecos_.size() == 0){
    return false;
  }  
  if (Tracks.find(activeReco_) ==  Tracks.end()) {
    return false;
  }
  for (CTrackRec* track : Tracks.at(activeReco_)) {
    if (track->Primary) {
      return true;
    }
  }
  return false;
}

//------------------------------------------------------------------------

auto CEventRec::get_primary() const -> Option<g::Tracklet> {
  
  if (!has_primary()) {
    return None;
  }
  auto vertex          = std::make_shared<g::RecoHit>();
  auto cev_prim        = GetPrimaryTrack(); 

  if (cev_prim != nullptr) {
    vertex->x         = cev_prim->VertexPosition.X();
    vertex->y         = cev_prim->VertexPosition.Y();
    vertex->z         = cev_prim->VertexPosition.Z();
  }
  
  g::Tracklet primary(vertex);
  if (cev_prim != nullptr) { 
    auto cev_prim_edep = cev_prim->EnergyDeposition; 
    auto cev_prim_vid  = cev_prim->VolumeId; 
    for (usize k=0;k<cev_prim_edep.size();k++) {
      primary.edeps.push_back(std::make_tuple(cev_prim_vid[k], cev_prim_edep[k]));
    }
    for (auto const &k : cev_prim->ColumnDensityUntilStep) {
      primary.coldens.push_back(k);
    }
  }
  
  if (!primaryMomentumDirection_.contains(activeReco_)) {
    std::cerr << std::format("Unable to access reconstruction results [momentum direction] for reco with name '{}'", activeReco_) << std::endl;
    return Some(primary);
  }
  
  auto dir             = primaryMomentumDirection_.at(activeReco_);
  primary.vertex_mom_x = dir.X();
  primary.vertex_mom_y = dir.Y();
  primary.vertex_mom_z = dir.Z(); 
  
  auto stop            = std::make_shared<g::RecoHit>();
  
  if (!primaryStoppingPosition_.contains(activeReco_)) {
    std::cerr << std::format("Unable to access reconstruction results [stopping position] for reco with name '{}'", activeReco_) << std::endl;
    return Some(primary);
  }
  auto psp             = primaryStoppingPosition_.at(activeReco_); 
  stop->x              = psp.X();
  stop->y              = psp.Y();
  stop->z              = psp.Z();
  

  if (!primaryStoppingTime_.contains(activeReco_)) {
    std::cerr << std::format("Unable to access reconstruction results [stopping time] for reco with name '{}'", activeReco_) << std::endl;
    return Some(primary);
  } 
  stop->time           = primaryStoppingTime_.at(activeReco_);
  
  if (!primaryStoppingVolume_.contains(activeReco_)) {
    std::cerr << std::format("Unable to access reconstruction results [stopping volume] for reco with name '{}'", activeReco_) << std::endl;
    return Some(primary);
  } 
  stop->volume         = primaryStoppingVolume_.at(activeReco_);
  
  // -- assign to priamry
  //primary.vertex     = vertex;
  //primary.stop       = stop;
  primary.set_stop(stop); 
  if (!Chi2.contains(activeReco_)) {
    std::cerr << std::format("Unable to access reconstruction results [chi2] for reco with name '{}'", activeReco_) << std::endl;
    return Some(primary);
  } 
  primary.gof          = Some(Chi2.at(activeReco_));
  
  if (!primaryBeta_.contains(activeReco_)) {
    std::cerr << std::format("Unable to access reconstruction results [beta] for reco with name '{}'", activeReco_) << std::endl;
    return Some(primary);
  } 
  primary.beta         = primaryBeta_.at(activeReco_);
  
  if (!primaryBetaError_.contains(activeReco_)) {
    std::cerr << std::format("Unable to access reconstruction results [beta error] for reco with name '{}'", activeReco_) << std::endl;
    return Some(primary);
  } 
  primary.beta_err     = primaryBetaError_.at(activeReco_);
  return Some(primary);
}
    
//------------------------------------------------------------------------

auto CEventRec::fill_from_telemetry(g::TelemetryEvent* event,
                                    u8  packet_type,
                                    f64 gcutime,
                                    const g::TofPaddleMap& pmap,
                                    const g::TrkStripMap& smap,
                                    const g::DsiJChnPaddleIdMap& lgmap,
                                    cb::CRawTrk* raw_trk,
                                    cb::CRawTof* raw_tof,
                                    const bool apply_elena_cut,  
                                    const double mev_cut,
                                    const bool use_corrected_edep) -> void {
  primaryBetaGenerated_ = NAN;
  //primaryMomentumDirectionGenerated_;
  primaryKineticEnergyGenerated_ = NAN;
  runNumber_         = event->tof_event.run_id;
  subRunNumber_      = 0;
  eventNumber_       = event->tof_event.event_id;
  eventId_           = event->tof_event.event_id;
  eventTime_         = (f64) (event->tof_event.get_timestamp48())*1e-5; 
  gps_time_lower_    = event->tof_event.timestamp32;
  gps_time_upper_    = event->tof_event.timestamp16; 
  int TofQualityId   = 1<<8;
  int Quality        = TofQualityId + (int)event->tof_event.status;   
  event_quality.push_back(Quality);  
  // trigger sources 
  for (auto const &src : event->tof_event.get_trigger_sources()) {
    trigger_sources.push_back((u8)src);
  }
  auto trigger_pids = event->tof_event.get_trigger_pids(lgmap);
  for (auto const& pid : trigger_pids) {
    trigger_vids.push_back(pmap.at(pid).volume_id);
  }
  Vec<g::TofHit> hits_for_recevent = {};
  usize hit_index = 0;
  for (auto const &hit : event->tof_event.hits) { // 
    hit_index += 1;
    if (apply_elena_cut && !elena_cut(hit)) {
      continue;
    }
    // we have to fix the hit index in the raw tree 
    // this works, because the order of hits in 
    // tof_event->hits is the same as when we used 
    // it in CRawTof;::fill_from_telemetry
    if (raw_tof != nullptr) {
      raw_tof->hits.hindex[hit_index]= hits_for_recevent.size() -1;
    }
    hits_for_recevent.push_back(hit);
  }
  for (auto const &hit : hits_for_recevent) {
    auto pdl = pmap.at((int)hit.paddle_id);
    auto pr  = pdl.get_principal();
    f32 x0   = hit.get_x_pos();
    //std::cout << "Paddle has length! " << hit.paddle_len << " !" << std::endl; 
    GRecoHit reco_hit;
    f32 pr_x = 10*(pdl.global_pos_x_l0_B - pdl.global_pos_x_l0_A)/(10*pdl.length);
    f32 pr_y = 10*(pdl.global_pos_y_l0_B - pdl.global_pos_y_l0_A)/(10*pdl.length);
    f32 pr_z = 10*(pdl.global_pos_z_l0_B - pdl.global_pos_z_l0_A)/(10*pdl.length);
    
    f32 x = 10*pdl.global_pos_x_l0_A + pr_x*x0;
    f32 y = 10*pdl.global_pos_y_l0_A + pr_y*x0;
    f32 z = 10*pdl.global_pos_z_l0_A + pr_z*x0;
    auto pos = TVector3(x,y,z);
    //std::cout << std::format("Paddle len {} ID {}", pdl.length, pdl.paddle_id) << std::endl;
    //std::cout << std::format("Hit X0 {} X {} Y {} Z {}",x0, x,y,z) << std::endl;
    reco_hit.volume_id_     = pdl.volume_id ;
    reco_hit.hit_position_  = pos           ;
    //reco_hit.SetTime    ( hit.get_t0() - first_time);//scaled to first hit
    reco_hit.hit_time_      = hit.event_t0  ;
    // change in Aug 2026 - new formulat for energy deposition
    if (use_corrected_edep) {
        reco_hit.energydep_     = hit.edep_corrected;
    } else { 
        reco_hit.energydep_     = hit.get_edep();
    }
    //std::cout << hit << std::endl;
    //std::cout << reco_hit.pretty_print() << std::endl;
    hitseries_.push_back(reco_hit); 
  }
  // do the same thing with the hit index again, 
  // this time for the tracker
  hit_index = 0;
  for (auto const &hit : event->trk_hits) {
    hit_index += 1;
    if (hit.energy < mev_cut) {
      continue;
    }
    GRecoHit reco_hit;
    //auto pos = TVector3(x,y,z);
    //std::cout << std::format("Paddle len {} ID {}", pdl.length, pdl.paddle_id) << std::endl;
    //std::cout << std::format("Hit X0 {} X {} Y {} Z {}",x0, x,y,z) << std::endl;
    auto strp = smap.at((int)hit.get_strip_id());
    f32 x = 10*strp.global_pos_x_l0;
    f32 y = 10*strp.global_pos_y_l0;
    f32 z = 10*strp.global_pos_z_l0;
    auto pos = TVector3(x,y,z);
     
    reco_hit.volume_id_     = strp.volume_id ;
    reco_hit.hit_time_      = NAN;
    reco_hit.hit_position_  = pos           ;
    //reco_hit.SetTime    ( hit.get_t0() - first_time);//scaled to first hit
    reco_hit.energydep_     = hit.energy;
    hitseries_.push_back(reco_hit); 
    if (raw_trk != nullptr) {
      raw_trk->hindex[hit_index] = hitseries_.size() - 1;
    }
  } 
  PacketType = packet_type;
  // FIX of the event time
  if(abs(eventTime_-(long int)gcutime) > 3600) {
    eventTime_ = (long int)gcutime;
  }

  //rawtof.event_status = (u8)event.status;
  //rawtof.eventid      = event.event_id;
  //rawtof.runid        = event.run_id;
  ////rawtof.timestamp_abs48 = timestamp_abs;
  //// FIXME - this is now only a 32bit timestamp
  //rawtof.timestamp_gps48 = event.get_timestamp48();
  //// trigger hits 
  //rawtof.trg.mtb_link_ids = event.get_rb_link_ids();
  ////usize nhits = event.hits.size();
  //for (auto const &hit : event.get_trigger_hits()) {
  //  u8 dsi = std::get<0>(hit);
  //  u8 j   = std::get<1>(hit);
  //  u8 ch  = std::get<2>(hit);
  //  u8 thr = (u8)std::get<3>(hit);
  //  rawtof.trg.dsi.push_back(dsi);
  //  rawtof.trg.j  .push_back(j);
  //  rawtof.trg.ch .push_back(ch);
  //  rawtof.trg.th .push_back(thr);
  //  if (!lgmap.contains(dsi)) {
  //    spdlog::error("Can not find DSI {} in LG map!", dsi);
  //    continue;
  //  } 
  //  if (!lgmap.at(dsi).contains(j)) {
  //    spdlog::error("Can not find J {} for DSI {} in LG map!", j, dsi);
  //    continue;
  //  }
  //  if (!lgmap.at(dsi).at(j).contains(ch)) {
  //    spdlog::error("Can not find Ch {} for DSI {} J {} in LG map!", ch, dsi, j);
  //    continue;
  //  }
  //  rawtof.trg.paddle_id.push_back(lgmap.at(dsi).at(j).at(ch));
  //}
  //
  //// trigger sources 
  //for (auto const &src : event.get_trigger_sources()) {
  //  rawtof.trg.trigger_sources.push_back((u8)src);
  //}
  //
  //auto event_quality = Vec<int>();
  //int TofQualityId = 1<<8;
  //int Quality = TofQualityId + (int)event.status;   
  //event_quality.push_back(Quality);  
  //recevent.SetEventQuality(event_quality);
  //
  //recevent.SetTriggerSources (rawtof.trg.trigger_sources );
  //Vec<u32> trigger_volume = {};
  //for (auto const& pid : rawtof.trg.paddle_id) {
  //  auto pdl = paddle_map.at(pid);
  //  trigger_volume.push_back(pdl.volume_id);
  //}
  //recevent.SetTriggerVolumeId(trigger_volume);  
  //
  //// TofHits (HG) 
  //Vec<TofHit> hits_for_recevent = {};
  //for (auto const &hit : event.hits) { // 
  //  rawtof.hits.phase      .push_back(hit.phase);
  //  rawtof.hits.base_a     .push_back(hit.baseline_a ); 
  //  rawtof.hits.base_b     .push_back(hit.baseline_b ); 
  //  rawtof.hits.base_a_rms .push_back(hit.baseline_a_rms ); 
  //  rawtof.hits.base_b_rms .push_back(hit.baseline_b_rms ); 
  //  rawtof.hits.time_a     .push_back(hit.get_time_a() ); 
  //  rawtof.hits.time_b     .push_back(hit.get_time_b() ); 
  //  rawtof.hits.peak_a     .push_back(hit.get_peak_a() ); 
  //  rawtof.hits.peak_b     .push_back(hit.get_peak_b() ); 
  //  rawtof.hits.charge_a   .push_back(hit.get_charge_a() ); 
  //  rawtof.hits.charge_b   .push_back(hit.get_charge_b()); 
  //  rawtof.hits.paddle_id  .push_back(hit.paddle_id);
  //  rawtof.hits.t_0        .push_back(hit.event_t0);
  //  rawtof.hits.x_0        .push_back(hit.get_x_pos());
  //  if (apply_elena_cut) {
  //    if (elena_cut(hit)) {
  //      hits_for_recevent.push_back(hit);
  //    }
  //  } else {
  //    hits_for_recevent.push_back(hit);
  //  }
  //}
  //if (hits_for_recevent.size() > 0) {
  //  for (auto const &hit : hits_for_recevent) {
  //    auto pdl = paddle_map.at(hit.paddle_id);
  //    auto pr  = pdl.get_principal();
  //    f32 x0   = hit.get_x_pos();
  //     
  //    GRecoHit reco_hit;
  //    f32 pr_x = 10*(pdl.global_pos_x_l0_B - pdl.global_pos_x_l0_A)/(10*pdl.length);
  //    f32 pr_y = 10*(pdl.global_pos_y_l0_B - pdl.global_pos_y_l0_A)/(10*pdl.length);
  //    f32 pr_z = 10*(pdl.global_pos_z_l0_B - pdl.global_pos_z_l0_A)/(10*pdl.length);
  //    
  //    f32 x = 10*pdl.global_pos_x_l0_A + pr_x*x0;
  //    f32 y = 10*pdl.global_pos_y_l0_A + pr_y*x0;
  //    f32 z = 10*pdl.global_pos_z_l0_A + pr_z*x0;
  //    auto pos = TVector3(x,y,z);
  //    //std::cout << std::format("Paddle len {} ID {}", pdl.length, pdl.paddle_id) << std::endl;
  //    //std::cout << std::format("Hit X0 {} X {} Y {} Z {}",x0, x,y,z) << std::endl;
  //    reco_hit.SetVolumeId( pdl.volume_id          );
  //    reco_hit.SetPosition( pos         );
  //    //reco_hit.SetTime    ( hit.get_t0() - first_time);//scaled to first hit
  //    reco_hit.SetTime( hit.event_t0 );
  //    reco_hit.SetTotalEnergyDeposition( hit.get_edep());
  //    recevent.AddHit(reco_hit); 
  //  }
  //  recevent.SetEventTime  (double(event.get_timestamp48())*1e-5); 
  //  recevent.SetGPSTime(event.timestamp32, event.timestamp16);
  //}
}

//------------------------------------------------------------------------

auto CEventRec::GetGPSTime() const -> f64 {
  u64 gps_time48 = 0x273000000000000 | (u64) gps_time_upper_ << 32 | (u64) gps_time_lower_;
  // this timestamp has 10ns precision 
  return (f64)gps_time48 * 1e-8;
}

//------------------------------------------------------------------------

auto CEventRec::ListAvailableReconstructions() const -> Vec<std::string> {
  return registeredRecos_;
}

//------------------------------------------------------------------------

auto CEventRec::GetPrimaryTrack() const -> CTrackRec* {
  if (!has_primary()) {
    return nullptr;
  }
  for (CTrackRec* track : Tracks.at(activeReco_)) {
    if (track->Primary) {
      return track;
    }
  }
  return nullptr;
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

//------------------------------------------------------------------------

GSimulationParameter::GSimulationParameter() {
  // binning and names as in crane's GSimulationParameter
  HPrimaryBetaGenerated       = TH1D("HPrimaryBetaGenerated", "", 100, 0, 1);
  HPrimaryBetaTriggered       = TH1D("HPrimaryBetaTriggered", "", 100, 0, 1);
  HPrimaryCosZenBetaGenerated = TH2D("HPrimaryCosZenBetaGenerated", "", 101, -1, 1, 100, 0, 1);
  HPrimaryCosZenBetaTriggered = TH2D("HPrimaryCosZenBetaTriggered", "", 101, -1, 1, 100, 0, 1);
  HPrimaryCosZenLogEGenerated = TH2D("HPrimaryCosZenLogEGenerated", "", 101, -1, 1, 80, -2, 6);
  HPrimaryCosZenLogETriggered = TH2D("HPrimaryCosZenLogETriggered", "", 101, -1, 1, 80, -2, 6);
  // the histograms are members, the file must not own them
  for (TH1* h : std::initializer_list<TH1*>{&HPrimaryBetaGenerated, &HPrimaryBetaTriggered,
                                            &HPrimaryCosZenBetaGenerated, &HPrimaryCosZenBetaTriggered,
                                            &HPrimaryCosZenLogEGenerated, &HPrimaryCosZenLogETriggered}) {
    h->SetDirectory(nullptr);
  }
  // defaults as in crane's GSimulationParameter::Reset
  runId_                    = 0;
  subRunId_                 = 0;
  randomSeed_               = 0;
  triggerLevel_             = 0;
  UseTrigger                = false;
  UseDigitizer              = false;
  UniformBeta               = false;
  PrimaryBetaLow            = -1;
  PrimaryBetaHigh           = -1;
  PrimaryIsotropic          = false;
  PrimaryIsotropicUpDownAll = -1;
  PrimaryLimit              = -1;
  PrimaryFile               = false;
  UniformLogE               = false;
  PrimaryLogELow            = -1;
  PrimaryLogEHigh           = -1;
  PrimaryPdg                = 0;
  nEventsGeneratedPerFile_  = 0;
}

auto GSimulationParameter::fill_generated(f64 beta, f64 kinetic_energy, f64 cos_zen) -> void {
  HPrimaryBetaGenerated.Fill(beta);
  HPrimaryCosZenBetaGenerated.Fill(cos_zen, beta);
  HPrimaryCosZenLogEGenerated.Fill(cos_zen, std::log10(kinetic_energy));
}

auto GSimulationParameter::fill_triggered(f64 beta, f64 kinetic_energy, f64 cos_zen) -> void {
  HPrimaryBetaTriggered.Fill(beta);
  HPrimaryCosZenBetaTriggered.Fill(cos_zen, beta);
  HPrimaryCosZenLogETriggered.Fill(cos_zen, std::log10(kinetic_energy));
}

GSimulationParameter::~GSimulationParameter() {
}

//------------------------------------------------------------------------

//void gondola::TrackerEnergyResponseFunction(double adc, uint layer, uint row, uint module, uint channel) -> double {
//  double digi_V = adc;
//  double digi_ekin;
//  TF1 ftf;
//  TGraph gtf;
//  bool found = GetTransferFunction(layer, row, module, channel, ftf, gtf);
//  if (found) {
//    //
//    // determines the function range
//    //
//    double adc_min, adc_max;
//    double V_min, V_max;
//    if (func_tf_invert_) {
//      adc_min = ftf.GetMinimum();
//      adc_max = ftf.GetMaximum();
//      V_min   = ftf.GetXmin();
//      V_max   = ftf.GetXmax();
//    } else {
//      adc_min = ftf.GetXmin();
//      adc_max = ftf.GetXmax();
//      V_min = ftf.GetMinimum();
//      V_max = ftf.GetMinimum();
//    }
//    if (adc < 0) {
//      adc = 0.;
//    }
//    // we assume the transfer function cross the origin by construction
//    if (adc < adc_min) {
//      digi_V = V_min;
//    } else {
//      if (adc >= adc_min && adc <= adc_max) {// within function range
//        // if the discrete ADC value is inside the TF1 codomain
//        // convert to mV using the TF1
//        if (func_tf_invert_) {
//          digi_V = ftf.GetX(adc);
//        } else {
//          digi_V = ftf.Eval(adc);
//        } else {
//          // else if the discrete ADC value is outside the TF1 codomain
//          // convert to mV using the eval of the anti-TGraph of the transfer funciton
//          digi_V = gtf.Eval(adc);
//        }
//      } else {
//        digi_V = 0.; /// FIX do we want to apply a nominal conversion??
//    }
//  //#define mV2keV 0.841// mV to keV
//  f32 mV2keV = 0.841;
//  digi_ekin = digi_V * mV2keV; // mV to keV
//  return digi_ekin / 1000.; // keV to MeV
//}


#endif
