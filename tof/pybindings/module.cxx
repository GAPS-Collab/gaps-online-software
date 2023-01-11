#include <fstream>

#include <pybind11/pybind11.h>
#include <pybind11/stl.h>
#include <pybind11/complex.h>
#include <pybind11/functional.h>
#include <pybind11/chrono.h>

#include "packets/REventPacket.h"
#include "packets/RPaddlePacket.h"
#include "packets/TofPacket.h"
#include "packets/CommandPacket.h"
#include "packets/RBEnvPacket.h"

#include "serialization.h"
#include "blobroutines.h"
#include "WaveGAPS.h"
#include "TOFCommon.h"

#include "TofTypeDefs.h"

using namespace GAPS;

/***********************************************/

bytestream wrap_encode_ushort(u16 value, u32 start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<2; foo++) stream.push_back(0);
  encode_ushort(value, stream, start_pos);
  return stream;
}

/***********************************************/

bytestream wrap_encode_ushort_rev(u16 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<2; foo++) stream.push_back(0);
  encode_ushort_rev(value, stream, start_pos);
  return stream;
}

/***********************************************/

bytestream wrap_encode_uint32(u32 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<4; foo++) stream.push_back(0);
  encode_uint32(value, stream, start_pos);
  return stream;
}

/***********************************************/

bytestream wrap_encode_uint32_rev(u32 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<4; foo++) stream.push_back(0);
  encode_uint32_rev(value, stream, start_pos);
  return stream;
}

/***********************************************/

bytestream wrap_encode_uint64_rev(u64 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<8; foo++) stream.push_back(0);
  encode_uint64_rev(value, stream, start_pos);
  return stream;
}

/***********************************************/

bytestream wrap_encode_uint64(u64 value, size_t start_pos) {
  bytestream stream;
  for (size_t foo=0; foo<8; foo++) stream.push_back(0);
  encode_uint64(value, stream, start_pos);
  return stream;
}

/********************/
// helpers

int static_helper(RPaddlePacket& pp){
    return RPaddlePacket::calculate_length();
}

std::string tof_command_to_str(const TofCommand &cmd) {
 switch (cmd) {
   case TofCommand::PowerOn               : {return "PowerOn"              ;}
   case TofCommand::PowerOff              : {return "PowerOff"             ;}
   case TofCommand::PowerCycle            : {return "PowerCycle"           ;} 
   case TofCommand::RBSetup               : {return "RBSetup"              ;}
   case TofCommand::SetThresholds         : {return "SetThresholds"        ;} 
   case TofCommand::SetMtConfig           : {return "SetMtConfig"          ;}
   case TofCommand::StartValidationRun    : {return "StartValidationRun"   ;}
   case TofCommand::RequestWaveforms      : {return "RequestWaveforms"     ;}
   case TofCommand::DataRunStart          : {return "DataRunStart"         ;}
   case TofCommand::DataRunEnd            : {return "DataRunEnd"           ;}
   case TofCommand::VoltageCalibration    : {return "VoltageCalibration"   ;}
   case TofCommand::TimingCalibration     : {return "TimingCalibration"    ;}
   case TofCommand::CreateCalibrationFile : {return "CreateCalibrationFile";}
   case TofCommand::RequestEvent          : {return "RequestEvent"         ;}
   case TofCommand::RequestMoni           : {return "RequestMoni"          ;}
   case TofCommand::Unknown               : {return "Unknown"              ;}
 } // end case   
 return "Unknown";
}

vec_u16 ch_head_getter(BlobEvt_t evt)
{
    vec_u16 ch_head;
    for (size_t k=0; k<NCHN; k++) 
    {ch_head.push_back(evt.ch_head[k]);}
    return ch_head;
}

vec_u64 ch_trail_getter(BlobEvt_t evt)
{
    vec_u64 ch_trail;
    for (size_t k=0; k<NCHN; k++) 
    {ch_trail.push_back(evt.ch_trail[k]);}
    return ch_trail;
}

vec_vec_i16 ch_getter(BlobEvt_t evt)
{
    vec_vec_i16 channels;
    for (size_t k=0; k<NCHN; k++) 
      {  channels.push_back({});
         for (size_t l=0; l < NWORDS; l++)
            {
               channels[k].push_back(evt.ch_adc[k][l]);
            }
      }
    return channels;
}

