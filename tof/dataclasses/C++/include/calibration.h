#ifndef CALIBRATION_H_INCLUDED
#define CALIBRATION_H_INCLUDED

/********************************
 * Calibration related tasks
 *
 */

#include <vector>
#include <string>

#include "TOFCommon.h"

/**
 * Read a text file with calibration constants.
 *
 *
 */ 

#include <vector>
#include <string>

#include "TOFCommon.h"
#include "blobroutines.h"
#include "tof_typedefs.h"

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
