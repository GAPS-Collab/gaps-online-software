## Tof (Physics) data format

As of gaps-online-software version >= 0.7 (OMILU _Bluefin Trevali_) the format for regualar GAPS TOF physics data has more or less stabilized. Regualar data will be saved in a file with the ending `.tof.gaps` and contain a list of `TofPacket`. 
Each `TofPacket` will then be most likely contain a `TofEvent</code> or a housekeeping packet, or anything else, e.g. an Alert we were sending.

===TOF event structures===
The dataformat of the `TofEvent` as of gaps-online-software OMILU-0.7 is like the following:

```
  <TofPacket - PacketType::TofEvent   <- TofPacket is not part of TofEvent, but it the first layer in the datastream
    <TofEvent
      u8 compression_level     <- might become deprecated soon (maybe even is)
      u8 quality               <- either Unknown, Silver, Gold, Diamond or FourLeafClover
      <TofEventHeader
        u32 run_id         
        u32 event_id          
        u32 timestamp32      
        u16 Timestamp16      
        u16 prim_beta       
        u16 prim_beta_unc    
        u16 prim_charge       <- primary reconstructed quantities
        u16 prim_charge_unc  
        u16 prim_outer_tof_x  
        u16 prim_outer_tof_y  
        u16 prim_outer_tof_z  
        u16 prim_inner_tof_x  
        u16 prim_inner_tof_y 
        u16 prim_inner_tof_z  
        u8  nhit_outer_tof
        u8  nhit_inner_tof
        u8  ctr_etx   
        u8  npaddles       >
      <MasterTriggerEvent 
        u32  event_id       
        u32  timestamp 
        u32  tiu_timestamp  
        u32  tiu_gps_32     
        u32  tiu_gps_16     
        u8   n_paddles      
        bool board_mask [N_LTBS]
        bool hits       [N_LTBS][N_CHN_PER_LTB]
        u32  crc           
        bool broken       >   
      Vec<RBEvent> [
        <RBEvent
          <RBEventHeader
            u8   channel_mask          
            u16  stop_cell             
            u32  crc32                 
            u16  dtap0                 
            u16  drs4_temp             
            bool is_locked             
            bool is_locked_last_sec    
            bool lost_trigger          
            u16  fpga_temp             
            u32  event_id              
            u8   rb_id                 
            u64  timestamp_48          
            bool broken      >        
            Vec<Vec<u16>> adc [NCHAN:[NWORDS]]    <- len of the vector is the bitsum of channel mask
            Vec<TofHit> [
              <TofHit 
                u8 paddle_id;
                u16 time_a
                u16 time_b
                u16 peak_a
                u16 peak_b
                u16 charge_a
                u16 charge_b
                u16 charge_min_i
                u16 x_pos
                u16 t_average
                u32 timestamp_32
                u16 timestamp_16  >
              .. .. 
            ]>            <- End RBEvent
      .. ..]              <- End Vec<RBEvent>
      Vec<RBMissingHit> [   <- Missing hits MTB hit, but no RB hit received
        u32 event_id       
        u8  ltb_hit_index  
        u8  ltb_id         
        u8  ltb_dsi       
        u8  ltb_j          
        u8  ltb_ch         
        u8  rb_id          
        u8  rb_ch    >
        .. ..       
      ]                    <- end Vec<RBMissingHit>
    >                      <- end TofEvent
  >                        <- end TofPacket
```

Please note the nested structure. Access methods will be provided, so that the user doesn't have to go too far into the tree. Currently, there are several redundant fields, however, they are helping with the debugging, e.g. we can double check if all the timestamps are in sync, or the event buidler is combining hits with the correct event ids. When we gain more experience, we might want to remove some of the redundant fields. 

The event format is flexible - as well as the number of `RBEvent`, `RBMissingHit` might change and `RBEvent` is of flexible size too.

`RBMissingHit` is currently a helper to identify hits which we have seen by the master trigger, howwever, we did not get RBEvent information for these in time.

The *size in memory* of this structure is flexible, but currently it is the following
```
  <TofEvent
   quality and compression level  :    2 bytes
   TofEventHeader                 :   43 bytes
 
   nboard * RBEvent 
     RBEventHeader                :   35 bytes 
     adc                          : 2048 bytes * nchan
     TofHit                       :   30 bytes * nhits 
   nmissing * RBMissingHit        :   15 bytes * missing >
```

This give us for example for an average number of hits of 5 paddles on 5 different boards and a missing hit a size of
2 + 43 + 5[boards]*(35 + 2*2048 + 30) + 15 = **20865 bytes**

