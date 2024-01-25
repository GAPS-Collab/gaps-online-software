from django.db import models
# Create your models here.
class LTB(models.Model):
    """
    Representation of a local trigger board
    """
    ltb_id          = models.PositiveSmallIntegerField(primary_key=True, unique=True)
    ltb_dsi         = models.PositiveSmallIntegerField(null=True, help_text="DSI connector number on the MTB")
    ltb_j           = models.PositiveSmallIntegerField(null=True, help_text="J connector number on the MTB")
    ltb_ch1_rb      = models.PositiveSmallIntegerField(null=True)
    ltb_ch2_rb      = models.PositiveSmallIntegerField(null=True)
    ltb_ch3_rb      = models.PositiveSmallIntegerField(null=True)
    ltb_ch4_rb      = models.PositiveSmallIntegerField(null=True)
    ltb_ch5_rb      = models.PositiveSmallIntegerField(null=True)
    ltb_ch6_rb      = models.PositiveSmallIntegerField(null=True)
    ltb_ch7_rb      = models.PositiveSmallIntegerField(null=True)
    ltb_ch8_rb      = models.PositiveSmallIntegerField(null=True)
    ltb_ch9_rb      = models.PositiveSmallIntegerField(null=True)
    ltb_ch10_rb     = models.PositiveSmallIntegerField(null=True)
    ltb_ch11_rb     = models.PositiveSmallIntegerField(null=True)
    ltb_ch12_rb     = models.PositiveSmallIntegerField(null=True)
    ltb_ch13_rb     = models.PositiveSmallIntegerField(null=True)
    ltb_ch14_rb     = models.PositiveSmallIntegerField(null=True)
    ltb_ch15_rb     = models.PositiveSmallIntegerField(null=True)
    ltb_ch16_rb     = models.PositiveSmallIntegerField(null=True)
    ltb_ch17_rb     = models.PositiveSmallIntegerField(null=True)
    ltb_ch18_rb     = models.PositiveSmallIntegerField(null=True)
    ltb_ch19_rb     = models.PositiveSmallIntegerField(null=True)
    ltb_ch20_rb     = models.PositiveSmallIntegerField(null=True)
    ltb_ch1_rb_ch   = models.PositiveSmallIntegerField(null=True)
    ltb_ch2_rb_ch   = models.PositiveSmallIntegerField(null=True)
    ltb_ch3_rb_ch   = models.PositiveSmallIntegerField(null=True)
    ltb_ch4_rb_ch   = models.PositiveSmallIntegerField(null=True)
    ltb_ch5_rb_ch   = models.PositiveSmallIntegerField(null=True)
    ltb_ch6_rb_ch   = models.PositiveSmallIntegerField(null=True)
    ltb_ch7_rb_ch   = models.PositiveSmallIntegerField(null=True)
    ltb_ch8_rb_ch   = models.PositiveSmallIntegerField(null=True)
    ltb_ch9_rb_ch   = models.PositiveSmallIntegerField(null=True)
    ltb_ch10_rb_ch  = models.PositiveSmallIntegerField(null=True)
    ltb_ch11_rb_ch  = models.PositiveSmallIntegerField(null=True)
    ltb_ch12_rb_ch  = models.PositiveSmallIntegerField(null=True)
    ltb_ch13_rb_ch  = models.PositiveSmallIntegerField(null=True)
    ltb_ch14_rb_ch  = models.PositiveSmallIntegerField(null=True)
    ltb_ch15_rb_ch  = models.PositiveSmallIntegerField(null=True)
    ltb_ch16_rb_ch  = models.PositiveSmallIntegerField(null=True)
    ltb_ch17_rb_ch  = models.PositiveSmallIntegerField(null=True)
    ltb_ch18_rb_ch  = models.PositiveSmallIntegerField(null=True)
    ltb_ch19_rb_ch  = models.PositiveSmallIntegerField(null=True)
    ltb_ch20_rb_ch  = models.PositiveSmallIntegerField(null=True)

    def get_channels_to_rb(self):
        """
        Return a dictionary with the following structure:
        LTB channel -> [RB_id, RB_channel]
        """
        ch_to_rb = dict()
        for ch in range(1,21):
            this_rb    = self.__getattribute__(f'ltb_ch{ch}_rb')
            this_rb_ch = self.__getattribute__(f'ltb_ch{ch}_rb_ch')
            ch_to_rb[ch] = [this_rb, this_rb_ch]

        return ch_to_rb

    def is_populated(self):
        if self.ltb_dsi is None:
            return False
        return True

    def set_channels_to_rb(self, data):
        """
        Set a mapping for each ltb channel.

        Args:
            data (dict) : Mapping {LTB CHANNEL -> [RB, RB_CHAN]}
        """
        for ltb_ch in data:
            setattr(self,f'ltb_ch{ltb_ch}_rb', data[ltb_ch][0])
            setattr(self,f'ltb_ch{ltb_ch}_rb_ch', data[ltb_ch][1])

    def __str__(self):
        return self.__repr__()

    def __repr__(self):
        _repr = '<LTB:\n'
        _repr += f'ID  : {self.ltb_id}\n'             
        _repr += f'DSI : {self.ltb_dsi}\n'           
        _repr += f'J   : {self.ltb_j}\n'              
        _repr += f'ch1_rb : {self.ltb_ch1_rb}\n'      
        _repr += f'ch2_rb : {self.ltb_ch2_rb}\n'      
        _repr += f'ch3_rb : {self.ltb_ch3_rb}\n'      
        _repr += f'ch4_rb : {self.ltb_ch4_rb}\n'      
        _repr += f'ch5_rb : {self.ltb_ch5_rb}\n'      
        _repr += f'ch6_rb : {self.ltb_ch6_rb}\n'      
        _repr += f'ch7_rb : {self.ltb_ch7_rb}\n'      
        _repr += f'ch8_rb : {self.ltb_ch8_rb}\n'      
        _repr += f'ch9_rb : {self.ltb_ch9_rb}\n'      
        _repr += f'ch10_rb : {self.ltb_ch10_rb}\n'    
        _repr += f'ch11_rb : {self.ltb_ch11_rb}\n'    
        _repr += f'ch12_rb : {self.ltb_ch12_rb}\n'    
        _repr += f'ch13_rb : {self.ltb_ch13_rb}\n'    
        _repr += f'ch14_rb : {self.ltb_ch14_rb}\n'    
        _repr += f'ch15_rb : {self.ltb_ch15_rb}\n'    
        _repr += f'ch16_rb : {self.ltb_ch16_rb}\n'    
        _repr += f'ch17_rb : {self.ltb_ch17_rb}\n'    
        _repr += f'ch18_rb : {self.ltb_ch18_rb}\n'    
        _repr += f'ch19_rb : {self.ltb_ch19_rb}\n'    
        _repr += f'ch20_rb : {self.ltb_ch20_rb}\n'    
        _repr += f'ch1_rb_ch: {self.ltb_ch1_rb_ch}\n' 
        _repr += f'ch2_rb_ch: {self.ltb_ch2_rb_ch}\n' 
        _repr += f'ch3_rb_ch: {self.ltb_ch3_rb_ch}\n' 
        _repr += f'ch4_rb_ch: {self.ltb_ch4_rb_ch}\n' 
        _repr += f'ch5_rb_ch: {self.ltb_ch5_rb_ch}\n' 
        _repr += f'ch6_rb_ch: {self.ltb_ch6_rb_ch}\n' 
        _repr += f'ch7_rb_ch: {self.ltb_ch7_rb_ch}\n' 
        _repr += f'ch8_rb_ch: {self.ltb_ch8_rb_ch}\n' 
        _repr += f'ch9_rb_ch: {self.ltb_ch9_rb_ch}\n' 
        _repr += f'ch10_rb_ch: {self.ltb_ch10_rb_ch}\n'  
        _repr += f'ch11_rb_ch: {self.ltb_ch11_rb_ch}\n'  
        _repr += f'ch12_rb_ch: {self.ltb_ch12_rb_ch}\n'  
        _repr += f'ch13_rb_ch: {self.ltb_ch13_rb_ch}\n'  
        _repr += f'ch14_rb_ch: {self.ltb_ch14_rb_ch}\n'  
        _repr += f'ch15_rb_ch: {self.ltb_ch15_rb_ch}\n'  
        _repr += f'ch16_rb_ch: {self.ltb_ch16_rb_ch}\n'  
        _repr += f'ch17_rb_ch: {self.ltb_ch17_rb_ch}\n'  
        _repr += f'ch18_rb_ch: {self.ltb_ch18_rb_ch}\n'  
        _repr += f'ch19_rb_ch: {self.ltb_ch19_rb_ch}\n'  
        _repr += f'ch20_rb_ch: {self.ltb_ch20_rb_ch}\n'  
        return _repr


