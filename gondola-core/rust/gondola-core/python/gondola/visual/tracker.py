"""
Tracker related plots and visualizations
"""

from .. import _gondola_core as _gc 

import numpy as np
import matplotlib
import matplotlib.pyplot as plt
import charmingbeauty as cb 
import charmingbeauty.layout as lo
from matplotlib import patches

from loguru import logger
from copy import deepcopy as copy

strip_lines = _gc.tracker.strip_lines

def plot_strip_lines(ax, det, layer : int, color='k'):
    """
    Use lines to indicate detector strips. Automatically apply 
    correct orientation for even/odd layers.

    # Arguments:
        ax    : axis instance to plot on 
        det   : iterable providing x,y coordinates
        layer : tracker layer (0-9)

    # Keyword Arguments:
        color : the color of the lines
    """
    strip_widths = strip_lines()
    if layer % 2 == 0: # even layer
        for r,sw in strip_widths:
            ax.plot([det[0] + sw, det[0] + sw], [det[1] - r, det[1] + r], color=color, alpha=0.4, lw=0.5)
    else:
        for r,sw in strip_widths:
            ax.plot([det[0] - r, det[0]+ r], [det[1] + sw, det[1] + sw], color=color, alpha=0.4, lw=0.5)

#-------------------------------------------------------

def plot_tracker_proj(ax,\
                      hits,\
                      projection   = "xy",\
                      use_energy   = False,\
                      cmap         = matplotlib.colormaps['seismic'],\
                      circle_color ='w',\
                      color_energy = False,\
                      cnorm_max    = None,\
                      hitstyle     = {'edgecolor': 'w', 'alpha' : 0.5 , 'marker' : 'o'} ):
    """
    Plot 2d projection of the tracker on the given axis. Hits can be indicated if 
    available. Use together with "prepare_layer_fig" for a full plot with 
    labeled axes.

    # Arguments: 
      * ax         : The axes to plot on 
      * hits       : An iterable of gondola_core.events.TrackerHit

    # Keyword Arguments:
      * projection   : The projection to plot. Can be either "xy", "xz" or "yz" 
      * cmap         : A matplotlib compatible colormap to color the hits
      * circle_color : The color with the circle showing the detector outlines 
                       are indicated 
      
      * color_energy : Color the circles for the hits with the energy deposition
                       FIXME - color scale needs to be normalized together with the 
                       TOF
      * cnorm_max    : The colors can be normalized. Specify a maximum for the normalization
                       (min will be 0)
      * hitstyle     : Specify the style of the plotted hits by a dictionary which will be 
                       passed on to plt.scatter
    """
    if not projection in ["xy", "xz", "yz"]:
        raise ValueError("projection argument needs to be either 'xy', 'xy' or 'xz'!")

    hit_layers = list(set([k.layer for k in hits]))
    strip_dict = _gc.db.TrackerStrip.all_as_dict() 
    det_coords = [(k.global_pos_x_det_l0, k.global_pos_y_det_l0, k.global_pos_z_det_l0) for k in strip_dict.values()]
    det_coords = [k for k in set(det_coords)]
    strip_coord     = [strip_dict[k].coordinates for k in strip_dict]
    hit_strip_coord = [strip_dict[h.strip_id].coordinates for h in hits]
    hit_det_coord   = [strip_dict[h.strip_id].detector_coordinates for h in hits]
    #def get_stripcoordinates(h : _gc.events.TrackerHit):
    #    geo = strip_dict[h.strip_id] 
    #    return (geo.global_pos_x_l0, geo.global_pos_y_l0, geo.global_pos_z_l0)
    #
    #def get_detcoordinates(h : _gc.events.TrackerHit):
    #    geo = strip_dict[h.strip_id]
    #    return (geo.global_pos_x_det_l0, geo.global_pos_y_det_l0, geo.global_pos_z_det_l0)

    #all_dets = [(k.global_pos_x_det_l0, k.global_pos_y_det_l0, k.global_pos_z_det_l0) for k in go.db.TrackerStrip.objects.all()]
    idx = 0,1 
    match projection:
        case "xy":
            pass
        case "xz":
            idx = 0,2
        case "yz":
            idx = 1,2 
    # this is for all detectors   
    all_dets_coord = set([(k[idx[0]],k[idx[1]]) for k in det_coords])
   
    # this is for hit detectors
    #strip_coord = [get_stripcoordinates(k) for k in hits]
    #det_coord   = [get_detcoordinates(k) for k in hits]
    adc         = [k.adc for k in hits]
    if use_energy: # this requires that the hit underwent coordinates
        adc = [k.energy for k in hits]
    else:
        adc = np.array(adc) / 12
    xs = [h.x for h in hits]
    ys = [h.y for h in hits]
    zs = [h.z for h in hits]
    if color_energy:
        if cnorm_max != None:
            cm_norm_pts = plt.Normalize(vmin=0, vmax=cnorm_max)
        else:
            cm_norm_pts = plt.Normalize(vmin=min(adc), vmax=max(adc))
    # construct the detector plot
    match projection:
        case "xy":
            for k in all_dets_coord:
                patch = patches.Circle(k, radius=5, fill=False, color='gray', alpha=0.1)
                for layer in hit_layers:
                    plot_strip_lines(ax, k, layer, color='gray')
                ax.add_patch(patch)
            # only for hit strips
            for k in hit_det_coord:
                patch = patches.Circle(k, radius=5, fill=False, color=circle_color)
                ax.add_patch(patch)
                for layer in hit_layers:
                    plot_strip_lines(ax, k, layer, color='gray')
            if not color_energy:
                ax.scatter(xs, ys, marker=hitstyle['marker'], s=adc, facecolor='none', edgecolor=hitstyle['edgecolor'])
            else:
                ax.scatter(xs, ys, marker=hitstyle['marker'], s=100*adc,\
                           edgecolor=hitstyle['edgecolor'], color=cmap(cm_norm_pts(adc)))
        case "xz":
            for k in all_dets_coord:
                det = [k[0] - 5, k[1]]
                rect_patch = patches.Rectangle(det, width=10, height=0.25, color='gray', alpha=0.5)
                ax.add_patch(rect_patch)
            # only for hits 
            for k in hit_det_coord:
                det = [k[0] - 5, k[2]]
                rect_patch = patches.Rectangle(det, width=10, height=0.25, color=circle_color, alpha=0.8)
                ax.add_patch(rect_patch)
            if not color_energy:
                ax.scatter(xs, zs, marker=hitstyle['marker'], s=adc, facecolor='none', edgecolor=hitstyle['edgecolor'])
            else:
                ax.scatter(xs, zs, marker=hitstyle['marker'], s=100*adc,\
                           edgecolor=hitstyle['edgecolor'], color=cmap(cm_norm_pts(adc)))
        case "yz":
            for k in all_dets_coord:
                det = [k[0] - 5, k[1]]
                rect_patch = patches.Rectangle(det, width=10, height=0.25, color='gray', alpha=0.5)
                ax.add_patch(rect_patch)
            # only the hits 
            for k in hit_det_coord:
                det = [k[1] - 5, k[2]]
                rect_patch = patches.Rectangle(det, width=10, height=0.25, color=circle_color, alpha=0.8)
                ax.add_patch(rect_patch)
            if not color_energy:
                ax.scatter(ys, zs, marker=hitstyle['marker'], s=adc, facecolor='none', edgecolor=hitstyle['edgecolor'])
            else:
                ax.scatter(ys, zs, marker=hitstyle['marker'], s=100*adc,\
                           edgecolor=hitstyle['edgecolor'], color=cmap(cm_norm_pts(adc)))
    return ax

