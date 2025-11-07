"""
Building blocks of the TOF
"""

from re import I
from django.db import models
import numpy as np
import matplotlib.pyplot as plt
import tqdm

from pathlib import Path
from matplotlib.patches import Rectangle

vtk_import_success = False
try:
    import vtk
    vtk_import_success = True
except  ImportError as e:
    print('[gaps-db] Unable to import vtk, plotting options might be limited!')

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

class TofPaddleTimingConstant(models.Model):
    """
    An unknown constant which can be added to the individual 
    paddle event times for a more precise beta calculation.
    """
    data_id              = models.AutoField(primary_key=True,
                                            help_text="Identify this specific dataset")
    paddle_id            = models.PositiveSmallIntegerField(null=False)
    volume_id            = models.PositiveBigIntegerField(
                                    default=0,
                                    null=False,
                                    unique=True,
                                    help_text="The VolumeId as used in the GAPS simulation code")
    utc_timestamp_start  = models.PositiveBigIntegerField(null=False, default=0,
                                                          help_text="UNIX Timestamp for first point in time where this constant is relevant")
    utc_timestamp_stop   = models.PositiveBigIntegerField(null=False, default=0,
                                                          help_text="UNIX Timestamp for last point in time when this constant is relevant")
    name                 = models.CharField(max_length=1024,
                                            null=True,
                                            default="",
                                            help_text="A for better indentification of the offsets")
    version              = models.PositiveIntegerField(null=True, default=0, help_text="Version identifier for Paddle timing constants")
    timing_copnstant     = models.FloatField(
                                    default=0,
                                    null=False,
                                    help_text="Actual constant in ns")

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
    # this is to cache the points for plotting
    points   = None
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
    normal_x                   = models.FloatField(null=False,
                                                   default=0.0,
                                                   help_text="Normal vector x component (SiPM pointing) in global coordinate system")
    normal_y                   = models.FloatField(null=False,
                                                   default=0.0,
                                                   help_text="Normal vector y component (SiPM pointing) in global coordinate system")
    normal_z                   = models.FloatField(null=False,
                                                   default=0.0,
                                                   help_text="Normal vector z component (SiPM pointing) in global coordinate system")
    


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
    global_pos_x_l0_B          = models.FloatField(
                                     null=False,
                                     default=0.0,
                                     help_text="Global X (L0) position of the B side")
    global_pos_y_l0_B          = models.FloatField(
                                     null=False,
                                     default=0.0,
                                     help_text="Global X (L0) position of the B side")
    global_pos_z_l0_B          = models.FloatField(
                                     null=False,
                                     default=0.0,
                                     help_text="Global X (L0) position of the B side")
    

    coax_cable_time           = models.FloatField(
                                     null=False,
                                     default=0.0,
                                     help_text="Time the signal lingers in the cox cable as calculated by Jeff")
    harting_cable_time        = models.FloatField(
                                     null=False,
                                     default=0.0,
                                     help_text="Time the signal lingers in the harting cable as calculated by Jeff")

    @property 
    def principal(self):
        """
        Return the vector along the longest axis
        """
        pr = (self.global_pos_x_l0_A - self.global_pos_x_l0,
              self.global_pos_y_l0_A - self.global_pos_y_l0,
              self.global_pos_z_l0_A - self.global_pos_z_l0)
        pr = np.array(pr)
        length    = np.sqrt((pr[0]**2 + pr[1]**2 + pr[2]**2))
        pr = pr/length
        return pr

    @property
    def normal(self):
        """
        Return the normal vector of the paddle
        """
        nrm = (self.normal_x, self.normal_y, self.normal_z)
        nrm = np.array(nrm)
        return nrm

    def _create_box(self):

        # we can kind of cheat and do a cheap transform
        # the principal is either x, y or z axis, no mix-ins
        pr   = self.principal
        norm = self.normal
        cube = vtk.vtkCubeSource()
        edgepaddle = False
        if (pr == np.array([1,0,0])).all() or (pr == np.array([-1,0,0])).all():
            cube.SetXLength(self.length)  # Width in X
            if (norm == np.array([0,1,0])).all() or (norm == np.array([0,-1,0])).all():
                cube.SetYLength(self.height)
                cube.SetZLength(self.width)
            elif (norm == np.array([0,0,1])).all() or (norm == np.array([0,0,-1])).all():
                cube.SetZLength(self.height)
                cube.SetYLength(self.width)
            else:
                raise ValueError
        elif (pr == np.array([0,1,0])).all() or (pr == np.array([0,-1,0])).all():
            cube.SetYLength(self.length)  # Width in X
            if (norm == np.array([1,0,0])).all() or (norm == np.array([-1,0,0])).all():
                cube.SetZLength(self.width)
                cube.SetXLength(self.height)
            elif (norm == np.array([0,0,1])).all() or (norm == np.array([0,0,-1])).all():
                cube.SetXLength(self.width)
                cube.SetZLength(self.height)
            else:
                raise ValueError
        elif (pr == np.array([0,0,1])).all() or (pr == np.array([0,0,-1])).all():
            cube.SetZLength(self.length)
            # set the other two and then we have to rotat by 45 deg
            cube.SetXLength(self.height)
            cube.SetYLength(self.width)
            edgepaddle = True
            #transform_filter = vtk.vtkTransformPolyDataFilter()
            #transform_filter.SetInputData(cube.GetOutput())
            #transform_filter.SetTransform(trafo)
            #transform_filter.Update()
        else:
            ValueError(f'Unexpected principal axes {pr}')
        cube.Update()
        transform = vtk.vtkTransform()
        transform.Translate(self.global_pos_x_l0, self.global_pos_y_l0, self.global_pos_z_l0)
        if edgepaddle:
            transform.RotateWXYZ(45, [0,0,1])  # angle, (x, y, z) axis to rotate around
        transform_filter = vtk.vtkTransformPolyDataFilter()
        transform_filter.SetInputData(cube.GetOutput())
        transform_filter.SetTransform(transform)
        transform_filter.Update()
        #print (cube)
        # Extract the transformed points
        points = transform_filter.GetOutput().GetPoints()
        num_points = points.GetNumberOfPoints()
        transformed_points = np.array([points.GetPoint(i) for i in range(num_points)])
        return transformed_points
        #if vtk_import_success:
        #    # Use vtkCubeSource to create a cube (or box)
        #    cube.SetXLength(self.length)  # Width in X
        #    cube.SetYLength(self.width)  # Height in Y
        #    cube.SetZLength(self.height)  # Depth in Z
        #    cube.Update()         # Update the cube geometry
        #    return cube
        #else:
        #    raise NotImplementedError('This requires vtk to be available on your system!')

    def _apply_transform(self, box, trafo):
        # Apply the transformation to the cube
        transform_filter = vtk.vtkTransformPolyDataFilter()
        transform_filter.SetInputData(box.GetOutput())
        transform_filter.SetTransform(trafo)
        transform_filter.Update()

        # Extract the transformed points
        points = transform_filter.GetOutput().GetPoints()
        num_points = points.GetNumberOfPoints()
        transformed_points = np.array([points.GetPoint(i) for i in range(num_points)])
        return transformed_points

    def _create_transform(self):
        if not vtk_import_success:
            raise NotImplementedError('This requires vtk to be available on your system!')
        else:   
            # Create a transformation object
            transform = vtk.vtkTransform()
            # Calculate the axis of rotation (cross product of x-axis and principal vector)
            principal_vector = self.principal
            normal_vector    = self.normal
            x_axis = np.array([1,0,0])
            # the normal axis is in th 
            rotation_axis = np.cross(x_axis, principal_vector)
            # Calculate the angle of rotation (dot product and arccos)
            cos_angle = np.dot(x_axis, principal_vector)
            angle = np.degrees(np.arccos(cos_angle))


            # Apply the rotation (angle in degrees)
            transform.Translate(self.global_pos_x_l0, self.global_pos_y_l0, self.global_pos_z_l0)
            transform.RotateWXYZ(angle, rotation_axis)  # angle, (x, y, z) axis to rotate around
            #transform.RotateWXYZ(ang2, rot_ax2)  # angle, (x, y, z) axis to rotate around
            #normal_vector = np.array([1,0,0])
            
            #normal_vector = normal_vector / np.sqrt(normal_vector[0]**2 + normal_vector[1]**2 + normal_vector[2]**2)
            #rot_ax2  = np.cross(np.array([0,0,1]), normal_vector)
            ##rot_ax2  = np.cross(principal_vector, normal_vector) 
            #cos_ang2 = np.dot(np.array([0,0,1]), normal_vector) 
            ##normal_vector ist x, principal y um y 90deg

            #ang2     = np.degrees(np.arccos(cos_ang2))
            #print (f'Performing normal adjustment, {ang2}, {principal_vector} {normal_vector} {rot_ax2}')
            #transform.RotateWXYZ(ang2, rot_ax2)  # angle, (x, y, z) axis to rotate around
                        
            #normal_transform = vtk.vtkTransform()
            # Get the rotation vector
            #rotvec = np.array([0,0,0])
            #for j,k in enumerate(self.normal):
            #    if self.normal[k] > 0:

            #    if j == 2:
            #        continue
            #    if k > 0:
            #        if j == 0:
            #            normal_transform.RotateWXYZ(90, [0,1,0])
            #        if j == 1:
            #            normal_transform.RotateWXYZ(90, [
            # Apply the transformation to the box
            return transform
            #transform_filter = vtk.vtkTransformPolyDataFilter()
            #transform_filter.SetTransform(transform)
            #transform_filter.SetInputData(box_polydata)
            #transform_filter.Update()

    def _create_transform2(self, shortest_axis):
        # Create a transformation object
        transform = vtk.vtkTransform()
        # Calculate the axis of rotation (cross product of x-axis and principal vector)
        principal_vector = newaxis
        normal_vector    = self.normal
        z_axis = np.array([0,0,1])
        # the normal axis is in th 
        rotation_axis = np.cross(z_axis, shortest_axis)
        # Calculate the angle of rotation (dot product and arccos)
        cos_angle = np.dot(x_axis, principal_vector)
        angle = np.degrees(np.arccos(cos_angle))


        # Apply the rotation (angle in degrees)
        transform.Translate(self.global_pos_x_l0, self.global_pos_y_l0, self.global_pos_z_l0)
        transform.RotateWXYZ(angle, rotation_axis)  # angle, (x, y, z) axis to rotate around
        return transform 
            #return transform_filter.GetOutput()

    def calculate_principal_axes(transformed_points):
        # The box has 8 corner points; they represent the transformed box in 3D.
        # The first corner can be used as a reference, and we can calculate the principal axes 
        # by computing vectors between the first corner and others.
        
        origin = transformed_points[0]  # Reference point (first corner)
        
        # Find points along the X, Y, and Z axes relative to the origin
        x_axis_vector = transformed_points[1] - origin  # X-axis aligned point
        y_axis_vector = transformed_points[3] - origin  # Y-axis aligned point
        z_axis_vector = transformed_points[4] - origin  # Z-axis aligned point
    
        # Normalize the vectors to get the direction of the principal axes
        new_x_axis = x_axis_vector / np.linalg.norm(x_axis_vector)
        new_y_axis = y_axis_vector / np.linalg.norm(y_axis_vector)
        new_z_axis = z_axis_vector / np.linalg.norm(z_axis_vector)
        
        # Also return the lengths of the axes (dimensions)
        x_length = np.linalg.norm(x_axis_vector)
        y_length = np.linalg.norm(y_axis_vector)
        z_length = np.linalg.norm(z_axis_vector)
        return new_x_axis, new_y_axis, new_z_axis, x_length, y_length, z_length

    def get_shortest_dimension_vector(x_axis, y_axis, z_axis, x_length, y_length, z_length):
        lengths = np.array([x_length, y_length, z_length])
        vectors = [x_axis, y_axis, z_axis]
    
        # Find the shortest dimension and corresponding vector
        min_index = np.argmin(lengths)
        shortest_vector = vectors[min_index]
        shortest_length = lengths[min_index]
    
        return shortest_vector, shortest_length

    def get_projections(self):
        """
        Returns xy, xz, yz projections to be plotted with 
        python matplotlib
        """
        if self.points is None:
            self._cache_box_points()
        xy_points = self.points[:, :2]
        xy_patch  = Rectangle(xy_points.min(axis=0), *(xy_points.max(axis=0) - xy_points.min(axis=0)),
                              fill=None, edgecolor='r')
        #xz_points = self.points[:, 1:]
        xz_points = self.points[:, [0,2]]
        xz_patch  = Rectangle(xz_points.min(axis=0), *(xz_points.max(axis=0) - xz_points.min(axis=0)),
                              fill=None, edgecolor='r')
        yz_points = self.points[:, 1:]
        yz_patch  = Rectangle(yz_points.min(axis=0), *(yz_points.max(axis=0) - yz_points.min(axis=0)),
                              fill=None, edgecolor='r')

        return xy_patch, xz_patch, yz_patch


    def _cache_box_points(self):
        if self.points is None:
            box    = self._create_box()
            self.points = box
            
    def draw_xy(self, fill=False, lw=0.8, edgecolor='b', facecolor='b', alpha=0.7) -> Rectangle:
        """
        Draw a matplotlib patch for xy projection
        """
        self._cache_box_points()
        xy_points = self.points[:, :2]
        #if self.paddle_id in [57]:
        if (self.principal == np.array([0,0,1])).all() or (self.principal == np.array([0,0,-1])).all():
            xy_patch  = Rectangle(xy_points.min(axis=0), *(xy_points.max(axis=0) - xy_points.min(axis=0)),
                                  fill=fill, edgecolor=edgecolor, facecolor=facecolor, lw=lw, alpha=alpha)
            # this is an edge paddle. For drawing use the angle feature of the Rectangle patch
            #xy_anchor0 = (self.global_pos_x_l0_A - self.width/np.sqrt(2), self.global_pos_y_l0_A - self.width/np.sqrt(2))
            corners = xy_patch.get_corners()
            # +x +y
            if self.paddle_id in [57,149,150,151]:
                anchor  = (corners[1][0], corners[1][1])
                rotation_point = (corners[1][0], corners[1][1])
                angle = 135
                xy_patch  = Rectangle(anchor, self.width, self.height,
                                      rotation_point = rotation_point, angle = angle,
                                      fill=fill, edgecolor=edgecolor, facecolor=facecolor, lw=lw, alpha=alpha)
            # -x -y
            if self.paddle_id in [58,152,153,154]:
                anchor  = (corners[0][0], corners[0][1])
                rotation_point = (corners[0][0], corners[0][1])
                angle = 45
                xy_patch  = Rectangle(anchor, self.width, self.height,
                                      rotation_point = rotation_point, angle = angle,
                                      fill=fill, edgecolor=edgecolor, facecolor=facecolor, lw=lw, alpha=alpha)
            # corner -x -y
            if self.paddle_id in [59,155,156,157]:
                anchor  = (corners[3][0], corners[3][1])
                rotation_point = (corners[3][0], corners[3][1])
                angle = -45
                xy_patch  = Rectangle(anchor, self.width, self.height,
                                      rotation_point = rotation_point, angle = angle,
                                      fill=fill, edgecolor=edgecolor, facecolor=facecolor, lw=lw, alpha=alpha)
            # corner +x -y
            if self.paddle_id in [60,158,159,160]:
                anchor  = (corners[3][0], corners[3][1])
                rotation_point = (corners[2][0], corners[2][1])
                angle = 45
                xy_patch  = Rectangle(anchor, self.width, self.height,
                                      rotation_point = rotation_point, angle = angle,
                                      fill=fill, edgecolor=edgecolor, facecolor=facecolor, lw=lw, alpha=alpha)

        else:
            xy_patch  = Rectangle(xy_points.min(axis=0), *(xy_points.max(axis=0) - xy_points.min(axis=0)),
                                  fill=fill, edgecolor=edgecolor, facecolor=facecolor, lw=lw, alpha=alpha)
        return xy_patch
    
    def draw_xz(self, fill=False, lw=0.8, edgecolor='b', facecolor='b', alpha=0.7) -> Rectangle:
        """
        Draw a matplotlib patch for xy projection
        """
        self._cache_box_points()
        xz_points = self.points[:, [0,2]]
        xz_patch  = Rectangle(xz_points.min(axis=0), *(xz_points.max(axis=0) - xz_points.min(axis=0)),
                              fill=fill, edgecolor=edgecolor, facecolor=facecolor, alpha=alpha, lw=lw)
        return xz_patch

    def draw_yz(self, fill=False, lw=0.8, edgecolor='b',facecolor='b', alpha=0.7) -> Rectangle:
        """
        Draw a matplotlib patch for xy projection
        """
        self._cache_box_points()
        yz_points = self.points[:, 1:]
        yz_patch  = Rectangle(yz_points.min(axis=0), *(yz_points.max(axis=0) - yz_points.min(axis=0)),
                              fill=fill, edgecolor=edgecolor, facecolor=facecolor, alpha=alpha, lw=lw)
        return yz_patch


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
        _repr += f'\n   cable times [ns] (JAZ) :'
        _repr += f'\n    \u21B3 Coax: {self.coax_cable_time} Harting: {self.harting_cable_time}'
        _repr += f'\n  ** Coordinates (L0) & dimensions **'
        _repr += f'\n   length, width, height [cm]'
        _repr += f'\n    \u21B3 [{self.length:.2f}, {self.width:.2f}, {self.height:.2f}]'
        _repr += f'\n   normal vector (global) (SiPM pointing direction)'
        _repr += f'\n    \u21B3 [{self.normal_x:.2f}, {self.normal_y:.2f}, {self.normal_z:.2f}]'
        _repr += f'\n   center [mm]:'
        _repr += f'\n    \u21B3 [{self.global_pos_x_l0:.2f}, {self.global_pos_y_l0:.2f}, {self.global_pos_z_l0:.2f}]'
        _repr += f'\n   A-side [mm]:'
        _repr += f'\n    \u21B3 [{self.global_pos_x_l0_A:.2f}, {self.global_pos_y_l0_A:.2f}, {self.global_pos_z_l0_A:.2f}]>'
        _repr += f'\n   B-side [mm]:'
        _repr += f'\n    \u21B3 [{self.global_pos_x_l0_B:.2f}, {self.global_pos_y_l0_B:.2f}, {self.global_pos_z_l0_B:.2f}]>'
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

