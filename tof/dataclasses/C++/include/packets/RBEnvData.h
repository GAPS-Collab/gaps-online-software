#ifndef RBENVDATA_H_INCLUDED
#define RBENVDATA_H_INCLUDED


typedef struct {
  float temperature;
  float voltage;
  float current;
  float power;
  float preamp_temp;
  float preamp_bias;
  float temperature_rb;
  float voltage_rb;
  float current_rb;
  float power_rb;
  float lol_status;
} RBEnvData;

#endif 
