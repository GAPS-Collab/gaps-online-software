#ifndef CALIBRATION_H_INCLUDED
#define CALIBRATION_H_INCLUDED

/********************************
 * Calibration related tasks
 *
 */

#include <vector>
#include <string>


/**
 * Read a text file with calibration constants.
 *
 *
 */ 

#include <vector>
#include <string>

#include "tof_typedefs.h"
#include "TOFCommon.h"
#include "blobroutines.h"
#include "events.h"


/**
 * The original "RemoveSpikes" from J. Zweerink
 *
 */
void spike_cleaning_jeff(Vec<f32> &voltages);



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


/**
 * Read a file with calibration constants.
 *
 */
std::vector<Calibrations_t> read_calibration_file (std::string filename);


void apply_tcal(const u16 stop_cell,
                const Calibrations_t &cal,
                f32 times[NWORDS]);

/**
 * Apply voltage calibration constants to raw adc data
 *
 */ 
void apply_vcal(const u16 stop_cell,
                const i16 adc[NWORDS],
                const Calibrations_t &cal,
                f32 waveform[NWORDS]);

#endif
