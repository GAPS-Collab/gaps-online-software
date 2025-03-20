#ifndef CARASPACE_H_INCLUDED
#define CARASPACE_H_INCLUDED
#include <iostream>
#include <fstream>
#include <cmath>

#include "tof_typedefs.h"
#include "packets/tof_packet.h"
#include "telemetry_dataclasses.hpp"
#include "calibration.h"
#include "errors.hpp"
#include "result/result.h"

namespace r = result;

namespace Gaps {
  /// Get all files in a certain directory in case it is a directory, for 
  /// a single file just get the file <3 ChatGPT
  std::vector<std::string> list_path_contents_sorted(const std::string& input);

  /// These are objects which can be stored in a caraspace frame  
  enum class CRFrameObjectType : u8 {
    Unknown           = 0,
    TofPacket         = 10,
    TelemetryPacket   = 20,
  };

  struct CRFrameObject {
    static const u16 HEAD = 0xAAAA;
    static const u16 TAIL = 0x5555;
    
    u8 version;
    CRFrameObjectType ftype;
    Vec<u8> payload;
  
    /// Decode a serializable from a bytestream  
    static auto from_bytestream(Vec<u8> stream, usize &pos) -> CRFrameObject;
     
    /// string representation for printing
    auto to_string() -> std::string;
  };


  struct CRFrame {
    static constexpr u16 HEAD = 0xAAAA;
    static constexpr u16 TAIL = 0x5555;
      
    static auto from_bytestream(Vec<u8> stream, usize &pos) -> CRFrame;
    
    std::map<std::string, std::tuple<u64, CRFrameObjectType>> index;
    Vec<u8> bytestorage;
    auto to_string() const -> std::string;
    
    static auto parse_index(Vec<u8> stream, usize &pos) -> std::map<std::string, std::tuple<u64, CRFrameObjectType>>;

    /// extract a tofpacket if this frame object is of the correct type
    auto get_tofpacket(std::string name) -> r::Result<TofPacket,Gaps::IOError>;
    auto get_telemetrypacket(std::string name) -> Gaps::Telemetry::Packet;
  };

  struct CRReader {
    CRReader();
    CRReader(std::string pathname);
    void set_path(std:: string pathname);
    CRReader(const CRReader&) = delete;
    CRFrame get_next_frame();
    auto get_filenames() const -> Vec<std::string>;
    /// All packets have been read from the file. 
    /// If they should be read again, the reader 
    /// has to be created again
    auto is_exhausted() const -> bool;
    /// The number of files this reader has read
    /// from the file
    auto n_packets_read() const -> bool;
    
    /// get the RBCalibration map
    /// the paramter is the number of RBs we expect in this run
    auto get_rbcalibrations(u8 n_rb) -> RBCalibrationMap;     

  private:  
    bool             exhausted_        ;
    usize            n_packets_read_   ;
    Vec<std::string> filenames_        ;
    std::ifstream    stream_file_      ;
    usize            fileindex_        ;
    void             prime_next_file_();
  };
}
#endif
