#ifndef SIMCLASSES_H_INCLUDED
#define SIMCLASSES_H_INCLUDED



/**
 * A container to hold simulated primary information
 *
 */ 
struct SimPrimary {
  u32 pdg;
  f64 theta;
  f64 phi;
  f64 initial_energy_per_nucleon;
  Vec<f64> energy_depositions_kev;
  Vec<u32> volume_id;
  Vec<f64> x;
  Vec<f64> y;
  Vec<f64> z;

  SimPrimary();

  f64 get_beta();

  Vec<u8> to_bytestream();
  
  static SimPrimary from_bytestream(const Vec<u8> &bytestream,
                                    u64 &pos);
}


#endif
