#include <filesystem>

#include "G4SDManager.hh"

#include "event_action.hpp"
#include "events/mc_event.hpp"
#include "io/parsers.h"

namespace gon  = gondola;
namespace fs = std::filesystem;

gon::EventAction::EventAction(std::string fname, const SimConfig& cfg){
  auto path = fs::current_path().string();
  spdlog::info("Will write to {}", path);
  writer    = std::shared_ptr<CRWriter>(new CRWriter(path, fname, 0, Some(0), None));
  digitizer = std::shared_ptr<GapsDigitizer>(new GapsDigitizer(cfg));
  digitizer->load_tof_pulse_templates(cfg.digi_tof_pulse_templates);
  //std::cout << digitizer->n_tof_pulse_templates() << std::endl; 
}

gon::EventAction::~EventAction(){}

auto gon::EventAction::BeginOfEventAction(const G4Event* event) -> void {
  // we can actually install filters here, to only select events with 
  // specific conditions on the primary

}

//------------------------------------------------------------------------

auto gon::EventAction::EndOfEventAction(const G4Event* event)   -> void {
  McHitCollection* mchit_collection = nullptr;
  G4SDManager* SDman = G4SDManager::GetSDMpointer();
  G4HCofThisEvent* HCE = event->GetHCofThisEvent(); 
  auto mc_event = McEvent();
  if (HCE != nullptr) {
    G4int Id           = SDman->GetCollectionID("Collection");
    mchit_collection = (McHitCollection*)HCE->GetHC(Id);
    gon::McHit* hit;
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
    auto vertex     = std::make_shared<gon::RecoHit>();
    vertex->x       = p_vertex->GetX0();
    vertex->y       = p_vertex->GetY0();
    vertex->z       = p_vertex->GetZ0(); 
    vertex->time    = time;
    vertex->energy  = p_kin_e;
    //std::cout << "ENERGY " << p_kin_e << std::endl; 
    auto primary_track = gon::Tracklet(vertex);
    primary_track.vertex_mom_x = p_mom_dir.getX();
    primary_track.vertex_mom_y = p_mom_dir.getY();
    primary_track.vertex_mom_z = p_mom_dir.getZ();
    primary_track.pdg          = p_particle->GetPDGcode(); 
    primary_track.beta         = gun->event_beta;
    mc_event.primary           = std::move(primary_track);
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
  if (digitizer->use_digitizer) {
    // currently, no trigger 
    // setting this to false here will cause all events to trigger 
    bool event_has_triggered = false;
    auto tel_event_res = digitizer->digitize_event(mc_event, event_has_triggered); 
    if (tel_event_res.is_some()) {
      auto tel_ev       = tel_event_res.unwrap();
      tel_ev.version    = 1; // we did not test version 0
                             // in this context it does not matter
                             // which version it is
      tel_ev.tof_event.version = gon::ProtocolVersion::Unknown;
      tel_ev.tof_event.n_trigger_paddles  = 69;
      tel_ev.tof_event.drs_dead_lost_hits = 67;
      tel_ev.tof_event.run_id             = 420;
      tel_ev.tof_event.timestamp32        = 67;
      tel_ev.tof_event.timestamp16        = 69;
      tel_ev.tof_event.quality            = gon::EventQuality::Gold;
      //std::cout << tel_ev.to_string() << std::endl; 
      // wrap it into a TelemetryPacket
      gon::TelemetryPacket pack;
      pack.payload      = tel_ev.to_bytestream();
      usize pos = 0;
      auto debug        = gon::TelemetryEvent::from_bytestream(pack.payload, pos);
      pos = 0;
      //std::cout << "DEBUG " << debug.unwrap().to_string() << std::endl;
      //std::cout << "DEBUG " << debug.unwrap().tof_event.to_string() << std::endl;
      //std::cout << "Got a length of " << pack.payload.size() << " for payload" << std::endl;
      //for (auto const &foo : pack.payload) {
      //  std::cout << (int) foo << std::endl;
      //}
      pack.header.length = pack.payload.size() + gon::TelemetryPacketHeader::SIZE;
      // of course we are only simulating interesting stuff ;) 
      pack.header.ptype = gon::TelemetryPacketType::InterestingEvent; 
      //pack.header.
      
      auto f_obj_tel    = CRFrameObject();
      f_obj_tel.version = 0;
      f_obj_tel.ftype   = CRFrameObjectType::TelemetryPacket; 
      f_obj_tel.payload = pack.to_bytestream();
      //std::cout << "Frame object payload size " << f_obj_tel.payload.size() << std::endl;
      //std::cout << "Writing " << pack.to_string() << std::endl;
      //std::cout << g::TelemetryPacket::from_bytestream(f_obj_tel.payload, pos).to_string() << std::endl;
      frame.put_fobject(f_obj_tel, "TelemetryEvent");
      //f_obj_payload     
    }
  }
  writer->add_frame(frame); 
  if (processed_events % 10000 == 0) {
    std::cout << "Processed " << processed_events << " events!" << std::endl;
  }
  processed_events++;
}

