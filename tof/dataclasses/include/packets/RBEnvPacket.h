#ifndef RBENVPACKET_H_INCLUDED
#define RBENVPACKET_H_INCLUDED

#include <string>
#include <vector>
#include <ostream>

#include "RBEnvData.h"

// size of the packet is fixed, in bytes
// 11 fields a 2 bytes, + 2 * 2 bytes header and tail each + 1 byte rbid
static const uint8_t RBENVPACKETSIZE = 27;

// sensor ranges
static const float TEMPSENSMAX = 100;
static const float TEMPSENSMIN = 0;

static const float VOLTSENSMAX = 100;
static const float VOLTSENSMIN = 0;

static const float CURRENTSENSMAX = 100;
static const float CURRENTSENSMIN = 0;

static const float POWERSENSMAX = 100;
static const float POWERSENSMIN = 0;

static const float PREAMPTEMPSENSMAX = 100;
static const float PREAMPTEMPSENSMIN = 0;

static const float PREAMPBIASSENSMAX = 100;
static const float PREAMPBIASSENSMIN = 0;

static const float TEMPRBSENSMAX = 100; 
static const float TEMPRBSENSMIN = 0;

static const float VOLTRBSENSMAX = 100; 
static const float VOLTRBSENSMIN = 0;

static const float CURRENTRBSENSMAX = 100;
static const float CURRENTRBSENSMIN = 0;

static const float POWERRBSENSMAX = 100;
static const float POWERRBSENSMIN = 0;

struct RBEnvPacket {

 public:
  uint16_t head = 0xAAAA;
  uint16_t tail = 0x5555;

  // readout board id
  uint8_t rb_id;

  // 12 bit sensors    
  uint16_t temperature;
  uint16_t voltage;
  uint16_t current;
  uint16_t power;
  uint16_t preamp_temp;
  uint16_t preamp_bias;
  uint16_t temperature_rb;
  uint16_t voltage_rb;
  uint16_t current_rb;
  uint16_t power_rb;
  uint16_t lol_status;
 
 public:
  void fill_from_envdata(RBEnvData* env_data);


  /**
   * String representation
   *
   */
  std::string to_string() const;

  /**
   * Reset all fields to 0 values
   * FIXME - nan would be better
   */
  void reset();

  /**
   * Transcode to bytestream
   *
   *
   */
  std::vector<unsigned char>serialize() const;

  /**
   * Transcode from bytestream
   *
   * Returns:
   *    position where the event is found in the bytestream
   *    (tail position +=1, so that bytestream can be iterated
   *    over easily)
   */
  uint32_t deserialize(std::vector<unsigned char>& payload,
                       uint32_t start_pos=0);


  // getters
  uint8_t get_rb_id(); 
  float get_temperature();
  float get_voltage();
  float get_current();
  float get_power();
  float get_preamp_temp();
  float get_preamp_bias();
  float get_temperature_rb();
  float get_voltage_rb();
  float get_current_rb();
  float get_power_rb();
  float get_lol_status();

  // setters
  void set_rb_id         (uint8_t rb_id); 
  void set_temperature   (float value);
  void set_voltage       (float value);
  void set_current       (float value);
  void set_power         (float value);
  void set_preamp_temp   (float value);
  void set_preamp_bias   (float value);
  void set_temperature_rb(float value);
  void set_voltage_rb    (float value);
  void set_current_rb    (float value);
  void set_power_rb      (float value);
  void set_lol_status    (float value);

};

std::ostream& operator<<(std::ostream& os, const RBEnvPacket& h);



#endif
