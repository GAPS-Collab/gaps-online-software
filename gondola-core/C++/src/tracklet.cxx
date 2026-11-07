// This file is part of gaps-online-software and published 
// under the GPLv3 license

#include <format>
#include <ostream>

#include "tracklet.hpp" 
#include "io/parsers.h"
namespace g = gondola;
using namespace result;

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

auto g::RecoHit::to_bytestream() const -> Vec<u8> { 
  Vec<u8> stream;
  stream.push_back(0xAA);
  stream.push_back(0xAA);
  auto bytes = g::to_le_bytes(x);
  stream.insert(stream.end(),bytes.begin(), bytes.end());
  bytes      = g::to_le_bytes(x_err );
  stream.insert(stream.end(),bytes.begin(), bytes.end());
  bytes      = g::to_le_bytes(y     );
  stream.insert(stream.end(),bytes.begin(), bytes.end());
  bytes      = g::to_le_bytes(y_err );
  stream.insert(stream.end(),bytes.begin(), bytes.end());
  bytes      = g::to_le_bytes(z     );
  stream.insert(stream.end(),bytes.begin(), bytes.end());
  bytes      = g::to_le_bytes(z_err );
  stream.insert(stream.end(),bytes.begin(), bytes.end());
  bytes      = g::to_le_bytes(time  );
  stream.insert(stream.end(),bytes.begin(), bytes.end());
  bytes      = g::to_le_bytes(energy);
  stream.insert(stream.end(),bytes.begin(), bytes.end());
  bytes      = g::to_le_bytes(volume);
  stream.insert(stream.end(),bytes.begin(), bytes.end());
  stream.push_back(0x55);
  stream.push_back(0x55);
  return stream;
}

//-------------------------------------------

namespace gondola {
  RecoHit& RecoHit::operator+=(const RecoHit& rhs) {
    if (volume != rhs.volume) {
      //spdlog::error("Unable to merge hits in different volumes!"); 
      return *this;
    }
    // the position is the energy weighted mean of both
    // (the mean, in case they have no energy)
    const f32 total = energy + rhs.energy;
    if (total > 0) {
      x = (x*energy + rhs.x*rhs.energy)/total;
      y = (y*energy + rhs.y*rhs.energy)/total;
      z = (z*energy + rhs.z*rhs.energy)/total;
    } else {
      x = (x + rhs.x)/2;
      y = (y + rhs.y)/2;
      z = (z + rhs.z)/2;
    }
    energy = total;
    if (rhs.time < time) {
      time = rhs.time;
    }
    if (rhs.x_err > x_err) {
      x_err = rhs.x_err;
    }
    if (rhs.y_err > y_err) {
      y_err = rhs.y_err;
    }
    if (rhs.z_err > z_err) {
      z_err = rhs.z_err;
    }
    return *this;
  }
}
//-------------------------------------------

g::Tracklet::Tracklet() {
    vertex_ = std::make_shared<g::RecoHit>(); 
    stop_   = std::make_shared<g::RecoHit>();
    is_infinite = true;
}

//-------------------------------------------

