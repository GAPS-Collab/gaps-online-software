"""
Variable plot visualization
FIXME - might get moved around
"""

import matplotlib
import matplotlib.pyplot as plt
import matplotlib.cm as cm
import numpy as np

import charmingbeauty as cb
import charmingbeauty.layout as lo

import dashi as d
d.visual()

from .. import db

###############################################

def timeseries_plot(times, 
                    data,
                    title='',
                    xlabel='',
                    ylabel='',
                    savename=''):
    """
    Create a general timeseries plot for a variable over 
    the mission elapsed time

    # Arguments:
        * times : gcutimes, ideally re-normalized to run start time
        * data  : quantity to plot over time

    # Keyword Arguments:
        title     : axis title
        xlabel    : label for the x-axis
        ylabel    : label for the y-axis
        savevname : save plot with this filename, 
                    if None, don't save
    """
    
    fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT)
    ax  = fig.gca()
    
    ax.set_title(title, loc='right')
    ax.set_ylabel(ylabel, loc='top', rotation=0)
    ax.set_xlabel(xlabel, loc='right')
    ax.plot(times, data)
    #ax.legend(loc='upper left', frameon=False, ncol=2, fontsize=8,bbox_to_anchor=(0.05, 1.25))
    cb.visual.adjust_minor_ticks(ax, which='both')
    if savename is not None:
        fig.savefig(savename)
    return fig

###############################################

def timeseries_multiplot(times, 
                         variables,
                         labels,
                         title='',
                         xlabel='',
                         ylabel='',
                         savename=''):
    """
    Create a general timeseries plot for multiple variables over 
    the mission elapsed time.
    This is basically the same as timeseries_plot, however, for 
    multiple variables which will all be plotted in the same axis.

    # Arguments:
        * times      : gcutimes, ideally re-normalized to run start time
        * variables  : quantities to plot over time. This should be a list 
                       of lists (or arrays)
        * labels     : individual labels for the variables, same ordering 
                       structure

    # Keyword Arguments:
        title     : axis title
        xlabel    : label for the x-axis
        ylabel    : label for the y-axis
        savevname : save plot with this filename, 
                    if None, don't save
    """
    assert len(labels) == len(variables)

    fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT)
    ax  = fig.gca()
    
    ax.set_title(title, loc='right')
    ax.set_ylabel(ylabel, loc='top', rotation=0)
    ax.set_xlabel(xlabel, loc='right')
    #times   = np.array([j[0] for j in mtb_hk])
    for idx,data in enumerate(variables):
        ax.plot(times, data, label=labels[idx])
    ax.legend(loc='upper left', frameon=False, ncol=2, fontsize=8,bbox_to_anchor=(0.05, 1.25))
    cb.visual.adjust_minor_ticks(ax, which='both')
    if savename is not None:
        fig.savefig(savename)
    return fig

###############################################

