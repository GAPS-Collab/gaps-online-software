/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
#pragma once 

#include <memory>
#include<limits>
#include<cmath>

#include "gondola.hpp" 
#include "result/result.h"
#include "errors.hpp"

namespace gondola {

  /// A calibrated hit, oblivious to everything which 
  /// happened on low-level
  struct RecoHit {
    static constexpr u16   HEAD = 0xAAAA;
    static constexpr u16   TAIL = 0x5555;
    /// size of the serialized hit in bytes
    static constexpr usize SIZE = 40;

    f32 x      = std::numeric_limits<f32>::quiet_NaN();
    f32 x_err  = std::numeric_limits<f32>::quiet_NaN();
    f32 y      = std::numeric_limits<f32>::quiet_NaN();
    f32 y_err  = std::numeric_limits<f32>::quiet_NaN();
    f32 z      = std::numeric_limits<f32>::quiet_NaN();
    f32 z_err  = std::numeric_limits<f32>::quiet_NaN();
    f32 time   = std::numeric_limits<f32>::quiet_NaN();
    f32 energy = std::numeric_limits<f32>::quiet_NaN();
    u32 volume = 0;

    auto to_string() const -> std::string;
    /// Serializaton to bytes - needed to be 
    /// written to a file
    auto to_bytestream() const -> Vec<u8>;
    /// Deserialization, mirrors the rust implementation
    static auto from_bytestream(const Vec<u8> &stream, usize &pos)
      -> result::Result<RecoHit, IOError>;

    /// Merge another hit in the same volume into this one
    /// (hits in other volumes are ignored): the energies are
    /// summed, the earliest time is kept, the position is the
    /// energy weighted mean and the errors are the larger ones
    RecoHit& operator+=(const RecoHit& rhs);
    
    friend std::ostream& operator<<(std::ostream& os, const RecoHit& et);
  };
  
  inline RecoHit operator+(RecoHit lhs, const RecoHit& rhs) {
      lhs += rhs; 
      return lhs;
  }

  /// A straight line as it could be part of a track 
  struct Tracklet {
    static constexpr u16   HEAD = 0xAAAA;
    static constexpr u16   TAIL = 0x5555;
    /// size of the serialized tracklet in bytes
    static constexpr usize SIZE = 2 + 2*RecoHit::SIZE + 1 + 4*4 + 4 + 2;

    bool is_infinite                = false; 
    f32  vertex_mom_x               = std::numeric_limits<f32>::quiet_NaN(); 
    f32  vertex_mom_y               = std::numeric_limits<f32>::quiet_NaN(); 
    f32  vertex_mom_z               = std::numeric_limits<f32>::quiet_NaN();
    /// particle type identifier (see PDG codes) 
    i32  pdg                        = 0;
    f32  beta                       = std::numeric_limits<f32>::quiet_NaN();
    f32  beta_err                   = std::numeric_limits<f32>::quiet_NaN();
   
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
    /// The stop, also if the tracklet is infinite. 
    ///
    /// The simulation fills the stop of the primary, but does 
    /// not unset is_infinite. Stop values are NaN if the 
    /// primary did not stop.
    auto get_stop_raw() const -> std::shared_ptr<RecoHit>;
    auto set_stop(std::shared_ptr<RecoHit> stop) -> void;
    auto to_string()  const -> std::string;
    /// Serializaton to bytes - needed to be 
    /// written to a file
    auto to_bytestream() const -> Vec<u8>;
    /// Deserialization, mirrors the rust implementation
    static auto from_bytestream(const Vec<u8> &stream, usize &pos)
      -> result::Result<Tracklet, IOError>;
    
    private:
      std::shared_ptr<RecoHit> vertex_ ; 
      std::shared_ptr<RecoHit> stop_   ; 
  };
  
  std::ostream& operator<<(std::ostream& os, const gondola::Tracklet& et);
}

