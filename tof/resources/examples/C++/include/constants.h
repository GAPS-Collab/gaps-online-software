#ifndef CONSTANTS_H_INCLUDED
#define CONSTANTS_H_INCLUDED

// These may be defined elsewhere, but I don't know where. So, I am
// going to put them here until this is integrated into the DAQ. JAZ

const int NRB   = 50; // Technically, it is 49, but we don't use 0
const int NCH   = 8;
const int NTOT  = NCH * NRB; // NTOT is the number of SiPMs
const int NPAD  = NTOT/2;        // NPAD: 1 per 2 SiPMs

struct PaddleInfo {
  int   VolumeID[NPAD];            //
  float Location[NPAD][3];         // X, Y, Z in detector coordinates
  int   Orientation[NPAD];         // Orientation in detector (e.g. +/-X)
  float Dimension[NPAD][3];        // Physical size in mm
  float CoaxLen[NPAD];             // Coax cable length (ns)
  float HardingLen[NPAD];          // Harding cable length (ns)
  int   SiPM_A[NPAD];              // SiPM channel of A end
  int   SiPM_B[NPAD];              // SiPM channel of B end
  bool  IsUmbrella[NPAD];          // Paddle in the Umbrella
  bool  IsCube[NPAD];              // Paddle in the cube
  bool  IsCortina[NPAD];           // Paddle in the cortina
};

struct SiPMInfo {
  int PB[NTOT];                  // PB supplying this SiPM
  int PB_ch[NTOT];               // Channel on PB
  int LTB[NTOT];                 // LTB reading this SiPM
  int LTB_ch[NTOT];              // Channel on LTB
  int RB[NTOT];                  // RB reading out this SiPM
  int RB_ch[NTOT];               // Channel on RB
  int PaddleID[NTOT];            // Which paddle contains this SiPM
  int PaddleEnd[NTOT];           // 0->A, 1->B
};

#endif
