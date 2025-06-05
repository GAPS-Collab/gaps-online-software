import gaps_online as go
import uproot as up
import numpy as np
import matplotlib
import matplotlib.pyplot as plt
import tqdm
from pathlib import Path
import argparse
import dashi as d

matplotlib.use('agg')

if __name__ == '__main__':

    parser = argparse.ArgumentParser(description='Paddle occupancy graphic plot from root files (reco or MC)')
    parser.add_argument('-dir', '--data_dir', type=str, default = '', help = 'A directory with .root files from MC or Reconstruction')
    parser.add_argument('-id','--data_id', type=str, default ='_', help='the data id will go at the beginning of output files. the default is "_"')
    args = parser.parse_args()

    #cb.set_style_present()
    d.visual()

    files = Path(f'{args.data_dir}').glob('*.root')
    files = [k for k in files]
    vid_hid_map = go.db.get_vid_hid_map()

    pid_hist = d.factory.hist1d(np.array([]), bins=np.arange(0.5, 160.5, 1))
    occu = {k : 0 for k in range(1,161)}
    for f in tqdm.tqdm(files, total=len(files), desc='Creating plot...'):
        f = up.open(f)
        event_pids = []
        vids = f.get('TreeRec').get('Rec').get('hitseries_').get('hitseries_.volume_id_').array()
        for ev in vids:
            pids = [vid_hid_map[k] for k in ev if k < 200000000]
            event_pids.extend(pids)
            for pid in pids:
                occu[pid] += 1
        pid_hist.fill(np.array(event_pids))

    #normalize the occupancy,
    max_occu = 0
    for k in occu:
        if occu[k] > max_occu:
            max_occu = occu[k]
    for k in occu:
        occu[k] = occu[k]/max_occu
        if occu[k] == 0:
            occu[k] = np.nan

    #plot paddle occupancy,
    #fig = plt.figure(figsize=lo.FIGSIZE_A4_LANDSCAPE_HALF_HEIGHT)
    fig = plt.figure()
    ax  = fig.gca()
    pid_hist.line(filled=True, alpha=0.4, color='tab:blue')
    #cb.visual.adjust_minor_ticks(ax)
    ax.set_ylim(bottom=0)
    fig.savefig(f'{args.data_id}_pid_hist_reco.png')

    cm = matplotlib.colormaps['gnuplot2']
    fig, ax = go.tof.visual.tof_projection_xy(occu, cmap=cm)
    fig.savefig(f'{args.data_id}_occu-xy.png')
    fig, ax = go.tof.visual.unroll_cbe_sides(paddle_occupancy=occu, cmap=cm)
    fig.savefig(f'{args.data_id}_occu-cbe.png')
    fig, ax = go.tof.visual.unroll_cor(paddle_occupancy=occu, cmap=cm)
    fig.savefig(f'{args.data_id}_occu-cor.png')
