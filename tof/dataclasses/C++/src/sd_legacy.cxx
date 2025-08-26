#ifdef BUILD_WITH_ROOT

#include <iostream>

#include "sd_legacy.hpp"
#include "TChain.h"

ClassImp(GRecoHit);
ClassImp(CEventBase);
ClassImp(CTrackBase);
ClassImp(CEventRec);
ClassImp(CTrackRec);


void gondola::read_sd_legacy_example() {
  TChain * input_tree = new TChain("TreeRec");
  input_tree->Add("/srv/gaps/example-data/Run9125.gse5_241213_142800UTC_rec.root");
  //input_tree->Add(fName.c_str());
  auto input_event = new CEventRec();
  input_tree->SetBranchAddress("Rec", &input_event);
  u64 max_events = input_tree->GetEntries();
  for (u64 evid=0; evid<max_events; evid++) {
    input_tree->GetEntry(evid);
    std::cout << input_event->runNumber_ << std::endl; 
    //std::cout << input_event->run_number << std::endl; 
    std::cout << input_event->eventId_ << std::endl; 
  }
}

#endif
