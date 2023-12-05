/**))))))*************************************************
 *
 * C++ bindings for tof manifest components. These are
 *
 *  - RBs
 *  - LTBs
 *  - Paddles
 *  
 *  for the detailed description of the database layout, 
 *  see the gaps-db project in the same repository
 *
 */

#ifndef TOFMANIFEST_H_INCLUDED
#define TOFMANIFEST_H_INCLUDED

struct LTB {
    u16 ltb_id         ; 
    u16 ltb_dsi        ; 
    u16 ltb_j          ; 
    u16 ltb_ch1_rb     ; 
    u16 ltb_ch2_rb     ; 
    u16 ltb_ch3_rb     ; 
    u16 ltb_ch4_rb     ; 
    u16 ltb_ch5_rb     ; 
    u16 ltb_ch6_rb     ; 
    u16 ltb_ch7_rb     ; 
    u16 ltb_ch8_rb     ; 
    u16 ltb_ch9_rb     ; 
    u16 ltb_ch10_rb    ; 
    u16 ltb_ch11_rb    ; 
    u16 ltb_ch12_rb    ; 
    u16 ltb_ch13_rb    ; 
    u16 ltb_ch14_rb    ; 
    u16 ltb_ch15_rb    ; 
    u16 ltb_ch16_rb    ; 
    u16 ltb_ch17_rb    ; 
    u16 ltb_ch18_rb    ; 
    u16 ltb_ch19_rb    ; 
    u16 ltb_ch20_rb    ; 
    u16 ltb_ch1_rb_ch  ; 
    u16 ltb_ch2_rb_ch  ; 
    u16 ltb_ch3_rb_ch  ; 
    u16 ltb_ch4_rb_ch  ; 
    u16 ltb_ch5_rb_ch  ; 
    u16 ltb_ch6_rb_ch  ; 
    u16 ltb_ch7_rb_ch  ; 
    u16 ltb_ch8_rb_ch  ; 
    u16 ltb_ch9_rb_ch  ; 
    u16 ltb_ch10_rb_ch ; 
    u16 ltb_ch11_rb_ch ; 
    u16 ltb_ch12_rb_ch ; 
    u16 ltb_ch13_rb_ch ; 
    u16 ltb_ch14_rb_ch ; 
    u16 ltb_ch15_rb_ch ; 
    u16 ltb_ch16_rb_ch ; 
    u16 ltb_ch17_rb_ch ; 
    u16 ltb_ch18_rb_ch ; 
    u16 ltb_ch19_rb_ch ; 
    u16 ltb_ch20_rb_ch ; 
}



#endif

