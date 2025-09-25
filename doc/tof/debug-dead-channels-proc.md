---
title: "Debugging dead TOF channels"
author: A Stoessl
date: June 12, 2024  
geometry: margin=2cm
output: pdf_document
---

# Debugging dead TOF channels

## How to check for dead channels?

The `liftof-tui` program (installed on gse machines under $HOME/tof-moni) and
called with `./liftof-tui` can display waveforms in the terminal.
To do the TOF checkout, the TOF has to be set to trigger on the *ANY* trigger,
which means each paddle triggers independently.

A dataset of at least 30 mins should be saved to disk.

* The suggested procedure is a guideline and might need to be adapted depending on the situation

* The most likely issue to occur seem to be a power connector issue which might be resolved by reseating power connectors

* Please consult with Field about switching RATs on/off.

![Procedure diagram](debug-dead-channels.pdf)


## Notes:


---


---


---


---


---


---


---