class RBCalibration(models.Model):
    """
    Readoutboard timing + voltage calibration
    """
    pass

class Panel(models.Model):
    """ 
    A tof panel (can be subsection of a face)
    """
    panel_id                  = models.PositiveSmallIntegerField(
                                    unique=True,
                                    primary_key=True)
    desc                      = models.CharField(max_length=128)
    normal_coordinate_no_sign = models.CharField(max_length=1,\
                                                null=True,\
                                                help_text="Unsigned ormal coordinate in the official GAPS coordinate system")
    asc_pid_order             = models.CharField(max_length=2,\
                                                null=True,\
                                                help_text="Signed direction of the ascending paddle ids. E.g. +x would mean paddle ids increase in ascending x direction")

    smallest_paddle_id        = models.PositiveSmallIntegerField(null=True)
    n_paddles                 = models.PositiveSmallIntegerField(null=True)
    measurement_unit          = models.CharField(max_length=2)
    dw_paddle                 = models.PositiveSmallIntegerField(null=True, help_text="The distance between two paddle centers in 'width' direction, that is the second smallest dimenson of the paddle. This is basically the 'overlap'")
    dh_paddle                 = models.PositiveSmallIntegerField(null=True,help_text="The distance between two paddle centers in 'height' direction, thet is the smalles dimension of the paddle. Witout wrapping, this would be the paddle height")

    smallest_paddle_id_x      = models.PositiveSmallIntegerField(null=True,help_text="The global x position of the paddle with the smallest id")
    smallest_paddle_id_y      = models.PositiveSmallIntegerField(null=True,help_text="The global y position of the paddle with the smallest id")
    smallest_paddle_id_z      = models.PositiveSmallIntegerField(null=True,help_text="The global z position of the paddle with the smallest id")

    in_panel_pzddle_no_1      = models.PositiveSmallIntegerField(null=True)
    in_panel_pzddle_no_2      = models.PositiveSmallIntegerField(null=True)
    in_panel_pzddle_no_3      = models.PositiveSmallIntegerField(null=True)


    def fill_from_spreadsheet(self, data):
        self.panel_id = int(data["Panel Number"])
        self.desc     = data["Panel Description"]

    def __str__(self):
        return self.__repr__()

    def __repr__(self):
        _repr = '<Panel:'
        _repr += f'\n  id    : {self.panel_id}'
        _repr += f'\n  descr : {self.desc}>'
        return _repr

