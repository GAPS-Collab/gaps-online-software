#! /usr/bin/env python

"""
GAps Zero-setup Event viewer.
"""

import sys
import numpy as np
import streamlit as st
import pyvista as pv
import gondola as gon
import matplotlib
import matplotlib.pyplot as plt
import charmingbeauty as cb 
cb.visual.set_style_streamlit_dark()

from stpyvista import stpyvista
from streamlit_js_eval import streamlit_js_eval

#def generate_file_streamer(infile):
#    reader = gon.io.TelemetryPacketReader(infile)
#    return streamer 

# prepare online calibration for energy calculation
TRK_ONLINE_CAL = gon.calibration.TrackerOnlineCalibration.from_file('/srv/gaps/gaps-online-software/gondola-core/rust/gondola-core/python/gondola/tracker_cal')

def add_arrow(ax, p_arrow_h, p_arrow_t, c0=0, c1=1, fc = 'w', ec = 'w'):
    x_0 = p_arrow_h[c0]/10
    y_0 = p_arrow_h[c1]/10
    dx  = (p_arrow_t[c0] - p_arrow_h[c0])/10
    dy  = (p_arrow_t[c1] - p_arrow_h[c1])/10
    #print(x_0, y_0, dx, dy)
    length = 10
    alpha = 1
    width = 0.5
    head_starts_at_zero = True
    head_width = width*10
    head_length = length*1.2
    shape = 'full'
    arrow_params={'length_includes_head':False,\
     'shape':shape,\
     'head_starts_at_zero':head_starts_at_zero}
    #print (head_width)
    #print (x_0,y_0)
    arr = ax.arrow(x_0, y_0, dx*length,\
     dy*length, fc=fc, ec=ec,\
     alpha=alpha, width=width,\
     head_width=head_width,\
     head_length=head_length,\
     **arrow_params)


@st.fragment 
def reset_camera():
    st.session_state.plotter.view_isometric()

@st.fragment 
def prev_event() -> None:
    print("prev_event() is not yet implemented!")

@st.fragment
def get_current_evid() -> int:
    return st.session_state.event.tof.event_id 

@st.fragment 
def get_current_runid() -> int:
    return st.session_state.event.tof.run_id

@st.fragment
def next_event():
    st.session_state.prev_event = st.session_state.event
    for pack in st.session_state.reader:
        if pack is None:
            st.session_state.reader.rewind()
            return 
        
        if pack.is_event_packet:
            ev = st.session_state.event  = gon.events.TelemetryEvent.from_telemetrypacket(pack)
            st.session_state.event_ptype = pack.header.packet_type  
    #st.session_state.ev 