# Eventually we don't want to have this within "tof_db"

class TrackerStrip(models.Model):
    """
    Geometry information about each tracker strip
    """
    strip_id            = models.PositiveIntegerField(
                            primary_key=True,
                            null=False,
                            default=0,
                            unique=True,
                            help_text="The unique identifier for this strip, which is Layer-Row-Module-Channel (5 digit number)")
    layer               = models.IntegerField(null=False, default=0) 
    row                 = models.IntegerField(null=False, default=0) 
    module              = models.IntegerField(null=False, default=0) 
    channel             = models.IntegerField(null=False, default=0)  
    global_pos_x_l0     = models.FloatField(null=False, default=0)
    global_pos_y_l0     = models.FloatField(null=False, default=0)
    global_pos_z_l0     = models.FloatField(null=False, default=0)
    global_pos_x_det_l0 = models.FloatField(null=False, default=0)
    global_pos_y_det_l0 = models.FloatField(null=False, default=0)
    global_pos_z_det_l0 = models.FloatField(null=False, default=0)
    principal_x         = models.FloatField(null=False, default=0)
    principal_y         = models.FloatField(null=False, default=0)
    principal_z         = models.FloatField(null=False, default=0)
    volume_id           = models.PositiveBigIntegerField(
                                default=0,
                                null=False,
                                unique=True,
                                help_text="The VolumeId as used in the GAPS simulation code")

    @staticmethod
    def create_id(layer, row, module, channel):
        return channel + module*100 + row*10000 + layer*100000

    # FIXME - include in save hook!
    def get_id(self):
        return self.create_id(self.layer, self.row, self.module, self.channel)

    @staticmethod
    def get_hid_vid_map() -> dict:
        """
        Get the map of hardware id (strip id) to volume id
        """
        hid_vid_map    = dict()
        all_trk_strips = TrackerStrip.objects.all() 
        for k in all_trk_strips:
            hid_vid_map[k.strip_id] = k.volume_id
        return hid_vid_map

    def __str__(self):
        return self.__repr__()

    def __repr__(self):
        _repr = f'<TrackerStrip [{self.strip_id}]:'
        _repr += f'\n  Layer     : {self.layer}'                   
        _repr += f'\n  Row       : {self.row} '                    
        _repr += f'\n  Module    : {self.module}'                   
        _repr += f'\n  Channel   : {self.channel}'                   
        _repr += f'\n  Volume ID : {self.volume_id}'  
        _repr += f'\n  -- str pos. (from sim) --'
        _repr += f'\n  X: {self.global_pos_x_l0:.1f} Y: {self.global_pos_y_l0:.1f} Z: {self.global_pos_z_l0:.1f}'                 
        _repr += f'\n  -- det pos. (from sim) --'
        _repr += f'\n  X: {self.global_pos_x_det_l0:.1f} Y: {self.global_pos_y_det_l0:.1f} Z: {self.global_pos_z_det_l0:.1f}'                 
        _repr += f'\n  -- principal dir (from sim) --'
        _repr += f'\n  X: {self.principal_x:.1f} Y: {self.principal_y:.1f} Z: {self.principal_z:.1f}>'                 
        return _repr

