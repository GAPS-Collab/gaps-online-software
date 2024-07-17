"""
Building blocks of the TOF
"""

from django.db import models

class RAT(models.Model):
    """
    RAT in this context means Readout And Trigger (box). This is a unit component of the TOF
    and contains a LocalTriggerBoard, two ReadoutBoards and a PowerBOard

    The LTB id here is the RAT id, here is some conversation about that
        ' Achim: No worries! Thanks for you answer (also I was busy with meetings this morning). That sounds good, however, I am still confused, sorry for being a bit slow with this. I was wondering about LTB 8. The reason why I am asking is that in the RAT table, it says RAT 8 has RB1 and 11 (as you also said) but then it says LTB 1, but in the "Paddle End Master Spreadsheet" column "I" it says "RAT number = LTB number", so should the LTB in RAT 8 be 1 or 8?  The reason why I need to know is because I get the trigger mask from the MTB, but it is a descriptor of LTBs which have triggered, so I need to make the connection LTB id - >LTB channel -> RB id -> RB channel. I do have this relation implemented, but something is not consistent, so I am currently hunting this bug, so I am just double checking everything. Thanks a lot for your help!
        '  4:02 PM
        ' Sydney:  oh, I see where your confusion is!in the RAT table, I list all the board ID numbers for the PB, RBs, and LTB inside each RAT. for the RBs and PBs, these board IDs are significant (for the RBs, it distinguishes which ip address the data will come out on. for the PBs, we will implement unique lookup tables that associate ADC values with actual measured voltages).However, for our LTBs, the board ID number listed in the RAT table is just so that I can keep track of each of the 22 LTBs that we have. the LTB board ID doesn't matter at all for data taking and control; each board behaves exactly the same way, uses the same firmware, is controlled identically.what does matter is the location of the LTB, and in particular which RAT the LTB is inside of (because this determines which paddles are connected and triggering). that is why in the paddle master spreadsheet, the LTB channel is just listed with the associated RAT.so, in conclusion, you should be able to completely ignore the LTB column of the RAT table
    """
    rat_id                    = models.PositiveSmallIntegerField(unique=True, primary_key=True)
    pb_id                     = models.PositiveSmallIntegerField()
    # rb1 will control the LTB
    rb1_id                    = models.PositiveSmallIntegerField()
    # rb2 will control the Preamps/PB
    rb2_id                    = models.PositiveSmallIntegerField()
    ltb_id                    = models.PositiveSmallIntegerField()
    ltb_harting_cable_length  = models.PositiveSmallIntegerField(help_text="Length of the Harting cable in feet")

    def __str__(self):
        return self.__repr__()

    def __repr__(self):
        _repr = '<RAT:'
        _repr += f'\n  ID                : {self.rat_id}'                   
        _repr += f'\n  PB                : {self.pb_id} '                    
        _repr += f'\n  RB1               : {self.rb1_id}'                   
        _repr += f'\n  RB2               : {self.rb2_id}'                   
        _repr += f'\n  LTB               : {self.ltb_id}'                   
        _repr += f'\n  H. cable len [cm] : {self.ltb_harting_cable_length}>' 
        return _repr

##########################################################################

class DSICard(models.Model):
    """
    A DSI card which is plugged into one of five slots on the MTB
    The DSI card provides the connection to RBs and LTBs and has 
    a subdivision, which is called 'j'
    """
    dsi_id          = models.PositiveSmallIntegerField(unique=True, primary_key=True)
    j1_rat_id       = models.PositiveSmallIntegerField(null=True)
    j2_rat_id       = models.PositiveSmallIntegerField(null=True)
    j3_rat_id       = models.PositiveSmallIntegerField(null=True)
    j4_rat_id       = models.PositiveSmallIntegerField(null=True)
    j5_rat_id       = models.PositiveSmallIntegerField(null=True)
 
    def has_rat(self, rat_id : int) -> bool:
        """
        True if this RAT box is plugged in to any of the j 
        connectors on this specific DSI card
        """
        return (self.j1_rat_id == rat_id)\
            or (self.j2_rat_id == rat_id)\
            or (self.j3_rat_id == rat_id)\
            or (self.j4_rat_id == rat_id)\
            or (self.j5_rat_id == rat_id)

    def get_j(self, rat_id : int) -> int:
        """
        Get the j connetor for this specific RAT
        Raises ValueError if the RAT is not connected
        """
        if not self.has_rat(rat_id):
            raise ValueError(f"RAT {rat_id} is not connected to {self}")
        match rat_id:
            case self.j1_rat_id:
                return 1
            case self.j2_rat_id:
                return 2
            case self.j3_rat_id:
                return 3
            case self.j4_rat_id:
                return 4
            case self.j5_rat_id:
                return 5

    def __repr__(self):
        _repr  = '<DSI Card:'
        _repr += f'\n  ID     : {self.dsi_id}'          
        _repr += '\n  -- -- -- --'
        _repr += f'\n  J1 RAT : {self.j1_rat_id}'       
        _repr += f'\n  J2 RAT : {self.j2_rat_id}'       
        _repr += f'\n  J3 RAT : {self.j3_rat_id}'       
        _repr += f'\n  J4 RAT : {self.j4_rat_id}'       
        _repr += f'\n  J5 RAT : {self.j5_rat_id}>'       
        return _repr

    def __str__(self):
        return self.__repr__()

##########################################################################