@st.fragment
def create_event_plots(event, no_plot_first_bins_wf=False) -> dict:
    """
    Plots from the cahced events for an interactive view

    # Keyworkd Args:
        * no_plot_first_bins_wf : If True, don't plot the first bins in a waveform, since 
          these might just spikes.
    """
    # return values
    data = {
            'merged_event'            : None,
            'packet_type'             : None,
            'tof_xy'                  : None,
            'gaps_xy'                 : None,
            'gaps_xz'                 : None,
            'gaps_yz'                 : None,
            'tof_xy_all'              : None,
            'tof_cbe'                 : None,
            'tof_cor'                 : None,
            'tof_event'               : None,
            'tof_hits'                : [],
            'trk_hits'                : [],
            'trk_layers'              : [],
            'trk_pointcloud'          : [],
            'n_trk_hits_masked'       : None,
            'n_trk_hits_no_mask_info' : None
    }

    ev    = event 
    wf_ev = None
    ptype = st.session_state.event_ptype
    #ev_filename , ptype, (ev, wf_ev) = st.session_state.ev_viewer_cache[st.session_state.ev_viewer_idx]
    n_trk_hits_masked       = None 
    n_trk_hits_no_mask_info = None
    telemetry = False
    if isinstance(ev, gon.events.TelemetryEvent):
        ev_tof = ev.tof
        data['tof_event'] = ev_tof
        data['merged_event'] = ev
        data['packet_type']     = ptype
        telemetry = True
    else:
        ev_tof = ev
        data['tof_event'] = ev_tof
        if ev_tof.hits:
            paddle_style          = {'edgecolor' : 'w', 'lw' : 1.0}
            data['tof_xy']    , _ = gon.visual.tof.tof_projection_xy(event=ev_tof, cmap=matplotlib.colormaps['seismic'])
            data['tof_cbe']   , _ = gon.visual.tof.unroll_cbe_sides  (event=ev_tof, cmap=matplotlib.colormaps['seismic'], paddle_style = paddle_style)
            data['tof_cor']   , _ = gon.visual.tof.unroll_cor        (event=ev_tof, cmap=matplotlib.colormaps['seismic'], paddle_style = paddle_style)
            data['tof_xy_all'], data['tof_xz_all'], data['tof_yz_all'] \
                    = gon.visual.tof.tof_2dproj(event=ev_tof, cmap=matplotlib.colormaps['seismic'])
        data['tof_hits'] = ev_tof.hits
    
    if telemetry:
        if ev.tof.hits:
            paddle_style          = {'edgecolor' : 'w', 'lw' : 1.0}
            data['tof_xy']    , _ = gon.visual.tof.tof_projection_xy(event=ev_tof, cmap=matplotlib.colormaps['seismic'])
            data['tof_cbe']   , _ = gon.visual.tof.unroll_cbe_sides  (event=ev_tof, cmap=matplotlib.colormaps['seismic'], paddle_style = paddle_style)
            data['tof_cor']   , _ = gon.visual.tof.unroll_cor        (event=ev_tof, cmap=matplotlib.colormaps['seismic'], paddle_style = paddle_style)
            data['tof_xy_all'], data['tof_xz_all'], data['tof_yz_all'] \
                    = gon.visual.tof.tof_2dproj(event=ev_tof, cmap=matplotlib.colormaps['seismic'])
        data['tof_hits'] = ev.tof.hits
        for h in ev.tracker:
            data['trk_hits'].append(h)
        ## FIXME - the pointcloud needs masking
        data['trk_pointcloud']          = ev.tracker_pointcloud 
        data['n_trk_hits_masked']       = n_trk_hits_masked
        data['n_trk_hits_no_mask_info'] = n_trk_hits_no_mask_info
        #data['trk_plots'] = plot_tracker(data['trk_hits'], strip_dict)
        if wf_ev is not None:
            calib = st.session_state.tof_calib 
            if no_plot_first_bins_wf:
                data['waveform_figs'], __ = gon.visual.tof.plot_waveforms(wf_ev, calib=calib, with_hits=True, skip_bins=10)  
            else:
                data['waveform_figs'], __ = gon.visual.tof.plot_waveforms(wf_ev, calib=calib, with_hits=True)  
    return data
    