def tof_projection_xy(paddle_occupancy = {}, 
                      cmap = matplotlib.colormaps['hot'],
                      show_cbar = True):
    """
    Show the projection of all paddles which
    are facing in z-direction
    These are the whole Umbrella as well as 
    CBE TOP + Bottom.
    While this plot can show the occupancy of TOF paddles,
    it can also be 'hijacked' to just highlight certain
    paddles.

    
    # Keyword Arguments:
        paddle_occupancy : The number of events per paddle
        cmap             : Colormap - can be lambda function
                           to return color value based on 
                           'occupancy' numbker
        show_cbar        : Show the colorbar on the figure
    """
    fig, axs        = plt.subplots(1, 3, figsize=(18, 5), gridspec_kw={'width_ratios': [1, 1, 1]})
    umb_paddles     = db.get_umbrella_paddles()
    cbe_top_paddles = db.Paddle.objects.filter(panel_id=1)
    cbe_bot_paddles = db.Paddle.objects.filter(panel_id=2)
    xmin, xmax      = -100,100
    ymin, ymax      = -100,100
    zmin, zmax      = -25, 120
    title           = 'TOF UMB/CBE TOP/CBE BOT xy projection'
    for pdl in umb_paddles:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[0].add_patch(pdl.draw_xy(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[0].add_patch(pdl.draw_xy(fill=True, edgecolor='k', facecolor='w'))
        axs[0].set_xlabel('x [cm]', loc='right')
        axs[0].set_ylabel('y [cm]', loc='top')#, rotation=90)
        axs[0].set_aspect('equal')
        axs[0].set_xlim(-200, 200)
        axs[0].set_ylim(-200, 200)
        axs[0].set_title('UMB', loc='right')
    for pdl in cbe_top_paddles:   
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[1].add_patch(pdl.draw_xy(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[1].add_patch(pdl.draw_xy(fill=True, edgecolor='k', facecolor='w'))
        axs[1].set_xlabel('x [cm]', loc='right')
        axs[1].set_ylabel('y [cm]', loc='top')#, rotation=90)
        axs[1].set_xlim(-100, 100)
        axs[1].set_ylim(-100, 100)
        axs[1].set_aspect('equal')
        axs[1].set_title('CBE TOP', loc='right')
    for pdl in cbe_bot_paddles:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[2].add_patch(pdl.draw_xy(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[2].add_patch(pdl.draw_xy(fill=True, edgecolor='k', facecolor='w'))
        axs[2].set_xlabel('x [cm]', loc='right')
        axs[2].set_ylabel('y [cm]', loc='top')#, rotation=270)
        axs[2].set_xlim(-100, 100)
        axs[2].set_ylim(-100, 100)
        axs[2].set_aspect('equal')
        axs[2].set_title('CBE BOT')
        
    axs[0].spines['top'].set_visible(True)
    axs[1].spines['top'].set_visible(True)
    axs[2].spines['top'].set_visible(True)
    axs[0].spines['right'].set_visible(True)
    axs[1].spines['right'].set_visible(True)
    axs[2].spines['right'].set_visible(True)
    if paddle_occupancy and show_cbar:
        cbar_ax = fig.add_axes([0.9, 0.0, 0.05, 1.0])
        cbar_ax.set_axis_off()
        sm = cm.ScalarMappable(cmap=cmap, norm=matplotlib.colors.Normalize())
        sm.set_array([0, 1])
        ax = plt.sca(cbar_ax)
        plt.colorbar(sm, ax=cbar_ax, label='Relative occupancy')
    fig.suptitle(title, x=0.9)
    return fig, axs

###############################################

def unroll_cbe_sides(paddle_occupancy = {},
                     cmap             = matplotlib.colormaps['hot'],
                     show_cbar        = True):
    """
    Project the sides of the cube on xz and yz as well 
    as add the 'edge' paddles.

    While this plot can show the occupancy of TOF paddles,
    it can also be 'hijacked' to just highlight certain
    paddles.
    
    # Keyword Arguments:
        paddle_occupancy : The number of events per paddle
        cmap             : Colormap - can be lambda function
                           to return color value based on 
                           'occupancy' numbker
        show_cbar        : Show the colorbar on the figure
    """
    fig, axs  = plt.subplots(1, 4, sharey=True,figsize=(22, 5), gridspec_kw={'width_ratios': [1, 1, 1, 1]})
    # normal +X
    cbe_front = db.Paddle.objects.filter(panel_id=3) 
    # edge normal +X+Y
    ep_1      = db.Paddle.objects.filter(paddle_id=57)
    # normal +Y
    cbe_sb    = db.Paddle.objects.filter(panel_id=4)
    # endge normal -X+Y
    ep_2      = db.Paddle.objects.filter(paddle_id=58)
    # normal -X 
    cbe_back  = db.Paddle.objects.filter(panel_id=5) 
    # edge normal -X-Y
    ep_3      = db.Paddle.objects.filter(paddle_id=59)
    # normal -Y 
    cbe_bb    = db.Paddle.objects.filter(panel_id=6)
    # edge normal +X-Y
    ep_4      = db.Paddle.objects.filter(paddle_id=60)

    xmin, xmax = -110,110
    ymin, ymax = -110,110
    zmin, zmax = -25, 120
    title      = 'Relative occupancy, xy projection'
    
    ep = ep_1[0]
    if paddle_occupancy:
        color = cmap(paddle_occupancy[ep.paddle_id])
        axs[0].add_patch(ep.draw_yz(fill=True, edgecolor=color, facecolor=color))
    else:
        axs[0].add_patch(ep.draw_yz(fill=True, edgecolor='k', facecolor='w'))

    for pdl in cbe_front:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[0].add_patch(pdl.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[0].add_patch(pdl.draw_yz(fill=True, edgecolor='b', facecolor='w'))
    axs[0].set_xlabel('y [cm]', loc='right')
    axs[0].set_ylabel('z [cm]', loc='top')#, rotation=90)
    axs[0].set_aspect('equal')
    axs[0].set_xlim(-80, 90)
    axs[0].set_ylim(-10, 120)
    axs[0].set_title('CBE +X', loc='right')
    
    # +Y side
    ep = ep_2[0]
    if paddle_occupancy:
        color = cmap(paddle_occupancy[ep.paddle_id])
        axs[1].add_patch(ep.draw_xz(fill=True, edgecolor=color, facecolor=color))
    else:
        axs[1].add_patch(ep.draw_xz(fill=True, edgecolor='k', facecolor='w'))

    for pdl in cbe_sb:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[1].add_patch(pdl.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[1].add_patch(pdl.draw_xz(fill=True, edgecolor='b', facecolor='w'))
    axs[1].set_xlabel('x [cm]', loc='right')
    #axs[1].set_ylabel('z [cm]', loc='top')#, rotation=90)
    axs[1].set_aspect('equal')
    axs[1].set_xlim(-90, 80)
    axs[1].set_ylim(-10, 120)
    axs[1].set_title('CBE +Y', loc='right')
    axs[1].invert_xaxis()

    ep = ep_3[0]
    if paddle_occupancy:
        color = cmap(paddle_occupancy[ep.paddle_id])
        axs[2].add_patch(ep.draw_yz(fill=True, edgecolor=color, facecolor=color))
    else:
        axs[2].add_patch(ep.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    for pdl in cbe_back:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[2].add_patch(pdl.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[2].add_patch(pdl.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    axs[2].set_xlabel('y [cm]', loc='right')
    #axs[2].set_ylabel('z [cm]', loc='top')#, rotation=90)
    axs[2].set_xlim(-90, 80)
    axs[2].set_ylim(-10, 120)
    axs[2].set_aspect('equal')
    axs[2].invert_xaxis()
    axs[2].set_title('CBE -X', loc='right')
    
    # -Y side
    ep = ep_4[0]
    if paddle_occupancy:
        color = cmap(paddle_occupancy[ep.paddle_id])
        axs[3].add_patch(ep.draw_xz(fill=True, edgecolor=color, facecolor=color))
    else:
        axs[3].add_patch(ep.draw_xz(fill=True, edgecolor='k', facecolor='w'))

    for pdl in cbe_bb:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[3].add_patch(pdl.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[3].add_patch(pdl.draw_xz(fill=True, edgecolor='b', facecolor='w'))
    axs[3].set_xlabel('x [cm]', loc='right')
    #axs[3].set_ylabel('z [cm]', loc='top')#, rotation=90)
    axs[3].set_aspect('equal')
    axs[3].set_xlim(-80, 90)
    axs[3].set_ylim(-10, 120)
    axs[3].set_title('CBE +Y', loc='right')
    #axs[3].invert_xaxis()

    axs[0].spines['top'].set_visible(True)
    axs[1].spines['top'].set_visible(True)
    axs[2].spines['top'].set_visible(True)
    axs[3].spines['top'].set_visible(True)
    axs[0].spines['right'].set_visible(True)
    axs[1].spines['right'].set_visible(True)
    axs[2].spines['right'].set_visible(True)
    axs[3].spines['right'].set_visible(True)
    
    plt.subplots_adjust(wspace=0)
    
    if paddle_occupancy and show_cbar:
        cbar_ax = fig.add_axes([0.9, 0.0, 0.05, 1.0])
        cbar_ax.set_axis_off()
        sm = cm.ScalarMappable(cmap=cmap, norm=matplotlib.colors.Normalize())
        sm.set_array([0, 1])
        ax = plt.sca(cbar_ax)
        plt.colorbar(sm, ax=cbar_ax, label='Relative occupancy')
        fig.suptitle(title, x=0.9)
    return fig, axs

###############################################

def unroll_cor(paddle_occupancy = {},
               cmap             = matplotlib.colormaps['hot'],
               show_cbar        = True):
    """
    Project the cortina on xz and yz as well 
    as add the 'edge' paddles.

    While this plot can show the occupancy of TOF paddles,
    it can also be 'hijacked' to just highlight certain
    paddles.
    
    # Keyword Arguments:
        paddle_occupancy : The number of events per paddle
        cmap             : Colormap - can be lambda function
                           to return color value based on 
                           'occupancy' numbker
        show_cbar        : Show the colorbar on the figure
    """
    fig, axs  = plt.subplots(1, 4, sharey=True, figsize=(22, 5), gridspec_kw={'width_ratios': [1, 1, 1, 1]})
    # normal +X
    cor_front = db.Paddle.objects.filter(panel_id=14) 
    # edge normal +X+Y
    ep_1      = db.Paddle.objects.filter(panel_id=18)
    # normal +Y
    cor_sb    = db.Paddle.objects.filter(panel_id=15)
    # endge normal -X+Y
    ep_2      = db.Paddle.objects.filter(panel_id=19)
    # normal -X 
    cor_back  = db.Paddle.objects.filter(panel_id=16) 
    # edge normal -X-Y
    ep_3      = db.Paddle.objects.filter(panel_id=20)
    # normal -Y 
    cor_bb    = db.Paddle.objects.filter(panel_id=17)
    # edge normal +X-Y
    ep_4      = db.Paddle.objects.filter(panel_id=21)

    xmin, xmax = -100,130
    ymin, ymax = -25,175
    title      = 'Relative occupancy, xy projection'
    
    for ep in ep_1:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[ep.paddle_id])
            axs[0].add_patch(ep.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[0].add_patch(ep.draw_yz(fill=True, edgecolor='k', facecolor='w'))

    for pdl in cor_front:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[0].add_patch(pdl.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[0].add_patch(pdl.draw_yz(fill=True, edgecolor='b', facecolor='w'))
    axs[0].set_xlabel('y [cm]', loc='right')
    axs[0].set_ylabel('z [cm]', loc='top')#, rotation=90)
    #axs[0].set_aspect('equal')
    axs[0].set_xlim(xmin, xmax)
    axs[0].set_ylim(ymin, ymax)
    axs[0].set_title('COR +X', loc='right')
    
    # +Y side
    for ep in ep_2:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[ep.paddle_id])
            axs[1].add_patch(ep.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[1].add_patch(ep.draw_xz(fill=True, edgecolor='k', facecolor='w'))

    for pdl in cor_sb:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[1].add_patch(pdl.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[1].add_patch(pdl.draw_xz(fill=True, edgecolor='b', facecolor='w'))
    axs[1].set_xlabel('x [cm]', loc='right')
    #axs[1].set_ylabel('z [cm]', loc='top')#, rotation=90)
    #axs[1].set_aspect('equal')
    axs[1].set_xlim(-1*xmax, -1*xmin)
    axs[1].set_ylim(ymin, ymax)
    axs[1].set_title('COR +Y', loc='right')
    axs[1].invert_xaxis()

    for ep in ep_3:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[ep.paddle_id])
            axs[2].add_patch(ep.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[2].add_patch(ep.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    
    for pdl in cor_back:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[2].add_patch(pdl.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[2].add_patch(pdl.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    axs[2].set_xlabel('y [cm]', loc='right')
    #axs[2].set_ylabel('z [cm]', loc='top')#, rotation=90)
    axs[2].set_xlim(-1*xmax, -1*xmin)
    axs[2].set_ylim(ymin, ymax)
    #axs[2].set_aspect('equal')
    axs[2].invert_xaxis()
    axs[2].set_title('COR -X', loc='right')
    
    # -Y side
    for ep in ep_4:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[ep.paddle_id])
            axs[3].add_patch(ep.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[3].add_patch(ep.draw_xz(fill=True, edgecolor='k', facecolor='w'))

    for pdl in cor_bb:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[3].add_patch(pdl.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[3].add_patch(pdl.draw_xz(fill=True, edgecolor='b', facecolor='w'))
    
    axs[3].set_xlabel('x [cm]', loc='right')
    #axs[3].set_ylabel('z [cm]', loc='top')#, rotation=90)
    #axs[3].set_aspect('equal')
    axs[3].set_xlim(xmin, xmax)
    axs[3].set_ylim(ymin, ymax)
    axs[3].set_title('COR +Y', loc='right')
    #axs[3].invert_xaxis()

    axs[0].spines['top'].set_visible(True)
    axs[1].spines['top'].set_visible(True)
    axs[2].spines['top'].set_visible(True)
    axs[3].spines['top'].set_visible(True)
    axs[0].spines['right'].set_visible(True)
    axs[1].spines['right'].set_visible(True)
    axs[2].spines['right'].set_visible(True)
    axs[3].spines['right'].set_visible(True)
    
    plt.subplots_adjust(wspace=0)

    if paddle_occupancy and show_cbar:
        cbar_ax = fig.add_axes([0.9, 0.0, 0.05, 1.0])
        cbar_ax.set_axis_off()
        sm = cm.ScalarMappable(cmap=cmap, norm=matplotlib.colors.Normalize())
        sm.set_array([0, 1])
        ax = plt.sca(cbar_ax)
        plt.colorbar(sm, ax=cbar_ax, label='Relative occupancy')
        fig.suptitle(title, x=0.9)
    return fig, axs

###############################################

def tof_2dproj(paddle_occupancy = {},
               cmap             = matplotlib.colormaps['hot'],
               show_cbar        = True):
    """
    Project the whole TOF on the 2d plane in a meaningful 
    way. 

    While this plot can show the occupancy of TOF paddles,
    it can also be 'hijacked' to just highlight certain
    paddles.
    
    # Keyword Arguments:
        paddle_occupancy : The number of events per paddle
        cmap             : Colormap - can be lambda function
                           to return color value based on 
                           'occupancy' numbker
        show_cbar        : Show the colorbar on the figure
    """
    fig, axes = plt.subplots(2, 4, sharey=True,figsize=(22, 5), gridspec_kw={'width_ratios': [1, 1, 1, 1]})
    # normal +X
    cbe_front = db.Paddle.objects.filter(panel_id=3) 
    # edge normal +X+Y
    ep_1      = db.Paddle.objects.filter(paddle_id=57)
    # normal +Y
    cbe_sb    = db.Paddle.objects.filter(panel_id=4)
    # endge normal -X+Y
    ep_2      = db.Paddle.objects.filter(paddle_id=58)
    # normal -X 
    cbe_back  = db.Paddle.objects.filter(panel_id=5) 
    # edge normal -X-Y
    ep_3      = db.Paddle.objects.filter(paddle_id=59)
    # normal -Y 
    cbe_bb    = db.Paddle.objects.filter(panel_id=6)
    # edge normal +X-Y
    ep_4      = db.Paddle.objects.filter(paddle_id=60)
    
    ### CORTINA
    # normal +X
    cor_front = db.Paddle.objects.filter(panel_id=14) 
    # edge normal +X+Y
    epc_1      = db.Paddle.objects.filter(panel_id=18)
    # normal +Y
    cor_sb    = db.Paddle.objects.filter(panel_id=15)
    # endge normal -X+Y
    epc_2      = db.Paddle.objects.filter(panel_id=19)
    # normal -X 
    cor_back  = db.Paddle.objects.filter(panel_id=16) 
    # edge normal -X-Y
    epc_3      = db.Paddle.objects.filter(panel_id=20)
    # normal -Y 
    cor_bb    = db.Paddle.objects.filter(panel_id=17)
    # edge normal +X-Y
    epc_4      = db.Paddle.objects.filter(panel_id=21)

    xmin, xmax = -110,110
    ymin, ymax = -110,110
    zmin, zmax = -25, 120
    title      = 'Relative occupancy, xy projection'
    
    axs = axes[0]
    ep = ep_1[0]
    if paddle_occupancy:
        color = cmap(paddle_occupancy[ep.paddle_id])
        axs[0].add_patch(ep.draw_yz(fill=True, edgecolor=color, facecolor=color))
    else:
        axs[0].add_patch(ep.draw_yz(fill=True, edgecolor='k', facecolor='w'))

    for pdl in cbe_front:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[0].add_patch(pdl.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[0].add_patch(pdl.draw_yz(fill=True, edgecolor='b', facecolor='w'))
    axs[0].set_xlabel('y [cm]', loc='right')
    axs[0].set_ylabel('z [cm]', loc='top')#, rotation=90)
    axs[0].set_aspect('equal')
    axs[0].set_xlim(-80, 90)
    axs[0].set_ylim(-10, 120)
    axs[0].set_title('CBE +X', loc='right')
    
    # +Y side
    ep = ep_2[0]
    if paddle_occupancy:
        color = cmap(paddle_occupancy[ep.paddle_id])
        axs[1].add_patch(ep.draw_xz(fill=True, edgecolor=color, facecolor=color))
    else:
        axs[1].add_patch(ep.draw_xz(fill=True, edgecolor='k', facecolor='w'))

    for pdl in cbe_sb:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[1].add_patch(pdl.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[1].add_patch(pdl.draw_xz(fill=True, edgecolor='b', facecolor='w'))
    axs[1].set_xlabel('x [cm]', loc='right')
    #axs[1].set_ylabel('z [cm]', loc='top')#, rotation=90)
    axs[1].set_aspect('equal')
    axs[1].set_xlim(-90, 80)
    axs[1].set_ylim(-10, 120)
    axs[1].set_title('CBE +Y', loc='right')
    axs[1].invert_xaxis()

    ep = ep_3[0]
    if paddle_occupancy:
        color = cmap(paddle_occupancy[ep.paddle_id])
        axs[2].add_patch(ep.draw_yz(fill=True, edgecolor=color, facecolor=color))
    else:
        axs[2].add_patch(ep.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    for pdl in cbe_back:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[2].add_patch(pdl.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[2].add_patch(pdl.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    axs[2].set_xlabel('y [cm]', loc='right')
    #axs[2].set_ylabel('z [cm]', loc='top')#, rotation=90)
    axs[2].set_xlim(-90, 80)
    axs[2].set_ylim(-10, 120)
    axs[2].set_aspect('equal')
    axs[2].invert_xaxis()
    axs[2].set_title('CBE -X', loc='right')
    
    # -Y side
    ep = ep_4[0]
    if paddle_occupancy:
        color = cmap(paddle_occupancy[ep.paddle_id])
        axs[3].add_patch(ep.draw_xz(fill=True, edgecolor=color, facecolor=color))
    else:
        axs[3].add_patch(ep.draw_xz(fill=True, edgecolor='k', facecolor='w'))

    for pdl in cbe_bb:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[3].add_patch(pdl.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[3].add_patch(pdl.draw_xz(fill=True, edgecolor='b', facecolor='w'))
    axs[3].set_xlabel('x [cm]', loc='right')
    #axs[3].set_ylabel('z [cm]', loc='top')#, rotation=90)
    axs[3].set_aspect('equal')
    axs[3].set_xlim(-80, 90)
    axs[3].set_ylim(-10, 120)
    axs[3].set_title('CBE +Y', loc='right')
    #axs[3].invert_xaxis()

    axs[0].spines['top'].set_visible(True)
    axs[1].spines['top'].set_visible(True)
    axs[2].spines['top'].set_visible(True)
    axs[3].spines['top'].set_visible(True)
    axs[0].spines['right'].set_visible(True)
    axs[1].spines['right'].set_visible(True)
    axs[2].spines['right'].set_visible(True)
    axs[3].spines['right'].set_visible(True)
    
    ### CORTINA
    xmin, xmax = -100,130
    ymin, ymax = -25,175
    title      = 'Relative occupancy, xy projection'
    
    axs = axes[1]
    for ep in epc_1:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[ep.paddle_id])
            axs[0].add_patch(ep.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[0].add_patch(ep.draw_yz(fill=True, edgecolor='k', facecolor='w'))

    for pdl in cor_front:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[0].add_patch(pdl.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[0].add_patch(pdl.draw_yz(fill=True, edgecolor='b', facecolor='w'))
    axs[0].set_xlabel('y [cm]', loc='right')
    axs[0].set_ylabel('z [cm]', loc='top')#, rotation=90)
    #axs[0].set_aspect('equal')
    axs[0].set_xlim(xmin, xmax)
    axs[0].set_ylim(ymin, ymax)
    axs[0].set_title('COR +X', loc='right')
    
    # +Y side
    for ep in epc_2:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[ep.paddle_id])
            axs[1].add_patch(ep.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[1].add_patch(ep.draw_xz(fill=True, edgecolor='k', facecolor='w'))

    for pdl in cor_sb:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[1].add_patch(pdl.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[1].add_patch(pdl.draw_xz(fill=True, edgecolor='b', facecolor='w'))
    axs[1].set_xlabel('x [cm]', loc='right')
    #axs[1].set_ylabel('z [cm]', loc='top')#, rotation=90)
    #axs[1].set_aspect('equal')
    axs[1].set_xlim(-1*xmax, -1*xmin)
    axs[1].set_ylim(ymin, ymax)
    axs[1].set_title('COR +Y', loc='right')
    axs[1].invert_xaxis()

    for ep in epc_3:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[ep.paddle_id])
            axs[2].add_patch(ep.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[2].add_patch(ep.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    
    for pdl in cor_back:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[2].add_patch(pdl.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[2].add_patch(pdl.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    axs[2].set_xlabel('y [cm]', loc='right')
    #axs[2].set_ylabel('z [cm]', loc='top')#, rotation=90)
    axs[2].set_xlim(-1*xmax, -1*xmin)
    axs[2].set_ylim(ymin, ymax)
    #axs[2].set_aspect('equal')
    axs[2].invert_xaxis()
    axs[2].set_title('COR -X', loc='right')
    
    # -Y side
    for ep in epc_4:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[ep.paddle_id])
            axs[3].add_patch(ep.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[3].add_patch(ep.draw_xz(fill=True, edgecolor='k', facecolor='w'))

    for pdl in cor_bb:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            axs[3].add_patch(pdl.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            axs[3].add_patch(pdl.draw_xz(fill=True, edgecolor='b', facecolor='w'))
    
    axs[3].set_xlabel('x [cm]', loc='right')
    #axs[3].set_ylabel('z [cm]', loc='top')#, rotation=90)
    #axs[3].set_aspect('equal')
    axs[3].set_xlim(xmin, xmax)
    axs[3].set_ylim(ymin, ymax)
    axs[3].set_title('COR +Y', loc='right')
    #axs[3].invert_xaxis()
    for axs in axes:
        axs[0].spines['top'].set_visible(True)
        axs[1].spines['top'].set_visible(True)
        axs[2].spines['top'].set_visible(True)
        axs[3].spines['top'].set_visible(True)
        axs[0].spines['right'].set_visible(True)
        axs[1].spines['right'].set_visible(True)
        axs[2].spines['right'].set_visible(True)
        axs[3].spines['right'].set_visible(True)
    
    plt.subplots_adjust(wspace=0)

    if paddle_occupancy and show_cbar:
        cbar_ax = fig.add_axes([0.9, 0.0, 0.05, 1.0])
        cbar_ax.set_axis_off()
        sm = cm.ScalarMappable(cmap=cmap, norm=matplotlib.colors.Normalize())
        sm.set_array([0, 1])
        ax = plt.sca(cbar_ax)
        plt.colorbar(sm, ax=cbar_ax, label='Relative occupancy')
        fig.suptitle(title, x=0.9)
    return fig, axs

###############################################

def plot_rb_paddles(rb):
    """ 
    Create the TOF projection plots and 
    mark the respective panels corresponding 
    to the ReadoutBoard
    
    # Arguments:
        rb : go.db.ReadoutBoard
    """
    poc = {k : 0 for k in range(161)}
    for pid in rb.pids:
        poc[pid] = 1
    cmap = lambda x: 'royalblue' if x == 1 else 'lightsteelblue'
    fig1,axs1 = tof_projection_xy(paddle_occupancy = poc,
                                  cmap = cmap,
                                  show_cbar = False)
    fig2,axs2 = unroll_cbe_sides(paddle_occupancy = poc,
                                 cmap = cmap,
                                 show_cbar = False)
    fig3,axs3 = unroll_cor(paddle_occupancy = poc,
                           cmap = cmap,
                           show_cbar = False)
    return (fig1,fig2,fig3),(axs1,axs2,axs3)

###############################################

def plot_ltb_threshold_timeseries(times, ltb_hk, savename = None):
    """
    The LTB thresholds over the given times.

    # Arguments: 
        * times    : mission elapsed time
        * ltb_hk   : a list of LTBMoniData 

    # Keyword Arguments:
        * savename : filename of the plot to save 
    """
    board_ids = [k.board_id for k in ltb_hk]
    if not len(ltb_hk):
        raise ValueError('Given list for ltb housekeeping data is empty!')
    if not len(set(board_ids)) == 1:
        raise ValueError('It seems the data contains LTB connected to different RBs!')
    board_id = board_ids[0]
    thresh0  = [k.thresh0 for k in ltb_hk]
    thresh1  = [k.thresh1 for k in ltb_hk]
    thresh2  = [k.thresh2 for k in ltb_hk]
    labels   = ['HIT', 'BETA', 'VETO']
    fig = timeseries_multiplot(times,
                               [thresh0, thresh1, thresh2],
                               labels,
                               title  = f'LTB Threasholds Board {board_id}',
                               xlabel = 'MET [hours] (gcu)',
                               ylabel = 'mV',
                               savename = savename)
    return fig

###############################################


def plot_paddle_charge2d(reader      = None,\
                         charge_a    = [],\
                         charge_b    = [],\
                         paddle_id   = 0,\
                         charge_bins = np.linspace(0,100,70),
                         plot_dir    = None):
    """
    Plot the paddle charge (from the TofHits) in a symmetric 
    2d histogram
    """
    if reader is not None:
        raise NotImplementedError("Feature not yet implemented! Use charge_a, charge_b instead")
    fig = plt.figure(figsize=lo.FIGSIZE_A4_SQUARE)
    ax  = fig.gca()
    h   = d.factory.hist2d((charge_a,charge_b), (charge_bins, charge_bins))
    h.imshow(cmap=matplotlib.colormaps['coolwarm'])
    ax.set_ylim(bottom=0)
    ax.set_xlim(left=0)
    ax.set_title(f'Charge A vs B Paddle {paddle_id}', loc='right')
    #ax.set_yscale('symlog')
    ax.set_ylabel('Charge (B) in pC', loc='top')
    ax.set_xlabel('Charge (A) in pC', loc='right')
    ax.spines['top'].set_visible(True)
    ax.spines['right'].set_visible(True)
    if plot_dir is not None:
        fig.savefig(f'{plot_dir}/charge_a_vs_b_{k}.webp')
    return fig

###############################################

def mtb_rate_plot(reader            = None,
                  mtbmonidata       = [],
                  use_gcutime       = False,
                  mtb_moni_interval = 10,
                  plot_dir          = None):
    """

    # Arguments

        * reader
        
    """
    if reader is not None and mtbmonidata:
        raise ValueError("Giving a reader and a list of MTBMoniData is confusng, since we don't know which one to use!")
    
    xlabel = 'MET [s] (gcu)'
    
    if not use_gcutime:
        print(f'Will plot {len(mtbmonidata)} MrbMoniData rates')
        met = np.array(range(len(mtbmonidata)))*mtb_moni_interval/3600 + mtb_moni_interval
        xlabel  = 'MET [h] (monitime)'
        rates   = np.array([k.rate for k in mtbmonidata])
        l_rates = np.array([k.lost_rate for k in mtbmonidata])
        
    fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT)
    ax  = fig.gca()
    ax.set_ylabel('Hz', loc='top')
    ax.set_xlabel(xlabel, loc='right')#, loc='right')

    #times   = np.array([j[0] for j in mtbmoni])
    #times  -= times[0]
    #rates   = np.array([j[1].rate for j in mtbmoni])
    #l_rates = np.array([j[1].lost_rate for j in mtbmoni])
    #print (times[l_rates<500][-1])
    #print(f'-> Avg MTB rate {rates.mean():4.2f}')
    #print(f'-> Avg Lost rate {l_rates.mean():4.2f}')
    ax.plot(met, rates,   lw=0.8, alpha=0.7, label=f'rate (avg {int(rates.mean())} Hz)', color='tab:blue')
    ax.plot(met, l_rates, lw=0.8, alpha=0.7, label=f'lost rate (avg {int(l_rates.mean())} Hz)', color='tab:red')
    ax.legend(loc='upper right', frameon=False,\
              ncol=2, bbox_to_anchor=(0.65,1.05),\
              bbox_transform=fig.transFigure, 
              fontsize=8)
    ax.set_title(f'MTB rates', loc='right')
    cb.visual.adjust_minor_ticks(ax, which='both')
    if plot_dir is not None:
        fig.savefig(f'{plot_dir}/mtb_rates.webp')
    return fig

###############################################

def plot_hg_lg_hits(reader   = None,
                    events   = [],
                    plot_dir = None,
                    split_by_threshold = False):
    """
    Plot the HG vs the LG (trigger) hits
    """
    if reader is not None and events:
        raise ValueError("If reader and events are both given, we don't know which one to use!")
    
    if reader:
        pass 
    if events:
        hits = [(len(ev.hits),\
                 len(ev.trigger_hits),\
                 len(ev.rb_link_ids),\
                 ev.rb_link_ids) for ev in tqdm.tqdm(events, desc='Getting hits...')]

    #print(f'-> We found {len(nthits)} LG and {len(nhits)} HG hits!'
    no_hitmissing    = 0
    one_hitmissing   = 0
    lttwo_hitmissing = 0
    for k in hits:
        if k[0] == k[1]:
            no_hitmissing += 1
        elif abs(k[0] - k[1]) == 1:
            one_hitmissing += 1
        else:
            lttwo_hitmissing += 1
            
    textbox  = f'NHits : {len(hits):.2e}\n'
    textbox += f'{100*no_hitmissing/len(hits):.2f} \% for N(LG) == N(HG)\n'
    textbox += f'{100*one_hitmissing/len(hits):.2f}\% for abs(N(LG) - N(HG)) = 1\n'
    textbox += f'{100*lttwo_hitmissing/len(hits):.2f}\% for abs(N(LG) - N(HG)) $>=$ 2'
    fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE)
    ax  = plt.gca()
    nhits        = [k[0] for k in hits]
    nthits       = np.array([k[1] for k in hits])
    rblinkids    = np.array([k[2] for k in hits])
    all_expected = nthits + rblinkids
    h   = d.factory.hist1d(nhits, np.arange(-0.5,30.5,1))
    h2  = d.factory.hist1d(nthits, np.arange(-0.5,30.5,1))
    h3  = d.factory.hist1d(rblinkids, np.arange(-0.5,30.5,1))
    h.line(filled=True, alpha=0.7, color='tab:blue', label='HG')
    h2.line(color='tab:blue', label='LG')
    h3.line(color='tab:red', label='RB LINK ID')
    ax.set_yscale('log')
    ax.set_xlabel('TOF hits', loc='right')
    ax.set_ylabel('events', loc='top')
    ax.set_title('TOF HG (readout) vs LG (data) hits', loc='right')
    ax.text(0.5, 0.7, textbox, transform=fig.transFigure, fontsize=10)
    ax.legend(frameon=False, fontsize=8, ncol=3, bbox_to_anchor=(0.45,1.01),\
              bbox_transform=fig.transFigure)
    return fig, hits

###############################################

def eventbld_hb_plots(reader = None, 
                      heartbeats = []):
    """
    Plot the relevant quantities from the event
    builder heartbeats
    """
    pass