class Paddle(models.Model):
    """
    A single TOF paddle with 2 ends 
    comnected
    """
    paddle_id                 = models.PositiveSmallIntegerField(
                                    unique=True,
                                    primary_key=True,
                                    help_text="Paddle identifier (1-160)")
    volume_id                 = models.PositiveBigIntegerField(
                                    default=0,
                                    null=False,
                                    unique=True,
                                    help_text="The VolumeId as used in the GAPS simulation code")
    panel_id                  = models.PositiveSmallIntegerField(
                                    null=False,
                                    default=0,
                                    help_text="The id of the panel this paddle is part of")
    # connections
    mtb_link_id               = models.PositiveSmallIntegerField(
                                    default=0,
                                    null=False,
                                    help_text="The MTB link ID (of the RB) this paddle is connected to!")
    rb_id                     = models.PositiveSmallIntegerField(
                                    default=0,
                                    null=False,
                                    help_text="The RB this Paddle is connected to!")
    rb_chA                    = models.PositiveSmallIntegerField(
                                    default=0,
                                    null=False,
                                    help_text="RB channel the paddle side A is connected to!")
    rb_chB                    = models.PositiveSmallIntegerField(
                                    default=0,
                                    null=False,
                                    help_text="RB channel the paddle side B is connected to!")
    ltb_id                     = models.PositiveSmallIntegerField(
                                    default=0,
                                    null=False,
                                    help_text="The LTB (RAT ID) this Paddle is connected to!")
    ltb_chA                    = models.PositiveSmallIntegerField(
                                    default=0,
                                    null=False,
                                    help_text="LTB channel the paddle side A is connected to!")
    ltb_chB                    = models.PositiveSmallIntegerField(
                                    default=0,
                                    null=False,
                                    help_text="LTB channel the paddle side B is connected to!")
    pb_id                      = models.PositiveSmallIntegerField(
                                     default=0,
                                     null=False,
                                     help_text="The PB ID this Paddle is connected to!")
    pb_chA                     = models.PositiveSmallIntegerField(
                                     default=0,
                                     null=False,
                                     help_text="PB channel the paddle side A is connected to!")
    pb_chB                     = models.PositiveSmallIntegerField(
                                     default=0,
                                     null=False,
                                     help_text="PB channel the paddle side B is connected to!")
    cable_len                  = models.FloatField(
                                    default=0,
                                    null=False,
                                    help_text="Signal cable length (LG or HG?) in chm")
    dsi                        = models.PositiveSmallIntegerField(
                                    default=0,
                                    null=False,
                                    help_text="The DSI card this paddle is connected to!")
    j_rb                       = models.PositiveSmallIntegerField(
                                    default=0,
                                    null=False,
                                    help_text="The j connnection this paddle's ltb is connected to!")
    j_ltb                      = models.PositiveSmallIntegerField(
                                    default=0, 
                                    null=False,
                                    help_text="The j connection this paddle's rb is connected to!")
    # coordinates/orientation
    height                     = models.FloatField(null=False,
                                                   default=0.0,
                                                   help_text="(Local) height of the paddle")
    width                      = models.FloatField(null=False,
                                                   default=0.0,
                                                   help_text="(Local) width of the paddle")
    length                     = models.FloatField(null=False,
                                                   default=0.0,
                                                   help_text="(Local) length of the paddle")
    global_pos_x_l0            = models.FloatField(
                                     null=False,
                                     default=0.0,
                                     help_text="Global X center position from simulation")
    global_pos_y_l0            = models.FloatField(
                                     null=False,
                                     default=0.0,
                                     help_text="Global Y center position from simulation")
    global_pos_z_l0            = models.FloatField(
                                     null=False,
                                     default=0.0,
                                     help_text="Global Z center position from simulation")
    global_pos_x_l0_A          = models.FloatField(
                                     null=False,
                                     default=0.0,
                                     help_text="Global X (L0) position of the A side")
    global_pos_y_l0_A          = models.FloatField(
                                     null=False,
                                     default=0.0,
                                     help_text="Global X (L0) position of the A side")
    global_pos_z_l0_A          = models.FloatField(
                                     null=False,
                                     default=0.0,
                                     help_text="Global X (L0) position of the A side")
    @property
    def lt_slot(self) -> int:
        """
        Convert DSI and J connection to the actual 
        slot they are plugged in on the MTB (0-24)
        """
        return (self.dsi-1)*5 + self.j_ltb - 1
   
    @property
    def center_pos(self) -> tuple:
        return (self.global_pos_x_l0, self.global_pos_y_l0, self.global_pos_z_l0)

    @property
    def sideA_pos(self) -> tuple:
        return (self.global_pos_x_l0_A, self.global_pos_y_l0_A, self.global_pos_z_l0_A)

    @property
    def rb_slot(self) -> int:
        """
        Convert DSI and J connection to the actual 
        slot they are plugged in on the MTB (0-24)
        """
        return (self.dsi-1)*5 + self.j_rb - 1
    
    def __repr__(self):
        _repr = '<Paddle:'
        _repr += f'\n  ** identifiers **'
        _repr += f'\n   pid                : {self.paddle_id}'     
        _repr += f'\n   vid                : {self.volume_id}'
        _repr += f'\n   panel id           : {self.panel_id}'
        _repr += f'\n  ** connedtions **'
        _repr += f'\n   DSI/J/CH (LG) [A]  : {self.dsi}  | {self.j_ltb} | {self.ltb_chA:02}'
        _repr += f'\n   DSI/J/CH (HG) [A]  : {self.dsi}  | {self.j_rb} | {self.rb_chA:02}'
        _repr += f'\n   DSI/J/CH (LG) [B]  : {self.dsi}  | {self.j_ltb} | {self.ltb_chB:02}'
        _repr += f'\n   DSI/J/CH (HG) [B]  : {self.dsi}  | {self.j_rb} | {self.rb_chB:02}'
        _repr += f'\n   RB/CH         [A]  : {self.rb_id:02} | {self.rb_chA}'
        _repr += f'\n   RB/CH         [B]  : {self.rb_id:02} | {self.rb_chB}'
        _repr += f'\n   LTB/CH        [A]  : {self.ltb_id:02} | {self.ltb_chA}'
        _repr += f'\n   LTB/CH        [B]  : {self.ltb_id:02} | {self.ltb_chB}'
        _repr += f'\n   PB/CH         [A]  : {self.pb_id:02} | {self.pb_chA}'
        _repr += f'\n   PB/CH         [B]  : {self.pb_id:02} | {self.pb_chB}'
        _repr += f'\n   MTB Link ID        : {self.mtb_link_id}'
        _repr += f'\n   cable len [cm] :'
        _repr += f'\n    \u21B3 {self.cable_len}'
        _repr += f'\n    (Harting -> RB)'
        _repr += f'\n  ** Coordinates (L0) & dimensions **'
        _repr += f'\n   length, width, height [mm]'
        _repr += f'\n    \u21B3 [{self.length:.2f}, {self.width:.2f}, {self.height:.2f}]'
        _repr += f'\n   center [mm]:'
        _repr += f'\n    \u21B3 [{self.global_pos_x_l0:.2f}, {self.global_pos_y_l0:.2f}, {self.global_pos_z_l0:.2f}]'
        _repr += f'\n   A-side [mm]:'
        _repr += f'\n    \u21B3 [{self.global_pos_x_l0_A:.2f}, {self.global_pos_y_l0_A:.2f}, {self.global_pos_z_l0_A:.2f}]>'
        return _repr

    def __str__(self):
        return self.__repr__()

