#ifndef CRANELOGGING_H_INCLUDED
#define CRANELOGGING_H_INCLUDED
/*
 * Crane::Logging - a logging system for CRANE
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

// logging macros

#include <iostream>

#ifdef USE_BOOST_LOG
#include <boost/log/trivial.hpp>
#include <boost/log/sources/record_ostream.hpp>
#include <boost/log/sources/global_logger_storage.hpp>
#include <boost/log/utility/setup.hpp>
#include <boost/log/sources/severity_feature.hpp>
#include <boost/log/sources/logger.hpp>
#include <boost/log/support/date_time.hpp>
#include <boost/log/expressions/formatters/date_time.hpp>
#include <boost/log/expressions.hpp>
#include <boost/log/attributes/named_scope.hpp>
#include <boost/log/attributes/mutable_constant.hpp>


namespace expr = boost::log::expressions;
namespace logging = boost::log;
namespace src = boost::log::sources;
namespace attrs = boost::log::attributes;

#define LOG_LOCATION do { \
  boost::log::attribute_cast<boost::log::attributes::mutable_constant<int>>(boost::log::core::get()->get_global_attributes()["Line"]).set(__LINE__); \
  boost::log::attribute_cast<boost::log::attributes::mutable_constant<std::string>>(boost::log::core::get()->get_global_attributes()["File"]).set(__FILE__); \
  boost::log::attribute_cast<boost::log::attributes::mutable_constant<std::string>>(boost::log::core::get()->get_global_attributes()["Function"]).set(__func__); \
} while(0);
#else
#include <chrono>
#include <ctime>
#endif


// log_trace
#ifndef NDEBUG

#ifdef USE_BOOST_LOG
#define log_trace(...) do { \
LOG_LOCATION; \
BOOST_LOG_SEV(_craneSevLogCout::get(), Crane::Common::LOGLEVEL::trace) << __VA_ARGS__; \
} while(0)
#else
#define log_trace(...) do { \
if (Crane::Common::GetGlobalLogLevel() == Crane::Common::LOGLEVEL::trace) {\
auto timenow = \
std::chrono::system_clock::to_time_t(std::chrono::system_clock::now()); \
std::cout << "[trace] - " << __VA_ARGS__ << "<" << __func__ << ">(" << __FILE__ <<":" << __LINE__ << ")"  << " - " << ctime(&timenow); \
} \
} while(0)
#endif

#else //NDEBUG case, ompile it out
#define log_trace(...) ((void) 0);
#endif

// log_debug
#ifndef NDEBUG

#ifdef USE_BOOST_LOG
#define log_debug(...) do { \
  LOG_LOCATION; \
  BOOST_LOG_SEV(_craneSevLogCout::get(), Crane::Common::LOGLEVEL::debug) << __VA_ARGS__; \
} while(0)
#else
#define log_debug(...) do { \
if (Crane::Common::GetGlobalLogLevel() <= Crane::Common::LOGLEVEL::debug) {\
auto timenow = \
std::chrono::system_clock::to_time_t(std::chrono::system_clock::now()); \
std::cout << "[debug] - " << __VA_ARGS__ << " <" << __func__ << ">(" << __FILE__ <<":" << __LINE__ << ")"  << " - " << ctime(&timenow); \
} \
} while(0)
#endif

#else //NDEBUG case, ompile it out
#define log_debug(...) ((void) 0);
#endif

// log_info
#ifdef USE_BOOST_LOG
#define log_info(...) do { \
  LOG_LOCATION; \
  BOOST_LOG_SEV(_craneSevLogCout::get(), Crane::Common::LOGLEVEL::info) << __VA_ARGS__; \
} while(0)
#else
#define log_info(...) do { \
if (Crane::Common::GetGlobalLogLevel() <= Crane::Common::LOGLEVEL::info) {\
auto timenow = \
std::chrono::system_clock::to_time_t(std::chrono::system_clock::now()); \
std::cout << "[Info] - " << __VA_ARGS__ << "<" << __func__ << ">(" << __FILE__ <<":" << __LINE__ << ")"  << " - " << ctime(&timenow); \
} \
} while(0)
#endif

// log_warn
#ifdef USE_BOOST_LOG
#define log_warn(...) do { \
  LOG_LOCATION; \
  BOOST_LOG_SEV(_craneSevLogCout::get(), Crane::Common::LOGLEVEL::warning) << __VA_ARGS__; \
} while(0)
#else
#define log_warn(...) do { \
if (Crane::Common::GetGlobalLogLevel() <= Crane::Common::LOGLEVEL::warning) {\
auto timenow = \
std::chrono::system_clock::to_time_t(std::chrono::system_clock::now()); \
std::cout << "[Warning] - " << __VA_ARGS__ << "<" << __func__ << ">(" << __FILE__ <<":" << __LINE__ << ")"  << " - " << ctime(&timenow); \
} \
} while(0)
#endif


// log_error
#ifdef USE_BOOST_LOG
#define log_error(...) do { \
  LOG_LOCATION; \
  BOOST_LOG_SEV(_craneSevLogCerr::get(), Crane::Common::LOGLEVEL::error) << __VA_ARGS__; \
} while(0)
#else
#define log_error(...) do { \
if (Crane::Common::GetGlobalLogLevel() <= Crane::Common::LOGLEVEL::error) {\
auto timenow = \
std::chrono::system_clock::to_time_t(std::chrono::system_clock::now()); \
std::cout << "[ERROR] - " << __VA_ARGS__ << "<" << __func__ << ">(" << __FILE__ <<":" << __LINE__ << ")"  << " - " << ctime(&timenow); \
} \
} while(0)
#endif

// log_fatal
#ifdef USE_BOOST_LOG
#define log_fatal(...) do { \
  LOG_LOCATION; \
  BOOST_LOG_SEV(_craneSevLogCerr::get(), Crane::Common::LOGLEVEL::fatal) << __VA_ARGS__; \
  throw Crane::Common::FatalException(); \
} while(0)
#else
#define log_fatal(...) do { \
auto timenow = \
std::chrono::system_clock::to_time_t(std::chrono::system_clock::now()); \
std::cout << "[FATAL] - " << __VA_ARGS__ << "<" << __func__ << ">(" << __FILE__ <<":" << __LINE__ << ")"  << " - " << ctime(&timenow); \
throw Crane::Common::FatalException(); \
} while(0)
#endif


namespace Crane{
namespace Common{

  enum class LOGLEVEL {
    trace   =  10,
    debug   =  20,
    info    =  30,
    warning =  40,
    error   =  50,
    fatal   =  60
    };

  
  std::string severity_to_str (const LOGLEVEL& severity);

  #ifdef USE_BOOST_LOG
  //BOOST_LOG_ATTRIBUTE_KEYWORD(severity, "Severity", LOGLEVEL);

  /**
   * Helper operator to print severity as string
   *
   */
  template< typename CharT, typename TraitsT >
  std::basic_ostream< CharT, TraitsT >&
  operator<< (
    std::basic_ostream< CharT, TraitsT >& strm,
    LOGLEVEL severity
  )
  {
    strm << severity_to_str(severity);
    return strm;
  }
  #endif

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
  void set_loglevel(Crane::Common::LOGLEVEL severity);

