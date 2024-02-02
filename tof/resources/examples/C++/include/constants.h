#ifndef CONSTANTS_H_INCLUDED
#define CONSTANTS_H_INCLUDED

// These may be defined elsewhere, but I don't know where. So, I am
// going to put them here until this is integrated into the DAQ. JAZ

const int NRB   = 50; // Technically, it is 49, but we don't use 0
const int NCH   = 8;
const int NTOT  = NCH * NRB; // NTOT is the number of SiPMs
const int NPAD  = NTOT/2;        // NPAD: 1 per 2 SiPMs

#endif