class Paddle(models.Model):
    paddle_id                 = models.PositiveSmallIntegerField(unique=True, primary_key=True)
    volume_id                 = models.PositiveBigIntegerField(
                                  unique=True,
                                  help_text="The VolumeId as used in the GAPS simulation code")
    pos_in_panel              = models.CharField(max_length=4,
                                                 null=True,
                                                 default="")
    height                    = models.FloatField(null=True)
    width                     = models.PositiveSmallIntegerField(null=True)
    length                    = models.PositiveSmallIntegerField(null=True)
    unit                      = models.CharField(
                                    null=True,
                                    max_length=2)
    global_pos_x_l0           = models.FloatField(
                                    null=True,
                                    blank=True,
                                    help_text="Global X coordinate from simulation")
    global_pos_y_l0           = models.FloatField(
                                    null=True,
                                    blank=True,
                                    help_text="Global Y coordinate from simulation")
    global_pos_z_l0           = models.FloatField(
                                    null=True,
                                    blank=True,
                                    help_text="Global Z coordinate from simulation")
    def __str__(self):
        return self.__repr__()

    def __repr__(self):
    
        _repr  = '<Paddle:\n'
        _repr += f'  pid            : {self.paddle_id}\n'
        _repr += f'  vid            : {self.volume_id}\n'
        _repr += f'  L0 coord (sim) : [{self.global_pos_x_l0}, {self.global_pos_y_l0}, {self.global_pos_z_l0}] >'
        return _repr

