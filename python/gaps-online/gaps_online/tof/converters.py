"""
Read/Write data taken with gaps_online_software
"""
import gaps_tof as gt
from dataclasses import dataclass

import numpy as np
import h5py

@dataclass
class RBMoniData:
     clock_cycles            : np.int64
     board_id                : np.int32
     rate                    : np.float32
     tmp_drs                 : np.float32
     tmp_clk                 : np.float32
     tmp_adc                 : np.float32
     tmp_zynq                : np.float32
     tmp_lis3mdltr           : np.float32
     tmp_bm280               : np.float32
     pressure                : np.float32
     humidity                : np.float32
     mag_x                   : np.float32
     mag_y                   : np.float32
     mag_z                   : np.float32
     mag_tot                 : np.float32
     drs_dvdd_voltage        : np.float32
     drs_dvdd_current        : np.float32
     drs_dvdd_power          : np.float32
     p3v3_voltage            : np.float32
     p3v3_current            : np.float32
     p3v3_power              : np.float32
     zynq_voltage            : np.float32
     zynq_current            : np.float32
     zynq_power              : np.float32
     p3v5_voltage            : np.float32
     p3v5_current            : np.float32
     p3v5_power              : np.float32
     adc_dvdd_voltage        : np.float32
     adc_dvdd_current        : np.float32
     adc_dvdd_power          : np.float32
     adc_avdd_voltage        : np.float32
     adc_avdd_current        : np.float32
     adc_avdd_power          : np.float32
     drs_avdd_voltage        : np.float32
     drs_avdd_current        : np.float32
     drs_avdd_power          : np.float32
     n1v5_voltage            : np.float32
     n1v5_current            : np.float32
     n1v5_power              : np.float32
      
     @property
     def dtype(self):
         return np.dtype([
                 ('clock_cycles', np.int64),
                 ('board_id', np.int32),
                 ('rate', np.float32),
                 ('tmp_drs', np.float32),
                 ('tmp_clk', np.float32),
                 ('tmp_adc', np.float32),
                 ('tmp_zynq', np.float32),
                 ('tmp_lis3mdltr', np.float32),
                 ('tmp_bm280', np.float32),
                 ('pressure', np.float32),
                 ('humidity', np.float32),
                 ('mag_x', np.float32),
                 ('mag_y', np.float32),
                 ('mag_z', np.float32),
                 ('mag_tot', np.float32),
                 ('drs_dvdd_voltage', np.float32),
                 ('drs_dvdd_current', np.float32),
                 ('drs_dvdd_power', np.float32),
                 ('p3v3_voltage', np.float32),
                 ('p3v3_current', np.float32),
                 ('p3v3_power', np.float32),
                 ('zynq_voltage', np.float32),
                 ('zynq_current', np.float32),
                 ('zynq_power', np.float32),
                 ('p3v5_voltage', np.float32),
                 ('p3v5_current', np.float32),
                 ('p3v5_power', np.float32),
                 ('adc_dvdd_voltage', np.float32),
                 ('adc_dvdd_current', np.float32),
                 ('adc_dvdd_power', np.float32),
                 ('adc_avdd_voltage', np.float32),
                 ('adc_avdd_current', np.float32),
                 ('adc_avdd_power', np.float32),
                 ('drs_avdd_voltage', np.float32),
                 ('drs_avdd_current', np.float32),
                 ('drs_avdd_power', np.float32),
                 ('n1v5_voltage', np.float32),
                 ('n1v5_current', np.float32),
                 ('n1v5_power', np.float32)])

     def __init__(self, tofpacket):
         if tofpacket.packet_type == gt.PacketType.MasterTrigger:
             mte = gt.MasterTriggerEvent.from_bytestream(tofpacket.payload, 0)
             self.clock_cycles = mte.timestamp


     def __array__(self, dt=None) -> np.ndarray:
         if dt is None:
             dt = self.dtype
         return np.array([
             (self.clock_cycles    ,
             self.board_id         ,
             self.rate             ,
             self.tmp_drs          ,
             self.tmp_clk          ,
             self.tmp_adc          ,
             self.tmp_zynq         ,
             self.tmp_lis3mdltr    ,
             self.tmp_bm280        ,
             self.pressure         ,
             self.humidity         ,
             self.mag_x            ,
             self.mag_y            ,
             self.mag_z            ,
             self.mag_tot          ,
             self.drs_dvdd_voltage ,
             self.drs_dvdd_current ,
             self.drs_dvdd_power   ,
             self.p3v3_voltage     ,
             self.p3v3_current     ,
             self.p3v3_power       ,
             self.zynq_voltage     ,
             self.zynq_current     ,
             self.zynq_power       ,
             self.p3v5_voltage     ,
             self.p3v5_current     ,
             self.p3v5_power       ,
             self.adc_dvdd_voltage ,
             self.adc_dvdd_current ,
             self.adc_dvdd_power   ,
             self.adc_avdd_voltage ,
             self.adc_avdd_current ,
             self.adc_avdd_power   ,
             self.drs_avdd_voltage ,
             self.drs_avdd_current ,
             self.drs_avdd_power   ,
             self.n1v5_voltage     ,
             self.n1v5_current     ,
             self.n1v5_power       )], dtype=dt)

     def add_tofpacket(self, tofpacket):
         if int(tofpacket.packet_type) == 100:
             moni = gt.RBMoniData.from_bytestream(tofpacket.payload, 0)
         else:
             ValueError("This is not a RBMoniPacket!")
         self.board_id               = moni.board_id
         self.rate                   = moni.rate
         self.tmp_drs                = moni.tmp_drs
         self.tmp_clk                = moni.tmp_clk
         self.tmp_adc                = moni.tmp_adc
         self.tmp_zynq               = moni.tmp_zynq
         self.tmp_lis3mdltr          = moni.tmp_lis3mdltr
         self.tmp_bm280              = moni.tmp_bm280
         self.pressure               = moni.pressure
         self.humidity               = moni.humidity
         self.mag_x                  = moni.mag_x
         self.mag_y                  = moni.mag_y
         self.mag_z                  = moni.mag_z
         self.mag_tot                = moni.mag_tot
         self.drs_dvdd_voltage       = moni.drs_dvdd_voltage
         self.drs_dvdd_current       = moni.drs_dvdd_current
         self.drs_dvdd_power         = moni.drs_dvdd_power
         self.p3v3_voltage           = moni.p3v3_voltage
         self.p3v3_current           = moni.p3v3_current
         self.p3v3_power             = moni.p3v3_power
         self.zynq_voltage           = moni.zynq_voltage
         self.zynq_current           = moni.zynq_current
         self.zynq_power             = moni.zynq_power
         self.p3v5_voltage           = moni.p3v5_voltage
         self.p3v5_current           = moni.p3v5_current
         self.p3v5_power             = moni.p3v5_power
         self.adc_dvdd_voltage       = moni.adc_dvdd_voltage
         self.adc_dvdd_current       = moni.adc_dvdd_current
         self.adc_dvdd_power         = moni.adc_dvdd_power
         self.adc_avdd_voltage       = moni.adc_avdd_voltage
         self.adc_avdd_current       = moni.adc_avdd_current
         self.adc_avdd_power         = moni.adc_avdd_power
         self.drs_avdd_voltage       = moni.drs_avdd_voltage
         self.drs_avdd_current       = moni.drs_avdd_current
         self.drs_avdd_power         = moni.drs_avdd_power
         self.n1v5_voltage           = moni.n1v5_voltage
         self.n1v5_current           = moni.n1v5_current
         self.n1v5_power             = moni.n1v5_power