##########################################################################

class Panel(models.Model):
    """ 
    A tof panel (can be subsection of a face)
    """
    panel_id                  = models.PositiveSmallIntegerField(
                                                    unique=True,
                                                    primary_key=True)
    description               = models.CharField(
                                    null=False,
                                    default="",
                                    max_length=128)
    normal_x                  = models.SmallIntegerField(
                                    null=False,
                                    default=0)
    normal_y                  = models.SmallIntegerField(
                                    null=False,
                                    default=0)
    normal_z                  = models.SmallIntegerField(
                                    null=False,
                                    default=0)
    paddle0                   = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle1                   = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle2                   = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle3                   = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle4                   = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle5                   = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle6                   = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle7                   = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle8                   = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle9                   = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle10                  = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle11                  = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )

    dw_paddle                 = models.PositiveSmallIntegerField(null=True, help_text="The distance between two paddle centers in 'width' direction, that is the second smallest dimenson of the paddle. This is basically the 'overlap'")
    dh_paddle                 = models.PositiveSmallIntegerField(null=True,help_text="The distance between two paddle centers in 'height' direction, thet is the smalles dimension of the paddle. Witout wrapping, this would be the paddle height")

    @property
    def paddles(self) -> list:
        paddles = []
        if self.paddle0 is not None:
            paddles.append(self.paddle0)
        if self.paddle1 is not None:
            paddles.append(self.paddle1)
        if self.paddle2 is not None:
            paddles.append(self.paddle2)
        if self.paddle3 is not None:
            paddles.append(self.paddle3)
        if self.paddle4 is not None:
            paddles.append(self.paddle4)
        if self.paddle5 is not None:
            paddles.append(self.paddle5)
        if self.paddle6 is not None:
            paddles.append(self.paddle6)
        if self.paddle7 is not None:
            paddles.append(self.paddle7)
        if self.paddle8 is not None:
            paddles.append(self.paddle8)
        if self.paddle9 is not None:
            paddles.append(self.paddle9)
        if self.paddle10 is not None:
            paddles.append(self.paddle10)
        if self.paddle11 is not None:
            paddles.append(self.paddle11)
        return paddles


    @property
    def pids(self) -> list:
        """
        Paddle ids in this panel
        """
        pids = [k.paddle_id for k in self.paddles]
        return pids

    @property
    def n_paddles(self) -> int:
        return len(self.paddles)

    @property
    def ltbs(self) -> list:
        ltbs = list(set([k.ltb_id for k in self.paddles]))
        return ltbs

    @property 
    def rbs(self) -> list:
        rbs = list(set([k.rb_id for k in self.paddles]))
        return rbs

    @property
    def dsis(self) -> list:
        dsi = list(set([k.dsi for k in self.paddles]))
        return dsi

    @property
    def j_ltbs(self) -> list:
        js = list(set([k.j_ltb for k in self.paddles]))
        return js
    
    @property
    def j_rbs(self) -> list:
        js = list(set([k.j_rb for k in self.paddles]))
        return js

    def __repr__(self):
        _repr = '<Panel:'
        _repr += f'\n  id    : {self.panel_id}'
        _repr += f'\n  descr : {self.description}'
        _repr += '\n  orientation:'
        _repr += f'\n   [{self.normal_x},{self.normal_y},{self.normal_z}]'
        _repr += f'\n  paddle list ({self.n_paddles} paddles)'
        _repr += f'\n   {self.paddle0}'
        if self.paddle1 is not None:
            _repr += f'\n   {self.paddle1}'
        if self.paddle2 is not None:
            _repr += f'\n   {self.paddle2}'
        if self.paddle3 is not None:
            _repr += f'\n   {self.paddle3}'
        if self.paddle4 is not None:
            _repr += f'\n   {self.paddle4}'
        if self.paddle5 is not None:
            _repr += f'\n   {self.paddle5}'
        if self.paddle6 is not None:
            _repr += f'\n   {self.paddle6}'
        if self.paddle7 is not None:
            _repr += f'\n   {self.paddle7}'
        if self.paddle8 is not None:
            _repr += f'\n   {self.paddle8}'
        if self.paddle9 is not None:
            _repr += f'\n   {self.paddle9}'
        if self.paddle10 is not None:
            _repr += f'\n   {self.paddle10}'
        if self.paddle11 is not None:
            _repr += f'\n   {self.paddle11}'
        _repr += '>'
        return _repr

    def __str__(self):
        return self.__repr__()