class PaddleEnd(models.Model):
    """
    One end of a paddle with SiPM array
    """
    PADDLE_END    = [('A', 'A'), ('B', 'B')]
    paddle_end_id = models.PositiveSmallIntegerField(
                        primary_key=True,
                        unique=True,
                        help_text="PaddleID + 1000 for A and PaddleID + 2000 for B")
    paddle_id     = models.PositiveSmallIntegerField()

    end           = models.CharField(max_length=1, choices=PADDLE_END)
    end_location  = models.CharField(max_length=2,\
                                    help_text="Location of the paddle end relative to the paddle center")
    panel_id      = models.PositiveSmallIntegerField()
    pos_in_panel  = models.CharField(max_length=4,\
                                     help_text="Identifier in global coordinates about the location in the panel",\
                                     null=True,\
                                     default="")
    cable_length  = models.FloatField(help_text="Cable length in cm")
    rat           = models.PositiveSmallIntegerField()
    ltb_id        = models.PositiveSmallIntegerField()
    rb_id         = models.PositiveSmallIntegerField()
    pb_id         = models.PositiveSmallIntegerField()
    ltb_ch        = models.PositiveSmallIntegerField()
    pb_ch         = models.PositiveSmallIntegerField()
    #FIXME - is this starting from 0 or 1?
    rb_ch         = models.PositiveSmallIntegerField()
    dsi           = models.PositiveSmallIntegerField()
    rb_harting_j  = models.PositiveSmallIntegerField()
    ltb_harting_j = models.PositiveSmallIntegerField()
    
    def setup_unique_paddle_end_id(self):
        """
        Introduce a uuid. We have 160 paddles with 2 ends. Make the uuid the following
        paddle_end_id(end :<A || B>): return 1000 if end == A else 2000
        uuid = paddle_end_id(end) + paddle_id (so the paddle id is preceeded by 1 or 2 and the 
        2nd and 3rd digits are filled up with 0s if necessary
        """
        if self.end == 'A':
            self.paddle_end_id = 1000 + self.paddle_id
        elif self.end == 'B':
            self.paddle_end_id = 2000 + self.paddle_id
        else:
            raise ValueError(f'PaddleEnd unique identifier can not be created for paddle end {self.end} with paddle id {self.paddle_id}')

    def fill_from_spreadsheet(self, data):
        self.paddle_id     = int(data['Paddle Number']) 
        self.end           = data['Paddle End (A/B)'] 
        self.end_location  = data['Paddle End Location']                       
        panel_id           = str(data['Panel Number'])
        if panel_id.startswith('E'):
            # this are these individual edge paddles
            # we replace them with 1000 + the number 
            # after E-X
            panel_id = panel_id.replace("E-X","")
            self.panel_id = int(panel_id) + 1000
        else:
            self.panel_id = int(panel_id)
        self.cable_length  = int(data['Cable length (cm)'] )
        self.rat           = int(data['RAT Number'] )
        ltb_info           = data['LTB Number-Channel'].split('-')
        rb_info            = data['RB Number-Channel'].split('-')
        pb_info            = data['PB Number-Channel'].split('-')
        self.ltb_id        = int(ltb_info[0]) 
        self.rb_id         = int(rb_info[0])
        self.pb_id         = int(pb_info[0])
        self.ltb_ch        = int(ltb_info[1])
        self.pb_ch         = int(pb_info[1] )
        self.rb_ch         = int(rb_info[1] )
        
        # in some spreadsheets, the label differs,
        # so we are just looking for some variant
        good = False
        for label in 'DSI card slot', 'DSI Card Slot', 'DSI card slot ', 'DSI Card Slot ':
            try:
                self.dsi           = int(data[label])
                good               = True
                #print("Found good key!")
                break
            except KeyError:
                #print(f".. can't find key {label}, trying next variant..")
                continue
        if not good:
            raise ValueError("Could not get DSI assignment!")
        rb_h_j             = data['RB Harting Connection'].replace('J','')
        ltb_h_j            = data['LTB Harting Connection'].replace('J','')
        #rb_h_j             = data['RB Harting Connection'].split('_')
        #ltb_h_j            = data['LTB Harting Connection'].split('_')
        self.rb_harting_j  = int(rb_h_j)
        self.ltb_harting_j = int(ltb_h_j)

    def __str__(self):
        return self.__repr__()

    def __repr__(self):
        try:
            panel  = Panel.objects.filter(panel_id=self.panel_id)[0]
        except Exception as e:
            panel  = 'UNKNOWN'
        try:
            paddle = Paddle.objects.filter(paddle_id=self.paddle_id)[0] 
        except:
            paddle = 'UNKNOWN'
        _repr = '<PaddleEnd:'
        _repr += f'\n  ** identifiers **'
        _repr += f'\n   id             : {self.paddle_end_id}'     
        _repr += f'\n   pid            : {self.paddle_id}'     
        _repr += f'\n   end (A|B)      : {self.end}'  
        _repr += f'\n  ** connedtions **'
        _repr += f'\n   DSI/J/CH (LG)  :  {self.dsi} | {self.ltb_harting_j} | {self.ltb_ch:02}'
        _repr += f'\n   DSI/J/CH (HG)  :  {self.dsi} | {self.rb_harting_j} | {self.rb_ch:02}'
        _repr += f'\n   RB/CH          : {self.rb_id:02} | {self.rb_ch:02}'
        _repr += f'\n   PB/CH          : {self.pb_id:02} | {self.pb_ch:02}'
        _repr += f'\n   RAT id         : {self.rat}'
        _repr += f'\n   cable len [cm] :'
        _repr += f'\n    \u21B3 {self.cable_length}'
        _repr += f'\n    (Harting -> RB)'
        _repr += f'\n  ** panel & location **'
        _repr += f'\n   end ->         : {self.end_location}' 
        #_repr += f'\n   panel id       : {self.panel_id}'     
        _repr += f'\n   loc. in panel  : {self.pos_in_panel}'
        _repr += f'\n   {panel}'
        _repr += f'\n   {paddle}>'
        return _repr

