"""
Variable plot visualization
FIXME - might get moved around
"""

import matplotlib.pyplot as plt
import numpy as np

import charmingbeauty as cb
import charmingbeauty.layout as lo

import dashi as d
d.visual()

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




