/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
#pragma once 

#include <memory>
#include "gondola.hpp" 

namespace gondola {

  /// A calibrated hit, oblivious to everything which 
  /// happened on low-level
  struct RecoHit {
    f32 x     ;
    f32 x_err ;
    f32 y     ;
    f32 y_err ;
    f32 z     ;
    f32 z_err ;
    f32 time  ;
    f32 energy;
    u32 volume;

    auto to_string() const -> std::string;
    /// Serializaton to bytes - needed to be 
    /// written to a file
    auto to_bytestream() const -> Vec<u8>;

  };
  
  std::ostream& operator<<(std::ostream& os, const gondola::RecoHit& et);

  /// A straight line as it could be part of a track 
  struct Tracklet {
    bool is_infinite                ; 
    f32  vertex_mom_x               ; 
    f32  vertex_mom_y               ; 
    f32  vertex_mom_z               ;
    /// particle type identifier (see PDG codes) 
    i32  pdg                        ;
    // these are for SD interoperatibility and won't get 
    // serialized
    f32  beta                       ;
    f32  beta_err                   ;
   
    Tracklet(std::shared_ptr<RecoHit> vertex);
    Tracklet();
    /// The energy depositions per each 
    /// crossed volume
    Vec<std::tuple<u32,f32>> edeps  ;
    /// Column density (if applicable) 
    Vec<f64>                 coldens;
    /// Goodness of fit, most likely some chi^2 
    /// or similar
    Option<f32>  gof                ; 
    auto get_vertex() const -> std::shared_ptr<RecoHit>;
    auto get_stop()   const -> Option<std::shared_ptr<RecoHit>>;
    auto set_stop(std::shared_ptr<RecoHit> stop) -> void;
    auto to_string()  const -> std::string;
    /// Serializaton to bytes - needed to be 
    /// written to a file
    auto to_bytestream() const -> Vec<u8>;
    
    private:
      std::shared_ptr<RecoHit> vertex_ ; 
      std::shared_ptr<RecoHit> stop_   ; 
  };
  
  std::ostream& operator<<(std::ostream& os, const gondola::Tracklet& et);
}

