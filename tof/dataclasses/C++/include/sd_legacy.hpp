#ifdef BUILD_ROOTCOMPONENTS
#ifndef SD_LEGACY_H_INCLUDED
#define SD_LEGACY_H_INCLUDED

#include "TObject.h"
#include "TVector3.h"

#include "tof_typedefs.h" 

namespace gondola {
  auto read_sd_legacy_example() -> void; 
}

typedef i32 CFitStatusType;

class CEventBase : public TObject {
  public: 
    CEventBase() {};
  //private:
    u32 runNumber_;
    u32 subRunNumber_;
    u32 eventNumber_;
    u64 eventTime_;
    u32 eventId_;

    // generated primary
    f64       primaryBetaGenerated_;
    TVector3  primaryMomentumDirectionGenerated_;
    f64       primaryKineticEnergyGenerated_;
    
    ClassDef(CEventBase, 13)
};

class CTrackBase : public TObject {
  public:
    CTrackBase() {};
  protected:
    bool     Primary;
    u32      TrackId;
    u32      VertexVolumeId;
    TVector3 VertexPosition;
    u32      LastVolumeId;
    TVector3 LastPosition;
    f64      ColumnDensity; // column density [g/cm2]

    Vec<f64>      EnergyDeposition;
    Vec<f64>      GlobalTime;
    Vec<f64>      StepLength;
    Vec<u32>      VolumeId;
    Vec<TVector3> Position;
    Vec<TVector3> PositionResidual;
    Vec<TVector3> MomentumDirection;
    Vec<f64>      Depth; // column density [g/cm2]
    Vec<f64>      ColumnDensityUntilStep; // column density [g/cm2]
    
    ClassDef(CTrackBase, 5);
};

class CTrackRec : public CTrackBase {
  public: 
    CTrackRec() {};
  private:
    bool Used; ///< true if track is used in vertex fit
    bool Associated;

    f64  Chi2;
    i32  Ndof;
    CFitStatusType FitStatus; ///< 1 -> ok, -1 -> error
    
    ClassDefOverride(CTrackRec, 7);
};

class GRecoHit : public TObject { 
  public: 
    GRecoHit() {};
  private:  
    u32 volume_id_;
    f64 energydep_;
    TVector3 hit_position_;
    f64 hit_time_;
    i32 index_; ///< hit index in input CEventRec vectors
    
    ClassDef(GRecoHit, 5);
};  
typedef Vec<GRecoHit> GRecoHitSeries;

class CEventRec : public CEventBase {
  public:
    CEventRec() {};
  private:
    std::string                            activeReco_;
    std::map<std::string,TVector3>         primaryStoppingPosition_;
    std::map<std::string,u32>              primaryStoppingVolume_;
    std::map<std::string,f64>              primaryStoppingTime_;
    std::map<std::string,f64>              primaryBeta_;
    std::map<std::string,f64>              primaryBetaError_;///< error on measured beta (needed to calculate uncertainty on estimated primary mass)
    std::map<std::string,TVector3>         primaryMomentumDirection_;
    std::map<std::string,Vec<f64>>         primaryEnergyDepositions_; 
    std::map<std::string,Vec<i32>>         HitTrackIndex; ///< track index of the associated track
    std::map<std::string,f64>              Chi2; ///< chi2 of vertex fit
    std::map<std::string,i32>              Ndof; ///< ndof of vertex fit
    std::map<std::string,Vec<f64>>         ParCov; ///< covariance matrix of vertex fit parameters
    std::map<std::string,CFitStatusType>   FitStatus; ///< fit status of vertex fit

    std::map<std::string,Vec<f64>>         SdFitPar; ///< slowdown fit parameters {range, Ekin}
    std::map<std::string,Vec<f64>>         SdFitErr; ///< slowdown fit errors
    std::map<std::string,f64>              SdFitChi2  ; ///< slowdown fit chi-square
    std::map<std::string,i32>              SdFitNdof  ; ///< slowdown fit # of degrees of freedom
    GRecoHitSeries hitseries_              = Vec<GRecoHit>({});

    std::map<std::string, Vec<CTrackRec*>> Tracks;
    Vec<std::string>                       registeredRecos_;
    
    ClassDefOverride(CEventRec, 12)
};


#endif
#endif
