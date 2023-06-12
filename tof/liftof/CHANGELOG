# Release MOA - 0.4
_Moa is the Hawaiian name for the female spotted boxfish, Ostracion meleagris_

## Bugfixes:

* Fixes a critical bug, where data of channel 9 was run 
  through the waveform analysis routine and ended up 
  spamming the paddle packet cache

## New projects:

* `gaps_db` : SQLite/PostgreSQL db with paddle, RB and 
              LTB information, obtained from xlsx 
              spreadsheets.

* `liftof-analysis` : Use the rust analysis tools for 
		      offline checks (in development) 

## Features

* Tof Mapping: Established a way to translate 
  mapping xlsx spreadsheet into a sqlite database 
  format. This can be accessed to get a list of 
  RBs and LTBs which provide the basic information
  to query the RBs for event data.

* [liftof-rb] - Calibration flags: Flags `--vcal`, `--tcal` 
  and `--noi` set the respective settings and name 
  the resulting datafile accordingly

* `liftof.service` - liftof-rb integration in systemd on the 
  readoutboards.

* Remote start stop: The central command server is 
  issueing start/stop signals to all RBs.



## Improvements


# Release MANINI - 0.3
_Manini is the Hawaiian name for the Convict Tang, Acanthurus triostegus_

- Used for "initial systems testing" at SSL
- continuous mode - rbs send all data
- establishing API

