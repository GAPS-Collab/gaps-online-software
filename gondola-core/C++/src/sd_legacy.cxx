#ifdef BUILD_CXX_WITH_ROOT

#include <iostream>

#include "TChain.h"
#include "TFileMerger.h"

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
  rawtrk              = new Crane::Calibration::CRawTrk()   ;
  rawtof              = new Crane::Calibration::CRawTof()   ;
  rawhd               = new Crane::Calibration::CRawHeader();
  raw_tree            = new TChain("TreeRaw");
  raw_tree->Add(fname.c_str());
  raw_tree->SetBranchAddress("Trk", &rawtrk); 
  raw_tree->SetBranchAddress("Tof", &rawtof);
}

gondola::SDRootReader::~SDRootReader() {
  delete event;
  delete tchain;
}

auto gondola::SDRootReader::get_event(u64 event_idx) -> void {
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

auto gondola::SDRootReader::get_event_tof_energies(u64 event_idx) -> Vec<f32> {
  tchain->GetEntry(event_idx); 
  return event->get_tof_energies();
  //std::cout << event->pretty_print() << std::endl;
}

auto gondola::SDRootReader::get_event_trk_energies(u64 event_idx) -> Vec<f32> {
  tchain->GetEntry(event_idx); 
  return event->get_trk_energies();
  //std::cout << event->pretty_print() << std::endl;
}

//------------------------------------------------------------------------

gondola::SDRootWriter::SDRootWriter(std::string fname, std::string geo_file,  std::string file_mode) {
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
  raw_tree->Branch("Tof", &rawtof);
  //tchain->Add(fname.c_str());
  //tchain->SetBranchAddress("Rec", &event);
  //nevents_total       = tchain->GetEntries();
}

gondola::SDRootWriter::~SDRootWriter() {
  reco_tree->Fill();
  reco_tree->Write();
  if (raw_tree != nullptr) {
    raw_tree->Fill();
    raw_tree->Write();
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
  if (par == nullptr) {
    par = new GSimulationParameter();
  }
  if (par_tree == nullptr) {
    par_tree = new TTree("GSimulationParameter", "GSimulationParameter");
  }
  par->runId_ = run_id;
  par->productionHostName_ = hostname;
  par->CraneVersion_.push_back(crane_version);
  par_tree->Fill();
  par_tree->Write();
}


auto gondola::SDRootWriter::add_event(TelemetryEvent* ev, u8 packet_type, f64 gcutime) -> void {
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
   rawtrk->fill_from_telemetry(ev);
   rawtof->fill_from_telemetry(ev);
   event ->fill_from_telemetry(ev, packet_type, gcutime, pmap, smap, lgmap, rawtrk, rawtof, true, 0.4);
   //std::cout << rawtrk->pretty_print() << std::endl;
   reco_tree->Fill();  
   raw_tree->Fill();
   //reco_tree->Write();
   //raw_tree->Write();
   //std::cout << event->pretty_print() << std::endl;
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

auto CEventRec::from_telemetry(g::TelemetryEvent const &event) -> CEventRec {
  auto sd_event = CEventRec();
  return sd_event;
}
    
auto CEventRec::get_tof_energies() const -> Vec<f32> {
  auto energies = Vec<f32>();
  for (auto const &h : hitseries_) {
    if (h.volume_id_ < 200000000) {
      energies.push_back(h.energydep_);
    }
  }
  return energies;
};

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

auto CEventRec::fill_from_telemetry(g::TelemetryEvent* event,
                                    u8  packet_type,
                                    f64 gcutime,
                                    const g::TofPaddleMap& pmap,
                                    const g::TrkStripMap& smap,
                                    const g::DsiJChnPaddleIdMap& lgmap,
                                    cb::CRawTrk* raw_trk,
                                    cb::CRawTof* raw_tof,
                                    const bool apply_elena_cut,  
                                    const double mev_cut) -> void {

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
    reco_hit.energydep_     = hit.get_edep();
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
