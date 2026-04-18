#ifdef BUILD_CXX_WITH_ROOT
#ifndef SD_LEGACY_H_INCLUDED
#define SD_LEGACY_H_INCLUDED

#include <memory>

#include "TObject.h"
#include "TVector3.h"
#include "TChain.h"

#include "tof_typedefs.h" 
#include "telemetry_dataclasses.hpp"


typedef i32 CFitStatusType;

class CEventBase : public TObject {
  public: 
    CEventBase() {}
 
    u32      runNumber_;
    u32      subRunNumber_;
    u32      eventNumber_;
    // upstream change in SD
    long int eventTime_;
    u32      eventId_;

    // generated primary
    f64       primaryBetaGenerated_;
    TVector3  primaryMomentumDirectionGenerated_;
    f64       primaryKineticEnergyGenerated_;
    
    ClassDef(CEventBase, 13)
};

class CTrackBase : public TObject {
  public:
    CTrackBase() {}
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
    CTrackRec() {}
  
    auto pretty_print() const -> std::string;
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
    GRecoHit() {}
    // getters for compatibility - not sure if 
    // changing the private members to public 
    // will cause any issues
    auto GetVolId() const -> u32;
    auto GetEDep()  const -> f64;
    auto GetPos()   const -> TVector3;
    auto GetTime()  const -> f64;
    auto GetIdx()   const -> i32;
    auto pretty_print() const -> std::string;
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
  
  // This is the actual compatibility layer. 
  // Populate a CEventRec object from a 
  // MergedEvent
  public:
    static auto from_telemetry(gondola::TelemetryEvent const &event) -> CEventRec;
    auto to_telemetry(HashMap<u32, u32> const &hid_vid_map) -> gondola::TelemetryEvent;
    auto pretty_print() const -> std::string;
    auto GetGPSTime() const -> f64;
  public:
    CEventRec() {}
    
    std::vector<unsigned char>             trigger_sources  {};
    std::vector<unsigned int>              trigger_vids     {};
    
    std::string                            activeReco_;
    std::vector<int>                       event_quality;     
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
 
    // newer stuff (v26.03)
    i32                                    PacketType;

    u32                                    gps_time_lower_  = 0;
    u16                                    gps_time_upper_  = 0; 
    ClassDefOverride(CEventRec, 17)
};

namespace Crane { 
  namespace Calibration {
    class CRawHeader : public TObject {
      public:
        CRawHeader() {}
        
        u8  type;
        u32 timestamp;
        u16 counter;
        u16 length;
        
        u64 systime;
        u32 eventid;
        
        Vec<u64>  trk_eventtime;
        Vec<u64>  trk_eventid;
        Vec<u16>  trk_eventid_valid;
        Vec<u16>  trk_layer;
        Vec<u8>   tof_packettype;
        
        ClassDef(CRawHeader, 4);
    };
    
    class CRawTrk : public TObject {
  
      public:

        CRawTrk() {}
        
        u8       flag;
        u32      eventid;
        Vec<u64> eventtime;
        Vec<u8>  layer;
        Vec<u8>  row;
        Vec<u8>  module;
        Vec<u8>  channel;
        Vec<u16> adcdata;

        Vec<i32> hindex; // hit index inside CEventRec

        ClassDef(CRawTrk, 4);
    };
  
    class CRawTofHits : public TObject {
      public:
        CRawTofHits() {}
        
        Vec<u8>  trigger_th  ;
        Vec<f64> timestamp48 ;
        Vec<u8>  paddle_id   ;
        Vec<f32> base_a      ;
        Vec<f32> base_b      ;
        Vec<f32> base_a_rms  ;
        Vec<f32> base_b_rms  ;
        Vec<f32> phase       ;
        Vec<f32> time_a      ;
        Vec<f32> time_b      ;
        Vec<f32> peak_a      ;
        Vec<f32> peak_b      ;
        Vec<f32> charge_a    ;
        Vec<f32> charge_b    ;
        Vec<f32> charge_min_i;
        Vec<f32> x_0         ;
        Vec<f32> t_0         ;

        Vec<f32> t_shift     ;
        Vec<i32> hindex      ; // hit index inside CEventRec
        ClassDef(CRawTofHits, 9);
    };

    class CRawTrigger : public TObject {
      public:

        CRawTrigger() {}
        Vec<u8> mtb_link_ids   ; ///<       
        Vec<u8> dsi            ;
        Vec<u8> j              ;
        Vec<u8> ch             ;
        Vec<u8> th             ;
        Vec<u8> paddle_id      ;
        Vec<u8> trigger_sources;
        
        ClassDef(CRawTrigger, 3);
    };

    class CRawTofWFs : public TObject {
      public:

        CRawTofWFs() {}
        
        Vec<f64>                timestamp48 {}; 
        Vec<u8>                 rb_id       {};
        Vec<u8>                 rb_channel  {};
        Vec<Vec<u16>>           adc         {};
        Vec<Vec<f32>>           voltages    {};
        Vec<Vec<f32>>           times       {};
        Vec<i32>                paddle_id   {};
        
