/*! \file 
 * \brief Tof event classes
 *
 * An event is basically anything with an 
 * event id.
 * - events for individual readoutboards
 *   - RBEventHeader     : header information of event
 *   - RBEvent           : contains header + active channels
 *   - RBWaveform        : A single waveform - this is for the 
 *                         telemetry stream, since larger packets
 *                         would be too big
 *
 *  For actual flight code, see the rust project 
 *
 *  This file is part of gaps-online-software and published 
 *  under the GPLv3 license
 *  
 */
#ifndef TOFEVENTS_H_INCLUDED
#define TOFEVENTS_H_INCLUDED

#include <tuple>
#include <array>
#include <format>

#include "result/result.h"

#include "tof_typedefs.h"
#include "packets/monitoring.h"
#include "packets/tof_packet.h"
#include "events/event_status.hpp"
#include "events/rb_event_header.hpp"
#include "events/hit_quality.hpp"
#include "events/post_flight_correction_functions.hpp"
#include "events/tof_hit.hpp"
#include "events/rb_event.hpp"
#include "events/event_quality.hpp"
#include "events/rb_waveform.hpp"
#include "events/tof_event.hpp"
#include "events/tof_event_summary.hpp"
#include "events/telemetry_event.hpp"
#include "calibration.h"
#include "version.h"
#include "errors.hpp"
#ifdef BUILD_CXX_DB
#include "database.h"
#endif

namespace r = result;

class RBCalibration;

#define NCHN 9
#define NWORDS 1024
#define N_LTBS 25
#define N_CHN_PER_LTB 16

namespace gondola {
  
  /*********************************************************/
  
  /// Speed of light in the paddle in cm/ns
  static const f32 C_LIGHT_PADDLE = 15.4; 
  
  /*********************************************************/

  enum class CompressionLevel : u8 {
    Unknown        =  0,
    None           = 10,
  };
  
  std::ostream& operator<<(std::ostream& os, const gondola::CompressionLevel& level);  
} 


#endif 
