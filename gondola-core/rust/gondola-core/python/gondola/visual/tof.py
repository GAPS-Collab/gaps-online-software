"""
TOF visualizations 
"""

from .. import _gondola_core as _gc 

import matplotlib
import matplotlib.cm as cm
import matplotlib.pyplot as plt
import charmingbeauty as cb
import charmingbeauty.layout as lo
import numpy as np
import dashi
dashi.visual()

def plot_hg_lg_hits(h_nhits            : dashi.histogram.hist1d,
                    h_nthits           : dashi.histogram.hist1d,
                    n_events           : int,
                    no_hitmissing      = None,
                    one_hitmissing     = None,
                    lttwo_hitmissing   = None,
                    extra_hits         = None,
                    markercolor        = "w") -> (plt.Figure, plt.Figure):
    """
    Plot the HG vs the LG (trigger) hits with extra annotations.
    The nhit histograms and missing hit data has to be obtained
    previously.

    # Returns:
      Two figures, the actual histogram and the ratio plot 

    # Arguments:

      * h_nhits        : histogram of number of HG hits 
      * h_nthits       : histogram of number of LG hits
                         (hits with come from the trigger system
                            = 'Trigger hits')
      * n_events       : The number of events from which the 
                         hits were obtained
      * no_hitmissing  : The number of events which had zero
                         hits missing
      * one_hitmissing : The number of events which had one HG 
                         hit. missing 
      * two_hitmissing : The number of events which had two HG 
                         hits missing 
     """
    textbox = ''
    if n_events is not None:
        textbox  = f'NHits : {n_events:.2e}\n'
    if no_hitmissing is not None:
        textbox += f'{100*no_hitmissing/n_events:.2f}' + r'\%' + f' for N(LG) == N(HG)\n'
    if one_hitmissing is not None:
        textbox += f'{100*one_hitmissing/n_events:.2f}' + r'\%' + f' for N(LG) - N(HG) == 1\n'
    if lttwo_hitmissing is not None:
        textbox += f'{100*lttwo_hitmissing/n_events:.2f}' + r'\%' + f' for N(LG) - N(HG) $>=$ 2\n'
    if extra_hits is not None:
        textbox += f'{100*extra_hits/n_events:.2f}' + r'\%' + f' with N(HG) $>$ N(LG)\n'
    
    fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE)
    ax  = plt.gca()
    h_nhits .line(filled=True, alpha=0.7, color='tab:blue', label='HG')
    h_nthits.line(color='tab:red', label='LG')
    ax.set_yscale('log')
    ax.set_xlabel('TOF hits', loc='right')
    ax.set_ylabel('events', loc='top')
    ax.set_title('TOF HG (readout) vs LG (trigger) hits', loc='right')
    ax.text(0.5, 0.6, textbox, transform=fig.transFigure, fontsize=10)
    ax.legend(frameon=False, fontsize=8, ncol=3, bbox_to_anchor=(0.45,1.01),\
              bbox_transform=fig.transFigure)
    ax.set_xlim(left=-1, right=25) 
    fig_ratio = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT)
    ax_ratio  = fig_ratio.gca()
    ax_ratio.set_xlabel('TOF hits', loc='right')
    ax_ratio.set_ylabel('ratio HG/LG', loc='top')
    ax_ratio.set_title('TOF HG (readout) / LG (trigger) hits', loc='right')
    ratio = dashi.histfuncs.histratio(h_nhits, h_nthits,\
                                  log=False, ylabel="HG/LG")
    ratio.scatter(color=markercolor, marker="o", markersize=3)
    ax_ratio.set_xlim(left=-1, right=25) 
    return (fig, fig_ratio)
    
#h_nrblnk.line(color='tab:red', label='RB LINK ID')

###############################################

