## Tof Commands

_this is based on slides from Sydney, Dec 2022_

### Implementation 

_obviously, this is subject to change_

Each command has a `TofCommandCode` (u8) and a data field (u32).

Commands will be filled in a `TofCommand` with a fixed size of 9 bytes
(2 (head) + 1 (cmd) + 4 (data) + 2 (foot)) and the following structure:
```
  HEAD           : u16 = 0xAAAA
  TofCommandCode : u8
  data           : u32
  TAIL           : u16 = 0x5555
```
This does not follow exactly Sydney's proposal, but has some advantages:
* The CommandClass is 1 byte shorter
* Each CommandPacket has the same (fixed) size
* A 32 bit data field can accomodate the event id, which is needed for the
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


The commands follow Sydney's list (Pad stands for padding):

| TofCommandCode | Command | Frequency | Data |
| -------------- | ------- | --------- | ---- |
| 10,11,12 | Power on/off to PBs + RBs + LTBs + preamps (all at once) or MT | < 5/day Command to power on/off PDU channels (to PDU) | Pad:u16+u8, PDU:u8 |
| 10,11,12 | Power on/off to LTB or preamp | < 2/day Command to power on/off various components (to TOF -> to RB)             | Pad:u16+u8, RB:u8 |
| 20 | RBsetup ? Command to run rbsetup on a particular RB or all (to TOF -> to RBs) <br />Comment: prolly not needed anymore? |                                        | Pad:u16+u8, RB:u8 |
| 21 | Set Thresholds  | < 3/day Command to set a threshold level on all LTBs (to TOF -> to  RBs)                             | Pad:u8, Threshold:u8, Level:u16 |
| 22 | Set MT Config 1/run | <10/day? Command to set MT trigger config (to TOF -> to MT)                                      | Pad:u16, TriggerConfig:u16 |
| 32 | Start Validation Run 1/run | <10/day? Command to take a small amount of data on an RB or all (E events)                | Pad:u16, E:u8, RB:u8 |
| 41 | Get full waveforms (360*E, after 32) (from TOF) | ?                                                                    | Pad:u32 |
| 31 | Start Data-Taking Run 1/run  | <10/day? Command to take regular data (to TOF -> to RBs)                                | Pad:u8, RunType:u8, E:u8, Time:u8 |
| 42 | Get reduced data packet (from Flight computer) | ?                                                                     | Pad:u32 |
| 30 | Stop Run < 1/run | < 10/day Command to stop a run (to TOF -> to RBs)                                                   | Pad:u32 |
| 50 | No input Calibration for an RB or all | ???                    | Pad:u16, RB:u8, Extra:u8 |
| 51 | Voltage Calibration for an RB or all | Runs 1/day Command to take 2 voltage calibration runs (to TOF -> to RBs) 12 B                    | VoltageLevel:u16, RB:u8, Extra:u8 |
| 52 | Timing Calibration for an RB or all | Run 1/day Command to take a timing calibration run (to TOF -> to RBs) 8 B                        | VoltageLevel:u16, RB:u8, Extra:u8 |
| 53 | Default Calibration for an RB or all | ???                    | VoltageLevel:u16, RB:u8, Extra:u8 |
| 54 | Create New Calibration File for an RB or all | Comment: does this really make sense? Whats the point of having a command that creates a file? Shouldnt it be integrated w/ smth else? | Pad:u8, VoltageLevel: u16, RB:u8 |

Note: when "all" is needed it will be rendered as all ones.

Further commands (Achim)
| CommandCode | Command | _Comment_ | Data |
| ----------- | ------- | --------- | ---- |
| 12          | PowerCycle   | Combines 10 + 11 | Refer to single cmds |
| 43          | RequestMoni  | Request monitoring/housekeeping data | Pad:u32 |

## Render the flowcharts for documentation

This uses mermaid to render the flowcharts, generating images can be done via
mermaid-cli.
Install with
`npm install -g @mermaid-js/mermaid-cli`

and then use it e.g. like this:
`mmdc -i input.mmd -o output.png -t dark -b transparent`

