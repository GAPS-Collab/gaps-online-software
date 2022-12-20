/***********************************/


pub const RPADDLEPACKETSIZE    : usize = 26;
pub const RPADDLEPACKETVERSION : &str = "rev1.0";


#[derive(Debug,Copy,Clone)]
pub struct PaddlePacket  {
  
  //unsigned short head = 0xF0F0;
  pub head         : u16,
  //pub p_length     : u16,
  pub paddle_id    : u8,
  pub time_a       : u16,
  pub time_b       : u16,
  pub peak_a       : u16,
  pub peak_b       : u16,
  pub charge_a     : u16,
  pub charge_b     : u16,
  pub charge_min_i : u16,
  pub pos_across   : u16,
  pub t_average    : u16,
  pub ctr_etx      : u8,
  pub tail         : u16,

  // fields which won't get 
  // serialized
  pub event_id     : u32
}

impl PaddlePacket {
  pub fn new() -> PaddlePacket {
    PaddlePacket{head         : 61680, // 0xF0F0
                 event_id     : 0,
                 paddle_id    : 0,
                 time_a       : 0,
                 time_b       : 0,
                 peak_a       : 0,
                 peak_b       : 0,
                 charge_a     : 0,
                 charge_b     : 0,
                 charge_min_i : 0,
                 pos_across   : 0,
                 t_average    : 0,
                 ctr_etx      : 0,
                 tail         :3855}// 0xF0F);

  }

  pub fn set_time_a(&mut self, time : f64 ) {
    let prec : f64 = 0.004;
    self.time_a = (time as f64/prec) as u16;
  }

  pub fn set_time_b(&mut self, time : f64 ) {
    let prec : f64 = 0.004;
    self.time_b = (time as f64/prec) as u16;
  }
  
  pub fn set_time(&mut self, time : f64, side : usize ) {
    assert!(side == 0 || side == 1);
    if side == 0 {self.set_time_a(time);}
    if side == 1 {self.set_time_b(time);}
  }

  pub fn reset(&mut self) {
    self.time_a       =  0;
    self.time_b       =  0;
    self.peak_a       =  0;
    self.peak_b       =  0;
    self.charge_a     =  0;
    self.charge_b     =  0;
    self.charge_min_i =  0;
    self.pos_across   =  0;
    self.t_average    =  0;
    self.ctr_etx      =  0;
  }


  pub fn print(&self)
  {
    println!("***** paddle packet *****");
    println!("=> head          {}", self.head);
    println!("=> time_a        {}", self.time_a);
    println!("=> time_b        {}", self.time_b);
    println!("=> peak_a        {}", self.peak_a);
    println!("=> peak_b        {}", self.peak_b);
    println!("=> charge_a      {}", self.charge_a);
    println!("=> charge_b      {}", self.charge_b);
    println!("=> charge_min_i  {}", self.charge_min_i);
    println!("=> pos_across    {}", self.pos_across);
    println!("=> t_average     {}", self.t_average);
    println!("=> ctr_etx       {}", self.ctr_etx);
    println!("=> tail          {}", self.tail);
    println!("*****");

  }
}


#[derive(Debug, Clone)]
pub struct TofEvent  {
  
  //unsigned short head = 0xF0F0;
  pub head         : u16,
  pub event_id     : u32,
  pub n_paddles    : u8, // we don't have more than 
                         // 256 paddles.
                         // HOWEVER!! For future gaps
                         // flights, we might...
                         // This will then overflow 
                         // and cause problems.

  pub paddle_packets : Vec::<PaddlePacket>,

  //pub p_length     : u16,
  pub tail         : u16,

  // fields which won't get 
  // serialized
  pub n_paddles_expected : u8
}


impl TofEvent {

  pub fn new(event_id : u32,
             n_paddles_expected : u8) -> TofEvent {
    TofEvent { 
      head           : 0,
      event_id       : event_id,
      n_paddles      : 0, // we don't have more than 
      paddle_packets : Vec::<PaddlePacket>::new(),
      tail           : 0,

      n_paddles_expected : n_paddles_expected
    }
  }

  pub fn is_complete(&self) -> bool {
    self.n_paddles == self.n_paddles_expected
  }

}
  //  unsigned short p_length= RPADDLEPACKETSIZE;
//
//  unsigned char paddle_id;
//  unsigned short time_a;
//  unsigned short time_b;
//  unsigned short peak_a;
//  unsigned short peak_b;
//  unsigned short charge_a;
//  unsigned short charge_b;
//  unsigned short charge_min_i;
//  unsigned short x_pos;
//  unsigned short t_average;
//
//  unsigned char ctr_etx;
//  unsigned short tail = 0xF0F;
//
//  // convert the truncated values
//  unsigned short get_paddle_id() const;
//  float get_time_a()             const;
//  float get_time_b()             const;
//  float get_peak_a()             const;
//  float get_peak_b()             const;
//  float get_charge_a()     const;
//  float get_charge_b()     const;
//  float get_charge_min_i() const;
//  float get_x_pos()        const;
//  float get_t_avg()        const;
//  // setters
//  void set_time_a(double);
//  void set_time_b(double);
//  void set_peak_a(double);
//  void set_peak_b(double);
//  void set_charge_a(double);
//  void set_charge_b(double);
//  void set_charge_min_i(double);
//  void set_x_pos(double);
//  void set_t_avg(double);
//
//  // don't serialize (?)
//  std::string version = RPADDLEPACKETVERSION; // packet version
//
//
//  // PaddlePacket legth is fixed
//  static unsigned short calculate_length();
//  void reset();
//
//  std::vector<unsigned char> serialize() const;
//  unsigned int deserialize(std::vector<unsigned char>& bytestream,
//                                unsigned int start_pos);
//
//  // easier print out
//  std::string to_string() const;
//}

//std::ostream& operator<<(std::ostream& os, const RPaddlePacket& pad);


/***********************************/

/***********************************/

