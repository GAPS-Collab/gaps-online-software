#ifndef TELEMETRY_READER_H_INCLUDED
#define TELEMETRY_READER_H_INCLUDED
// This file is part of gaps-online-software and published 
// under the GPLv3 license

#include <iostream>
#include <fstream>

#include "tof_typedefs.h"
#include "telemetry_dataclasses.hpp"
#include "io.hpp"

namespace gondola {
  /// Read serialized TelemetryPackets from an existing file
  ///
  /// Read GAPS binary files ("Berkeley binaries)
  struct TelemetryPacketReader {
    
    TelemetryPacketReader();
    TelemetryPacketReader(std::string pathname);
    TelemetryPacketReader(const TelemetryPacketReader&) = delete;
    
    auto set_path(std::string pathname) -> void;
    
    auto get_filenames() const -> Vec<std::string>; 
    
    auto get_next_packet() -> Gaps::Telemetry::Packet; 
    
    /// All packets have been read from the file. 
    /// If they should be read again, the reader 
    /// has to be created again
    auto is_exhausted()   const -> bool;
    private:
      /// Indicate if the reader has run out of 
      /// packets 
      bool exhausted_            ;
      
      /// Reader will emit packets from these files,
      /// if one file is exhausted, it moves on to 
      /// the next file automatically
      Vec<std::string> filenames_;       
      /// The index of the file the reader is 
      /// currently reading
      usize file_idx_            ;
      /// depending on the source of the telemetry files, 
      /// there might be duplicates, because we get them
      /// over different streams. 
      /// Suppress these multiple packets
      bool dedup                ;
      /// Ignore packets that have a gcu time earlier than start_time 
      f64 start_time            ; 
      /// Ignore packets that have a gcu time later than end_time
      f64 end_time              ; 
      /// The current file the reader is actually reading from 
      std::ifstream  stream_file_ ;     ;
      //file_reader         : BufReader<File>,
      /// Current (byte) position in the file
      usize cursor              ;
      /// Read only packets of type == PacketType
      //pub filter          : TelemetryPacketType,
      /// Number of read packets
      usize n_packs_read_       ;
      /// Number of skipped packets
      usize n_packs_skipped     ;
      /// Skip the first n packets
      usize skip_ahead          ;
      /// Stop reading after n packets
      usize stop_after          ;
      /// Number of encountered duplicates 
      usize n_duplicates        ;
      /// A cache to allow to quench duplicates 
      /// pkt counter -> pkt checksum
      //dedup_cache         : HashMap<u16, VecDeque<u16>>
      auto prime_next_file_() -> void;
  };
}

#endif 