class TrackerStripPedestal(models.Model):
    """
    The pedestal of each strip as retreived from the 
    text file
    """
    data_id              = models.AutoField(
                               primary_key=True,
                               help_text="Identify this specific dataset")
    strip_id             = models.PositiveIntegerField(
                               null=False,
                               default=0,
                               unique=False,
                               help_text="The unique identifier for this strip, which is Layer-Row-Module-Channel (5 digit number)")
    volume_id            = models.PositiveBigIntegerField(
                               default=0,
                               null=False,
                               unique=False,
                               help_text="The VolumeId as used in the GAPS simulation code")
    utc_timestamp_start  = models.PositiveBigIntegerField(null=False, default=0,
                                                    help_text="UTC Timestamp in UNIX format")
    utc_timestamp_stop   = models.PositiveBigIntegerField(null=False, default=0,
                                                    help_text="UTC Timestamp in UNIX format")
    name                 = models.CharField(max_length=1024,
                                            null=True,
                                            default="",
                                            help_text="A name for this strip pedestals")
    pedestal_mean = models.FloatField(
                        default=0,
                        null=False,
                        help_text="Mean value of the pedestal distribution")
    pedestal_sigma = models.FloatField(
                        default=0,
                        null=False,
                        help_text="Width of the pedestal distribution")
    is_mean_value  = models.BooleanField(
                        default=True,
                        null=False,
                        help_text="If no pedestal is set from a file, it defaults to a mean value. If none is available, this is 0")
    
    @staticmethod
    def get_from_file(filename, utc_start = 0, utc_stop = 0):
        """
        Create database compatible objects from a regular text file
        """
        strip_to_ped = dict()
        total_lines = 0
        with open(filename) as f:
            for line in f.readlines():
                total_lines += 1
        hid_vid_map = TrackerStrip.get_hid_vid_map()  
        if not isinstance(filename, Path):
            filename = Path(filename)
        with open(filename) as f:
            for line in tqdm.tqdm(f.readlines(), total=total_lines):
                #print (line)
                line = line.lstrip().rstrip()
                if line.startswith('#'):
                    continue
                #line = line.split(',')
                line = line.split()
                #print (line)
                layer, row, module, channel = int(line[0]), int(line[1]), int(line[2]), int(line[3])
                ped_mean, ped_sigma         = float(line[4]), float(line[5])
                strip_id = TrackerStrip.create_id(layer, row, module, channel)
                #print (pol_a2_0, pol_a2_1, pol_a2_2)
                #print ('----')
                ped                     = TrackerStripPedestal()
                ped.volume_id           = hid_vid_map[strip_id] 
                ped.utc_timestamp_start = utc_start 
                ped.utc_timestamp_stop  = utc_stop 
                ped.name                = filename.name
                ped.strip_id            = strip_id 
                ped.pedestal_mean       = ped_mean   
                ped.pedestal_sigma      = ped_sigma    
                strip_to_ped[strip_id]  = ped
        return strip_to_ped

    def __str__(self):
        return self.__repr__()
    
    def __repr__(self):
        _repr = f'<TrackerStripPedestal [{self.strip_id}]:'
        if self.is_mean_value:
            _repr += '\n !! -- Mean value for all strips !!'
            _repr += '\n !! -- values not for this individual strip !!'
        _repr += f'\n  UTC Timestmamps begin/end'
        _repr += f'\n    {self.utc_timestamp_start} //{self.utc_timestamp_stop} '  
        _repr += f'\n  Volume ID : {self.volume_id}'  
        _repr += f'\n  ped mean  : {self.pedestal_mean}'
        _repr += f'\n  ped sigma : {self.pedestal_sigma}>'

        return _repr

