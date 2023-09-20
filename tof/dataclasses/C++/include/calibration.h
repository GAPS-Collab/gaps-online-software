#ifndef CALIBRATION_H_INCLUDED
#define CALIBRATION_H_INCLUDED

/********************************
 * ReadoutBoard calibration:
 * - convert adc, time bins in 
 *   mV and nanoseconds.
 */

#include <vector>
#include <string>

#include "tof_typedefs.h"
#include "TOFCommon.h"
#include "blobroutines.h"
#include "events.h"



/**
 * The original "RemoveSpikes" from the
 * DRS4 manual
 *
 */
void spike_cleaning_drs4(Vec<f32> &voltages);



/** 
 * A set of calibration constants for a single readoutboard
 *
 */ 
struct RBCalibration {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;

  u8 rb_id;
  f32 d_v;
  bool serialize_event_data;
  Vec<Vec<f32>> v_offsets;
  Vec<Vec<f32>> v_dips;
  Vec<Vec<f32>> v_incs;
  Vec<Vec<f32>> t_bin;
  // data used to calculate calibration constants
  Vec<RBEvent> noi_data;
  Vec<RBEvent> vcal_data;
  Vec<RBEvent> tcal_data;

  RBCalibration();

  Vec<Vec<f32>> voltages    (const RBEventMemoryView &event, bool spike_cleaning = false) const;
  Vec<Vec<f32>> voltages    (const RBEvent &event, bool spike_cleaning = false) const;
  Vec<Vec<f32>> nanoseconds (const RBEventMemoryView &event) const;
  Vec<Vec<f32>> nanoseconds (const RBEvent &event) const;
  

  Vec<f32> voltages   (const RBEventMemoryView &event, const u8 channel) const;
  Vec<f32> nanoseconds(const RBEventMemoryView &event, const u8 channel) const;
  Vec<f32> voltages   (const RBEvent &event, const u8 channel) const;
  Vec<f32> nanoseconds(const RBEvent &event, const u8 channel) const;

  static RBCalibration from_bytestream(const Vec<u8> &bytestream,
                                       u64 &pos);

  static RBCalibration from_txtfile(const String &filename);
 
  std::string to_string() const;

  private:

    //! Check if the channel follows the convention 1-9
    bool channel_check(u8 channel) const;
};

std::ostream& operator<<(std::ostream& os, const RBCalibration& pck);

#endif