##########################################################################

class LocalTriggerBoard(models.Model):
    """
    Representation of a local trigger board.

    The individual LTB channels do not map directly to PaddleEnds. Rather two of them
    map to a paddle and then the whole paddle should get read out.
    To be more specific about this. The LTB has 16 channels, but we treat them as 8.
    Each 2 LTB channels get "married" internally in the board and will then continue
    on as 1 LTB channel, visible to the outside. The information about which end of 
    the Paddle crossed which threshhold is lost.
    How it works is that the two channels will be combined by the trigger logic:
    - There are 4 states (2 bits)
      - 0 - no hit
      - 1 - Hit
      - 2 - Beta
      - 3 - Veto
    
    Each defining an individual threshold. If that is crossed, the whole paddle
    (ends A+B) will be read out by the ReadoutBoard

    The LTB channels here are labeled 1-8. This is as it is in the TOF spreadsheet.
    Also dsi is labeled as in the spreadsheet and will start from one.

    It is NOT clear from this which ch on the rb is connected to which side, for that
    the paddle/RB tables need to be consulted.
    Again: rb_ch0 does NOT necessarily correspond to the A side!
    """
    board_id    = models.PositiveSmallIntegerField(primary_key=True, unique=True, 
                                                   help_text="The RAT id of the ltb")
    dsi         = models.PositiveSmallIntegerField(null=True, default=None, help_text="DSI connector number on the MTB")
    j           = models.PositiveSmallIntegerField(null=True, default=None, help_text="J connector number on the MTB")
    rat         = models.PositiveSmallIntegerField(null=True, default=None, help_text="RAT box the LTB is mounted in")
    ltb_id      = models.PositiveSmallIntegerField(null=True, default=None, help_text="The actual LTB id. This field is currently not used, forall major purposes we use the RAT ID as ltb id") 
    cable_len   = models.FloatField(default=float(0), help_text="The length of the Harting cable this LTB is connected to the MTB")
    
    paddle1     = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle2     = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle3     = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle4     = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle5     = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle6     = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle7     = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle8     = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    
    @property
    def paddles(self) -> list:
        """
        Get the paddles for this LTB in ascending 
        channel order
        """
        paddles = [self.paddle1, self.paddle2, self.paddle3, self.paddle4,
                   self.paddle5, self.paddle6, self.paddle7, self.paddle8]
        paddles = sorted(paddles, key=lambda x : x.ltb_chA)
        return paddles

    @property
    def rb_channels(self) -> list:
        """
        A sorted list of LTB channels 1-8 and their corresponding rb ids
        and channels.

        # Returns:
          [RB ID, (RB ch0, RB ch1)] where RB ch0/1 are the channels on 
          the RB which are connected to the same paddle
        """
        paddles = self.paddles 
        rb_channels = [(pdl.rb_id, (pdl.rb_chA, pdl.rb_chB)) for pdl in paddles]
        return rb_channels

    @property
    def rbs(self) -> list:
        """
        Return a list of all connected ReadoutBoards
        """
        all_boards = [self.paddle1.rb_id, self.paddle2.rb_id, self.paddle3.rb_id, self.paddle4.rb_id,\
                      self.paddle5.rb_id, self.paddle6.rb_id, self.paddle7.rb_id, self.paddle8.rb_id]
        all_boards = list(set(all_boards))
        return all_boards

    @property
    def pids(self) -> list:
        """
        Return a list of paddle ids connected to this LTB
        """
        all_pids = [self.paddle1.paddle_id, self.paddle2.paddle_id, self.paddle3.paddle_id, self.paddle4.paddle_id,\
                    self.paddle5.paddle_id, self.paddle6.paddle_id, self.paddle7.paddle_id, self.paddle8.paddle_id]
        return all_pids

    def has_pid(self, pid :int) -> bool:
        """
        Is this paddle id connected to the LTB?
        """
        pids = self.get_pids()
        return pid in pids

    def has_rb(self, rb : int) -> bool:
        """
        Is this Readoutboard connected to any of the paddles
        the LTB is connected to?
        """
        rbs = self.get_rbs()
        return rb in rbs

    def connected(self) -> bool:
        """
        Does this LTB exist? Or is the dsi/j slot it 
        would correspond to, empty?
        """
        return (self.dsi is not None) and (self.j is not None);

    @property
    def mtb_slot(self) -> int:
        """
        Dsi and j are mixed in the typically MTB 
        applications.
        This returns dsi - 1 + j - 1, since on the 
        MTB dsi and j start with 1
        """
        return (self.dsi - 1)*5 + (self.j - 1)
    
    @property
    def panels(self) -> list:
        """
        Return all panels this LTB is connected to
        """
        panels = [self.paddle1.panel, self.paddle2.panel,\
                  self.paddle3.panel, self.paddle4.panel,\
                  self.paddle5.panel, self.paddle5.panel,\
                  self.paddle7.panel, self.paddle8.panel]
        return list(set(panels))

    def get_paddle_for_channel(self):
        """
        Get the paddle for the combined channel
        """
        pass

    def __repr__(self) -> str:
        if not self.connected():
            _repr = '<LocalTriggerBoard: ID {}  - UNCONNECTED>'
            
        else:
            _repr = '<LocalTriggerBoard:'
            _repr += f'\n  LTB ID  : {self.board_id}'             
            _repr += f'\n  DSI/J   : {self.dsi}/{self.j}'     
            _repr += f'\n  RAT ID  : {self.rat}'
            _repr +=  '\n  H. cable len (MTB connection):'
            _repr += f'\n    ->      {self.cable_len}'
            _repr +=  '\n  -- -- -- -- -- -- -- -- -- -- -- -- -- --'
            _repr +=  '\n  LTB Ch -> RB Id, RB chn, Pdl ID, Pan ID:' 
            _repr += f'\n  1: {self.paddle1.ltb_chA:02},{self.paddle1.ltb_chB:02}  -> {self.paddle1.rb_id:02}   |   {self.paddle1.rb_chA},{self.paddle1.rb_chB} |  {self.paddle1.paddle_id:03}  | {self.paddle1.panel_id:02}' 
            _repr += f'\n  2: {self.paddle2.ltb_chA:02},{self.paddle2.ltb_chB:02}  -> {self.paddle2.rb_id:02}   |   {self.paddle2.rb_chA},{self.paddle2.rb_chB} |  {self.paddle2.paddle_id:03}  | {self.paddle2.panel_id:02}' 
            _repr += f'\n  3: {self.paddle3.ltb_chA:02},{self.paddle3.ltb_chB:02}  -> {self.paddle3.rb_id:02}   |   {self.paddle3.rb_chA},{self.paddle3.rb_chB} |  {self.paddle3.paddle_id:03}  | {self.paddle3.panel_id:02}' 
            _repr += f'\n  4: {self.paddle4.ltb_chA:02},{self.paddle4.ltb_chB:02}  -> {self.paddle4.rb_id:02}   |   {self.paddle4.rb_chA},{self.paddle4.rb_chB} |  {self.paddle4.paddle_id:03}  | {self.paddle4.panel_id:02}' 
            _repr += f'\n  5: {self.paddle5.ltb_chA:02},{self.paddle5.ltb_chB:02}  -> {self.paddle5.rb_id:02}   |   {self.paddle5.rb_chA},{self.paddle5.rb_chB} |  {self.paddle5.paddle_id:03}  | {self.paddle5.panel_id:02}' 
            _repr += f'\n  6: {self.paddle6.ltb_chA:02},{self.paddle6.ltb_chB:02}  -> {self.paddle6.rb_id:02}   |   {self.paddle6.rb_chA},{self.paddle6.rb_chB} |  {self.paddle6.paddle_id:03}  | {self.paddle6.panel_id:02}' 
            _repr += f'\n  7: {self.paddle7.ltb_chA:02},{self.paddle7.ltb_chB:02}  -> {self.paddle7.rb_id:02}   |   {self.paddle7.rb_chA},{self.paddle7.rb_chB} |  {self.paddle7.paddle_id:03}  | {self.paddle7.panel_id:02}' 
            _repr += f'\n  8: {self.paddle8.ltb_chA:02},{self.paddle8.ltb_chB:02}  -> {self.paddle8.rb_id:02}   |   {self.paddle8.rb_chA},{self.paddle8.rb_chB} |  {self.paddle8.paddle_id:03}  | {self.paddle8.panel_id:02}>' 
        return _repr
    
    def __str__(self) -> str:
        return self.__repr__()