class DSICard(models.Model):
    dsi_id          = models.PositiveSmallIntegerField(unique=True, primary_key=True)
    j1_rat_id       = models.PositiveSmallIntegerField(null=True)
    j2_rat_id       = models.PositiveSmallIntegerField(null=True)
    j3_rat_id       = models.PositiveSmallIntegerField(null=True)
    j4_rat_id       = models.PositiveSmallIntegerField(null=True)
    j5_rat_id       = models.PositiveSmallIntegerField(null=True)
  
    def add_rat_id_for_j(self, j, rat):
        if j == 1:
            self.j1_rat_id = rat
        elif j == 2:
            self.j2_rat_id = rat
        elif j == 3:
            self.j3_rat_id = rat
        elif j == 4:
            self.j4_rat_id = rat
        elif j == 5:
            self.j5_rat_id = rat
        else:
            raise ValueError(f'Do not have J connector with id {j}')

    def add_from_spreadsheet(self, data, card_id):
        """
        Fill the values from a spreadsheet. There is a caveat. Due to the dataformat in the 
        spreadsheet, the card_id must be given seperatly.
        """
        all_card_ids = []
        all_fields = data.keys()
        for k in all_fields:
            if k.startswith('DSI card'):
                c_id = k.split(' ')[2]
                all_card_ids.append(int(c_id))
        #print (all_card_ids)
        if not card_id in all_card_ids:
            raise ValueError(f"This row does not seem to contain information about DSI card {card_id}")
        j = data[f'DSI card {card_id}'] 
        print (j)
        if j.endswith('2'):
            print ("Redundant information, skipping this row.")
            return
        j = int(j.split('_')[0][1])
        self.dsi_id          = card_id
        rat_unamed_field = 2*card_id + card_id -2
        print(f'Will check {rat_unamed_field} for DSI {card_id}')
        rat_id = data[f'Unnamed: {rat_unamed_field}']
        
        try:
            rat_id = int(rat_id[7:])
        except ValueError as e:
            print(f'Can not get RAT id for this one. Exception {e}. RAT ID {rat_id}')
            return
        setattr(self, f'j{j}_rat_id', rat_id)
        
    def get_rat(self, j):
        match j:
            case 1:
                return self.j1_rat_id
            case 2:
                return self.j2_rat_id
            case 3:
                return self.j3_rat_id
            case 4:
                return self.j4_rat_id
            case 5:
                return self.j5_rat_id
            case _:
                print(f"Can't get rat id for J {f}")
                return None

    def __str__(self):
        return self.__repr__()

    def __repr__(self):
        _repr  = '<DSI CARD:\n'
        _repr += f'ID     : {self.dsi_id}\n'          
        _repr += f'J1 RAT : {self.j1_rat_id}\n'       
        _repr += f'J2 RAT : {self.j2_rat_id}\n'       
        _repr += f'J3 RAT : {self.j3_rat_id}\n'       
        _repr += f'J4 RAT : {self.j4_rat_id}\n'       
        _repr += f'J5 RAT : {self.j5_rat_id}\n'       
        return _repr

