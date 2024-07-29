#import gaps_tof as gt
#
#def get_sensors(moni : list[gt.RBMoniData]):
#    sensors = dict()
#    sensors["tmp_drs"         ]  = [k.tmp_drs for k in moni]   
#    sensors["tmp_clk"         ]  = [k.tmp_clk for k in moni]    
#    sensors["tmp_adc"         ]  = [k.tmp_adc for k in moni]   
#    sensors["tmp_zynq"        ]  = [k.tmp_zynq for k in moni]  
#    sensors["tmp_lis3mdltr"   ]  = [k.tmp_lis3mdltr for k in moni]   
#    sensors["tmp_bm280"       ]  = [k.tmp_bm280 for k in moni] 
#    sensors["pressure"        ]  = [k.pressure for k in moni] 
#    sensors["humidity"        ]  = [k.humidity for k in moni]    
#    sensors["max_x"           ]  = [k.mag_x for k in moni]       
#    sensors["mag_y"           ]  = [k.mag_y for k in moni]           
#    sensors["mag_z"           ]  = [k.mag_z for k in moni]            
#    sensors["mag_tot"         ]  = [k.mag_tot for k in moni]        
#    sensors["drs_dvdd_voltage"]  = [k.drs_dvdd_voltage for k in moni]   
#    sensors["drs_dvdd_current"]  = [k.drs_dvdd_current for k in moni]   
#    sensors["drs_dvdd_power"  ]  = [k.drs_dvdd_power for k in moni]    
#    sensors["p3v3_voltage"    ]  = [k.p3v3_voltage for k in moni]    
#    sensors["p3v3_current"    ]  = [k.p3v3_current for k in moni]    
#    sensors["p3v3_power"      ]  = [k.p3v3_power   for k in moni]    
#    sensors["zynq_voltage"    ]  = [k.zynq_voltage for k in moni]    
#    sensors["zynq_current"    ]  = [k.zynq_current for k in moni]    
#    sensors["zynq_power"      ]  = [k.zynq_power for k in moni]    
#    sensors["p3v5_voltage"    ]  = [k.p3v5_voltage for k in moni]    
#    sensors["p3v5_current"    ]  = [k.p3v5_current for k in moni]    
#    sensors["p3v5_power"      ]  = [k.p3v5_power for k in moni]    
#    sensors["adc_dvdd_voltage"]  = [k.adc_dvdd_voltage for k in moni]    
#    sensors["adc_dvdd_current"]  = [k.adc_dvdd_current for k in moni]    
#    sensors["adc_dvdd_power"  ]  = [k.adc_dvdd_power for k in moni]    
#    sensors["adc_avdd_voltage"]  = [k.adc_avdd_voltage for k in moni]    
#    sensors["adc_avdd_current"]  = [k.adc_avdd_current for k in moni]    
#    sensors["adc_avdd_power"  ]  = [k.adc_avdd_power for k in moni]    
#    sensors["drs_avdd_voltage"]  = [k.drs_avdd_voltage for k in moni]    
#    sensors["drs_avdd_current"]  = [k.drs_avdd_current for k in moni]    
#    sensors["drs_avdd_power"  ]  = [k.drs_avdd_power for k in moni]    
#    sensors["n1v5_voltage"    ]  = [k.n1v5_voltage for k in moni]   
#    sensors["n1v5_current"    ]  = [k.n1v5_current for k in moni]    
#    sensors["n1v5_power"      ]  = [k.n1v5_power for k in moni]
#    return sensors
#
#def get_temperature_group(sensors):
#    temps = dict()
#    for k in sensors:
#        if k.startswith('tmp'):
#            temps[k] = sensors[k]
#
#    return temps
#
#def get_voltage_group(sensors):
#    volts = dict()
#    for k in sensors:
#        if k.endswith('voltage'):
#            volts[k] = sensors[k]
#    return volts
#
#def get_current_group(sensors):
#    currents = dict()
#    for k in sensors:
#        if k.endswith('current'):
#            currents[k] = sensors[k]
#    return currents
#
#def get_power_group(sensors):
#    powers = dict()
#    for k in sensors:
#        if k.endswith('power'):
#            powers[k] = sensors[k]
#    return powers
#
