#ifndef TOFIO_H_INCLUDED
#define TOFIO_H_INCLUDED

#include <fstream>
#include <functional>

#include "result/result.h"

#include "events.h"
#include "packets/tof_packet.h"
#include "serialization.h"
#include "errors.hpp"

namespace r = result;

//template<typename T>
//requires HasFromByteStream<T>
//Vec<T> unpack<T>(String filename) {
//  usize pos = 0;
//  auto packets = get_tofpackets(filename);
//  for (const auto &p : packets) {
//    T data = T::from_bytestream(p.payload, 0);
// 
//
//  Vec<T> data;
//  return data;
//}

namespace gondola {
  /// Get all files in a certain directory in case it is a directory, for 
  /// a single file just get the file <3 ChatGPT
  ///
  /// # Arguments 
  ///   * input            : path or filename 
  ///   * use_telemetry_re : use the regex for telemetry files to find the files.
  ///                        Default is set to false, which will find L0 or TOF files
  auto list_path_contents_sorted(const std::string& input, bool use_telemetry_re = false) -> Vec<std::string>;
} 

namespace gondola {

  /// Read serialized TofPackets from 
  /// a file and emit them as packets
  class TofPacketReader {
    public: 
      TofPacketReader();
      TofPacketReader(String filename);
      TofPacketReader(const TofPacketReader&) = delete;
      //TofPacketReader& operator=(const TofPacketReader&) = delete;
      /// Set a filename where to read packets from. This is a binary file format,
      /// typically ending in ".tof.gaps"
      void set_filename(String filename);
      /// Walk over the file and return the next packet
      auto get_next_packet() -> r::Result<TofPacket, gondola::IOError>;
      /// Return the filename we assigned
      auto get_filename() const -> std::string;
      /// All packets have been read from the file. 
      /// If they should be read again, the reader 
      /// has to be created again
      auto is_exhausted() const -> bool;
      /// The number of files this reader has read
      /// from the file
      auto n_packets_read() const -> usize;

    private:
      std::ifstream  stream_file_;
      bool           exhausted_;
      usize          n_packets_read_;
      std::string    filename_;
  };
}


#endif
