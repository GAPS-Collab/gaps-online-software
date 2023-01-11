## Tof Commands

_this is based on slides from Sydney, Dec 2022_

### Implementation 

_obviously, this is subject to change_

Each command has a CommandClass ID and can have a value of size u32

Commands will be filled in a `CommandPacket` with a fixed size of 9 bytes
and the following structure:
```
  HEAD         : u16 = 0xAAAA
  CommnadClass : u8
  DATA         : u32
  TAIL         : u16 = 0x5555
```
This does not follow exactly Sydney's proposal, but has some advantages:
* The CommandClass is 1 byte shorter
* Each CommandPacket has the same (fixed) size
* A 32 bit DATA field can accomodate the event id, which is needed for the
  event query.

The CommandClass is a number (u8) so at max 255 different commands are available.
We divide the CommandClass in several subgroups. Currently, a large number of 
the command space is reserved.
* `<Reserved>`  : 0-9
* `<Reserved>`  : 100 - 199
* `<Reserved>`  : 200 - 255
* Power       : 10  - 19
* Config      : 20  - 29
* Run         : 30  - 39
* Request     : 40  - 49
* Calibration : 50  - 59


The commands follow Sydney's list:

| CommandCode | Command | Frequency | 
| ----------- | ------- | --------- | 
| 10,11,12 | Power on/off to PBs+RBs+LTBs+preamps (all at once) or MT | < 5/day Command to power on/off PDU channels (to PDU) |
| 10,11,12 | Power on/off to LTB or preamp | < 2/day Command to power on/off various components (to TOF -> to RB)             |
| 20 | RBsetup ? Command to run rbsetup on a particular RB (to TOF -> to RBs) |                                         |
| 21 | Set Thresholds  | < 3/day Command to set a threshold level on all LTBs (to TOF -> to  RBs)                       |
| 22 | Set MT Config 1/run | <10/day? Command to set MT trigger config (to TOF -> to MT)                                |
| 32 | Start Validation Run 1/run | <10/day? Command to take a small amount of data (some number E events,              |
| 41 | 360xE full waveforms (from TOF) | ? |                                                                            |
| 31 | Start Data-Taking Run 1/run  | <10/day? Command to take regular data (to TOF -> to RBs)                          | 
| 42 | Reduced data packet (from Flight computer) | ?                                                                   |
| 30 | Stop Run < 1/run | < 10/day Command to stop a run (to TOF -> to RBs)                                             | 
| 51 | Voltage Calibration | Runs 1/day Command to take 2 voltage calibration runs (to TOF -> to RBs) 12 B              | 
| 52 | Timing Calibration  | Run 1/day Command to take a timing calibration run (to TOF -> to RBs) 8 B                  |
| 53 | Create New Calibration File | ?                                                                                  |    

Further commands (Achim)
| CommandCode | Command | _Comment_ |
| ----------- | ------- | --------- |
| 12          | PowerCycle   | Combines 10 + 11 | 
| 43          | RequestMoni  | Request monitoring/housekeeping data | 

## Render the flowcharts for documentation

This uses mermaid to render the flowcharts, generating images can be done via
mermaid-cli.
Install with
`npm install -g @mermaid-js/mermaid-cli`

and then use it e.g. like this:
`mmdc -i input.mmd -o output.png -t dark -b transparent`
