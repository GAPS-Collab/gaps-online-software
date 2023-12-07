## Tof Commands

_This is based on slides from Sydney, Dec 2022, Achim's input, and, hopefully, a good dose of common sense._

### Implementation 

_This is subject to change._

Each command has a `TofCommandCode` (u8) and a data field (u32).

Commands will be filled in a `TofCommand` with a fixed size of 9 bytes
(2 (head) + 1 (cmd) + 4 (data) + 2 (foot)) and the following structure:
```
  HEAD           : u16 = 0xAAAA
  TofCommandCode : u8
  data           : u32
  TAIL           : u16 = 0x5555
```

The CommandClass is a number (u8) so at max 255 different commands are available.
We divide the CommandClass in several subgroups. Currently, a large number of 
the command space is reserved.
* `<Reserved>`   : 0-9
* `<Reserved>`   : 100 - 199
* `<Reserved>`   : 200 - 255
* Power          : 10  - 19
* Config         : 20  - 29
* Run            : 30  - 39
* Calibration    : 50  - 59
* Tof SW related : 60 - 69


The commands follow Sydney's list, Achim's list, and additions (Pad stands for padding, first user commands and then nerd commands):

| TofCommandCode | Command | Frequency | Data |
| -------------- | ------- | --------- | ---- |
| 0 | Unknown command | Deal with unrecognised command or corrupted commands | Pad:u32 |
| 1 | Ping command | Ping tof components to check whether they are online                                                                      | Pad:u16, ComponentType:u8, ComponentID:u8 |
| 2 | Monitor command | Request status of a specific tof component                                                                             | Pad:u16, ComponentType:u8, ComponentID:u8 |
| 10 | Power on/off/cycle command | < 5/day Command to power on/off/cycle MTB, RBs (only reboot), LTBs, Preamps. RATs are managed by flight cpu| Pad:u8, TofComponent:u8, ComponentID:u8, Status:u8 |
| 21 | Set LTB Threshold command  | < 3/day Command to set a threshold level on LTBs (to TOF -> to  RBs)                                       | LTB_ID:u8, Threshold_name:u8, Level:u16 |
| 22 | Set MTB Config command | < 3/day? Command to set MTB trigger config (to TOF -> to MTB)                                                  | Pad:u16, TriggerConfig:u16 |
| 28 | Set preamp bias command  | < 3/day Command to set a preamp bias on preamps (to TOF -> to RBs)                                           | Pad:u8, Preamp_ID:u8, Preamp_level:u16 |
| 30 | Stop data-taking run command  | <10/day? Command to stop taking regular data (to TOF -> to RBs)                                         | Pad:u32 |
| 31 | Start data-taking run command  | <10/day? Command to take regular data (to TOF -> to RBs)                                               | Pad:u8, RunType:u8, E:u8, Time:u8 |
| 32 | Start validation run command | <10/day? Command to take a small amount of data on an RB or every RB (E events)                          | Pad:u16, E:u8, RB:u8 |
| 33 | Get full waveforms (360*E, after 32) command | Command to take raw data from RBs                                                        | Pad:u32 |
| 50 | No input Calibration command | Command to take data without input @ 100 Hz in self trigger (E = 1000 default) for an RB or every RB     | Pad:u16, RB:u8, Extra:u8 |
| 51 | Voltage Calibration command | Command to take data with fixed voltage @ 100 Hz in self trigger (E = 1000 default) for an RB or every RB (it includes 50) | VoltageLevel:u16, RB:u8, Extra:u8 |
| 52 | Timing Calibration command | Command to take data with poisson self trigger (E = 5000 default) for an RB or every RB (it includes 50/51)| VoltageLevel:u16, RB:u8, Extra:u8 |
| 53 | Default Calibration command | Command to perform the default calibration step and calibrate RBs (it includes 50/51/52)                  | VoltageLevel:u16, RB:u8, Extra:u8 |

Note: when "all" is needed it will be rendered as all ones.

Nerd commands, related to TOF SW:

| TofCommandCode | Command | Frequency | Data |
| -------------- | ------- | --------- | ---- |
| 23 | Set RB data buffer size command  | Command to set RB data buffer size (to TOF -> to  RBs)                                               | Pad:u8, RB:u8, Size:u16 |
| 24 | Enable/disable trigger mode forced command  | Command to enable/disable trigger mode forced (to TOF -> to  RBs)                         | Pad:u16, RB:u8, Status:u8 |
| 25 | Enable/disable trigger mode forced for MTB command  | Command to enable/disable trigger mode forced for MTB (to TOF -> to  RBs)         | Pad:u16, Pad:u8, Status:u8 |
| 44 | Unspool event cache command | Command to unspool event cache if something bad happens                                                   | Pad:u32 |
| 60 | Restart systemd command | Command to restart systemd if some mischievous stuff happens                                                  | Pad:u32 |