size_t get_current_blobevent_size()
{
  return 36 + (NCHN*2) + (NCHN*NWORDS*2) + (NCHN*4) + 8;
}

bytestream blobevent_encoder(BlobEvt_t evt, size_t startpos)
{
  bytestream buffer;
  buffer.reserve(get_current_blobevent_size());
  for (size_t k=0; k<get_current_blobevent_size(); k++)
  {buffer.push_back(0);}
  encode_blobevent(&evt, buffer, startpos);
  return buffer;
}

BlobEvt_t blobevent_decoder(bytestream buffer, size_t startpos)
{
  BlobEvt_t evt = decode_blobevent(buffer, startpos);
  return evt;
}

std::string BlobEvtToString(BlobEvt_t event)
{
   std::string output = "";
   output += "head "      + std::to_string(event.head )      + "\n" ;
   output += "status "    + std::to_string(event.status )    + "\n" ;
   output += "len "       + std::to_string(event.len )       + "\n" ;
   output += "roi "       + std::to_string(event.roi )       + "\n" ;
   output += "dna "       + std::to_string(event.dna )       + "\n" ;
   output += "fw_hash "   + std::to_string(event.fw_hash )   + "\n" ;
   output += "id "        + std::to_string(event.id )        + "\n" ;
   output += "ch_mask "   + std::to_string(event.ch_mask )   + "\n" ;
   output += "event_ctr " + std::to_string(event.event_ctr ) + "\n" ;
   output += "dtap0 "     + std::to_string(event.dtap0 )     + "\n" ;
   output += "dtap1 "     + std::to_string(event.dtap1 )     + "\n" ;
   output += "timestamp " + std::to_string(event.timestamp ) + "\n" ;
   output += "stop_cell " + std::to_string(event.stop_cell ) + "\n" ;
   output += "crc32 "     + std::to_string(event.crc32 )     + "\n" ;
   output += "tail "      + std::to_string(event.tail)       ;
   return output;
}

template<class T>
void nullsetter(T foo) 
{
    std::cerr << "Can not set this property!" << std::endl;
}

void set_payload_helper(TofPacket &packet,
                        const vec_u8 payload)
{
    packet.payload = payload;
    packet.payload_size = payload.size();
}

void set_ptype_helper(TofPacket &packet,
                      const PacketType &ptype)
{
    packet.packet_type = ptype;
}

/********************/

BlobEvt_t read_event_helper(std::string filename, i32 n)
{
    FILE* f = fopen(filename.c_str(), "rb");
    BlobEvt_t event;
    while(n >= 0) {
      ReadEvent(f, &event, false);
      n--;
    }
    return event;
}

/********************/

int get_nevents_from_file(std::string filename){
  FILE* f = fopen(filename.c_str(), "rb");
  BlobEvt_t event;
  i32 result = 0;
  u32 nevents = 0;
  while (result >= 0) {
    result = ReadEvent(f, &event, false);
    nevents++;
  }
  return nevents;
}

/********************/

std::vector<Calibrations_t> read_calibration_file (std::string filename) {
  std::vector<Calibrations_t> all_channel_calibrations = std::vector<Calibrations_t>{NCHN};
  std::fstream calfile(filename.c_str(), std::ios_base::in);
  if (calfile.fail()) {
    std::cerr << "[ERROR] Can't open " << filename << " - not calibrating" << std::endl;
    return all_channel_calibrations;
  }
  for (size_t i=0; i<NCHN; i++) {
    for (size_t j=0; j<NWORDS; j++)
      calfile >> all_channel_calibrations[i].vofs[j];
    for (size_t j=0; j<NWORDS; j++)
      calfile >> all_channel_calibrations[i].vdip[j];
    for (size_t j=0; j<NWORDS; j++)
      calfile >> all_channel_calibrations[i].vinc[j];
    for (size_t j=0; j<NWORDS; j++)
      calfile >> all_channel_calibrations[i].tbin[j];
  }
  return all_channel_calibrations;
}

/********************/

