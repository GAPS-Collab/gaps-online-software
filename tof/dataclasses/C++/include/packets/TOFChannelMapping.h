#ifndef TCHANNELPADDLEMAPPING_H_INCLUDED
#define TCHANNELPADDLEMAPPING_H_INCLUDED

/**
 * Provide relationship between paddle and channel
 * FIXME/TODO - make this configurable during the
 * configure setep when building.
 * Extract information from file or db
 */

#include <map>
#include <utility>

namespace TOF {

/**
 * Extract the corresponding channels from a channel map for a certain
 * paddle id
 *
 */
std::pair<uint16_t, uint16_t> FindChannels(std::map<uint16_t, uint8_t> channelmap, uint8_t paddle_id) 
{
  std::pair<uint16_t, uint16_t> paddle_channels;
  bool channel0_found = false;

  for (auto const &k : channelmap)
    { 
      if ((k.second == paddle_id) & !channel0_found)
        { 
          paddle_channels.first = k.first;
          channel0_found = true;
        }
      if ((k.second == paddle_id) & channel0_found)
        {
          paddle_channels.second = k.first;
          break;
        }
      continue;
    }
  return paddle_channels;
}


/**
 * Construct the map which links paddles to channels, 
 * that is for each paddle_id (key) have a pair of channels
 * on this paddle
 *
 *
 */
std::map<uint8_t, std::pair<uint16_t, uint16_t>> BuildPaddleMap
(std::map<uint16_t, uint8_t> channelmap)
{
    std::map<uint8_t, std::pair<uint16_t, uint16_t>> paddlemap;
    size_t nchannels = channelmap.size();
    uint8_t paddle_id;
    for(size_t k=0;k<nchannels;k++)
      {
        paddle_id = channelmap[k];
        paddlemap[paddle_id]  = FindChannels(channelmap, paddle_id);
      }
    return paddlemap;
}


// channel -> paddle id
static const std::map <uint16_t, uint8_t>
  CHANNELMAP 
   = {
{ 0, 0},
{ 1, 0},
{ 2, 1},
{ 3, 1},
{ 4, 2},
{ 5, 2},
{ 6, 3},
{ 7, 3},
{ 8, 4},
{ 9, 4},
{10, 5},
{11, 5},
{12, 6},
{13, 6},
{14, 7},
{15, 7},
{16, 8},
{17, 8},
{18, 9},
{19, 9},
{20,10},
{21,10},
{22,11},
{23,11},
{24,12},
{25,12},
{26,13},
{27,13},
{28,14},
{29,14},
{30,15},
{31,15},
{32,16},
{33,16},
{34,17},
{35,17},
{36,18},
{37,18},
{38,19},
{39,19}

};

// paddle id -> channel
// (inverse of above)
static const std::map <uint8_t, std::pair<uint16_t, uint16_t>>
  PADDLEMAP = BuildPaddleMap(CHANNELMAP);

// find out if a channel is either A or B side
// "0" means "A" and "1" means "B"
static const std::map <uint16_t, uint8_t>
  CHANNELABMAP = {
  { 0, 0},
  { 1, 1},
  { 2, 0},
  { 3, 1},
  { 4, 0},
  { 5, 1},
  { 6, 0},
  { 7, 1},
  { 8, 0},
  { 9, 1},
  {10, 0},
  {11, 1},
  {12, 0},
  {13, 1},
  {14, 0},
  {15, 1},
  {16, 0},
  {17, 1},
  {18, 0},
  {19, 1},
  {20, 0},
  {21, 1},
  {22, 0},
  {23, 1},
  {24, 0},
  {25, 1},
  {26, 0},
  {27, 1},
  {28, 0},
  {29, 1},
  {30, 0},
  {31, 1},
  {32, 0},
  {33, 1},
  {34, 0},
  {35, 1},
  {36, 0},
  {37, 1},
  {38, 0},
  {39, 1}
};


// which paddle id is part of outer/inner tof
// paddleid -> tof subsystem (0 outer, 1 inner)
static const std::map<uint8_t, uint8_t> 
  TOFSUBSYSTEMMAP = {
  { 0, 0},
  { 1, 0},
  { 2, 0},
  { 3, 0},
  { 4, 0},
  { 5, 0},
  { 6, 0},
  { 7, 0},
  { 8, 0},
  { 9, 0},
  {10, 0},
  {11, 0},
  {12, 0},
  {13, 0},
  {14, 0},
  {15, 0},
  {16, 0},
  {17, 0},
  {18, 0},
  {19, 0}
};

/**
 * Invert a readoutboard -> channel map
 *
 */
std::map<uint16_t, uint8_t> BuildChannelReadoutBoardMap(
    std::map<uint8_t, std::vector<uint16_t>> rbchannelmap)
{
  std::map<uint16_t, uint8_t> channelrbmap;
  for (auto const &k : rbchannelmap)
    for (auto const &ch : k.second)
      { 
        channelrbmap[ch] = k.first;    
      }
  return channelrbmap;
}

// association readout board -> channels
static const std::map<uint8_t, std::vector<uint16_t>> 
  RBCHANNELMAP = {
  { 0, { 0, 1, 2, 3, 4, 5, 6, 7}},
  { 1, { 8, 9,10,11,12,13,14,15}},
  { 2, {16,17,18,19,20,21,22,23}},
  { 3, {24,25,26,27,28,29,30,31}},
  { 4, {32,33,34,35,36,37,38,39}},
  { 5, {40,41,42,43,44,45,46,47}}
};

// channel -> readout board
static const std::map<uint16_t, uint8_t> 
  CHANNELRBMAP = BuildChannelReadoutBoardMap(RBCHANNELMAP);


} //end namespace
#endif