        Vec<Vec<u16>>           adc_9       {};
        Vec<Vec<f32>>           voltages_9  {};
        Vec<Vec<f32>>           times_9     {};
        ClassDef(CRawTofWFs, 6);
    };


    class CRawTof : public TObject {
  
      public:
        CRawTof() {}
        char         flag;
        u32          runid;                //MTB pkt
        u32          eventid;              //summary pkt
        u8           event_status;
        f64          timestamp48 ;         //summary pkt
        u32          timestamp ;          //MTB pkt ???
        u64          timestamp_gps48 ;    //MTB pkt
        u64          timestamp_abs48 ;    //MTB pkt

        CRawTofHits  hits;      ///< hits
        CRawTofWFs   wfs;       ///< waveforms
        CRawTrigger  trg;    // = CRawTrigger();      ///< trigger hits
        
        ClassDef(CRawTof, 10);
    }; 
  }
}

//------------------------------------------------
// Reconstruction classes

namespace Crane{
  namespace Reconstruction {
    namespace TrackFit {
      
      typedef GRecoHit GDataHit;
      class Plane {
        public:
          Plane() {}
          f64 a, b, c, d;        //the (a, b, c, d) in a*x + b*y + c*z + d = 0.
      };
      
      class ReferencePlane : public Plane {
        public:
          ReferencePlane() {}
          TVector3 origin_;
          TVector3 orientation_;
      };
      
      class GDataPar {
    
        public:
          GDataPar(){}
          ReferencePlane par_ref_plane_; ///<reference system
          Vec<f64>       par_;           ///< state vector
          Vec<f64>       par_cov_;       ///< state vector covariance matrix
          
          f64            chi2_;
          i32            ndof_;
      };
      
      class GDataTime : public GDataPar {
        public:
          GDataTime() {}
          //friend class GFitLine;
          //friend class GFitTime;
          //friend class GDataTrack;
      };
      
      class GDataLine : public GDataPar {
        public:
          GDataLine() {}

          //friend class GFitLine;
          //friend class GFitStar;
          //friend class GDataTrack;
      };

      class GDataDEDX : public GDataPar {
        public:
          GDataDEDX(){}

          u16 par_type_; // = -1;
      };
      
      class GDataDEDX_NRS : public GDataDEDX {
        public:
      };

      class GDataPoint {
        public:
          GDataPoint() {}

          Vec<GDataHit>  hits_; ///< vector of hits
          ReferencePlane volume_plane_; ///< measured position and detection plane

          u64      hit_energy_       ;
          u64      hit_time_         ;
          u64      hit_energy_err_   ;
          u64      hit_time_err_     ;
          TVector3 hit_position_err_ ;
          char     flag_             ;
      };
      
      using GTrajectoryPoint = std::pair<f64,f64>;///< evaluated (pathlength,time) along a give trajectory
      // FIXME - this is a bug in Elena's code. It seems this is intended to 
      // inherit from TObject 
      class GDataTrack {
        public:
          GDataTrack() {}

          //friend class GFitDEDX;
          //friend class GFitLine;
          //friend class GFitTime;
          //friend class GFindVertex;

          char                   flag_;///< flag to mask/tag the track
          Vec<GDataPoint>        vxyz_; ///< associated points + errors
          GDataLine              pline_; ///< interpolated line + errors
          GDataTime              ptime_; ///< t0,beta + errors
          GDataDEDX              *pdedx_; ///< E0,R + errors
          Vec<GTrajectoryPoint>  vxyz_eval_; ///< calculated pathlength & time
          Vec<f64>               dedx_;       ///< dE/dx
          Vec<f64>               xdepth_;     ///<integrated material
          Vec<f64>               dedx_eval_;  ///< calculated dE/dx 
          GTrajectoryPoint       start_eval_;
          GTrajectoryPoint       stop_eval_;
            
          //ClassDef(GDataTrack,1);
      };

      class GDataVertex : public GDataPar {
        public:
          GDataVertex() {}

          std::pair<f64,f64> time_  ;//FIX TEMPORANEO MESSO A MANO QUI
          std::pair<f64,f64> xdepth_;// FIX TEMPORANEO MESSO A MANO QUI
      };

      class GDataEvent : public TObject {
        public:

          GDataEvent() {}
    
          i32 status_;
          Vec<GDataPoint> points_; ///< all points
          Vec<GDataTrack> tracks_; ///< all reconstructed tracks
          GDataVertex vertex_;             ///< reconstructed vertex

          ClassDef(GDataEvent,1);
      };
    }// end of namespace TrackFit
  } // end of namespace Reconstruction
} // end of namespace Crane

namespace gondola {
  auto read_sd_legacy_example(std::string filename) -> void; 
  
  /// Read SimpleDet Root files and emit 
  /// MergedEvents
  struct SDRootReader {
    SDRootReader(std::string);
    ~SDRootReader();
    auto get_event(u64 event_idx) -> void; 

    std::string filename;
    // root just hates modern memory management
    TChain* tchain;
    u64 nevents_total;
    CEventRec* event;
  };
}

#endif
#endif
