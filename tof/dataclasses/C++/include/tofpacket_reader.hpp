#ifndef TOFPACKETREADER_H_INCLUDED
#define TOFPACKETREADER_H_INCLUDED

#include <fstream>

#include "tof_typedefs.h"
#include "packets/TofPacket.h"

namespace Gaps {

  /// Read serialized TofPackets from 
  /// a file and emit them as packets
  class TofPacketReader {
    public: 
      TofPacketReader(String filename);
      void process_chunk();
      TofPacket get_next_packet();
      String get_filename() const;
    private:
      String filename_;
      usize  file_size_;
      usize  nchunks_;
      usize  current_pos_;
      TofPacket last_packet_;
      std::ifstream stream_file_;
  };
}

#endif
