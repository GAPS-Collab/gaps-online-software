# GAPS TOF channel/geometry mappings



### MTB to LTB

* The MTB has 5 DSI connectors (we use only 4) 
* Each DSI connector has 5 J connectors
* The bitmask thus works the following:
    - 32bit 
    - bit 0 (lsb) DSI0-J1
    - bit 1 (lsb) DSI0-J2
    - bit 2 (lsb) DSI0-J3
    ...
    - bit 6 (lsb) DSI1-j1