vec_vec_f64 offset_getter(const std::vector<Calibrations_t> &cal)
{
  vec_vec_f64 offsets;
  for (size_t k=0; k<NCHN; k++) 
    {  offsets.push_back({});
       for (size_t l=0; l < NWORDS; l++)
          {
             offsets[k].push_back(cal[k].vofs[l]);
          }
    }
  return offsets;
}

vec_vec_f64 dip_getter(const std::vector<Calibrations_t> &cal)
{
    vec_vec_f64 dips;
    for (size_t k=0; k<NCHN; k++) 
      {  dips.push_back({});
         for (size_t l=0; l < NWORDS; l++)
           {
             dips[k].push_back(cal[k].vdip[l]);
           }
      }
    return dips;
}

vec_vec_f64 increment_getter(const std::vector<Calibrations_t> &cal)
{
  vec_vec_f64 incs;
  for (size_t k=0; k<NCHN; k++) 
    {  incs.push_back({});
       for (uint l=0; l < NWORDS; l++)
        {
          incs[k].push_back(cal[k].vinc[l]);
        }
    }
  return incs;
}

vec_vec_f64 tbin_getter(const std::vector<Calibrations_t> cal)
{
    vec_vec_f64 tbins;
    for (size_t k=0; k<NCHN; k++) 
      {  tbins.push_back({});
         for (size_t l=0; l < NWORDS; l++)
            {
              tbins[k].push_back(cal[k].tbin[l]);
            }
      }
    return tbins;
}

vec_vec_f64 voltage_calibration_helper(const BlobEvt_t &evt,
                                                         std::vector<Calibrations_t> cal)
{
  vec_vec_f64 result;
  for (size_t k=0; k<NCHN; k++) {
    f64 trace_out[NWORDS];
    u32 n = sizeof(trace_out)/sizeof(trace_out[0]);
    VoltageCalibration(const_cast<short int*>(evt.ch_adc[k]),
                       trace_out,
                       evt.stop_cell,
                       cal[k]);
    result.push_back(std::vector<f64>(trace_out, trace_out + n));
  }
  return result;
}

vec_vec_f64 timing_calibration_helper(const BlobEvt_t &evt,
                                                            std::vector<Calibrations_t> cal)
{
  vec_vec_f64 result;
  for (size_t k=0; k<NCHN; k++) {
    f64 times[NWORDS];
    size_t n = sizeof(times)/sizeof(times[0]);
    TimingCalibration(times,
                      evt.stop_cell,
                      cal[k]);
    result.push_back(std::vector<double>(times, times + n));
  }
  return result;
}

/********************/

vec_vec_f64 remove_spikes_helper(vec_vec_f64 waveforms, 
                                 const BlobEvt_t &ev)
{  
   f64 wf [NCHN][NWORDS];
   i32 spikes[NWORDS];
   vec_vec_f64 unspiked;
   for (size_t ch=0; ch<NCHN; ch++) {
     unspiked.push_back({});
     for (size_t n=0; n<NWORDS; n++) {
       wf[ch][n] = waveforms[ch][n];
     }

   }
   RemoveSpikes(wf, ev.stop_cell, spikes);
   for (size_t ch=0; ch<NCHN; ch++) {
     for (size_t n=0; n<NWORDS; n++) {
       unspiked[ch].push_back(wf[ch][n]);
     } 
   }
   return unspiked;
}

/********************/

double calculate_pedestal_helper(vec_f64 wave,
                                 vec_f64 time,
                                 size_t ch)
{
  double* wave_arr = wave.data();
  double* time_arr = time.data();
  Waveform waveform = Waveform(wave_arr, time_arr, ch, 0);
  waveform.SetPedBegin(10); // 10-100                               
  waveform.SetPedRange(50);
  waveform.CalcPedestalRange();
  //waveform.SubtractPedestal();
  return waveform.GetPedestal();
}


/********************/

