#ifndef GO_ERRORS_H_INCLUDED
#define GO_ERRORS_H_INCLUDED


namespace Gaps {
  class IOError {
    public:
   
      enum class ErrorKind {
        StreamTooShort,
        WrongDelimiter,
        PacketNotFound,
        WrongPacketType,
        WrongHeaderBytes
      };
     
      IOError(ErrorKind kind, std::string reason = ""):
        kind(kind), reason(reason) {}
      
      ErrorKind kind;
      std::string reason;
  };
}  
#endif