##########################################################################

class ReadoutBoard(models.Model):
    """
    A Readoutboard with the connected paddles   
    """
    rb_id           = models.PositiveSmallIntegerField(
                         unique=True, 
                         primary_key=True,
                         help_text="The board id fo the readoutboard. Unique identifier")
    dsi             = models.PositiveSmallIntegerField(
                         default=0,
                         null=False,
                         help_text="The DSI card this paddle is connected to!")
    j               = models.PositiveSmallIntegerField(
                         default=0,
                         null=False,
                         help_text="The j connnection this paddle's ltb is connected to!")
    mtb_link_id     = models.PositiveSmallIntegerField(
                          default=0,
                          null=False,
                          help_text="The MTB link ID (of the RB) this paddle is connected to!")
    # paddles for the individual RB channels
    paddle12        = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle12_chA    = models.PositiveSmallIntegerField(
                          default=0,
                          null=True,
                          help_text="Channel which is connected to paddle A-side")
    paddle34        = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle34_chA    = models.PositiveSmallIntegerField(
                          default=0,
                          null=True,
                          help_text="Channel which is connected to paddle A-side")
    paddle56        = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle56_chA    = models.PositiveSmallIntegerField(
                          default=0,
                          null=True,
                          help_text="Channel which is connected to paddle A-side")
    paddle78        = models.ForeignKey(Paddle, models.SET_NULL, blank=True, null=True,related_name='+' )
    paddle78_chA    = models.PositiveSmallIntegerField(
                          default=0,
                          null=True,
                          help_text="Channel which is connected to paddle A-side")

    def guess_address(self):
        """
        Returns the ip address following a convention
        """
        ip_address = "10.0.1.1" + str(self.rb_id).zfill(2)
        return ip_address 

    @property
    def paddles(self) -> list:
        paddles = [self.paddle12, self.paddle34, self.paddle56, self.paddle78]
        paddles = [k for k in paddles if k is not None]
        return paddles

    @property
    def pids(self) -> list:
        return [k.paddle_id for k in self.paddles]

    @property
    def ltbs(self) -> list:
        return list(set([k.ltb_id for k in self.paddles]))

    @property
    def panels(self) -> list:
        return list(set([k.panel_id for k in self.paddles]))

    @property
    def dsis(self) -> list:
        return list(set([k.dsi for k in self.paddles]))

    @property
    def j_ltbs(self) -> list:
        js = list(set([k.j_ltb for k in self.paddles]))
        return js
    
    @property
    def j_rbs(self) -> list:
        js = list(set([k.j_rb for k in self.paddles]))
        return js

    def get_paddle(self, channel):
        """
        Returns the paddle connected to channel
        Channel runs from 1-8 (incl)
        """
        match channel:
            case 1:
                return self.paddle12
            case 2:
                return self.paddle12
            case 3:
                return self.paddle34
            case 4:
                return self.paddle34
            case 5:
                return self.paddle56
            case 6:
                return self.paddle56
            case 7:
                return self.paddle78
            case 8:
                return self.paddle78

    def __repr__(self):
        _repr  = '<ReadoutBoard:'
        _repr += f'\n  Board id    : {self.rb_id}'            
        _repr += f'\n  MTB Link ID : {self.mtb_link_id}'
        _repr += f'\n  DSI/J       : {self.dsi}/{self.j}'
        if self.paddles:
            _repr += f'\n **Connected paddles**'
        if self.paddle12 is not None:
            _repr += f'\n  Ch0/1(1/2)  : {self.paddle12}'         
        if self.paddle34 is not None:
            _repr += f'\n  Ch1/2(2/3)  : {self.paddle34}'         
        if self.paddle56 is not None:
            _repr += f'\n  Ch2/3(3/4)  : {self.paddle56}'         
        if self.paddle78 is not None:
            _repr += f'\n  Ch3/4(4/5)  : {self.paddle78}'         
        _repr += '>'
        return _repr

    def __str__(self):
        return self.__repr__()