def save_to_hdf(moni_data, filename, dataset_name='RBMoniData'):
    """
    Save all RBMoniData to a new hdf file
    """
    with h5py.File(filename, 'w') as file:
        # Check if the dataset already exists, create it if not
        dt = moni_data[0].dtype
        file.create_dataset(dataset_name, shape=moni_data.shape, dtype=dt, data=moni_data)
        file.close()
        #dt = np.dtype([
        #    ('clock_cycles', np.int32),
        #    ('board_id', np.int32),
        #    ('rate', np.float32),
        #    ('tmp_drs', np.float32),
        #    ('tmp_clk', np.float32),
        #    ('tmp_adc', np.float32),
        #    ('tmp_zynq', np.float32),
        #    ('tmp_lis3mdltr', np.float32),
        #    ('tmp_bm280', np.float32),
        #    ('pressure', np.float32),
        #    ('humidity', np.float32),
        #    ('mag_x', np.float32),
        #    ('mag_y', np.float32),
        #    ('mag_z', np.float32),
        #    ('mag_tot', np.float32),
        #    ('drs_dvdd_voltage', np.float32),
        #    ('drs_dvdd_current', np.float32),
        #    ('drs_dvdd_power', np.float32),
        #    ('p3v3_voltage', np.float32),
        #    ('p3v3_current', np.float32),
        #    ('p3v3_power', np.float32),
        #    ('zynq_voltage', np.float32),
        #    ('zynq_current', np.float32),
        #    ('zynq_power', np.float32),
        #    ('p3v5_voltage', np.float32),
        #    ('p3v5_current', np.float32),
        #    ('p3v5_power', np.float32),
        #    ('adc_dvdd_voltage', np.float32),
        #    ('adc_dvdd_current', np.float32),
        #    ('adc_dvdd_power', np.float32),
        #    ('adc_avdd_voltage', np.float32),
        #    ('adc_avdd_current', np.float32),
        #    ('adc_avdd_power', np.float32),
        #    ('drs_avdd_voltage', np.float32),
        #    ('drs_avdd_current', np.float32),
        #    ('drs_avdd_power', np.float32),
        #    ('n1v5_voltage', np.float32),
        #    ('n1v5_current', np.float32),
        #    ('n1v5_power', np.float32),
        #])
        #if dataset_name not in file:
        #    dataset = file.create_dataset(dataset_name, shape=moni_data.shape, dtype=dt, data=moni_data)
        #else:
        #    dataset = file[dataset_name]

        # Convert data instances to a structured numpy array
        #data_array = np.array(moni_data)
        #dataset[:] = data_array

def extract_moni_data(filename):
    """
    Get RBMoniData from a stream and save it as an hdf file
    """
    pack = gt.get_tofpackets(filename)
    last_mte = None
    all_moni = []
    for p in pack:
        if p.packet_type == gt.PacketType.MasterTrigger:
             last_mte = p

        if int(p.packet_type) == 100:
            moni = RBMoniData(last_mte)
            moni.add_tofpacket(p)
            all_moni.append(moni)

    return all_moni