#ifdef USE_BOOST_LOG

/**
 * Helper to initialize logging
 *  - only call this once or you add more sinks
 */
class _LoggerHelper {

      public:
          static _LoggerHelper* Initialize(){
          if (instance == nullptr)
              {
               instance = new _LoggerHelper();
              }
              return instance;
          } 

      private:
         _LoggerHelper(_LoggerHelper const&)   = delete;
         void operator=(_LoggerHelper const&)  = delete;
         static _LoggerHelper* instance;
          
         ~_LoggerHelper();
         _LoggerHelper()
              {
                 //BOOST_LOG_ATTRIBUTE_KEYWORD(Crane::Common::severity, "Severity", LOGLEVEL);
                 boost::shared_ptr< logging::core > core = logging::core::get();
                 core->remove_all_sinks();
                 // add the keywords we need to log the location 
                 core->add_global_attribute("Line", attrs::mutable_constant<int>(-1));
                 core->add_global_attribute("File", attrs::mutable_constant<std::string>(""));
                 core->add_global_attribute("Function", attrs::mutable_constant<std::string>("")); 
                 core->add_global_attribute("Severity", attrs::mutable_constant<LOGLEVEL>(LOGLEVEL::trace));
                 boost::log::add_common_attributes();

                 auto fmtTimeStamp =  expr::format_date_time< boost::posix_time::ptime >("TimeStamp", "%Y-%m-%d %H:%M:%S");
                 auto fmtSeverity = expr::attr<LOGLEVEL>("Severity");
                 auto fmtLine = expr::attr<int>("Line");
                 auto fmtFunction = expr::attr<std::string>("Function");
                 auto fmtFile = expr::attr<std::string>("File");

                 // Output message to console
                 auto console_sink_cout = boost::log::add_console_log(
                     std::cout,
                     logging::keywords::format = (
                       expr::stream <<
                        "[" << fmtSeverity << "]: " << expr::smessage
                       << " :<" << fmtFunction << ">("
                       << fmtFile <<":"<<fmtLine <<") - " 
                       <<fmtTimeStamp),
                     logging::keywords::auto_flush = true
                 );
                 console_sink_cout->set_filter(expr::attr<LOGLEVEL>("Severity") <= LOGLEVEL::warning);

                 // cerr stream
                 auto console_sink_cerr = boost::log::add_console_log(
                     std::cerr,

                     logging::keywords::format = (
                       expr::stream <<
                        "[" << fmtSeverity << "]: " << expr::smessage
                       << " :<" << fmtFunction << ">("
                       << fmtFile <<":"<<fmtLine <<") - " 
                       <<fmtTimeStamp),
                     //logging::keywords::format = COMMON_FMT,
                     logging::keywords::auto_flush = true
                 );
                 console_sink_cerr->set_filter(expr::attr<LOGLEVEL>("Severity") > LOGLEVEL::warning);
              };
  };
