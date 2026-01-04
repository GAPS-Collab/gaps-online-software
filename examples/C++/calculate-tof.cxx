/**
 * Example to illustrate how to read L0 binary files,
 * get TofPackets and calculate the time-of-flight 
 * for an example paddle combination
 * To use this example, the code has to be build with
 * BUILD_CARASPACE=ON and BUILD_CXXDB
 * 
 * March 2025, gaps-online-sw V0.10
 * The API will not be stable until V1.0 and is thus 
 * subject to change. Please refer to the respective 
 * README.md
 *
 */

#include <iostream>
#include <filesystem>
#include <chrono>
#include <algorithm>

#include "cxxopts.hpp"

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "io.hpp"
#include "calibration.h"
#include "database.h"
#include "caraspace.hpp"

namespace fs = std::filesystem;
namespace gt = Gaps::Telemetry;
namespace g  = gondola;

int main(int argc, char *argv[]){
  spdlog::cfg::load_env_levels();
    
  cxxopts::Options options("calculate-tof", "Read GAPS L0 (caraspace) files. These files contain ALL information, including the TOF disk (waveform) stream and ALL telemetry packets. Here we go a step further and calculate the time-of-fligt for an example paddle combination");
  options.add_options()
  ("h,help", "Print help")
  //("c,calibration", "Folder with binary calibration files for each RB", cxxopts::value<std::string>()->default_value(""))
  ("file", "A Caraspace file", cxxopts::value<std::string>())
  ("v,verbose", "Verbose output", cxxopts::value<bool>()->default_value("false"))
  ;
  options.parse_positional({"file"});
  auto result = options.parse(argc, argv);
  if (result.count("help")) {
    std::cout << options.help() << std::endl;
    exit(EXIT_SUCCESS);
  }
  if (!result.count("file")) {
    spdlog::error("No input file given!");
    std::cout << options.help() << std::endl;
    exit(EXIT_FAILURE);
  }
  auto pathname   = result["file"].as<std::string>();
  bool verbose = result["verbose"].as<bool>();
  
  fs::path path(pathname);
  if (!fs::exists(path)) {
    spdlog::error("Path {} does not exist!", pathname);
    exit(EXIT_FAILURE);
  }
  
  Vec<std::string> filenames;
  if (fs::is_directory(path)) {
    for (const auto& entry : fs::directory_iterator(path)) {
      if (entry.is_regular_file()) {
        std::string filename = entry.path().string();
        filenames.push_back(filename);
      }
    }
  }
  std::cout << "Will read " << filenames.size() << " files!" << std::endl; 
  std::string tp_name        = "PacketType.TofEvent";
  std::string tel_ev_nogaps  = "TelemetryPacketType.NoGapsTriggerEvent";
  std::string tel_ev_boring  = "TelemetryPacketType.BoringEvent";
  std::string tel_ev_intrst  = "TelemetryPacketType.InterestingEvent";

  u64 n_frames_processed  = 0;
  u64 n_telemetry_errors  = 0;
  u64 n_tof_telemetry_err = 0;

  // as an example, we choose paddles 9 and 69
  u8 p0 = 69;
  u8 p1 = 9;
  Vec<f32> phases;

  for (auto const &f : filenames) {
    auto start = std::chrono::high_resolution_clock::now();
    auto reader = gondola::CRReader(f);
    u64 n_frames_processed_file = 0;
    while (!reader.is_exhausted()) {

      auto frame = gondola::CRFrame();
      try {
        frame = reader.get_next_frame();
      } catch (const std::exception& e) {
        std::string emeesage = std::format("--> Exception '{}' caught!", e.what());
        std::string message = std::format("--> File {} with {} frames processed! In total, we proceseed {} frames", f, n_frames_processed_file, n_frames_processed);
        std::cout << emeesage << std::endl;
        std::cout << message << std::endl;
        break;
      }
      ++n_frames_processed;
      ++n_frames_processed_file;

      gt::Packet pack;
      if (frame.index.contains(tel_ev_nogaps)) {
        pack = frame.get_telemetrypacket(tel_ev_nogaps);
      } else if (frame.index.contains(tel_ev_boring)) {
        pack = frame.get_telemetrypacket(tel_ev_boring);
      } else if (frame.index.contains(tel_ev_intrst)) {
        pack = frame.get_telemetrypacket(tel_ev_intrst);
      } else {
        continue;
      }

      TofPacket tp;
      if (frame.index.contains(tp_name)) {
        auto tdata = frame.get_tofpacket(tp_name);
        if (tdata.is_err()) {
          continue;
        }
        tp = tdata.unwrap();
        u64 pos = 0;
        auto evdata = g::TofEvent::from_bytestream(tp.payload, pos);
        if (evdata.is_err()) {
          continue;
        }
        auto ev = evdata.unwrap();
        for (g::TofHit const &h : ev.get_hits()) {
          // calculate tof for a certain paddle combination from 
          // telemetry data
          // for now, just to test, we will check the value of the phase
          phases.push_back(h.phase);      
        }
      }

      //if (verbose) {
      //  std::cout << "---- TELEMETRY -----" << std::endl;
      //  std::cout << frame.to_string() << std::endl;
      //  std::cout << pack.to_string() << std::endl;
      //}
      //usize pos = 0;
      //auto result = gt::MergedEvent::from_bytestream(pack.payload, pos);
      //// in case of errors, we just move on
      //if (result.is_err()) {
      //  std::string message = result.unwrap_err().reason;
      //  spdlog::error(message);
      //  ++n_telemetry_errors;
      //  continue;
      //}
      //auto m_ev = result.unwrap();
      //auto tofdata = TofEventSummary::from_bytestream(m_ev.tof_data, pos);
      ////if (tofdata.is_err()) {
      ////  ++n_tof_telemetry_err;
      ////  continue;
      ////}
      ////TofEventSummary tes = tofdata.unwrap();
      //for (TofHit const &h : tofdata.hits) {
      //  // calculate tof for a certain paddle combination from 
      //  // telemetry data
      //  // for now, just to test, we will check the value of the phase
      //  phases.push_back(h.phase);      
      //}
      //for (gt::TrkHit const &h : m_ev.trk_hits) {
      //  std::cout << h.to_string() << std::endl;
      //}

      if (n_frames_processed % 1000 == 0) {
        auto end = std::chrono::high_resolution_clock::now();
        std::chrono::duration<double> elapsed = end - start;
        std::cout << "--> ------------------------------" << std::endl;
        std::cout << "--> Processesd " << n_frames_processed << " frames in " << elapsed << std::endl;
        std::cout << "--> Saw " << n_telemetry_errors << " errores when reading telemetry files!" << std::endl;
        //std::cout << "--> Saw " << n_tofpacket_errors << " errores when reading tofstream files!" << std::endl;
        auto start = std::chrono::high_resolution_clock::now();
      }
    }
  } 
  //auto paddles = Gaps::get_tofpaddles();
  //for (auto const &p : paddles) {
  //  std::cout << "************* PADDLE " << (int)p.first << "***********" << std::endl;
  //  std::cout << p.second << std::endl;
  //  std::cout << "\n\n" << std::endl;
  //}
  spdlog::info("Finished");
  f32 max_phase = *std::max_element(phases.begin(), phases.end());
  f32 min_phase = *std::min_element(phases.begin(), phases.end());
  spdlog::info("We found a maximum phase value of {}", max_phase);
  spdlog::info("We found a minimum phase value of {}", min_phase);
  return EXIT_SUCCESS;
}