##########################################################################

class MTBChannel(models.Model):
    """
    Summary of DSI/J/LTBCH (0-319)
    This is not "official" but provides a way of indexing all
    the individual channels
    """
    mtb_ch      = models.PositiveBigIntegerField(primary_key=True, unique=True)
    dsi         = models.PositiveSmallIntegerField(null=True)
    j           = models.PositiveSmallIntegerField(null=True)
    ltb_id      = models.PositiveSmallIntegerField(null=True)
    ltb_ch      = models.PositiveSmallIntegerField(null=True)
    rb_id       = models.PositiveSmallIntegerField(null=True)
    rb_ch       = models.PositiveSmallIntegerField(null=True)
    mtb_link_id = models.PositiveSmallIntegerField(null=True)
    paddle_id   = models.PositiveSmallIntegerField(null=True)
    paddle_isA  = models.BooleanField(null=True) 
    hg_ch       = models.PositiveSmallIntegerField(unique=True, null=True)
    lg_ch       = models.PositiveSmallIntegerField(unique=True, null=True)

    def set_lg_channel(self):
        if self.dsi is None or self.j is None or self.ltb_ch is None:
            self.lg_ch = None
            return
        self.lg_ch = ((self.dsi - 1)*80) + ((self.j - 1)*16) + (self.ltb_ch-1)
    
    def set_hg_channel(self):
        if self. rb_id is None or self.rb_ch is None:
            self.hg_ch = None
            return
        self.hg_ch = ((self.rb_id - 1)*9) + (self.rb_ch - 1)

    def __repr__(self):
        _repr  = '<MTBChannel:'
        _repr += f'\n  Channel ID : {self.mtb_ch}'
        _repr += f'\n  DSI/J/     : {self.dsi}/{self.j}' 
        _repr += '\n  LTB ID/CH => RB ID/CH'
        _repr += f'\n   |-> {self.ltb_id}/{self.ltb_ch} => {self.rb_id}/{self.rb_ch}'
        _repr += f'\n  MTB Link ID [RB] : {self.mtb_link_id}'
        _repr += '\n  LG CH => HG CH'
        _repr += f'\n   |-> {self.lg_ch} => {self.hg_ch}'
        _repr += f'\n  Paddle Id: {self.paddle_id}'
        pend = 'A'
        if not self.paddle_isA:
            pend = 'B'
        _repr += f'\n  Paddle End: {pend}>'
        return _repr
    
    def __str__(self):
        return self.__repr__()

##########################################################################


#class RBCalibration(models.Model):
#    """
#    Readoutboard timing + voltage calibration
#    """
#    pass




class Run(models.Model):
    """
    Meta information which defines a data run
    """

    run_id         = models.PositiveBigIntegerField(primary_key=True,
                                                    help_text="Uniquely assigned run id by the TOF CPU")
    # default is a 24 hour run
    runtime_secs   = models.PositiveBigIntegerField(null=True, default=86400,
                                                    help_text="Duration of the run in seconds")
    # do a calibration before run
    calib_before   = models.BooleanField(null=True, default=True,
                                         help_text="Run calibration right before run start")
    shifter        = models.SmallIntegerField(null=True, default=0,
                                              help_text="Shifter ID")
    run_type       = models.SmallIntegerField(null=True, default=0,
                                              help_text="Type of run, like PHYSICS or DEBUG")
    run_path       = models.CharField(max_length=1024,
                                      null=True,
                                      default="",
                                      help_text="Data location on TOF computer")
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



######################################

# Specters of the past...



