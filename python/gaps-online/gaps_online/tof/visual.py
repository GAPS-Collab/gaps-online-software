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




