"""
Every possible event type in GAPS which is relevant for 
"online" analysis 
"""

from . import _gondola_core 

TofHit                           =  _gondola_core.events.TofHit             
TofHit.__module__                = __name__
TofHit.__name__                  = 'TofHit'
TrackerHit                       =  _gondola_core.events.TrackerHit         
TrackerHit.__module__            = __name__ 
TrackerHit.__name__              = 'TrackerHit'
RBEventHeader                    =  _gondola_core.events.RBEventHeader      
RBEventHeader.__module__         = __name__ 
RBEventHeader.__name__           = 'RBEventHeader' 
RBEvent                          =  _gondola_core.events.RBEvent    
RBEvent.__module__               = __name__ 
RBEvent.__name__                 = 'RBEvent'
RBWaveform                       =  _gondola_core.events.RBWaveform    
RBWaveform.__module__            = __name__
RBWaveform.__name__              = 'RBWaveform'

TofEvent                         =  _gondola_core.events.TofEvent           
TofEvent.__module__              = __name__ 
TofEvent.__name__                = 'TofEvent' 
TelemetryEvent                   =  _gondola_core.events.TelemetryEvent     
TelemetryEvent.__module__        = __name__ 
TelemetryEvent.__name__          = 'TelemetryEvent' 
TrackerDAQEvent                  = _gondola_core.events.TrackerDAQEvent 
TrackerDAQEvent.__module__       = __name__ 
TrackerDAQEvent.__name__         = 'TrackerDAQEvent' 
TrackerDAQEventPacket            = _gondola_core.events.TrackerDAQEventPacket
TrackerDAQEventPacket.__module__ = __name__ 
TrackerDAQEventPacket.__name__   = 'TrackerDAQEventPacket'

McEvent                          = _gondola_core.events.McEvent 
McEvent.__module__               = __name__ 
McEvent.__name__                 = 'McEvent' 

McHit                            = _gondola_core.events.McHit 
McHit.__module__                 = __name__ 
McHit.__name__                   = 'McHit'

# functions 
strip_id                  =  _gondola_core.events.strip_id           
strip_id.__module__       = __name__

# enums
EventQuality              =  _gondola_core.events.EventQuality       
EventQuality.__module__   = __name__ 
TriggerType               =  _gondola_core.events.TriggerType        
TriggerType.__module__    = __name__ 
LTBThreshold              =  _gondola_core.events.LTBThreshold       
LTBThreshold.__module__   = __name__ 
EventStatus               =  _gondola_core.events.EventStatus        
EventStatus.__module__    = __name__
DataType                  =  _gondola_core.events.DataType           
DataType.__module__       = __name__
