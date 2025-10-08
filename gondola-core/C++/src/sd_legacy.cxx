#ifdef BUILD_WITH_ROOT

#include <iostream>

#include "TChain.h"

#include "sd_legacy.hpp"
#include "telemetry_dataclasses.hpp"
#include "events.h"
#include "database.h"

namespace cb = Crane::Calibration;
namespace tf = Crane::Reconstruction::TrackFit;
namespace  g = gondola;
namespace gt = Gaps::Telemetry; 

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


void gondola::read_sd_legacy_example() {
  auto hid_vid_map    = gondola::get_hid_vid_map();
  TChain * input_tree = new TChain("TreeRec");
  input_tree->Add("/srv/gaps/gaps-online-software/example-data/Run9125.gse5_241213_151813UTC_rec.root");
  //input_tree->Add("/srv/gaps/gaps-online-software/example-date/Run9125.gse5_241213_125815UTC_rec.root");
  //input_tree->Add("/srv/gaps/gaps-online-software/example-data/ethernet241213_1347_rec.root");
  //input_tree->Add(fName.c_str());
  auto input_event = new CEventRec();
  input_tree->SetBranchAddress("Rec", &input_event);
  u64 max_events = input_tree->GetEntries();
  for (u64 evid=0; evid<max_events; evid++) {
    input_tree->GetEntry(evid);
    std::cout << input_event->runNumber_ << std::endl; 
    //std::cout << input_event->run_number << std::endl; 
    std::cout << input_event->eventId_ << std::endl; 
    std::cout << input_event->to_telemetry(hid_vid_map).to_string() << std::endl;
    break;
  }
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

//------------------------------------------------------------------------

auto CEventRec::from_telemetry(gt::MergedEvent const &event) -> CEventRec {
  auto sd_event = CEventRec();
  return sd_event;
}

//------------------------------------------------------------------------

auto CEventRec::to_telemetry(HashMap<u32, u32> const &vid_hid_map) -> gt::MergedEvent {
  auto event = gt::MergedEvent();
  event.event_id = eventId_;
  for (auto const &hit : hitseries_) {
    // check if tracker or tof hit 
    if (hit.GetVolId() > 200000000) {
      Gaps::Telemetry::TrkHit trk_hit;
      u32 hw_id = vid_hid_map.at((u32)hit.GetVolId());
      auto lrms = Gaps::Telemetry::TrkHit::decode_id(hw_id);
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

#endif
