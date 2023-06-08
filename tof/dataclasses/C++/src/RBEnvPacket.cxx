#include "packets/RBEnvPacket.h"
#include "serialization.h"

std::string RBEnvPacket::to_string() const
{
  std::string repr = "== REBEnvPacket (RB " + std::to_string(rb_id) + " ):\n";
  repr += "temperature " + std::to_string(temperature)     + "\n";
  repr += "voltage "     + std::to_string(voltage)         + "\n";
  repr += "current"      + std::to_string(current)         + "\n";
  repr += "power "       + std::to_string(power)           + "\n";
  repr += "preamp_temp"  + std::to_string(preamp_temp)     + "\n";
  repr += "preamp_bias"  + std::to_string(preamp_bias)     + "\n";
  repr += "temp_rb"      + std::to_string(temperature_rb)  + "\n";
  repr += "voltage_rb"   + std::to_string(voltage_rb)      + "\n";
  repr += "current_rb"   + std::to_string(current_rb)      + "\n";
  repr += "power_rb"     + std::to_string(power_rb)        + "\n";
  repr += "lol_stat"     + std::to_string(lol_status)      + "\n";
  return repr;
}

/********************************************/

void RBEnvPacket::reset() 
{
  temperature     = 0;
  voltage         = 0;
  current         = 0;
  power           = 0;
  preamp_temp     = 0;
  preamp_bias     = 0;
  temperature_rb  = 0;
  voltage_rb      = 0;
  current_rb      = 0;
  power_rb        = 0;
  lol_status      = 0;
}

/********************************************/

void RBEnvPacket::fill_from_envdata(RBEnvData* env_data)
{
  temperature    = encode_12bitsensor(env_data->temperature   ,TEMPSENSMIN, TEMPSENSMAX);
  voltage        = encode_12bitsensor(env_data->voltage       ,VOLTSENSMIN, VOLTSENSMAX);
  current        = encode_12bitsensor(env_data->current       ,CURRENTSENSMIN, CURRENTSENSMAX);
  power          = encode_12bitsensor(env_data->power         ,POWERSENSMIN, POWERSENSMAX);
  preamp_temp    = encode_12bitsensor(env_data->preamp_temp   ,PREAMPTEMPSENSMIN, PREAMPTEMPSENSMAX);
  preamp_bias    = encode_12bitsensor(env_data->preamp_bias   ,PREAMPBIASSENSMIN, PREAMPBIASSENSMAX);
  temperature_rb = encode_12bitsensor(env_data->temperature_rb,TEMPRBSENSMIN, TEMPRBSENSMAX);
  voltage_rb     = encode_12bitsensor(env_data->voltage_rb    ,VOLTRBSENSMIN, VOLTRBSENSMAX);
  current_rb     = encode_12bitsensor(env_data->current_rb    ,CURRENTRBSENSMIN, CURRENTRBSENSMAX);
  power_rb       = encode_12bitsensor(env_data->power_rb      ,POWERRBSENSMIN, POWERRBSENSMAX);
  lol_status     = encode_12bitsensor(env_data->lol_status    ,TEMPSENSMIN, TEMPSENSMAX);
}

/********************************************/

std::vector<uint8_t>RBEnvPacket::serialize() const
{
  std::vector<uint8_t> buffer(RBENVPACKETSIZE);
  uint16_t pos = 0; // position in bytestream
  encode_ushort(head,  buffer, pos); pos+=2;
  buffer[pos] = rb_id;

  encode_ushort(temperature     ,buffer,  pos);pos+=2;  
  encode_ushort(voltage         ,buffer,  pos);pos+=2; 
  encode_ushort(current         ,buffer,  pos);pos+=2;
  encode_ushort(power           ,buffer,  pos);pos+=2;
  encode_ushort(preamp_temp     ,buffer,  pos);pos+=2;
  encode_ushort(preamp_bias     ,buffer,  pos);pos+=2;
  encode_ushort(temperature_rb  ,buffer,  pos);pos+=2;
  encode_ushort(voltage_rb      ,buffer,  pos);pos+=2;
  encode_ushort(current_rb      ,buffer,  pos);pos+=2;
  encode_ushort(power_rb        ,buffer,  pos);pos+=2;
  encode_ushort(lol_status      ,buffer,  pos);pos+=2;  

  encode_ushort(tail, buffer, pos); pos+=2;  // done
  return buffer; 
}

/********************************************/

uint32_t RBEnvPacket::deserialize(std::vector<unsigned char>& payload,
                                  uint32_t start_pos)
{
  reset ();
  // start from position in payload
  //unsigned short value; 
  //unsigned int end_pos = start_pos;
  // check if we find the header at start_pos
  uint16_t value = decode_ushort(payload, start_pos);
  if (!(value == head))
    {std::cerr << "[ERROR] no header found!" << std::endl;}
  uint16_t pos = 2 + start_pos; // position in bytestream, 2 since we 
                                // just decoded the header
  rb_id           = payload[pos]; pos += 1;
  temperature     = decode_ushort(payload, pos); pos+=2;
  voltage         = decode_ushort(payload, pos); pos+=2;
  current         = decode_ushort(payload, pos); pos+=2; 
  power           = decode_ushort(payload, pos); pos+=2;
  preamp_temp     = decode_ushort(payload, pos); pos+=2;
  preamp_bias     = decode_ushort(payload, pos); pos+=2;
  temperature_rb  = decode_ushort(payload, pos); pos+=2;
  voltage_rb      = decode_ushort(payload, pos); pos+=2;
  current_rb      = decode_ushort(payload, pos); pos+=2;
  power_rb        = decode_ushort(payload, pos); pos+=2;
  lol_status      = decode_ushort(payload, pos); pos+=2;

  // sanity check
  if (head != decode_ushort(payload, pos))
    {
      std::cerr << "[WARN] tof env package is broken!" << std::endl;
      //is_broken = true;
    }
  return pos;
}

