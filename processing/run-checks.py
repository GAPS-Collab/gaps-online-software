#! /usr/bin/env python 

"""
Simple plots for data quality/run assessment
"""

import argparse
import sys

from pathlib import Path 
from datetime import datetime, timedelta

import gondola as go

import matplotlib.pyplot as plt 
import matplotlib 
# no interactive plots
matplotlib.use('agg')

import tqdm
import numpy as np
import dashi as d 
d.visual()

import charmingbeauty as cb 
import charmingbeauty.layout as lo
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

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run-dir', default=Path('/data0/gaps/csbf/csbf-data/binaries/ethernet'),\
                        help='A directory with files L0 (caraspace) style',\
                        type=Path)
    parser.add_argument('-n','--n-events', type=int,\
                        default=-1, help='Restrict the number of events to be processed')
    parser.add_argument('--paddle-plots', action='store_true',\
                        default=False,
                        help='Create the plots for the individual paddles (takes long)')
    parser.add_argument('--tracks-only', action='store_true',\
                        help='Select only track trigger events')
    parser.add_argument('--moni-only', action='store_true',\
                        help='Look only at moni/housekeeping data')
    parser.add_argument('--no-cleaning', action='store_true',\
                        help='Explicetely do NOT cut away bad run start/stop periods for run 9125')
    parser.add_argument('--from-telemetry', action='store_true',\
                        help='Use telemetry files instead of L0 (faster)')
    #parser.add_argument('-v','--verbose', action='store_true',\
    #                    help='More verbose output')
    args = parser.parse_args()

    # get run meta data 
    runmeta       = sorted([k for k in args.run_dir.glob('*.meta.toml')]) 
    if len(runmeta) != 1:
        raise ValueError("Ambiguos run meta information - there are more than 1 .meta.tmol files - or none at all!") 
    runmeta       = go.run.RunMeta.load(runmeta[0]) 
    print (f'-> Loaded meta data for run {runmeta.run_id}') 
    
    # ---------- Create output directory for plots 
    plots_main_outdir = f'run-check-plots-{runmeta.run_id}'
    
    # ---------- Load run files (list for tqdm progressbar)
    runfiles      = sorted([k for k in args.run_dir.glob('*.gaps')], key=lambda x : go.io.get_rundata_from_file(x.name)['utctime'])
    print (f'-> Found {len(runfiles)} for run {runmeta.run_id}') 
    # clean files - check MTB moni data to see why 
    if runmeta.run_id == 9125 and not args.no_cleaning:
        run_file_start  = go.io.get_rundata_from_file(runfiles[0].name)['utctime'] 
        run_file_start  = datetime.strptime(run_file_start, '%y%m%d_%H%M%S')
        clean_run_start = run_file_start + timedelta(hours=1) 
        clean_run_end   = run_file_start + timedelta(hours=4) # we take 3 hours here 
        get_ts          = lambda x: datetime.strptime(go.io.get_rundata_from_file(x.name)['utctime'], '%y%m%d_%H%M%S')
        runfiles        = sorted([k for k in filter(lambda x : get_ts(x) > clean_run_start and get_ts(x) <= clean_run_end, runfiles)], key = lambda x : go.io.get_rundata_from_file(x.name)['utctime'])  
        print (f'-> After cleaning for run 9125, {len(runfiles)} files remain!')
    print (f'-> First file {runfiles[0] }, last file { runfiles[-1]}') 
    
    # for comparison get the same run from telemetry binaries 
    if args.from_telemetry:
        if runmeta.run_id == 9125:
            start = '241213_134934'
            end   = '241213_164817'
        if runmeta.run_id == 9113:
            start = '241212_150112' 
            end   = '241212_192351'
        start = go.io.get_unix_timestamp(start)
        end   = go.io.get_unix_timestamp(end)
        runfiles = go.io.grace_get_telemetry_binaries(start, end, data_dir='/data-ssd0/ground-data/2024/telemetry')


    # FIXME 
    only_tracks = args.tracks_only 
    # HACK 
    only_tracks = True

    t_offsets   = dict()
    offsets     = go.db.TofPaddleTimingConstant.as_dict_by_name("GraceV1")
    for k in offsets:
        t_offsets[k] = offsets[k].timing_constant
        
    trigger_map   = go.db.get_dsi_j_ch_pid_map()
    # --------------- set up plots for the TOF system 
    #go.tof.analysis.TofAnalysis.NHIT_BINS = np.arange(-0.5,7.5,1) 
    tof_analysis                  = go.tof.analysis.TofAnalysis() 
    tof_analysis.active           = True 
    tof_analysis.skip_mangled     = False 
    tof_analysis.skip_timeout     = False
    tof_analysis.event_cache_size = 10e6
    tof_analysis.hit_cache_size   = 100e6
    tof_analysis.paddle_plots     = args.paddle_plots
    #reco_tel      = go.reconstruction.Reconstruction( active = True )  
    ## this is on the remerged events which have more tracker hits 
    #reco_remerged = go.reconstruction.Reconstruction( active = True )
    n_events      = 0
    #done          = False
    #n_hits_tel    = [] 
    #n_hits_remerged = []
    #event_ids = []
    # read L0 files 
    # ---------------- prepare moni data 
    tof_hk_name       = 'TelemetryPacketType.AnyTofHK' 
    mtb_moni          = go.monitoring.MtbMoniDataSeries() 
    mtb_moni.max_size = int(1e7)  
    evbhb             = go.monitoring.EventBuilderHBSeries() 
    evbhb.max_size    = int(1e7) 
    
    # ---------------- event data
    telly_name  = 'TelemetryEvent'
    is_done     = False 
    for f in tqdm.tqdm(runfiles, desc='Load Run data...', colour='blue'):
        if is_done:
            break 
        if args.from_telemetry:
            reader = go.io.TelemetryPacketReader(str(f)) 
            for pack in reader:
                if pack.is_event_packet: 
                    ev = go.events.TelemetryEvent.from_telemetrypacket(pack)
                    if (n_events >= args.n_events) and (args.n_events > 0):
                        is_done = True
                        break 
                    n_events += 1 
                    #event_ids.append(ev.tof.event_id)
                    tof_analysis.add_event(ev.tof) 
                    if ev.tof.event_id == 1334:
                        print (ev) 
                else:
                    continue
        else:
            reader = go.io.CRReader(str(f)) 
            for frame in reader:
                # while we are on it, get the monitoring data 
                if frame.has(tof_hk_name):
                    tp   = frame.get_telemetrypacket(tof_hk_name)  
                    ts   = tp.header.gcutime 
                    tp   = go.packets.TofPacket.from_bytestream(tp.payload, 0) 
                    match tp.packet_type:
                        case go.packets.TofPacketType.MtbMoniData:
                            moni = go.monitoring.MtbMoniData.from_tofpacket(tp)
                            #moni.timestamp = int(ts)
                            mtb_moni.add(moni)
                            mtb_moni.add_timestamp(int(ts))
                        case go.packets.TofPacketType.EventBuilderHB:
                            moni = go.monitoring.EventBuilderHB.from_tofpacket(tp) 
                            evbhb.add(moni) 
                            evbhb.add_timestamp(int(ts))
                if args.moni_only:
                    continue
                if only_tracks:
                    telly_name = 'TelemetryPacketType.NoGapsTriggerEvent'
                else: 
                    telly_name = 'TelemetryPacketType.InterestingEvent'
                    if not frame.has(telly_name):
                        telly_name = 'TelemetryPacketType.BoringEvent' 
                
                if not frame.has(telly_name):
                    continue 
            
                ev = frame.get_telemetryevent(telly_name)


                if (n_events >= args.n_events) and (args.n_events > 0):
                    is_done = True
                    break 
                n_events += 1 
                #event_ids.append(ev.tof.event_id)
                tof_analysis.add_event(ev.tof) 
                if ev.tof.event_id == 1334:
                    print (ev) 
   
    if not args.from_telemetry:
        # mtb monitoring plots 
        plot_outdir = Path(plots_main_outdir) / 'monitoring' 
        plot_outdir.mkdir(exist_ok=True, parents=True)
        
        # -------- mtb moni 
        df = mtb_moni.get_dataframe()
        times = np.array(mtb_moni.timestamps)
        times = (times - times[0]) / 3600.0
        fig   = go.visual.style.gander_scatter_plot(times, df['rate'],\
                    figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT,
                    title='MTB rate',
                    xlabel='MET [h]',
                    ylabel='Hz', color='w')
        fig.savefig(plot_outdir / 'mtb_rate.png')
        # ---------- event builder heartbeat 
        df    = evbhb.get_dataframe()
        times = np.array(evbhb.timestamps)
        times = (times - times[0]) / 3600.0
        fig   = go.visual.style.gander_scatter_plot(times, 100*df['n_timed_out']/df['n_sent'],\
                    figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT,
                    title='Fraction of timed out events',
                    xlabel='MET [h]',
                    ylabel='\\%',
                    ylabel_rot=0,
                    color='w')
        fig.savefig(plot_outdir / 'mtb_timed_out_frac.png')
        fig   = go.visual.style.gander_scatter_plot(times, 100*df['data_mangled_ev']/df['n_sent'],\
                    figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT,
                    title='Fraction of events with data mangling',
                    xlabel='MET [h]',
                    ylabel='\\%',
                    ylabel_rot=0,
                    color='w')
        fig.savefig(plot_outdir / 'mtb_data_mangled_frac.png')
        #fig   = go.visual.style.gander_scatter_plot(times, 100*df['n_timed_out']/df['n_sent_trigger'],\
        #            figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT,
        #            title='Fraction of timed out events',
        #            xlabel='MET [h]',
        #            ylabel='%', color='w')
        #fig.savefig(plot_outdir / 'mtb_n_timed_out.png')
        ## FIXME - is the track trigger the combo trigger?
        #fig   = go.visual.style.gander_scatter_plot(times, 100*df['n_timed_out_combo']/df['n_sent_trigger_combo'],\
        #            figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT,
        #            title='Fraction of timed out events [combo]',
        #            xlabel='MET [h]',
        #            ylabel='%', color='w')
        #fig.savefig(plot_outdir / 'mtb_n_timed_out_combo.png')
    if args.moni_only:
        print ('-> Finisehd. As we ran witn --moni-only, no events have been analyzed.')
        sys.exit(0)

    print ('-> Creating histograms ..') 
    #print (tof_analysis.NHIT_BINS)
    tof_analysis.finish() 
    #print (tof_analysis.NHIT_BINS)
    print (tof_analysis.pretty_print_statistics())
    print ('-> Saving figures!')
    plot_outdir = Path(plots_main_outdir) / 'nhit' 
    plot_outdir.mkdir(exist_ok=True, parents=True)
    
    # create all the plots
    # ---------------------- nhit plots
    for k in ('TOF Hits'    , 'hit'),\
             ('TOF CBE Hits', 'nhit_cbe'),\
             ('TOF UMB Hits', 'nhit_umb'),\
             ('TOF COR hits', 'nhit_cor'): 
        fig = go.visual.style.gander_plot(tof_analysis.nhit_plots[k[1]], 'nhit', k[0], log=True)
        fig.savefig(plot_outdir / f'{k[1]}.png')

    for k in ('i_vs_o_nhit'    ,'inner'   ,'outer','OUTER TOF vs INNER TOF'),\
             ('umb_vs_cor_nhit','COR hits','UMB hits', 'UMB vs COR'),\
             ('cbe_vs_umb_nhit','CBE hits','UMB hits', 'UMB vs CBE'),\
             ('cbe_vs_cor_nhit','CBE hits','COR hits', 'COR vs CBE'):
        fig = go.visual.style.gander_2dplot(tof_analysis.nhit_plots[k[0]], k[1], k[2], k[3], show_nentries = True)
        fig.savefig(plot_outdir / f'{k[0]}.png')

    # --------------------------- The "usual" missing HG hits plot 
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
    plot_outdir = Path(plots_main_outdir) / 'edep' 
    if not plot_outdir.exists():
        plot_outdir.mkdir(exist_ok=True, parents=True)
    for k in ('EDEP UMB', 'edep_umb'), ('EDEP CBE', 'edep_cbe'), ('EDEP_COR', 'edep_cor'): 
        fig = go.visual.style.gander_plot(tof_analysis.edep_plots[k[1]], 'MeV', k[0])
        fig.savefig(plot_outdir / f'{k[1]}.png')
    for k in ('EDEP UMB vs COR', 'umb_vs_cor_edep'),\
             ('EDEP CBE vs COR', 'cbe_vs_cor_edep'),\
             ('EDEP CBE vs UMB', 'cbe_vs_umb_edep'): 
        fig = go.visual.style.gander_2dplot(tof_analysis.edep_plots[k[1]], 'MeV', 'MeV', k[0])
        fig.savefig(plot_outdir / f'{k[1]}.png')
    plot_outdir = Path(plots_main_outdir) / 'occupancy' 
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
    plot_outdir = Path(plots_main_outdir) / 'timing' 
    plot_outdir.mkdir(exist_ok=True, parents=True)

    for k in ('beta',fr'$\beta$',''), ('t_inner','ns','Time (inner TOF)'), ('t_outer','ns','Time (outer TOF)'),\
             ('t_diff', 'ns', 'Time delta (outer -inner)'), ('ph_delay', 'ns', fr'9th chan. phase $\phi$') , ('dist', 'mm', 'Reco dist.'),\
             ('cos_theta', fr'$\cos(\theta)$', ''), ('cos2_theta',fr'$\cos^2(\theta)$', ''), \
             ('x_outer', 'mm', 'First hit x outer'), ('y_outer', 'mm', 'First hit y outer'), ('z_outer', 'mm', 'First hit z outer'),\
             ('x_inner', 'mm', 'First hit x innner'), ('y_inner', 'mm', 'First hit y inner'),\
             ('z_inner', 'mm', 'First hit z inner'), ('pid_inner', 'PID', 'First PID inner'), ('pid_outer', 'PID', 'First PID outer'):
        if k[0] in ['beta'] :
            fig = go.visual.style.gander_plot(tof_analysis.tmg_plots[k[0]], k[1], k[2], log=True)
        else:
            fig = go.visual.style.gander_plot(tof_analysis.tmg_plots[k[0]], k[1], k[2], log=True)
        fig.savefig(plot_outdir / f'{k[0]}.png')

    for k in ('dist_vs_beta',fr'$\beta$','mm'), ('dist_vs_tdiff','mm','ns'), ('beta_vs_theta',fr'$\beta$',fr'$\theta$'):
        fig = go.visual.style.gander_2dplot(tof_analysis.tmg_plots[k[0]], k[1], k[2], '')
        fig.savefig(plot_outdir / f'{k[0]}.png')
    
    if args.paddle_plots: 
        for pid in tqdm.tqdm(range(1,161), desc='Creating paddle plots..'):
            plot_outdir = Path(plots_main_outdir) / 'paddles' / f'{pid}' 
            if not plot_outdir.exists():
                plot_outdir.mkdir(exist_ok=True, parents=True)
            for side,label_side in ('a','A'),('b','B'):
                for k in (f'amp_{side}','mV', f'Peak {label_side}'),\
                         (f'charge_{side}','pC', f'Charge {label_side}'),\
                         (f'time_{side}', 'ns', f'LE Time {label_side}'),\
                         (f'bl_{side}', 'mV', f'Baseline {label_side}'),\
                         (f'bl_{side}_rms', 'mV', f'Baseline RMS {label_side}'):
                    for log in True,False:
                        fig = go.visual.style.gander_plot(tof_analysis.paddle_plots[pid][k[0]], k[1], k[2], log=log)
                        if log:
                            fig.savefig(plot_outdir / f'{k[0]}-{pid}.png')
                        else:
                            fig.savefig(plot_outdir / f'{k[0]}-{pid}-log.png')

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