#class RB(models.Model):
#    """
#    Representation of a readoutboard
#
#    """
#    
#    rb_id            = models.PositiveSmallIntegerField(unique=True, primary_key=True)
#    dna              = models.PositiveBigIntegerField(unique=True, null=True)
#    ch1_paddle       = models.ForeignKey(PaddleEnd, models.SET_NULL, blank=True, null=True,related_name='+' )
#    ch2_paddle       = models.ForeignKey(PaddleEnd, models.SET_NULL, blank=True, null=True,related_name='+' )
#    ch3_paddle       = models.ForeignKey(PaddleEnd, models.SET_NULL, blank=True, null=True,related_name='+' )
#    ch4_paddle       = models.ForeignKey(PaddleEnd, models.SET_NULL, blank=True, null=True,related_name='+' )
#    ch5_paddle       = models.ForeignKey(PaddleEnd, models.SET_NULL, blank=True, null=True,related_name='+' )
#    ch6_paddle       = models.ForeignKey(PaddleEnd, models.SET_NULL, blank=True, null=True,related_name='+' )
#    ch7_paddle       = models.ForeignKey(PaddleEnd, models.SET_NULL, blank=True, null=True,related_name='+' )
#    ch8_paddle       = models.ForeignKey(PaddleEnd, models.SET_NULL, blank=True, null=True,related_name='+' )
#    mtb_link_id      = models.PositiveSmallIntegerField(unique=True, null=True)
#
#    def get_pid_for_channel(self, ch):
#        return get_channel(ch).paddle_id
#
#    def get_ploc_for_channel(self, ch):
#        panel = get_channel(ch).panel_id
#        panel = Panel.objects.filter(id=panel)[0]
#        print(panel)
#
#    def set_channel(self, ch, pend):
#        match ch:
#            case 1:
#                self.ch1_paddle = pend
#            case 2:
#                self.ch2_paddle = pend
#            case 3:
#                self.ch3_paddle = pend
#            case 4:
#                self.ch4_paddle = pend
#            case 5:
#                self.ch5_paddle = pend
#            case 6:
#                self.ch6_paddle = pend
#            case 7:
#                self.ch7_paddle = pend
#            case 8:
#                self.ch8_paddle = pend
#            case _:
#                raise ValueError(f"Can't set paddle for channel {ch}")
#
#    def get_channel(self, ch):
#        match ch:
#            case 1:
#                return self.ch1_paddle
#            case 2:
#                return self.ch2_paddle
#            case 3:
#                return self.ch3_paddle
#            case 4:
#                return self.ch4_paddle
#            case 5:
#                return self.ch5_paddle
#            case 6:
#                return self.ch6_paddle
#            case 7:
#                return self.ch7_paddle
#            case 8:
#                return self.ch8_paddle
#            case _:
#                raise ValueError(f"Don't have paddle for channel {ch}")
#
#
#
#
#####################################################
#
#def get_dsi_j_for_ltb(ltb, rats, dsi_cards, dry_run = False):
#    try:
#        rat = [k for k in rats if k.ltb_id == ltb.ltb_id][0]
#    except Exception as e:
#        print (f'Can not get rat for ltb with id {ltb.ltb_id}')
#        return
#    dsi, j = 0,0
#    for k in dsi_cards:
#        if k.j1_rat_id == rat.rat_id:
#            dsi = k.dsi_id
#            j   = 1
#            break
#
#        if k.j2_rat_id == rat.rat_id:
#            dsi = k.dsi_id
#            j   = 2
#            break
#        
#        if k.j3_rat_id == rat.rat_id:
#            dsi = k.dsi_id
#            j   = 3
#            break
#        
#        if k.j4_rat_id == rat.rat_id:
#            dsi = k.dsi_id
#            j   = 4
#            break
#        
#        if k.j5_rat_id == rat.rat_id:
#            dsi = k.dsi_id
#            j   = 5
#            break
#
#    ltb.ltb_dsi = dsi
#    ltb.ltb_j   = j
#    print(f" Will write dsi {dsi} and j {j}")
#    if not dry_run:
#        ltb.save()
#class PaddleEnd(models.Model):
#    """
#    One end of a paddle with SiPM array
#    """
#    PADDLE_END    = [('A', 'A'), ('B', 'B')]
#    paddle_end_id = models.PositiveSmallIntegerField(
#                        primary_key=True,
#                        unique=True,
#                        help_text="PaddleID + 1000 for A and PaddleID + 2000 for B")
#    paddle_id     = models.PositiveSmallIntegerField()
#
#    end           = models.CharField(max_length=1, choices=PADDLE_END)
#    end_location  = models.CharField(max_length=2,\
#                                    help_text="Location of the paddle end relative to the paddle center")
#    panel_id      = models.PositiveSmallIntegerField()
#    pos_in_panel  = models.CharField(max_length=4,\
#                                     help_text="Identifier in global coordinates about the location in the panel",\
#                                     null=True,\
#                                     default="")
#    cable_length  = models.FloatField(help_text="Cable length in cm")
#    rat           = models.PositiveSmallIntegerField()
#    ltb_id        = models.PositiveSmallIntegerField()
#    rb_id         = models.PositiveSmallIntegerField()
#    pb_id         = models.PositiveSmallIntegerField()
#    ltb_ch        = models.PositiveSmallIntegerField()
#    pb_ch         = models.PositiveSmallIntegerField()
#    #FIXME - is this starting from 0 or 1?
#    rb_ch         = models.PositiveSmallIntegerField()
#    dsi           = models.PositiveSmallIntegerField()
#    rb_harting_j  = models.PositiveSmallIntegerField()
#    ltb_harting_j = models.PositiveSmallIntegerField()
#    mtb_link_id   = models.PositiveSmallIntegerField(unique=False)
#
#    def setup_unique_paddle_end_id(self):
#        """
#        Introduce a uuid. We have 160 paddles with 2 ends. Make the uuid the following
#        paddle_end_id(end :<A || B>): return 1000 if end == A else 2000
#        uuid = paddle_end_id(end) + paddle_id (so the paddle id is preceeded by 1 or 2 and the 
#        2nd and 3rd digits are filled up with 0s if necessary
#        """
#        if self.end == 'A':
#            self.paddle_end_id = 1000 + self.paddle_id
#        elif self.end == 'B':
#            self.paddle_end_id = 2000 + self.paddle_id
#        else:
#            raise ValueError(f'PaddleEnd unique identifier can not be created for paddle end {self.end} with paddle id {self.paddle_id}')
#
#    def fill_from_spreadsheet(self, data):
#        self.paddle_id     = int(data['Paddle Number']) 
#        self.end           = data['Paddle End (A/B)'] 
#        self.end_location  = data['Paddle End Location']           
#        print ("-- -- keys --")
#        print (data)
#        self.mtb_link_id   = data['MTB Link ID']
#        panel_id           = str(data['Panel Number'])
#        if panel_id.startswith('E'):
#            # this are these individual edge paddles
#            # we replace them with 1000 + the number 
#            # after E-X
#            panel_id = panel_id.replace("E-X","")
#            self.panel_id = int(panel_id) + 1000
#        else:
#            self.panel_id = int(panel_id)
#        self.cable_length  = int(data['Cable length (cm)'] )
#        self.rat           = int(data['RAT Number'] )
#        ltb_info           = data['LTB Number-Channel'].split('-')
#        rb_info            = data['RB Number-Channel'].split('-')
#        pb_info            = data['PB Number-Channel'].split('-')
#        self.ltb_id        = int(ltb_info[0]) 
#        self.rb_id         = int(rb_info[0])
#        self.pb_id         = int(pb_info[0])
#        self.ltb_ch        = int(ltb_info[1])
#        self.pb_ch         = int(pb_info[1] )
#        self.rb_ch         = int(rb_info[1] )
#        
#        # in some spreadsheets, the label differs,
#        # so we are just looking for some variant
#        good = False
#        for label in 'DSI card slot', 'DSI Card Slot', 'DSI card slot ', 'DSI Card Slot ':
#            try:
#                self.dsi           = int(data[label])
#                good               = True
#                #print("Found good key!")
#                break
#            except KeyError:
#                #print(f".. can't find key {label}, trying next variant..")
#                continue
#        if not good:
#            raise ValueError("Could not get DSI assignment!")
#        rb_h_j             = data['RB Harting Connection'].replace('J','')
#        ltb_h_j            = data['LTB Harting Connection'].replace('J','')
#        #rb_h_j             = data['RB Harting Connection'].split('_')
#        #ltb_h_j            = data['LTB Harting Connection'].split('_')
#        self.rb_harting_j  = int(rb_h_j)
#        self.ltb_harting_j = int(ltb_h_j)
#
#    def __str__(self):
#        return self.__repr__()
#
#    def __repr__(self):
#        try:
#            panel  = Panel.objects.filter(panel_id=self.panel_id)[0]
#        except Exception as e:
#            panel  = 'UNKNOWN'
#        try:
#            paddle = Paddle.objects.filter(paddle_id=self.paddle_id)[0] 
#        except:
#            paddle = 'UNKNOWN'
#        _repr = '<PaddleEnd:'
#        _repr += f'\n  ** identifiers **'
#        _repr += f'\n   id             : {self.paddle_end_id}'     
#        _repr += f'\n   pid            : {self.paddle_id}'     
#        _repr += f'\n   end (A|B)      : {self.end}' 
#        _repr += f'\n   MTB Link ID    : {self.mtb_link_id}'
#        _repr += f'\n  ** connedtions **'
#        _repr += f'\n   DSI/J/CH (LG)  :  {self.dsi} | {self.ltb_harting_j} | {self.ltb_ch:02}'
#        _repr += f'\n   DSI/J/CH (HG)  :  {self.dsi} | {self.rb_harting_j} | {self.rb_ch:02}'
#        _repr += f'\n   RB/CH          : {self.rb_id:02} | {self.rb_ch:02}'
#        _repr += f'\n   PB/CH          : {self.pb_id:02} | {self.pb_ch:02}'
#        _repr += f'\n   RAT id         : {self.rat:02}'
#        _repr += f'\n   LTB id         : {self.ltb_id:02}'
#        _repr += f'\n   cable len [cm] :'
#        _repr += f'\n    \u21B3 {self.cable_length}'
#        _repr += f'\n    (Harting -> RB)'
#        _repr += f'\n  ** panel & location **'
#        _repr += f'\n   end ->         : {self.end_location}' 
#        #_repr += f'\n   panel id       : {self.panel_id}'     
#        _repr += f'\n   loc. in panel  : {self.pos_in_panel}'
#        _repr += f'\n   {panel}'
#        _repr += f'\n   {paddle}>'
#        return _repr
#
#
#class LiftofSettings(models.Model):
#    """
#    Run settings to be used with liftof-cc
#    """
#    data_dir                   : model.String,
#    calibration_dir            : model.String,
#    db_path                    : model.String,
#    runtime_sec                : model.PositiveBigIntegerField(blank=True, null=True)
#    packs_per_file             : model.PositiveBigIntegerField(blank=True, null=True)
#    fc_pub_address             : model.String,
#    fc_sub_address             : model.String,
#    mtb_address                : model.String,
#    cpu_moni_interval_sec      : model.PositiveBigIntegerField(blank=True, null=True)
#    rb_ignorelist              : model.Vec<u8>,
#    run_analysis_engine        : model.bool,
#  #mtb_settings               : MTBSettings,
#  #event_builder_settings     : TofEventBuilderSettings,
#  #analysis_engine_settings   : AnalysisEngineSettings,

##
