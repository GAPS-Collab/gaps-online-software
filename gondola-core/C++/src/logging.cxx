#include <iostream>

#include "logging.hpp"

/*******************************************************************/

std::string Gaps::severity_to_str (const LOGLEVEL& severity) {
  switch (severity) { 
    case LOGLEVEL::trace      : return "trace"    ;
    case LOGLEVEL::debug      : return "debug"    ;
    case LOGLEVEL::info       : return "Info"     ;
    case LOGLEVEL::warn       : return "Warn"     ;
    case LOGLEVEL::err        : return "ERROR"    ;
    case LOGLEVEL::critical   : return "FATAL"    ;
    case LOGLEVEL::off        : return "OFF"      ;
    default                   : return "-unknown-";
  }
  return "";
} 

void Gaps::set_loglevel(LOGLEVEL level) {
  spdlog::set_pattern("[%^%l%$] [%s - %!:%#] [%Y-%m-%d %H:%M:%S] -- %v");
  spdlog::set_level(level);
}

