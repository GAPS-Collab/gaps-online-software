/***************************************************
 * C++ bindings to access the GAPS-DB 
 *
 *
 */

#ifndef TOFMANIFEST_H_INCLUDED
#define TOFMANIFEST_H_INCLUDED

struct Paddle {
  u16 paddle_id;
  u64 volume_id;
  std::string pos_in_paddle;
  f32 height;
  f32 width;
  f32 length;
  std::string unit;
  f32 global_pos_x_l0;
  f32 global_pos_y_l0;
  f32 global_pos_z_l0;
}

#endif