class RAT(models.Model):
    rat_id                    = models.PositiveSmallIntegerField(unique=True, primary_key=True)
    pb_id                     = models.PositiveSmallIntegerField()
    rb1_id                    = models.PositiveSmallIntegerField()
    rb2_id                    = models.PositiveSmallIntegerField()
    ltb_id                    = models.PositiveSmallIntegerField()
    ltb_harting_cable_length  = models.PositiveSmallIntegerField(help_text="Length of the Harting cable in feet")

    def fill_from_spreadsheet(self, data):
        self.rat_id                    = int(data['RAT number'])
        self.pb_id                     = int(data['PB'])
        self.rb1_id                    = int(data['RB1'])
        self.rb2_id                    = int(data['RB2'])
        #self.ltb_id                    = int(data['LTB'])
        # The actual ltb_id is not used!
        # This is a weird quirk, see my conversation with Sydney:
        # Achim: No worries! Thanks for you answer (also I was busy with meetings this morning). That sounds good, however, I am still confused, sorry for being a bit slow with this. I was wondering about LTB 8. The reason why I am asking is that in the RAT table, it says RAT 8 has RB1 and 11 (as you also said) but then it says LTB 1, but in the "Paddle End Master Spreadsheet" column "I" it says "RAT number = LTB number", so should the LTB in RAT 8 be 1 or 8?  The reason why I need to know is because I get the trigger mask from the MTB, but it is a descriptor of LTBs which have triggered, so I need to make the connection LTB id - >LTB channel -> RB id -> RB channel. I do have this relation implemented, but something is not consistent, so I am currently hunting this bug, so I am just double checking everything. Thanks a lot for your help!
        #  4:02 PM
        # Sydney:  oh, I see where your confusion is!in the RAT table, I list all the board ID numbers for the PB, RBs, and LTB inside each RAT. for the RBs and PBs, these board IDs are significant (for the RBs, it distinguishes which ip address the data will come out on. for the PBs, we will implement unique lookup tables that associate ADC values with actual measured voltages).However, for our LTBs, the board ID number listed in the RAT table is just so that I can keep track of each of the 22 LTBs that we have. the LTB board ID doesn't matter at all for data taking and control; each board behaves exactly the same way, uses the same firmware, is controlled identically.what does matter is the location of the LTB, and in particular which RAT the LTB is inside of (because this determines which paddles are connected and triggering). that is why in the paddle master spreadsheet, the LTB channel is just listed with the associated RAT.so, in conclusion, you should be able to completely ignore the LTB column of the RAT table
        self.ltb_id                    = self.rat_id
        #print(data)
        #self.ltb_harting_cable_length  = int(data['LTB Harting cable length'].split(' ')[0])
        self.ltb_harting_cable_length  = int(data['LTB Harting cable length'])

    def __str__(self):
        return self.__repr__()

    def __repr__(self):
        _repr = '<RAT:\n'
        _repr += f'ID               : {self.rat_id}\n'                   
        _repr += f'PB               : {self.pb_id}\n'                    
        _repr += f'RB1              : {self.rb1_id}\n'                   
        _repr += f'RB2              : {self.rb2_id}\n'                   
        _repr += f'LTB              : {self.ltb_id}\n'                   
        _repr += f'HRT CBL LEN [FT] : {self.ltb_harting_cable_length}>' 
        return _repr

class Run(models.Model):
    """
    Meta information which defines a data run
    """

    run_id         = models.PositiveBigIntegerField(primary_key=True)
    #shifter        =
    #                 help_text="Name of the responsible person for data taking"
    #comment        = 
    #                  help_text="Purpose of this run"
    #timestamp      = 
    #                  help_text="UTC timestamp of run start"
    #trigger_config = 
    #                  help_text="Trigger configuration of run start"
    #prescale       =
    #                  help_text="Applied prescale factor for trigger config"
    #configuration  = 
    #                  help_text="Serialized .toml" file for run configuration"

