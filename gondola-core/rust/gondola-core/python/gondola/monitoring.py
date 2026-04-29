"""
Monitoring and Housekeeping data structures 

For each set of monitoring parameters, there 
exists
1) an individual set of parameters, taken at 
   a specific point in time ("MoniData") 
2) a collection of these points ("MoniDataSeries") 

The single set of parameters ("MoniData") can be 
obtained from Tof/TelemetryPackets. 

The series allow to load such packets from entire files
and specifically allows to translate the read data to 
polars dataframes.
"""

from . import _gondola_core 

CPUMoniData                           = _gondola_core.monitoring.CPUMoniData        
CPUMoniData.__module__                = __name__

DataSinkHB                            = _gondola_core.monitoring.DataSinkHB 
DataSinkHB.__module__                 = __name__

EventBuilderHB                        = _gondola_core.monitoring.EventBuilderHB 
EventBuilderHB.__module__             = __name__

GcuEvBldStatsMoniData                 = _gondola_core.monitoring.GcuEvBldStatsMoniData
GcuEvBldStatsMoniData.__module__      = __name__

LTBMoniData                           = _gondola_core.monitoring.LTBMoniData 
LTBMoniData.__module__                = __name__

MasterTriggerHB                       = _gondola_core.monitoring.MasterTriggerHB 
MasterTriggerHB.__module__            = __name__

MtbMoniData                           = _gondola_core.monitoring.MtbMoniData 
MtbMoniData.__module__                = __name__

PAMoniData                            = _gondola_core.monitoring.PAMoniData 
PAMoniData.__module__                 = __name__ 

PBMoniData                            = _gondola_core.monitoring.PBMoniData 
PBMoniData.__module__                 = __name__

RBMoniData                            = _gondola_core.monitoring.PBMoniData 
RBMoniData.__module__                 = __name__

SipPosMoniData                        = _gondola_core.monitoring.SipPosMoniData 
SipPosMoniData.__module__             = __name__ 

SipPresMoniData                       = _gondola_core.monitoring.SipPresMoniData 
SipPresMoniData.__module__            = __name__ 

SipTimeMoniData                       = _gondola_core.monitoring.SipTimeMoniData 
SipTimeMoniData.__module__            = __name__

TrackerGpsMoniData                    = _gondola_core.monitoring.TrackerGpsMoniData 
TrackerGpsMoniData.__module__         = __name__ 

CoolingMoniData                       = _gondola_core.monitoring.CoolingMoniData 
CoolingMoniData.__module__            = __name__

WastieMoniData                        = _gondola_core.monitoring.WastieMoniData 
WastieMoniData.__module__             = __name__

# The corresponding moni series
CPUMoniDataSeries                           = _gondola_core.monitoring.CPUMoniDataSeries        
CPUMoniDataSeries.__module__                = __name__

DataSinkHBSeries                            = _gondola_core.monitoring.DataSinkHBSeries 
DataSinkHBSeries.__module__                 = __name__

EventBuilderHBSeries                        = _gondola_core.monitoring.EventBuilderHBSeries 
EventBuilderHBSeries.__module__             = __name__

GcuEvBldStatsMoniDataSeries                 = _gondola_core.monitoring.GcuEvBldStatsMoniDataSeries
GcuEvBldStatsMoniDataSeries.__module__      = __name__

LTBMoniDataSeries                           = _gondola_core.monitoring.LTBMoniDataSeries 
LTBMoniDataSeries.__module__                = __name__

MasterTriggerHBSeries                       = _gondola_core.monitoring.MasterTriggerHBSeries 
MasterTriggerHBSeries.__module__            = __name__

MtbMoniDataSeries                           = _gondola_core.monitoring.MtbMoniDataSeries 
MtbMoniDataSeries.__module__                = __name__

PAMoniDataSeries                            = _gondola_core.monitoring.PAMoniDataSeries 
PAMoniDataSeries.__module__                 = __name__ 

PBMoniDataSeries                            = _gondola_core.monitoring.PBMoniDataSeries 
PBMoniDataSeries.__module__                 = __name__

RBMoniDataSeries                            = _gondola_core.monitoring.PBMoniDataSeries 
RBMoniDataSeries.__module__                 = __name__

SipPosMoniDataSeries                        = _gondola_core.monitoring.SipPosMoniDataSeries 
SipPosMoniDataSeries.__module__             = __name__ 

SipPresMoniDataSeries                       = _gondola_core.monitoring.SipPresMoniDataSeries 
SipPresMoniDataSeries.__module__            = __name__ 

SipTimeMoniDataSeries                       = _gondola_core.monitoring.SipTimeMoniDataSeries 
SipTimeMoniDataSeries.__module__            = __name__

TrackerGpsMoniDataSeries                    = _gondola_core.monitoring.TrackerGpsMoniDataSeries 
TrackerGpsMoniDataSeries.__module__         = __name__ 

CoolingMoniDataSeries                       = _gondola_core.monitoring.CoolingMoniDataSeries 
CoolingMoniDataSeries.__module__            = __name__

WastieMoniDataSeries                        = _gondola_core.monitoring.WastieMoniDataSeries 
WastieMoniDataSeries.__module__             = __name__

