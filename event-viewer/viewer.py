#! /usr/bin/env python

"""
Event viewer for events from the database
"""
import pylab as p
import numpy as np
import hepbasestack.layout as lo
from matplotlib import patches

from prepare_hits import prepare_hits_helper
from tracker_mapping import channel_coordinates

def plot_layers(hits):
    """

    Args:
        hits:

    Returns:

    """

    print(f'We found {len(hits)} hits!')
    n_layers = len(set([k.layer for k in hits]))
    all_layers = list(set([k.layer for k in hits]))
    print(f'We found the following layers {all_layers}')

    all_dets = []
    all_strips = []
    for r in range(6):
        for m in range(6):
            for c in range(32):
                all_dets.append( \
                    (channel_coordinates(r, m, 2, c, only_detectors=True), channel_coordinates(r, m, 2, c)) \
                    )
                # all_strips.append(channel_coordinates(r,m,2,c))
    # all_dets_x = [k[0] for k in all_dets]
    # all_dets_y = [k[1] for k in all_dets]
    fig,axs = p.subplots(3,3, figsize=(lo.FIGSIZE_A4_SQUARE[0]*3, lo.FIGSIZE_A4_SQUARE[1]*3))
    for i, layer in enumerate(all_layers):
        strip_coord = [channel_coordinates(k.row, k.mod, k.layer, k.channel) for k in hits if k.layer == layer]
        det_coord   = [channel_coordinates(k.row, k.mod, k.layer, k.channel, only_detectors=True) for k in hits if
                       k.layer == layer]
        adc = [k.adc for k in hits if k.layer == layer]
        adc = np.array(adc) / 12
        xs = [k[0] for k in strip_coord]
        ys = [k[1] for k in strip_coord]

        #figs.append(p.figure(figsize=lo.FIGSIZE_A4_SQUARE))
        #ax = figs[-1].gca()
        ax = axs.flat[layer]
        ax.scatter(xs, ys, marker='o', s=adc, facecolor='none', edgecolor='r')
        ax.set_title(f'Layer {layer}', loc='right')
        for k in det_coord:
            patch = patches.Circle(k, radius=50, fill=False, color='k')
            # im.set_clip_path(patch)
            ax.add_patch(patch)
        for k in all_dets:
            patch = patches.Circle(k[0], radius=50, fill=False, color='gray', alpha=0.1)
            # rect_patch = patches.Rectangle(k[1], width=1, height=100, color='gray', alpha=0.1)
            ax.add_patch(patch)
            # ax.add_patch(rect_patch)

        ax.spines['top'].set_visible(True)
        ax.spines['right'].set_visible(True)
        ax.grid(0)
        ax.set_aspect('equal')
    for i, ax in enumerate(axs.flat):
        if not i in all_layers:
            ax.axis('off')
    return fig

if __name__ == '__main__':

    import argparse
    parser = argparse.ArgumentParser(description="Show 2d views of tracker hits")
    parser.add_argument('dbfile', help="input database SQLITE file")
    parser.add_argument('--t0', type=int, default=0, help='Run start time')
    parser.add_argument('--t1', type=int, default=10**12, help='Run end time')
    parser.add_argument('--event-id', type=int, help='eventid')
    args = parser.parse_args()

    hits = prepare_hits_helper(args.dbfile, args.t0, args.t1)
    hits = hits[args.event_id]
    plot_layers(hits)
    p.show()