#-------------------------------------------------------

def prepare_layer_fig(layer, projection='XY'):
    """
    Set up figure and axis objects for a 2d projection
    of tracker layers.

    This will NOT plot any detectors/hits.
    It will ONLY prepare the figure/axes objects with 
    labels and such!

    # Keyword Arguments:
        * projection : Either 'XY', 'XZ' or 'ZY'. Show 
                       the tracker from the respective 
                       side
    """
    fig = plt.figure(figsize=lo.FIGSIZE_A4_SQUARE)
    ax  = fig.gca()
    ax.set_title(f'{layer}', loc='right')
    ax.spines['top'].set_visible(True)
    ax.spines['right'].set_visible(True)
    ax.grid(0)
    ax.set_aspect('equal')
    if projection == 'XY':
        ax.set_xlabel('X [cm]', loc='right')
        ax.set_ylabel('Y [cm]', loc='top')
        ax.set_xlim(-79,79)
        ax.set_ylim(-79,79) 
    if projection == 'XZ':
        ax.set_xlabel('X [cm]', loc='right')
        ax.set_ylabel('Z [cm]', loc='top')
    if projection == 'YZ':
        ax.set_xlabel('Y [cm]', loc='right')
        ax.set_ylabel('Z [cm]', loc='top')
    return fig, ax

#-------------------------------------------------------

