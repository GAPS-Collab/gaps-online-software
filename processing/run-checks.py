#! /usr/bin/env python 

"""
Simple plots for data quality/run assessment
"""

import tqdm
import argparse
from pathlib import Path 
import gondola as go

import matplotlib.pyplot as plt 
import matplotlib 
# no interactive plots
matplotlib.use('agg')

import numpy as np
import dashi as d 
d.visual()

import charmingbeauty as cb 
cb.visual.set_style_default()
cb.visual.set_style_streamlit_dark()

# check if gondola version satisfies required script version 
REQUIRED_VERSION = "0.12.19" 
if not go.version_at_least(REQUIRED_VERSION):
    print(f'ERROR - got version {go.get_version_major()}.{go.get_version_minor()}.{go.get_version_patch()}')
    raise ImportError(f"gondola needs to be at least version {REQUIRED_VERSION}! Please update your dependency (e.g. uv lock --upgrade")

#UMB_PIDS     = range(61,109)
#CBE_BOT_PIDS = range(12,25) 
#CBE_TOP_PIDS = range(1,13) 

def main():

    description = """Run over L0 files to create control plots (currently TOF only)"""
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument('--run-dir', default=Path('/data0/gaps/csbf/csbf-data/binaries/ethernet'),\
                        help='A directory with files L0 (caraspace) style',\
                        type=Path)
    parser.add_argument('-n','--n-events', type=int,\
                        default=-1, help='Restrict the number of events to be processed')
    parser.add_argument('--paddle-plots', action='store_true',\
                        default=False,
                        help='Create the plots for the individual paddles (takes long)')
    
    #parser.add_argument('-v','--verbose', action='store_true',\
    #                    help='More verbose output')
    args = parser.parse_args()
  
    t_offsets = dict()
    offsets   = go.db.TofPaddleTimingConstant.as_dict_by_name("GraceV1")
    for k in offsets:
        t_offsets[k] = offsets[k].timing_constant
        
    trigger_map   = go.db.get_dsi_j_ch_pid_map()
    runfiles      = [k for k in args.run_dir.glob('*.gaps')] # make it a list fo tqdm
    tof_analysis  = go.tof.analysis.TofAnalysis() 
    tof_analysis.active = True 
    tof_analysis.skip_mangled = False 
    tof_analysis.skip_timeout = False
    #reco_tel      = go.reconstruction.Reconstruction( active = True )  
    ## this is on the remerged events which have more tracker hits 
    #reco_remerged = go.reconstruction.Reconstruction( active = True )
    n_events      = 0
    #done          = False
    #n_hits_tel    = [] 
    #n_hits_remerged = []
    #event_ids = []
    # read L0 files 
    for f in tqdm.tqdm(runfiles):
        reader = go.io.CRReader(str(f)) 
        for frame in reader:
            ev = frame.get_telemetryevent('TelemetryEvent')
            if n_events == args.n_events:
                break 
            n_events += 1 
            #event_ids.append(ev.tof.event_id)
            tof_analysis.add_event(ev.tof) 
            if ev.tof.event_id == 1334:
                print (ev) 
    print ('-> Creating histograms ..') 
    tof_analysis.finish() 
    print (tof_analysis.pretty_print_statistics())
    print ('-> Saving figures!')
    plot_outdir = Path(f'run-check-plots') / 'nhit' 
    if not plot_outdir.exists():
        plot_outdir.mkdir(exist_ok=True, parents=True)
    # create all the plots
    # first - n hits
    for k in ('TOF Hits', 'hit'), ('TOF CBE Hits', 'nhit_cbe'), ('TOF UMB Hits', 'nhit_umb'), ('TOF COR hits', 'nhit_cor'): 
        fig = go.visual.style.gander_plot(tof_analysis.nhit_plots[k[1]], 'nhit', k[0])
        fig.savefig(plot_outdir / f'{k[1]}.png')

    for k in ('INNER TOF vs OUTER TOF','i_vs_o_nhit'), ('UMB vs COR', 'umb_vs_cor_nhit'), ('CBE vs UMB', 'cbe_vs_umb_nhit'), ('CBE vs COR','cbe_vs_cor_nhit'):
        fig = go.visual.style.gander_2dplot(tof_analysis.nhit_plots[k[1]], 'nhit', 'nhit', k[0])
        fig.savefig(plot_outdir / f'{k[1]}.png')
    fig, fig_ratio  = go.visual.tof.plot_hg_lg_hits(tof_analysis.nhit_plots['hit'],\
                                                    tof_analysis.nhit_plots['thit'],\
                                                    n_events         = tof_analysis.n_events,\
                                                    no_hitmissing    = tof_analysis.no_hitmiss,\
                                                    one_hitmissing   = tof_analysis.one_hitmiss,\
                                                    lttwo_hitmissing = tof_analysis.two_hitmiss,\
                                                    extra_hits       = tof_analysis.extra_hits)
        
    fig.savefig(plot_outdir / f'all_hits.png')
    fig_ratio.savefig(plot_outdir / f'hit_ratio.png')
    # second, edep 
    plot_outdir = Path(f'run-check-plots') / 'edep' 
    if not plot_outdir.exists():
        plot_outdir.mkdir(exist_ok=True, parents=True)
    for k in ('EDEP UMB', 'edep_umb'), ('EDEP CBE', 'edep_cbe'), ('EDEP_COR', 'edep_cor'): 
        fig = go.visual.style.gander_plot(tof_analysis.edep_plots[k[1]], 'MeV', k[0])
        fig.savefig(plot_outdir / f'{k[1]}.png')
    for k in ('EDEP UMB vs COR', 'umb_vs_cor_edep'), ('EDEP CBE vs COR', 'cbe_vs_cor_edep'), ('EDEP CBE vs UMB', 'cbe_vs_umb_edep'): 
        fig = go.visual.style.gander_2dplot(tof_analysis.edep_plots[k[1]], 'MeV', 'MeV', k[0])
        fig.savefig(plot_outdir / f'{k[1]}.png')
    plot_outdir = Path(f'run-check-plots') / 'occupancy' 
    if not plot_outdir.exists():
        plot_outdir.mkdir(exist_ok=True, parents=True)
    # occupancy for HG 
    occu   = tof_analysis.occupancy
    occu_norm = dict()
    max_occu  = max(occu.values())
    for k in occu:
        occu_norm[k] = occu[k]/max_occu
    #for k in sorted(occu.keys()):
    #    print (f'-- -- paddle {k} : {occu[k]}')
    fig,__ = go.visual.tof.tof_projection_xy(occu)  
    fig.gca().set_title('HG occupancy', loc='right')
    fig.savefig(plot_outdir / 'hg-occupancy-xy.png')
    fig,__ = go.visual.tof.unroll_cbe_sides(occu_norm) 
    fig.savefig(plot_outdir / 'hg-occupancy-cbe.png')
    fig,__ = go.visual.tof.unroll_cor(occu_norm) 
    fig.savefig(plot_outdir / 'hg-occupancy-cor.png')

    # occupancy for LG 
    occu   = tof_analysis.occupancy_t
    occu_norm = dict()
    max_occu  = max(occu.values())
    for k in occu:
        occu_norm[k] = occu[k]/max_occu
    #for k in sorted(occu.keys()):
    #    print (f'-- -- paddle {k} : {occu[k]}')
    fig,__ = go.visual.tof.tof_projection_xy(occu)  
    fig.gca().set_title('LG occupancy', loc='right')
    fig.savefig(plot_outdir / 'lg-occupancy.png')
    fig,__ = go.visual.tof.unroll_cbe_sides(occu_norm) 
    fig.savefig(plot_outdir / 'lg-occupancy-cbe.png')
    fig,__ = go.visual.tof.unroll_cor(occu_norm) 
    fig.savefig(plot_outdir / 'lg-occupancy-cor.png')

    # timing plots 
    plot_outdir = Path(f'run-check-plots') / 'timing' 
    if not plot_outdir.exists():
        plot_outdir.mkdir(exist_ok=True, parents=True)

    for k in ('beta',fr'$\beta$',''), ('t_inner','ns','Time (inner TOF)'), ('t_outer','ns','Time (outer TOF)'),\
             ('t_diff', 'ns', 'Time delta (outer -inner)'), ('ph_delay', fr'$\phi$', 'Phase delay') , ('dist', 'mm', 'Reco dist.'),\
             ('cos_theta', fr'$\cos(\theta)$', ''), ('cos2_theta',fr'$\cos^2(\theta)$', ''), \
             ('x_outer', 'mm', 'First hit x outer'), ('y_outer', 'mm', 'First hit y outer'), ('z_outer', 'mm', 'First hit z outer'),\
             ('x_inner', 'mm', 'First hit x innner'), ('y_inner', 'mm', 'First hit y inner'),\
             ('z_inner', 'mm', 'First hit z inner'), ('pid_inner', 'PID', 'First PID inner'), ('pid_outer', 'PID', 'First PID outer'):
        fig = go.visual.style.gander_plot(tof_analysis.tmg_plots[k[0]], k[1], k[2])
        fig.savefig(plot_outdir / f'{k[0]}.png')

    for k in ('dist_vs_beta',fr'$\beta$','mm'), ('dist_vs_tdiff','mm','ns'), ('beta_vs_theta',fr'$\beta$',fr'$\theta$'):
        fig = go.visual.style.gander_2dplot(tof_analysis.tmg_plots[k[0]], k[1], k[2], '')
        fig.savefig(plot_outdir / f'{k[0]}.png')
    
    if args.paddle_plots: 
        for pid in tqdm.tqdm(range(1,161), desc='Creating paddle plots..'):
            plot_outdir = Path(f'run-check-plots') / 'paddles' / f'{pid}' 
            if not plot_outdir.exists():
                plot_outdir.mkdir(exist_ok=True, parents=True)
            for side,label_side in ('a','A'),('b','B'):
                for k in (f'amp_{side}','mV', f'Peak {label_side}'),\
                         (f'charge_{side}','pC', f'Charge {label_side}'),\
                         (f'time_{side}', 'ns', f'LE Time {label_side}'),\
                         (f'bl_{side}', 'mV', f'Baseline {label_side}'),\
                         (f'bl_{side}_rms', 'mV', f'Baseline RMS {label_side}'):
                    fig = go.visual.style.gander_plot(tof_analysis.paddle_plots[pid][k[0]], k[1], k[2])
                    fig.savefig(plot_outdir / f'{k[0]}-{pid}.png')
            for k in ('x0', 'mm', 'Position on paddle'), ('t0', 'ns', fr'$T_0$'),\
                     ('edep', 'MeV', 'Energy Dep.'):
                fig = go.visual.style.gander_plot(tof_analysis.paddle_plots[pid][k[0]], k[1], k[2])
                fig.savefig(plot_outdir / f'{k[0]}-{pid}.png')
            
            for k in ('charge2d','pC [A]','pC [B]',''), ('amp2d','mV','mV',''), ('pos_edep','mm','MeV',''): 
                fig = go.visual.style.gander_2dplot(tof_analysis.paddle_plots[pid][k[0]], k[1], k[2], k[3])
                fig.savefig(plot_outdir / f'{k[0]}.png')

    print ('-> .. done!')



    #print (event_ids)
    #    if done:
    #        break
    #    reader = go.io.CRReader(str(f)) 
    #    frames_ntot = reader.count_frames()
    #    for frame in reader:
    #        ev = frame.get_telemetryevent('TelemetryEvent') 
    #        if len(ev.tof.trigger_sources) != 1:
    #            continue # we restrict ourselves to track triggers
    #        if ev.tof.trigger_sources[0] != go.events.TriggerType.Track:
    #            continue
    #        if ev.tof.event_status == go.events.EventStatus.AnyDataMangling or ev.tof.event_status == go.events.EventStatus.EventTimeOut:
    #            continue
    #        # a simple track selection
    #        # we want only through-going, fast (clean) tracks 
    #        # with exactly 3 hits - UMB, CBE TOP, CBE BOT
    #        trigger_pids = [int(k) for k in ev.tof.get_triggered_paddles(trigger_map)] 
    #        if len(trigger_pids) != 3:
    #            continue 
    #        condition = 0
    #        for k in trigger_pids:
    #            if k in UMB_PIDS:
    #                condition += 1
    #            if k in CBE_BOT_PIDS:
    #                condition += 1 
    #            if k in CBE_TOP_PIDS:
    #                condition += 1 
    #        # check if these TOF hits are also present in the readout 
    #        
    #        if condition != 3:
    #            continue
    #        # some hit cleaning 
    #        ev.set_tof_timing_offsets(t_offsets)
    #        ev.tof_normalize_hit_times()
    #        ev.tof_remove_non_causal_hits() 
    #        ev.tof_lightspeed_cleaning(0.35) 
    #        #print (ev.tof)
    #        #raise
    #        if len(ev.tof.hits) < 3: 
    #            continue
    #        reco_tel.reco(ev)
    #        n_hits_tel.append(len(ev.tracker)) 
    #        # create a new merged event with the hits from all 
    #        # the tracker events
    #        ev_remerged = frame.get_telemetryevent('TelemetryEvent')
    #        #print (f'--> Have {len(ev_remerged.tracker)}')
    #        ev_remerged.delete_all_tracker_hits()
    #        #print (f'--> Have {len(ev_remerged.tracker)}')
    #        trk_hits = frame.get_tracker_hitseries()
    #        ev_remerged.add_tracker_hits(trk_hits) 
    #        #print (f'--> Have {len(ev_remerged.tracker)}')
    #        # some hit cleaning 
    #        ev_remerged.set_tof_timing_offsets(t_offsets)
    #        ev_remerged.tof_normalize_hit_times()
    #        ev_remerged.tof_remove_non_causal_hits() 
    #        ev_remerged.tof_lightspeed_cleaning(0.35) 
    #        n_hits_remerged.append(len(ev_remerged.tracker))
    #        reco_remerged.reco(ev_remerged)
    #        #print (reco_remerged.beta_c) 
    #        #print (reco_tel.beta_c) 
    #        #print (ev)
    #        #raise
    #        n_events += 1 
    #        if args.n_events > 0:
    #            if n_events >= args.n_events:
    #                done = True
    #                break
    ##print (n_hits_remerged) 
    ##print (n_hits_tel) 
    #reco_remerged.finish()
    #reco_tel.finish() 
    #
    #fig_chi2 = plt.figure(figsize=cb.layout.FIGSIZE_A4_LANDSCAPE) 
    #ax = fig_chi2.gca() 
    #ax.set_xlabel('chi2',loc='right')
    #ax.set_ylabel('nevents',loc='top',rotation=90) 
    #ax.set_title('Linefit chi2 for track events', loc='right')
    #reco_tel.chi2.line(filled=True,color='w', alpha=0.7, label='merged')
    #reco_remerged.chi2.line(filled=True,color='r', alpha=0.5, label='from pk80')
    #ax.legend(loc='upper right')
    #ax.set_ylim(bottom=0)
    #fig_chi2.savefig('chi2-before-after.png')
    #
    #fig_beta = plt.figure(figsize=cb.layout.FIGSIZE_A4_LANDSCAPE) 
    #ax = fig_beta.gca() 
    #ax.set_xlabel('beta',loc='right')
    #ax.set_ylabel('nevents',loc='top',rotation=90) 
    #ax.set_title('Reco beta for track events', loc='right')
    #reco_tel.beta.line(filled=True,color='w', alpha=0.7, label='merged')
    #reco_remerged.beta.line(filled=True,color='r', alpha=0.5, label='from pk80')
    #ax.legend(loc='upper right')
    #ax.set_ylim(bottom=0)
    #fig_beta.savefig('beta-before-after.png')

    #fig_nhits = plt.figure(figsize=cb.layout.FIGSIZE_A4_LANDSCAPE) 
    #ax = fig_nhits.gca() 
    #ax.set_xlabel('n trk hits',loc='right')
    #ax.set_ylabel('nevents',loc='top',rotation=90) 
    #ax.set_title('N(Trk hit)', loc='right')
    #h = d.factory.hist1d(n_hits_tel, bins=np.arange(-0.5,250,1)) 
    #h2 = d.factory.hist1d(n_hits_remerged, bins=np.arange(-0.5,250,1)) 
    #h.line(filled=True,color='w', alpha=0.7, label='merged')
    #h2.line(filled=True,color='r', alpha=0.5, label='from pk80')
    #ax.legend(loc='upper right')
    #ax.set_ylim(bottom=0)
    #fig_nhits.savefig('ntrkhits-before-after.png')


    #print (f'-> Finished. {n_events} were reconstructed!')
if __name__ == '__main__':
    main()

