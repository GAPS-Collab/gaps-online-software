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
        WrongHeaderBytes,
        WrongTailBytes,
        EventHeaderCorrupt
      };
     
      IOError(ErrorKind kind, std::string reason = ""):
        kind(kind), reason(reason) {}
      
      ErrorKind kind;
      std::string reason;
  };

  //class FatalException : public std::exception {
  //  virtual const char* what() const throw() { 
  //    return "Abort program due to a FatalException thrown in gaps-online-software!!";
  //  }
  //};
}  
#endif