def plot_tracker(hits = None,\
                 mask = None,\
                 circle_color='w') -> dict:
    """
    Create a number of 2d plots for the tracker. This includes
    individual plots for the individual layers as well as
    combined projections in xy, xz and yz. 
    
    # Keyword Arguments:
      * hits : A list of tracker hits. When given then 
               plot only those layers which have hits 
               If None given, plot all layers 
               (e.g. to be used with the mask argument)
      * mask : Indicate masked strips (grey them out) 

    """
    if not hits:
        logger.warning(f'No hits available for tracker layer plots!')
        return dict()
    strip_dict  = _gc.db.TrackerStrip.all_as_dict() 

    #n_layers   = len(set([k.layer for k in hits]))
    all_layers = list(set([k.layer for k in hits]))

    all_dets = []
    #all_strips = []

    all_dets = [(k.global_pos_x_det_l0, k.global_pos_y_det_l0, k.global_pos_z_det_l0) for k in strip_dict.values()]

    # this is used later for the individual layer plots
    #all_dets_xz = set([(k[0],k[2]) for k in all_dets])
    #all_dets_yz = set([(k[1],k[2]) for k in all_dets])
    all_dets_xy = set([(k[0],k[1]) for k in all_dets])

    # prepare output dictionary
    figures = dict()
 
    # prep the hits
    #strip_coord = [strip_dict[h.strip_id].coordinates for h in hits]
    #det_coord   = [strip_dict[h.strip_id].detector_coordinates for h in hits]
    #adc         = [k.adc for k in hits]
    #adc         = np.array(adc) / 12
    #xs          = [k[0] for k in strip_coord]
    #ys          = [k[1] for k in strip_coord]
    #zs          = [k[2] for k in strip_coord]
    
    # have one plot for the xy projection
    fig, ax     = prepare_layer_fig(layer='XY hit projection')
    plot_tracker_proj(ax, hits, projection='xy')
    figures['trk_proj_xy'] = copy(fig)
    
    # another plot for the xz projection
    fig, ax     = prepare_layer_fig(layer='XZ hit projection', projection='XZ')
    plot_tracker_proj(ax, hits, projection='xz')
    figures['trk_proj_xz'] = copy(fig)
    #ax.scatter(xs, zs, marker='o', s=adc, facecolor='none', edgecolor='r')
    ## strips
    #for k in all_dets_xz:
    #    det = [k[0] - 5, k[1]]
    #    rect_patch = patches.Rectangle(det, width=10, height=0.25, color='gray', alpha=0.1)
    #    ax.add_patch(rect_patch)
    #figures['trk_proj_xz'] = copy(fig)

    # another plot for the yz projection
    fig, ax     = prepare_layer_fig(layer='YZ hit projection', projection='YZ')
    plot_tracker_proj(ax, hits, projection='yz')
    figures['trk_proj_yz'] = copy(fig)
    #strip_coord = [get_stripcoordinates(k, strip_dict) for k in hits]
    #det_coord   = [get_detcoordinates(k, strip_dict) for k in hits]
    #adc = [k.adc for k in hits]
    #adc = np.array(adc) / 12
    #ys = [k[1] for k in strip_coord]
    #zs = [k[2] for k in strip_coord]
    #dets_ys = [k[0] for k in all_dets_yz]
    #dets_zs = [k[1] for k in all_dets_yz]
    # strips
    #for k in all_dets_yz:
    #    det = [k[0] - 5, k[1]]
    #    rect_patch = patches.Rectangle(det, width=10, height=0.25, color='gray', alpha=0.1)
    #    ax.add_patch(rect_patch)
    #ax.scatter(dets_ys, dets_zs, marker='+', color='gray')
    #ax.scatter(ys, zs, marker='o', s=adc, facecolor='none', edgecolor='r')
    #for k in det_coord:
    #    patch = patches.Circle(k, radius=5, fill=False, color='k')
    #    # im.set_clip_path(patch)
    #    ax.add_patch(patch)
    #figures['trk_proj_xz'] = copy(fig)

    # one plot per layer
    for __, layer in enumerate(all_layers):
        strip_coord = [strip_dict[h.strip_id].coordinates for h in hits if h.layer == layer]
        det_coord   = [strip_dict[h.strip_id].detector_coordinates for h in hits if h.layer == layer]
        adc         = [k.adc for k in hits if k.layer == layer]
        adc         = np.array(adc) / 12
        xs          = [k[0] for k in strip_coord]
        ys          = [k[1] for k in strip_coord]
        
        fig, ax = prepare_layer_fig(layer=f'Layer {layer}') 
        ax.scatter(xs, ys, marker='o', s=adc, facecolor='none', edgecolor='r')
        ax.scatter(xs, ys, marker='.', color='k', alpha=0.1)
        ax.set_title(f'Layer {layer}', loc='right')
        for k in det_coord:
            patch = patches.Circle(k, radius=5, fill=False, color=circle_color)
            # im.set_clip_path(patch)
            ax.add_patch(patch)
            #go.tracker.visual.plot_strip_lines(ax,k, layer, color=circle_color)
            plot_strip_lines(ax,k, layer, color=circle_color)
        for k in all_dets_xy:
            patch = patches.Circle(k, radius=5, fill=False, color='gray', alpha=0.1)
            # rect_patch = patches.Rectangle(k[1], width=1, height=100, color='gray', alpha=0.1)
            ax.add_patch(patch)
            # ax.add_patch(rect_patch)

        figures[f'trk_layer_{layer}'] = copy(fig)
    return figures

#-------------------------------------------------------

#import numpy as np
#import matplotlib.pyplot as plt
#import charmingbeauty.layout as lo 
#
#SILI_RADIUS = 5 # mm, with guardring and all
#
##-------------------------------------------------------
#

#-------------------------------------------------------

