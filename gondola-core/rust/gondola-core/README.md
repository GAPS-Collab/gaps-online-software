# GONDOLA - python wrapper for gaps-online-software 

Python wrapper for gaps-online-software, a software library 
written in Rust, which was mainly used for data acquisition 
and control of the TOF system in the [GAPS experiment](https://gaps1.astro.ucla.edu/gaps/) 
during its first flight.
The library allows to read raw science data as well as monitoring data for 
several subsystems of the GAPS experiment, perform calibration as well as 
provide some basic functionality to create basic plots.

# CHANGELOG 

This project is currently still under rapid development, while we try to keep the API 
somewhat stable, please strap in for a bit of a rough ride when upgrading. 
However, the amount of features is increasing constantly and rapidly 

## v0.12.25

* apply timing constants for tof directly to telemetry events
  (`TelemetryEvent::tof_set_timing_constants`)
* FIX - missing MasterTriggerHB timestamps

## v0.12.23 

* TrackerHits are now sortable and hashable (python) by using strip id and adc. All 
  other quantities are ignored 

## v0.12.21 

* FIX `get_version()` revived 
* Read WastieHK monitoring data

## v0.12.20

* adds fixed version for TOF occupancy plots, kudos Grace 
  'grace_<tof_projection_xy,unroll_cbe_sides,unroll_cor>` 
* adds TrackerOfflineCalibration (not yet ready for production use) 
* adds to interface of `TelemetryEvent` - change tof event in-place 
  (`TelemetryEvent.tof` only returns a copy) with
  `TelemetryEvent.tof<_remove_non_causal_hits, _normalize_hit_times>`
  and friends 

## v0.12.19

* `version_at_least` - check version complience 

## v0.12.18

* adds new keywoards to telemetry packet reader to skip packets read in the beginning or 
  at the end 

## v0.12.16 

* OHP temperatures 