namespace py = pybind11;
PYBIND11_MODULE(gaps_tof, m) {
    m.doc() = "GAPS Tof dataclasses and utility tools";
   
    py::enum_<TofCommand>(m, "TofCommand")
     .value("PowerOn"              ,TofCommand::PowerOn) 
      .value("PowerOff"             ,TofCommand::PowerOff) 
      .value("PowerCycle"           ,TofCommand::PowerCycle) 
      .value("RBSetup"              ,TofCommand::RBSetup) 
      .value("SetThresholds"        ,TofCommand::SetThresholds) 
      .value("SetMtConfig"          ,TofCommand::SetMtConfig) 
      .value("StartValidationRun"   ,TofCommand::StartValidationRun) 
      .value("RequestWaveforms"     ,TofCommand::RequestWaveforms) 
      .value("DataRunStart"         ,TofCommand::DataRunStart) 
      .value("DataRunEnd"           ,TofCommand::DataRunEnd)    
      .value("VoltageCalibration"   ,TofCommand::VoltageCalibration) 
      .value("TimingCalibration"    ,TofCommand::TimingCalibration)
      .value("CreateCalibrationFile",TofCommand::CreateCalibrationFile) 
      .value("RequestEvent"         ,TofCommand::RequestEvent) 
      .value("RequestMoni"          ,TofCommand::RequestMoni) 
      .value("Unknown"              ,TofCommand::Unknown) 
      .export_values();

    py::class_<CommandPacket>(m, "CommandPacket") 
      .def(py::init<TofCommand const&, u32 const>())  
      .def("to_bytesream",    &CommandPacket::to_bytestream)
      .def("from_bytestream", &CommandPacket::from_bytestream)
      .def("get_command" ,    [](const CommandPacket &pk) {
                                  return pk.command;
                              })
      .def("__repr__",        [](const CommandPacket &pk) {
                                  return "<CommandPacket : "
                                  + tof_command_to_str(pk.command)
                                  + " "
                                  + std::to_string(pk.value) + ">";
                                  }) 
    ;

    py::enum_<PacketType>(m, "PacketType")
      .value("Unknown", PacketType::Unknown   )
      .value("Command", PacketType::Command   )
      .value("RBEvent", PacketType::RBEvent   )
      .value("Monitor", PacketType::Monitor   )
      .value("HeartBeat", PacketType::HeartBeat )
      .export_values();

    py::enum_<PADDLE_END>(m, "PADDLE_END")
        .value("A", PADDLE_END::A)
        .value("B", PADDLE_END::B)
        .value("UNKNOWN", PADDLE_END::UNKNOWN)
        .export_values();
    
    py::class_<REventPacket>(m, "REventPacket")
        .def(py::init())
        .def("serialize",         &REventPacket::serialize)
        .def("deserialize",       &REventPacket::deserialize)
        .def("calculate_length",  &REventPacket::calculate_length)
        .def("is_broken",         &REventPacket::is_broken)
        .def("reset",             &REventPacket::reset)
        .def("add_paddle_packet", &REventPacket::add_paddle_packet)
        .def_readwrite("event_id"           ,&REventPacket::event_ctr)
        .def_readwrite("n_paddles"          ,&REventPacket::n_paddles)
        .def_readwrite("utc_timestamp"      ,&REventPacket::utc_timestamp)
        .def_readwrite("primary_beta"       ,&REventPacket::primary_beta)
        .def_readwrite("primary_beta_unc"   ,&REventPacket::primary_beta_unc)
        .def_readwrite("primary_charge"     ,&REventPacket::primary_charge)
        .def_readwrite("primary_charge_unc" ,&REventPacket::primary_charge_unc)
        .def_readwrite("primary_outer_tof_x",&REventPacket::primary_outer_tof_x)
        .def_readwrite("primary_outer_tof_y",&REventPacket::primary_outer_tof_y)
        .def_readwrite("primary_outer_tof_z",&REventPacket::primary_outer_tof_z)
        .def_readwrite("primary_inner_tof_x",&REventPacket::primary_inner_tof_x)
        .def_readwrite("primary_inner_tof_y",&REventPacket::primary_inner_tof_y)
        .def_readwrite("primary_inner_tof_z",&REventPacket::primary_inner_tof_z)
        .def("__repr__",          [](const REventPacket &ev) {
                                  return "<REventPacket : " + ev.to_string(true) + "'>";
                                  }) 

    ;
    py::class_<TofPacket>(m, "TofPacket")
        .def(py::init())
        .def("to_bytestream",         &TofPacket::to_bytestream)
        .def("from_bytestream",       &TofPacket::from_bytestream)
        .def("set_payload",           &set_payload_helper)
        .def("set_packet_type",       &set_ptype_helper) 
        .def_readonly("head",         &TofPacket::head)
        .def_readonly("tail",         &TofPacket::tail)
        .def_readonly("payload",      &TofPacket::payload)
        .def_readonly("payload_size", &TofPacket::payload_size)
        .def_readonly("packet_type",  &TofPacket::packet_type)
        .def("__repr__",          [](const TofPacket &pkg) {
                                  return "<TofPacket : " + pkg.to_string() + "'>";
                                  }); 

    py::class_<RPaddlePacket>(m, "RPaddlePacket")
        .def(py::init())
        .def("serialize",         &RPaddlePacket::serialize)
        .def("deserialize",       &RPaddlePacket::deserialize)
        .def("calculate_length",  &static_helper)
        .def("reset",             &RPaddlePacket::reset)
        .def("get_paddle_id",     &RPaddlePacket::get_paddle_id) 
        .def("get_time_a",        &RPaddlePacket::get_time_a) 
        .def("get_time_b",        &RPaddlePacket::get_time_b) 
        .def("get_peak_a",        &RPaddlePacket::get_peak_a) 
        .def("get_peak_b",        &RPaddlePacket::get_peak_b) 
        .def("get_charge_a",      &RPaddlePacket::get_charge_a) 
        .def("get_charge_b",      &RPaddlePacket::get_charge_b) 
        .def("get_charge_min_i",  &RPaddlePacket::get_charge_min_i) 
        .def("get_x_pos",         &RPaddlePacket::get_x_pos) 
        .def("get_t_avg",         &RPaddlePacket::get_t_avg) 
        //.def("set_paddle_id",     &RPaddlePacket::set_paddle_id) 
        .def("set_time_a",        &RPaddlePacket::set_time_a) 
        .def("set_time_b",        &RPaddlePacket::set_time_b) 
        .def("set_peak_a",        &RPaddlePacket::set_peak_a) 
        .def("set_peak_b",        &RPaddlePacket::set_peak_b) 
        .def("set_charge_a",      &RPaddlePacket::set_charge_a) 
        .def("set_charge_b",      &RPaddlePacket::set_charge_b) 
        .def("set_charge_min_i",  &RPaddlePacket::set_charge_min_i) 
        .def("set_x_pos",         &RPaddlePacket::set_x_pos) 
        .def("set_t_avg",         &RPaddlePacket::set_t_avg) 

        // atributes
        .def_readwrite("paddle_id" ,    &RPaddlePacket::paddle_id)
        //.def_readwrite("time_a" ,       &RPaddlePacket::time_a)
        //.def_readwrite("time_b" ,       &RPaddlePacket::time_b)
        //.def_readwrite("charge_a" ,     &RPaddlePacket::charge_a)
        //.def_readwrite("charge_b" ,     &RPaddlePacket::charge_b)
        //.def_readwrite("charge_min_i" , &RPaddlePacket::charge_min_i)
        //.def_readwrite("x_pos" ,        &RPaddlePacket::x_pos)
        //.def_readwrite("t_avg" ,        &RPaddlePacket::t_average)

        .def("__repr__",          [](const RPaddlePacket &pp) {
                                  return "<RPaddlePacket : " + pp.to_string() + "'>";
                                  }) 

    ;


    py::class_<RBEnvPacket>(m, "RBEnvPacket")
        .def(py::init())
        .def("serialize",          &RBEnvPacket::serialize)
        .def("deserialize",        &RBEnvPacket::deserialize)
        .def("reset",              &RBEnvPacket::reset)
        .def("get_temperature",    &RBEnvPacket::get_temperature) 
        .def("get_voltage",        &RBEnvPacket::get_voltage) 
        .def("get_current",        &RBEnvPacket::get_current) 
        .def("get_power",          &RBEnvPacket::get_power) 
        .def("get_preamp_temp",    &RBEnvPacket::get_preamp_temp) 
        .def("get_preamp_bias",    &RBEnvPacket::get_preamp_bias) 
        .def("get_temperature_rb", &RBEnvPacket::get_temperature_rb) 
        .def("get_voltage_rb",     &RBEnvPacket::get_voltage_rb) 
        .def("get_current_rb",     &RBEnvPacket::get_current_rb) 
        .def("get_power_rb",       &RBEnvPacket::get_power_rb) 
        .def("get_lol_status",     &RBEnvPacket::get_lol_status) 
        .def("__repr__",          [](const RBEnvPacket &pp) {
                                  return "<RBEnvPacket : " + pp.to_string() + "'>";
                                  }) 
    ;

    py::class_<BlobEvt_t>(m, "BlobEvt")
       .def(py::init())
       .def_readwrite("head"                    ,&BlobEvt_t::head ) 
       .def_readwrite("status"                  ,&BlobEvt_t::status )
       .def_readwrite("len"                     ,&BlobEvt_t::len )
       .def_readwrite("roi"                     ,&BlobEvt_t::roi )
       .def_readwrite("dna"                     ,&BlobEvt_t::dna )
       .def_readwrite("fw_hash"                 ,&BlobEvt_t::fw_hash )
       .def_readwrite("id"                      ,&BlobEvt_t::id )
       .def_readwrite("ch_mask"                 ,&BlobEvt_t::ch_mask )
       .def_readwrite("event_ctr"               ,&BlobEvt_t::event_ctr )
       .def_readwrite("dtap0"                   ,&BlobEvt_t::dtap0 )
       .def_readwrite("dtap1"                   ,&BlobEvt_t::dtap1 )
       .def_readwrite("timestamp"               ,&BlobEvt_t::timestamp )
       //.def_property("ch_head"      , &ch_head_getter , &nullsetter<std::vector<unsigned short>  )
       .def("get_ch_head"                       ,&ch_head_getter)
       .def("get_ch_adc"                        ,&ch_getter )
       .def("get_ch_trail"                      ,&ch_trail_getter )
       .def_readwrite("stop_cell"               ,&BlobEvt_t::stop_cell )
       .def_readwrite("crc32"                   ,&BlobEvt_t::crc32 )
       .def_readwrite("tail"                    ,&BlobEvt_t::tail)
       .def("__repr__",  [] (const BlobEvt_t &event) { 
               return "<BlobEvt_t \n" + BlobEvtToString(event) + ">";
               })
   ;
    
   py::class_<Waveform>(m, "Waveform")
       .def(py::init<int>())
       .def("SetWave"             , &Waveform::SetWave            )
       .def("SetTime"             , &Waveform::SetTime            )
       .def("SetThreshold"        , &Waveform::SetThreshold       )
       .def("GetWaveSize"         , &Waveform::GetWaveSize        ) 
       .def("SetBin"              , &Waveform::SetBin             )
       .def("GetBin"              , &Waveform::GetBin             )
       .def("GetBinTime"          , &Waveform::GetBinTime         )

       // this is simply  not defined
       //.def("GetBinDC"            , &Waveform::GetBinDC           )
       .def("GetMaxBin"           , &Waveform::GetMaxBin          )
       .def("GetMaxBinTime"       , &Waveform::GetMaxBinTime      )
       .def("GetMaxVal"           , &Waveform::GetMaxVal          )
       .def("GetMinBin"           , &Waveform::GetMinBin          )
       .def("GetMinBinTime"       , &Waveform::GetMinBinTime      )
       .def("GetMinVal"           , &Waveform::GetMinVal          )
       .def("GetPeakValue"        , &Waveform::GetPeakValue       )
       .def("Rescale"             , &Waveform::Rescale            ) 
       .def("Integrate"           , &Waveform::Integrate          )
       .def("SetPedestal"         , &Waveform::SetPedestal        )
       .def("SetRunPedestal"      , &Waveform::SetRunPedestal     )
       .def("SetPedRange"         , &Waveform::SetPedRange        )
       .def("SetPedBegin"         , &Waveform::SetPedBegin        )
       .def("GetPedRange"         , &Waveform::GetPedRange        )
       .def("GetPedBegin"         , &Waveform::GetPedBegin        )
       .def("GetPedestal"         , &Waveform::GetPedestal        )
       .def("GetPedsigma"         , &Waveform::GetPedsigma        )
       .def("CalcPedestalRange"   , &Waveform::CalcPedestalRange  )
       .def("CalcPedestalDynamic" , &Waveform::CalcPedestalDynamic)
       .def("SubtractPedestal"    , &Waveform::SubtractPedestal   )
       .def("SetMaxPeaks"         , &Waveform::SetMaxPeaks        )
       .def("GetMaxPeaks"         , &Waveform::GetMaxPeaks        )
       .def("CleanUpPeaks"        , &Waveform::CleanUpPeaks       )
       .def("GetNumPeaks"         , &Waveform::GetNumPeaks        )
       .def("SetCFDSFraction"     , &Waveform::SetCFDSFraction    )
       .def("SetCFDEFraction"     , &Waveform::SetCFDEFraction    )
       .def("SetCFDEOffset"       , &Waveform::SetCFDEOffset      )
       .def("FindPeaks"           , &Waveform::FindPeaks          )
       .def("FindTdc"             , &Waveform::FindTdc            )
       .def("GetSpikes"           , &Waveform::GetSpikes          )
       .def("GetTdcs"             , &Waveform::GetTdcs            )
       .def("GetCharge"           , &Waveform::GetCharge          )
       .def("GetHeight"           , &Waveform::GetHeight          )
       .def("GetWidth"            , &Waveform::GetWidth           )
       .def("GetPulsepars"        , &Waveform::GetPulsepars       ) 
       .def("GetPulsechi2"        , &Waveform::GetPulsechi2       ) 
       .def("GetNDF"              , &Waveform::GetNDF             ) 
       .def("FitPulse"            , &Waveform::FitPulse           )
       .def("GetNsPerBin"         , &Waveform::GetNsPerBin        ) 
       .def("GetOffset"           , &Waveform::GetOffset          ) 
       .def("GetTimingCorr"       , &Waveform::GetTimingCorr      )
       .def("GetImpedance"        , &Waveform::GetImpedance       ) 
       .def("SetImpedance"        , &Waveform::SetImpedance       )
       .def("__repr__",  [] (const Waveform &waveform) { 
               return "<GAPSWaveform>";
               })
   ;
   
   py::class_<Calibrations_t>(m, "Calibrations")
       .def(py::init())
   ;

   // serialization functions
   m.def("decode_ushort",         &decode_ushort);
   m.def("encode_ushort",         &wrap_encode_ushort);
   m.def("encode_ushort_rev",     &wrap_encode_ushort_rev);
   
   m.def("decode_uint32",         &decode_uint32);
   m.def("encode_uint32",         &wrap_encode_uint32);
   m.def("encode_uint32_rev",     &wrap_encode_uint32_rev);

   m.def("encode_48",             &encode_48);
   m.def("encode_48_rev",         &encode_48_rev);

   m.def("decode_uint64",         &decode_uint64);
   m.def("encode_uint64",         &wrap_encode_uint64);
   m.def("encode_uint64_rev",     &wrap_encode_uint64_rev);

   m.def("encode_blobevent",      &blobevent_encoder);
   m.def("decode_blobevent",      &blobevent_decoder);   
   m.def("get_current_blobevent_size", &get_current_blobevent_size);

   // functions to read and parse blob files
   m.def("search_for_2byte_marker",  &search_for_2byte_marker);
   m.def("get_2byte_marker_indices", &get_2byte_markers_indices);
   m.def("get_events_from_stream",   &get_events_from_stream);
   m.def("get_nevents_from_file",    &get_nevents_from_file);
   m.def("ReadEvent",                &read_event_helper);

   m.def("voltage_calibration",      &voltage_calibration_helper);
   m.def("timing_calibration",       &timing_calibration_helper);
   m.def("remove_spikes",           &remove_spikes_helper);
   m.def("read_calibration_file",    &read_calibration_file);
   m.def("get_offsets",              &offset_getter);
   m.def("get_vincs",                &increment_getter);
   m.def("get_vdips",                &dip_getter);
   m.def("get_tbins",                &tbin_getter);
   
   // waveform stuff
   m.def("calculate_pedestal",       &calculate_pedestal_helper);
}
