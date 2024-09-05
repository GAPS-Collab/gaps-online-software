#ifndef CALIBRATION_H_INCLUDED
#define CALIBRATION_H_INCLUDED

/********************************
 * ReadoutBoard calibration:
 * - convert adc, time bins in 
 *   mV and nanoseconds.
 *******************************/

#include <vector>
#include <string>

#include "tof_typedefs.h"
#include "events.h"

class RBEvent;


/// The original "RemoveSpikes" from the
/// DRS4 manual
void spike_cleaning_drs4(Vec<Vec<f32>> &wf, u16 tCell, i32 spikes[]);

/// An adjusted, simpler version of the spike cleaing written by Jamie
void spike_cleaning_simple(Vec<Vec<f32>> &voltages, bool calibrated = true);

/// Jamie's simpler version with single-width spike correction
void spike_cleaning_all(Vec<Vec<f32>> &voltages, bool calibrated = true);

/** 
 * A set of calibration constants for a single readoutboard
 *
 */ 
struct RBCalibration {
  static const u16 HEAD = 0xAAAA;
  static const u16 TAIL = 0x5555;
  static bool serialize_event_data;

  /// id of the RB this calibration belongs to
  u8 rb_id;
  /// voltage difference between noi and voltage data
  f32 d_v;
  /// timestamp when the calibration has been taken
  u32 timestamp;
  Vec<Vec<f32>> v_offsets;
  Vec<Vec<f32>> v_dips;
  Vec<Vec<f32>> v_incs;
  Vec<Vec<f32>> t_bin;
  // data used to calculate calibration constants
  /// The no-input data used to calculate the constants
  Vec<RBEvent> noi_data;
  /// The constant voltage data used to calculate the constants
  Vec<RBEvent> vcal_data;
  /// The timing calibration data used to calculate the constants
  Vec<RBEvent> tcal_data;

  RBCalibration();

  /// get the voltage values for the traces of the event
  Vec<Vec<f32>> voltages    (const RBEvent &event, bool spike_cleaning = false) const;
  Vec<Vec<f32>> nanoseconds (const RBEvent &event) const;
  
  Vec<f32> voltages   (const RBEvent &event, const u8 channel) const;
  Vec<f32> nanoseconds(const RBEvent &event, const u8 channel) const;

  /**
   * Factory function for RBCalibration
   *
   * @param
   * @param
   * @param 
   */
  static RBCalibration from_bytestream(const Vec<u8> &bytestream,
                                       u64 &pos,
                                       bool discard_events = true);

  /// load a calibration from a txt file with constants
  /// This does not allow to load the data assigned to 
  /// the calibration
  static RBCalibration from_txtfile(const String &filename);


  /// Load a calibration from a file with a TofPacket of 
  /// type RBCalibration in it. This should be the default
  /// way to load a calibration file
  static RBCalibration from_file(const String &filename,
                                 bool discard_events = true);
  /// String representation for printing 
  std::string to_string() const;

  /// Should the associated data be loaded 
  /// in case it is available when 
  /// from_bytestream is called?
  static void disable_eventdata();

  private:

    /// Check if the channel follows the convention 1-9
    bool channel_check(u8 channel) const;
};

std::ostream& operator<<(std::ostream& os, const RBCalibration& pck);

#endif