def tof_2dproj(event            = None,
               cmap             = matplotlib.colormaps['seismic'],
               paddle_style     = {'edgecolor' : 'w', 'lw' : 0.4},
               show_cbar        = True,
               no_ax_no_ticks   = False,
               cs_is_energy     = False,
               cnorm_max        = None) -> list:
    """
    Plots the entire TOF system in 2d projection, that is all panels 
    overlaid on each other

    # Keyword Arguments:
        event          : A TofEvent. If not None, then the hits will be shown
                         on top of the 2d projections 
        cs_is_energy   : Use the colorscale for energy instead of timing
        no_ax_no_ticks : Don't show any axis or axis ticks for a plain view 

    # Returns:
        list of figures, xy, xz, xy projections
    """
    projection_figures = []

    fig = plt.figure(figsize=lo.FIGSIZE_A4_SQUARE)
    ax = fig.gca()
    paddles         = _gc.db.TofPaddle.all()
    title           = 'XY projection'
    if event is not None:
        event.normalize_hit_times()
        ts = np.array([h.t0 for h in event.hits])
        if len(ts) > 0:
            cm_norm_pts = plt.Normalize(vmin=min(ts), vmax=max(ts))
            if cs_is_energy:
                edeps = np.array([h.edep for h in event.hits])
                if cnorm_max is None:
                    cm_norm_pts = plt.Normalize(vmin=min(edeps), vmax=max(edeps))
                else:
                    cm_norm_pts = plt.Normalize(vmin=0, vmax=cnorm_max)
    for pdl in paddles:
        if event is not None:
            ax.add_patch(pdl.draw_xy(fill=False,\
                                     edgecolor=paddle_style['edgecolor'],
                                     lw=paddle_style['lw'],
                                     facecolor='tab:blue'))#, alpha=0.3))
        else:
            ax.add_patch(pdl.draw_xy(fill=True, edgecolor='k', facecolor='w'))
    if event is not None:
        for h in event.hits:
            if cs_is_energy:
                ax.scatter([0.1*h.x], [0.1*h.y], alpha = 0.8 , marker='o', s=100*h.edep,
                           lw=1.5, edgecolor=paddle_style['edgecolor'], color=cmap(cm_norm_pts(h.edep)))
            else:
                ax.scatter([0.1*h.x], [0.1*h.y], alpha = 0.8 , marker='o', s=100*h.edep,
                           lw=1.5, edgecolor=paddle_style['edgecolor'], color=cmap(cm_norm_pts(h.t0)))
    ax.grid(0) 
    ax.set_xlabel('x [cm]', loc='right')
    ax.set_ylabel('y [cm]', loc='top')#, rotation=90)
    ax.set_aspect('equal')
    ax.set_xlim(-200, 200)
    ax.set_ylim(-200, 200)
    ax.set_title(title, loc='right')
    if no_ax_no_ticks:
        ax.set_axis_off()
    else:
        ax.spines['top'].set_visible(True)
        ax.spines['right'].set_visible(True)
    projection_figures.append(fig)
    
    # XZ projection
    fig = plt.figure(figsize=lo.FIGSIZE_A4_SQUARE)
    ax = fig.gca()
    title           = 'XZ projection'
    if event is not None:
        ts = np.array([h.t0 for h in event.hits])
        if len(ts) > 0:
            cm_norm_pts = plt.Normalize(vmin=min(ts), vmax=max(ts))

    for pdl in paddles:
        if event is not None:
            ax.add_patch(pdl.draw_xz(fill=False,\
                                     edgecolor=paddle_style['edgecolor'],
                                     lw=paddle_style['lw'],
                                     facecolor='tab:blue'))#, alpha=0.3))
        else:
            ax.add_patch(pdl.draw_xz(fill=True, edgecolor='k', facecolor='w'))
    if event is not None:
        for h in event.hits:
            if cs_is_energy:
                ax.scatter([0.1*h.x], [0.1*h.z], alpha = 0.8 , marker='o', s=100*h.edep,
                        lw=1.5, edgecolor=paddle_style['edgecolor'], color=cmap(cm_norm_pts(h.edep)))
            else:
                ax.scatter([0.1*h.x], [0.1*h.z], alpha = 0.8 , marker='o', s=100*h.edep,
                        lw=1.5, edgecolor=paddle_style['edgecolor'], color=cmap(cm_norm_pts(h.t0)))
    ax.grid(0) 
    ax.set_xlabel('x [cm]', loc='right')
    ax.set_ylabel('z [cm]', loc='top')#, rotation=90)
    ax.set_aspect('equal')
    ax.set_xlim(-200, 200)
    ax.set_ylim(-25, 250)
    ax.set_title(title, loc='right')

    if no_ax_no_ticks:
        ax.set_axis_off()
    else:
        ax.spines['top'].set_visible(True)
        ax.spines['right'].set_visible(True)
    if show_cbar:
        cbar_ax = fig.add_axes([0.9, 0.0, 0.05, 1.0])
        cbar_ax.set_axis_off()
        sm = cm.ScalarMappable(cmap=cmap, norm=matplotlib.colors.Normalize())
        #sm.set_array([0, 1])
        #ax = plt.sca(cbar_ax)
        if cs_is_energy:
            plt.colorbar(sm, ax=cbar_ax, label='Energy Dep. [MeV]')
        else:
            plt.colorbar(sm, ax=cbar_ax, label='Timing [ns]')
        fig.sca(ax)
    projection_figures.append(fig)

    # YZ projection
    fig = plt.figure(figsize=lo.FIGSIZE_A4_SQUARE)
    ax  = fig.gca()
    title           = 'XY projection'
    if event is not None:
        ts = np.array([h.t0 for h in event.hits])
        if len(ts) > 0:
            cm_norm_pts = plt.Normalize(vmin=min(ts), vmax=max(ts))

    for pdl in paddles:
        if event is not None:
            ax.add_patch(pdl.draw_yz(fill=False,\
                                     edgecolor=paddle_style['edgecolor'],
                                     lw=paddle_style['lw'],
                                     facecolor='tab:blue'))#, alpha=0.3))
        else:
            ax.add_patch(pdl.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    if event is not None:
        for h in event.hits:
            if cs_is_energy:
                ax.scatter([0.1*h.y], [0.1*h.z], alpha = 0.8 , marker='o', s=100*h.edep,
                           lw=1.5, edgecolor=paddle_style['edgecolor'], color=cmap(cm_norm_pts(h.edep)))
            else:
                ax.scatter([0.1*h.y], [0.1*h.z], alpha = 0.8 , marker='o', s=100*h.edep,
                        lw=1.5, edgecolor=paddle_style['edgecolor'], color=cmap(cm_norm_pts(h.t0)))
    ax.grid(0) 
    ax.set_xlabel('y [cm]', loc='right')
    ax.set_ylabel('z [cm]', loc='top')#, rotation=90)
    ax.set_aspect('equal')
    ax.set_xlim(-200, 200)
    ax.set_ylim(-25, 250)
    ax.set_title(title, loc='right')
    if no_ax_no_ticks:
        ax.set_axis_off()
    else:
        ax.spines['top'].set_visible(True)
        ax.spines['right'].set_visible(True)
    projection_figures.append(fig)
    return projection_figures

#--------------------------------------------------------

def tof_hits_time_evolution(ev, line_color='k', t_err=0.35) -> plt.Figure: #, twindows=None):
    """
    A simple plot plotting normalized event
    times on the x axis and the energy deposition
    for each hit on the y axis.

    # Return:
        
    """
    fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT)
    ax  = fig.gca()
    ev.normalize_hit_times()
    hits = sorted([h for h in ev.hits], key=lambda x : x.event_t0)
    if len(hits) == 0:
        return fig
    # the first hit
    first_hit = hits[0]
    ax.vlines(first_hit.event_t0,0, first_hit.edep, color=line_color)
    ax.fill_betweenx([0,first_hit.edep], first_hit.event_t0 - t_err, first_hit.event_t0 + t_err, color=line_color, alpha=0.2)
    prior_hit = first_hit
    for h in hits[1:]:
        # indicate lightspeed cleaning
        min_tdiff_cvac = 1e9*1e-3*prior_hit.distance(h)/299792458.0
        twindow        = prior_hit.event_t0 - t_err + min_tdiff_cvac;
        # lenient strategy
        ax.fill_betweenx([0,prior_hit.edep], prior_hit.event_t0, twindow, color='tab:red', alpha=0.4)
        # with an aggressive strategy, we could clean even more
        ax.fill_betweenx([0,prior_hit.edep], twindow, twindow + 2*t_err,
                         color='tab:blue',
                         alpha=0.2)
        ax.vlines(h.event_t0, 0, h.edep, color=line_color)
        ax.fill_betweenx([0, h.edep], h.event_t0 - t_err, h.event_t0 + t_err, color=line_color, alpha=0.2)
        prior_hit = h
    ax.set_xlabel('Event t0 [ns]', loc='right')
    ax.set_ylabel('Hit EDep', loc='top')
    ax.set_title('TOF Hitseries', loc='right')
    ax.set_ylim(bottom=0)
    cb.visual.adjust_minor_ticks(ax, which='both')
    return fig