/*********************************************/

uint8_t RBEnvPacket::get_rb_id()
{
  return rb_id;
}

i32 RBEnvPacket::get_temperature()
{
  i32 value = decode_12bitsensor(temperature, TEMPSENSMIN, TEMPSENSMAX);
  return value;
}

i32 RBEnvPacket::get_voltage()
{
  i32 value = decode_12bitsensor(voltage, VOLTSENSMIN, VOLTSENSMAX);
  return value;
}

i32 RBEnvPacket::get_current()
{
  i32 value = decode_12bitsensor(current, CURRENTSENSMIN, CURRENTSENSMAX);
  return value;
}

i32 RBEnvPacket::get_power()
{
  i32 value = decode_12bitsensor(power, POWERSENSMIN, POWERSENSMAX);
  return value;
}

i32 RBEnvPacket::get_preamp_temp()
{
  i32 value = decode_12bitsensor(preamp_temp, PREAMPTEMPSENSMIN, PREAMPTEMPSENSMAX);
  return value;
}

i32 RBEnvPacket::get_preamp_bias()
{
  i32 value = decode_12bitsensor(preamp_bias, PREAMPBIASSENSMIN, PREAMPBIASSENSMAX);
  return value;
}

i32 RBEnvPacket::get_temperature_rb()
{
  i32 value = decode_12bitsensor(temperature_rb, TEMPRBSENSMIN, TEMPRBSENSMAX);
  return value;
}

i32 RBEnvPacket::get_voltage_rb()
{
  i32 value = decode_12bitsensor(voltage_rb, VOLTRBSENSMIN, VOLTRBSENSMAX);
  return value;
}

i32 RBEnvPacket::get_current_rb()
{
  i32 value = decode_12bitsensor(current_rb, CURRENTRBSENSMIN, CURRENTRBSENSMAX);
  return value;
}

i32 RBEnvPacket::get_power_rb()
{
  i32 value = decode_12bitsensor(power_rb, POWERRBSENSMIN, POWERRBSENSMAX);
  return value;
}

i32 RBEnvPacket::get_lol_status()
{
  i32 value = decode_12bitsensor(lol_status, TEMPSENSMIN, TEMPSENSMAX);
  return value;
}

// setters
void RBEnvPacket::set_rb_id         (uint8_t readoutboard_id)
{
  rb_id = readoutboard_id;
}

void RBEnvPacket::set_temperature   (i32 value)
{
  temperature = encode_12bitsensor(value, TEMPSENSMIN, TEMPSENSMAX );
}

void RBEnvPacket::set_voltage       (i32 value)
{
  voltage = encode_12bitsensor(value, VOLTSENSMIN, VOLTSENSMAX);
}

void RBEnvPacket::set_current       (i32 value)
{
  current = encode_12bitsensor(value, CURRENTSENSMIN, CURRENTSENSMAX);
}

void RBEnvPacket::set_power         (i32 value)
{
  power = encode_12bitsensor(value, POWERSENSMIN, POWERSENSMAX );
}

void RBEnvPacket::set_preamp_temp   (i32 value)
{
  preamp_temp = encode_12bitsensor(value, PREAMPTEMPSENSMIN, PREAMPTEMPSENSMAX);
}

void RBEnvPacket::set_preamp_bias   (i32 value)
{
  preamp_bias = encode_12bitsensor(value, PREAMPBIASSENSMIN, PREAMPBIASSENSMAX);
}

void RBEnvPacket::set_temperature_rb(i32 value)
{
  temperature_rb = encode_12bitsensor(value, TEMPRBSENSMIN, TEMPRBSENSMAX);
}

void RBEnvPacket::set_voltage_rb    (i32 value)
{
  voltage_rb = encode_12bitsensor(value, VOLTRBSENSMIN, VOLTRBSENSMAX );
}

void RBEnvPacket::set_current_rb    (i32 value)
{
  current_rb = encode_12bitsensor(value, CURRENTRBSENSMIN, CURRENTRBSENSMAX );
}

void RBEnvPacket::set_power_rb      (i32 value)
{
  power_rb = encode_12bitsensor(value, POWERRBSENSMIN, POWERRBSENSMAX );
}

void RBEnvPacket::set_lol_status    (i32 value)
{
  // FIXME
  lol_status = encode_12bitsensor(value, TEMPSENSMIN, TEMPSENSMAX );
}

/********************************************/

std::ostream& operator<<(std::ostream& os, const RBEnvPacket& h)
{
    os << h.to_string();
    return os;
}