@st.fragment
def page_event_view():
    """
    A simple event viewer
    """
    ev_data = create_event_plots(st.session_state.event)
    if not ev_data:
        st.write('No events available!')
        st.write('-> Please go to Run -> Load Run to load some events for this analysis!')
        return
    
    #tracker_plots = plot_tracker(ev_data['trk_hits'], strip_dict)

    st.subheader(f"Run {ev_data['tof_event'].run_id} Event {ev_data['tof_event'].event_id}")
    l_col, r_col, __, __, __, __, __, __  = st.columns(8, vertical_alignment="bottom")
    l_col.button("PrevEvent", on_click=prev_event, args=[st.session_state]) 
    r_col.button("NextEvent", on_click=next_event, args=[st.session_state]) 
    tab_event, tab_tof_panels, tab_tof_waveforms, tab_tracker_layers, tab_2d, tab_3d = st.tabs(["Event", "Tof panels", "Tof waveforms", "Trk layers", "2d projections", "3d view"])
    with tab_event:
        if ev_data['packet_type'] is not None:
            st.badge(f"{ev_data['packet_type']}", color='blue')
        if ev_data['tof_event'].status == gon.events.EventStatus.AnyDataMangling:
            st.badge("AnyDataMangling", color='red')
        if ev_data['tof_event'].status == gon.events.EventStatus.EventTimeOut:
            st.badge("EventTimedOut", color='red')
        
        with st.expander('Event properties'):
            if ev_data['merged_event'] is not None:
                st.text(f'{ev_data["merged_event"]}')
                st.divider()
            if not ev_data['tof_event'].event_id == 0: 
                ## the formatting here looks weird, but apperas nicely in the app
                st.text(f'''           Trigger sources : {ev_data["tof_event"].trigger_sources}
                Status                  : {ev_data["tof_event"].status}
                Event ID              : {ev_data["tof_event"].event_id}
                Timestamp        : {ev_data["tof_event"].timestamp48}''')
                mapping = gon.db.get_dsi_j_ch_pid_map()
                st.divider()
                st.text(f'TRIGGER HITS : {[h for h in ev_data["tof_event"].trigger_hits]}')
                st.text(f'RB LINK IDs : {[int(h) for h in ev_data["tof_event"].rb_link_ids]}')
                if ev_data['tof_event'].get_missing_paddles_hg(mapping):
                    st.text(f'MISSING HG HITS: {[int(h) for h in ev_data["tof_event"].get_missing_paddles_hg(mapping)]}')
                
                st.subheader(f"{len(ev_data['tof_hits'])} TOF hits")
                for h in ev_data['tof_hits']:
                    with st.expander(f"Paddle {h.paddle_id}"):
                        st.text(f'{h}')
                st.subheader(f"{len(ev_data['trk_hits'])} TRK hits")
        
        for h in ev_data['trk_hits']:
            with st.expander(f'Strip {h.strip_id}'):
                st.text(f'{h}')
        if ev_data['n_trk_hits_masked'] is None:
            st.text('No tracker strip mask applied!')
        else:
            n_strips_masked = ev_data['n_trk_hits_masked']
            if n_strips_masked > 0:
                st.text(f'{n_strips_masked} hits have been masked due to marked as bad in the used strip mask!')

    with tab_tof_panels:
        if ev_data['tof_xy'] is not None:
            st.pyplot(ev_data['tof_xy'])
        if ev_data['tof_cbe'] is not None:
            st.pyplot(ev_data['tof_cbe'])
        if ev_data['tof_cor'] is not None:
            st.pyplot(ev_data['tof_cor'])

    with tab_tof_waveforms: 
        #st.subheader(f"Waveforms for {len(st.session_state['waveform_figs'])} paddles")
        st.session_state.no_plot_first_bins_wf = st.checkbox("Skip the first 10 bins (ns) when plotting waveforms", value=st.session_state.no_plot_first_bins_wf, on_change=lambda : create_event_plots(st.session_state.event))
        st.session_state.no_plot_first_bins_wf2 = st.checkbox("Skip the first 100 bins (ns) when plotting waveforms", value=st.session_state.no_plot_first_bins_wf2, on_change=lambda : create_event_plots(st.session_state.event))
        st.session_state.no_plot_last_bins_wf = st.checkbox("Skip the last 250 bins (ns) when plotting waveforms", value=st.session_state.no_plot_last_bins_wf, on_change=lambda : create_event_plots(st.session_state.event))
        st.divider()
        ev_data = create_event_plots(st.session_state.event,no_plot_first_bins_wf=st.session_state.no_plot_first_bins_wf)
        if not ev_data:
            st.write('No events available!')
            st.write('-> Please go to Run -> Load Run to load some events for this analysis!')
            return
        if 'waveform_figs' in ev_data:
            for wf_plot in ev_data['waveform_figs']:
                if st.session_state.no_plot_last_bins_wf:
                    ax = wf_plot.gca()
                    ax.set_xlim(right=250)
                if st.session_state.no_plot_first_bins_wf2:
                    ax = wf_plot.gca()
                    ax.set_xlim(left=100)
                st.pyplot(wf_plot)

    with tab_tracker_layers:
        #ev_filename , ptype, (ev, wf_ev) = st.session_state.ev_viewer_cache[st.session_state.ev_viewer_idx]
        ev = st.session_state.event 
        wf_ev = None 
        ptype = st.session_state.event_ptype 
        if hasattr(ev,'tracker'):
            trk_plots = gon.visual.tracker.plot_tracker(hits = ev.tracker) 
        else:
            trk_plots = False
        if trk_plots:
            st.pyplot(trk_plots['trk_proj_xy'], use_container_width=False)
            st.pyplot(trk_plots['trk_proj_xz'], use_container_width=False)
            st.pyplot(trk_plots['trk_proj_yz'], use_container_width=False)
            layer_keys = [k for k in trk_plots.keys() if k not in ('trk_proj_xy', 'trk_proj_xz', 'trk_proj_yz')]
            for k in layer_keys:
                layer = int(k[10:])
                with st.expander(f"Layer {layer}"):
                    st.pyplot(trk_plots[k], use_container_width=False) 
        #if ev_data['trk_plots']:
        #    for k in ev_data['trk_plots'].keys():
        #        fig = ev_data['trk_plots'][k]
        #        st.pyplot(fig, use_container_width=False)

    with tab_2d:
        hitstyle={ 'edgecolor' : 'w', 'alpha' : 0.5 , 'marker' : 'o'} 
        circle_color = 'w'
        if not st.session_state.use_dark_theme:
            circle_color = 'k'
            hitstyle={'edgecolor' : 'k', 'alpha' : 0.5, 'marker' : 'o'} 
        #tof_ev = copy(ev_data['tof_event'])
        plot_tracker2d = st.checkbox('Add tracker projections', key='add_tracker_proj')
        cs_is_energy   = st.checkbox('Use color scale for energy instead of timing', key='cs_is_energy')
        viewer_apply_lightspeed_cleaning = st.checkbox("Apply lightspeed cleaning for TOF hit. (Can't be undone!)", key='lightspeed_cleaning_2dview')
        cleaning_tolerance = 0.35
        #if viewer_apply_lightspeed_cleaning:
        cleaning_tolerance= st.number_input(
          f"Allowed error in ns for lightspeed cleaning",
          value=0.35,
          min_value=0.0,
          step=0.05,
          key='lightspeed_cleaning_tolerance',
          placeholder="Perform lightspeed cleaning for TOF hits`")
        if viewer_apply_lightspeed_cleaning:
            ev_data['tof_event'].lightspeed_cleaning(t_err = cleaning_tolerance)
        color = 'w'
        paddle_style     = {'edgecolor' : 'w', 'lw' : 0.4}

        if not st.session_state.use_dark_theme:
            color = 'k'
            paddle_style     = {'edgecolor' : 'k', 'lw' : 0.4}
        show_linefit   = st.checkbox('Show a simple linefit')
        if show_linefit:
            search_anchor  = st.checkbox('Search anchor point iteratively (slower)')
            xs = [k[0] for k in ev_data['trk_pointcloud']]
            xs.extend([h.x for h in ev_data['tof_event'].hits])

            ys = [k[1] for k in ev_data['trk_pointcloud']]
            ys.extend([h.y for h in ev_data['tof_event'].hits])

            zs = [k[2] for k in ev_data['trk_pointcloud']]
            zs.extend([h.z for h in ev_data['tof_event'].hits])
            
            xs = np.array(xs)
            ys = np.array(ys)
            zs = np.array(zs)
            
            reco = gon.reconstruction.line_fit(xs, ys, zs, search_anchor = search_anchor)
            # plot in z from -25 to 250
            p0, chi2 = reco[0](2200),reco[1]
            #chi2/(len(xs) - 6)
            p1 = reco[0](-200)
            print ('RCONSTRUCTION!',p0, p1, chi2)
            p0 = np.array(p0)/10
            p1 = np.array(p1)/10
            p_arrow_h = reco[0](500) # somewhat close to the end
            p_arrow_t = reco[0](300) # somewhat close to the end
        all_colormaps = list(plt.colormaps())
        selected_cmap = st.selectbox('Choose a maptlotlib colormap for the times of the TOF hits', 
                                     all_colormaps,
                                     index=all_colormaps.index('seismic'),
                                     key = 'cmap_selectbox_2dproj')
        cmap  = matplotlib.colormaps[selected_cmap]
        no_ax = st.checkbox("Don't show axes", value=True)
        show_cbar = st.checkbox("Show a colorbar", value=True)
        print (ev_data['tof_event'])
        print (ev_data['tof_event'].rb_events)
        
        tof_xy_all, tof_xz_all, tof_yz_all \
            = gon.visual.tof.tof_2dproj(
                                        event=ev_data['tof_event'],\
                                        #event = gon.events.TofEvent(),\
                                        cmap=cmap,
                                        paddle_style   = paddle_style,
                                        no_ax_no_ticks = no_ax,
                                        show_cbar      = show_cbar,
                                        cnorm_max      = 2,
                                        cs_is_energy   = cs_is_energy)
        if plot_tracker2d or show_linefit:
            ax = tof_xy_all.gca()
            if show_linefit:
                ax.plot([p0[0],p1[0]],[p0[1],p1[1]], color=color, lw=1.0)
                ax.text(p0[0]+5,p0[1]+5,'$\mu$')
                ax.text(p0[0]+5,p0[1]-4, '$\chi^2$ = ' + f'{chi2:.2f}', fontsize=5)
                add_arrow(ax, p_arrow_h, p_arrow_t, fc=paddle_style['edgecolor'], ec=paddle_style['edgecolor'])
            if plot_tracker2d:    
                gon.visual.tracker.plot_tracker_proj(
                    ax,\
                    ev_data['trk_hits'],\
                    projection='xy',\
                    use_energy=cs_is_energy,\
                    circle_color = circle_color,\
                    hitstyle = hitstyle,\
                    cmap = cmap,\
                    color_energy=cs_is_energy
                )
            
            ax = tof_xz_all.gca()
            if show_linefit:
                ax.plot([p0[0],p1[0]],[p0[2],p1[2]], color=color, lw=1.0)
                ax.text(p0[0]+5,p0[2]+5,'$\mu$')
                add_arrow(ax, p_arrow_h, p_arrow_t,c0=0,c1=2, fc=paddle_style['edgecolor'], ec=paddle_style['edgecolor'])
            if plot_tracker2d:
                gon.visual.tracker.plot_tracker_proj(
                    ax,\
                    ev_data['trk_hits'],\
                    projection='xz',\
                    use_energy=cs_is_energy,\
                    circle_color = circle_color,\
                    hitstyle = hitstyle,\
                    cmap = cmap,\
                    color_energy=cs_is_energy
                )
            
            ax = tof_yz_all.gca()
            if show_linefit:
                ax.plot([p0[1],p1[1]],[p0[2],p1[2]], color=color, lw=1.0)
                ax.text(p0[1]+5,p0[2]+5,'$\mu$')
                add_arrow(ax, p_arrow_h, p_arrow_t, c0=1,c1=2, fc=paddle_style['edgecolor'], ec=paddle_style['edgecolor'])
            if plot_tracker2d:
                gon.visual.tracker.plot_tracker_proj(
                    ax,\
                    ev_data['trk_hits'],\
                    projection='yz',\
                    use_energy=cs_is_energy,\
                    circle_color = circle_color,\
                    hitstyle = hitstyle,\
                    cmap = cmap,\
                    color_energy=cs_is_energy
                )
        
        st.pyplot(tof_xy_all)
        st.pyplot(tof_xz_all)
        st.pyplot(tof_yz_all)
        st.subheader('TOF Noise identification - "lightspeed cleaning"') 
        time_evolution = gon.visual.tof.tof_hits_time_evolution(ev_data['tof_event'],line_color='w', t_err = cleaning_tolerance)
        st.pyplot(time_evolution)

    with tab_3d:
        # Load mesh
        show_3dplot = st.checkbox('Show 3d plot! (experimental, might take a while)')
        show_linefit   = st.checkbox('Show a simple linefit', key='lf_for_3d')
        xs = [k[0] for k in ev_data['trk_pointcloud']]
        xs.extend([h.x for h in ev_data['tof_event'].hits])

        ys = [k[1] for k in ev_data['trk_pointcloud']]
        ys.extend([h.y for h in ev_data['tof_event'].hits])

        zs = [k[2] for k in ev_data['trk_pointcloud']]
        zs.extend([h.z for h in ev_data['tof_event'].hits])
        sizes = [k[3] for k in ev_data['trk_pointcloud']]
        sizes.extend([h.edep for h in ev_data['tof_event'].hits])

        xs = np.array(xs)
        ys = np.array(ys)
        zs = np.array(zs)
        sizes = 5*np.array(sizes) 
        
        plotter = st.session_state.plotter
        if st.button("Reset"):
            del st.session_state.plotter 
            st.session_state.plotter = pv.Plotter(window_size=[800, 600], off_screen=True)
        
        if show_linefit:
            
            reco = gon.reconstruction.line_fit(xs, ys, zs, search_anchor = False)
            if reco is not None:
                # plot in z from -25 to 250
                p0, chi2 = reco[0](2200),reco[1]
                #chi2/(len(xs) - 6)
                p1 = reco[0](-200)
                print ('RCONSTRUCTION!',p0, p1, chi2)
                p0 = np.array(p0)
                p1 = np.array(p1)
                p_arrow_h = reco[0](500) # somewhat close to the end
                p_arrow_t = reco[0](300) # somewhat close to the end
            else:
                show_linefit = False

        if show_3dplot:
            
            #mesh = pv.read("/srv/gaps/gaps-online-software/event-viewer/sample.ply")
            #print ("Mesh loaded!") 
            # Create a PyVista plotter
            plotter.set_background("#0E1117")
            #print ("Plotter created")
            # Example: point cloud
            #points = np.random.rand(100, 3) * 10  # 100 random points in space
            points = np.array([k for k in zip(xs,ys,zs)])/10
            #print (points)
            #sizes = np.linspace(5, 20, len(points))       # point sizes
            colors = np.random.rand(len(points), 3)       # RGB colors in [0,1]
            
            point_cloud = pv.PolyData(points)
            point_cloud["colors"] = (colors * 255).astype(np.uint8)
            #point_cloud["sizes"] = sizes
            point_cloud["scales"] = sizes 
            # Create sphere source for glyphs
            sphere = pv.Sphere(radius=1.0)
            
            # Glyph each point with its own size
            glyphs = point_cloud.glyph(
                geom=sphere,
                scale="scales",   # use per-point scaling
                orient=False
            )

            
            # Create PyVista plotter
            plotter = pv.Plotter(window_size=[800, 600], off_screen=True)
            plotter.set_background("#0E1117")
            #plotter.set_background("#000000")

            ## Add point cloud (with custom sizes & colors)
            plotter.add_mesh(glyphs, scalars="colors", rgb=True)
            #plotter.add_points(
            #    point_cloud,
            #    scalars="colors",
            #    rgb=True,
            #    render_points_as_spheres=True,
            #    point_size=10,  # global scale, sizes will modulate this
            #)
            #
            ## Add line
            # Example: line between two points
            if show_linefit:
                line_points = (p0/10,p1/10)
            #line_points = np.array([[0, 0, 0], [5, 5, 5]])
                line = pv.Line(line_points[0], line_points[1])
                plotter.add_mesh(line, color="#F0F0F0", line_width=5)
            # Add wireframe mesh
            #plotter.add_mesh(mesh, color="#F0F0F0", style="wireframe", line_width=1)
            paddles = gon.db.TofPaddle.all()
            for pdl in paddles:
                box = pv.wrap(pdl._create_box())
            #box = pv.Box(bounds=(0, 1, 0, 2, 0, 0.5)) 
            # Add mesh as wireframe
            #plotter.add_mesh(mesh, color="#F0F0F0", style="wireframe", line_width=1)
                plotter.add_mesh(box, color="#F0F0F0", style="wireframe", line_width=1)
            #print ("mesh added")

            # Render inside Streamlit
            plotter.reset_camera()
            #col1, col2, col3, col4, __, __, __, __, __, __, __, __ = st.columns(12)
            #with col1:
            #    if st.button("🔄 Reset"):
            #        plotter.reset_camera()    
            #with col2:
            #    if st.button("X-Axis"):
            #        plotter.view_xz()
            #        plotter.reset_camera()
            #with col3:
            #    if st.button("Y-Axis"):
            #        plotter.view_yz()
            #        plotter.reset_camera() 
            #with col4:
            #    if st.button("Z-Axis"):
            #        plotter.view_xy()
            #        plotter.reset_camera()
            stpyvista(plotter, key="ply_viewer")
            print ("stpvista done")