##########################################################################

class TrackerStripMask(models.Model):
    """
    A simple "on/off" switch for tracker strips. If they are not marked 
    as "active", then they should be removed from the calibrated events.
    """
    data_id       = models.AutoField(
                        primary_key=True,
                        help_text="Identify this specific dataset")
    strip_id      = models.PositiveIntegerField(
                        null=False,
                        default=0,
                        unique=False,
                        help_text="The unique identifier for this strip, which is Layer-Row-Module-Channel (5 digit number)")

    volume_id     = models.PositiveBigIntegerField(
                        default=0,
                        null=False,
                        unique=False,
                        help_text="The VolumeId as used in the GAPS simulation code")
    utc_timestamp_start  = models.PositiveBigIntegerField(null=False, default=0,
                                                    help_text="UTC Timestamp in YYMMDDHHMMSS format")
    utc_timestamp_stop   = models.PositiveBigIntegerField(null=False, default=0,
                                                    help_text="UTC Timestamp in YYMMDDHHMMSS format")
    name                 = models.CharField(max_length=1024,
                                    null=True,
                                    default="",
                                    help_text="A name for this strip mask. There might be serveral per same day, so having only a timestamp might be confusing")
    active = models.BooleanField(
                        default=True,
                        null=False,
                        help_text="The strip is in working condition or not")
    
    def __str__(self):
        return self.__repr__()
    
    def __repr__(self):
        _repr = f'<TrackerStripMask [{self.strip_id}]:'
        _repr += f'\n  Volume ID : {self.volume_id}'  
        _repr += f'\n  UTC Timestmamps begin/end'
        _repr += f'\n    {self.utc_timestamp_start} // {self.utc_timestamp_stop} '  
        _repr += f'\n  Name      : {self.name}'  
        _repr += f'\n  Mask      : {self.active}>'  
        return _repr

    @staticmethod
    def get_from_file(filename, utc_start = 0, utc_stop = 0):
        strip_to_mask = dict()
        total_lines   = 0
        n_entries     = 0
        with open(filename) as f:
            for line in f.readlines():
                total_lines += 1
        hid_vid_map = TrackerStrip.get_hid_vid_map() 
        n_entries   = 0
        if not isinstance(filename, Path):
            name = Path(filename).name 
        else:
            name = filename.name
        with open(filename) as f:
            for line in tqdm.tqdm(f.readlines(), total=total_lines):
                n_entries += 1
                module_id, mask = line.split()
                mask = int(mask, base=16)
                #print (module_id, mask)
                layer  = int(module_id[0])
                row    = int(module_id[1])
                module = int(module_id[2])
                for k in range(32):
                    strip_mask = mask >> k & 0x1
                    tsmask   = TrackerStripMask()
                    strip_id = TrackerStrip.create_id(layer, row, module, k)
                    vid      = hid_vid_map[strip_id]
                    tsmask.strip_id  = strip_id
                    tsmask.active    = bool(strip_mask)
                    tsmask.volume_id = vid
                    tsmask.name      = name
                    strip_to_mask[strip_id] = tsmask
       
        return strip_to_mask

