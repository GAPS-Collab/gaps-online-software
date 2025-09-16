#ifndef CALIBRATION_H_INCLUDED
#define CALIBRATION_H_INCLUDED

/********************************
 * ReadoutBoard calibration:
 * - convert adc, time bins in 
 *   mV and nanoseconds.
 *******************************/

#include <vector>
#include <string>
#include <map>

#include "tof_typedefs.h"
#include "events.h"

class RBEvent;

namespace gondola {

  /// The original "RemoveSpikes" from the
  /// DRS4 manual
  auto spike_cleaning_drs4(Vec<Vec<f32>> &wf, u16 tCell, i32 spikes[]) -> void;
  
  /// An adjusted, simpler version of the spike cleaing written by Jamie
  auto spike_cleaning_simple(Vec<Vec<f32>> &voltages, bool calibrated = true) -> void;
  
  /// Jamie's simpler version with single-width spike correction
  auto spike_cleaning_all(Vec<Vec<f32>> &voltages, bool calibrated = true) -> void;

  /// Readoutboard constants for timing and voltage calibration for 
  /// a single board
  struct RBCalibration {
    static constexpr u16 HEAD = 0xAAAA;
    static constexpr u16 TAIL = 0x5555;
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
    static auto from_bytestream(const Vec<u8> &bytestream, u64 &pos, 
                                bool discard_events = true)
       -> RBCalibration;
  
    /// Load a calibration from a file with a TofPacket of 
    /// type RBCalibration in it. This should be the default
    /// way to load a calibration file
    static auto from_file(const String &filename, bool discard_events = true)
      -> RBCalibration;
    /// String representation for printing 
    auto to_string() const -> std::string;
  
    /// Should the associated data be loaded 
    /// in case it is available when 
    /// from_bytestream is called?
    static void disable_eventdata();
  
    private:
  
      /// Check if the channel follows the convention 1-9
      auto channel_check(u8 channel) const -> bool;
  };

  /// convenience function to load all the calibration files from a certain directory
  auto load_tof_calibrations(std::string const &pathname) -> std::map<u8, RBCalibration>;

  /// shortcut for the typically used map of rb_id -> calibrations
  typedef std::map<u8, RBCalibration> RBCalibrationMap;
} // end namespace

std::ostream& operator<<(std::ostream& os, const gondola::RBCalibration& pck);


#endif