def page_view():
    # dummy points
    #trk_points   = st.session_state.event.tracker_pointcloud 
    trk_hits     = st.session_state.event.tracker 
    cali_hits    = [] 
    for h in trk_hits:
        TRK_ONLINE_CAL.calibrate(h) 
        cali_hits.append(h) 
    trk_hits     = cali_hits
    #trk_hits     = [k for k in map(TRK_ONLINE_CAL.calibrate, trk_hits)]
    trk_points   = [(10*h.x,10*h.y,10*h.z,np.nan,h.energy) for h in trk_hits]
    print ('---- DATA POINTS ----') 
    for k in trk_points: 
        print (f'-- {k}')
    points       = st.session_state.event.tof.pointcloud + trk_points 
    #sizes        = np.array([k[4] for k in points])/100.
    sizes        = np.array([k[4] for k in points])*st.session_state.hit_bubble_scale
    print (sizes) 
    #sizes        = np.array([5 for k in points])
    #print ('---- DATA POINTS ----') 
    #for k in points: 
    #    print (f'-- {k}')

    # set up a good width for the actual vtk plotter
    l_col_size  = 0.8 
    r_col_size  = 1 - l_col_size
    inner_width = streamlit_js_eval(js_expressions='window.innerWidth', key='WIDTH', want_output=True)
    if inner_width is None:
        inner_width = 1.0
    window_size = l_col_size*inner_width
    window_size = [int(window_size), int(window_size/1.618)]
    st.session_state.plotter = pv.Plotter(window_size=window_size, off_screen=True)
    plotter = st.session_state.plotter
    # set up the plotter 
    #mesh = pv.read("/srv/gaps/gaps-online-software/event-viewer/sample.ply")
    plotter.set_background("#0E1117")
    points = np.array([(k[0],k[1],k[2]) for k in points])/10
    colors = np.random.rand(len(points), 3)       # RGB colors in [0,1] 
    point_cloud = pv.PolyData(points)
    point_cloud["colors"] = (colors * 255).astype(np.uint8)
    point_cloud["scales"] = sizes 
    # Create sphere source for glyphs
    sphere = pv.Sphere(radius=1.0)
    # Glyph each point with its own size
    glyphs = point_cloud.glyph(
        geom=sphere,
        scale="scales",   # use per-point scaling
        orient=False
    )
 
    ## Add point cloud (with custom sizes & colors)
    plotter.add_mesh(glyphs, scalars="colors", rgb=True)
    paddles = gon.db.TofPaddle.all()
    for pdl in paddles:
        box = pv.wrap(pdl._create_box())
        plotter.add_mesh(box, color="#F0F0F0", style="wireframe", line_width=1)

    # Render inside Streamlit
    plotter.reset_camera()
    plotter.view_isometric()
    #plotter.view_xy()
    axes_scale = 1 
    plotter.add_axes(interactive  = True,
                     line_width   = 3,
                     color        = "w",
                     shaft_length = axes_scale*0.8,
                     tip_length   = axes_scale*0.2,
                     ambient      = axes_scale*0.5)
    plotter.show_axes()
    st.header(f'Run {get_current_runid()} Event {get_current_evid()}')
    l_col, r_col = st.columns([l_col_size,r_col_size], vertical_alignment="top")
    with r_col:
        st.write('Shift + Drag,Pan')
        st.write('Scroll,Zoom')
        st.write('LMB + Drag,Free rotate')
        st.write('Key-W,Show wireframe')
        st.write('Key-V,Show vertices')
        st.write('Key-S,Show surface')
        st.write('Key-R,Reset view')
        st.write('Ctrl + LMB + Drag,Rotate around center')
        st.button("Reset camera!", on_click=reset_camera) 
        st.button('Prev. event!', on_click=prev_event) 
        st.button('Next event!', on_click=next_event)
    with l_col:
        stpyvista(plotter, key="ply_viewer")
    st.write(f'window size : {window_size}')


