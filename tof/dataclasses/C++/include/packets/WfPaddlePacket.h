#ifndef GAPS_WAVEFORMPADDLE_PACKET_H_INCLUDED
#define GAPS_WAVEFORMPADDLE_PACKET_H_INCLUDED

#include "RPaddlePacket.h"
#include "TOFCommon.h"

namespace GAPS {

/**********************************
 * A paddle packet also holding the 
 * waveform information
 *
 */
struct WfPaddlePacket {

    RPaddlePacket packet_;
    std::vector<float> get_waveform(GAPS::PADDLE_END side);
    std::vector<float> get_times(GAPS::PADDLE_END side);

    std::vector<unsigned char> serialize();
    void from_bytstream(std::vector<unsigned char> bytestream);
    //private:
    //std::vector<
};

}

#endif
