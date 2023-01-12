# readoutboard software

## client which runs on the readoutboards

* Access the dedicated memory /dev/uio0 /dev/uio1 /dev/uio2
  to control DRS4 and readout blobs

* Send blob information per request over zmq socket

### Connectivity

### Design philosopy

### Threaded model
```mermaid
---
title: LIFTOF Readoutboard clients
---
%%{
  init: {
    'flowchart': { 'curve': 'monotoneY' },
    'theme': 'base',
    'themeVariables': {
      'primaryColor': '#2B3467',
      'primaryTextColor': '#FCFFE7',
      'primaryBorderColor': '#BAD7E9',
      'lineColor': '#EB455F',
      'secondaryColor': '#006100',
      'tertiaryColor': '#BAD7E9'
    }
  }
}%%
flowchart TB
  TofComputer -- TofCommand --> ControlThread
  subgraph ReadoutBoard
  ControlThread
  EventCacheThread
  BufferWorker
  HeartBeatThread
  MonitoringThread
  end
  ControlThread -- TofResponse --> TofComputer
  ControlThread -- controls --> EventCacheThread
  ControlThread -- controls --> BufferWorker
  ControlThread -- controls --> MonitoringThread
  ControlThread -- controls --> HeartBeatThread
  HeartBeatThread == Heartbeat ==> TofComputer
  EventCacheThread == Event ==> TofComputer
  MonitoringThread == Moni  ==> TofComputer
  BufferWorker == Bytestream ==> EventCacheThread
```
### Helpful resources

https://linux-kernel-labs.github.io/refs/heads/master/labs/memory_mapping.html
