#ifdef __CINT__
#pragma link C++ class CEventBase+;
#pragma link C++ class CEventRec+;
#pragma link C++ class CTrackBase+;
#pragma link C++ class CTrackRec+;
#pragma link C++ class GRecoHit+;
#pragma link C++ class Crane::Calibration::CRawHeader+;
#pragma link C++ class Crane::Calibration::CRawTrk+;
#pragma link C++ class Crane::Calibration::CRawTofHits+;
#pragma link C++ class Crane::Calibration::CRawTrigger+;
#pragma link C++ class Crane::Calibration::CRawTofWFs+;
#pragma link C++ class Crane::Calibration::CRawTof+;
#pragma link C++ class GSimulationParameter+;

// Reco stuff
#pragma link C++ class Crane::Reconstruction::TrackFit::GDataEvent+;
// not a TObject
#pragma link C++ class Crane::Reconstruction::TrackFit::Plane+;
// not a TObject
#pragma link C++ class Crane::Reconstruction::TrackFit::ReferencePlane+;
// not a TObject
#pragma link C++ class Crane::Reconstruction::TrackFit::GDataPar+;
// not a TObject
#pragma link C++ class Crane::Reconstruction::TrackFit::GDataLine+;
// not a TObject
#pragma link C++ class Crane::Reconstruction::TrackFit::GDataPoint+;
// not a TObject
#pragma link C++ class Crane::Reconstruction::TrackFit::GDataTrack+;
//// not a TObject
#pragma link C++ class Crane::Reconstruction::TrackFit::GDataTime+;
// not a TObject
#pragma link C++ class Crane::Reconstruction::TrackFit::GDataDEDX+;
// not a TObject
#pragma link C++ class Crane::Reconstruction::TrackFit::GDataDEDX_NRS+;

#endif
