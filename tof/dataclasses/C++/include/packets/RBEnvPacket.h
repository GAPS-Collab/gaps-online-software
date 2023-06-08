#ifndef RBENVPACKET_H_INCLUDED
#define RBENVPACKET_H_INCLUDED

#include <string>
#include <vector>
#include <ostream>

#include "RBEnvData.h"
#include "tof_typedefs.h"

// size of the packet is fixed, in bytes
// 11 fields a 2 bytes, + 2 * 2 bytes header and tail each + 1 byte rbid
static const u8 RBENVPACKETSIZE = 27;

// sensor ranges
static const i32 TEMPSENSMAX = 100;
static const i32 TEMPSENSMIN = 0;

static const i32 VOLTSENSMAX = 100;
static const i32 VOLTSENSMIN = 0;

static const i32 CURRENTSENSMAX = 100;
static const i32 CURRENTSENSMIN = 0;

static const i32 POWERSENSMAX = 100;
static const i32 POWERSENSMIN = 0;

static const i32 PREAMPTEMPSENSMAX = 100;
static const i32 PREAMPTEMPSENSMIN = 0;

static const i32 PREAMPBIASSENSMAX = 100;
static const i32 PREAMPBIASSENSMIN = 0;

static const i32 TEMPRBSENSMAX = 100; 
static const i32 TEMPRBSENSMIN = 0;

static const i32 VOLTRBSENSMAX = 100; 
static const i32 VOLTRBSENSMIN = 0;

static const i32 CURRENTRBSENSMAX = 100;
static const i32 CURRENTRBSENSMIN = 0;

static const i32 POWERRBSENSMAX = 100;
static const i32 POWERRBSENSMIN = 0;

struct RBEnvPacket {

 public:
  u16 head = 0xAAAA;
  u16 tail = 0x5555;

  // readout board id
  u8 rb_id;

  // 12 bit sensors    
  u16 temperature;
  u16 voltage;
  u16 current;
  u16 power;
  u16 preamp_temp;
  u16 preamp_bias;
  u16 temperature_rb;
  u16 voltage_rb;
  u16 current_rb;
  u16 power_rb;
  u16 lol_status;
 
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
  u8 get_rb_id(); 
  i32 get_temperature();
  i32 get_voltage();
  i32 get_current();
  i32 get_power();
  i32 get_preamp_temp();
  i32 get_preamp_bias();
  i32 get_temperature_rb();
  i32 get_voltage_rb();
  i32 get_current_rb();
  i32 get_power_rb();
  i32 get_lol_status();

  // setters
  void set_rb_id         (u8 rb_id); 
  void set_temperature   (i32 value);
  void set_voltage       (i32 value);
  void set_current       (i32 value);
  void set_power         (i32 value);
  void set_preamp_temp   (i32 value);
  void set_preamp_bias   (i32 value);
  void set_temperature_rb(i32 value);
  void set_voltage_rb    (i32 value);
  void set_current_rb    (i32 value);
  void set_power_rb      (i32 value);
  void set_lol_status    (i32 value);

};

std::ostream& operator<<(std::ostream& os, const RBEnvPacket& h);



#endif