#endif

#ifndef USE_BOOST_LOG
LOGLEVEL GetGlobalLogLevel();
  
std::string severity_to_str (const LOGLEVEL& severity);

class CGlobalLogger {

 public:    
  static CGlobalLogger* GetInstance();

  LOGLEVEL GetGlobalLogLevel();
  void SetGlobalLogLevel(LOGLEVEL severity);

  CGlobalLogger(CGlobalLogger const&)   = delete;
  void operator=(CGlobalLogger const&)  = delete;

 private:
  static CGlobalLogger* instance;
  CGlobalLogger();
  ~CGlobalLogger();
  LOGLEVEL globalLogLevel_;    
};
#endif

} // end namespace Logging
} // end namespace Crane

// operator << for cout stream
std::ostream& operator<<(std::ostream& os, Crane::Common::LOGLEVEL& severity);

#ifdef USE_BOOST_LOG
// operator << for boost::formatting
logging::formatting_ostream& operator<<(logging::formatting_ostream& os,    logging::to_log_manip< Crane::Common::LOGLEVEL> const& manip);

// initialize 2 loggers - one for cout and one for cerr
BOOST_LOG_INLINE_GLOBAL_LOGGER_DEFAULT(_craneSevLogCout, src::severity_logger<Crane::Common::LOGLEVEL>)
BOOST_LOG_INLINE_GLOBAL_LOGGER_DEFAULT(_craneSevLogCerr, src::severity_logger<Crane::Common::LOGLEVEL>)

// a call to init_log by iniializng a global instance of _LoggerHelper;
static Crane::Common::_LoggerHelper* _lgHelper =  Crane::Common::_LoggerHelper::Initialize();
#endif


#endif //include guard
