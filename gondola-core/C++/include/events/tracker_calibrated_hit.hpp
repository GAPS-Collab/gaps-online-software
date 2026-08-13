/// This file is part of gaps-online-software and published 
/// under the GPLv3 license
  
#pragma once
  
namespace gondola {
  struct TrkCalibratedHit {
    u16 strip_id;
    u16 adc;
    //calibrated_hit(uint16_t strip_id, uint16_t adc) : strip_id(strip_id), adc(adc) {}
  };
}
