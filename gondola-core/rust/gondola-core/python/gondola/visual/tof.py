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

#--------------------------------------------------------

def tof_projection_xy(paddle_occupancy = {}, 
                      event            = None,
                      cmap             = matplotlib.colormaps['hot'],
                      paddle_style     = {'edgecolor' : 'w', 'lw' : 0.4},
                      show_cbar        = True,
                      overlay_panels   = False,
                      indicate_empty   = 'gray',
                      umbrella_only    = False,
                      lognorm          = False):
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
        event            : Plot hits from TofEvent or TofEventSummary
        cmap             : Colormap - can be lambda function
                           to return color value based on 
                           'occupancy' numbker
        show_cbar        : Show the colorbar on the figure
        overlay_panels   : Only return one axes, have the TOF CBE bottom
                           and CBE TOP panels overlaid over the umbrella
                           (or under it)
        indicate_empty   : In case we are using this for paddle occupancy,
                           indicate empty paddles with the given color instead
                           using a value from the color map. If this behavior is 
                           not desired, set this to an empty string.
        umbrella_only    : Show only the umbrella, add the colorbar next to it
    """
    if overlay_panels:
        fig = plt.figure(figsize=lo.FIGSIZE_A4_SQUARE)
        axs = [fig.gca()]
    else:
        fig, axs        = plt.subplots(1, 3, figsize=(18, 5), gridspec_kw={'width_ratios': [1, 1, 1]})
    
    all_paddles     = _gc.db.TofPaddle.all() 
    umb_paddles     = [k for k in all_paddles if k.paddle_id > 60 and k.paddle_id < 109]
    cbe_top_paddles = [k for k in all_paddles if k.panel_id == 1] 
    cbe_bot_paddles = [k for k in all_paddles if k.panel_id == 2]
    xmin, xmax      = -100,100
    ymin, ymax      = -100,100
    zmin, zmax      = -25, 120
    title           = 'TOF UMB/CBE TOP/CBE BOT xy projection'
    if event is not None:
        ts = np.array([h.t0 for h in event.hits])
        cm_norm_pts = plt.Normalize(vmin=min(ts), vmax=max(ts))
    
    if paddle_occupancy:
        # FIXME - this is a kludge for the ICRC approved plot!
        umb_norm = matplotlib.colors.Normalize(vmin=min(paddle_occupancy.values()), vmax=max(paddle_occupancy.values()))
        if umbrella_only:
            umb_occu = {k : paddle_occupancy[k] for k in paddle_occupancy if 60 < k < 109}
            paddle_occupancy = umb_occu
            if lognorm:
                umb_norm = matplotlib.colors.LogNorm(vmin=min(umb_occu.values()), vmax=max(umb_occu.values()))
            else:
                umb_norm = matplotlib.colors.Normalize(vmin=min(umb_occu.values()), vmax=max(umb_occu.values()))
    for pdl in umb_paddles:
        if paddle_occupancy:
            max_occu = max(paddle_occupancy.values())
            #color = cmap(paddle_occupancy[pdl.paddle_id]/max_occu)
            color = cmap(umb_norm(paddle_occupancy[pdl.paddle_id]))
            if not paddle_occupancy[pdl.paddle_id] and indicate_empty:
                color = indicate_empty
            axs[0].add_patch(pdl.draw_xy(fill=True, edgecolor=color, facecolor=color))
            if pdl.panel_id == 7:
                axs[0].text(pdl.global_pos_x_l0 - 4.5, 75, f'{pdl.paddle_id}', color='w', fontsize=6)
            if pdl.panel_id == 10 or pdl.panel_id == 8:
                axs[0].text(pdl.global_pos_x_l0 - 4.5, 160, f'{pdl.paddle_id}', color='w', fontsize=6)
            if pdl.panel_id == 11 or pdl.panel_id == 13:
                axs[0].text(pdl.global_pos_x_l0 - 4.5, -170, f'{pdl.paddle_id}', color='w', fontsize=6)
            if pdl.panel_id == 9 or pdl.panel_id == 12:
                axs[0].text(-80, pdl.global_pos_y_l0 - 4.5, f'{pdl.paddle_id}', color='w', fontsize=6)


        else:
            if event is not None:
                axs[0].add_patch(pdl.draw_xy(fill=False,\
                                             edgecolor=paddle_style['edgecolor'],
                                             lw=paddle_style['lw'],
                                             facecolor='tab:blue'))#, alpha=0.3))
            else:
                axs[0].add_patch(pdl.draw_xy(fill=True, edgecolor='k', facecolor='w'))
        axs[0].set_xlabel('x [cm]', loc='right')
        axs[0].set_ylabel('y [cm]', loc='top')#, rotation=90)
        axs[0].set_aspect('equal')
        axs[0].set_xlim(-200, 200)
        axs[0].set_ylim(-200, 200)
        if overlay_panels:
            if umbrella_only:
                # this is a bad kludge. Sorry. 
                axs[0].set_title('TOF umbrella trigger hit occupancy', loc='right')
            else:
                axs[0].set_title('XY projection', loc='right')
        else:
            axs[0].set_title('TOF umbrella', loc='right')
    if event is not None:
        #print (hit_pids)
        umb_pids = [umbp.paddle_id for umbp in umb_paddles]
        for h in event.hits:
            if h.paddle_id in umb_pids:
        #for j, pdl_hit in enumerate(hit_pids):
            #if pdl_hit in umb_pids:
                #print (f"adding scatter for {0.1*h.x}, {0.1*h.y}")
                axs[0].scatter([0.1*h.x], [0.1*h.y], alpha = 0.8 , marker='o', s=100*h.edep,
                               lw=1.5, edgecolor=paddle_style['edgecolor'], color=cmap(cm_norm_pts(h.t0)))

    if umbrella_only:
        umb_occu = {k : paddle_occupancy[k] for k in paddle_occupancy if 60 < k < 109}
        if paddle_occupancy and show_cbar:
            cbar_ax = fig.add_axes([0.85, 0.0, 0.1, 1.0])
            cbar_ax.set_axis_off()
            sm = cm.ScalarMappable(cmap=cmap, norm=umb_norm)
            sm.set_array([min(umb_occu.values()), max(umb_occu.values())])
            ax = plt.sca(cbar_ax)
            plt.colorbar(sm, ax=cbar_ax, label='triggers', shrink=10.0, aspect=20)
        if not overlay_panels:
            fig.suptitle(title, x=0.9)
        axs[0].spines['top'].set_visible(True)
        axs[0].spines['right'].set_visible(True)

        return fig, axs
        pass 

    axid = 1
    if overlay_panels:
        axid = 0
    for pdl in cbe_top_paddles:   
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            if not paddle_occupancy[pdl.paddle_id] and indicate_empty:
                color = indicate_empty
            axs[axid].add_patch(pdl.draw_xy(fill=True, edgecolor=color, facecolor=color))
            axs[axid].text(pdl.global_pos_x_l0 - 4, 75, f'{pdl.paddle_id}', color='w', fontsize=9)
        else:
            if event is not None:
                axs[axid].add_patch(pdl.draw_xy(fill=False,\
                                             edgecolor=paddle_style['edgecolor'],
                                             lw=paddle_style['lw'],
                                             facecolor='tab:blue'))#, alpha=0.3))
            else:
                axs[axid].add_patch(pdl.draw_xy(fill=True, edgecolor='k', facecolor='w'))
        if not overlay_panels:
            axs[axid].set_xlabel('x [cm]', loc='right')
            axs[axid].set_ylabel('y [cm]', loc='top')#, rotation=90)
            axs[axid].set_xlim(-100, 100)
            axs[axid].set_ylim(-100, 100)
            axs[axid].set_aspect('equal')
            axs[axid].set_title('CBE TOP', loc='right')

    if event is not None:
        cbpdl_pids = [cbpdl.paddle_id for cbpdl in cbe_top_paddles]
        for h in event.hits:
            if h.paddle_id in cbpdl_pids:
                axs[axid].scatter([0.1*h.x], [0.1*h.y], alpha = 0.8 , marker='o', s=100*h.edep,
                                lw=1.5, edgecolor=paddle_style['edgecolor'], color=cmap(cm_norm_pts(h.t0)))

    axid = 2
    if overlay_panels:
        axid = 0
    for pdl in cbe_bot_paddles:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            if not paddle_occupancy[pdl.paddle_id] and indicate_empty:
                color = indicate_empty
            axs[axid].add_patch(pdl.draw_xy(fill=True, edgecolor=color, facecolor=color))
            axs[axid].text(pdl.global_pos_x_l0 - 4, 75, f'{pdl.paddle_id}', color='w', fontsize=9)
        
        else:
            if event is not None:
                axs[axid].add_patch(pdl.draw_xy(fill=False,\
                                             edgecolor=paddle_style['edgecolor'],
                                             lw=paddle_style['lw'],
                                             facecolor='tab:blue'))#, alpha=0.3))
            else:
                axs[axid].add_patch(pdl.draw_xy(fill=True, edgecolor='k', facecolor='w'))
        if not overlay_panels:
            axs[axid].set_xlabel('x [cm]', loc='right')
            axs[axid].set_ylabel('y [cm]', loc='top')#, rotation=270)
            axs[axid].set_xlim(-100, 100)
            axs[axid].set_ylim(-100, 100)
            axs[axid].set_aspect('equal')
            axs[axid].set_title('CBE BOT')
    if event is not None:
        cbpdl_pids = [cbpdl.paddle_id for cbpdl in cbe_bot_paddles]
        for h in event.hits:
            if h.paddle_id in cbpdl_pids:
                axs[axid].scatter([0.1*h.x], [0.1*h.y], alpha = 0.8 , marker='o', s=100*h.edep,
                               lw=1.5, edgecolor=paddle_style['edgecolor'], color=cmap(cm_norm_pts(h.t0)))
        
    axs[0].spines['top'].set_visible(True)
    axs[0].spines['right'].set_visible(True)
    if not overlay_panels:
        axs[1].spines['top'].set_visible(True)
        axs[2].spines['top'].set_visible(True)
        axs[1].spines['right'].set_visible(True)
        axs[2].spines['right'].set_visible(True)
    if paddle_occupancy and show_cbar:
        cbar_ax = fig.add_axes([0.9, 0.0, 0.05, 1.0])
        cbar_ax.set_axis_off()
        sm = cm.ScalarMappable(cmap=cmap, norm=matplotlib.colors.Normalize())
        sm.set_array([0, 1])
        ax = plt.sca(cbar_ax)
        plt.colorbar(sm, ax=cbar_ax, label='Relative occupancy')
    if not overlay_panels:
        fig.suptitle(title, x=0.9)
    return fig, axs

#--------------------------------------------------------

def unroll_cbe_sides(paddle_occupancy = {},
                     event            = None,
                     cmap             = matplotlib.colormaps['hot'],
                     paddle_style    = {'edgecolor' : 'w', 'lw' : 0.4},
                     show_cbar        = True,
                     indicate_empty   = 'gray'
                     ):
    """
    Project the sides of the cube on xz and yz as well 
    as add the 'edge' paddles.

    While this plot can show the occupancy of TOF paddles,
    it can also be 'hijacked' to just highlight certain
    paddles.
    
    # Keyword Arguments:
        paddle_occupancy : The number of events per paddle
        event            : Plot a tof event
        cmap             : Colormap - can be lambda function
                           to return color value based on 
                           'occupancy' numbker
        show_cbar        : Show the colorbar on the figure
        indicate_empty   : In case we are using this for paddle occupancy,
                           indicate empty paddles with the given color instead
                           using a value from the color map. If this behavior is 
                           not desired, set this to an empty string.
    """
    fig, axs  = plt.subplots(1, 4, sharey=True,figsize=(22, 5), gridspec_kw={'width_ratios': [1, 1, 1, 1]})
    all_paddles = _gc.db.TofPaddle.all() 
    # normal +X
    cbe_front = [ k for k in all_paddles if k.panel_id==3]
    # edge normal +X+Y
    ep_1      = [ k for k in all_paddles if k.paddle_id==57]
    # normal +Y
    cbe_sb    = [ k for k in all_paddles if k.panel_id==4]
    # endge normal -X+Y
    ep_2      = [ k for k in all_paddles if k.paddle_id==58]
    # normal -X 
    cbe_back  = [ k for k in all_paddles if k.panel_id==5]
    # edge normal -X-Y
    ep_3      = [ k for k in all_paddles if k.paddle_id==59]
    # normal -Y 
    cbe_bb    = [ k for k in all_paddles if k.panel_id==6]
    # edge normal +X-Y
    ep_4      = [ k for k in all_paddles if k.paddle_id==60]

    xmin, xmax = -110,110
    ymin, ymax = -110,110
    zmin, zmax = -25, 120
    title      = 'Relative occupancy, xy projection'
    
    if event is not None:
        # coordinates are in mm
        hit_pids = [h.paddle_id for h in event.hits]
        xs = 0.1*np.array([h.x for h in event.hits])
        ys = 0.1*np.array([h.y for h in event.hits])
        zs = 0.1*np.array([h.z for h in event.hits])
        ts = np.array([h.t0 for h in event.hits])
        en = np.array([h.edep for h in event.hits])
        cm_norm_pts = plt.Normalize(vmin=min(ts), vmax=max(ts))

    ep = ep_1[0]
    cbe_front.append(ep_1[0])
    for pdl in cbe_front:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            if not paddle_occupancy[pdl.paddle_id] and indicate_empty:
                color = indicate_empty
            axs[0].add_patch(pdl.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            if event is not None:
                axs[0].add_patch(pdl.draw_yz(fill=False, facecolor='tab:blue', **paddle_style))
            else:
                axs[0].add_patch(pdl.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    

    ##for pdl in cbe_front:
    ##    if paddle_occupancy:
    ##        color = cmap(paddle_occupancy[pdl.paddle_id])
    ##        axs[0].add_patch(pdl.draw_yz(fill=True, edgecolor=color, facecolor=color))
    ##    else:
    ##        if event is not None:
    ##            axs[0].add_patch(pdl.draw_yz(fill=False, facecolor='tab:blue', **paddle_style))
    ##        else:
    ##            axs[0].add_patch(pdl.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    
    if event is not None:
        #print (hit_pids)
        cbe_front_pids = [k.paddle_id for k in cbe_front]
        for h in event.hits:
        #for j, pdl_hit in enumerate(hit_pids):
            if h.paddle_id in cbe_front_pids:
                axs[0].scatter([0.1*h.y], [0.1*h.z], alpha = 0.8 , marker='o', s=100*h.edep,
                               lw=1.5, edgecolor='k', color=cmap(cm_norm_pts(h.t0)))
    axs[0].set_xlabel('y [cm]', loc='right')
    axs[0].set_ylabel('z [cm]', loc='top')#, rotation=90)
    axs[0].set_aspect('equal')
    axs[0].set_xlim(-80, 90)
    axs[0].set_ylim(-10, 120)
    axs[0].set_title('CBE +X', loc='right')
    
    # +Y side
    ep = ep_2[0]
    cbe_sb.append(ep_2[0])
    #if paddle_occupancy:
    #    color = cmap(paddle_occupancy[ep.paddle_id])
    #    if not paddle_occupancy[ep.paddle_id] and indicate_empty:
    #        color = indicate_empty
    #    axs[1].add_patch(ep.draw_xz(fill=True, edgecolor=color, facecolor=color))
    #else:
    #    if event is not None:
    #        axs[1].add_patch(ep.draw_xz(fill=False, facecolor='tab:blue', **paddle_style))
    #    else:
    #        axs[1].add_patch(ep.draw_xz(fill=True, edgecolor='k', facecolor='w'))

    for pdl in cbe_sb:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            if not paddle_occupancy[pdl.paddle_id] and indicate_empty:
                color = indicate_empty
            axs[1].add_patch(pdl.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            if event is not None:
                axs[1].add_patch(pdl.draw_xz(fill=False, facecolor='tab:blue', **paddle_style))
            else:
                axs[1].add_patch(pdl.draw_xz(fill=True, edgecolor='k', facecolor='w'))
   
    if event is not None:
        #print (hit_pids)
        for j, pdl_hit in enumerate(hit_pids):
            if pdl_hit in [umbp.paddle_id for umbp in cbe_sb]:
                axs[1].scatter([xs[j]], [ys[j]], alpha = 0.8 , marker='o', s=100*en[j],
                               lw=1.5, edgecolor='k', color=cmap(cm_norm_pts(ts[j])))
    axs[1].set_xlabel('x [cm]', loc='right')
    #axs[1].set_ylabel('z [cm]', loc='top')#, rotation=90)
    axs[1].set_aspect('equal')
    axs[1].set_xlim(-90, 80)
    axs[1].set_ylim(-10, 120)
    axs[1].set_title('CBE +Y', loc='right')
    axs[1].invert_xaxis()

    ep = ep_3[0]
    cbe_back.append(ep_3[0])
    #if paddle_occupancy:
    #    color = cmap(paddle_occupancy[ep.paddle_id])
    #    if not paddle_occupancy[ep.paddle_id] and indicate_empty:
    #        color = indicate_empty
    #    axs[2].add_patch(ep.draw_yz(fill=True, edgecolor=color, facecolor=color))
    #else:
    #    if event is not None:
    #        axs[2].add_patch(ep.draw_yz(fill=False, facecolor='tab:blue', **paddle_style))
    #    else:
    #        axs[2].add_patch(ep.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    for pdl in cbe_back:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            if not paddle_occupancy[pdl.paddle_id] and indicate_empty:
                color = indicate_empty
            axs[2].add_patch(pdl.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            if event is not None:
                axs[2].add_patch(pdl.draw_yz(fill=False, facecolor='tab:blue', **paddle_style))
            else:
                axs[2].add_patch(pdl.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    if event is not None:
        #print (hit_pids)
        for j, pdl_hit in enumerate(hit_pids):
            if pdl_hit in [umbp.paddle_id for umbp in cbe_back]:
                axs[2].scatter([ys[j]], [zs[j]], alpha = 0.8 , marker='o', s=100*en[j],
                               lw=1.5, edgecolor='k', color=cmap(cm_norm_pts(ts[j])))
    axs[2].set_xlabel('y [cm]', loc='right')
    #axs[2].set_ylabel('z [cm]', loc='top')#, rotation=90)
    axs[2].set_xlim(-90, 80)
    axs[2].set_ylim(-10, 120)
    axs[2].set_aspect('equal')
    axs[2].invert_xaxis()
    axs[2].set_title('CBE -X', loc='right')
    
    # -Y side
    ep = ep_4[0]
    cbe_bb.append(ep_4[0])
    #if paddle_occupancy:
    #    color = cmap(paddle_occupancy[ep.paddle_id])
    #    if not paddle_occupancy[ep.paddle_id] and indicate_empty:
    #        color = indicate_empty
    #    axs[3].add_patch(ep.draw_xz(fill=True, edgecolor=color, facecolor=color))
    #else:
    #    if event is not None:
    #        axs[3].add_patch(ep.draw_xz(fill=False, edgecolor='k', facecolor='tab:blue', alpha=0.3))
    #    else:
    #        axs[3].add_patch(ep.draw_xz(fill=True, edgecolor='k', facecolor='w'))
    for pdl in cbe_bb:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            if not paddle_occupancy[pdl.paddle_id] and indicate_empty:
                color = indicate_empty
            axs[3].add_patch(pdl.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            if event is not None:
                axs[3].add_patch(pdl.draw_xz(fill=False,facecolor='tab:blue', **paddle_style))
            else:
                axs[3].add_patch(pdl.draw_xz(fill=True, edgecolor='k', facecolor='w'))
    if event is not None:
        #print (hit_pids)
        for j, pdl_hit in enumerate(hit_pids):
            if pdl_hit in [umbp.paddle_id for umbp in cbe_bb]:
                axs[3].scatter([xs[j]], [zs[j]], alpha = 0.8 , marker='o', s=100*en[j],
                               lw=1.5, edgecolor='k', color=cmap(cm_norm_pts(ts[j])))

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

#--------------------------------------------------------

def unroll_cor(paddle_occupancy = {},
               event            = None,
               cmap             = matplotlib.colormaps['hot'],
               paddle_style     = {'edgecolor' : 'w', 'lw' : 0.4},
               show_cbar        = True,
               indicate_empty   = 'gray'):
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
        indicate_empty   : In case we are using this for paddle occupancy,
                           indicate empty paddles with the given color instead
                           using a value from the color map. If this behavior is 
                           not desired, set this to an empty string.
    """
    fig, axs  = plt.subplots(1, 4, sharey=True, figsize=(22, 5), gridspec_kw={'width_ratios': [1, 1, 1, 1]})
    all_paddles = _gc.db.TofPaddle.all()
    # normal +X
    cor_front = [k for k in all_paddles if k.panel_id==14]
    # edge normal +X+Y
    ep_1      = [k for k in all_paddles if k.panel_id==18]
    # normal +Y
    cor_sb    = [k for k in all_paddles if k.panel_id==15]
    # endge normal -X+Y
    ep_2      = [k for k in all_paddles if k.panel_id==19]
    # normal -X 
    cor_back  = [k for k in all_paddles if k.panel_id==16] 
    # edge normal -X-Y
    ep_3      = [k for k in all_paddles if k.panel_id==20]
    # normal -Y 
    cor_bb    = [k for k in all_paddles if k.panel_id==17]
    # edge normal +X-Y
    ep_4      = [k for k in all_paddles if k.panel_id==21]

    xmin, xmax = -100,130
    ymin, ymax = -25,175 # these are the z-coordinates
    title      = 'Relative occupancy, xy projection'
    
    if event is not None:
        # coordinates are in mm
        event_hits = sorted(event.hits, key=lambda x : x.t0)
        hit_pids = [h.paddle_id for h in event_hits]
        xs = 0.1*np.array([h.x for h in event_hits])
        ys = 0.1*np.array([h.y for h in event_hits])
        zs = 0.1*np.array([h.z for h in event_hits])
        ts = np.array([h.t0 for h in event_hits])
        en = np.array([h.edep for h in event_hits])
        cm_norm_pts = plt.Normalize(vmin=min(ts), vmax=max(ts))
    
    for ep in ep_1:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[ep.paddle_id])
            if not paddle_occupancy[ep.paddle_id] and indicate_empty:
                color = indicate_empty
            axs[0].add_patch(ep.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            if event is not None:
                axs[0].add_patch(ep.draw_yz(fill=False, facecolor='tab:blue', **paddle_style))
            else:
                axs[0].add_patch(ep.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    if event is not None:
        for j, pdl_hit in enumerate(hit_pids):
            if pdl_hit in [pdl_.paddle_id for pdl_ in ep_1]:
                axs[0].scatter([ys[j]], [zs[j]], alpha = 0.8 , marker='o', s=100*en[j],
                               lw=1.5, edgecolor='k', color=cmap(cm_norm_pts(ts[j])))

    for pdl in cor_front:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            if not paddle_occupancy[pdl.paddle_id] and indicate_empty:
                color = indicate_empty
            axs[0].add_patch(pdl.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            if event is not None:
                axs[0].add_patch(pdl.draw_yz(fill=False, facecolor='tab:blue', **paddle_style))
            else:
                axs[0].add_patch(pdl.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    
    if event is not None:
        for j, pdl_hit in enumerate(hit_pids):
            if pdl_hit in [pdl_.paddle_id for pdl_ in cor_front]:
                axs[0].scatter([ys[j]], [zs[j]], alpha = 0.8 , marker='o', s=100*en[j],
                               lw=1.5, edgecolor='k', color=cmap(cm_norm_pts(ts[j])))

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
            if not paddle_occupancy[ep.paddle_id] and indicate_empty:
                color = indicate_empty
            axs[1].add_patch(ep.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            if event is not None:
                axs[1].add_patch(ep.draw_xz(fill=False, facecolor='tab:blue', **paddle_style))
            else:
                axs[1].add_patch(ep.draw_xz(fill=True, edgecolor='k', facecolor='w'))
    if event is not None:
        for j, pdl_hit in enumerate(hit_pids):
            if pdl_hit in [pdl_.paddle_id for pdl_ in ep_2]:
                axs[1].scatter([xs[j]], [zs[j]], alpha = 0.8 , marker='o', s=100*en[j],
                                lw=1.5, edgecolor='k', color=cmap(cm_norm_pts(ts[j])))

    for pdl in cor_sb:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            if not paddle_occupancy[pdl.paddle_id] and indicate_empty:
                color = indicate_empty
            axs[1].add_patch(pdl.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            if event is not None:
                axs[1].add_patch(pdl.draw_xz(fill=False, facecolor='tab:blue', **paddle_style))
            else:
                axs[1].add_patch(pdl.draw_xz(fill=True, edgecolor='k', facecolor='w'))
    if event is not None:
        for j, pdl_hit in enumerate(hit_pids):
            if pdl_hit in [pdl_.paddle_id for pdl_ in cor_sb]:
                axs[1].scatter([zs[j]], [zs[j]], alpha = 0.8 , marker='o', s=100*en[j],
                               lw=1.5, edgecolor='k', color=cmap(cm_norm_pts(ts[j])))
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
            if not paddle_occupancy[ep.paddle_id] and indicate_empty:
                color = indicate_empty
            axs[2].add_patch(ep.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            if event is not None:
                axs[2].add_patch(ep.draw_yz(fill=False, facecolor='tab:blue', **paddle_style))
            else:
                axs[2].add_patch(ep.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    if event is not None:
        for j, pdl_hit in enumerate(hit_pids):
            if pdl_hit in [pdl_.paddle_id for pdl_ in ep_3]:
                axs[2].scatter([ys[j]], [zs[j]], alpha = 0.8 , marker='o', s=100*en[j],
                               lw=1.5, edgecolor='k', color=cmap(cm_norm_pts(ts[j])))
    
    for pdl in cor_back:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            if not paddle_occupancy[pdl.paddle_id] and indicate_empty:
                color = indicate_empty
            axs[2].add_patch(pdl.draw_yz(fill=True, edgecolor=color, facecolor=color))
        else:
            if event is not None:
                axs[2].add_patch(pdl.draw_yz(fill=False, facecolor='tab:blue', **paddle_style))
            else:
                axs[2].add_patch(pdl.draw_yz(fill=True, edgecolor='k', facecolor='w'))
    
    if event is not None:
        for j, pdl_hit in enumerate(hit_pids):
            if pdl_hit in [pdl_.paddle_id for pdl_ in cor_back]:
                axs[2].scatter([ys[j]], [zs[j]], alpha = 0.8 , marker='o', s=100*en[j],
                               lw=1.5, edgecolor='k', color=cmap(cm_norm_pts(ts[j])))
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
            if not paddle_occupancy[ep.paddle_id] and indicate_empty:
                color = indicate_empty
            axs[3].add_patch(ep.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            if event is not None:
                axs[3].add_patch(ep.draw_xz(fill=False, facecolor='tab:blue', **paddle_style))
            else:
                axs[3].add_patch(ep.draw_xz(fill=True, edgecolor='k', facecolor='w'))
    if event is not None:
        for j, pdl_hit in enumerate(hit_pids):
            if pdl_hit in [pdl_.paddle_id for pdl_ in ep_4]:
                axs[3].scatter([xs[j]], [zs[j]], alpha = 0.8 , marker='o', s=100*en[j],
                               lw=1.5, edgecolor='k', color=cmap(cm_norm_pts(ts[j])))

    for pdl in cor_bb:
        if paddle_occupancy:
            color = cmap(paddle_occupancy[pdl.paddle_id])
            if not paddle_occupancy[pdl.paddle_id] and indicate_empty:
                color = indicate_empty
            axs[3].add_patch(pdl.draw_xz(fill=True, edgecolor=color, facecolor=color))
        else:
            if event is not None:
                axs[3].add_patch(pdl.draw_xz(fill=False, facecolor='tab:blue', **paddle_style))
            else:
                axs[3].add_patch(pdl.draw_xz(fill=True, edgecolor='k', facecolor='w'))
    
    if event is not None:
        for j, pdl_hit in enumerate(hit_pids):
            if pdl_hit in [pdl_.paddle_id for pdl_ in cor_bb]:
                axs[3].scatter([xs[j]], [zs[j]], alpha = 0.8 , marker='o', s=100*en[j],
                               lw=1.5, edgecolor='k', color=cmap(cm_norm_pts(ts[j])))
    
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



