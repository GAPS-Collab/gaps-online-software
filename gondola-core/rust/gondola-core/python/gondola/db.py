"""
Gondola core database functionality. This is a wrapper around 
the database functions in gondola-db
"""

from . import _gondola_core as _gc  
from matplotlib.patches import Rectangle

import vtk
import numpy as np


TofPaddle                    = _gc.db.TofPaddle
TofPaddle.__module__         = __name__
TofPaddle.__name__           = "TofPaddle"
TofPaddle.__doc__            = _gc.db.TofPaddle.__doc__
ReadoutBoard                 = _gc.db.ReadoutBoard
ReadoutBoard.__module__      = __name__ 
ReadoutBoard.__name__        = 'ReadoutBoard' 
TrackerStrip                 = _gc.db.TrackerStrip
TrackerStrip.__module__      = __name__ 
TrackerStrip.__name__        = 'TrackerStrip' 
TrackerStripMask             = _gc.db.TrackerStripMask 
TrackerStripMask.__module__  = __name__ 
TrackerStrip.__name__        = 'TrackerStrip' 
TrackerStripPedestal         = _gc.db.TrackerStripPedestal 
TrackerStripPedestal.__module__ = __name__ 
TrackerStripPedestal.__name__   = 'TrackerStripPedestal' 
TrackerStripTransferFunction = _gc.db.TrackerStripTransferFunction 
TrackerStripTransferFunction.__module__ = __name__ 
TrackerStripTransferFunction.__name__    = 'TrackerStripPedestal' 
TrackerStripCmnNoise         = _gc.db.TrackerStripCmnNoise
TrackerStripCmnNoise.__module__  = __name__ 
TrackerStripCmnNoise.__name__    = 'TrackerStripCmnNoise'
TofPaddleTimingConstant            = _gc.db.TofPaddleTimingConstant 
TofPaddleTimingConstant.__module__ = __name__ 
TofPaddleTimingConstant.__name__   = 'TofPaddleTimingConstant'
get_all_rbids_in_db          = _gc.db.get_all_rbids_in_db
get_hid_vid_map              = _gc.db.get_hid_vid_map
get_vid_hid_map              = _gc.db.get_vid_hid_map
get_dsi_j_ch_pid_map         = _gc.db.get_dsi_j_ch_pid_map

__all__ = ['TofPaddle','ReadoutBoard', 'TofPaddleTimingConstant', 'TrackerStrip','TrackerStripPedestal', 'TrackerStripTransferFunction',\
'TrackerStripCmnNoise']

#----------------------------------------
# extend the TofPaddles with a few methods
# to draw 3d/2d objects

def _create_box(self):

    # we can kind of cheat and do a cheap transform
    # the principal is either x, y or z axis, no mix-ins
    pr   = self.principal
    norm = self.normal
    cube = vtk.vtkCubeSource()
    edgepaddle_u = False
    edgepaddle_v = False
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
        if (pr == np.array([0,0,1])).all():
            edgepaddle_u = True
        if (pr == np.array([0,0,-1])).all():
            edgepaddle_v = True
        #transform_filter = vtk.vtkTransformPolyDataFilter()
        #transform_filter.SetInputData(cube.GetOutput())
        #transform_filter.SetTransform(trafo)
        #transform_filter.Update()
    else:
        ValueError(f'Unexpected principal axes {pr}')
    cube.Update()
    transform = vtk.vtkTransform()
    transform.Translate(self.global_pos_x_l0, self.global_pos_y_l0, self.global_pos_z_l0)
    #if edgepaddle_u:
    #    transform.RotateWXYZ(90, [0,0,1])  # angle, (x, y, z) axis to rotate around
    # FIXME - all are edgepaddle v
    if edgepaddle_v:
        if (norm == np.array([1.0, 1.0, 0.0])).all():
            transform.RotateWXYZ(45, [0,0,1])  # angle, (x, y, z) axis to rotate around
        if (norm == np.array([1.0, -1.0, 0.0])).all():
            transform.RotateWXYZ(-45, [0,0,1])  # angle, (x, y, z) axis to rotate around
        if (norm == np.array([-1.0, 1.0, 0.0])).all():
            transform.RotateWXYZ(-45, [0,0,1])  # angle, (x, y, z) axis to rotate around
        if (norm == np.array([-1.0, -1.0, 0.0])).all():
            transform.RotateWXYZ(45, [0,0,1])  # angle, (x, y, z) axis to rotate around
    transform_filter = vtk.vtkTransformPolyDataFilter()
    transform_filter.SetInputData(cube.GetOutput())
    transform_filter.SetTransform(transform)
    transform_filter.Update()
    #print (cube)
    # Extract the transformed points
    return transform_filter.GetOutput() 

TofPaddle._create_box = _create_box

def _create_box_points(self):
    points = self._create_box().GetPoints()
    #points = transform_filter.GetOutput().GetPoints()
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

TofPaddle._create_box_points = _create_box_points
#TofPaddle.points      = None
#setattr(TofPaddle, "points", None)
#TofPaddle._points      = None
#def _set_points(self, points): 
#    self._points = points 
#TofPaddle.points = property(fget = lambda self : self._points, fset = _set_points, doc="points as for 2d/3d projection") 

#def _cache_box_points(self):
#    if self.points is None:
#        box    = self._create_box()
#        self.points = box
#
#TofPaddle._cache_box_points = _cache_box_points 

def _draw_xy(self, fill=False, lw=0.8, edgecolor='b', facecolor='b', alpha=0.7) -> Rectangle:
    """
    Draw a matplotlib patch for xy projection
    """
    #self._cache_box_points()
    box = self._create_box_points() 
    xy_points = box[:, :2]
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

TofPaddle.draw_xy = _draw_xy


def _draw_xz(self, fill=False, lw=0.8, edgecolor='b', facecolor='b', alpha=0.7) -> Rectangle:
    """
    Draw a matplotlib patch for xy projection
    """
    box = self._create_box_points()
    xz_points = box[:, [0,2]]
    xz_patch  = Rectangle(xz_points.min(axis=0), *(xz_points.max(axis=0) - xz_points.min(axis=0)),
                          fill=fill, edgecolor=edgecolor, facecolor=facecolor, alpha=alpha, lw=lw)
    return xz_patch

TofPaddle.draw_xz = _draw_xz

def _draw_yz(self, fill=False, lw=0.8, edgecolor='b',facecolor='b', alpha=0.7) -> Rectangle:
    """
    Draw a matplotlib patch for xy projection
    """
    box = self._create_box_points()
    yz_points = box[:, 1:]
    yz_patch  = Rectangle(yz_points.min(axis=0), *(yz_points.max(axis=0) - yz_points.min(axis=0)),
                          fill=fill, edgecolor=edgecolor, facecolor=facecolor, alpha=alpha, lw=lw)
    return yz_patch

TofPaddle.draw_yz = _draw_yz

