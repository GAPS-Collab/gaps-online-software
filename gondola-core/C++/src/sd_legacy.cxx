#ifdef BUILD_WITH_ROOT

#include <iostream>

#include "TChain.h"

#include "sd_legacy.hpp"
#include "telemetry_dataclasses.hpp"

namespace cb = Crane::Calibration;
namespace tf = Crane::Reconstruction::TrackFit;

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
  TChain * input_tree = new TChain("TreeRec");
  input_tree->Add("/srv/gaps/gaps-online-software/example-data/Run9125.gse5_241213_151813UTC_rec.root");
  //input_tree->Add(fName.c_str());
  auto input_event = new CEventRec();
  input_tree->SetBranchAddress("Rec", &input_event);
  u64 max_events = input_tree->GetEntries();
  for (u64 evid=0; evid<max_events; evid++) {
    input_tree->GetEntry(evid);
    std::cout << input_event->runNumber_ << std::endl; 
    //std::cout << input_event->run_number << std::endl; 
    std::cout << input_event->eventId_ << std::endl; 
    break;
  }
}

auto CEventRec::from_telemetry(gt::MergedEvent const &event) -> CEventRec {
  auto sd_event = CEventRec();
  return sd_event;
}

auto CEventRec::to_telemetry() -> gt::MergedEvent {
  auto event = gt::MergedEvent();
  return event;
}

#endif