##########################################################################

class TrackerStripTransferFunction(models.Model):
    """
    The polynomial version of the transfer function as 
    given in textfiles.
    This is based on work from R.Munini.
    """
    data_id       = models.AutoField(
                        primary_key=True,
                        help_text="Identify this specific dataset")
    strip_id      = models.PositiveIntegerField(
                        null=False,
                        default=0,
                        unique=False,
                        help_text="The unique identifier for this strip, which is Layer-Row-Module-Channel (5 digit number)")

    volume_id      = models.PositiveBigIntegerField(
                        default=0,
                        null=False,
                        unique=False,
                        help_text="The VolumeId as used in the GAPS simulation code")
    utc_timestamp_start  = models.PositiveBigIntegerField(null=False, default=0,
                                                    help_text="UTC Timestamp in YYMMDDHHMMSS format")
    utc_timestamp_stop   = models.PositiveBigIntegerField(null=False, default=0,
                                                    help_text="UTC Timestamp in YYMMDDHHMMSS format")
    name           = models.CharField(max_length=1024,
                                    null=True,
                                    default="",
                                    help_text="A name for this transfer fn. There might be serveral per same day, so having only a timestamp might be confusing")
    
    # coefficients 
    pol_a2_0       = models.FloatField(default=0, null=False, help_text = "coefficient for transfer function")
    pol_a2_1       = models.FloatField(default=0, null=False, help_text = "coefficient for transfer function")    
    pol_a2_2       = models.FloatField(default=0, null=False, help_text = "coefficient for transfer function")

    pol_b3_0       = models.FloatField(default=0, null=False, help_text = "coefficient for transfer function")
    pol_b3_1       = models.FloatField(default=0, null=False, help_text = "coefficient for transfer function")
    pol_b3_2       = models.FloatField(default=0, null=False, help_text = "coefficient for transfer function")
    pol_b3_3       = models.FloatField(default=0, null=False, help_text = "coefficient for transfer function")

    pol_c3_0       = models.FloatField(default=0, null=False, help_text = "coefficient for transfer function")
    pol_c3_1       = models.FloatField(default=0, null=False, help_text = "coefficient for transfer function")
    pol_c3_2       = models.FloatField(default=0, null=False, help_text = "coefficient for transfer function")
    pol_c3_3       = models.FloatField(default=0, null=False, help_text = "coefficient for transfer function")

    pol_d3_0       = models.FloatField(default=0, null=False, help_text = "coefficient for transfer function")     
    pol_d3_1       = models.FloatField(default=0, null=False, help_text = "coefficient for transfer function")
    pol_d3_2       = models.FloatField(default=0, null=False, help_text = "coefficient for transfer function")
    pol_d3_3       = models.FloatField(default=0, null=False, help_text = "coefficient for transfer function")
    
    def poly_a(self,xs):
        ys = np.zeros(len(xs))
        mask = xs <= 190 
        ys[mask] = self.pol_a2_0 + self.pol_a2_1*xs[mask] + self.pol_a2_2*(xs[mask]**2) 
        return ys
    
    def poly_b(self, s):
        ys = np.zeros(len(xs))
        mask = np.logical_and(190 < xs, xs <= 500)
        ys[mask] = self.pol_b3_0 + self.pol_b3_1*xs[mask] + self.pol_b3_2*(xs[mask]**2) + self.pol_b3_3*(xs[mask]**3) 
        return ys
    
    def poly_c(self, xs):
        ys = np.zeros(len(xs))
        mask = np.logical_and(500 <  xs, xs <= 900)
        ys[mask] = self.pol_c3_0 + self.pol_c3_1*xs[mask] + self.pol_c3_2*(xs[mask]**2) + self.pol_c3_3*(xs[mask]**3) 
        return ys
    
    def poly_d(self, xs):
        ys = np.zeros(len(xs))
        mask = np.logical_and(900 <  xs, xs <= 1600)
        ys[mask] = self.pol_d3_0 + self.pol_d3_1*xs[mask] + self.pol_d3_2*(xs[mask]**2) + self.pol_d3_3*(xs[mask]**3) 
        return ys
    
    def trafo(self, xs):
        if isinstance(xs, float) or isinstance(xs, int):
            xs = np.array(xs)
        a = self.poly_a(xs)
        b = self.poly_b(xs)
        c = self.poly_c(xs)
        d = self.poly_d(xs) 
        ys = a + b + c + d
        return ys

    @staticmethod
    def get_from_file(filename):
        strip_to_tf = dict()
        total_lines = 0
        with open(filename) as f:
            for line in f.readlines():
                total_lines += 1
        
        hid_vid_map = TrackerStrip.get_hid_vid_map()  
        if not isinstance(filename, Path):
            filename = Path(filename)
        with open(filename) as f:
            for line in tqdm.tqdm(f.readlines(), total=total_lines):
                #print (line)
                line =line.lstrip().rstrip()
                if line.startswith('#'):
                    continue
                line = line.split(',')
                #print (line)
                layer, row, module, channel = int(line[0]), int(line[1]), int(line[2]), int(line[3])
                strip_id = TrackerStrip.create_id(layer, row, module, channel)
                pol_a2_0, pol_a2_1, pol_a2_2 = [float(k) for k in line[4:7]]
                #print (pol_a2_0, pol_a2_1, pol_a2_2)
                #print ('----')
                pol_b3_0, pol_b3_1, pol_b3_2, pol_b3_3 = [float(k) for k in line[7:11]]
                pol_c3_0, pol_c3_1, pol_c3_2, pol_c3_3 = [float(k) for k in line[11:15]]
                pol_d3_0, pol_d3_1, pol_d3_2, pol_d3_3 = [float(k) for k in line[15:19]]
                tf = TrackerStripTransferFunction()
                tf.volume_id     = hid_vid_map[strip_id] 
                tf.name          = filename.name
                tf.utc_timestamp = 0
                tf.strip_id = strip_id 
                tf.pol_a2_0 = pol_a2_0 
                tf.pol_a2_1 = pol_a2_1    
                tf.pol_a2_2 = pol_a2_2    
                
                tf.pol_b3_0 = pol_b3_0    
                tf.pol_b3_1 = pol_b3_1    
                tf.pol_b3_2 = pol_b3_2    
                tf.pol_b3_3 = pol_b3_3    
                
                tf.pol_c3_0 = pol_c3_0    
                tf.pol_c3_1 = pol_c3_1    
                tf.pol_c3_2 = pol_c3_2    
                tf.pol_c3_3 = pol_c3_3    
                
                tf.pol_d3_0 = pol_d3_0    
                tf.pol_d3_1 = pol_d3_1    
                tf.pol_d3_2 = pol_d3_2    
                tf.pol_d3_3 = pol_d3_3    

                strip_to_tf[strip_id] = tf 
        return strip_to_tf
    
    def __repr__(self):
        _repr = f'<TrackerStripTransferFunction [{self.strip_id}]:'
        _repr += f'\n  Volume ID : {self.volume_id}'  
        _repr += f'\n  Name      : {self.name}'  
        _repr += f'\n  UTC Timestmamps begin/end'
        _repr += f'\n    {self.utc_timestamp_start} //{self.utc_timestamp_stop} '  
        # FIXME - make this nicer, e.g. 
        _repr += f'\n  Poly A    :{self.pol_a2_0}*adc + {self.pol_a2_1}*adc + {self.pol_a2_2}*(adc**2) for adc < 190'
        _repr += f'\n  Poly B    :{self.pol_b3_0}*adc + {self.pol_b3_1}*adc + {self.pol_b3_2}*(adc**2) + {self.pol_b3_3}*(adc**3) for 190 < adc <= 500'
        _repr += f'\n  Poly C    :{self.pol_c3_0}*adc + {self.pol_c3_1}*adc + {self.pol_c3_2}*(adc**2) + {self.pol_c3_3}*(adc**3) for 500 < adc <= 900'
        _repr += f'\n  Poly D    :{self.pol_d3_0}*adc + {self.pol_d3_1}*adc + {self.pol_d3_2}*(adc**2) + {self.pol_d3_3}*(adc**3) for 900 < adc <= 1600>'
        return _repr
    
    def __str__(self):
        return self.__repr__()

