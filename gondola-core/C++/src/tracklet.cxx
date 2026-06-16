// This file is part of gaps-online-software and published 
// under the GPLv3 license

#include <format>
#include <ostream>

#include "tracklet.hpp" 

namespace g = gondola;

//-------------------------------------------

auto g::RecoHit::to_string() const -> std::string {
  std::string repr = "<RecoHit:";
  repr += std::format("\n  x {:.2f} +- {:.2f}, y {:.2f} +- {:.2f}, z {:.2f} +- {:.2f}", x, x_err, y, y_err, z, z_err); 
  repr += std::format("\n  t {:.2f}", time);
  repr += std::format("\n  E {:.2e}", energy);
  repr += std::format("\n  vol {}", volume);
  repr += ">";
  return repr;
}

//-------------------------------------------

g::Tracklet::Tracklet(std::shared_ptr<RecoHit> vertex) : 
  vertex_(vertex) {
    auto stop   = std::make_shared<g::RecoHit>();
    is_infinite = true;
}

//-------------------------------------------
    
auto g::Tracklet::get_vertex() const -> std::shared_ptr<RecoHit> {
  return vertex_; 
}

//-------------------------------------------

auto g::Tracklet::get_stop() const -> Option<std::shared_ptr<RecoHit>> {
  if (is_infinite) {
    return None;
  } else {
    return Some(stop_); 
  }
}

//-------------------------------------------

auto g::Tracklet::set_stop(std::shared_ptr<RecoHit> stop) -> void {
  stop_ = stop; 
}

//-------------------------------------------

auto g::Tracklet::to_string() const -> std::string {
  std::string repr = "<Tracklet:";
  repr += std::format("\n  Vertex    : {}", vertex_->to_string());
  repr += std::format("\n  Vertex Mom: x {:.2f} y {:.2f} z {:.2f}", vertex_mom_x, vertex_mom_y, vertex_mom_z );
  repr += std::format("\n  Beta      : {:.2f} +- {:.2f}", beta, beta_err);
  repr += "\n -- -- -- -- -- -- -- -- ";
  if (!is_infinite) {
    repr += std::format("\n  Stop  : {}", stop_->to_string());
  } else {
    repr += "\n  -- infinite track";
  }
  repr += "\n  deposited energies:\n  -> ";
  for (auto const &k : edeps) {
    repr += std::format("({},{:.2e})", std::get<0>(k), std::get<1>(k));
  } 
  if (gof.is_some()) {
    // make explicit copy since unwrap violates const
    auto _gof = gof;
    repr += std::format("\n Goodness-Of-Fit: {:.2e}", _gof.unwrap());
  }
  repr += ">";
  return repr;
}

//-------------------------------------------

namespace gondola {
  
  std::ostream& operator<<(std::ostream& os, const g::RecoHit& th) {
    os << th.to_string();
    return os;
  }
  
  std::ostream& operator<<(std::ostream& os, const g::Tracklet& th) {
    os << th.to_string();
    return os;
  }
}