g::Tracklet::Tracklet(std::shared_ptr<RecoHit> vertex) : 
  vertex_(vertex) {
    stop_   = std::make_shared<g::RecoHit>();
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

auto g::Tracklet::get_stop_raw() const -> std::shared_ptr<RecoHit> {
  return stop_; 
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

auto g::Tracklet::to_bytestream() const -> Vec<u8> { 
  Vec<u8> stream;
  stream.push_back(0xAA);
  stream.push_back(0xAA);
  auto bytes = vertex_->to_bytestream(); 
  stream.insert(stream.end(),bytes.begin(), bytes.end());
  bytes = stop_->to_bytestream(); 
  stream.insert(stream.end(),bytes.begin(), bytes.end());
  stream.push_back((u8)is_infinite); 
  bytes      =   g::to_le_bytes(vertex_mom_x)           ; 
  stream.insert(stream.end(),bytes.begin(), bytes.end());
  bytes      =   g::to_le_bytes(vertex_mom_y)           ; 
  stream.insert(stream.end(),bytes.begin(), bytes.end());
  bytes      =   g::to_le_bytes(vertex_mom_z)           ; 
  stream.insert(stream.end(),bytes.begin(), bytes.end());
  bytes      =   g::to_le_bytes(pdg)           ; 
  stream.insert(stream.end(),bytes.begin(), bytes.end());
  bytestream_extend(stream, (f32)beta);
  stream.push_back(0x55);
  stream.push_back(0x55);
  return stream;
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


//-------------------------------------------

auto g::RecoHit::from_bytestream(const Vec<u8> &stream, usize &pos)
  -> result::Result<g::RecoHit, g::IOError> {
  if (stream.size() < pos + RecoHit::SIZE) {
    auto message = std::format("Stream too short for RecoHit! {} bytes left", stream.size() - pos);
    return Err(g::IOError(g::IOError::ErrorKind::StreamTooShort, message));
  }
  u16 head = g::parse_u16(stream, pos);
  if (head != RecoHit::HEAD) {
    auto message = std::format("Decoding of HEAD failed! Got {} instead!", head);
    return Err(g::IOError(g::IOError::ErrorKind::WrongHeaderBytes, message));
  }
  auto hit   = RecoHit();
  hit.x      = g::parse_f32(stream, pos);
  hit.x_err  = g::parse_f32(stream, pos);
  hit.y      = g::parse_f32(stream, pos);
  hit.y_err  = g::parse_f32(stream, pos);
  hit.z      = g::parse_f32(stream, pos);
  hit.z_err  = g::parse_f32(stream, pos);
  hit.time   = g::parse_f32(stream, pos);
  hit.energy = g::parse_f32(stream, pos);
  hit.volume = g::parse_u32(stream, pos);
  u16 tail   = g::parse_u16(stream, pos);
  if (tail != RecoHit::TAIL) {
    auto message = std::format("Decoding of TAIL failed! Got {} instead!", tail);
    return Err(g::IOError(g::IOError::ErrorKind::WrongTailBytes, message));
  }
  return Ok(hit);
}

//-------------------------------------------

auto g::Tracklet::from_bytestream(const Vec<u8> &stream, usize &pos)
  -> result::Result<g::Tracklet, g::IOError> {
  if (stream.size() < pos + Tracklet::SIZE) {
    auto message = std::format("Stream too short for Tracklet! {} bytes left", stream.size() - pos);
    return Err(g::IOError(g::IOError::ErrorKind::StreamTooShort, message));
  }
  u16 head = g::parse_u16(stream, pos);
  if (head != Tracklet::HEAD) {
    auto message = std::format("Decoding of HEAD failed! Got {} instead!", head);
    return Err(g::IOError(g::IOError::ErrorKind::WrongHeaderBytes, message));
  }
  auto vertex = RecoHit::from_bytestream(stream, pos);
  if (vertex.is_err()) {
    return Err(vertex.unwrap_err());
  }
  auto stop   = RecoHit::from_bytestream(stream, pos);
  if (stop.is_err()) {
    return Err(stop.unwrap_err());
  }
  auto tracklet = Tracklet(std::make_shared<RecoHit>(vertex.unwrap()));
  tracklet.set_stop(std::make_shared<RecoHit>(stop.unwrap()));
  tracklet.is_infinite  = g::parse_bool(stream, pos);
  tracklet.vertex_mom_x = g::parse_f32(stream, pos);
  tracklet.vertex_mom_y = g::parse_f32(stream, pos);
  tracklet.vertex_mom_z = g::parse_f32(stream, pos);
  tracklet.pdg          = g::parse_i32(stream, pos);
  tracklet.beta         = g::parse_f32(stream, pos);
  u16 tail = g::parse_u16(stream, pos);
  if (tail != Tracklet::TAIL) {
    auto message = std::format("Decoding of TAIL failed! Got {} instead!", tail);
    return Err(g::IOError(g::IOError::ErrorKind::WrongTailBytes, message));
  }
  return Ok(tracklet);
}
