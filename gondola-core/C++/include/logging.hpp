#ifndef LOGGING_H_INCLUDED
#define LOGGING_H_INCLUDED

/*
 * logging system
 *
 * it defines 6 macros which can be used for logging.
 *
 * In increasing severity:
 *  
 *  log_trace   - most verbose log level for nitty-gritty details (e.g. inside a loop)  
 *  log_debug   - detailed information which should help debugging a program
 *  log_info    - short, compressed information to tell the user about a programs status
 *  log_warn    - alert the user about pitfalls
 *  log_error   - an error occured. Most likely critical, however do NOT abort
 *  log_fatal   - abort the program with a message giving some information about
 *            what went wrong
 *
 *  log_trace, log_debug, log_info, log_warn will log to std::cout, log_error and log_fatal will log to std::cerr
 *  log_fatal also will call std::exit(EXIT_FAILURE)
 *
 *  The definition of the macros is connected to the definition of the NDEBUG
 *  macro - if it is defined, all log_debug and log_trace statements will be compiled out.
 *  The NDEBUG macros is set in case a "Release" build is done with the CMake
 *  build system. For that case, have a look at the CMAKE_BUILD_TYPE variable
 *  which can be set to "Release" or "Debug".
 *
 *  Use the macros as in the following example:
 *    .. doSomething();
 *    .. log_warn("There can be " << howMany <<  " dragons!");
 *    .. doSomethingElse();
 *    
 *  FIXME: logging to a file is not yet implemented
 */        


#define SPDLOG_ACTIVE_LEVEL SPDLOG_LEVEL_TRACE // Don't forget define SPDLOG_ACTIVE_LEVEL macro.

#include <iostream>
#include <sstream>
#include <spdlog/spdlog.h>
#include <spdlog/sinks/stdout_color_sinks.h>


/// Helper to enable << syntax for logging macros
template<typename... Args>
std::string concatenate_args(Args&&... args) {
  std::ostringstream oss;
  (oss << ... << args());
  return oss.str();
}


// log_trace
#ifndef NDEBUG

#define log_trace(...) SPDLOG_TRACE(concatenate_args([&](){ std::ostringstream oss; oss << __VA_ARGS__; return oss.str(); }))

#else //NDEBUG case, ompile it out
#define log_trace(...) ((void) 0);
#endif

// log_debug
#ifndef NDEBUG
#define log_debug(...) SPDLOG_DEBUG(concatenate_args([&](){ std::ostringstream oss; oss << __VA_ARGS__; return oss.str(); }))

#else //NDEBUG case, ompile it out
#define log_debug(...) ((void) 0);
#endif

#define log_info(...) SPDLOG_INFO(concatenate_args([&](){ std::ostringstream oss; oss << __VA_ARGS__; return oss.str(); }))

#define log_warn(...) SPDLOG_WARN(concatenate_args([&](){ std::ostringstream oss; oss << __VA_ARGS__; return oss.str(); }))


// log_error
#define log_error(...) SPDLOG_ERROR(concatenate_args([&](){ std::ostringstream oss; oss << __VA_ARGS__; return oss.str(); }))

// log_fatal
#define log_fatal(...) do { SPDLOG_ERROR(concatenate_args([&](){ std::ostringstream oss; oss << __VA_ARGS__; return oss.str(); })); \
throw Gaps::FatalException(); \
} while(0)

namespace Gaps {
  typedef spdlog::level::level_enum LOGLEVEL;    

  std::string severity_to_str (const LOGLEVEL& severity);

  /**
   * \brief Exception for log_fatal(..) macro
   *
   * log_fatal(..) will raise this exception to cause the 
   * code to abort
   *
   */
  class FatalException : public std::exception
  {
    virtual const char* what() const throw()
    { 
      return "Abort program due to a log_fatal(..) statement in the gaps code!";
    }
  };

  /**
   * Suppress log messages if their severity is lower than 
   * @param severity
   *
   */
  void set_loglevel(LOGLEVEL severity);
}



#endif //include guard
