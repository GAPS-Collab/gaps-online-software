---
title: "TOF Checkout at CSBF"
author: A Stoessl
date: June 12, 2024  
geometry: margin=2cm
output: pdf_document
---


# TOF Checkout at CSBF

## General notes

* Among the issues with the paddles, the most frequent issue was a a non-working power 
  connection either on the SiPM or on the RAT side.
* One RB ethernet connection had to be reseated. The RB was operating normally but then 
  disconnected from the network. After reseating the ethernet cable on the RAT side, it 
  connected again.
  _the switch-control is very helpful in debugging ethernet issues - it shows a mask with link status for each port_
* The short ethernet cables of RAT 3 were extended with the tthernet extender cables
* Currently, in total there are 5 bad channels


## Paddles with issues

| Paddle ID | Panel    | Issue     | Comment | HG | LG    |  
|-----------|----------|-----------|---------|----|-------|
| 12B       | CBE top  | No pulses | Following the debugging procedure, it seems the power connector on the RAT end is not working | x  | (?) not checked | 
| 44A       | CBE side | No pulses | Long debugging session with Philip,Erik,Achim and the scope. No signal at the scope at all when connecting to HG and LG. Might be a dead preampp | x | x | 
| 48A       | CBE side | No pulses | Reseating the power connector on SiPM fixed the issue | Y  | Y |
| 49A       | CBE side | Strange double shaped pulse on the scope | LG ok, trigger ok, pulse shape seems to be consistently looking differently, might be salvageable | Y | Y | 
| 59A       | CBE edge (top) | No pulse | Reseating power connector and swapping power connector on RAT side did not help. Since the SiPM was already covered in foam, no further possibilities of debugging. | x | x |
| 111A      | COR      | No pulse | Reseating pc on SiPM side fixed the issue | Y  | Y | 
| 124A      | COR      | No pulse | Reseating pc on RAT fixed the issue | Y | Y | 
| 124B      | COR      | No pulse | Reseating pc on RAT fixed the issue. *Since A and B had issues which got fixed simulataneously, we are inclided to think that the issue was actually resolved by the RAT pc* | Y | Y | 
| 144B      | COR      | No pulse | Initalially good, the channel went bad after the 3PP at the radiator side was installed (B is also close to the radiator) | x | x | 
| 147A      | COR      | No pulse | Reseating pc on SiPM side fixed the issue | Y  | Y |
| 153A      | COR EDG     | No pulse | Investigation with the scope showed no LG siganl | x | x | 
| 155B      | COR EDG     | No pulse | Bad power cable. Replacement fixed the issue | Y | Y | 
| 156B      | COR EDG     | No pulse | Reseating pc on RAT side fixed the issue (was not properly locked | Y | Y | 