if __name__ == '__main__':
    # Set up i/o 
    infile = sys.argv[1]
    print (f'-> Reading file {infile}')
    reader = gon.io.TelemetryPacketReader(infile) 
    packet_index = reader.count_packets() 
    reader.rewind()
    for pack in reader:
        if pack.is_event_packet:
            # FIXME - allow to display a specific event
            ev = gon.events.TelemetryEvent.from_telemetrypacket(pack) 
            if ev.tof.event_id == 1557344:
                break
            #break 

    # keep static variables
    st.session_state.reader                 = reader  
    st.session_state.event                  = gon.events.TelemetryEvent.from_telemetrypacket(pack)
    st.session_state.event_ptype            = pack.header.packet_type
    st.session_state.prev_event             = st.session_state.event
    st.session_state.hit_bubble_scale       = 1
    st.session_state.hit_bubble_log         = False
    st.session_state.no_plot_first_bins_wf  = False
    st.session_state.no_plot_first_bins_wf2 = False
    st.session_state.no_plot_last_bins_wf   = False
    st.session_state.no_plot_last_bins_wf2  = False
    st.session_state.use_dark_theme         = True
    st.session_state.plotter                = pv.Plotter(off_screen=True)


    # The app
    st.set_page_config(page_title="Gaze - GAPS zero-setup event viewer",
                       page_icon=":balloon:",
                       layout="wide")
    st.logo('../resources/assets/GAPSLOGO_2023.png', size='large')
    pg = st.navigation([
        st.Page(page_view          , title="Event view"),
        st.Page(page_event_view    , title="Event view II")
    ]) 

    

    pg.run()    