#---------------------------------------------------------------------------------

def plot_waveforms(tof_ev, calib : dict = None, with_hits = False, skip_bins=0):
    """
    Return a list of figures with all the waveforms from a 
    specific tof event
    
    # Arguments:
        tof_ev    : TofEvent with waveforms
        calib     : A dictionary with RB calibrations
        with_hits : Indicate extracted hit time in the plots
        skip_bins : Zero the first [skip_bins]. This might be a 
                    helpful option in case there is a big spike 
                    in the beginning
    """
    wfs = tof_ev.waveforms
    figures, axes = [],[]
    hits = dict()
    for h in tof_ev.hits:
        hits[h.paddle_id] = h

    for wf in wfs:
        fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT)
        ax  = fig.gca()
        if calib is None:
            adc_a = wf.adc_a
            adc_b = wf.adc_b
            if skip_bins:
                for k in range(skip_bins):
                    adc_a[k] = 0
                    adc_b[k] = 0
            ax.plot(adc_a, color='tab:blue', lw=1.2, label=f'{wf.paddle_id} A')
            ax.plot(adc_b, color='tab:red', lw=1.2, label=f'{wf.paddle_id} B')
            ax.set_xlabel('bin', loc='right')
            ax.set_ylabel('ADC', loc='top')
            ax.legend(frameon=False, loc='upper right')
        else:
            wf.calibrate(calib[wf.rb_id])
            voltages_a = wf.voltages_a
            voltages_b = wf.voltages_b
            if skip_bins:
                for k in range(skip_bins):
                    voltages_a[k] = 0
                    voltages_b[k] = 0
            ax.plot(wf.times_a, voltages_a, lw=0.9, color='tab:blue', label=f'{wf.paddle_id} A')
            ax.plot(wf.times_b, voltages_b, lw=0.9, color='tab:red', label=f'{wf.paddle_id} B')
            ax.set_xlabel('ns', loc='right')
            ax.set_ylabel('mV', loc='top')
            ax.legend(frameon=False, loc='upper right')
        if with_hits:
            try:
                ax.vlines(hits[wf.paddle_id].time_a, 0, max(voltages_a), lw=0.75, color='tab:blue')
            except KeyError:
                textbox = 'Hit extr. failed!'
                ax.text(0.2, 0.8, textbox, transform=fig.transFigure, fontsize=8)
                print ('No hit for waveform!')
            try:
                ax.vlines(hits[wf.paddle_id].time_b, 0, max(voltages_b), lw=0.75, color='tab:red')
            except KeyError:
                textbox = 'Hit extr. failed!'
                ax.text(0.2, 0.8, textbox, transform=fig.transFigure, fontsize=8)
                print ('No hit for waveform!')
        figures.append(fig)
        axes.append(ax)

    return figures, axes

