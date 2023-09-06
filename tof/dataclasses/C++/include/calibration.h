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

  u8 rb_id;
  Vec<Vec<f32>> v_offsets;
  Vec<Vec<f32>> v_dips;
  Vec<Vec<f32>> v_incs;
  Vec<Vec<f32>> t_bin;

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

  private:

    /**
     * Check if the channel follows the convention 1-9
     *
     */
    bool channel_check(u8 channel) const;
};

#endif
