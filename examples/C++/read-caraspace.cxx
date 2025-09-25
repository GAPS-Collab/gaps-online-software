/**
 * Binary to illustrate how to read GAPS L0 file with the
 * caraspace library.
 * To use this example, the code has to be build with
 * BUILD_CARASPACE=ON
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
#include "cxxopts.hpp"

#include "spdlog/spdlog.h"
#include "spdlog/cfg/env.h"

#include "io.hpp"
#include "calibration.h"
#include "database.h"
#include "caraspace.hpp"

namespace fs = std::filesystem;
namespace gt = Gaps::Telemetry;

int main(int argc, char *argv[]){
  spdlog::cfg::load_env_levels();
    
  cxxopts::Options options("read-caraspace", "Read GAPS L0 (caraspace) files. These files contain ALL information, including the TOF disk (waveform) stream and ALL telemetry packets");
  options.add_options()
  ("h,help", "Print help")
  //("c,calibration", "Folder with binary calibration files for each RB", cxxopts::value<std::string>()->default_value(""))
  ("file", "A Caraspace file", cxxopts::value<std::string>())
  ("directory", "A directory containing .gaps (caraspace) files, e.g. L0 Gaps files", cxxopts::value<std::string>())
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

  std::string cooling_name   = "TelemetryPacketType.CoolingHK";

  u64 n_frames_processed  = 0;
  u64 n_telemetry_errors  = 0;
  u64 n_tof_telemetry_err = 0;
        
  auto start = std::chrono::high_resolution_clock::now();

  auto trk_mask = Gaps::get_trackerstripmasks();
  auto trk_ped  = Gaps::get_trackerstrippedestals();

  // as an example, count tracker hits
  u64 n_trk_hits        = 0;
  u64 n_trk_hits_masked = 0;
  u64 n_evt_no_trk_hits = 0;
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

      if (frame.index.contains(cooling_name)) {
        pack = frame.get_telemetrypacket(cooling_name);
        std::cout << pack.to_string() << std::endl;
        usize pos = 0;
        auto cooling = gt::Cooling::from_bytestream(pack.payload, pos);
        if (cooling.is_ok()) {
          std::cout << cooling.unwrap().to_string() << std::endl;
        } else {
          std::cout << cooling.unwrap_err().reason << std::endl;
        }
        //std::exit(1);
      }

      if (verbose) {
        std::cout << "---- TELEMETRY -----" << std::endl;
        std::cout << frame.to_string() << std::endl;
        std::cout << pack.to_string() << std::endl;
      }
      usize pos = 0;
      auto result = gt::MergedEvent::from_bytestream(pack.payload, pos);
      // in case of errors, we just move on
      if (result.is_err()) {
        std::string message = result.unwrap_err().reason;
        spdlog::error(message);
        ++n_telemetry_errors;
        continue;
      }
      auto m_ev = result.unwrap();
      for (gt::TrkHit const &h : m_ev.trk_hits) {
        auto strip_id = Gaps::TrackerStrip::create_id(h.layer, h.row, h.module, h.channel);
        if (trk_mask[strip_id]) {
          // only count active strips
          ++n_trk_hits;
          // just as an example - subtract a pedestal
          auto adc_no_pedestal = h.adc - trk_ped[strip_id].pedestal_mean;
          //adc_no_pedestal;
        } else {
          ++n_trk_hits_masked;
        }
          //std::cout << h.to_string() << std::endl;
      }
      if (m_ev.trk_hits.size() == 0) {
        ++n_evt_no_trk_hits;
      }
      for (TofHit const &h : m_ev.tof_event.hits) {
        // do someting with h
        //std::cout << h.to_string() << std::endl;
      }

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
  auto end = std::chrono::high_resolution_clock::now();
  auto elapsed = end - start;
  std::cout << "--> ----FINISHED--------------" << std::endl;
  std::cout << "--> Processesd " << n_frames_processed << " frames in " << elapsed << std::endl;
  std::cout << "--> Saw " << n_telemetry_errors << " errores when reading telemetry files!" << std::endl;
  std::cout << "--> Saw " << n_tof_telemetry_err << " errores when reading tofdata from telemetry files!" << std::endl;
  std::cout << "--> Saw " << n_trk_hits << " valid tracker hits!" << std::endl;
  std::cout << "--> Saw " << n_trk_hits_masked << " inactive tracker hits!" << std::endl;
  std::cout << "--> Saw " << n_evt_no_trk_hits << " events without any tracker hits!" << std::endl;
  //std::cout << "--> Saw " << n_tofpacket_errors << " errores when reading tofstream files!" << std::endl;
  spdlog::info("Finished");
  return EXIT_SUCCESS;
}