##########################################################################

class TrackerStripCmnNoise(models.Model):
    """
    Pulser measurement to get common noise in the tracker 
    under control
    """
    data_id              = models.AutoField(
                               primary_key=True,
                               help_text="Identify this specific dataset")
    strip_id             = models.PositiveIntegerField(
                               null=False,
                               default=0,
                               unique=False,
                               help_text="The unique identifier for this strip, which is Layer-Row-Module-Channel (5 digit number)")
    volume_id            = models.PositiveBigIntegerField(
                              default=0,
                              null=False,
                              unique=False,
                              help_text="The VolumeId as used in the GAPS simulation code")
    utc_timestamp_start  = models.PositiveBigIntegerField(null=False, default=0,
                                                    help_text="UTC Timestamp in YYMMDDHHMMSS format")
    utc_timestamp_stop   = models.PositiveBigIntegerField(null=False, default=0,
                                                    help_text="UTC Timestamp in YYMMDDHHMMSS format")
    name                 = models.CharField(max_length=1024,
                                          null=True,
                                          default="",
                                          help_text="A name for this transfer fn. There might be serveral per same day, so having only a timestamp might be confusing")
    
    gain                 = models.FloatField(default=0, null=False, help_text = "Gain from pulser (?)")
    pulse_chn            = models.FloatField(default=0, null=False, help_text = "Pulsed channel")    
    pulse_avg            = models.FloatField(default=0, null=False, help_text = "Avg pulse")
    gain_is_mean         = models.BooleanField(default=False, null=False, help_text = "Is the value for the gain a mean value?")
    pulse_is_mean        = models.BooleanField(default=False, null=False, help_text = "Is the value for the pulse avg and chn a mean value?")

    @staticmethod
    def get_from_file(filename, utc_start = 0, utc_stop = 0):
        strip_to_tf = dict()
        total_lines = 0
        with open(filename) as f:
            for line in f.readlines():
                total_lines += 1
        hid_vid_map = TrackerStrip.get_hid_vid_map() 
        n_entries   = 0
        mean_pls    = 0
        mean_avg    = 0
        if not isinstance(filename, Path):
            filename = Path(filename)
        with open(filename) as f:
            for line in tqdm.tqdm(f.readlines(), total=total_lines):
                #print (line)
                line =line.lstrip().rstrip()
                if line.startswith('#'):
                    continue
                n_entries += 1
                line = line.split()
                #print (line)
                layer, row, module, channel = int(line[0]), int(line[1]), int(line[2]), int(line[3])
                strip_id = TrackerStrip.create_id(layer, row, module, channel)
                pulse_chn, pulse_avg = [float(k) for k in line[4:6]]
                #print (pol_a2_0, pol_a2_1, pol_a2_2)
                #print ('----')
                cmnnoise = TrackerStripCmnNoise()
                # FIXME 
                cmnnoise.volume_id           = hid_vid_map[strip_id] 
                cmnnoise.name                = filename.name
                cmnnoise.utc_timestamp_start = utc_start 
                cmnnoise.utc_timestamp_stop  = utc_stop 
                cmnnoise.strip_id            = strip_id 
                cmnnoise.gain                = 0
                cmnnoise.pulse_chn           = pulse_chn    
                cmnnoise.pulse_avg           = pulse_avg    
                mean_pls                    += pulse_chn 
                mean_avg                    += pulse_avg
                strip_to_tf[strip_id]        = cmnnoise 
       
        mean_pls /= n_entries 
        mean_avg /= n_entries 

        # make sure we have one entry per strip even if we don't have the data
        for strip_id in hid_vid_map:
            if not strip_id in strip_to_tf:
                cmnnoise = TrackerStripCmnNoise()
                # FIXME 
                cmnnoise.volume_id           = hid_vid_map[strip_id] 
                cmnnoise.name                = filename.name
                cmnnoise.utc_timestamp_start = utc_start 
                cmnnoise.utc_timestamp_stop  = utc_stop 
                cmnnoise.strip_id            = strip_id 
                cmnnoise.gain                = 0
                cmnnoise.pulse_chn           = int(mean_pls)     
                # no mean val
                cmnnoise.pulse_avg           = 0
                #cmnnoise.pulse_avg           = mean_avg    
                cmnnoise.pulse_is_mean       = True
                strip_to_tf[strip_id]        = cmnnoise 
                
        return strip_to_tf
  
    @staticmethod 
    def add_gains(filename, cmnnoise_dict):
        """
        Take a previously created dictionary of common
        noise data and add the gains from a different 
        file
        """
        #ngains      = 0
        total_lines = 0
        n_entries   = 0
        mean_gain   = 0
        with open(filename) as f:
            for line in f.readlines():
                total_lines += 1
        with open(filename) as f:
            for line in tqdm.tqdm(f.readlines(), total=total_lines):
                #print (line)
                line =line.lstrip().rstrip()
                if line.startswith('#'):
                    continue
                #line = line.split(',')
                n_entries += 1 
                line = line.split()
                layer, row, module, channel = int(line[0]), int(line[1]), int(line[2]), int(line[3])
                strip_id = TrackerStrip.create_id(layer, row, module, channel)
                gain     = float(line[4])
                mean_gain += gain 
                #print (line)
                cmnnoise_dict[strip_id].gain = gain 
        # calculate mean gain
        mean_gain /= n_entries
        for k in cmnnoise_dict:
            if cmnnoise_dict[k].gain == 0:
                # no mean gain!
                cmnnoise_dict[k].gain = 1
                #cmnnoise_dict[k].gain = mean_gain 
                cmnnoise_dict[k].gain_is_mean = True 

    def __repr__(self):
        _repr = f'<TrackerStripCmnNoise [{self.strip_id}]:'
        _repr += f'\n  Volume ID     : {self.volume_id}'  
        _repr += f'\n  UTC Timestmamps begin/end:'
        _repr += f'\n    {self.utc_timestamp_start} // {self.utc_timestamp_stop} '  
        _repr += f'\n  Name          : {self.name}'  
        _repr += f'\n  Gain is mean  : {self.gain_is_mean}'
        _repr += f'\n  Pulse is mean : {self.pulse_is_mean}'
        _repr += f'\n  Pulse Chn     : {self.pulse_chn}, Gain : {self.gain:.2f}, Pulse Avg : {self.pulse_avg:.2f}>'
        return _repr
    
    def __str__(self):
        return self.__repr__()

##########################################################################

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
