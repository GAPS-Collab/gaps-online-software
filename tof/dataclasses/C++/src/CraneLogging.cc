#include <iostream>


#include "CraneLogging.hh"

namespace cc = Crane::Common;

#ifndef USE_BOOST_LOG
cc::CGlobalLogger* cc::CGlobalLogger::instance = nullptr;
#endif

/*******************************************************************/


/*******************************************************************/

std::string cc::severity_to_str (const Crane::Common::LOGLEVEL& severity)
{
    switch (severity) { 
        case cc::LOGLEVEL::trace   : return "trace"  ;
        case cc::LOGLEVEL::debug   : return "debug"  ;
        case cc::LOGLEVEL::info    : return "Info"   ;
        case cc::LOGLEVEL::warning : return "Warning";
        case cc::LOGLEVEL::error   : return "ERROR"  ;
        case cc::LOGLEVEL::fatal   : return "FATAL"  ;
    }
    return "";
} 

/*******************************************************************/

std::ostream& operator<<(std::ostream& os, cc::LOGLEVEL& severity)
{
    os << severity_to_str(severity);
    return os;
} 

/*******************************************************************/

#ifdef USE_BOOST_LOG
logging::formatting_ostream& operator<< (logging::formatting_ostream& os,
logging::to_log_manip< cc::LOGLEVEL> const& manip)
{
    cc::LOGLEVEL severity = manip.get();
    switch (severity) { 
        case cc::LOGLEVEL::trace   : os << "trace"; break;
        case cc::LOGLEVEL::debug   : os << "debug"; break;
        case cc::LOGLEVEL::info    : os << "Info"; break;
        case cc::LOGLEVEL::warning : os << "Warning"; break;
        case cc::LOGLEVEL::error   : os << "ERROR"; break;
        case cc::LOGLEVEL::fatal   : os << "FATAL"; break;
    }
    return os;
} 
#endif

/*******************************************************************/

#ifndef USE_BOOST_LOG
cc::LOGLEVEL cc::GetGlobalLogLevel()
    {return cc::CGlobalLogger::GetInstance()->GetGlobalLogLevel();}
#endif

/*******************************************************************/

void cc::set_loglevel(cc::LOGLEVEL level)
{
  #ifdef USE_BOOST_LOG
  logging::core::get()->set_filter
  (
      expr::attr<LOGLEVEL>("Severity") >= level
  );
  #else
  cc::CGlobalLogger::GetInstance()->SetGlobalLogLevel(level);
  #endif
}

/*******************************************************************/

#ifdef USE_BOOST_LOG
cc::_LoggerHelper* cc::_LoggerHelper::instance = nullptr;
#endif

/*******************************************************************/

#ifndef USE_BOOST_LOG
cc::CGlobalLogger* cc::CGlobalLogger::GetInstance()
  {
    if (instance == nullptr)
      {
        instance = new CGlobalLogger();
      }
    return instance; 
 }

/*******************************************************************/

cc::LOGLEVEL cc::CGlobalLogger::GetGlobalLogLevel()
  { return globalLogLevel_;}

/*******************************************************************/

void cc::CGlobalLogger::SetGlobalLogLevel(cc::LOGLEVEL severity)
  { globalLogLevel_ = severity;}

/*******************************************************************/

cc::CGlobalLogger::CGlobalLogger(){}

/*******************************************************************/

cc::CGlobalLogger::~CGlobalLogger(){}
#endif