class RB(models.Model):
    """
    Representation of a readoutboard

    """
    
    rb_id            = models.PositiveSmallIntegerField(unique=True, primary_key=True)
    dna              = models.PositiveBigIntegerField(unique=True, null=True)
    ch1_paddle       = models.ForeignKey(PaddleEnd, models.SET_NULL, blank=True, null=True,related_name='+' )
    ch2_paddle       = models.ForeignKey(PaddleEnd, models.SET_NULL, blank=True, null=True,related_name='+' )
    ch3_paddle       = models.ForeignKey(PaddleEnd, models.SET_NULL, blank=True, null=True,related_name='+' )
    ch4_paddle       = models.ForeignKey(PaddleEnd, models.SET_NULL, blank=True, null=True,related_name='+' )
    ch5_paddle       = models.ForeignKey(PaddleEnd, models.SET_NULL, blank=True, null=True,related_name='+' )
    ch6_paddle       = models.ForeignKey(PaddleEnd, models.SET_NULL, blank=True, null=True,related_name='+' )
    ch7_paddle       = models.ForeignKey(PaddleEnd, models.SET_NULL, blank=True, null=True,related_name='+' )
    ch8_paddle       = models.ForeignKey(PaddleEnd, models.SET_NULL, blank=True, null=True,related_name='+' )

    def get_pid_for_channel(self, ch):
        return get_channel(ch).paddle_id

    def get_ploc_for_channel(self, ch):
        panel = get_channel(ch).panel_id
        panel = Panel.objects.filter(id=panel)[0]
        print(panel)

    def set_channel(self, ch, pend):
        match ch:
            case 1:
                self.ch1_paddle = pend
            case 2:
                self.ch2_paddle = pend
            case 3:
                self.ch3_paddle = pend
            case 4:
                self.ch4_paddle = pend
            case 5:
                self.ch5_paddle = pend
            case 6:
                self.ch6_paddle = pend
            case 7:
                self.ch7_paddle = pend
            case 8:
                self.ch8_paddle = pend
            case _:
                raise ValueError(f"Can't set paddle for channel {ch}")

    def get_channel(self, ch):
        match ch:
            case 1:
                return self.ch1_paddle
            case 2:
                return self.ch2_paddle
            case 3:
                return self.ch3_paddle
            case 4:
                return self.ch4_paddle
            case 5:
                return self.ch5_paddle
            case 6:
                return self.ch6_paddle
            case 7:
                return self.ch7_paddle
            case 8:
                return self.ch8_paddle
            case _:
                raise ValueError(f"Don't have paddle for channel {ch}")

    def guess_address(self):
        ip_address = "10.0.1.1" + str(self.rb_id).zfill(2)
        return ip_address 

    def __str__(self):
        return self.__repr__()

    def __repr__(self):
        _repr  = '<ReadoutBoard:'
        _repr += f'\n  Board id : {self.rb_id}'            
        _repr += f'\n  dna      : {self.dna}'          
        _repr += f'\n  **Connected paddle ends**'
        _repr += f'\n  Ch0(1)   : {self.ch1_paddle}'         
        _repr += f'\n  Ch1(2)   : {self.ch2_paddle}'         
        _repr += f'\n  Ch2(3)   : {self.ch3_paddle}'         
        _repr += f'\n  Ch3(4)   : {self.ch4_paddle}'         
        _repr += f'\n  Ch4(5)   : {self.ch5_paddle}'         
        _repr += f'\n  Ch5(6)   : {self.ch6_paddle}'         
        _repr += f'\n  Ch6(7)   : {self.ch7_paddle}'         
        _repr += f'\n  Ch7(8)   : {self.ch8_paddle}>'         
        return _repr

####################################################

def get_dsi_j_for_ltb(ltb, rats, dsi_cards, dry_run = False):
    try:
        rat = [k for k in rats if k.ltb_id == ltb.ltb_id][0]
    except Exception as e:
        print (f'Can not get rat for ltb with id {ltb.ltb_id}')
        return
    dsi, j = 0,0
    for k in dsi_cards:
        if k.j1_rat_id == rat.rat_id:
            dsi = k.dsi_id
            j   = 1
            break

        if k.j2_rat_id == rat.rat_id:
            dsi = k.dsi_id
            j   = 2
            break
        
        if k.j3_rat_id == rat.rat_id:
            dsi = k.dsi_id
            j   = 3
            break
        
        if k.j4_rat_id == rat.rat_id:
            dsi = k.dsi_id
            j   = 4
            break
        
        if k.j5_rat_id == rat.rat_id:
            dsi = k.dsi_id
            j   = 5
            break

    ltb.ltb_dsi = dsi
    ltb.ltb_j   = j
    print(f" Will write dsi {dsi} and j {j}")
    if not dry_run:
        ltb.save()

